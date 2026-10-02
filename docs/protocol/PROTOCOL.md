# Protocol

**What the chain does today, exactly.** Every statement here describes code in [`chain/`](../../chain/README.md)
that exists and was verified on 18 September 2026, except §12, assets, which was verified on 22 September 2026. Where the direction intends something the code does not
yet do, it says so under "Not yet". Decisions behind these rules are in [`../DECISIONS.md`](../DECISIONS.md).

This describes the **Substrate L1** built on the Polkadot SDK (ADR-013). It is the chain: it produces and
finalises blocks, its validator set is governed, and it reproduces the behaviour M1 established (M3.3).

> **The custom Rust chain in `framework/` was deleted at M3.5 on 20 September 2026, and this document
> never described it.** It had no finality and its transaction validation accepted forged signatures for
> small-order public keys, which is why nothing of it was carried forward. `HANDOFF.md` §2.0 records what
> went and what remains of it.

**Pinned release:** `polkadot-stable2606-1`, as `polkadot-sdk = "=2606.1.0"` (ADR-022, ADR-033).

---

## 1. Accounts and money

- An account is an `AccountId32`. Addresses are SS58 with **prefix 42** on development and test networks
  (ADR-024). The mainnet prefix is not decided and is a Public Release criterion.
- Account keys are **Sr25519** (ADR-023). Signatures are `MultiSignature`, so the runtime also accepts
  Ed25519 and ECDSA, as any standard Substrate runtime does (ADR-038).
- The currency is **CGT**, the Creator-God Token (ADR-045), with **18 decimals** (ADR-035). The atomic,
  indivisible unit is the **Spark**: `1 CGT = 10^18 Sparks`. Every amount in the runtime, in genesis and
  on the wire is an integer count of Sparks in `u128`.
- The **existential deposit is 100 CGT**, `10^20` Sparks (ADR-036). An account whose free balance falls
  below it is removed from state and its remaining balance is destroyed, which lowers total issuance.
- `pallet-balances` provides transfers, holds, freezes and reserves. Scaled arithmetic uses the SDK's
  helpers, never a hand-written `a * b / c` (AGENTS.md §5, ADR-035).

**No fee is charged.** There is no `pallet-transaction-payment` in the runtime, because a `WeightToFee` is
a fee decision and fee classes and burn shares are OPEN-4. **No CGT is created by any path**: there is no
issuance (OPEN-1), no treasury (OPEN-2), and no block reward. Producing blocks leaves total issuance
unchanged, which is pinned by a test.

## 2. Transactions

A transaction is a standard Substrate `UncheckedExtrinsic`. A signed one carries the account, the
signature and these transaction extensions, in this order:

| Extension | What it enforces |
| --- | --- |
| `CheckNonZeroSender` | the sender is not the all-zero account |
| `CheckSpecVersion` | the runtime version the transaction was signed for |
| `CheckTxVersion` | the transaction version it was signed for |
| `CheckGenesis` | **the genesis hash**, so a transaction signed for one network is not valid on another (D-009) |
| `CheckEra` | mortality |
| `CheckNonce` | the account's nonce, which is what makes a transaction happen once |
| `CheckWeight` | the block's weight and length limits |

- **Replaying a transaction is refused** as `Stale`. A transaction ahead of the account's nonce is not
  applied and is held as `Future`. Both are pinned by tests.
- **Signature verification is the SDK's.** Sr25519 refuses a forged signature for a small-order key;
  Ed25519 follows ZIP-215 through `ed25519-zebra`, which accepts some signatures a stricter rule would
  reject. That is deliberate, because consensus needs every node to agree on validity, and it is recorded
  with its measurements in ADR-038.

## 3. State and the state root

State is the standard FRAME trie. `frame-executive` computes the state root at the end of every block and
seals it into the header. **An importing node recomputes it and refuses the block if it differs**, which
is what stops a block claiming a state nobody can reach. Pinned by a test.

## 4. Blocks

- **Block production is Aura** (ADR-019). Each block carries a pre-runtime digest naming its slot, and a
  node authors only in the slots its key is entitled to.
