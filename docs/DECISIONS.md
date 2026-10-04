# Decisions

**The only decision log.** Every decision that shapes the protocol, its economics or the
platform is recorded here or in an architecture decision record indexed here, with the options
that were considered and why one was chosen. If a document anywhere else disagrees with this log,
this log is right and the other document is wrong.

The log has two parts:

- **D-000 to D-012**, written out in full below. They were made during the September 2026
  realignment.
- **ADR-001 onwards**, one file each in [`decisions/`](decisions/README.md), in context / decision /
  consequences form. They record the owner's direction from 13 September 2026 onwards.

A later decision supersedes an earlier one; it never rewrites it. A superseded entry below stays
for the record, and its status says what replaced it. Where an entry and a later ADR disagree, the
ADR is right.

The project owner delegated D-001 to D-003 to the engineering lead on 2026-09-13. The owner's own
direction of the same day **superseded all three** (ADR-003, ADR-004). The economic model is now
[`economics/CGT.md`](economics/CGT.md); the values it leaves undecided are in
[`economics/OPEN_QUESTIONS.md`](economics/OPEN_QUESTIONS.md) and are not to be invented.

## Index

| ID | Decision | Status |
| --- | --- | --- |
| D-000 | CGT has 18 decimals | Implemented. The 13 billion supply it restated is superseded by ADR-003; the precision stands (U-1 open) |
| D-001 | Fees: energy first, CGT fee fallback | **Superseded** by ADR-004. The fee numbers and split are withdrawn (OPEN-4); the sponsorship principle is retained (`CGT.md` §9) |
| D-002 | Staking rewards from a fixed reserve, never minted | **Superseded** by ADR-004 |
| D-003 | Genesis distribution | **Superseded** by ADR-003 (split open, OPEN-2) |
| D-004 | On-chain transaction nonces | Implemented (M1) in the custom chain. Carried to the Polkadot SDK chain as a requirement (ADR-013) |
| D-005 | Atomic transaction execution | Implemented (M1) in the custom chain. Carried as a requirement (ADR-013) |
| D-006 | One state database, scoped state root | Implemented (M1) in the custom chain. Replaced by Substrate's state trie; the requirement carries (ADR-013) |
| D-007 | Slot-scheduled block production, BFT finality next | **Superseded** by ADR-013. Production implemented (M1) in the custom chain; custom finality will not be built |
| D-008 | Direct-write RPC endpoints are development-only | Implemented (M1) in the custom chain. The requirement, that state changes only through blocks, carries (ADR-013) |
| D-009 | Domain-separated transaction signatures | Adopted, not implemented in the custom chain. Expected to be met by the Polkadot SDK's signed payload; verified against the pinned release (ADR-013) |
| D-010 | Remove `modules/zk`; no security claims for CVP | CVP disabled in the node (M1). Neither is carried to the Polkadot SDK chain (ADR-001, ADR-013) |
| D-011 | Legacy clients are frozen; the launcher is the client | **Amended** by ADR-011: two web surfaces are added; the frozen applications stay frozen |
| D-012 | Documentation governance | Implemented; amended 2026-09-13 to include the ADRs |
| [ADR-001](decisions/ADR-001-innovation-budget.md) | The innovation budget: innovate where failure is loud, stay boring where it is silent | Accepted |
| [ADR-002](decisions/ADR-002-value-from-spending.md) | CGT's value comes from demand to spend it: the spend test | Accepted |
| [ADR-003](decisions/ADR-003-supply-model.md) | Base supply of one hundred trillion CGT, released on a decay curve | Accepted; numbers open. Supersedes D-003 and D-000's supply figure |
| [ADR-004](decisions/ADR-004-issuance-and-burn.md) | Perpetual issuance for infrastructure, with fee burn as the counterweight | Accepted; numbers open. Supersedes D-002 and D-001's numbers |
| [ADR-005](decisions/ADR-005-useful-work-mining.md) | Useful-work mining on the Mesh, without artificial difficulty | Accepted |
| [ADR-006](decisions/ADR-006-demand-sinks.md) | Five structural demand sinks | Accepted |
| [ADR-007](decisions/ADR-007-display-layer.md) | A legible display unit over CGT, with no peg | Accepted |
| [ADR-008](decisions/ADR-008-language-discipline.md) | Language discipline: payment for work, never appreciation | Accepted |
| [ADR-009](decisions/ADR-009-universal-minting.md) | First release: universal minting, with game assets as the flagship | Accepted |
| [ADR-010](decisions/ADR-010-agent-rails.md) | Agent rails: delegated session keys and an MCP server | Accepted; amended by ADR-014 |
| [ADR-011](decisions/ADR-011-web-surface.md) | Two web surfaces: a public viewer and a thin remote console | Accepted. Amends D-011 |
| [ADR-012](decisions/ADR-012-substrate-base.md) | A purpose-built Substrate L1 as the base | **Superseded** by ADR-013 |
| [ADR-013](decisions/ADR-013-polkadot-sdk-migration.md) | Migrate the base layer to the Polkadot SDK | Accepted. Supersedes ADR-012 and D-007. Its consequence that the vault is not rebuilt is amended by ADR-023 (key derivation changes) |
| [ADR-014](decisions/ADR-014-agent-keys-authorised-not-created.md) | QOR ID authorises agent keys; it never creates them | Accepted. Amends ADR-010 |
| [ADR-015](decisions/ADR-015-infrastructure-ownership.md) | Which service owns which infrastructure concern | Accepted. Superseded for the devnet's nodes by ADR-068. Single-provider risk accepted until it is revisited before mainnet. Clarified 15 September 2026: QOR ID serves its own pages on its own subdomain; Vercel serves marketing and web surfaces only; the archive node is a dependency of the public viewer (ADR-028) |
| [ADR-016](decisions/ADR-016-sign-in-with-unlock.md) | Unlocking the vault signs in to QOR ID without a second approval | Accepted, under the owner's delegation. Narrows roadmap item L1.4 |
| [ADR-017](decisions/ADR-017-password-accounts-no-chain-identity-without-a-key.md) | A password-only account has no chain identity until it proves a key | Accepted. Resolves inventory Q-16; not implemented until the Substrate work |
| [ADR-018](decisions/ADR-018-standalone-chain-grandpa-finality.md) | A standalone chain, with its own validators and GRANDPA finality | Accepted. Resolves Q-1; revisited before mainnet |
| [ADR-019](decisions/ADR-019-aura-block-production.md) | Blocks are produced by Aura | Accepted. Resolves Q-2 |
| [ADR-020](decisions/ADR-020-permissioned-validators-now-npos-later.md) | A permissioned validator set chosen by governance now; nominated proof of stake later | Accepted. Resolves Q-3; its session manager is `pallet-validator-set`, named by the owner on 17 September 2026 |
| [ADR-021](decisions/ADR-021-collective-governance-now-opengov-later.md) | Governance by a collective now; a hybrid with OpenGov before mainnet | Accepted. Resolves Q-4; U-10 open |
| [ADR-022](decisions/ADR-022-pin-polkadot-stable2606-1.md) | Pin the Polkadot SDK at polkadot-stable2606-1 | Accepted. Resolves Q-5 |
| [ADR-023](decisions/ADR-023-sr25519-with-ecosystem-derivation.md) | Sr25519 accounts, derived the way the ecosystem derives them | Accepted. Resolves Q-6. Amends ADR-013 on the vault's derivation |
| [ADR-024](decisions/ADR-024-ss58-addresses-prefix-42-until-mainnet.md) | SS58 addresses, with prefix 42 until mainnet | Accepted. Resolves Q-7; the registry is archived and no registration process was found; the mainnet prefix a Public Release criterion |
| [ADR-025](decisions/ADR-025-drc369-on-pallet-nfts.md) | DRC-369 on pallet-nfts, with custom pallets for its semantics | Accepted. Resolves Q-8 |
| [ADR-026](decisions/ADR-026-agent-delegation-with-pallet-proxy.md) | Agent delegation through pallet-proxy, with custom spend caps | Accepted. Resolves Q-9 |
| [ADR-027](decisions/ADR-027-identity-stays-off-chain.md) | Identity stays off chain in QOR ID; only the account is on chain | Accepted. Resolves Q-10 |
| [ADR-028](decisions/ADR-028-provenance-from-events-archive-node-and-indexer.md) | Provenance comes from events, an archive node and an indexer | Accepted. Resolves Q-11 |
| [ADR-029](decisions/ADR-029-sponsor-pays-the-existential-deposit.md) | A sponsor pays the existential deposit, and the user then holds it | Accepted. Resolves Q-12; mechanics (U-4) proposed for review |
| [ADR-030](decisions/ADR-030-existential-deposit-against-a-stated-target.md) | The existential deposit is set against a stated target | Accepted as a method. Resolves Q-13; the value (100 CGT) awaits the owner, and U-1 is open |
| [ADR-031](decisions/ADR-031-no-fungible-game-items-in-the-first-release.md) | No fungible game items in the first release | Accepted. Resolves Q-14. `pallet-assets` is present for fractionalization only, which is not a reversal |
| [ADR-032](decisions/ADR-032-chain-location-and-names.md) | Where the new chain lives, and its crate and pallet names | Accepted. Resolves Q-15; `.cursorrules` amended so Gnostic naming covers internals only; the fifth pallet is `pallet-validator-set` (17 September 2026) |
| [ADR-033](decisions/ADR-033-dependency-versions.md) | Dependency versions: the pinned SDK governs, everything else stays current | Accepted |
| [ADR-034](decisions/ADR-034-ticker-dmrg.md) | The ticker is DMRG; the name stays Creator-God Token | **Superseded** by ADR-045 |
| [ADR-035](decisions/ADR-035-eighteen-decimals.md) | CGT has eighteen decimal places, and scaled arithmetic uses the SDK's helpers | Accepted. Settles U-1 |
| [ADR-036](decisions/ADR-036-existential-deposit-100-dmrg.md) | The existential deposit is 100 CGT | Accepted. Completes ADR-030 |
| [ADR-037](decisions/ADR-037-sudo-on-development-and-test-networks-only.md) | `pallet-sudo` on development and test networks only, absent from mainnet | Accepted |
| [ADR-038](decisions/ADR-038-ed25519-follows-the-sdk-zip-215-rule.md) | Ed25519 follows the SDK's ZIP-215 rule; R-1 met by standard behaviour | Accepted |
| [ADR-039](decisions/ADR-039-account-derivation-and-the-scheme-change.md) | Derivation paths, and what the key-scheme change does to accounts that already exist | Accepted under the owner's delegation. Completes ADR-017, ADR-023 and ADR-024 |
| [ADR-040](decisions/ADR-040-the-launchers-chain-client-is-subxt.md) | The launcher's chain client is `subxt`, driven by the chain's own metadata | Accepted under the owner's delegation. Carries out L3.1, the second half of M3.4 |
| [ADR-041](decisions/ADR-041-multiaddress-and-accountidlookup.md) | The runtime looks accounts up with `AccountIdLookup`, and an address is a `MultiAddress` | Accepted. Resolves Q-17, closes F-Q10, amends ADR-040 decision 3 |

