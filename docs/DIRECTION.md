# Direction

**The only roadmap, and the canonical definition of what Demiurge is.** This document settles "what are
we building". Decisions behind it are indexed in [`DECISIONS.md`](DECISIONS.md). The economic model is
[`economics/CGT.md`](economics/CGT.md). What the chain does today, exactly, is
[`protocol/PROTOCOL.md`](protocol/PROTOCOL.md). How the current code measures up against all of it is
[`audit/RECONCILIATION.md`](audit/RECONCILIATION.md).

**Last updated:** 2026-09-22

---

## 1. What Demiurge is

Demiurge is a creative engine built on a purpose-built Substrate L1. Four primitives are native to the
protocol rather than bolted on as contracts, and everything else is a surface over them.

**The chain** has fast finality, on-chain governance, and fee sponsorship so that end users can
transact without holding tokens.

**DRC-369** is Demiurge's asset standard. Its assets are stateful, nestable and royalty-bearing. An asset
carries state, history, physics properties, nested components, rental terms and fractional ownership. It
is a living object, not a receipt pointing at a file.

**QOR ID** is identity: an unexportable vault, session keys and scoped delegation. It is one identity
across the launcher, the web and agents.

**CGT**, the Creator God Token, is the settlement currency for every flow of value in the system.

People reach these primitives through client surfaces:
- **The QOR Launcher**, the desktop application, and the host for everything built above it. Its surfaces
  today are Gate, Nexus, Vault, Inventory, Chain, Gates, Projects and Settings; Library, Social (named Agora until 2026-09-28) and Mesh
  are placeholders, Studio is a link out, and Market does not exist yet (`SYSTEMS.md`). Products it
  launches will be separate processes it supervises and signs for (ADR-046, Proposed).
- **A web surface** in two parts, described in §5.

**Six products are built on that substrate**, each with a blueprint in
[`blueprints/`](blueprints/ECOSYSTEM.md) and a track of its own in §7:
- **Qontrol**, version control for creative work, git-compatible on disk (P1). Its first surface,
  Projects, is built, line-by-line diffs included since 23 September 2026, and since 22 September 2026
  it mints a commit as a DRC-369 asset (M4.1).
- **QFX**, the launcher's living visual layer, authored by creators and paid for when used (P2). Layer
  one's first slice, the backdrop, is built.
- **QOR Engine**, a creation tool that is a custom Godot build tracking upstream: "QOR Engine, built on
  Godot" (P3).
- **GNOSIS**, music production whose project is a mergeable tree of text files, versioned by Qontrol (P4).
- **Market and Library**, where work is listed and bought in CGT, and where what you bought installs,
  patches and launches (P5, carrying L7.1 and L7.2).
- **Stream**, a placeholder name for music distribution and licensing over the Mesh (P6). It does not
  exist before M8.

**One substrate, six apps.** Every product signs in with QOR ID, stores creations as DRC-369 assets,
versions them with Qontrol, distributes through the Mesh and settles in CGT. A product that would need its
own identity, asset format, storage or payment rail has found a substrate gap: it is recorded in
[`blueprints/ECOSYSTEM.md`](blueprints/ECOSYSTEM.md) §4, never filled locally. The six substrate
decisions they share are ADR-046 to ADR-051: ADR-047 (the asset format) and ADR-051 (QFX's rendering
layer) were accepted on 22 September 2026, and the rest are Proposed.

**Scope discipline.** Anything proposed later must justify itself against one of the four primitives.
A proposal that strengthens none of them is out of scope. The six products are held to the same rule:
each is a surface over the primitives, not a seventh thing beside them.

The product name is **Demiurge-Cloud**. The protocol name is **Demiurge**. The repository is
`QOR-MATRIX/demiurge-chain`, public since 29 September 2026 (ADR-063) and in the organisation since 1 October
(ADR-064); `ALaustrup/demiurge-cloud` is the private archive of everything before it.

### The base layer today

The sentence "built on a purpose-built Substrate L1" **is true of the code as of 17 September 2026.**
This section used to say it was not; that was written before `chain/` existed and is corrected here.
- **The chain is `chain/`**, a Substrate L1 on the pinned Polkadot SDK
  ([ADR-013](decisions/ADR-013-polkadot-sdk-migration.md)). The custom Rust implementation that preceded
  it, in `framework/`, was retired and deleted at M3.5 on 20 September 2026.
- **It has the properties this section once said were missing.** Aura produces blocks and GRANDPA
  finalises them (ADR-018, ADR-019), and the validator set comes from governance through
  `pallet-session` and `pallet-validator-set` rather than from a configuration file (ADR-020). Two
  validators agree, finalise, and one recovers after being killed — 13 checks.
- **What is still missing is above the base layer**, and each for a recorded reason: no transaction
  payment (OPEN-4), no issuance (OPEN-1), no treasury (OPEN-2), no assets beyond M4.1's first pallet (M4), and no sponsorship (U-4).
- The module-by-module mapping is
  [`architecture/MIGRATION_INVENTORY.md`](architecture/MIGRATION_INVENTORY.md). **Seventeen of its
  twenty questions are decided** (ADR-017 to ADR-032, and Q-17 by ADR-041). Three were added on 22 September
  2026: Q-18, whether eight royalty recipients per asset is enough, decided on 28 September 2026 (ADR-057,
  eight stays); and, open until before the freeze, Q-19, whether an asset's root depends on its files alone or also on the commit it was minted from; and
  Q-20, whether the media-type table that labels each file is part of that root. The owner confirmed ADR-018 to
  ADR-032 on 17 September 2026. The hold AGENTS.md §7 placed on migration code lifted on 22 September 2026,
  when the owner acknowledged a ten-line summary of its DRC-369 section and M2.1 was ticked against it.

## 2. The innovation budget

Work is classified by its failure mode, not by its ambition
([ADR-001](decisions/ADR-001-innovation-budget.md)).

**Innovate hard where failure is loud and recoverable.** That means asset semantics, remix royalty
mechanics, the creator economy, agent rails, creative surfaces and the incentive design of the Mesh.
Nobody has done these well. A mistake there shows quickly and can be fixed without touching anyone's
keys or balances. This is where Demiurge is genuinely different.

**Stay boring on purpose where failure is silent and permanent.** That means consensus, key custody,
vault design, genesis allocation, the issuance schedule, the address format, the storage layout and
upgrade paths. Proven Substrate components are used there.

Being conservative in the places that cannot be repaired is not timidity. It is what buys the freedom
to be radical in the places that can.

## 3. The economy, in brief

The full model is [`economics/CGT.md`](economics/CGT.md). Values it leaves undecided are in
[`economics/OPEN_QUESTIONS.md`](economics/OPEN_QUESTIONS.md) and are not to be invented.

**Where value comes from.** CGT derives its value from demand to spend it, not from demand to hold it.
Every fee, reward, sink and incentive must pass one test: *if CGT could never be traded for dollars,
would it still be worth holding?*

**Supply.** The base supply is one hundred trillion CGT. The large majority is allocated at genesis and
released over a multi-decade decay curve to validators, seeders, the treasury and the creator and player
rewards pool.

**Issuance and burn.** A small perpetual issuance, targeted under one percent a year, pays for the
Mesh's storage and bandwidth and for validator security. A share of every fee is burned as its
counterweight. Issuance and burn are one balance mechanism and are never described apart.

**The Mesh.** Installing the launcher makes a machine a potential node. Seeders are paid for storage
and bandwidth the network actually consumes, verified against on-chain fingerprints. There is no
artificial difficulty.

**Demand sinks.** Five structural sinks create the demand to spend CGT: Mesh hosting, access gating,
staking for distribution, agent compute, and escrow and reputation bonds.

**Display.** Users read a legible display unit, working name "credits", never raw CGT. The display unit
is not a peg.

**Language.** Earnings language means payment for work, hosting and licensing, and nothing else.

## 4. The first release

The first release makes the **asset primitive** excellent, not any single vertical
([ADR-009](decisions/ADR-009-universal-minting.md)).

**Universal minting is the floor.** Anyone uploads any file and mints it, as a single asset or as a
collection, on chain, through Studio or the SDK.

**Game assets are the flagship proof.** Statefulness, nesting, physics properties, remix royalties and
sponsored fees all matter at once in a game asset. That makes it the clearest demonstration of what
DRC-369 does that other standards do not. Music is the strongest second: it has the cleanest per-play
micropayment story, but exercises less of the standard.

**SDKs ship alongside,** so that third-party wallets, marketplaces and engines can support DRC-369 from
the start. Demiurge is a standard others adopt, not a walled garden. The DRC-369 wire format and token
identity are settled before any SDK is published, because publication makes them expensive to change.