- **Target block time is six seconds** (`MILLISECS_PER_BLOCK`). It is a configuration type, so no runtime
  logic depends on a fixed slot time (ADR-018's parachain condition).
- **A block may use at most two seconds of compute**, with 75% of that available to normal extrinsics, and
  at most 5 MB of length.
- Every block carries the **timestamp inherent**, which Aura checks against the slot.

## 5. Finality

**GRANDPA** (ADR-018). This is the chain's first real finality: the custom chain had none and reported
every transaction as finalized regardless.

- Finality needs more than two thirds of the voters. With two validators and one down, the chain keeps
  producing blocks and **stops finalising** until the other returns. That is safety, not a fault.
- Verified on 18 September 2026 with two validators: they agree on the block at a common height, GRANDPA
  finalises, and both agree on what is finalised.

## 6. Who validates

- **The validator set is chosen by governance**, through `pallet-session` with `pallet-validator-set` as
  its session manager (ADR-020). It is not derived from stake: nominated proof of stake comes later, by
  the path in ADR-020, because staking before CGT is distributed would secure the chain with tokens only
  the owner holds.
- A validator is identified by its **account**, so a later parachain move swaps the session manager
  without changing identity.
- **No governance action can stop the chain.** The set may never fall below `MinValidators`, which is one
  on a development chain, and `new_session` never hands `pallet-session` an empty set: if storage somehow
  held too few it keeps the set already in use and emits `PreviousSetKept`. Three guards, each verified by
  deleting it.
- Misbehaviour is handled by governance removing the validator. Nothing is bonded, so there is nothing to
  slash.

## 7. Importing a block

- An importing node executes the block and refuses it if the **state root** or the **extrinsics root** does
  not match what the block produces.
- **The parent is checked by the node, not the runtime.** `execute_block` initialises before it checks, and
  initialising writes `BlockHash(n-1)` from the header, so the runtime's parent assertion compares the
  value it just wrote. The import queue resolves a block's parent in its database first, and a block whose
  parent is unknown is never executed. Recorded where it was found, with the test that pins it.
- A validator killed and restarted **catches up** and agrees about the blocks produced while it was away.

## 8. Networking and storage

Standard `sc-network` and the SDK's database. A **validator will not generate its own network key**: it
needs `--node-key` or an existing key file, or it exits with `NetworkKeyNotFound`.

**A node with no peers stops authoring**, which is Substrate refusing to build a one-sided chain while it
might be partitioned. `--force-authoring` overrides that and is how a single-node development chain runs.

## 9. Genesis

- **Development and test specifications only.** There is no mainnet chain specification and there will not
  be one until OPEN-2, the genesis allocation split, is decided.
- `dev` is a single validator, Alice; `local` is Alice and Bob. Their names and identifiers are
  `Demiurge Development` / `demiurge_dev` and `Demiurge Local Testnet` / `demiurge_local`, so a person can
  always ask a node which chain it is with `system_chain`.
- The chain's properties — `CGT`, 18 decimals, prefix 42 — come from the runtime's own constants, so the
  node cannot disagree with the runtime about the unit.
- Endowed accounts are the SDK's **well-known public test keys**, whose secrets everyone has, with a
  placeholder endowment. This is **not** a genesis allocation of the base supply, and a test asserts the
  amount stays a millionfold below it.
- `pallet-sudo` is present on development and test networks only, behind the `sudo` build feature, and is
  **absent from any mainnet runtime** (ADR-037).

## 10. Running a node

```bash
cd chain
cargo build -p demiurge-node --release --features sudo
./target/release/demiurge-node --dev --tmp
```

Two validators, agreement, finality and catch-up: `node scripts/check-two-validators.mjs`.

To give a fresh account CGT on a development chain, send it from Alice's development endowment:
`cd scripts && npm install && node dev-fund.mjs <address> [amount]`. It is a transfer, it creates nothing, and it refuses
any node whose `system_chainType` is not `Development`.

## 11. RPC

Standard Substrate RPC. There is no custom RPC surface: no method writes state outside a block, and there
is no faucet, no admin mint and no privileged endpoint. What the chain does, it does through signed
transactions in blocks (D-008).

## 12. Assets (DRC-369: M4.1, M4.2's royalties and nesting, and R-2)

`pallet-nfts` is the ownership ledger (ADR-025), mounted as `Nfts` at index 7. `pallet-drc369`, mounted as `Drc369` at
index 8, makes an item a DRC-369 asset (ADR-047, ADR-052). `pallet-utility`, mounted as `Utility` at index 9, puts
several calls in one transaction (ADR-053). `pallet-drc369-royalties`, mounted as `Drc369Royalties` at index 10, holds
royalty terms and settles sales in CGT (ADR-061). `spec_version` is 6 since 2 October 2026, when
`Drc369Royalties::buy_exact` and the runtime API `Drc369RoyaltiesApi` were added (5 on 1 October, for `Drc369::nest`
and `unnest`); `transaction_version` is 2, since 29 September 2026, when `Drc369::mint` gained its
`derived_from` argument. No existing call's encoding changed on 1 or 2 October.

- **An asset is `(collection, item)`**, two `u32`s. Each creator has one **singles collection**, created by their first
  mint; they own it, pay its deposit and administer it.
- **`Drc369::mint(content, commit, name, revisable, derived_from)`** is signed, and mints to the signer, in the
  signer's singles collection. There is no recipient and no collection parameter, so a mint cannot land in anyone
  else's account (requirement 7).
- **`derived_from`** optionally names the asset this one remixes, as `(collection, item)`. Anyone may name any
  DRC-369 asset — it obliges the remix, not its source — and it is never written again. The record's `remix_depth` is
  `0` for an original and the source's depth plus one for a remix, and a mint deeper than **16** is refused
  (`RemixTooDeep`), as is a source that is not a DRC-369 asset (`UnknownSource`).
- **`content` is 41 bytes** (ADR-047 decision 3): a one-byte algorithm tag (`0` BLAKE3-256, `1` SHA-256, `2`
  BLAKE2-256), a 32-byte root, and the manifest's size as a little-endian `u64`. The root is the BLAKE3-256 hash of the
  asset's manifest, which is off chain; the chain stores it and never computes it. A tag that is none of the three does
  not decode, so the transaction is invalid. Of the three, **only BLAKE3-256 is accepted**, and a size of zero is
  refused.
- **What an asset may be, and what the chain does not hold.** The manifest fingerprints arbitrary bytes, so an
  asset can be an archive, a video, a song, its stems, an image, a document, a font or a 3D model; the launcher
  labels each file with a media type from a fixed table and records anything else as `application/octet-stream`.
  **The chain holds none of it.** It holds the 41-byte reference, the commit, the name and the ownership record —
  not the files, and not even the manifest that lists them. It therefore proves *which* bytes an asset is and who
  has owned it since which block, and it cannot produce them: today the bytes are in the minting machine's own
  content store, and serving them to anyone else is the Mesh (M8). A transfer moves the record and the proof, not
  a payload.
- **`commit`** is the source commit, tagged: `0` SHA-1 (20 bytes) or `1` SHA-256 (32 bytes), a git object id, never a
  branch. It is optional, for a mint that did not come from a repository.
- **`name`**, up to 256 bytes, becomes the item's `pallet-nfts` metadata. It is set once.
- **`Drc369::revise(collection, item, content, commit)`**: the current owner only, while the asset is revisable. It
  replaces the current reference and commit; the reference the asset was minted with (`origin`) never changes. A
  revision to the same reference and commit is refused.
- **`Drc369::make_permanent(collection, item)`**: the current owner only. One-way: no call makes an asset revisable
  again, and `revise` is refused from then on, for every later owner.
- **Nesting** (M4.2, and requirement R-2 of M4.5). **`Drc369::nest(collection, item, parent)`** places the asset
  `(collection, item)` inside the asset `parent`, a `(collection, item)` pair. **`Drc369::unnest(collection, item)`**
  takes it out again. Both are signed. What `nest` refuses:
  - a signer who does not hold **both** assets (`NotOwner`) — the child alone is not enough;
  - either end not being a DRC-369 asset (`UnknownAsset`, `UnknownParent`);
  - an asset already inside another (`AlreadyNested`);
  - **a cycle** (`NestingCycle`): the parent is the asset itself, or is nested anywhere inside it. The chain walks
    from the parent to its root to find out, at most **8** steps;
  - a depth past **8** (`NestedTooDeep`): a free-standing asset is at depth 0, an asset inside it at 1;
  - an asset that itself holds assets (`HoldsAssets`): a tree is built from the root down, which keeps the depth
    bound exact without measuring a subtree;
  - a parent already holding **64** assets (`TooManyChildren`).

  `unnest` needs the asset to be nested (`NotNested`) and the signer to hold it. It may take out an asset that still
  holds others; they stay inside it.
- **A nested asset stays where it is.** While an asset is nested, **or holds a nested asset**, `pallet-nfts` refuses
  to transfer or burn it, with its own `ItemLocked`: `pallet-drc369` is the ledger's `Locker` (ADR-025). That holds
  for `Nfts::transfer` by the owner or by an approved account, for the same call inside `Utility::batch_all`, and for
  `Drc369Royalties::buy`, which ends in a transfer and so fails whole, moving no CGT. A listing may exist on such an
  asset; it cannot be bought until the asset is taken out. To move a tree, take it apart, move the assets and nest
  them again; one `batch_all` can carry all of that. Nesting changes neither an asset's owner nor its record, takes
  no deposit and charges nothing.
- **Events:** `Drc369::SinglesCollectionCreated`, `Minted` (with `origin`, `commit`, `revisable`, `derived_from`), `Revised` (with
  `from`, `to`, `commit`), `Locked` (made permanent, with the reference it fixes), `Nested { parent, child, by, depth }`
  and `Unnested { parent, child, by }`, besides `pallet-nfts`' own
  `Created`, `Issued`, `ItemMetadataSet` and `Transferred`.
- **What a client reads:** `Nfts::Account`, keyed by owner, then collection, then item, is the owner index;
  `Drc369::Assets` holds each asset's `origin`, `current`, `commit`, `revisable`, `derived_from` and `remix_depth`;
  `Drc369::RemixCount` how many remixes name it; `Drc369::ParentOf` the asset it is nested inside, if any, and
  `Drc369::ChildCount` how many it holds (which ones is in the `Nested` and `Unnested` events; the chain keeps no
  list); `Drc369Royalties::RoyaltyTerms` and `Drc369Royalties::Listings` hold its terms and listing; `Nfts::ItemMetadataOf` holds the
  name. The runtime API `Drc369Api` answers `assets_of(owner)` and `asset(collection, item)` from the same state.
- **One way in.** Of `pallet-nfts`'s thirty-nine calls, the base call filter lets through only `transfer`,
  `approve_transfer`, `cancel_approval` and `clear_all_transfer_approvals`; every other one is refused as
  `CallFiltered`. `pallet-drc369`'s `mint` is the only way an item is made, and `pallet-drc369-royalties`'s `buy` the
  only sale: `pallet-nfts`'s own `set_price` and `buy_item` pay no royalty and are refused.
- **Several assets in one transaction** (M4.6, ADR-053). Of `pallet-utility`'s calls the filter lets through only
  `batch_all`, the atomic one: any inner call failing reverts the whole transaction, so a trade of several assets
  either happens entirely or not at all. **A batch is not a second way in**: for a signed origin each inner call is
  dispatched through this same filter, so a batch carrying `Nfts::mint` is refused exactly as a bare one is. Only a
  root origin bypasses filters, and root exists on development and test networks only (ADR-037). A message may ride
  in the same batch as a `System::remark_with_event`, which is public and permanent.
- **Deposits are held, not spent, and are placeholders (U-14):** 500 CGT for a singles collection, 400 CGT per asset,
  and 100 CGT plus 0.625 CGT a byte for its name, reserved from the minter. They are derived from the existential
  deposit by ADR-030's arithmetic and are not decided values.
- **Royalty terms** (ADR-061, ADR-062). **`Drc369Royalties::set_terms(collection, item, recipients, remix)`**: only the
  asset's creator — the owner of its collection — and only while they hold it; they may change the terms as often as
  they like while they do, and never while anyone else holds it. Once any remix names the asset
  (`Drc369::RemixCount` above zero), its remix share may be lowered but not raised (`RemixShareLocked`). Up to **eight**
  recipients (ADR-057), each with a `Permill` share of every sale, no account twice, no share of zero, the shares
  summing to at most one whole. `remix` is the share of every sale of a remix of this asset owed to these recipients;
  it needs at least one recipient.
- **A sale settled in CGT.** **`list(collection, item, price)`**: the holder offers the asset at a price in Sparks
  above zero, or changes the price. **`unlist(collection, item)`**: the seller or the holder; or anyone, once the
  seller no longer holds the asset and the listing is void. **`buy(collection, item, max_price)`**: refused if the
  listing is void, the buyer is the seller, or the price is above `max_price`. It pays, from the buyer, in one
  transaction:
  1. if the asset was derived from a source with terms, the source's remix share of the price, rounded down, divided
     between the source's recipients in proportion to their shares, each rounded down — **one level only**: the
     source's own source is paid nothing from this sale;
  2. each of the asset's own recipients their share of what step 1 left, rounded down;
  3. the seller everything else.

  The parts sum to exactly the price. It then transfers the asset to the buyer and removes the listing. If any
  payment cannot be made — the buyer cannot pay it, or it would leave a recipient without an account below the
  existential deposit (`PaymentCannotBeReceived`) — nothing moves. No platform share and no fee are taken.
- **Buying the work that was looked at** (`spec_version` 6). **`buy_exact(collection, item, max_price, content)`**
  is `buy` with one more condition: `content` is the content reference the buyer expects, and if the asset's
  `current` reference is any other — its holder revised it after the buyer looked — the sale is refused whole with
  `ContentChanged` and nothing moves. `max_price` already held the price; this holds the work. `buy` keeps its call
  index and encoding, and so `transaction_version` stays 2; a client that sends `buy` still works and is not
  protected. The launcher sends `buy_exact`.
- **Asking what a sale would pay.** The runtime API **`Drc369RoyaltiesApi::sale_preview(collection, item, price,
  buyer)`** answers, for any asset someone holds, listed or not: its source, each remix and royalty payment, the
  holder and what the holder would receive (the parts sum to `price`), and `refusal`, the error the sale would fail
  with, if any. The parts come from the same function `buy` uses. **With a buyer**, the whole settlement is run and
  rolled back, so `refusal` is what that buyer's transaction would get. **Without one**, only what would stop every
  buyer is checked: a zero price, a part its recipient cannot receive, and an asset held in place by nesting
  (`ItemLocked`). `None` if it is not a DRC-369 asset. It moves nothing. **No client uses it yet**: the launcher
  still computes its preview with its own copy of the split.
- **Royalty events:** `TermsSet`, `Listed`, `Unlisted` and `Sold { collection, item, from, to, price, source, remix,
  royalties, seller_received }`, where `remix` and `royalties` list each payment as `(account, amount)`.
- **Weights are placeholders** (ADR-052 decision 8, ADR-061), owed to M7.2. No fee is charged anyway.

## 13. Not yet

Each of these is a roadmap item, not an omission from this description.

- **Fees, issuance, burn and a treasury.** Nothing is charged or paid in CGT (OPEN-1 to OPEN-4, M6).
- **The rest of DRC-369**: state and XP, physics, rental, fractional ownership and
  burning (M4.2 to M4.5); and sponsorship and agent caps (M4.4, M5). Their pallets are named (ADR-032) and not
  written. Remix provenance and royalties exist since 29 September 2026, and nesting with cycles refused (R-2) since
  1 October 2026; remix *rights* do not. Moving a parent together with what it holds does not exist either: a tree
  is held in place.
- **Governance that can execute.** ADR-021's collective and then OpenGov. Until one exists, the governance
  origin is root, which on a development network is the sudo key and on a mainnet runtime has no caller at
  all.
- **Nominated proof of stake** (ADR-020), with the preconditions that record names.
- **A mainnet chain specification, prefix and genesis hash.**