| [ADR-042](decisions/ADR-042-two-domains.md) | Two domains: `qorsync.dev` for operations, `demiurge.cloud` for what people see | **Accepted**, 28 September 2026, by the project owner (proposed 20 September) |
| [ADR-043](decisions/ADR-043-qor-id-as-an-identity-provider.md) | A browser frontend signs in to QOR ID by redirect, not by holding a refresh token | **Accepted**, 4 October 2026, by the project owner, with ADR-069, whose web surface needs it (proposed 20 September). Item 4, the CORS allowlist, was already done |
| [ADR-044](decisions/ADR-044-validators-are-not-publicly-addressable.md) | Validators have no public hostname; public RPC is a separate node | Accepted, 3 October 2026, by ADR-068. Restates ADR-015's archive-node clarification |

| [ADR-045](decisions/ADR-045-the-ticker-returns-to-cgt.md) | The ticker returns to CGT | **Accepted**, by the owner's instruction. Supersedes ADR-034, and requirement R-4 in the migration inventory |

| [ADR-046](decisions/ADR-046-the-launcher-is-an-app-host.md) | The launcher hosts products as separate OS processes it spawns, supervises and signs for | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-047](decisions/ADR-047-the-object-model.md) | The object model: a BLAKE3 manifest root on chain, chunking beneath it, and a mint that pins a commit | Accepted, 22 September 2026, by the project owner, with its three owner questions answered as recommended. Carries out M2.3 |
| [ADR-048](decisions/ADR-048-interchange-formats.md) | Interchange formats, so a creator can leave with their work | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-049](decisions/ADR-049-data-ownership-and-the-user-directory.md) | A creator's work lives in their QOR ID directory, keeps working offline, and leaves in open formats | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-050](decisions/ADR-050-where-the-products-live.md) | Where each product lives, and a release gate of its own for each | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-051](decisions/ADR-051-the-shells-rendering-layer-for-qfx.md) | QFX renders on one WebGL2 canvas behind the interface, under contracts the chrome owns | Accepted, 22 September 2026, by the project owner, as amended that day to fit layer one as built. Amends `DIRECTION.md` §5 and `DESIGN_SYSTEM.md` §1 and §6 |
| [ADR-052](decisions/ADR-052-assets-in-the-runtime.md) | Assets in the runtime: `pallet-nfts` configured from recorded sources, placeholder deposits, and one way in | Accepted, 22 September 2026, under the owner's delegation for engineering choices. Carries out M4.1's chain half; its deposits are placeholders under U-14 |