### Agent rails

The first release ships the agent *surface*, not agent products
([ADR-010](decisions/ADR-010-agent-rails.md)).

**Delegated authentication, not stored credentials.** The agent generates its own key and QOR ID
authorises it with scoped, revocable permissions (ADR-014); the agent never holds the user's identity. Capabilities are enumerable per operation, spend
caps are enforced by the protocol, and any single agent can be revoked without affecting the others.

**One MCP server.** Demiurge ships an MCP server rather than a bespoke plugin per LLM vendor. That turns
"any LLM can navigate the ecosystem" into a documentation problem rather than an integration problem.

**The reference capability is generate-and-mint.** A user's model creates work and mints it as DRC-369 on
chain, within limits the user set.

Autonomous agents with their own identity and wallet, such as licensing bots, shops and level testers,
come later on the same rails.

## 5. The web surface

There are two web surfaces, and they are never merged
([ADR-011](decisions/ADR-011-web-surface.md)).

**The public viewer needs no authentication.** Every published work gets a shareable, indexable page
showing the asset, its provenance chain, its creator, its licence terms and its price. No wallet is
required. This is how Demiurge content escapes the launcher and reaches people who have installed
nothing, and it is the primary growth surface.

**The remote console requires QOR ID.** It covers wallet operations, CGT transfers, asset management
and project status: read, manage, transact.

Creation, publishing and Mesh seeding stay in the launcher, where the local vault and node live. The
console is kept thin so that it never becomes a competing second product.

**Visual identity.** Every surface keeps Demiurge's established identity: professional and restrained,
dark and confident, with typographic hierarchy and generous spacing doing the work rather than
decoration. No neon gradients, glows, particle backgrounds or cyberpunk styling. When in doubt,
subtract. Where current code departs from this, the reconciliation report records it (§7 there).

**One thing moves behind the launcher's interface: QFX's canvas** (ADR-051, accepted 22 September 2026). Its
rule, in the owner's words: **the design system governs the default theme and the chrome; QFX governs the
canvas; the default must still pass `check-design.mjs` unchanged.** Layer one's backdrop (P2.1) is a slow drift
in the theme's own colours under a scrim the chrome owns, live by default at a restrained amplitude, still under
reduced motion, and off at one switch. Nothing above it — no text, control or status colour — is ever the
canvas's. The dreamlike look is a creator's choice, never the default.

## 6. Where things stand

Verified 13 to 15 September 2026, and the rows the chain work changed were re-verified on 20 September
2026, after M3.5 retired and deleted the custom devnet. The launcher and product rows were re-verified on
21 September 2026. Evidence is in
[`audit/RECONCILIATION.md`](audit/RECONCILIATION.md), which is a dated audit kept as written, and in
[`HANDOFF.md`](../HANDOFF.md) §1 for anything later than it.

