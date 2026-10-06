# Migration inventory: the custom chain onto the Polkadot SDK

**Status:** Written 14 September 2026 under [ADR-013](../decisions/ADR-013-polkadot-sdk-migration.md). **Eighteen of
its twenty questions are decided** (§8, Resolution status): Q-16 by ADR-017 and Q-1 to Q-15 by ADR-018 to ADR-032, all on
15 September 2026; **Q-17 was added and decided on 20 September 2026** (ADR-041); **Q-18, Q-19 and Q-20 were added on 22
September 2026; Q-18 was decided on 28 September 2026 (ADR-057, eight stays), and Q-19 and Q-20 are open**, and the format cannot be frozen until both are answered. Every claim was re-checked against the pinned
release, `polkadot-stable2606-1`, and corrected where needed (ADR-022). **The owner confirmed on 17 September 2026 that
ADR-018 to ADR-032 read correctly**, which lifted the hold this line used to carry. **The owner reviewed this document on
22 September 2026, through a ten-line summary of its DRC-369 section, and roadmap item M2.1 was ticked against that
summary** — the summary is kept below, as reviewed, so the tick can be checked against what was read. That lifted the
hold AGENTS.md §7 places on migration code. Where a recommendation is given it is marked as one.
**Scope:** Every module and base-layer crate in `framework/`, plus the clients and services that talked
to that chain. For each: what it was, what it maps to, and one disposition. **`framework/` was retired and deleted at
M3.5 on 20 September 2026**, so the left-hand side of every mapping below now describes code that is gone; this document
is the record of what it contained and where each part went, which is why it outlived it. Its line numbers refer to the
tree as it stood before the deletion.

### The owner's review (M2.1)

Pasted at the top of the report of 22 September 2026 and acknowledged by the owner that day. Kept verbatim; it
summarises §3.5, §5 and §9 as they then stood, and it is not a substitute for them.

1. The old chain's DRC-369 was one module, deleted with `framework/` on 20 September. On chain it could mint, transfer, approve, burn, set state and XP, nest assets, and name one royalty recipient. None of it ever moved any currency.
2. Everything else — royalties to several people, remix royalties, rental, fractional ownership, physics — was library code that nothing called.
3. Under ADR-025 it maps to the standard `pallet-nfts` for ownership (collections, items, approvals, transfer locks, deposits), with custom pallets for what makes DRC-369 different.
4. Written new: the content fingerprint, a list of what each owner holds that can be queried, state and XP, nesting, royalties including remix, rental, and physics in fixed-point numbers.
5. Fractional ownership uses the standard `pallet-nft-fractionalization`. `pallet-assets` is present for that alone, not for fungible game items (ADR-031).
6. What carries is behaviour, not code: about a dozen of its tests become acceptance tests (M4.5). No code carries, and neither does the old token ID. Nothing was ever published on it, so nothing outside breaks.
7. Two defects fixed in the old chain on 14 September carry over as requirements: a mint must be authorised, and nesting requires owning the parent.
8. R-2: nesting must refuse cycles, meaning an asset inside itself or A inside B inside A, within a bounded depth, because a cycle makes every asset in it impossible to burn. It was never fixed in the old chain and lands with the new pallet at M4.5.
9. What the Substrate framework forces: every field has a size limit, file bytes live off chain with only a fingerprint on chain, history comes from events read by an indexer, and physics can't use floating point. Royalties can only be enforced on sales the chain settles, and deposits and fees stand in the way of minting without holding CGT (M4.4).
10. ADR-047, which you accepted today, answers the format questions this section raised. Your "acknowledged" here ticks M2.1, the last block before M4.

**Carried forward by the owner in the same acknowledgement:** the eight-recipient royalty bound, as Q-18 (§8), because
a song's credits can pass eight (`docs/blueprints/gnosis.md`, "What GNOSIS needs from the asset format").

---

## 1. How to read this

### Dispositions

| Disposition | Meaning |
| --- | --- |
| **Standard** | A pallet or client component from the Polkadot SDK, used as shipped and only configured. This is the stay-boring layer (ADR-001, ADR-013 rule 1). |
| **Custom** | A pallet written for Demiurge, in the creative layer: asset semantics, royalties, agent rails, economics, the Mesh (ADR-013 rule 2). |
| **Open** | Cannot be classified until the owner decides something. The options are listed, and a recommendation is given where there is one. |
| **Not carried** | Nothing in the module is migrated, for the reason given. This fourth category exists because several modules are stubs, are unreachable, or were already ruled out by an earlier decision. |

### Evidence labels

- **Verified (repo):** read in this repository's code on 13 or 14 September 2026, with `file:line`.
- **Verified (SDK):** read in the Polkadot SDK source on 14 September 2026. The branch was `master`;
  the latest stable release listed was `polkadot-stable2606-1`. Links are in §10.
- **Inferred:** reasoned from the code or from general Substrate practice, and not checked against
  the release that will be pinned. Every inferred claim is re-checked when that release is chosen.
- **Re-check against `polkadot-stable2606-1`** (15 September 2026, ADR-022):
  - **Inferred claims about the SDK (9):** one partly wrong, F-D2, now corrected. Three are judgements that source
    cannot settle. Five are confirmed.
  - **Verified (SDK) claims (22):** none wrong. Two were imprecise, F-D1's size limit and F-Q3's "enum", now corrected.
    Twenty are confirmed at the tag.

### Naming

Pallet names in this document were **placeholders**. The owner named the chain's crates and pallets in ADR-032, and
resolved the conflict with `.cursorrules` there: Gnostic naming covers internals only, and anything in runtime metadata
uses plain, greppable names. The fifth pallet, the session manager of ADR-020, is `pallet-validator-set`, approved on
17 September 2026.

---

## 2. Summary

The custom runtime has fewer live modules than its workspace suggests. Four modules are dispatched by
transactions:

- **Balances**, **Energy** and **SessionKeys** (`framework/core/src/runtime.rs:395-486`);
- **DRC-369** (`runtime.rs:487-493`).

Every other module name falls through to a `ModuleRegistry` that is created empty and never filled
(`runtime.rs:158, 494-498`). Governance, qor-identity, agentic, zk and yield-nfts are referenced by
nothing. game-assets is a no-op. game-registry is excluded from the build. CVP is disabled in the
node. None of the validator, staking, slashing or era logic in `framework/consensus` ever runs, because
the node never calls `finalize_block` (`framework/node/src/chain.rs`). Validator stake exists only in
memory, filled from genesis. All of this is verified (repo).

What that means for the migration:

- **The stay-boring layer is almost entirely replaced by standard components.** Consensus, finality,
  networking, storage, the transaction format, nonces, fees, balances, treasury and runtime upgrades
  all have standard equivalents. Very little of the custom code there needs to survive, other than
  the behaviour its tests pin down.
- **The creative layer is mostly still to be built.** DRC-369 is the only substantial custom module,
  and of its claimed capabilities only ownership, state/XP, nesting and a single royalty setting are
  reachable on chain today. None of them moves CGT (§5).
- **Three defects in the custom runtime are not carried.** They are listed so that their tests become
  acceptance tests on the new chain:
  1. A self-transfer through `BalanceCall::Transfer` credits the sender and creates CGT. Both balances
     are read before either is written (`framework/modules/balances/src/balances.rs:85-104`), and the
     runtime only checks `from == caller` (`runtime.rs:404-408`). Verified (repo).
  2. DRC-369 mint writes any `owner` the caller names, with no authorisation
     (`framework/modules/drc369/src/nft.rs:389-444`). Verified (repo).
  3. DRC-369 nest checked ownership of the child but not the parent, so a stranger could block an
     owner's burn by nesting under it (`nft.rs:745-785`). Verified (repo).

  Items 1 to 3 were fixed in the custom chain on 14 September 2026, with tests, and remain
  requirements for the new chain. Two further requirements were recorded that day without patching the
  custom chain: strict signature verification (R-1, §3.1) and refusing nesting cycles (R-2, §3.5). A
  third, R-3 (§3.2), was recorded the same day: no service outside the chain can cause CGT to be
  created. It compounds R-1. A fourth, R-4 (§3.1), was recorded on 17 September: the new chain is
  written with one ticker throughout, in names as well as in what it displays. That ticker is `CGT`
  (ADR-045).