| [ADR-053](decisions/ADR-053-one-transaction-for-a-trade.md) | One transaction for a trade: `pallet-utility`'s atomic batch, and nothing else of it | Accepted, 22 September 2026, under the owner's delegation for engineering choices. Carries out M4.6 for L4.5; only `batch_all` is reachable, and it opens no second way to make an asset |
| [ADR-054](decisions/ADR-054-unlocking-with-windows-hello.md) | Unlocking the vault with Windows Hello: a TPM key's signature seals a second copy of the phrase, bound to the vault file | Accepted, 26 September 2026, by the project owner, who chose it over a DPAPI unlock and a plain key file. Extends ADR-016; opt-in, Windows only. **Superseded by ADR-055** |
| [ADR-055](decisions/ADR-055-the-vault-opens-with-windows-hello-alone.md) | The vault opens with Windows Hello alone: no passphrase, no password sign-in, a passphrase vault moved once | Accepted, 28 September 2026, by the project owner, who chose Hello only over a silent DPAPI unlock. Supersedes ADR-054; amends ADR-016; Windows only. **Superseded the same day by ADR-056** |
| [ADR-056](decisions/ADR-056-no-lock-screen.md) | No lock screen: the vault's key in the OS keychain, nothing asked, and QOR ID never blocks the launcher | Accepted, 28 September 2026, by the project owner. Supersedes ADR-055; amends ADR-016 |
| [ADR-057](decisions/ADR-057-eight-royalty-recipients.md) | Eight royalty recipients per asset: `MaxRoyaltyRecipients` stays 8 | Accepted, 28 September 2026, by the project owner. Answers Q-18 and ADR-047's first open item |
| [ADR-058](decisions/ADR-058-ci-on-woodpecker.md) | CI runs on Woodpecker, not GitHub Actions: a Fly.io server at `ci.qorsync.dev`, builds on the owner's computer | Accepted, 28 September 2026, by the project owner. Actions switched off; Codeberg the fallback Decision 1 superseded by ADR-059 **Superseded by ADR-063** |
| [ADR-059](decisions/ADR-059-operations-on-oracle-always-free.md) | The operations server runs on Oracle Cloud's Always Free tier: QOR ID, Postgres, Redis and the CI server behind Caddy on `qorsync.dev`, $0 | Accepted, 28 September 2026, by the project owner. Supersedes ADR-058 decision 1; departs from ADR-015 for these services; no public RPC until a private sudo key **Superseded by ADR-060** |
| [ADR-060](decisions/ADR-060-operations-on-the-owners-computer.md) | The operations stack runs on the owner's computer, published by Cloudflare Tunnel: QOR ID and Woodpecker at `id.`/`ci.qorsync.dev`, $0, no card | Accepted, 28 September 2026, under the owner's delegation after they rejected Oracle. Supersedes ADR-059; `qorsync.dev`'s DNS moves to Cloudflare **Superseded by ADR-063** |
| [ADR-061](decisions/ADR-061-royalties-and-the-settled-sale.md) | Royalties, remix royalties, and the sale settled in CGT they bind to: `pallet-drc369-royalties` | Accepted, 29 September 2026, under the owner's delegation for engineering choices. Carries out M4.2's royalty half; decision 2 superseded by ADR-062 |
| [ADR-062](decisions/ADR-062-royalty-terms-a-creator-can-correct.md) | Royalty terms a creator can correct while they hold the asset, and a remix share that never rises once the work is remixed | Accepted, 29 September 2026, by the project owner. Supersedes ADR-061 decision 2; confirms its one level of remix royalty |
| [ADR-063](decisions/ADR-063-public-repository-actions-and-railway.md) | A public repository without its history (`ALaustrup/demiurge-chain`), CI on GitHub Actions, and QOR ID with Postgres and Redis on Railway | Accepted, 29 September 2026, by the project owner. Supersedes ADR-058 and ADR-060, and ADR-015's Fly.io for these three services. Amended by ADR-064 |
| [ADR-064](decisions/ADR-064-the-repository-lives-in-the-qor-matrix-organisation.md) | The public repository, and CI with it, lives in the organisation `QOR-MATRIX` (`QOR-MATRIX/demiurge-chain`), where Actions jobs run; the personal account's are refused by a billing lock | Accepted, 1 October 2026, by the project owner. Amends ADR-063 |
| [ADR-065](decisions/ADR-065-nesting-and-the-cycle-rule.md) | Nesting in `pallet-drc369`: only the owner of both assets, depth 8 and 64 children, cycles refused by a bounded walk (R-2), and a nested asset and the asset holding it held in place | Accepted, 1 October 2026, under the delegation; choices 1, 2 and 5 confirmed by ADR-067 |
| [ADR-066](decisions/ADR-066-buying-what-was-seen-and-the-market.md) | `buy_exact` and `ContentChanged`, the runtime API `Drc369RoyaltiesApi::sale_preview`, QOR ID's `email_leaves_this_machine`, and the Market's first slice (only the holder is offered Clear on a void listing) | Accepted, 3 October 2026, by the project owner |
| [ADR-067](decisions/ADR-067-the-owner-confirms-the-pending-choices.md) | The owner confirms ADR-065's choices 1, 2 and 5, keeps buying by an asset's number beside the Market, and approves the readability check's measuring rule | Accepted, 3 October 2026, by the project owner |
| [ADR-068](decisions/ADR-068-the-devnet-on-railway.md) | The devnet runs on Railway: two private validators and a public RPC node at `rpc.qorsync.dev`, `Demiurge Devnet`, and genesis holding only a sudo account and a faucet of marked test CGT | Accepted, 3 October 2026, by the project owner. Supersedes ADR-015 for the devnet's nodes; accepts ADR-044 |
| [ADR-069](decisions/ADR-069-arqade-the-gaming-platform.md) | ARQADE, the gaming platform: a product under ADR-050 (P7, gate `arqade`), a third web surface amending ADR-011, sign-in by ADR-043, signing by delegated keys with a bridge of its own record, one devnet and one faucet, U-16 for payouts, and gaps G-14 to G-16 | **Accepted**, 4 October 2026, by the project owner, the day it was proposed. Amends ADR-011; accepted with ADR-043 |
| [ADR-070](decisions/ADR-070-game-vaults.md) | Game Vaults: one keyless payout account per published game, derived from its DRC-369 Cartridge and governed by whoever holds it; a payout authority bounded by an on-chain policy; each outcome paid once; loosening and withdrawals delayed; prizes reserved; small payouts accrued | **Proposed**, 4 October 2026. Decision 1 is the owner's direction; decisions 2 to 10 await the owner |