| Component | State |
| --- | --- |
| Chain (`chain/`) | **The only chain** (ADR-013, ADR-032). A Substrate L1 on the pinned Polkadot SDK: Aura authors, GRANDPA finalises, and the validator set comes from governance through `pallet-session` and `pallet-validator-set` (ADR-018 to ADR-020). Since 2026-09-22 it mounts `pallet-nfts` and `pallet-drc369` (M4.1, ADR-052), and `pallet-utility` with only `batch_all` reachable, so several assets move in one all-or-nothing transaction (M4.6, ADR-053); `spec_version` 3. Two validators agree, finalise, and one catches up after a restart (13 checks, re-run 2026-09-22). 74 workspace tests pass with the runtime wasm built. No transaction payment, no issuance and no treasury, on purpose (OPEN-1, OPEN-2, OPEN-4). Local use only; nothing is deployed. |
| Chain (`framework/`) | **Retired and deleted at M3.5, 20 September 2026**, after everything referring to it was rewritten. It was custom Rust with no finality, and untrusted: its non-strict Ed25519 verification accepted signatures nobody made for small-order keys (R-1). Nothing in it is carried forward, and CI fails if the directory returns. |
| CGT on chain | 18 decimals, decided (ADR-035), with an existential deposit of 100 CGT (ADR-036). **`chain/` declares no total-supply constant at all**, because issuance is OPEN-1 and the genesis split is OPEN-2; the superseded 13 billion figure went with `framework/`, and left the launcher on 2026-09-20, which is the removal ADR-003 asks for by name: the launcher's ceiling on a typed amount is now named for what it is and is not a supply, and the Settings surface shows no total (`RECONCILIATION.md`, "Remediation since this audit"). No CGT fee is charged, and there is no path that creates CGT outside a development chain specification. The ticker is `CGT` (ADR-034), and `chain/` was written with it from the start, which is requirement R-4. |
| Economic model | Decided in direction (ADR-002 to ADR-008). Rates, split, curve and burn shares are open. Nothing implemented. |
| DRC-369 | **M4.1's chain half exists, since 2026-09-22** (ADR-052): `pallet-drc369` over `pallet-nfts` mints an asset carrying ADR-047's 41-byte content reference and its pinned commit, revises it until a one-way switch makes it permanent, creates one singles collection per creator, and lists what an owner holds, from storage and through a runtime API. Only it can create an asset. Its deposits are placeholders (U-14) and its weights are placeholders owed to M7.2. Nesting, state and XP, physics, remix, royalties, rental and fractions are not started. **The launcher mints a Qontrol project's commit and lists what an account holds from chain storage** (M4.1's launcher half, the same day). The custom chain's module — mint, transfer, state and XP, nesting, burn, approvals and one royalty setting, none of which moved CGT — was deleted with `framework/` and is not carried forward. What it must do, including requirement R-2 (refuse nesting cycles), is in the migration inventory and in M4.5. The wire format is decided (ADR-047, accepted 2026-09-22, M2.3), and M2.1 was ticked the same day against the owner's review of a ten-line summary, so nothing stands before M4. |
| QOR ID (`services/qor-auth`) | Username and keypair sign-in with JWTs. Changes on 2026-09-14: committed secrets and the seeded admin removed (rotation confirmed by the owner); agents register their own keys and QOR ID only authorises them (ADR-014); agent endpoints scoped to their controller; key links authenticated and proven; logout and session revocation take effect at once; registration creates no CGT and calls no chain (migration inventory R-3); every mounted route does what it reports or refuses, including email verification, password reset and the admin routes (SECURITY.md). On 2026-09-15: email through Resend (sent live from `demiurge.cloud` to Resend's test address; delivery to a real inbox unconfirmed), password resets that end every session, single-use hashed backup codes, and sign-in that refuses every failure alike; a new verification link for an account whose link lapsed, and backup codes a signed-in account can regenerate with its password; an account can add or change its email address with its password, confirmed by the new address, announced to the old one, and keeping its backup codes; the lockouts that remain are accepted and named in SECURITY.md; emails rewritten in the design system with a plain-text part; the pages the links open spend a token only when their button is pressed, so a mail scanner cannot burn one; and signed bounce and complaint reports from Resend stop sending to an address that cannot or should not receive mail (not yet received live), with an admin-only, audited route to take a mark off; no token, key, link or address may reach a log at any level, checked at runtime and in CI, and no SQL statement may be built from values, checked at build time (`alpha.no-secrets-in-logs`, `alpha.parameterised-sql`); QOR ID serves its own pages on its own subdomain (ADR-015, clarified). A password-only account will have no chain identity until it proves a key (ADR-017; not implemented until the Substrate work). Built, tested and checked end to end against Postgres 16 and Redis. Capabilities and spend caps are recorded, not enforced (Q-9). |
| QOR Launcher (`tools/qor-launcher`) | Phase 1: vault, QOR ID sign-in, CGT send and receive, chain view, settings, five themes, accessibility settings. Since 2026-09-20 it is a `subxt` client of `chain/`, building every call from the metadata that node serves (ADR-040), with an Sr25519 vault and SS58 addresses (ADR-039). Since 2026-09-21 it also has a Projects surface, Qontrol's first slice (P1.1, done on 2026-09-23 when its line-by-line diffs arrived; since 2026-09-26 it branches, switches, discards, commits the ticked files only, holds back a huge or credential-shaped file until it is accepted, and its helper is bundled so an installed launcher can commit: P1.2), and QFX layer one's backdrop with an Ambience setting (P2.1). Since 2026-09-22 Projects mints a commit as a DRC-369 asset, Inventory lists an account's assets from chain storage as cards (M4.1, P2.8), and each card's own menu sends several of them to another account in one all-or-nothing transaction (L4.4, L4.5, M4.6). Since 2026-09-23 the trade window has two sides with a lane between them and offers the accounts this machine has traded with before, and Sell drafts a listing that stays on this machine and says so (L4.6, half of it, unticked). Since 2026-09-28 there is no lock screen: the vault's key is in the operating system's keychain and nothing is asked to open it, QOR ID never blocks the launcher and has no password route, and older vaults move once (ADR-056, superseding ADR-055's Windows Hello the same day). 162 host tests pass (2026-09-28), plus three that need a running development node — a transfer, a mint and a trade; the qontrol-git helper's 16 pass; the eight view checks — design 8, accessibility 41, gates 34, Projects 104, Inventory 127, the Gate 65, contrast 54 and readability 281 — pass (2026-09-28), with two intermittent failures recorded in `HANDOFF.md` §4 item 19. **Readability, since 2026-09-22:** the interface had been painted beneath the QFX backdrop's scrim, which made most of the launcher unreadable with the backdrop on; it is lifted above it, three colours outside the ink ramp are raised, and `check-readability.mjs` measures every run of text as painted on every screen in every theme. Gate, Nexus, Vault and Inventory are built. Library, Social and Mesh are placeholder pages; Studio is an external tile; Market does not exist. |
| Products (P1 to P6) | Six blueprints and six Proposed substrate records (ADR-046 to ADR-051). Two first slices are built, both inside the launcher: Qontrol's Projects surface, with its diffs since 2026-09-23, and QFX layer one. QOR Engine, GNOSIS, Market and Stream have not started. None is a product yet. |
| SDKs, web apps, CLI | Frozen. None produces a transaction the current node accepts. |
| Agent rails, MCP server, public viewer, remote console | Not started. |
| Production (`demiurge.cloud`, `rpc.demiurge.cloud`) | Offline. |
| Audits | None. Nothing here should hold real value. |

## 7. Roadmap

Ordered by dependency. A chain milestone (M) is not started until the one before it is done. Three kinds
of track run alongside the chain milestones rather than after them:
- the **security track** (§7.1), which does not depend on the chain;
- the **launcher track** (L), where each milestone names the chain milestone it depends on;
- the **product tracks** (P1 to P6), one per product, where each item names what it depends on.

Release gates are defined in [`GATES.toml`](GATES.toml).

### M0: Honest single node (done, 2026-09-11)

Transactions enter blocks through the pool. Mint endpoints require signatures. CGT is 18 decimals.
`qor-auth` builds. Launcher phase 1 is complete.

### M1: Multi-validator devnet on the custom chain (done, 2026-09-13)

On-chain nonces, atomic execution, one state database with a scoped root, a deterministic stake-weighted
proposer, sealed blocks, gossip, import rules, catch-up sync, and dev-only direct writes (D-004 to
D-008). Verified live with three validators. Its behaviour becomes the acceptance tests for M3
([`MIGRATION_INVENTORY.md`](architecture/MIGRATION_INVENTORY.md) §9).

### 7.1 Security track (started 2026-09-14, ahead of M2)

These are defects in active code that would matter the moment anything is deployed. They are not
migration work. Each carries forward as a requirement for the Polkadot SDK chain and QOR ID. Details are
in [`SECURITY.md`](../SECURITY.md).

1. [x] Remove the committed token secrets and database credentials from configuration. **Rotating them
   is the owner's action**; the values remain in git history.
2. [x] Remove the seeded admin account and its recovery code (migrations 008 and 010).
3. [x] Refuse the Balances self-transfer that created CGT.
4. [x] Authorise DRC-369 mint, and require owning the parent to nest.
5. [x] Stop `qor-auth` generating, storing and returning agent private keys (ADR-014; migration 011).
6. [x] Ownership checks on the agent endpoints, and a signed, bound wallet link, in `qor-auth`.
7. [x] Domain-separate QOR ID challenge signing in the launcher and `qor-auth`. A key signs
   `demiurge:qor-id:challenge:v1:` followed by the challenge, never the bare challenge. The launcher
   signs nothing that is not in the service's challenge format.

Two defects in the custom chain are **not patched**, by the owner's instruction. They are requirements for
the Polkadot SDK chain in the migration inventory: strict signature verification (R-1) and refusing
nesting cycles (R-2).

### M2: Migration review and base-layer decisions (done, 2026-09-22)

1. [x] The owner reviews [`MIGRATION_INVENTORY.md`](architecture/MIGRATION_INVENTORY.md).

   **Ticked on 2026-09-22, against a summary, and the tick says so.** The owner's review was of a ten-line
   summary of the inventory's DRC-369 section, pasted at the top of the 22 September report and
   acknowledged by the owner the same day. It was not a reading of the whole document. The ten lines are
   kept verbatim in the inventory's "The owner's review" section. The review carried one item forward: the
   eight-recipient royalty bound (ADR-047, open item 1; inventory Q-18), which `beta.royalty-recipients`
   counts, so the format cannot be frozen while it stands unexamined.
2. [x] The inventory's open questions Q-1 to Q-16 are decided and recorded as ADRs (ADR-017 to ADR-032, 15 September
   2026). They cover:
   - standalone chain or parachain; block production; how the validator set is chosen; governance;
   - the Polkadot SDK release; the key scheme and address format;
   - the DRC-369 ownership ledger; agent delegation; QOR ID on chain; provenance for the viewer;
   - accounts that hold nothing; the existential deposit; fungible game items;
   - how a password-only account relates to chain identity;
   - where the new chain lives in the repository, and its names.
3. [x] The DRC-369 wire format decisions that FRAME forces are taken here, because the SDK depends on
   them: field bounds, token identity, the physics fixed-point encoding, and the rental time unit.

   **Ticked on 2026-09-22: ADR-047, accepted by the owner that day, takes all four** — bounds for every
   field, token identity as `(CollectionId, ItemId)` at `u32`/`u32`, physics as `FixedU128`/`FixedI128`
   rounded half-to-even at 10⁻⁶, and the rental time unit as the block number — together with the
   content fingerprint (a BLAKE3-256 manifest root, its algorithm tagged) and a mint that pins a commit.
   The owner answered its three reserved questions as recommended. This decides the format; it does not
   freeze it. `beta.wire-format-frozen` still asks for the format to be frozen before any SDK is
   published, and M4.1 is where it first meets code.

### M3: The base chain on the Polkadot SDK

1. [x] A node and runtime built from standard components: block production and finality, networking,
   storage, balances, and sudo on development and test networks only (ADR-021, ADR-037).

   **Narrowed on 2026-09-18, by the owner's decision.** Transaction payment and treasury were in this
   item and have moved to **M6.4 and M6.5**, where the economic mechanisms live. They are not dropped and
   nothing about them changed; they moved because each needs a value this project has not decided.
   Transaction payment needs a `WeightToFee`, which is OPEN-4, the fee classes and burn shares; treasury
   needs OPEN-2, the genesis allocation split. AGENTS.md §5 forbids inventing either. **The owner's
   reason for moving them rather than deciding under milestone pressure:** OPEN-2 and OPEN-4 are two of
   the most consequential economic decisions in the project, and a milestone wanting to tick is the wrong
   thing to decide them for.

   **Started 2026-09-17**, once the owner confirmed ADR-018 to ADR-032. The runtime
   crate exists at `chain/runtime` (ADR-032) with system, timestamp, Aura, GRANDPA, balances and
   feature-gated sudo, and the node at `chain/node`. **The chain produces and finalises blocks** (run
   2026-09-17): Aura authored and GRANDPA finalised, which is the first finality the project has had.
   `pallet-session` and `pallet-validator-set` are mounted, so **the validator set comes from governance,
   not from the chain specification** (ADR-020), verified by reading `Session::Validators` from the
   running chain.

   Under the narrowed scope, what remains for this item is the standard components themselves, which are
   in place and running. **Transaction payment is deliberately absent** until OPEN-4 decides fee classes and
   burn shares: a `WeightToFee` is exactly the kind of value AGENTS.md §5 forbids inventing. Treasury
   likewise waits on OPEN-2.

   **Ticked on 2026-09-20**, against the tree rather than against this text. Each component was read in
   `chain/runtime/src/lib.rs`'s `construct_runtime!` and matched to evidence: block production is
   `pallet_aura` and finality `pallet_grandpa`, both exercised by
   `chain/scripts/check-two-validators.mjs` (13 of 13 — two validators author, hold the same block hash
   at a common height, and GRANDPA finalises); networking and storage are exercised by the same run's
   last three checks, where a killed validator restarts, catches up and agrees about the blocks made
   while it was away; balances is `pallet_balances`, covered by `chain/runtime/tests/acceptance.rs`;
   and `pallet_sudo` is behind a `sudo` feature that is **off by default** in both
   `chain/runtime/Cargo.toml` and `chain/node/Cargo.toml`, which is what "development and test networks
   only" has to mean in a build (ADR-037). 51 workspace tests pass with the runtime wasm built.

   **It is ticked under the narrowed scope and not under the scope it was written with.** Transaction
   payment and treasury are M6.4 and M6.5 and are counted by `beta.chain`; no gate loses them by this
   tick.
2. [ ] A chain specification at the base supply of one hundred trillion CGT. Until OPEN-2 is decided,
   genesis pools on development networks use values clearly marked as placeholders.
3. [x] The M1 acceptance tests pass on the new chain, plus a test refusing any self-transfer that creates
   CGT.

   **Narrowed on 2026-09-18, by the owner's decision, the same way M3.1 was.** As written this item
   reached the migration inventory §9 in full, which also lists acceptance tests for DRC-369 and
   requirement R-2, for agent session keys, and for the energy module. **None of those subjects exists
   yet**, so ticking the item as written would have claimed coverage of code that is not there. Each has
   moved to the milestone where its subject lives: **DRC-369 and R-2 to M4.5**, **agent session keys to
   M5.6**, and **energy to M6.6**, with the fee-class work under OPEN-4 that replaces it. Nothing is
   dropped.

   What this item now covers, and what is done: `chain/runtime/tests/acceptance.rs` covers the balances and existential
   deposit rules, the self-transfer defect by all three routes into the pallet, that producing blocks
   creates no CGT, and requirement R-1 (closed by ADR-038 as met by standard behaviour).
   `chain/runtime/tests/block_import.rs` adds the harness: producer and importer reaching the same state
   through real blocks with a valid Aura digest, a block with a tampered state root or extrinsics root
   refused, a run of blocks importing in order, and replay refusal with real signed extrinsics.
   `chain/scripts/check-two-validators.mjs` closes the rest: two validators agreeing on a block, GRANDPA
   finalising, and a killed validator catching up and agreeing about the blocks it missed. 13 checks,
   run 2026-09-18. It also exercises the parent-hash rule, which **cannot** be covered from inside the
   runtime, because `execute_block` initialises before it checks; that is the node's job.

   Everything §9 lists for the behaviour M1 actually established — blocks, transfers, nonces, import,
   finality and multiple validators — is covered, which is what this item now means.
4. [x] The launcher's chain client is rebuilt against standard extrinsics. The vault's sealing is unchanged; its key derivation moves to Sr25519 (ADR-023).

   **Both halves are done.** The key half landed on 2026-09-19 (L3.2, ADR-039): the vault derives Sr25519
   keys exactly as `sp-core` and Polkadot.js do, shows SS58 addresses and signs under the chain's signing
   context, with sealing and custody unchanged, as ADR-023 said they would be. The client half landed on
   2026-09-20 (L3.1, ADR-040): transactions built and signed from the node's own runtime metadata, the
   custom RPC vocabulary gone, and a transfer proven end to end against a development node — approved in
   the host dialog, signed in the vault, submitted, and finalised by GRANDPA.
5. [x] The custom chain in `framework/` is retired, and `PROTOCOL.md` is rewritten to describe the new chain.

   **`PROTOCOL.md` was rewritten on 2026-09-18**, and `framework/` was deleted on **2026-09-20**. The
   retirement was done in two steps on purpose: everything that referred to it first, verified with the
   tree still present, and then the deletion on its own, because that is the step that is not cheap to
   undo. 150 files, 49,058 lines.

   What the reference work covered: AGENTS.md §4, §7 and §8 and `.cursorrules`, which now say there is one
   chain and point precision at `chain/runtime/src/denomination.rs`; CI, where the reference job became a
   `chain/` job that **is** a quality gate and the security job now fails if the directory returns;
   `HANDOFF.md` §2.0, which was a two-chain table and is now a record of what went and what remains;
   `scripts/run-local-stack.md`, whose chain sections were written for the devnet; and the migration
   inventory's requirements, which outlived their subject — **R-1** closed as met by standard behaviour
   (ADR-038), **R-2** moved to M4.5 with the pallet that will own it, **R-3** closed in QOR ID, **R-4**
   met by `chain/` having been written with `CGT` from the start.

   **Nothing it established is lost**: its behaviour is in `chain/`'s acceptance tests and the
   two-validator script, its module-by-module mapping is in the inventory, and what it did on the wire is
   in `PROTOCOL.md`'s history. Its 285 tests passed on their last run, the day it went. It was untrusted —
   non-strict Ed25519 verification, accepting signatures nobody made for small-order keys — so nothing in
   it is copied forward, and CI fails if the directory returns.

   **Not deleted with it, deliberately:** the build and deploy files that referred to it — `Dockerfile`,
   `docker/Dockerfile.node`, `docker/docker-compose.testnet.yml`, `config/production/demiurge-node.service`,
   `testnet/systemd/*.service` and `fly.toml`. They were already recorded as non-working (`HANDOFF.md` §5)
   and are a separate cleanup; `fly.toml` in particular is to be rewritten for `chain/` under ADR-015
   rather than removed.

### M4: The asset primitive

The core of the first release.
1. [x] DRC-369 ownership, mint, transfer, collections, owner enumeration and a content fingerprint, with
   events and runtime query APIs.

   **Ticked on 2026-09-22, against the tree, part by part** (ADR-047, ADR-052). *Ownership* is
   `pallet-nfts`, mounted as `Nfts`. *Mint* is `Drc369::mint`, signed, to the signer, in the signer's own
   collection. *Transfer* is `Nfts::transfer` and its approvals, the only `pallet-nfts` calls the runtime's
   call filter lets through, so `pallet-drc369` is the only way an asset is created. *Collections*: one
   singles collection per creator, created by the first mint. *Owner enumeration*: `pallet-nfts`'s own
   `Account` index, read from storage by the launcher. *Content fingerprint*: ADR-047's 41-byte reference,
   BLAKE3-256 of the manifest, tagged; and the pinned commit beside it. *Events*: `Minted`, `Revised`,
   `Locked` and `SinglesCollectionCreated`, each carrying the reference. *Runtime query APIs*: `Drc369Api`,
   `assets_of` and `asset`. Evidence: 13 pallet tests (the five the owner named shown to fail first against
   planted faults), 6 runtime tests (the call filter shown to fail first), then 70 workspace tests with the wasm
   built, two validators 13 of 13, and a mint from the launcher finalised on a development node and found in
   the owner's enumeration.

   **Two things in it are placeholders, and neither is a decided value.** The deposits a mint holds are
   derived from the existential deposit by ADR-030's arithmetic and opened as U-14. The weights of
   `pallet-drc369`'s calls are assembled from `pallet-nfts`'s reference weights and are owed to M7.2, as
   every unbenchmarked weight is.
2. [ ] Authorised nesting with bounded depth, state and XP, and royalties with remix royalties settled in
   CGT.

   **The royalty half is built (29 September 2026, ADR-061); the item stays unticked until nesting and state and
   XP are.** `pallet-drc369-royalties`, mounted as `Drc369Royalties`, holds each asset's terms — up to eight
   recipients (ADR-057) with a share of every sale, and a remix share, set by the creator — and the one sale a
   royalty can bind to: a listing bought and settled in CGT, which pays a remix's direct source, then the asset's
   recipients, then the seller, in one transaction. `pallet-drc369` records `derived_from` and `remix_depth` at mint,
   bounded at 16. No platform share (U-15) and no fee (OPEN-4). **The owner ruled the same day (ADR-062):** remix
   royalties stay one level deep, and a creator may correct their terms whenever they hold the asset, with a remix
   share that never rises once the work is remixed. Evidence: 19 royalty tests and 2 remix tests, every rule
   shown to fail against a planted fault first, and the workspace suite with the wasm built. Remix
   *rights* are access gating (ADR-006) and not part of it; the launcher's selling surface is L4.6.
3. [ ] Rental, fractional ownership and fixed-point physics.
4. [ ] Sponsored fees and deposits, so that a new creator can mint without holding CGT first (inventory
   F-Q1, F-D7).
5. [ ] **The DRC-369 acceptance tests, and requirement R-2**: state and XP, nesting, royalty arithmetic,
   authorised mint, the parent-owner check, and a nesting cycle refused within a bounded depth. Moved
   here from M3.3 on 2026-09-18, because the pallets they test are this milestone's.
6. [x] **Several assets moved in one signature**: `pallet-utility`'s `batch_all` mounted and let through the
   call filter, so a trade of many assets either happens entirely or not at all.

   **Done 2026-09-22** (ADR-053), the same day the owner asked for trade. `Utility` is at index 9 and
   `spec_version` is 3. **Only `batch_all` is reachable** — `batch` and `force_batch` carry on past a failure,
   which for a trade means handing over some assets and keeping the rest — and it opens no second way to make
   an asset, because a signed origin's inner calls go through the same base filter. Evidence: four runtime
   tests, each shown to fail first against a planted fault (the filter also allowing `batch`; the filter
   allowing `Nfts::mint`; the trade built on `force_batch`; the filter refusing every `Utility` call), and 74
   workspace tests passing with the wasm built. Its weights are the SDK's reference weights, placeholders owed
   to M7.2. Numbered after the acceptance tests because a dozen documents and `GATES.toml` refer to M4.5 by
   that number.

### M5: SDK, agent rails and the web surface

1. [ ] An SDK that builds and signs transactions from the runtime's metadata, with a reference integration.
   The SDK is published only after the wire format is frozen.
2. [ ] Delegated agent keys with scoped capabilities, protocol-enforced spend caps and revocation.
3. [ ] The MCP server, and generate-and-mint as its reference capability.
4. [ ] The public viewer, backed by the provenance source chosen in M2.
5. [ ] The thin remote console.
6. [ ] **The agent session-key acceptance tests**: authorise, revoke and expiry, against the agent
   authorisations of ADR-014. Moved here from M3.3 on 2026-09-18, because delegated agent keys are this
   milestone's.

### M6: The economic mechanisms

The release mechanism must exist before the first public testnet (OPEN-3).
1. [ ] Genesis pools and the decay-curve release.
2. [ ] Perpetual issuance and burn per fee class, as one mechanism.
3. [ ] Validator payment from the release and issuance budget.
4. [ ] **Transaction payment** (`pallet-transaction-payment` with a burn-share handler per fee class).
   Moved here from M3.1 on 2026-09-18: it needs a `WeightToFee`, which is OPEN-4.
5. [ ] **Treasury.** Moved here from M3.1 on the same day: it needs OPEN-2, the genesis allocation split.
6. [ ] **The allowance accounting the energy module carried**, re-expressed against whatever OPEN-4's fee
   classes decide. Moved here from M3.3 on 2026-09-18: the custom chain's energy module is not carried
   (ADR-013), and what replaces it is this milestone's fee work.

OPEN-1 to OPEN-4 must be decided before this milestone's values are set. Items 4 and 5 are the reason
M3.1 could be narrowed without dropping anything: the work did not disappear, it moved to the milestone
whose values it depends on.

### M7: Public testnet

1. [ ] Executable governance and runtime upgrades through it.
2. [ ] Weight benchmarks for every call. Runbooks written from scratch. Monitoring.
3. [ ] An external security review of the runtime, the launcher vault and QOR ID.
4. [ ] The public testnet opens as the Beta gate, with the first-release scope of ADR-009. It is
   deliberately not called a release.

### M8: The platform protocols

The protocol side of the platform. The launcher surfaces for these are L7.

1. [ ] The Mesh: seeder work verification (U-6) and seeder payment.
2. [ ] Entitlements from DRC-369, for the Library.
3. [ ] The staking-for-distribution sink, for Market (U-7).
4. [ ] Escrow and reputation bonds (U-8).

Mainnet genesis comes after M8's prerequisites are clear. It forecloses OPEN-1 to OPEN-3, and nothing is
decided for it here.

### The launcher track

The QOR Launcher is a first-class deliverable with milestones of its own. They run **alongside** the chain
milestones, not after them. Each states the chain milestone it depends on; one with no dependency can
proceed now. Release gates that refer to these items are defined in [`GATES.toml`](GATES.toml).

#### L0: Phase 1 (done, 2026-09-13)

Vault, QOR ID sign-in, CGT send and receive, chain view, settings, five themes, accessibility settings.
65 host tests pass.

#### L1: Hardening and identity (no chain dependency; next)

1. [x] Visual identity in line with §5: the particle field, glows and pointer light removed; status colours
   taken from theme tokens; one type and spacing scale; the design system documented (reconciliation §7).
2. [x] Accessibility settings that take effect: the contrast override and the motion preference.
3. [x] Amounts never pass through JavaScript numbers; no raw Spark counts in user-facing views; no
   2-decimal fallback.
4. [ ] Host-side confirmation before any signature, except the QOR ID sign-in or name claim that directly
   follows unlocking or creating the vault, which opening the vault approves (ADR-016; since ADR-056 nothing is asked to open it). The webview cannot
   repoint the RPC or QOR ID
   endpoints without that confirmation. Implemented and unit-tested; it stays unchecked until the native
   dialogs are exercised in a running launcher, against the list in `HANDOFF.md`.
5. [x] Domain-separated QOR ID challenge signing, shared with security track item 7.
6. [ ] The launcher's host tests run in CI. The job is written (`.github/workflows/ci.yml`, "QOR Launcher
   host"): frontend build, format, lints and tests. It has never executed, because no job in **any** workflow in this repository has ever run — every run fails at startup, and a trivial probe workflow pushed on 2026-09-21 (run `35622108578`) failed the same way, which rules the workflow file out as the cause (`HANDOFF.md` §1). That is not the same as no workflow run has
   started since the repository transfer, so this stays unchecked until L1.7 makes it real.
7. [ ] CI on `main` starts and passes, and produces the coverage report `GATES.toml` reads (cargo-llvm-cov,
   for the pallets it lists). Every run since 9 September 2026 failed to start or was cancelled, and a
   permanently red CI signal makes the release gates worthless.

#### L2: Development dashboard (no chain dependency)

1. [x] Gate criteria in `GATES.toml` approved by the owner (2026-09-14).
2. [ ] A launcher view showing each gate's criteria and their state, computed only from the signals
   `GATES.toml` defines. Built and tested from both sides: the host (`src-tauri/src/gates.rs`) applies the
   file's counting rules and nothing else, tested against the real `GATES.toml`; and
   `scripts/check-gates-view.mjs` drives the built view in a real rendering engine against a fixture whose
   numbers are known, so the view cannot round, weight or invent a number, drop a not-met unit, or draw a
   bar while anything is unmeasurable (34 checks, 2026-09-17). What is left needs a person at a running
   launcher: running a suite, approving once and declining once, and reading the CI units with `gh` signed
   in and signed out (`HANDOFF.md` §4 item 4).

#### L3: Chain client on the Polkadot SDK (depends on M3)

1. [x] Transactions built and signed from runtime metadata; the custom RPC client removed.

   Done 2026-09-20 (ADR-040). The client is `subxt`, under ADR-033 rule 2, and every call is addressed
   against the metadata the connected node serves, so nothing about the runtime is compiled in. The
   launcher declares its own `subxt::Config`. It was needed on the day because the runtime still used
   `IdentityLookup`, which made an extrinsic's address a bare `AccountId32`; ADR-041 moved the runtime to
   `AccountIdLookup` later the same day, so every value in it is now the SDK's own default and it stays as
   the one place a future divergence would be declared. `subxt`'s own signer is deliberately not
   used: it is synchronous and infallible, so the client reads the bytes to be signed and hands those to
   the vault, which draws the host dialog (L1.4) and signs inside its own lock. Transfers use
   `transfer_keep_alive`, carry the nonce from `system_accountNextIndex`, and are reported only once
   GRANDPA has finalised them. The endpoint is a WebSocket address now, and the chain surface shows the
   name the node reports, which is what tells a Demiurge node from any other Substrate node on the same
   port.

   **Two calls still refuse, for reasons that belong elsewhere.** Transaction history waits on ADR-028's
   indexer, since a Substrate node serves no history RPC; the starter claim waits on the economics, since
   the chain has no issuance and OPEN-1 and OPEN-2 are undecided.
2. [x] Account key scheme and address display as decided in inventory Q-6 and Q-7.

   Done 2026-09-19. Sr25519 keys derived the ecosystem's way, the first account the bare phrase and further
   accounts `//0`, `//1`, … as Talisman enumerates them; SS58 at the chain's prefix wherever a person sees an
   address, with the raw account ID in the vault's advanced panel only (ADR-023, ADR-024, ADR-039). QOR ID
   moved with it: it verifies Sr25519, accepts an account as SS58 or as advanced hex, stores the 32 bytes,
   and gives a password-only account no chain identity until it proves a key (ADR-017).

#### L4: Studio and assets (depends on M4)

1. [ ] Minting from any file, as a single asset or a collection.

   **Started on 2026-09-22, not done.** The launcher mints a Qontrol project's commit as one asset (M4.1).
   A file that is not in a project, and a collection of more than a creator's singles, are not built.

   **What an asset may be, said plainly because it belongs here and not later.** The manifest fingerprints
   arbitrary bytes: an archive, a video, a song and its stems, an image, a document, a font, a 3D model. The
   launcher labels each file from a fixed table of IANA-registered media types and records anything else as
   `application/octet-stream`. **The table was widened on 2026-09-22**, when the owner asked for this to be
   clear: archives, video, sound, images, documents, models and fonts are named; a project's own formats
   (`.blend`, `.fbx`, `.als`, `.psd`, `.tscn`) are not, because no registered type exists and inventing one
   would put a name the registry does not know into a manifest that is wire format after the freeze. **Widening
   the table found a question that must be answered before that freeze**: a file's media type is inside the
   hashed manifest, so the table is part of an asset's identity, and the format names no table — recorded as
   **Q-20** (ADR-047, open item 3), which `beta.media-types` counts.
   **The chain holds none of the bytes**
   — the reference, the commit, the name and the ownership record, not the files and not the manifest — so it
   proves which bytes an asset is and who has owned it, and cannot produce them until the Mesh (M8) serves
   them (`protocol/PROTOCOL.md` §12).
2. [x] Owned assets listed from on-chain enumeration.

   **Ticked on 2026-09-22.** The Inventory lists an account's DRC-369 assets from `pallet-nfts`'s owner
   index and `pallet-drc369`'s records, read from chain storage every time it is opened or refreshed, and
   again after a change. Since 22 September an asset is drawn as a card (P2.8). `check-inventory-view.mjs`
   holds the view to the host's answer (72 checks, shown to fail first against three faults in the view and
   three in the card); the launcher's live test holds the host to the chain.