---

## 3. Runtime modules

### 3.1 System, transactions and money

**Requirement R-4: the chain uses `CGT`, in names as well as on screen.** One symbol, written the same way in pallet
constants, call and event names, storage items, RPC method names, the runtime's `system_properties` symbol, and the
client and SDK surfaces that read them — greppable by design. `chain/` was written this way from the start. Naming a
new pallet still needs the owner's approval (ADR-032).

> **History.** The ticker was `CGT` from 17 to 21 September 2026 (ADR-034), and this requirement was first written
> in that name. [ADR-045](../decisions/ADR-045-the-ticker-returns-to-cgt.md) returned it to `CGT` in the owner's name,
> and the owner has since confirmed it is not changing again. The check named in the table below would flag any
> identifier reading `dmrg`; it still does not exist.


| Today | Maps to | Disposition | Notes |
| --- | --- | --- | --- |
| **Transaction nonces** at `System:TxNonce:{account}`, checked and incremented per transaction (`runtime.rs:52, 164-169, 319-334`) | `frame-system` account nonce with the `CheckNonce` extension | **Standard** | D-004 carries as a requirement. Standard `CheckNonce` rejects senders with no providers and no sufficients (Verified SDK); see F-Q1. |
| **Block number and timestamp**, not in the state root (`runtime.rs:55-58, 205-206`) | `frame-system` block number; `pallet-timestamp` | **Standard** | |
| **Transaction format and signature**: `Transaction{nonce, from, signature, data}`, signed over `SCALE(nonce) ‖ from ‖ SCALE(data)` with no chain identifier (`framework/core/src/transaction.rs:12-35, 70-76`) | `sp-runtime` signed extrinsic with `MultiSignature` and the standard transaction extensions | **Standard** | `CheckGenesis` puts the genesis hash in the signed payload (Verified SDK), which meets D-009. Every client that signs today is replaced (§4). **Requirement R-1: signature verification is strict.** The custom chain verifies with non-strict Ed25519 `verify` (`transaction.rs:58`). On 14 September 2026, `ed25519-dalek` 2.2.0 `verify` accepted a signature nobody made (R = identity, s = 0) for the identity-point key on 64 of 64 messages and for the all-zero key on 13 of 64; `verify_strict` accepted none (Verified). The Polkadot SDK chain must reject small-order public keys and non-canonical signatures in transaction validation, confirmed against the pinned release (Q-5). **Measured on 2026-09-18, and the result is split: Sr25519 refuses the forgery on 64 of 64 messages for both the identity-point and all-zero keys; Ed25519 accepts it on 64 of 64.** The cause is not the custom chain's accident. `sp-core` verifies Ed25519 with `ed25519-zebra`, which implements ZIP-215, a precisely specified rule chosen so every node agrees on validity; consensus needs that agreement more than strictness, and ZIP-215 deliberately accepts signatures `ed25519-dalek`'s strict mode rejects. Changing it means departing from the SDK's standard verification, which ADR-013 does not allow without a written reason. **What it means:** an account whose address is a small-order Ed25519 point can be spent from by anyone, but nobody holds a secret for such an address, so nothing is taken from a person. ADR-023 chose Sr25519 for account keys, so the chain's own scheme is not exposed. **Closed on 18 September 2026 as met by standard behaviour (ADR-038).** The owner refused restricting the runtime to Sr25519 signatures only: it would own a consensus-critical divergence permanently, in exchange for closing a hole that harms nobody, since no secret key produces a small-order address and so no person holds one. ADR-001's innovation budget names consensus and key handling as exactly where not to be clever. Pinned by tests, including one that fails if the SDK ever becomes stricter, at which point ADR-038 is revisited. Not patched in the custom chain, by the owner's instruction. |
| **Atomic execution**: call writes go to a nested overlay, kept on success and dropped on failure, with the nonce and cost kept (`runtime.rs:328-349`) | FRAME transactional dispatch | **Standard** | D-005 carries as a requirement. That FRAME gives every extrinsic its own storage layer by default is Inferred. |
| **Receipts** returned to the producer only, never stored (`runtime.rs:108-113, 354-365`) | `frame-system` events (`ExtrinsicSuccess`, `ExtrinsicFailed`, per-pallet events) | **Standard** | Fixes the DRC-369 mint that throws away the new token id (`nft.rs:1087-1090`). |
| **Balances**: `Balances:Account`, `Balances:TotalSupply`, and calls Transfer, Mint and Burn (`framework/modules/balances/src/balances.rs:73-247`) | `pallet-balances` | **Standard** | The self-transfer defect (§2) is not carried. The 13 billion cap check is not carried (ADR-003). `TotalSupply` becomes the standard total issuance. A sponsor pays the existential deposit and the user holds it (ADR-029). Its value is derived from a stated target, with 100 CGT proposed (ADR-030). |
| **Energy**: regenerating per-account allowance charged for every transaction, plus a `Sponsor` call that ignores its user (`framework/modules/energy/src/energy.rs:84-155, 243-252`) | A custom pallet, plus a custom transaction extension wrapping `pallet-transaction-payment` | **Custom**, design proposed for review (U-4, [`SPONSORSHIP.md`](SPONSORSHIP.md)) | The D-001 numbers are withdrawn (ADR-004); the sponsorship principle stays. A new key receives up to a full allowance on first use, so today's model is not carried as-is (`energy.rs:115-130`). Standard `SkipCheckIfFeeless` makes chosen calls feeless (Verified SDK), but that does not make a sender exist (F-Q1). |
| **CGT fees, burn and split**: none charged today (`runtime.rs:332-333` charges energy only) | `pallet-transaction-payment`, with a custom `OnChargeTransaction` handler for burn share per fee class | **Standard** mechanism, custom handler | The burn shares and fee classes are OPEN-4 and are not invented here. |
| **Treasury account** `demiurge:treasury:v1` (`framework/primitives/src/accounts.rs:28`) | `pallet-treasury` | **Standard** | Spending is governance-executed (see §3.3). |
| **Burn account** `demiurge:burn:v1` (`accounts.rs:35`) | Burning reduces total issuance directly in `pallet-balances` | **Not carried** | No account is needed to burn. |
| **Denomination constants**: 18 decimals, `TOTAL_SUPPLY = 13e9 CGT`, existential deposit, validator and nomination minimums, starter grant (`framework/primitives/src/denomination.rs:45-69`) | Runtime constants and genesis configuration | **Standard** (constants) | `TOTAL_SUPPLY` is not carried (ADR-003). Precision is U-1. The minimum validator stake is not enforced today: genesis defaults it to 1 Spark (`framework/node/src/config.rs:142`). |
| **Signature scheme abstraction** with optional Dilithium3 and hybrid keys (`framework/primitives/src/signature.rs:157-398`) | `MultiSignature` (Ed25519, Sr25519, ECDSA) | **Standard** for Ed25519. Post-quantum is **Not carried** | Nothing on chain uses the post-quantum path. |

### 3.2 Genesis, issuance and the economic mechanisms