**ADR-042 was accepted on 28 September 2026, ADR-044 on 3 October 2026 through ADR-068, and ADR-043 on 4 October
2026 with ADR-069**, because ARQADE is the browser frontend its decision 6 was waiting for. Its assessment is
[`architecture/HOSTING.md`](architecture/HOSTING.md) §3.

**ADR-046 to ADR-051 are the substrate decisions the six products need** before any of them grows its own
identity, asset format, storage or payment rail, and each product's blueprint in `docs/blueprints/` builds
on them. **ADR-047, the object model and the DRC-369 wire format, and ADR-051, QFX's rendering layer, were accepted
on 22 September 2026**, and **ADR-046, ADR-048, ADR-049 and ADR-050 on 28 September 2026**. Two surfaces already run ahead of them
at the owner's instruction — Qontrol's Projects surface and QFX layer one — and ADR-046 and ADR-051 each say
how the tree differs from them.

Milestones (M1, M2, ...) are defined in [`DIRECTION.md`](DIRECTION.md).

---

**About the records below.** D-000 to D-012 are kept exactly as they were written, which is what a
decision log is for. Two things about them will look wrong to a reader and are not:

- **They call the currency CGT, and so does the project again.** CGT was the ticker when they were
  written; ADR-034 changed it to `DMRG` on 17 September 2026 and ADR-045 changed it back on
  21 September. Both records stand, and neither rewrote what is below.