3. [ ] Amounts shown in the display unit (ADR-007), with raw CGT only in advanced views.
4. [x] **An asset's own menu**, on right click and from the keyboard: trade, sell, share.

   **Done 2026-09-22**, the day the owner asked for it. The card carries a button the keyboard reaches and the
   pointer's secondary button opens the same menu; it takes focus, the arrow keys move through it and Escape
   closes it. **Sell was offered and refused, with the reason** — a listing needs the royalty pallet (M4.2)
   and the indexer (M5.4) — because an item that silently does nothing is worse than one that says why.
   **Since 2026-09-23 it opens the listing form instead** (L4.6), and the note under it carries the same
   reason: what the form drafts publishes nowhere. Share
   copies the asset's content reference. Ten checks in `check-inventory-view.mjs`, shown to fail first when
   the menu button is removed and only the right click is left. **Corrected the same day**: the first item's
   label depended on how many assets the account held, so an account holding one saw "Send…" and not "Trade";
   the three names are now pinned by a check.
5. [x] **Trade**: several assets sent to one address in one approval, with the rest of the account's assets
   offered to add to it; an optional message, which is public and permanent and is said to be; and a last
   dialog that says ownership moves to that address irreversibly and the assets leave this account.

   **Done 2026-09-22** (M4.6, ADR-053), and **reworked on 2026-09-23 at the owner's request** into two sides
   with a lane between them: what leaves this account on the left, where it goes on the right, and a mark that
   travels the lane once whenever the trade changes — never a loop, and not drawn at all under reduced motion.
   **Recently traded with** offers the accounts this machine has sent assets to before, remembered locally
   after the chain finalised each trade (`partners.rs`) and sent nowhere. Reaching someone by their QOR ID
   rather than an address needs a directory lookup, which does not exist (L4.7). The whole trade is one
   `Utility::batch_all` built from the node's metadata, so it happens entirely or not at all; the message is a `System::remark_with_event` in the same
   transaction. Composing and agreeing are two screens, and the host's own dialog (L1.4) repeats the warning
   outside the webview before the vault signs. **Refused before anyone is asked**: an address that is not one,
   this account's own address, nothing chosen, more than sixteen assets, the same asset twice, a message over
   256 bytes, and an asset the chain says this account no longer holds. Evidence: 5 host tests shown to fail
   first against three faults (2 for the trade, 3 for the partner list — counted once, kept most recent first,
   bounded at twelve, and a file that cannot be read treated as an empty list rather than an error), 28 checks
   driving the flow in a real browser shown to fail first against three more, and **a live trade against a
   development node** — three assets minted, two traded away with a message, and the chain asked afterwards
   who holds what. **The live trade predates the rework**: what was re-run on 2026-09-23 is everything except
   it, so the two sides and the partner list have not been through a node or seen by a person.