| Today | Maps to | Disposition | Notes |
| --- | --- | --- | --- |
| **Genesis file**: JSON with chain id, validators, balances and parameters; hash over a canonical SCALE form; balances must not exceed 13 billion (`config.rs:92-176, 251-326`; `framework/node/src/genesis.rs:27-64`) | Substrate chain specification with per-pallet genesis configuration | **Standard** | The genesis hash, network id and data-directory binding all change. Nothing has been published on the custom genesis, so nothing external breaks. That no public state needs migrating is Inferred, because production is offline. |
| **Genesis allocation into named pools**: absent today | Custom pallet holding the pools and their release rules | **Custom** | The split is OPEN-2 and not invented here. The release mechanism must exist before the first public testnet (OPEN-3). |
| **Decay-curve release and perpetual issuance**: absent. The only mints are dev-only RPC methods (`framework/rpc/src/methods.rs:638, 741`) | Same custom pallet, paying validators and seeders per era | **Custom** | Rate and curve are OPEN-1 and OPEN-3. Issuance and burn are implemented as one mechanism (ADR-004). |
| **Team and ecosystem release**: absent; earlier terms withdrawn | `pallet-vesting` if the terms are a linear schedule; otherwise the custom release pallet | **Open** (U-12) | |
| **Faucet, admin mint, starter grant**: dev-only RPC writes outside blocks (`methods.rs:547, 638, 741`) | `pallet-sudo` and chain-specification endowments on development and test networks only | **Standard** (dev and test only) | Whether starter grants exist on a public network at all is **Open**. No mint path exists outside development and test networks. **Requirement R-3: no service outside the chain can cause CGT to be created.** Starter CGT distribution, if Demiurge wants it at all, is a chain-side decision with its own on-chain authorisation, decided and recorded like any other issuance. QOR ID, and any other identity or application service, never holds a key, an origin or an endpoint that mints or grants CGT, and nothing it sends can trigger one. Found on 14 September 2026: `services/qor-auth` registration called `balances_claimStarter` on whatever node `BLOCKCHAIN_RPC_URL` named, with no check that it was a development chain. It created nothing only because the custom node refuses the claim outside `--dev --faucet`, requires the claimant's own signature, and was sent one parameter where three are required (Verified). The call was removed from QOR ID the same day. **R-3 compounds R-1.** A mint path authorised by a signature is only as strong as signature verification. The custom node's RPC authorisation, which the claim uses, verifies with non-strict `verify` too (`framework/rpc/src/auth.rs:152`), so on a node with the faucet on, a claim for a small-order key could be authorised by a signature nobody made (Inferred: the R-1 test exercised the same function; the RPC path was not tested). R-1 alone lets a forger move funds that already sit under such a key; a mint path alone is bounded by the claimant's own key and the claim record. Together, they create supply for keys nobody holds, from nothing. On the new chain, R-1 is met before any path that creates CGT is enabled on any network. |

### 3.3 Validators, staking and governance

| Today | Maps to | Disposition | Notes |
| --- | --- | --- | --- |
| **Validator set**: fixed at genesis, held in memory, stake-weighted proposer (`framework/consensus/src/engine.rs:294-346`; `genesis.rs:71-87`) | `pallet-session`, plus a custom session manager | **Standard**, with a **Custom** session manager, `pallet-validator-set` | A permissioned set chosen by governance now, nominated proof of stake later (ADR-020). The pinned release has no pallet for a governance-chosen set on a standalone chain. The name was approved on 17 September 2026 (ADR-032). |
| **Staking, era rewards, slashing, commission**: implemented in `framework/consensus` but never called by the node; rewards credit in-memory stake by 42 Sparks (`engine.rs:484-558, 1156-1276`; `slashing.rs:431-516`) | None until nominated proof of stake (ADR-020); then `pallet-staking` or its successor, `pallet-offences` and equivocation reporting | **Not carried** (ADR-020) | Validator pay comes from the release curve and issuance (ADR-004, U-11). None of the custom code is carried. |
| **Governance**: proposals and square-root stake voting, where the stake is a number the caller supplies, voting power uses `f64`, and execution only encodes bytes (`framework/modules/governance/src/lib.rs:97-304, 173, 228, 348`). Unreferenced | Standard governance pallets: `pallet-referenda`, `pallet-conviction-voting`, `pallet-scheduler`, `pallet-preimage`; or `pallet-collective` | **Standard** (ADR-021) | A collective now, sudo on development and test networks only, full OpenGov before mainnet. Where voting power comes from is U-10, still open and to be decided before OpenGov. The code is not carried. |

### 3.4 Identity and delegation (QOR ID on chain)

| Today | Maps to | Disposition | Notes |
| --- | --- | --- | --- |
| **Session keys**: `SessionKeys:{primary}:{session}` stores an expiry block and nothing else. No scopes, no spend caps. Nothing consults it: a session key authorises nothing (`framework/modules/session-keys/src/session.rs:86-140, 211-235`; `transaction.rs:43-65`; `runtime.rs:317, 340`) | Option A: `pallet-proxy` with Demiurge proxy types as the call filter, plus a custom spend-cap pallet or extension. Option B: a custom delegation pallet. | **Standard** `pallet-proxy` plus custom spend caps (ADR-026) | Standard `pallet-proxy` filters which calls a delegate may make, supports removal and announcement delays, takes deposits, and has **no spending limit** (Verified SDK). "Session keys" means validator keys in Substrate (F-Q2). **Recommendation:** Option A. Revocation and call filtering are key-custody concerns, which ADR-001 keeps boring. Spend caps are custom under either option. |
| **qor-identity**: DID records and handles. Not dispatched, not in the state root, no authorisation on create, transfer or reserve (`framework/modules/qor-identity/src/did.rs:180-247`; `handle.rs:136-273`) | Option A: `pallet-identity`. Option B: a custom handle pallet. Option C: identity stays off chain in `services/qor-auth`, with only the account on chain. | **Not carried**; identity stays off chain (ADR-027) | `pallet-identity` is a registrar-and-judgement model with deposits, sub-accounts and username authorities. It holds no keys and delegates no signing (Verified SDK), so it is not the QOR ID model. Nothing in the current module is carried. |
| **agentic**: in-memory agent wallets holding their own keys; unlock does not verify the controller (`framework/modules/agentic/src/wallet.rs:154-184, 393-447`). Unreferenced | Agent rails are built on the delegation chosen above | **Not carried** | ADR-010 rules out agents holding their own keys for the first release. |

### 3.5 Assets (DRC-369 and neighbours)