- **They cite paths under `framework/`, which no longer exists.** That was the custom Rust devnet these
  decisions were implemented in. It was retired and deleted at M3.5 on 20 September 2026, so those paths
  resolve to nothing; the Status column above says which requirement carried to `chain/` and which did
  not. Nothing below is a description of code that runs today.

---

## D-000: CGT has 18 decimals

**Status:** Implemented September 2026. Recorded here for completeness.

CGT was defined at 2 decimals. At that precision the existential deposit (`CGT / 1000`)
evaluated to zero, a 0.001 CGT fee could not be represented, and per-item creator pricing
could not go below 0.01 CGT. 18 decimals is the ERC-20 convention that exchanges, bridges and
wallets assume.

- 1 CGT = 10^18 Sparks. A Spark is the atomic unit.
- Total supply: 13,000,000,000 CGT, fixed. **Superseded by ADR-003**: the base supply is
  100,000,000,000,000 CGT, released over a decay curve, with perpetual issuance (ADR-004). The
  precision in this entry stands; whether 18 decimals remains right is U-1.
- Single source of truth: `framework/primitives/src/denomination.rs`.

---

## D-001: Fees

> **Superseded by ADR-004 (13 September 2026).** The 0.001 CGT fee and the 50/30/20 split are
> withdrawn; burn shares per fee class are open (OPEN-4). The sponsorship principle is retained
> (`economics/CGT.md` §9). Kept below as the record.

### The question

Are transactions feeless, or do they cost CGT? The project previously promised both: feeless
transactions through an "energy" system, and a 0.001 CGT transfer fee with an 80% burn.

### What the code does today

Every transaction consumes 100 energy. Energy caps at 1,000 and regenerates 10 per block
(`framework/modules/energy/src/energy.rs`). No CGT is ever charged. A brand-new account's
regeneration is computed from block 0, so **a freshly generated keypair starts with a full
tank**. Keypairs are free, so today spam costs nothing, and validators earn nothing from
transactions.

### Options

| Option | For | Against |
| --- | --- | --- |
| **A. Purely feeless (energy only)** | Best possible user experience | Spam is free (unlimited new keypairs, each with free energy). No revenue for validators, so no reason to run one. No deflationary sink. |
| **B. Flat CGT fee on every transaction** | Simple. Spam has a price. | Every player must acquire CGT before doing anything, which breaks onboarding for games. |
| **C. Dynamic fee market (EIP-1559 style)** | Prices block space under congestion | Unpredictable costs are hostile to games. Premature at current volume. |
| **D. Energy first, CGT fee fallback** | Keeps "players never see gas" for real users and sponsored games, while spam and heavy use carry a real cost | More rules to implement and explain |

### Decision: D, energy first with a CGT fee fallback

1. **Every transaction has a fee of 0.001 CGT** (10^15 Sparks). This is the base fee; later
   versions may weight it by call type.