6. [ ] **Sell**: a listing other people can see, made from the asset's menu. Depends on the royalty pallet
   (M4.2), a listing settled in CGT (P5.5) and the indexer (M5.4) — without an indexer there is no shared view
   of listings at all. The USD half the owner asked for is **U-15**, undecided, with its fee share and its
   custody, escrow and legal questions.

   **Since 2026-09-29 the chain can settle a sale** (M4.2's royalty half, ADR-061): `Drc369Royalties::list` and
   `buy` pay royalties and the seller in CGT and hand the asset over in one transaction. What is still missing
   is the launcher calling them, and the indexer, without which a listing on chain is visible only to someone
   who already knows the asset to look up.

   **Half of it exists since 2026-09-23, and the item stays unticked, because the half that exists is the
   half nobody else can see.** Sell opens a form that drafts a listing on this machine
   (`src-tauri/src/listings.rs`, `src/views/SellDialog.tsx`): a title, a kind, the questions that kind asks, a
   price and a description. The draft is a file in the launcher's data directory, is sent nowhere and is
   visible to nobody else, and the form says so in the product — a banner before anything is typed, and a line
   after saving that says it went no further. **It was built so the shape of a listing is decided by looking
   at a real one rather than by imagining it**, and so the work of describing an asset survives until Market
   (P5) exists to receive it. **The price is exact**: the creator's own number in CGT, parsed by the host,
   excess precision refused rather than rounded, and no float (AGENTS.md §5). **It invents no economic value**:
   nothing takes a share, because what a platform takes is U-15 and undecided.

   **The owner's, before this goes further** (AGENTS.md §8): the six categories and the questions each one
   asks — Gaming, Entertainment, Creative tools, Software, Physical (offline), Other — are **a proposal
   written to be argued with**, not a decided vocabulary. The owner named Gaming, Entertainment and "Physical
   Offline" and said "etc."; the rest was filled in. They are deliberately not Market's payload kinds
   (`docs/blueprints/market.md`), which say what a buyer's machine does with the bytes; a category says what
   the thing is. **Physical (offline) carries a warning the others do not**, shown before it is chosen: the
   chain moves the record, not the object, and nothing here holds the money, checks that anything arrived or
   settles an argument about it. Evidence: 3 host tests and 16 checks driving the form in a real browser.

#### L5: Agent rails (depends on M5 and ADR-014)

1. [ ] The controller's vault signs each agent authorisation.
2. [ ] Agents listed, limited and revoked from the launcher.

#### L6: Distribution (no chain dependency; before the public testnet)

1. [ ] Signed installers for Windows, macOS and Linux.
2. [ ] An update channel that verifies signatures before installing.
3. [ ] The QOR Installer, sharing the launcher's Rust core.

#### L7: Platform surfaces (depends on M8)

1. [ ] Library: install, delta patch and launch.
2. [ ] Market.
3. [ ] Social (named Agora until 2026-09-28, by the owner).
4. [ ] Mesh seeding.
5. [ ] Studio inside the launcher.

### The product tracks

One track per product, added on 21 September 2026 at the owner's instruction, in the form ADR-050
(Proposed) describes. Each product's design is its blueprint in [`blueprints/`](blueprints/ECOSYSTEM.md);
this is the only list of its steps. Every item names what it depends on, and an item that names nothing
can start now. Each product has a release gate of its own in [`GATES.toml`](GATES.toml), appended after
Public Release; **nothing here changes Alpha, Beta or Public Release.**

A track is not active scope. A product enters active scope by ADR-050's decision 10, once the record that
defines it is accepted. Until then the two first slices below live inside `tools/qor-launcher/`, which is
active, and nothing is written under `products/` or `platform/`.

No item here sets an economic value. Where a product needs one, its blueprint names the open question
(OPEN-1, OPEN-2, OPEN-4, U-4, U-6, U-7) and leaves it empty.

#### P1: Qontrol (P1.1 to P1.5 depend on nothing)

Version control for creative work. [`blueprints/qontrol.md`](blueprints/qontrol.md).

1. [x] The first slice, as the owner scoped it: a Projects surface on the rail. Open a folder or create one
   from a code, music or game scaffold; see history, current changes with diffs, and branches; commit.
   gitoxide reads, and the `qontrol-git` helper stages and writes the tree in a separate process, so
   libgit2 never links into the host that holds the vault. Nothing leaves the machine.

   **Done on 2026-09-23, when the diffs arrived.** The rest was built on 2026-09-21 (`5a44207`) and left
   unticked because the diffs were not. Choosing a change shows what changed inside it, line by line with
   the file's own line numbers, read by gitoxide in `src-tauri/src/qontrol/diff.rs`: the index's blob
   against the working-tree file **converted to what git would store** under the repository's own
   `core.autocrlf` and `.gitattributes`, which is what `git diff` compares. So CRLF on disk under autocrlf is
   no change, a `text` attribute normalises on its own, `-diff` or a NUL byte makes a file binary (shown with
   its sizes), and an empty, conflicted or submodule path says what it is. It never runs a program: a diff
   driver's text conversion or external command is not applied, because Qontrol opens folders from
   strangers. A diff past 4,000 lines stops and says so. Qontrol's host tests are 24, all passing under
   `qontrol-no-skips`; three of the new ones are required by `qontrol.tests`, and they and two more failed
   against a planted raw-byte diff. `check-projects-view.mjs` has 67 checks, including a late answer for a
   file no longer chosen being dropped, and `check-readability.mjs` measures an open diff in every theme.
   Since 2026-09-22 the surface's controls use classes the stylesheet defines (`3f67870`) and it has a Mint
   panel (M4.1). **It runs from a development build only:** the helper is not bundled yet, which is P1.2,
   and nobody has yet opened a diff in a running launcher.
2. [x] The git layer completed: branch, switch, discard, per-file diff and commit; a guard before staging
   that warns on large files and on anything credential-shaped; and the helper bundled with the launcher
   (`externalBin`), so an installed launcher can commit.

   **Done on 2026-09-26.** *Branch* is made at HEAD by gitoxide and not switched to. *Switch* and *discard*
   are libgit2's safe checkout in the `qontrol-git` helper, because gitoxide 0.87 can only check out a whole
   index into a folder: a switch carries uncommitted changes across and refuses, touching nothing, when one
   would be overwritten; a discard puts back a modified or removed file and refuses a new one, because that
   would be deleting it. *Per-file commit* stages only the ticked changes, matched by exact name — libgit2's
   pathspec treats `take[1].wav` as a glob even when told not to, so the helper decides itself. *The guard*
   reads what the helper would stage (the same ignore rules as the commit) and holds back any file over
   50 MiB, or named or shaped like a credential, until the person says yes to it by name; its warnings never
   carry the text that matched, and the host runs it again at commit rather than trusting the view.
   *Bundling*: `npm run app:build` builds the helper, places it where Tauri takes a sidecar from, and
   applies `src-tauri/tauri.bundle.conf.json`, which declares it and ships libgit2's licence from the exact
   vendored source (the blueprint's risk 5). A Windows installer was built and opened: `qontrol-git.exe` sits
   beside `qor-launcher.exe`, where the launcher looks. Evidence: 39 Qontrol host tests and 16 helper tests,
   each new refusal proven to fail first where a single fault can reach it; `check-projects-view.mjs` 104
   and `check-readability.mjs` 231, the new view checks each failing against a planted fault. **Not done:**
   nobody has installed that installer and committed from it, the macOS and Linux bundles have not been
   built, and every view check drives the built page against a fixture host.
3. [ ] Scaffolds completed: Unity and Unreal variants, `qontrol.toml` and the set of files that need a lock,
   tested by a fixture repository that ends clean under both status implementations.
4. [ ] Reading a project as a creator sees it: a structural diff for session formats, Ableton's `.als`
   first.
5. [ ] Large binaries: content-defined chunking into a local content-addressed store, pointer files, and
   local advisory locks, correct under interruption. The fingerprint is ADR-047's BLAKE3-256 root, with its
   algorithm tagged.
6. [ ] Publish a version as a DRC-369 asset, and a fork as remix provenance. Depends on M2.3, M4.1 and M4.2,
   and on L1.4 for the signature.

   **Half built on 2026-09-22.** Publishing a version is the Projects surface's Mint (M4.1): the commit HEAD
   points at, by hash, as one asset. A fork as remix provenance waits on M4.2, and revising an existing asset
   with a later commit has a chain call and no button yet.
7. [ ] A remote, over the Mesh. Depends on M8.1 and M8.2.

#### P2: QFX (P2.1 to P2.5 depend on nothing on chain)

The launcher's living visual layer, in three layers. [`blueprints/qfx.md`](blueprints/qfx.md).

1. [x] Layer one's first slice: one WebGL2 canvas behind the interface; one shader drifting slowly
   between the active theme's own colours, lifted near the pointer; an Ambience setting with Off, Still
   and Live, Live the default at a restrained amplitude; reduced motion forcing Still; a frame-time budget
   that stops the canvas; the chrome's contrast guarantee, a scrim a theme cannot reach. No audio.

   **Done 2026-09-21** (`75b2ead`), the slice the owner scoped, with Off corrected the same day
   (`6e5bb28`): it had removed the scrim while the canvas stayed on screen. `check-contrast.mjs` applies every
   theme through the app and solves for the worst colour the canvas could paint (54 checks since 22
   September, proven to fail by lowering the scrim), and `check-accessibility.mjs` drives the backdrop
   through the app and counts frame callbacks (21 of its 34 checks). **It meets ADR-051 as the owner amended
   and accepted it on 22 September 2026**: only the built-in ambience runs behind the Gate, the scrim is the
   guarantee (0.90 since that day), the one-step budget is accepted for v0, and Still is the pause.
2. [ ] Layer one completed to its record (ADR-051): quality tiers and a governor within the accepted budget
   — at most 25% of the frame at the top tier, 15% at the lowest — capability detection by a timed calibration
   render, and the render-based checks: a white-shader fixture sampled behind text, flashes, and two
   screenshots a second apart under reduced motion.
3. [ ] Audio, in the owner's order: system sound first, the microphone later, neither in v0. Analysis in the
   host, off by default, opt-in, with an indicator while it listens. macOS system sound depends on L6.1.
4. [ ] Layer two: motion and material tokens, clamped by the chrome, and preset composition over them, so
   a theme is authored without code.
5. [ ] The theme package — a "QFX Theme" until the owner names the type: a text manifest, shaders validated
   in the host before they reach the webview, and the built-in themes loaded through it. Veridian and Abyss
   move here and are never the default. Behind the Gate only the built-in default ambience may run, never an
   installed theme.
6. [ ] Layer three: the personal space, with its scene editor as an editor plugin in QOR Engine's build.
   Depends on P3.2; and no scene becomes a theme until CI runs (L1.7) and the signing dialogs are verified by
   a person (L1.4).
7. [ ] Themes published, used and remixed as DRC-369 assets, their creators paid in CGT, with the shader code
   covered by the asset's fingerprint (ADR-047). Depends on M4.1, M4.2, M8.1 and M8.2, and on Market (P5.5).
8. [ ] DRC-369 assets drawn as cards, in four slices: **the card shell**, the sheen, the preview image and
   the model.

   **The shell is done, 2026-09-22**, at the owner's request and as the owner sequenced it: every asset the
   Inventory reads from chain storage is a card that leans towards the pointer and opens at size on its own
   title, carrying what the chain holds and, until there is a preview, a mark derived from the fingerprint
   itself. It is in `src/qfx/`, under the carve-out the owner widened that day from the backdrop to a
   QFX-owned surface (`GATES.toml`, `DESIGN_SYSTEM.md` §1); the two exempt rules did not change, so the card
   has no gradient, shadow or glow. **The sheen** is the one canvas showing through the card, never a second
   canvas (ADR-051). **The preview image** needs a mint to name one: ADR-047 has the Preview role and nothing
   sets it, and the bytes come from the temporary content store until the Mesh (M8). **The model** is a glTF
   or GLB entry rendered in that same canvas, which is the 3D view the owner asked for.

#### P3: QOR Engine (P3.1 depends on nothing)

A creation tool that is a custom Godot build tracking upstream. [`blueprints/qor-engine.md`](blueprints/qor-engine.md).

1. [ ] The build exists: a lightly rebranded Godot build on Windows, macOS and Linux — "QOR Engine, built on
   Godot", with an editor Godot's own documentation still describes — tracking the current stable 4.x line,
   pinned exactly, with one rebase performed, and that line and the MIT attribution pinned by a test.
2. [ ] Identity and versioning: QOR ID shown read-only in the editor through a Rust GDExtension; glTF and
   FBX in and glTF out, proven on fixtures; the Qontrol panel; the engine launched from the launcher as a
   separate process. Depends on P1.2 and on ADR-046.
3. [ ] The scene as a Demiurge artifact: signing through the vault from the editor, DTCG themes, FLAC,
   Opus and SMF, shader packages, and USD read (not written). Depends on L1.4.
4. [ ] Assets are owned: DRC-369 assets as live nodes, nesting that creates a royalty relationship, and the
   agent rails panel. Depends on M4.1, M4.2 and M5.2.
5. [ ] Publishing and delivery over the Mesh, with settlement in the editor. Depends on M6.4, M8.1, M8.2
   and M8.3.
6. [ ] Every QOR Engine project opens in stock Godot of the tracked version, with QOR's nodes degrading
   rather than failing without its additions — the owner's promise, 22 September 2026 — pinned by a test that
   opens fixture projects in an unmodified build (`a_project_opens_in_stock_godot`). Depends on P3.1.

#### P4: GNOSIS (P4.1 depends on nothing)

Music production on a mergeable project tree. [`blueprints/gnosis.md`](blueprints/gnosis.md).

1. [ ] The format alone: the specification, a deterministic writer, a round-trip reader, the typed merge
   drivers and a merge test suite, with a save that changes nothing producing a byte-identical tree. No
   audio code.
2. [ ] Playback on one platform: device output (WASAPI first on Windows), decode, transport, clips, gain and
   pan, committed and merged through Qontrol. Depends on P1.2, and on L1.4, because every GNOSIS commit is
   signed.
3. [ ] Recording and GNOSIS's own devices; DAWproject import and export; stems, a tempo map and an SMF
   exported automatically.
4. [ ] CLAP plugins hosted in a separate process, correct when a plugin changes its latency.
5. [ ] Song versions and stems as DRC-369 assets with remix royalties, their media on the Mesh. Depends on
   M2.3, which must take GNOSIS's asset requirements in before it freezes, and on M4.1, M4.2 and M8.1.

#### P5: Market and Library (P5.1 and P5.2 depend on nothing)

Listing, buying, installing and launching, for six kinds of payload. Library and Market remain L7.1 and
L7.2; these are the steps inside them. The creator mints; Market lists. [`blueprints/market.md`](blueprints/market.md).

1. [ ] Themes from outside the launcher: a registry loaded from disk, fetched and checked against a
   manifest hash, and validated against the token schema and the contrast floor at install, saying in the
   product that nothing is owned on chain yet. A user cannot override a safety check; a taste check, they can.
2. [ ] The installer core: content-defined chunking, per-chunk hashes, delta patches and supervised launch,
   against local manifests.
3. [ ] Listings and the catalogue, read-only, with one listing model for games, tools, editor plugins, QFX
   scenes, presets and themes. Depends on the indexer, M5.4.
4. [ ] Launch gated on ownership — offline too, which the owner requires, without QOR ID's attestation standing
   behind access to paid content — and downloads verified against the chain's fingerprint. Depends on M4.1.
5. [ ] Purchase with royalty splits, through the host dialog, priced in credits over CGT settlement (ADR-007).
   Depends on M4.2 and M6.4.
6. [ ] Delivery over the Mesh, and staking for distribution. Depends on M8.1 and M8.3, and on U-7, which the
   owner deferred to M6 with any platform share of sales.

#### P6: Stream, a placeholder name (P6.1 depends on nothing)

Music distribution and licensing over the Mesh. **A listening service first**, by the owner's decision of 22
September 2026; the steps below stay in dependency order, because licensing needs only M4 and listening needs
M8. [`blueprints/stream.md`](blueprints/stream.md).

1. [ ] A decision, not code: whatever U-6 decides must produce a delivery receipt, not only a storage
   proof. Without one, nothing can count a play, and the listening half cannot be built. **Deferred by the
   owner to M6** (22 September 2026).
2. [ ] Catalogue and licensing, as part of Market: audio as a DRC-369 payload, royalty policies, and licence
   and stem-pack purchase. Depends on M4.1, M4.2 and P5.5.
3. [ ] Delivery: lossless masters stored and lossy tiers streamed, Opus in the publish path, and per-album
   swarms with deadline-aware piece picking. Depends on M8.1.
4. [ ] Metered listening, creators paid automatically once per period — per-play payment on chain is ruled
   out for now — and a listener needs no chain key. Depends on U-6 producing a delivery receipt, and on M6
   for the payout mechanism; whatever funds or forfeits anything is OPEN-1, OPEN-2 or OPEN-4.

## 8. Scope

### Active

- `chain/`: the Substrate L1, and **the only chain in this repository** (ADR-013; located and named by
  ADR-032, which decided Q-15).
- `services/qor-auth/` and `tools/qor-launcher/`.
- `tools/qor-installer/`, when M8 reaches it.

The product tracks (§7) are planned, not active scope. Qontrol's Projects surface, its `qontrol-git`
helper and QFX layer one are part of `tools/qor-launcher/`. A product gets a directory under `products/`
only by ADR-050's decision 10, once its defining record is accepted.

`framework/`, the custom Rust devnet `chain/` replaced, was retired at M3.5 on 20 September 2026 and
deleted. It is not active, not a reference, and not to be restored; CI fails if the directory returns.

### Frozen (D-011, amended by ADR-011)

Not maintained and not a reference for anything: `apps/hub`, `apps/wallet-extension`, `apps/sophia`,
`apps/nft`, `apps/portal`, `apps/marketing-site`, `apps/games`, `apps/guru`, `cli/`, `sdk/`, `packages/`,
`client/DemiurgeClient`.

Pages inside those apps that describe the protocol, such as `apps/hub/src/app/docs/**`, are legacy
content: **they document the custom Rust devnet, which was deleted at M3.5**, and they still name the
currency `CGT` and describe the energy mechanism that went with it. Nothing there describes `chain/`.
They are not corrected, because correcting them would extend frozen scope; they are listed here instead,
so nobody mistakes them for documentation. The web surfaces of ADR-011 are new work. Whether any frozen
code is reused for them is unknown.

### Dead or not carried

- **Not built by any workspace:** `aeons/` (Substrate pallets targeting an old release; CGT at 8
  decimals there), `archons/`, `syzygies/`, `packages/blockchain-wasm`.
- **Deleted with `framework/` at M3.5, and not carried to `chain/`:** its `cvp`, `zk`, `yield-nfts`,
  `agentic`, `governance` and `game-registry` modules, its `consensus/src/{modular.rs, sharding.rs}`, and
  the staking and era logic nothing ever called. The migration inventory records where each one went, or
  that it went nowhere.

### Unverified operational files

These predate the realignment. Several reference a `blockchain/` directory that no longer exists. **Do
not deploy from them.**

- `scripts/*.sh`, `testnet/`, `docker/`, `config/` (including `config/production/genesis*.json`)
- `Dockerfile`, `fly.toml`, `demiurge-server.sh`, `ignite_demiurge.sh`

The only verified way to run the current stack is
[`scripts/run-local-stack.md`](../scripts/run-local-stack.md).

## 9. Risks being carried

- **The base layer is in transition.** `chain/` produces and finalises blocks, but M3 is not complete:
  M3.2, the chain specification at the base supply, is open, and nothing above the base layer — DRC-369,
  royalties, agent rails, the economics — exists yet.
- **The economics are not built.** There is no transaction payment, no issuance and no treasury, because
  each needs a value OPEN-1, OPEN-2 or OPEN-4 leaves undecided. A chain that charges nothing is not a
  chain anyone has paid to attack, and none of the pressure those mechanisms carry has been observed.
- **Security defects in QOR ID** (§7.1) are latent while production is offline and would be live on any
  deployment.
- **Irreversible choices are still open.** The genesis split, issuance, decay curve, key scheme and
  address format are all foreclosed by mainnet genesis. The DRC-369 wire format is foreclosed by SDK
  publication.
- **Six products compete with the chain for the same weeks.** Four have not started, and the two that
  have are launcher surfaces. Every product track ends in items that wait on M4 or M8. M2.1 and M2.3, which
  held all chain work back, were both ticked on 22 September 2026.
- **Unaudited.** Nothing here should hold real value.
- **Production is offline.** No external user can reach the chain.