| Today | Maps to | Disposition | Notes |
| --- | --- | --- | --- |
| **DRC-369 ownership, mint, transfer, approvals, operators, burn, metadata** (`nft.rs:389-617, 872-906`) | Option A: `pallet-nfts` as the ownership ledger, with a custom DRC-369 pallet for the semantics. Option B: a fully custom DRC-369 pallet. | **Standard** ledger with **Custom** semantics (ADR-025) | `pallet-nfts` provides collections, items, attributes, metadata, approvals, transfer locks, swaps, a price and buy call, pre-signed mint and deposits. It provides no nesting and no royalty enforcement: `pay_tips` is the nearest thing (Verified SDK). **Recommendation:** Option A. Ownership and approvals are commodity; the semantics are where DRC-369 differs (ADR-001). Option A also makes standard fractionalization available (below). The current token identity, a Blake2 hash of a counter (`nft.rs:1047-1055`), is not preserved. Nothing is published on it. |
| **Collections**: absent | `pallet-nfts` collections (Option A) or custom | Follows the row above | Required by ADR-009. |
| **Enumeration by owner**: absent; only a balance counter (`nft.rs:410`) | A storage index by owner, plus a runtime API for queries | **Custom** (index and API) | The RPC methods clients call for this are not registered today. |
| **Content fingerprint**: absent. `tokenURI` reads a key nothing writes (`methods.rs:2494-2498`) | A fingerprint field in the DRC-369 pallet; content off chain on the Mesh | **Custom** | "Any file" cannot live in state (F-D1). |
| **State and XP**: `NftState{xp, level, stats: Vec<u8>}`, reachable on chain (`nft.rs:1238-1243, 671`) | Custom DRC-369 pallet, or `pallet-nfts` attributes with typed custom logic | **Custom** | The stats vector must be bounded (F-D1). |
| **History and provenance**: absent (`chain_getTransactionHistory` ignores module calls, `methods.rs:421-470`) | Events, plus an off-chain indexer reading an archive node | **Standard** events; indexer SQD's squid SDK (ADR-028) | F-D2. |
| **Nesting**: reachable on chain; unbounded children (`nft.rs:745-818`). A parent-ownership check was added on 2026-09-14; there is still **no cycle check** | Custom pallet | **Custom** | F-D3. **Requirement R-2: nesting refuses cycles.** In the custom chain an owner can nest an asset under itself, or nest A under B and then B under A. `do_nest` checks only that the child is not already nested (Verified). Because burn refuses an asset with children, every asset in the cycle can then never be burned. The pallet must refuse any nest whose parent is the child or one of its descendants, within a bounded depth. Not fixed in the custom chain, by the owner's instruction. |
| **Royalties**: single recipient set on chain; `TransferWithPayment` only calculates and logs, and no CGT moves (`nft.rs:293, 506-586`). Multi-recipient and remix royalties are library code nothing calls, with three conflicting caps (50%, 25%, 50%) and a double count (`royalty.rs:40, 124-154`; `royalty_distributor.rs:151-216`) | Custom pallet | **Custom** | F-D5. None of the library code is carried as-is. |
| **Rental**: library only, no ownership check, no CGT movement, not consulted by transfer (`rental.rs:275-425`) | Custom pallet | **Custom** | F-D6. |
| **Fractional ownership**: library only, no ownership check, NFT never locked (`fractional.rs:223-452`) | `pallet-nft-fractionalization`, which locks a `pallet-nfts` item and mints `pallet-assets` fractions (Verified SDK) | **Standard** (ADR-025). `pallet-assets` is in the runtime for fractionalization only, which is not a reversal of ADR-031 | Nothing in the current code is carried. |
| **Physics**: `f32` properties stored as serde JSON; dev-only RPC setter that skips validation (`physics.rs:10-147, 363-370`; `methods.rs:2256-2280`) | Custom pallet, with a fixed-point representation | **Custom** | F-D4. |
| **metaverse.rs, security.rs, cvp_hooks.rs**: type definitions, an unauthenticated freeze, CVP hooks (`metaverse.rs:22-600`; `security.rs:325-352`) | none | **Not carried** | |
| **game-assets**: fungible game items keyed by game and asset type; every call is a no-op (`framework/modules/game-assets/src/assets.rs:31-82`) | None in the first release; `pallet-assets` for game items when a game needs it | **Not carried** (ADR-031) | `pallet-assets` is present for fractionalization only (ADR-025), which is not permission to build fungible game items. |
| **yield-nfts**: every call a no-op; described as NFTs that "earn passive income" (`framework/modules/yield-nfts/src/lib.rs:3`; `yield_nfts.rs:31-59`) | none | **Not carried** | No-op, and its framing breaks ADR-008. |
| **game-registry**: excluded from the workspace, does not build, in-memory map (`framework/Cargo.toml:20`; `registry.rs:174-418`) | none | **Not carried** | Whether a game registry belongs on chain is a later question. |

### 3.6 Ruled out earlier

| Today | Disposition | Reason |
| --- | --- | --- |
| **cvp**: about 9,200 lines; disabled in the node (`framework/node/src/service.rs:140`) | **Not carried** | D-010, ADR-001 |
| **zk**: every verifier returns `Ok(true)` (`framework/modules/zk/src/private_transfer.rs:42-46`) | **Not carried** | D-010 |
| **consensus/modular.rs, consensus/sharding.rs**: unreferenced | **Not carried** | ADR-001 |
| **Mesh payments**: absent | **Custom**, not started | ADR-005. Blocked on U-6, how seeder work is verified. |

---

## 4. Base-layer components and clients

### 4.1 Node side

| Today | Maps to | Disposition |
| --- | --- | --- |
| Slot schedule with Ed25519 seal and no finality (`engine.rs:294-362`; `chain.rs:149-327`) | Block production by Aura (ADR-019), finality by GRANDPA (ADR-018) | **Standard** |
| libp2p gossip, Kademlia, custom sync over gossip (`framework/network`) | `sc-network` and standard block sync | **Standard** |
| Transaction pool: first in, first out, a 500-transaction cap per block (`framework/network/src/pool.rs:47-86`; `chain.rs:34`) | `sc-transaction-pool` with block weight and length limits | **Standard** |
| RocksDB with a hand-written binary Merkle root over chosen prefixes (`framework/storage/src/root.rs:18-45`) | Substrate client database and state trie | **Standard**. D-006 carries as a requirement. |
| Blake2b-512 truncated to 32 bytes as the hash everywhere (`framework/core/src/block.rs:119-125`) | Substrate hashing (Blake2b-256) | **Standard**. The outputs differ from today's (Verified repo for today's function), so every hash changes. |
| JSON-RPC: about 60 custom methods, 15 of them dev-only direct writes, 5 subscriptions that never fire (`methods.rs`, `framework/rpc/src/server.rs`) | Standard Substrate JSON-RPC, plus custom runtime APIs for DRC-369 queries | **Standard**, with custom runtime APIs |
| No runtime upgrade path | Runtime upgrades by `set_code` through governance (or sudo on test networks) | **Standard** |
| Node CLI and validator key file (`framework/node/src/main.rs:367-463`) | Standard node CLI and keystore | **Standard** |

### 4.2 Outside the runtime

| Today | What changes | Disposition |
| --- | --- | --- |
| **Launcher chain client**: was 7 custom RPC methods, a hand-built transfer payload, and addresses shown as `0x` and 64 hex characters. **Rebuilt on 2026-09-20** (`tools/qor-launcher/src-tauri/src/chain/`) | Done (ADR-040). `subxt` at the latest release under ADR-033 rule 2, every call addressed against the metadata the node serves, a `subxt::Config` of the launcher's own, now with the `MultiAddress` the runtime moved to (ADR-041), the vault signing behind the host dialog rather than through `subxt`'s infallible signer, and the endpoint now a WebSocket address. Proven against a development node. | **Rebuilt** |
| **Launcher vault**: 24 words, Argon2id and XChaCha20-Poly1305 (`vault/seal.rs`), and **Sr25519 through `sp-core` since 2026-09-19** (`vault/derive.rs`), replacing SLIP-0010 Ed25519 at `m/44'/369'/account'/0'/index'` | Done (ADR-023, ADR-039). Sealing and custody unchanged, as planned. | **Amended** |
| **QOR ID service** (`services/qor-auth`) | Stays off chain. **Since 2026-09-19 it takes an account as SS58 or as advanced hex and stores its 32 bytes** (`chain_account_id`, migration 018), verifying Sr25519 (ADR-023, ADR-024, ADR-039); the `^0x[0-9a-f]{64}$` column of migration 009 is gone. Its challenge signing is domain-separated since 2026-09-14 (F-Q6), and still names no network (F-Q9). | **Amended** |
| **SDKs** (`sdk/`, `packages/`, Unreal SDK) | None produces a transaction the current node accepts: wrong payload, sr25519, unregistered methods. Rebuilt against the new chain's metadata (ADR-009). | Rebuild (frozen until then) |
| **MCP server**: `tools/spline-mcp-server` is empty | New code on the same signing path as the SDK, using delegated keys (ADR-010) | New |

---

## 5. Where FRAME makes DRC-369 harder

**F-D1. "Any file" cannot be stored in state; every field needs a bound.**
- Today, metadata, stats, resources, children and delegation permissions are all unbounded vectors,
  with a single 1 MiB cap on the whole call (`nft.rs:1150-1270`; `transaction.rs:89`). Verified (repo).
- Under FRAME, stored values are bounded and storage carries deposits and weights. `pallet-nfts`
  exposes explicit limits on string, key and value sizes (Verified SDK).
- File bytes therefore live off chain (on the Mesh), with only a fingerprint on chain.
- `pallet-transaction-storage` can index data up to the runtime's `MaxTransactionSize` per transaction (default
  8 MiB) for a retention period, with storage proofs (Verified SDK). That is a candidate for small files. Whether it fits the Mesh
  design is open.