2. **Energy pays the fee in full when the sender has enough.** Cost: 100 energy. Cap: 1,000.
   Regeneration: 10 per block. Nothing in CGT is charged.
3. **Energy only regenerates for accounts holding at least 1 CGT.** A new account starts at
   zero energy, not a full tank. This makes a Sybil swarm cost 1 CGT per account, locked,
   instead of nothing.
4. **Developers can sponsor energy** for their players (the existing `Energy::Sponsor` call),
   so a game can give its players a feeless experience without handing them CGT.
5. **The free lane is capped.** At most half of a block's transaction slots may be paid with
   energy. Fee-paying transactions are ordered first. A spammer holding energy can therefore
   slow the free lane but never block paying users.
6. **CGT fees are split: 50% burned, 30% to the block proposer, 20% to the treasury.**
7. If a sender has neither enough energy nor enough CGT, the transaction is not includable and
   never enters a block.

### Why

- The product promise that matters is that *players* never see gas. Sponsorship and the
  energy tier deliver that. It does not require the protocol itself to be free for everyone.
- A cost per Sybil account is the only real spam defence on a chain where keys are free.
- **Validators must be paid by the network they secure.** The proposer share gives them a
  direct reason to include transactions, not only to produce blocks.
- Burning half of every fee is the supply's only sink under a fixed cap (D-002), and it makes
  self-dealing by a proposer (stuffing blocks with its own fee-paying transactions) cost money.
- The treasury share funds the ecosystem from usage rather than from new issuance.

All values are protocol constants in `framework/primitives` and are expected to become
governance parameters once governance can execute changes (it cannot today).

---

## D-002: Staking rewards

> **Superseded by ADR-004 (13 September 2026).** Validators are paid for security work from the
> genesis release curve and the perpetual issuance budget, not from a reserve paying a rate. The
> figures below are withdrawn, and the wording "10% of the remaining reserve per year" breaks ADR-008.
> Kept below as the record.

### The question

Are validator rewards minted (inflation), or paid out of existing supply?

### What the code does today

`ConsensusEngine::distribute_era_rewards` computes a reward of `42` per block, which its comment
calls "42 CGT" but is actually 42 **Sparks** (10^-17 CGT), and adds it to an in-memory stake
figure. **Nothing is ever paid to a balance**, and the figure is lost on restart.

### Options

| Option | For | Against |
| --- | --- | --- |
| **A. Mint new CGT (for example 5% a year)** | Familiar from other proof-of-stake chains | Breaks the fixed 13 billion supply, which is project law. Dilutes every holder indefinitely. |
| **B. Fees only** | No issuance at all | Fee revenue is near zero at launch. Unpaid validators do not run nodes, so there is no network. |
| **C. Emission from a pre-allocated reserve** | Keeps supply fixed, pays validators from day one, predictable | The reserve is a finite pool, so rewards shrink over time |

### Decision: C, emission from the Archon Staking Reserve

1. **20% of supply (2,600,000,000 CGT) is placed at genesis in the Archon Staking Reserve**, a
   protocol account with no private key.
2. **Each era releases a fixed fraction of what remains in the reserve.** The fraction is
   96,214 parts per billion per era, which at 2-second blocks and 14,400-block eras (8 hours,
   1,095 eras a year) compounds to exactly **10% of the remaining reserve per year**.
   - Year 1: about 260 million CGT (2% of total supply).
   - Year 2: about 234 million. Year 10: about 101 million.
   - The reserve decays smoothly and never hits a cliff.
3. **Each era's release is divided among active validators in proportion to total backing
   stake** (own stake plus nominations). A validator's commission is taken from its share and
   the remainder goes to its nominators pro rata.
4. **Rewards are real transfers from the reserve to account balances.** Total supply does not
   change.
5. Validators additionally earn the proposer share of CGT fees (D-001).
6. **Slashed stake is burned.**
7. Stake bonds from real balances and unbonds over a delay (7 days), all via transactions.
   Minimum validator stake: 1,000,000 CGT. Minimum nomination: 100 CGT.

### Why

- The fixed 13 billion supply is the project's stated law (`.cursorrules`). Inflation would
  have to repeal that law. Nothing about securing the chain requires it.
- Paying from a reserve gives validators real income before fees exist, which is the only way a
  new network gets validators at all.
- A proportional release avoids a cliff where rewards suddenly stop, and front-loads rewards
  while the network most needs security, declining as fee revenue is expected to grow.

The per-era rate is derived from the era length and block time. If either changes, the rate
must be recomputed so the yearly figure stays at 10%.

---

## D-003: Genesis distribution

> **Superseded by ADR-003 (13 September 2026).** The base supply is 100,000,000,000,000 CGT, and the
> genesis split is open (OPEN-2). The team vesting terms are withdrawn (U-12). Kept below as the record.

### The question

How are the 13 billion CGT allocated at genesis? Three earlier documents gave three different
answers. None of them was implemented: **total supply is currently zero until something mints.**