- **The bounds become part of the DRC-369 wire format**, so they are chosen before any SDK is
  published (ADR-009).

**F-D2. Provenance is not queryable chain state.**
- The public viewer must show each asset's provenance chain (ADR-011).
- In FRAME, history is naturally recorded as events. Events are one storage value for the current block, cleared
  every block, so a runtime cannot query them later. By default a node keeps every finalized block body but only the
  last 256 blocks of state, so old events are gone (Verified at `polkadot-stable2606-1`; corrected from an earlier
  claim that old block data is not kept at all).
- So either the viewer depends on an archive node plus an indexer, which is a new off-chain
  dependency that the "no auth, no wallet" page would rely on, or history is kept on chain in bounded
  form, which costs storage and deposits.
- This choice was Q-11: events, an archive node and an indexer (ADR-028).

**F-D3. Nesting is not provided and has to respect weights.**
- `pallet-nfts` has no item-owns-item relation (Verified SDK), so nesting is fully custom.
- Moving a parent with its children, burning, or checking for cycles must stay within a block's
  weight. That means a depth limit, a child limit, and an ownership model that does not walk the tree
  on every transfer (Inferred).
- The current implementation has no parent-ownership or cycle check (§2), so there is nothing safe
  to port.

**F-D4. Physics values cannot stay as floats.**
- Physics is `f32` today, stored as JSON. NaN mass passes validation because every comparison with
  NaN is false (`physics.rs:13-132, 186-192`). Verified (repo).
- Substrate runtimes use fixed-point arithmetic from `sp-arithmetic`. That floating point must not
  appear in runtime state is standard practice rather than something verified in the SDK source, so
  it is Inferred.
- Physics therefore needs a fixed-point encoding and a quantisation rule for values that engines
  produce as floats. This is a wire-format decision.

**F-D5. Royalties can only be enforced on sales that go through the chain.**
- `pallet-nfts` transfers and its buy call pay no royalty (Verified SDK).
- A custom pallet can enforce royalties on sales and licences it settles. A plain transfer between
  two accounts carries no price, so nothing can be enforced on it.
- That limit holds on any chain. It becomes concrete here because the standard ownership ledger has
  no hook for it, so under DRC-369 Option A the custom pallet must control which transfers are
  allowed.
- Remix royalty graphs need bounded depth for the same weight reason as nesting.
- **The recipient bound is decided: eight (Q-18, ADR-057, 28 September 2026).** ADR-047 set `MaxRoyaltyRecipients` to 8 as an engineering value. A song's
  credits — writers, performers, a publisher, the sources of its samples — can pass eight, and the bound becomes wire
  format at the freeze, after which raising it is a breaking change. Carried forward by the owner on 22 September 2026,
  so that `beta.wire-format-frozen` cannot be met while it stands unexamined.

**F-D6. Time-based terms need a clock and bounded expiry.**
- Rental durations are in seconds today and are never enforced (`rental.rs:165-168, 405-425`).
- Under FRAME, expiry is either checked lazily on use or processed in bounded per-block hooks or the
  scheduler (Inferred). The choice between block number and timestamp as the unit of a rental is part
  of the wire format.

**F-D7. Deposits and fees sit in the way of "mint without holding tokens".**
- Storage deposits (`pallet-nfts` collections and items; Verified SDK) and fees both require CGT at
  mint time.
- Sponsored minting for new creators therefore has to cover deposits as well as fees, and the account
  still has to exist (F-Q1).

**F-D8. Every call needs weight benchmarks.** The DRC-369 surface is large (sixteen on-chain call types
today, `nft.rs:1150-1235`), and each call needs a benchmarked weight before a public network. This is
cost, not risk (Inferred).

---

## 6. Where FRAME makes QOR ID and its clients harder

Identity, accounts and the things that hold keys: QOR ID, the launcher's vault and its chain client.

**F-Q1. An account that holds nothing cannot send a transaction.**
- Standard `CheckNonce` returns `InvalidTransaction::Payment` when the sender has zero providers and
  zero sufficients (Verified SDK).
- `SkipCheckIfFeeless` skips fees for chosen calls (Verified SDK). It does not give the sender a
  provider reference.
- So "end users can transact without holding tokens" needs an explicit design. Options:
  - an existential deposit paid by a sponsor, which is CGT the user then holds;
  - a sufficient asset;
  - a custom provider reference granted by a sponsor pallet, which is security-sensitive.
- This interacts with the existential deposit value (§3.1) and with U-4. It is the most consequential
  friction in this inventory.

**F-Q2. "Session keys" already means something else.**
- In `pallet-session`, session keys are validators' consensus keys (Verified SDK).
- QOR ID's agent keys need a different name in code, documentation and the interface, or every
  developer reading Substrate material will be misled.

**F-Q3. Standard delegation filters calls but does not cap spending.**
- `pallet-proxy` gives call filters, revocation, announcement delays and per-proxy deposits, and has no
  spending limit (Verified SDK). Its proxy types are a type the runtime supplies, normally an enum, compiled into the
  runtime (Verified SDK),
  so adding a new kind of scope is a runtime upgrade (Inferred).
- ADR-010's protocol-enforced spend caps are therefore custom under either option in §3.4.
- The delegate signs and pays the fee for a proxied call, so an agent needs its own funded or
  sponsored account (Inferred).

**F-Q4. The launcher's key derivation will not match other Substrate wallets.**
- The launcher derived Ed25519 keys by SLIP-0010 at `m/44'/369'/…` until 19 September 2026. Verified
  (repo, before the change).
- An Ed25519 public key maps to the same 32-byte account (Verified SDK), so account bytes carry over.
- But the ecosystem's wallets default to Sr25519, and derive from the phrase with their own junction
  scheme (Inferred). The same 24 words would open different accounts in the launcher and in a
  third-party wallet unless a compatible path is chosen.
- This touches ADR-009's goal of third-party wallet support. The frozen wallet extension uses a third,
  incompatible scheme (`apps/wallet-extension/background/keyring.ts:44-56`).
- **Closed on 2026-09-19** (ADR-023, ADR-039). The launcher derives Sr25519 keys with `sp-core` itself, at
  the version the chain is pinned to: the first account is the bare phrase, and further accounts are the
  hard junctions `//0`, `//1`, … as Talisman enumerates them. Every account is pinned by a test against
  `Pair::from_string("<phrase>//n")`. The "Inferred" parts above are now Verified: Talisman from its source,
  Nova from its documentation, Polkadot.js from ADR-023.

**F-Q5. Address display changes, and QOR ID's database refuses the new form.**
- Substrate addresses are displayed in SS58 with a network prefix (Inferred for the prefix choice).
- `services/qor-auth` constrains `on_chain_address` to `^0x[0-9a-f]{64}$`
  (`services/qor-auth/migrations/009_fix_on_chain_address.sql:49`). Since 2026-09-14 both `link-keypair`
  and `link-wallet` take a 64-hex Ed25519 key and store `0x` followed by it (`link_verified_key` in
  `src/handlers/auth.rs`), so the service no longer contradicts itself. Verified (repo).
- The column and both routes still refuse SS58, so whatever Q-7 decides for display, storage or the routes
  need a migration of their own.
- **Closed on 2026-09-19** (ADR-024, ADR-039). Migration 018 replaced `on_chain_address` and
  `primary_pubkey` with `chain_account_id BYTEA`, 32 bytes, unique. Every route that names an account takes
  it as SS58 in `address` or as hex in `account_id`, and every response returns both. The prefix is
  configuration (`chain.ss58_prefix`, default 42), never a constant in a query.

**F-Q6. Sign-anything vaults become riskier next to a richer transaction format.**
- The vault signs arbitrary bytes for its own accounts (`vault/mod.rs:232-252`).
- QOR ID sign-in signs the raw challenge string with no domain tag (`identity/mod.rs:140, 169, 193`).
- The auth endpoint can be changed from the webview (`src-tauri/src/lib.rs:371-387`).
- A hostile auth server could ask for signatures over chosen bytes. Today forging a transaction this
  way is very unlikely (Inferred). Once the launcher signs standard extrinsic payloads, challenge
  signing should be domain-separated so the two can never overlap. This is cheap now and awkward
  later.
- **Status (2026-09-14):** challenge signing is domain-separated (security track item 7, L1.5). Both
  sides use the tag `demiurge:qor-id:challenge:v1:`. The launcher also refuses to sign a challenge
  that is not in the service's format. Under L1.4, the vault signs only after an approval in a
  dialog the host draws (or, for the QOR ID sign-in straight after the vault opens, the opening
  itself: ADR-016, ADR-056), and an endpoint change needs the same approval. This is implemented and
  unit-tested but not yet exercised in a running launcher.

**F-Q7. Standard identity is not QOR ID.** `pallet-identity` is registrar judgements and deposits, with
no keys and no delegation (Verified SDK). QOR ID on chain is custom or stays off chain (§3.4).

**F-Q8. A password-only account has an on-chain address that no key can sign for** (question Q-16).
- **Today.** Registration with a password gives the account an `on_chain_address` of SHA-256 over
  `demiurge:address:` followed by `username#discriminator:timestamp` (`services/qor-auth/src/services/auth_service.rs`,
  `hash_to_address`; called from `register` in `src/handlers/auth.rs`). Verified (repo).
  - It has the shape of an account id, so it passes the format constraint of migration 009, and it is
    returned at registration and in the profile.
  - No private key corresponds to it. Anything sent to it could never be moved (Inferred from the
    derivation).
  - Keypair and agent accounts do not have this problem: their address is their own Ed25519 public key.
- **Not patched, on purpose** (owner, 2026-09-15). Patching it now would settle, by accident, how
  password-only accounts relate to chain identity.
- **Closed on 2026-09-19**, the way ADR-017 decided. Registration stores no address, `hash_to_address` is
  removed, the profile reports none until a key is proven, and migration 018 cleared the derived addresses
  already stored.
- **Options.**
  1. **No chain identity until the user proves a key.** A password-only account has no `on_chain_address`
     until `link-keypair` or `link-wallet` binds a key the user has shown they hold, for example one the
     launcher vault creates. QOR ID holds nothing, and every address has a holder. The cost: such an
     account cannot receive or hold anything on chain until the user sets up a key, so web-only users need
     that onboarding step.
  2. **A key QOR ID generates and holds** for each password account, signing on the user's behalf. Web users
     transact with no wallet. The cost: QOR ID becomes a custodian and a single point of compromise for
     every such account's assets. That reverses ADR-014, where QOR ID authorises keys and never holds them,
     and breaks ADR-001's rule to stay boring on key custody.
  3. **A derived, keyless address (today).** Deterministic and free. The cost: the address can receive but
     never send, so anything sent to it is lost, while every interface shows it as a real account. Not
     viable for anything of value.
  4. **A chain-side account controlled through an authorisation QOR ID attests to**, such as a pure proxy,
     a multisig or a recovery arrangement whose controller is added once a key is proven, with sponsorship
     so it can exist without holding CGT (F-Q1, Q-12). The cost: custom and security-sensitive runtime
     design. QOR ID's attestation also becomes something the chain trusts, which R-3 forbids for creating
     value and which touches Q-9 and Q-10.
- **Decided on 15 September 2026: option 1** ([ADR-017](../decisions/ADR-017-password-accounts-no-chain-identity-without-a-key.md)).
  The owner took the recommendation below. It is not implemented yet: it lands with the Substrate work, and
  ADR-017 lists what changes in the account-creation path.
- **Recommendation: option 1.** A chain identity exists exactly where a key the user holds exists. It keeps
  QOR ID out of custody, matches ADR-014, and closes nothing off: option 4 can be layered on later for
  accounts that want recovery or sponsorship. Until Q-16 is decided, a derived address should be treated as
  unusable: nothing should be sent or granted to it, and the new chain should not treat it as anyone's
  account. Clearing the addresses already stored is a migration for the decision, not before it.

**F-Q9. An address prefix is not chain identification** (found 17 September 2026, after ADR-024; raised by the owner).
- **The gap.** ADR-024's specification has QOR ID accept SS58 and refuse any prefix but the chain's. That check reads
  like validation and is not:
  - **Prefix 42 is shared.** It is the generic Substrate prefix, used by test networks and by any chain that does not
    connect to Polkadot or Kusama, Paseo among them. An address from another chain on 42 decodes and passes.
  - **The account bytes are chain-agnostic.** The same 32 bytes are a valid account on every Substrate chain. Which
    chain an address "belongs to" is not a property of the string. The prefix is a display convention, and the chain's
    own identifier is its genesis hash (`CheckGenesis`, D-009). Verified (SDK, `polkadot-stable2606-1`).
  - So refusing other prefixes stops an address written in Polkadot's or Kusama's format, and nothing else.
- **Where it bites.** Not in signing: a linked key can always sign on Demiurge. It bites where the holder cannot or
  will not sign here.
  - A custodial deposit address for another network: the exchange holds the key and will never sign a Demiurge
    transaction, so anything sent there is unrecoverable.
  - An account with no key at all, such as a pure proxy, a multisig or a sovereign account on another chain. It
    decodes cleanly and can never be spent from here.
  - A signature solicited for one Demiurge network then replayed on another: today's challenge is domain-separated
    (`demiurge:qor-id:challenge:v1:`, `src/handlers/auth.rs:860`) but names no network, so nothing in it says
    devnet, testnet or mainnet. Verified (repo).
- **Today this is latent, not live.** Every stored address is derived from a key proven by a signed challenge
  (`consume_signed_challenge` then `link_verified_key`, `src/handlers/auth.rs:1236-1260`). Verified (repo). It becomes
  live the moment any path accepts an address as input, which is exactly what ADR-024's migration introduces.
- **Options.**
  1. **Prefix check only (ADR-024 as written).** Rejected: it identifies no chain, and it looks like it does.
  2. **Bind proof of possession to the network.** Every path that records an address requires a signature over a
     challenge that includes the chain's genesis hash and network name, alongside the existing domain separator, the
     account and an expiry. A signature then proves the holder signed *for this network*: one gathered on devnet does
     not link on mainnet, and one solicited by another project does not replay here.
  3. **Require an on-chain proof**, such as a transaction sent from the account on our chain. It proves the key works
     here, but it needs a funded account, which F-Q1 and Q-12 say a new user does not have. It also cannot be done at
     sign-up.
  4. **Accept unproven addresses but mark them** display-only. Two classes of address, the confusing one still stored,
     and every consumer must remember the difference.
  5. **Refuse address-only input everywhere.** A rule rather than a mechanism: no path stores an address that arrived
     without proof of possession.
- **Recommendation: 2 and 5 together.** No path records an address without proof of possession, and the challenge is
  bound to the network by its genesis hash. The prefix check stays, as an input-shape check and a guard against a
  Polkadot or Kusama address pasted by mistake, and is never described as chain identification. Where a network name
  is shown to a person, it comes from the genesis hash QOR ID is configured with, not from the prefix.
  - This does not close the custodial-deposit-address case by itself, because nothing can: if a user can get their
    custodian to sign this challenge, the account is usable here. What it closes is storing an address on the word of
    its format alone.
  - A payout or withdrawal address for another network is a different feature. If one is ever wanted, it is designed
    then, as an explicitly foreign address, and never through the account-linking path.
- **Half implemented, 2026-09-19** (ADR-039), with the SS58 migration.
  - **Recommendation 5 is in place.** No route stores an account that arrived without proof of possession: every one
    of them goes through `consume_signed_challenge`, and a test asserts that an account offered with a challenge
    nobody issued leaves the row empty. The prefix check is an input-shape check, described as one in the code, in
    the configuration comment and in the error a caller sees.
  - **Recommendation 2 is not.** The challenge still names no network. There is no deployed network whose genesis
    hash could be configured, and a challenge bound to the wrong hash would be worse than one bound to none, because
    it would look like the protection it is not. The reason is recorded beside the domain tag in
    `src/handlers/auth.rs` rather than only here, so whoever adds the network identity finds it.
  - **What is still open**, therefore: a signature gathered on one Demiurge network can be replayed to link the same
    account on another. It becomes closable as soon as a network has a genesis hash worth configuring, which is the
    first deployment (ADR-015).