### Options

| Option | Split | Assessment |
| --- | --- | --- |
| **A. The "Creation Model"** | Pleroma Mining 40% · Archon Staking 20% · Treasury 15% · Team 15% · Genesis Offering 10% | Sound structure. The 40% in-game rewards bucket is very large and is the pool most exposed to bot farming, because game rewards are triggered by off-chain signals. |
| **B. Treasury-heavy** | Treasury 77% · Staking 15% · Team 5% · Community 3% | Too centralised for a network meant to be credibly neutral, and it was paired with inflation (rejected in D-002). |
| **C. Godmode treasury plus ad-hoc minting** | 1B to an admin account, mint as needed | Not a distribution. An admin key that can create money is incompatible with CGT being money. |
| **D. Revised Creation Model** | See below | Keeps A's published structure and moves 5% from the most gameable pool to governance-accountable treasury |

### Decision: D, the revised Creation Model

| Bucket | Share | CGT | Release rule |
| --- | --- | --- | --- |
| Creator & Player Rewards | 35% | 4,550,000,000 | Released only by protocol programs approved by governance |
| Archon Staking Reserve | 20% | 2,600,000,000 | Emission schedule in D-002 |
| Treasury | 20% | 2,600,000,000 | Governance-executed spending, plus 20% of fees (D-001) |
| Core Team | 15% | 1,950,000,000 | 12-month cliff, then linear over 36 months (4 years total), enforced on-chain |
| Genesis Liquidity & Public Distribution | 10% | 1,300,000,000 | Governance-executed releases for liquidity and public distribution |

Rules that apply to all of it:

1. **The entire 13 billion CGT is created at genesis.** After genesis there is **no mint path at
   all** on a production network. The faucet and admin mint exist only on development networks
   (D-008).
2. **Every bucket is a keyless protocol account.** No private key can spend from any of them.
   Funds move only by protocol rule (emission, vesting) or governance execution.
3. There is no investor bucket. If investors are brought in later, their allocation comes out of
   the Treasury by governance vote, never from new issuance.
4. Starter grants to new users on a public network are paid from Creator & Player Rewards and
   must be gated by a QOR ID attestation, never by "one per keypair", because keypairs are free.

### Why

- A published structure changes as little as possible: only the one bucket with a concrete
  abuse problem moves.
- In-game reward pools are the classic target for bot farms. Shifting 5% of supply into a pool
  that needs a governance vote to spend reduces what a farm can reach.
- Keyless buckets with rule-based release are what make a fixed-supply claim verifiable by
  anyone reading the chain, rather than a promise.
- A 1-year cliff and 4-year vest is the market norm that exchanges and serious holders expect
  of a team allocation.

### Consequence for launch

Governance cannot currently execute anything (`execute_proposal` only returns encoded bytes).
Treasury, Creator & Player Rewards and Genesis Liquidity releases all depend on it, so
**executable governance is a hard prerequisite for mainnet**. It is on the roadmap in M4.

---

## D-004: On-chain transaction nonces

**Problem.** A transaction carries a `nonce`, but the runtime never checks it. Once a signed
transfer leaves the pool, anyone who saw it can submit it again and it executes again. That is a
replay attack on every payment. The launcher currently fills the field from `account_getNonce`,
which is the RPC layer's *request-authentication* counter, a node-local value unrelated to
consensus.

**Decision.** Each account has a consensus-state transaction nonce. A transaction is includable
only if its nonce equals the account's current nonce, and inclusion increments it. RPC request
nonces stay separate and node-local. The next transaction nonce is served by a dedicated RPC
method so clients cannot confuse the two.

---

## D-005: Atomic transaction execution

**Problem.** A transaction's writes go straight to storage as it runs. If a call fails halfway,
the writes before the failure stay. On a network, a proposer that keeps those partial writes and
a node that rejected the transaction compute different state, and the chain splits.

**Decision.** Each transaction executes against a write buffer. On success the buffer commits; on
failure it is discarded. The costs of inclusion (nonce increment, and energy or fee under D-001)
are charged whether or not the call succeeds, and a failed transaction is still recorded in the
block, as on every major chain. A transaction whose inclusion cost cannot be paid is not includable
and changes nothing.

---

## D-006: One state database, scoped state root

**Problem.** The node opens two RocksDB instances, one for the runtime and one for consensus. They
drifted apart and caused a total outage (every transaction failed). Separately, the state root is
a hash over *every* key in the database, including node-local data such as RPC authentication
nonces and indexes, so two honest nodes can never agree on it.

**Decision.** One database per node. The state root covers **consensus state only**: an explicit
list of key namespaces written by transaction execution. Blocks, indexes and node-local RPC data
are stored in the same database but outside the root.

---

## D-007: Block production and finality

> **Superseded by ADR-013 (14 September 2026).** Block production and finality come from standard
> Polkadot SDK components. The slot schedule below runs in the custom devnet until that chain is
> replaced; the custom BFT finality it planned will not be built. Kept below as the record.