**F-Q10. The runtime's address type is not the ecosystem's** (found 20 September 2026, while building the launcher's
chain client, ADR-040). **Closed the same day by [ADR-041](../decisions/ADR-041-multiaddress-and-accountidlookup.md):
the runtime now uses `AccountIdLookup` and an address is a `MultiAddress`.** The finding is kept as written, with its
correction, because it is the record of how the question was found and of one claim in it being wrong.
- **What was found.** `chain/runtime/src/lib.rs` sets `frame_system::Config::Lookup` to `IdentityLookup<AccountId>`,
  so an extrinsic's address field is a bare `AccountId32` and `Balances`' `dest` is a bare account. The Polkadot and
  Kusama runtimes, and the SDK's fuller node template, use `AccountIdLookup<AccountId, ()>`, whose address is
  `MultiAddress`. Verified (repo and SDK, `polkadot-stable2606-1`).
- **This is not a defect, and nothing is blocked by it.** `IdentityLookup` is a standard `sp_runtime` component and
  a legitimate choice: it is cheaper, and it removes a lookup the chain does not use. Clients that read the address
  type from metadata — Polkadot.js, Talisman and `subxt` — all handle it. The launcher does: it declares its own
  `subxt::Config` with `Address = AccountId32` (ADR-040, decision 3).
- **Corrected 20 September 2026.** This finding first said `IdentityLookup` was the minimal template's default.
  **It is not.** `frame_system::config_preludes::SolochainDefaultConfig`, which the SDK's `solochain` *and*
  `minimal` templates both derive from, sets `AccountIdLookup<AccountId, ()>`
  (`substrate/frame/system/src/lib.rs:399`). The prelude that sets `IdentityLookup` is `TestDefaultConfig`, whose
  `AccountId` is `u64` and which is for pallet unit tests (`:337`). Verified (SDK, at the pinned tag). So
  `chain/runtime/src/lib.rs:181` **overrides** the SDK's solochain default rather than inheriting it, and the
  override has no recorded reason.
- **Where it could bite.** A client that assumes `MultiAddress` because every other Substrate chain it has met used
  one will encode an extra variant byte, and every transaction it signs will be refused. That is a loud failure, not
  a quiet one, but it is a failure a third-party integrator meets before they meet anything else, and it is the kind
  of thing an SDK's first user reports as "your chain is broken".
- **What it would cost to change**, if it is ever wanted: a runtime change plus a storage-free migration, because no
  state is keyed by the address type; every client and every test that builds an extrinsic; and it is a wire-format
  change, so it belongs before the format is frozen at M5.1, not after.
- **Not decided in ADR-040, and written up separately for the owner.** ADR-040 deliberately left the runtime alone:
  it was a chain-side question, and the launcher worked either way. The options, what each costs, what breaks if it
  changes after M5.1, and the recommendation are in [`ADDRESS_TYPE.md`](ADDRESS_TYPE.md). It was **question Q-17**
  in §8.
- **Decided 20 September 2026: `AccountIdLookup`** (ADR-041). The correction above is what decided it: a departure
  from the SDK's own default, with no reason on record, is exactly what AGENTS.md §7 asks to be justified, and
  writing the case found no justification. Carried out the same day, runtime and launcher in one commit, because a
  node and a client on opposite sides of the change refuse each other.
- **One thing the write-up got wrong, recorded here too.** `ADDRESS_TYPE.md` §4 said `acceptance.rs` needed no
  change because it dispatches through `RuntimeOrigin::signed`. Eight call sites needed it: the `dest` **argument**
  of a balances call is `AccountIdLookupOf<T>`, which changes with `Lookup` whether or not an extrinsic address is
  involved.

---

## 7. Where it gets easier

This list keeps the inventory honest about the other direction.

- Replay across networks is closed by `CheckGenesis` (Verified SDK).
- Nonces, atomic dispatch, events, finality and runtime upgrades are all standard.
- Fractional ownership has a standard pallet if DRC-369 builds on `pallet-nfts` (Verified SDK).
- Third-party tooling can read runtime metadata instead of learning a custom JSON transaction shape.
  Today no SDK in the repository produces a transaction the node accepts.
- Ed25519 accounts keep their bytes (Verified SDK).

---

## 8. Open questions for the owner

Eighteen are decided: see Resolution status below, and the ADR named there. Q-17 was added and decided on
20 September 2026. **Q-18, Q-19 and Q-20 were added on 22 September 2026; Q-18 was decided on 28 September 2026
(ADR-057), and Q-19 and Q-20 are open.** The table keeps the questions as they
were asked.

| # | Question | Options | Blocks |
| --- | --- | --- | --- |
| Q-1 | Does "L1" mean a standalone chain or a parachain? | Standalone (own validators and finality); parachain (security from the relay chain, bought coretime). The master direction's "purpose-built Substrate L1" reads as standalone; that reading is Inferred. | Everything in §4.1 |
| Q-2 | Block production | Aura (fixed slot rotation) or BABE (randomised slots) | Node template, validator operations |
| Q-3 | How the validator set is chosen | Nominated proof of stake (`pallet-staking`) or a permissioned set changed by governance | Validator pay (U-11), staking UI, genesis |
| Q-4 | Governance pallets and voting power | OpenGov set, collective or council, or a hybrid; voting power from CGT or from something else | Treasury, releases, runtime upgrades, U-10 |
| Q-5 | Which Polkadot SDK release to pin | The latest stable listed on 14 September 2026 is `polkadot-stable2606-1` | Every Verified (SDK) claim gets re-checked against it |
| Q-6 | Account key scheme and derivation | Ed25519 with the launcher's SLIP-0010 path; Sr25519 with ecosystem derivation; or both with a compatibility path | Third-party wallets, the launcher vault, qor-auth verification (F-Q4) |
| Q-7 | Address format and network prefix | SS58 with a prefix to choose; hex in advanced views | Launcher and qor-auth display and storage (F-Q5) |
| Q-8 | DRC-369's ownership ledger | Build on `pallet-nfts` (recommended) or fully custom | Collections, fractionalization, token identity, SDK wire format |
| Q-9 | Agent delegation | `pallet-proxy` plus custom spend caps (recommended), or a custom delegation pallet | ADR-010, the MCP server |
| Q-10 | QOR ID on chain | `pallet-identity`, a custom handle pallet, or off chain only | Handles, the public viewer's creator display |
| Q-11 | Provenance for the public viewer | Events plus an archive node and indexer, or bounded on-chain history | ADR-011 (F-D2) |
| Q-12 | Accounts that hold nothing | Sponsor-paid existential deposit, a sufficient asset, or a custom sponsor provider | Sponsored onboarding and minting (F-Q1, F-D7) |
| Q-13 | Existential deposit value | Not decided anywhere current | Q-12 |
| Q-14 | Whether fungible game items are in the first release | `pallet-assets`, or later | game-assets row |
| Q-15 | Where the new chain lives in the repository, and its crate and pallet names | Owner's choice under the naming convention | The first line of migration code |
| Q-16 | How a password-only account relates to chain identity | No chain identity until a key is proven (recommended); a key QOR ID holds; today's keyless derived address; a chain-side account QOR ID attests to (F-Q8) | L3.2 address display, Q-10, anything that pays or grants to a password-only account |
| Q-17 | The runtime's address type | Keep `IdentityLookup` (a bare `AccountId32` on the wire); or `AccountIdLookup<AccountId, ()>`, the SDK's own solochain default and every relay chain's (recommended). Written up with costs and consequences in [`ADDRESS_TYPE.md`](ADDRESS_TYPE.md) | The wire format frozen at M5.1, the SDK, and every third-party integration (F-Q10) |
| Q-18 | Is eight royalty recipients per asset enough? | Keep `MaxRoyaltyRecipients` at 8 (ADR-047, decision 13, row 7); raise it, as `docs/blueprints/gnosis.md` recommends, because a song's credits — writers, performers, a publisher, the sources of its samples — can pass eight; or carry the long tail another way that ADR-047 does not yet describe. Examined, with an accepted ADR, before the freeze (ADR-047, open item 1) | `beta.wire-format-frozen`, which cannot be met while this is open (`beta.royalty-recipients` counts it), and GNOSIS (P4) |
| Q-20 | Is the media-type table part of an asset's identity? | Specify the mapping as part of the format, so every client computes one root; or drop `media_type` from the manifest and let a reader infer one; or keep the field for readers and exclude it from the hash. Today the format says "IANA" and names no table, and the launcher's table was widened on 22 September 2026, which changes the root a re-mint of the same files produces (ADR-047, open item 3) | `beta.wire-format-frozen`, which cannot be met while this is open (`beta.media-types` counts it), and every client that computes a root |
| Q-19 | What is an asset's root a function of? | The manifest as ADR-047 decides it, with the pinned commit's id and time and the branch's name inside, so identical files from two commits, or one commit from two branches, have two roots; or the files alone, with the commit and branch kept beside the manifest, so identical files always have one root. ADR-047 says both (decisions 5 and 8, and "What it makes possible"), and they conflict (ADR-047, open item 2) | `beta.wire-format-frozen`, which cannot be met while this is open (`beta.manifest-identity` counts it), and every client that computes a root |


### Resolution status

The release gates (`docs/GATES.toml`) read this table. When a question is decided, its status becomes
"Resolved by ADR-nnn", naming the ADR that records the decision.

| # | Status |
| --- | --- |
| Q-1 | Resolved by ADR-018 |
| Q-2 | Resolved by ADR-019 |
| Q-3 | Resolved by ADR-020 |
| Q-4 | Resolved by ADR-021 |
| Q-5 | Resolved by ADR-022 |
| Q-6 | Resolved by ADR-023 |
| Q-7 | Resolved by ADR-024 |
| Q-8 | Resolved by ADR-025 |
| Q-9 | Resolved by ADR-026 |
| Q-10 | Resolved by ADR-027 |
| Q-11 | Resolved by ADR-028 |
| Q-12 | Resolved by ADR-029 |
| Q-13 | Resolved by ADR-030 |
| Q-14 | Resolved by ADR-031 |
| Q-15 | Resolved by ADR-032 |
| Q-16 | Resolved by ADR-017 |
| Q-17 | Resolved by ADR-041 |
| Q-18 | Resolved by ADR-057 |
| Q-19 | Open. Found on 2026-09-22 building M4.1; answered before the freeze |
| Q-20 | Open. Found on 2026-09-22 widening the media-type table; answered before the freeze |

---

## 9. What survives as acceptance tests

Nothing here is ported as code. These tests and checks describe behaviour the new chain must reproduce,
or defects it must not have.

| Source | Tests | What they pin down |
| --- | --- | --- |
| `framework/core/tests/block_execution_test.rs` | 9 | Nonces, replay refusal, atomicity, producer and importer agreement, state root ignoring node-local data |
| `framework/node/tests/chain_test.rs` | 8 | Import rules, catch-up. **Partly reproduced 2026-09-18** in `chain/runtime/tests/block_import.rs`: producer and importer agreement, a tampered state root or extrinsics root refused, a run of blocks in order. **The parent-hash rule cannot be reproduced in the runtime**: `Executive::execute_block` calls `initialize_block` before `initial_checks`, and initialising writes `BlockHash(n-1)` from the header, so the assertion compares the value it just wrote. A node's import queue resolves the parent in its database first; that rule belongs to the multi-node harness. Catch-up also needs real nodes. |
| `framework/modules/balances/tests/balances_test.rs`, `framework/core/tests/block_execution_test.rs` | 11 + 1 | Transfer and existential deposit rules. `test_self_transfer_is_refused_and_creates_nothing` and `a_self_transfer_module_call_creates_no_cgt` (added 2026-09-14) pin that a self-transfer creates no CGT. |
| `framework/modules/energy/tests/energy_test.rs` | 7 | Allowance accounting, to revisit under U-4 |
| `framework/modules/session-keys/tests/session_keys_test.rs` | 7 | Authorise, revoke, expiry |
| DRC-369 tests in `nft.rs`, `royalty.rs`, `physics.rs` | about 12 relevant | State and XP, nesting, royalty arithmetic. `a_transaction_cannot_mint_into_another_account` and `nesting_under_a_parent_the_caller_does_not_own_is_refused` (added 2026-09-14) pin the mint and parent-ownership checks. Since 22 September 2026 `pallet-drc369` pins the authorised mint itself, in `an_unsigned_or_root_mint_is_refused` and `a_mint_goes_to_the_signer_in_the_signers_own_collection` (M4.1). **R-2 is met since nesting landed (ADR-065)**: `a_two_asset_cycle_is_refused` and `a_longer_cycle_is_refused` in `pallet-drc369`. |
| Requirement R-1 (strict signature verification) | **3 in `chain/runtime/tests/acceptance.rs`** (2026-09-18) | **Met by standard behaviour (ADR-038).** Sr25519, the account key scheme, refuses the forgery on 64 of 64 messages; Ed25519 accepts it under ZIP-215, which is a specification followed on purpose rather than the custom chain's accident, and which harms nobody because no secret key produces such an address. See §3.1's R-1 note. `services/qor-auth` has the same test for QOR ID's own verification (`a_forged_signature_for_a_small_order_key_is_rejected`). |
| Requirement R-4 (the ticker is `CGT` in names, not only on screen) | none; the custom chain is deliberately left reading `cgt` in names | A check that no identifier, call, event, storage item, RPC method or constant in the runtime or its clients contains `cgt`, and that the chain's `system_properties` reports `CGT` with the decided decimals. Greppable by design (ADR-032). |
| Requirement R-3 (no CGT created from outside the chain) | none in the custom chain | The new chain needs tests that total issuance changes only through calls with a chain-side authorised origin, and that any starter-grant call is refused where grants are off. QOR ID keeps an end-to-end check that registration sends nothing to a chain node (`services/qor-auth/scripts/e2e/registration-mints-nothing.mjs`). |
| The M1 live devnet checks (`scripts/run-local-stack.md` §3, now the Substrate two-validator script) | 13 checks | A transfer settles identically on every validator; replay refused; direct write refused; a killed validator resyncs |

Test counts come from counting test attributes, so they are approximate. Nothing was compiled for this
document.

---

## 10. Sources

Polkadot SDK source, `master`, read 14 September 2026:
- [`frame/system` `CheckNonce`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/system/src/extensions/check_nonce.rs)
- [`frame/system` `CheckGenesis`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/system/src/extensions/check_genesis.rs)
- [`sp-runtime` `MultiSigner` account mapping](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/primitives/runtime/src/lib.rs)
- [`pallet-nfts` README](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/nfts/README.md) and [source](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/nfts/src/lib.rs)
- [`pallet-nft-fractionalization`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/nft-fractionalization/src/lib.rs)
- [`pallet-proxy`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/proxy/src/lib.rs)
- [`pallet-session`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/session/src/lib.rs)
- [`pallet-identity` README](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/identity/README.md)
- [`pallet-skip-feeless-payment`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/transaction-payment/skip-feeless-payment/src/lib.rs)
- [`pallet-transaction-storage`](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/transaction-storage/src/lib.rs)
- [Polkadot SDK releases](https://github.com/paritytech/polkadot-sdk/releases)
- [`sp-arithmetic` fixed-point types](https://docs.rs/sp-arithmetic/latest/sp_arithmetic/fixed_point/struct.FixedU128.html)