**Problem.** Proposer selection iterates a `HashMap`, whose order differs between processes, so
two nodes pick different proposers. Blocks are signed before their state root is filled in, and
the signature is never sent to peers. Genesis validators are skipped. Produced blocks are never
broadcast.

**Options considered.** (a) Keep a single leader: not a network. (b) Slot-scheduled production
(Aura style) with a finality gadget added afterwards. (c) A full round-based BFT engine
(Tendermint or HotStuff style) from the start.

**Decision.** (b) now, then BFT finality. Time is divided into slots of one block time. Each slot
has one proposer, chosen deterministically and weighted by stake from the genesis validator set.
The proposer seals the finished header, state root included, and broadcasts it. Nodes import a
block only if it comes from that slot's scheduled proposer, extends their current head, and
re-executes to the same state root. Nodes that fall behind request missing blocks from peers.

A slot schedule is required by every BFT design anyway, and it produces a working multi-validator
devnet soonest. The **target** is deterministic finality: a two-thirds stake-weighted vote on each
block (M3). Until that exists, the network is a devnet and makes no finality claim.

---

## D-008: Direct-write RPC endpoints are development-only

**Problem.** Many RPC methods change state by writing storage directly, outside any block:
`balances_transfer`, `balances_claimStarter`, `admin_mintCgt`, the `consensus_*` staking and
registration methods, and the `drc369_*` write methods. On a network each write exists on one node
only, so that node's state root diverges and every block it produces is rejected.

**Decision.** These methods work only on a node started with `--dev`, which also refuses to join a
multi-validator network. On every other node they return an error directing the caller to submit
a transaction. `--faucet` and `--godmode-pubkey` require `--dev`. Each capability is rebuilt as a
transaction type as the roadmap reaches it.

---

## D-009: Domain-separated transaction signatures

**Problem.** A transaction signature covers `nonce || from || data` with no chain identifier, so a
transaction signed for a testnet is valid on any other network where the same key holds funds.

**Decision.** The signing payload gains a domain tag and the chain's genesis hash. This is a
breaking change to the transaction format, so it is made once, before any public testnet (M3),
together with the launcher and the reference signer.

---

## D-010: `modules/zk` is removed; CVP carries no security claim

Every verifier in `framework/modules/zk` returns `Ok(true)`. Nothing depends on it. It is removed
rather than left for someone to trust.

Since M1 the node runs consensus with CVP **disabled**. A block's proposer and the validators
importing it computed CVP's epoch state differently, so CVP cannot sit on the consensus path until it
is deterministic. Blocks carrying a CVP proof root are rejected. Consensus-Verified Polymorphism's default proof is computed
entirely from public inputs and proves nothing. Until real proofs are the default, no document may
describe CVP as a security property.

---

## D-011: Legacy clients are frozen; the launcher is the client

> **Amended by ADR-011 (13 September 2026).** Two web surfaces are added: a public viewer with no
> authentication, and a thin remote console behind QOR ID. Creation, publishing and Mesh seeding stay in
> the launcher. The frozen applications listed below remain frozen.

The QOR Launcher (`tools/qor-launcher`) is the platform and the only supported client. The web hub,
wallet extension, Sophia, NFT portal, marketing site, CLI, TypeScript SDKs and the Unreal client were
built against the pre-realignment protocol. They are **frozen**: not maintained, not used as
references, and not described as working until each is explicitly brought back into scope here.

The wallet extension's key derivation is non-standard, so keys created there cannot be recovered by
any other wallet. **No value should be held in it.**

---

## D-012: Documentation governance

1. Only documents indexed in [`docs/README.md`](README.md) are current.
2. The pre-realignment documentation (244 files) was **deleted** on 2026-09-13. It is never restored
   from git history, consulted, quoted or used for any reason.
3. [`DIRECTION.md`](DIRECTION.md) is the only roadmap. This file is the only decision log.
   [`economics/CGT.md`](economics/CGT.md) is the only economic model. None of them is duplicated
   anywhere else.
4. The code is the source of truth for what runs. A change that alters behaviour updates the affected
   current document in the same change. A document that states direction says plainly where the code
   does not follow it yet.
5. A fact not found in a current document is verified in the code and then written into a current
   document. It is never taken from deleted documentation or old git history.

*Amended 2026-09-13, when the owner's direction was recorded as ADRs:*

6. From ADR-001 onwards, each decision is a record of its own in [`decisions/`](decisions/README.md),
   indexed in this file. A record is never edited to change its decision; a later record supersedes
   it and both say so. Entries D-000 to D-012 stay in this file and are marked when superseded.
7. Values the economic model leaves undecided are listed in
   [`economics/OPEN_QUESTIONS.md`](economics/OPEN_QUESTIONS.md). No code, genesis file or document
   gives them a value until the owner decides one, the value is written into `CGT.md`, and an ADR
   records the reasoning.
