# Direction

**The only roadmap, and the canonical definition of what Demiurge is.** This document settles "what are
we building". Decisions behind it are indexed in [`DECISIONS.md`](DECISIONS.md). The economic model is
[`economics/CGT.md`](economics/CGT.md). What the chain does today, exactly, is
[`protocol/PROTOCOL.md`](protocol/PROTOCOL.md). How the current code measures up against all of it is
[`audit/RECONCILIATION.md`](audit/RECONCILIATION.md).

**Last updated:** 2026-10-06

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
  today are Gate, Nexus, Vault, Inventory, Market, Chain, Gates, Projects and Settings; Library, Social (named Agora until 2026-09-28) and Mesh
  are placeholders, and Studio is a link out. Market lists the DRC-369 listings on chain as the connected
  node reads them, with no search and no indexer (`SYSTEMS.md`). Products it launches will be separate
  processes it supervises and signs for (ADR-046, accepted 28 September 2026).
- **A web surface** in two parts, described in §5.

**Seven products are built on that substrate**, each with a blueprint in
[`blueprints/`](blueprints/ECOSYSTEM.md) and a track of its own in §7:
- **Qontrol**, version control for creative work, git-compatible on disk (P1). Its first surface,
  Projects, is built, line-by-line diffs included since 23 September 2026, and since 22 September 2026
  it mints a commit as a DRC-369 asset (M4.1).
- **QFX**, the launcher's living visual layer, authored by creators and paid for when used (P2). Layer
  one's first slice, the backdrop, is built.
- **QOR Engine**, which is **QQ** since ADR-081 (6 October 2026): an engine and editor for small,
  effects-led 2D and 3D games, in the launcher and on ARQADE, driveable by an LLM and generative (P3). The
  Godot build it was planned as is withdrawn.
- **GNOSIS**, music production whose project is a mergeable tree of text files, versioned by Qontrol (P4).
- **Market and Library**, where work is listed and bought in CGT, and where what you bought installs,
  patches and launches (P5, carrying L7.1 and L7.2).
- **Stream**, a placeholder name for music distribution and licensing over the Mesh (P6). It does not
  exist before M8.
- **ARQADE**, the gaming platform: games played with a QOR ID, every owned collectible a DRC-369 asset,
  CGT the only currency (P7). It is live at `https://qor-arqade-tau.vercel.app` since 5 October 2026.

**One substrate, seven apps.** Every product signs in with QOR ID, stores creations as DRC-369 assets,
versions them with Qontrol, distributes through the Mesh and settles in CGT. A product that would need its
own identity, asset format, storage or payment rail has found a substrate gap: it is recorded in
[`blueprints/ECOSYSTEM.md`](blueprints/ECOSYSTEM.md) §4, never filled locally. The six substrate
decisions they share are ADR-046 to ADR-051, all accepted: ADR-047 (the asset format) and ADR-051 (QFX's
rendering layer) on 22 September 2026, and ADR-046, ADR-048, ADR-049 and ADR-050 on 28 September 2026.

**Scope discipline.** Anything proposed later must justify itself against one of the four primitives.
A proposal that strengthens none of them is out of scope. The seven products are held to the same rule:
each is a surface over the primitives, not another thing beside them.

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
  payment (OPEN-4), no issuance (OPEN-1), no treasury (OPEN-2), no asset state and XP, physics, rental or
  fractions (M4.2, M4.3), and no sponsorship (U-4). DRC-369 itself, its royalties and settled sale
  (ADR-061), nesting (ADR-065) and ARQ Wallets (ADR-070) are built.
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

**Visual identity.** The rule is distinctiveness (ADR-080, accepted 6 October 2026): every surface is
intentional and recognisably Demiurge's, never the default output of a template or an AI. Glow, neon,
gradients, shadows, canvas and shader backdrops, pointer-reactive light and looping animation are allowed
on every surface. Three duties stay, because they protect people or keep themes working: reduce motion
stills or skips every moving effect; text is readable over whatever is behind it, measured as painted;
and colours, sizes and letter-spacing come from the theme's tokens and scales. `check-design.mjs` holds
the scales, and `check-contrast.mjs` and `check-readability.mjs` measure readability in every theme. A
surface that uses an effect carries the reduce-motion and readability duties itself.

**QFX's canvas moves behind the launcher's interface** (ADR-051, accepted 22 September 2026). It is no
longer an exception to the design system, because since ADR-080 there is nothing to exempt it from; its
obligations stay: readability as painted, still under reduced motion, and a frame-time budget. Layer one's
backdrop (P2.1) is a slow drift in the theme's own colours under a scrim the chrome owns, live by default at a
restrained amplitude, and off at one switch. Nothing above it — no text, control or status colour — is ever
the canvas's.

## 6. Where things stand

Re-verified on 6 October 2026 against the tree, CI and the live services. The first verification, 13 to
21 September 2026, is in [`audit/RECONCILIATION.md`](audit/RECONCILIATION.md), which is a dated audit kept
as written; [`HANDOFF.md`](../HANDOFF.md) §1 has anything later than this table.

| Component | State |
| --- | --- |
| Chain (`chain/`) | **The only chain** (ADR-013, ADR-032). A Substrate L1 on the pinned Polkadot SDK (`polkadot-stable2606-1`, on Rust 1.98.1 by ADR-072): Aura authors, GRANDPA finalises, and the validator set comes from governance through `pallet-session` and `pallet-validator-set` (ADR-018 to ADR-020). It mounts `pallet-nfts` and `pallet-drc369` (M4.1, ADR-052) with nesting (ADR-065), `pallet-drc369-royalties` with royalties and a sale settled in CGT (ADR-061), `pallet-utility` with only `batch_all` reachable (M4.6, ADR-053), and `pallet-arq-wallet` (ADR-070); `spec_version` 8. 150 workspace tests pass with the runtime wasm built (2026-10-04). Two validators agree, finalise, and one catches up after a restart: CI's nightly two-validators job. No transaction payment, no issuance and no treasury, on purpose (OPEN-1, OPEN-2, OPEN-4). **Demiurge Devnet is live** at `wss://rpc.qorsync.dev`: two validators and an RPC node on Railway (ADR-068), holding test CGT only. |
| Chain (`framework/`) | **Retired and deleted at M3.5, 20 September 2026**, after everything referring to it was rewritten. It was custom Rust with no finality, and untrusted: its non-strict Ed25519 verification accepted signatures nobody made for small-order keys (R-1). Nothing in it is carried forward, and CI fails if the directory returns. |
| CGT on chain | 18 decimals, decided (ADR-035), with an existential deposit of 100 CGT (ADR-036). **`chain/` declares no total-supply constant at all**, because issuance is OPEN-1 and the genesis split is OPEN-2; the superseded 13 billion figure went with `framework/`, and left the launcher on 2026-09-20, which is the removal ADR-003 asks for by name: the launcher's ceiling on a typed amount is now named for what it is and is not a supply, and the Settings surface shows no total (`RECONCILIATION.md`, "Remediation since this audit"). No CGT fee is charged, and there is no path that creates CGT outside a development chain specification. The ticker is `CGT` (ADR-034), and `chain/` was written with it from the start, which is requirement R-4. |
| Economic model | Decided in direction (ADR-002 to ADR-008). Rates, split, curve and burn shares are open. Nothing implemented. |
| DRC-369 | **Built on chain.** `pallet-drc369` over `pallet-nfts` mints an asset carrying ADR-047's 41-byte content reference and its pinned commit, revises it until a one-way switch makes it permanent, creates one singles collection per creator, and lists what an owner holds, from storage and through a runtime API (M4.1, ADR-052, 2026-09-22). Only it can create an asset. Royalties, remix royalties one level deep and a sale settled in CGT since 2026-09-29 (`pallet-drc369-royalties`, ADR-061, ADR-062); nesting, with requirement R-2's cycle refusal, since 2026-10-01 (ADR-065). Its deposits are placeholders (U-14) and its weights are placeholders owed to M7.2. State and XP, physics, rental and fractions are not started. **The launcher** mints a Qontrol project's commit, lists what an account holds, trades, lists for sale, withdraws and buys (L4.2, L4.5, L4.6). The wire format is decided (ADR-047, accepted 2026-09-22, M2.3) and not yet frozen (`beta.wire-format-frozen`). |
| QOR ID (`services/qor-auth`) | **Live at `https://id.qorsync.dev`**, deployed from `main` to Railway. Username and keypair sign-in with JWTs: Sr25519 keys, an account as SS58 or hex, and a password-only account with no chain identity until it proves a key (ADR-017, implemented with L3.2). Email verification, password reset and address change through Resend; single-use backup codes; an account page to change a password or add an email. Agents register their own keys and QOR ID only authorises them (ADR-014); capabilities and spend caps are recorded, not enforced (Q-9). Sign-in for apps by OAuth 2.1 with PKCE (ADR-073; ARQADE is registered); unique usernames (ADR-075); levels, XP, tasks and owed welcome grants (ADR-078, migration 020); avatars (ADR-079, migration 022); for the owner, the owed welcome grants listed and one marked paid against the hash of its transfer (admin routes, 8 October 2026; nothing pays them yet, P7.17); and the launcher signing a person into an app it opened with its own session (`/api/v1/oauth/handoff`, P7.18). No token, key, link or address may reach a log at any level, checked at runtime and in CI, and no SQL statement may be built from values (`alpha.no-secrets-in-logs`, `alpha.parameterised-sql`). The §7.1 defects are fixed; what remains is named in SECURITY.md, among it that an access token alone can still register an agent (L5.1). 172 tests pass against Postgres 16 and Redis 7.4 (2026-10-08). |
| QOR Launcher (`tools/qor-launcher`) | Version 0.1.8, unsigned, with no update channel (L6). A `subxt` client of `chain/` building every call from the metadata the node serves (ADR-040), with an Sr25519 vault and SS58 addresses (ADR-039); the vault's key is in the operating system's keychain and there is no lock screen (ADR-056). Built: Gate, Nexus, Vault, Inventory (assets as cards; trade, sell, withdraw and buy: L4.5, L4.6), Market (the listings on chain as the connected node reads them; no search, no indexer), Projects (Qontrol: P1.1, P1.2), Chain, Gates and Settings, with QFX layer one's backdrop (P2.1). `qor://pay` opens a signed payment request in the host dialog (ADR-076, ADR-077); the level bubble and XP bar show QOR ID progress (ADR-078); an avatar in a ring that glows from level 1, with upload, is on `main` after 0.1.8 and not yet in a build (ADR-079, ADR-080). On `main`'s pull request since 8 October 2026, not yet in a build: QQ's TypeScript preview (ADR-082; the Qt engine of ADR-083 replaces it), ARQADE opened in a window of its own and signed in with the launcher's session (P7.18, partly), and six QFX backdrops to choose from. Library, Social and Mesh are placeholder pages; Studio is an external tile; the Nexus still shows nine frozen-app tiles marked local or forming. 224 host tests pass (2026-10-09), and ten browser checks: design 3, accessibility 49, gates view 34, Projects 104, Inventory 238, Market 127, QQ 42, vault gate 65, contrast 54 and readability 361. CI runs all of them (L1.6). |
| Products (P1 to P7) | Seven blueprints and six substrate records (ADR-046 to ADR-051), all accepted. **ARQADE is live** on Vercel at `https://qor-arqade-tau.vercel.app` since 2026-10-05 (ADR-074): QOR ID sign-in, solo games, two multiplayer games, rankings, chat, devnet reads, tips approved in the launcher (P7.4) and levels. Four first slices are built inside the launcher: Qontrol's Projects surface (P1.1, P1.2), QFX layer one (P2.1, with six backdrops to choose from since 8 October 2026), Market's first slice (P5.3, without the indexer) and **QQ's TypeScript preview** (8 October 2026: a WebGL2 2D engine and editor whose scenes are versioned by Qontrol). **QQ moves to Qt 6 (ADR-083, 8 October 2026)**: its P3 items are rewritten for a native Studio and Player, and P3.1 is done (9 October 2026): QQ Studio edits a lit 3D world, kept as canonical QML in a Qontrol project and opened from the launcher. P3.2 is done (10 October 2026): worlds play, with physics, a player under keyboard, mouse and gamepad, particles, sound and logic rebuilt on each save, in the Studio and in the QQ Player, natively and in a browser. ARQADE shows the player's QOR ID avatar since 8 October 2026. GNOSIS and Stream have not started. |
| SDKs, web apps, CLI | Frozen. None produces a transaction the current node accepts. |
| Agent rails, MCP server, public viewer, remote console | Not started. |
| Production (`demiurge.cloud`, `rpc.demiurge.cloud`) | Offline. No production network exists; the public devnet is `wss://rpc.qorsync.dev`. |
| Audits | None. Nothing here should hold real value. |

## 7. Roadmap

Ordered by dependency. A chain milestone (M) is not started until the one before it is done. Three kinds
of track run alongside the chain milestones rather than after them:
- the **security track** (§7.1), which does not depend on the chain;
- the **launcher track** (L), where each milestone names the chain milestone it depends on;
- the **product tracks** (P1 to P7), one per product, where each item names what it depends on.

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
   is the owner's action**, and the owner confirmed it on 14 September 2026. The values remain only in the
   private archive's history (`ALaustrup/demiurge-cloud`); the public repository's history starts on
   29 September 2026.
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
   and are a separate cleanup; `fly.toml` in particular was to be rewritten for `chain/` under ADR-015
   rather than removed, a plan moot since the devnet runs on Railway (ADR-068). Since then `docker/docker-compose.testnet.yml` has left the tree: it is not in the
   public repository, whose history starts on 29 September 2026.

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

   **The nesting half is built (1 October 2026, ADR-065).** `Drc369::nest` and `unnest`, only by the owner of both
   assets; depth bounded at 8 and children at 64 (ADR-047 decision 13); `pallet-drc369` is `pallet-nfts`'s `Locker`,
   so a nested asset and the asset holding it cannot be transferred, sold or burned until taken out. `spec_version`
   is 5. State and XP remain, so the item stays unticked.
3. [ ] Rental, fractional ownership and fixed-point physics.
4. [ ] Sponsored fees and deposits, so that a new creator can mint without holding CGT first (inventory
   F-Q1, F-D7).
5. [ ] **The DRC-369 acceptance tests, and requirement R-2**: state and XP, nesting, royalty arithmetic,
   authorised mint, the parent-owner check, and a nesting cycle refused within a bounded depth. Moved
   here from M3.3 on 2026-09-18, because the pallets they test are this milestone's.

   **R-2 is met (1 October 2026, ADR-065):** a nest whose parent is the child, or anything inside it, is refused
   by a walk of at most eight reads; the parent-owner check, nesting, authorised mint and royalty arithmetic have
   their tests. Evidence: 10 pallet tests and 3 runtime tests, each shown to fail against a planted fault first,
   and 109 workspace tests with the wasm built. The item stays unticked until state and XP exist and are tested.
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
   Since 6 October 2026 ADR-080 allows those effects again (§5); the tokens and scales still bind.
2. [x] Accessibility settings that take effect: the contrast override and the motion preference.
3. [x] Amounts never pass through JavaScript numbers; no raw Spark counts in user-facing views; no
   2-decimal fallback.
4. [ ] Host-side confirmation before any signature, except the QOR ID sign-in or name claim that directly
   follows unlocking or creating the vault, which opening the vault approves (ADR-016; since ADR-056 nothing is asked to open it). The webview cannot
   repoint the RPC or QOR ID
   endpoints without that confirmation. Implemented and unit-tested; it stays unchecked until the native
   dialogs are exercised in a running launcher, against the list in `HANDOFF.md`.

   **Partly exercised, 5 and 6 October 2026:** the owner approved `qor://pay` tips in the host dialog of an
   installed launcher against Demiurge Devnet (P7.4). Declining a prompt, and the confirmation before the RPC
   or QOR ID endpoint changes, have not been exercised by a person, so the item stays unchecked.
5. [x] Domain-separated QOR ID challenge signing, shared with security track item 7.
6. [x] The launcher's host tests run in CI. The job is "QOR Launcher host (tools/qor-launcher): frontend
   build, format, lints, tests" in `.github/workflows/ci.yml`: the frontend build, `npm run check`'s nine
   browser checks, the `qontrol-git` helper's format, lints and tests, then the host's format, lints and
   `cargo test --features qontrol-no-skips`.

   **Ticked on 6 October 2026.** Before 29 September 2026 no GitHub Actions job here had ever started
   (Woodpecker ran CI on 28 and 29 September, ADR-058). Since then CI runs on GitHub Actions on the public
   repository (ADR-063), and this job runs on every pull request and every push to `main`. On `main`'s run for the merge of #13 (run `37496555161`, 6 October 2026) it
   passed: 204 host tests passed and 7 were ignored, and the helper's 16 passed.
7. [x] CI on `main` starts and passes, and produces the coverage report `GATES.toml` reads (cargo-llvm-cov,
   for the pallets it lists).

   **Ticked on 6 October 2026.** "Pleroma CI" runs on GitHub Actions on every pull request and every push to
   `main` since 29 September 2026 (ADR-063). Every completed run on `main` since the merge of #6 (5 October
   2026) has passed, the merges of #12 (`37470330563`) and #13 (`37496555161`) among them; the runs for the
   merges of #7 and #10 were cancelled by the next merge, as the workflow's concurrency rule does, and the
   merges of #2 to #4 earlier on 5 October failed. On `main`'s latest run the coverage job uploaded the
   `coverage` artifact cargo-llvm-cov writes for the pallets `beta.value-pallet-coverage` lists. The nightly
   run adds the two-validator job, which passed on 5 and 6 October, and the newest-Rust clippy report, which
   fails on lints in code the SDK's macros generate (ADR-072) and does not fail the run. Of the pull requests,
   #8 was merged with its launcher job red on the pull request, and the last runs of #3 and #4 were cancelled.

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
   **Since 2026-09-23 it opens the listing form instead** (L4.6), which until 2026-10-01 drafted a listing
   that published nowhere; **since 2026-10-01 Sell publishes a price on chain** (L4.6), and only a listing's
   description stays on this machine. Share
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
   is the indexer, without which a listing on chain is visible only to someone who already knows the asset to
   look up.

   **Since 2026-10-01 the launcher calls the settled sale.** Sell publishes a price on chain (`list`), Withdraw
   removes it (`unlist`), and an asset looked up by its number can be bought (`buy`); each shows what the sale
   pays before the host dialog, and a declined prompt moves nothing (`src-tauri/src/chain/sales.rs`, live test
   `a_sale_pays_every_part_and_hands_the_asset_over`). **Unticked:** nobody can find a listing without being
   given the asset's number (no indexer, M5.4), royalty terms cannot be set from the launcher, and no person has
   run it against the native dialogs.

   **The first half was built on 2026-09-23, and it was the half nobody else can see.** It is now Sell's
   second screen: a form that drafts a listing's description on this machine
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
(accepted 28 September 2026) describes. Each product's design is its blueprint in [`blueprints/`](blueprints/ECOSYSTEM.md);
this is the only list of its steps. Every item names what it depends on, and an item that names nothing
can start now. Each product has a release gate of its own in [`GATES.toml`](GATES.toml), appended after
Public Release; **nothing here changes Alpha, Beta or Public Release.**

A track is not active scope. A product enters active scope by ADR-050's decision 10, once the record that
defines it is accepted. ARQADE has: `products/arqade/` exists under decision 10 since ADR-069 was accepted on
4 October 2026. The first slices of Qontrol, QFX and Market live inside `tools/qor-launcher/`, which is
active, and nothing else is written under `products/` or `platform/`.

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
   panel (M4.1). **On that day it ran from a development build only:** the helper was bundled on 2026-09-26
   (P1.2), and nobody had yet opened a diff in a running launcher.
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

   **Six backdrops since 8 October 2026** (asked for by the owner, HANDOFF §4 item 57): Drift (the first
   one), Aurora, Nebula, Lattice, Starfield and Liquid, chosen under Settings, each reacting to the pointer
   and growing richer with the QOR ID level. Ambience and reduced motion govern every one.
   `check-contrast.mjs` covers any colour a backdrop could paint, and `check-readability.mjs` measures text over a
   flat white panel in the canvas's place, worse for light text than any backdrop. `check-accessibility.mjs`
   requires each of the six to compile and run, and Still to hold them (since 8 October 2026). The same day
   Starfield was rebuilt to cost one grid cell per pixel (CI's software renderer had drawn it at 11 frames a second),
   and Lattice and Liquid were made smooth through the pointer, where they had knotted and torn.
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
6. [ ] Layer three: the personal space, with its scene editor inside QQ Studio (ADR-083; first planned as a plugin
   in a Godot build, withdrawn by ADR-081).
   Depends on P3.2; and no scene becomes a theme until CI runs (L1.7) and the signing dialogs are verified by
   a person (L1.4).
7. [ ] Themes published, used and remixed as DRC-369 assets, their creators paid in CGT, with the shader code
   covered by the asset's fingerprint (ADR-047). Depends on M4.1, M4.2, M8.1 and M8.2, and on Market (P5.5).
8. [ ] DRC-369 assets drawn as cards, in four slices: **the card shell**, the sheen, the preview image and
   the model.

   **The shell is done, 2026-09-22**, at the owner's request and as the owner sequenced it: every asset the
   Inventory reads from chain storage is a card that leans towards the pointer and opens at size on its own
   title, carrying what the chain holds and, until there is a preview, a mark derived from the fingerprint
   itself. It is in `src/qfx/`, built under the carve-out the owner widened that day from the backdrop to a
   QFX-owned surface. Since ADR-080 (6 October 2026) there are no effect rules left to exempt it from; the
   card still has no gradient, shadow or glow, by choice. **The sheen** is the one canvas showing through the card, never a second
   canvas (ADR-051). **The preview image** needs a mint to name one: ADR-047 has the Preview role and nothing
   sets it, and the bytes come from the temporary content store until the Mesh (M8). **The model** is a glTF
   or GLB entry rendered in that same canvas, which is the 3D view the owner asked for.

#### P3: QOR Engine / QQ (P3.1 depends on nothing on chain)

QQ, the QOR Engine: a game engine for virtual worlds and experiences, built on **Qt 6** (C++20, QML, Qt Quick 3D and
Physics) under the owner's commercial licence, at `products/qq/` — a Runtime, a native **QQ Studio** and a **QQ
Player** that runs natively from the launcher and as WebAssembly in ARQADE's browser, with an **agent** that can
design and build a game from a description. QQ is QOR Engine (ADR-081); ADR-082 decided a first architecture and
**ADR-083 (8 October 2026) moved it to Qt**. Blueprint: [`blueprints/qq.md`](blueprints/qq.md).

**Under way since 9 October 2026**, on Qt 6.12 (Enterprise), in `products/qq/` (`products/qq/README.md`). P3.1 is done
the same day: QQ Studio edits a lit 3D world and keeps it as canonical QML in a Qontrol project, opened from the
launcher. P3.2 is done on 10 October 2026: worlds play in the Studio and in the QQ Player, natively and in a browser. P3.3 is
under way: the Studio's tools are offered to agents over MCP, and its design loop is built; its proof waits on a key.
Its six Qt Test suites run on the owner's machine, not in CI, which has no licensed Qt.

The TypeScript/WebGL2 slice built under ADR-082 on 8 October 2026 (`tools/qor-launcher/src/qq/`: 2D shapes,
particles, an editor surface, scenes saved through Qontrol; `check-qq-view.mjs`, 41 checks) stays as the launcher's
preview until the Qt Player plays in the launcher and in ARQADE (ADR-083 decision 6). It does not meet P3.1 below.

1. [x] **QQ Studio exists.** A Qt 6 application, built with CMake on Windows: a 3D viewport (Qt Quick 3D, physically
   based materials, image-based lighting, a directional light with shadows, tonemapping and bloom); a glTF 2.0 model
   loaded at runtime; a scene tree, an inspector and transform gizmos; scenes saved as canonical QML in a Qontrol
   project, where one changed property is one changed line; `.qq.json` format-1 scenes imported; started from the
   launcher's QQ surface with **Open in QQ Studio**. Tested by a Qt Test suite, run on the owner's machine and
   recorded. Depends on ADR-083 and Qt being installed.

   **Done 9 October 2026** (HANDOFF §4 item 64). Each clause, with its evidence on the owner's machine (Qt 6.12, MSVC
   2022): the viewport and its look, `tst_world` (3 cases judged by rendered pixels); the glTF model at runtime, the
   same; the scene tree, inspector and handles (move along each axis, turn about the vertical, scale evenly),
   `tst_studio` (8 cases in the real window, worked by clicks and drags); canonical QML in a project, one change one
   line, and `.qq.json` import, `tst_scene` (12 cases) and `tst_studio`; **Open in QQ Studio**, `check-qq-view.mjs`
   (the host is asked with the open project) and the host's `qq::` tests, with `qq_studio_starts_on_a_project`
   starting the deployed Studio on a real repository. All three Qt suites proven to fail by planted faults. Not in CI
   (ADR-083: no licensed Qt there), so the runs are recorded in HANDOFF.
2. [x] **Worlds play.** Play and Stop in the Studio with the scene restored exactly; physics (bodies, colliders, a
   character controller), input (keyboard, mouse, gamepad), 3D particles, spatial audio and QML behaviours reloaded
   while running; the QQ Player runs a scene natively and as a WebAssembly build in a browser, with its size and
   first-frame time measured. Depends on P3.1.

   **Done 10 October 2026** (HANDOFF §4 item 65). Each clause, with its evidence on the owner's machine (Qt 6.12, MSVC
   2022, an RTX 4060 laptop): Play and Stop with the scene, its unsaved state and the selection exactly restored,
   `tst_studio` (in the real window, Play from its button and Stop with Esc) and `tst_scene` (playing is a copy);
   bodies, colliders as their shapes, gravity and the character controller, `tst_play` (real frames); keyboard,
   gamepad (its state given: no pad attached) and mouse (a drag in the Studio turns the player), `tst_play` and
   `tst_studio`; particles counted in the frame and sound set up where it is and only in play, `tst_play`; logic rebuilt
   on each save while the world plays, `tst_play`; the Player natively, `tst_player` (the real executable: first frame
   1.4 s, 229 to 345 frames a second); in a browser, `build.ps1 -Web` with `player/measure-web.mjs` (headless Chrome,
   cross-origin isolated: first frame 5.8 to 6.0 s from the page loading, 220 to 227 frames a second, 44.1 MB served,
   11.7 MB brotli). Five planted faults, each caught by its own check. **In a browser, sound is not spatial**: Qt's
   spatial audio makes its sound on a worker, where the web has none, so there a sound's volume follows the player's
   distance, without panning. Not in CI (ADR-083), so the runs are recorded in HANDOFF.
3. [ ] **The agent designs and builds.** QQ Studio's MCP server (read the scene, add and change entities, write
   behaviours, play, stop, capture a frame, read the log) and a design loop that turns a description into a design
   brief, builds it, plays it, judges the captured frames and revises; the creator approves before anything is
   committed; the provider's key kept in the operating system's keychain and never logged. Proven by a scene built
   from a written description with no human edit, recorded with its frames. Depends on P3.2.

   **Under way, 10 October 2026** (HANDOFF §4 item 66): **the MCP server is built**: fourteen tools (read the scene and
   the kinds, add, change and remove entities, set the scene's look, write logic, play, stop, hold the controls, wait,
   place the eye, capture a frame, read the log) on a local pipe only the Studio's user can open, reached by any MCP
   client through `qq-mcp`; the agent cannot save or commit, and what it changes shows as unsaved. **The design loop is
   built** (HANDOFF §4 item 67, ADR-085): the Studio's Agent panel turns a description into a brief and a scene through
   Claude Opus 5.5 and the same tools, plays it, judges captured frames and revises, every step shown; the creator saves
   and commits, or undoes the run; the key is in Windows Credential Manager and never logged. Evidence: `tst_agent` (13
   cases run, the loop's against a stand-in for the API; eight planted faults caught), a session driven by the official
   MCP Python SDK, and the real endpoint reached over TLS. **Not yet: the proof**, a scene built from a written
   description with no human edit, recorded with its frames, made by whoever runs it with their own provider key and
   billed to that key: under ADR-086 the project pays for no generation, and the owner is not asked to fund it.
4. [ ] **Publishing a game.** A Publish action commits the scene through Qontrol and mints that commit as a Cartridge
   (a DRC-369 asset), signed by the launcher's vault behind the host dialog, asked over the local channel the
   launcher opens for the Studio it started; the Studio never holds a key. Depends on P3.2, M4.1 and L1.4's native
   dialogs being exercised, and on the owner's decision on confining a stranger's game
   ([`architecture/QQ_LOGIC_SANDBOX.md`](architecture/QQ_LOGIC_SANDBOX.md), proposed 10 October 2026).
5. [ ] **ARQADE delivery.** Published games appear in ARQADE; Play loads the WebAssembly QQ Player with the game,
   signed in through QOR ID when the player is; the TypeScript preview and its check are retired in this change
   (ADR-083 decision 6). Depends on P3.4 and P7 (ARQADE running), and on that same decision: the proposal is that
   strangers' games play only here, from an origin of their own, sandboxed in ARQADE's page.
6. [ ] **Collectibles, generation and the Mesh.** DRC-369 items as in-game collectibles; generative assets (models,
   textures, sound) from a description through the agent; games delivered over the Mesh; in-game purchases settled in
   CGT; access gating on paid collectibles. Depends on M4.2, M5.2, M5.3, M6.4 and M8. Generation is paid by the creator
   who uses it, on their own provider account (ADR-086); nothing is generated on an account of the project's.

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
   scenes, presets and themes. Depends on the indexer, M5.4. **A first slice without the indexer exists since 2
   October 2026:** the launcher's Market screen reads every DRC-369 listing from chain storage at the finalised
   block, bounded and paged. Not ticked: it has no listing model for the six kinds and no search.
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

#### P7: ARQADE (P7.2 depends on nothing)

The gaming platform, named by the owner on 4 October 2026: games played with a QOR ID, every owned collectible a
DRC-369 asset, CGT the only currency, and a developer SDK. Its starting point was an existing arcade on OpenAI Sites;
since 5 October 2026 it is hosted on Vercel from `products/arqade/` (ADR-074). [`blueprints/arqade.md`](blueprints/arqade.md); its decisions are ADR-069, accepted on 4 October 2026. No item
here sets an economic value; payouts are U-16.

1. [x] Baseline: the arcade's source in `products/arqade/` with its tests in CI, an integration inventory of every
   control, and the false claims removed (CRGT, DMRG, USD prices for Energy, the deleted chain's
   `chain_getBlockNumber`). Depends on ADR-069 accepted (ADR-050 decision 10).

   **Done 5 October 2026.** Imported 4 October (`f647c82`) and in CI since (`arqade` job, green on every run that completed it, from
   `37260506426`); the false claims removed the same day; the inventory is
   [`products/arqade/INTEGRATION_INVENTORY.md`](../products/arqade/INTEGRATION_INVENTORY.md), every control with its
   source, permission, persistence, failure and verification. Also: renamed ARQADE, powered by Demiurge, on screen
   (storage keys kept, so no visitor loses progress); two more false displays fixed (a sidebar "chain connection
   pending" that read nothing, and the terminal's `chain` printing "block #undefined"); the 13 inherited lint errors
   fixed and lint made a CI gate. Each solo game was played in headless Chrome after the fixes, with no page errors.
2. [x] Live reads from Demiurge Devnet, read-only: the genesis hash checked, the finalised head, an account's CGT
   balance in integer Sparks, and its DRC-369 inventory from finalised state, shown stale rather than zero on loss.

   **Done 4 October 2026.** `lib/chain.ts` checks the genesis and name and reads the finalised and best heads
   (`/api/chain`); `lib/account.ts` reads `System::Account` and `Drc369Api_assets_of` at the finalised block and decodes
   both as the runtime encodes them (`/api/chain/account?address=`), refusing an address for another network or with a
   broken checksum before any read; the QOR Identity screen looks an account up and keeps the last answer, marked
   stale, if a refresh fails. Evidence: 6 tests against bytes a real `--dev` node produced at `spec_version` 8 (two
   named assets, one a remix), whose balance `@polkadot/api` decoded identically; the reader against the live devnet
   agreed with `@polkadot/api` on the owner's launcher account (100,000 test CGT, no assets); the route answered in the
   site's own worker runtime; and the panel was driven in headless Chrome: a lookup, then a refused checksum.
3. [x] Verified QOR sign-in in the browser, bound to the account's `sub`, with the practice alias migrated only on
   proof of both identities. Depends on ADR-043 accepted and built, and on a way for ARQADE's server to verify a
   token without QOR ID's secret (ADR-069 decision 4).

   **Built 5 October 2026 (ADR-073), and live since that day.** QOR ID: `/oauth/authorize` (its own sign-in page), `/oauth/token`
   (PKCE always; rotating, single-use refresh tokens; reuse ends the session), `/oauth/userinfo`, `/oauth/revoke`, a
   registry in `QOR_OAUTH_CLIENTS`, sessions that record their app; 7 tests and the log check over the whole flow.
   ARQADE: a server-side session behind one HttpOnly cookie, `userinfo` on every check, sign-out that revokes at QOR ID,
   and a player bound to a QOR identity only on proof of both; 9 tests. Driven end to end in Chrome against a local QOR
   ID. QOR ID's side went live on 5 October (merged; ARQADE registered in Railway, read back from outside).

   **Moved to Vercel, 5 October 2026 (ADR-074).** The Sites host could not be published or checked from here, so the
   site became standard Next.js with Postgres, and the live arcade's players became QOR ID accounts (no ChatGPT sign-in,
   no Explorer alias). 41 tests, the arenas' SQL against Postgres; the whole flow in Chrome against Postgres 16 and a
   local QOR ID, a match created by the signed-in player. **Live since 5 October 2026** at
   `https://qor-arqade-tau.vercel.app`: the owner created the Vercel project with its database, and ARQADE's Vercel
   callback is registered at QOR ID. QOR ID is the only sign-in, and every player is a QOR ID account.
4. [x] A player approves a CGT payment from their own Vault, previewed and finalised, never with a key held by
   ARQADE. Depends on the signing path ADR-069 decision 5 chooses, with its own record, and for production on M5.2.

   **Accepted 5 October 2026: [ADR-076](decisions/ADR-076-arqade-hands-payments-to-the-launcher.md)** (option A for
   devnet play): a signed `qor://pay` link, the launcher's host dialog, the result read from finalised blocks; devnet
   only, at most 100,000 CGT per request. Corrected by [ADR-077](decisions/ADR-077-corrections-to-adr-076.md): the host
   dialog approves it (not Windows Hello), no fee is shown because none is charged, the devnet is checked by genesis.

   **Built 5 October 2026**, with tips to a game's creator as its first use (the owner's choice; ADR-071's
   tips). Launcher: `src/pay.rs` and `src/chain/pay.rs`, the `qor` scheme; 8 tests, among them ARQADE's own link pinned.
   ARQADE: the signed request, the scan of finalised blocks, the Tip panel, migration `0002`; 6 tests. A real `batch_all`
   on a local dev chain was found, and a forgery, a wrong amount and a wrong recipient were not.

   **Ticked on 6 October 2026: live on the devnet.** Merged in #8; launcher 0.1.7 and later carry `qor://pay`; the
   owner set `QOR_PAY_SIGNING_KEY` and `QOR_PAY_TIP_ADDRESS` in ARQADE's Vercel project, and tipped successfully
   through the launcher's host dialog on 5 and 6 October 2026. Devnet only, as ADR-076 decides: production still
   depends on M5.2.
5. [ ] Paid play on the devnet: a test-CGT entry finalised, one session authorised, the result validated by the
   server, and one DRC-369 trophy and one funded test-CGT payout delivered exactly once, surviving reload and retry.
   Depends on P7.3 and P7.4; the payout comes from the game's ARQ Wallet once P7.11 exists.
6. [ ] Card universes: cards as DRC-369 assets with declared sets, decks from finalised ownership, sales through
   `buy_exact` with the chain's royalties, and packs. Depends on P7.4; packs and editions on G-14 and G-15.
7. [ ] An owned asset that evolves through verified play, keeping its identity and history. Depends on M4.2's state
   and XP.
8. [ ] The developer kit: a typed game SDK over M5.1, Demiurge Devnet and a local node documented for developers,
   test CGT from the ecosystem's faucet, and three reference games (a trophy game, a card game, an evolving asset).
   Depends on M5.1, P7.5 to P7.7; published only after `beta.wire-format-frozen`.
9. [ ] Third-party games released through Market's listing model, played in an isolated origin, with game and rule
   versions pinned for every paid session. Depends on P5.3, P7.8 and a record for third-party sign-in.
10. [ ] Production payouts and any paid prize. Depends on U-16 decided and the owner's legal review.
11. [ ] ARQ Wallets: one keyless payout account per published game, derived from its DRC-369 Cartridge and governed by
    whoever holds it, paying only within a policy on chain, each outcome once, with loosening and withdrawals delayed,
    prizes reserved before paid rounds open, and payouts below the existential deposit accrued; "ignition" in the SDK.
    **Its chain half is built** (4 October 2026, `pallet-arq-wallet`): everything above at `spec_version` 7, **on Demiurge Devnet since
    that day**; the held prizes at `spec_version` 8, **on the devnet since the same day**. The SDK derives a wallet's
    address and the outcome and round ids, and builds the calls' arguments; sending them waits on M5.1. The owner's direction and name of 4 October 2026; ADR-070, accepted that day, and
    [`products/arqade/sdk/docs/arq-wallet.md`](../products/arqade/sdk/docs/arq-wallet.md). Depends on
    `pallet-arq-wallet` and U-16's bounds.
12. [ ] Self-publishing: a project submitted at any stage becomes its DRC-369 Cartridge, whose profile manifest is the
    store page and whose revisions are the devlog; four stages (concept, prototype, early access, released) publish
    themselves once the SDK's `checkReadiness` finds nothing missing; a game priced free or up to 10,000 CGT (the
    owner's ceiling) and bought as a DRC-369 licence. ADR-071, accepted 4 October 2026. The profile, its checks and the price check
    exist in `products/arqade/sdk/`; the rest depends on P7.4, P7.11, and an interim content
    store (G-3).
13. [ ] In-game purchases as non-fungible DRC-369 items declared in the profile, resold under the chain's royalties.
    Depends on G-17 (primary sale) for selling copies; until then, items minted ahead and listed.
14. [ ] Backing in CGT: all-or-nothing campaigns held in a keyless Campaign Vault with milestone release, memberships,
    and tips; backers receive DRC-369 rewards and never proceeds. Devnet first. Depends on the ARQ Wallet's pallet
    family, U-17 and the owner's legal review; recurring memberships on M5.2.
15. [ ] Agentic creation: ARQADE's tools on ADR-010's MCP server, used by an external LLM under an agent key the
    developer authorised; the model prepares, the developer signs what is public or moves CGT. Depends on M5.3, and on
    M5.2 for capped agent signing.
16. [ ] Levels and tasks across the ecosystem (ADR-078): one level and XP per QOR ID, granted only by QOR ID or an app's
    server for checked tasks, each once; the level bubble and the XP bar in the launcher and ARQADE; ring styles and
    themes as unlocks; at most three accounts per network address. Depends on nothing.

    **Built 6 October 2026, and live in QOR ID and ARQADE the same day; in the launcher from 0.1.8.** QOR ID: `progress_events`, `welcome_grants`, `signup_addresses`
    (migration 020); `GET /api/v1/profile/progress`, `POST /api/v1/profile/progress/tutorial`, `POST /oauth/progress`
    (an app's own tasks, with its secret and the player's token); XP for a verified email, a linked key and a sign-in to
    ARQADE; userinfo carries the progress; three sign-ups per address in 30 days, read from Railway's `X-Real-IP`, kept as
    a keyed hash. 157 tests then; 166 with avatars (ADR-079), the same day. The launcher: the level bubble on the avatar, the XP bar with the next unlock, the tutorial
    reported. ARQADE: the bubble, the level row on the QOR Identity card, a first match and a first tip reported.
    Checked end to end in Chrome against a local QOR ID: signing in to ARQADE granted 10 XP and the card showed it.
    **Ring styles are drawn** in the launcher since ADR-079 and ADR-080 (6 October 2026): a plain edge at level 0,
    a glowing ring from level 1 and a wider glow from level 6, on `main` after 0.1.8 and not yet in a build. Themes as
    unlocks are not built, so the item stays unticked.
17. [ ] The welcome grant (ADR-078): 100 CGT once per new QOR ID when the tutorial, the email and a key are confirmed,
    from the Welcome account; test CGT on the devnet first. Real CGT depends on U-18 and the owner's legal review.
    **Owed grants are recorded** by QOR ID (6 October 2026); paying them from the Welcome account is not built.
    Since 8 October 2026 the owner can list what is owed (`GET /api/v1/admin/grants/owed`: account, chain
    account, amount; no email address) and mark one paid against the hash of its finalised transfer
    (`POST /api/v1/admin/grants/{user_id}/mark-paid`, audited, refused for a grant already paid). Neither
    moves CGT; the payer that holds the Welcome account's key is still to build.
18. [ ] ARQADE inside the launcher (ADR-078 decision 5): a Play section, signed in with the launcher's QOR ID, payments
    approved in the host dialog. Depends on P7.4.
    **Partly built, 8 October 2026:** ARQADE on the rail opens the site in a window of its own, made by the host
    (`src-tauri/src/arqade.rs`) and given no capability, so its page cannot call the host. The host decides
    every address it loads: ARQADE and QOR ID stay, a `qor://pay` link goes straight to the host dialog with no
    round trip through the operating system, any other web address opens in the browser, anything else is
    refused (five host tests). **Signed in with the launcher's session since 8 October 2026:** a signed-in launcher
    opens the window at ARQADE's sign-in, stops it on its way to QOR ID's page, and has QOR ID issue ARQADE's code for
    that exact request with the launcher's own session (`POST /api/v1/oauth/handoff`: a person's own session only,
    never an app's; checked as the sign-in page checks; the code bound to ARQADE's PKCE challenge, so nothing new
    travels in an address). Without a session, or if QOR ID refuses, QOR ID's page shows. Three QOR ID tests, three
    more host tests, and the flow is in the log check. **Not yet:** QOR ID deployed with the route, and a person using
    it in the running launcher, which come before the tick.

## 8. Scope

### Active

- `chain/`: the Substrate L1, and **the only chain in this repository** (ADR-013; located and named by
  ADR-032, which decided Q-15).
- `services/qor-auth/` and `tools/qor-launcher/`.
- `products/arqade/`: ARQADE, the first product to enter active scope by ADR-050's decision 10 (ADR-069, accepted
  4 October 2026). TypeScript, not a Cargo workspace; its CI job type checks, tests and builds it.
- `tools/qor-installer/`, when M8 reaches it.

The other product tracks (§7) are planned, not active scope. Qontrol's Projects surface, its `qontrol-git`
helper, QFX layer one and Market's first slice are part of `tools/qor-launcher/`. A product gets a directory under `products/`
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

The only verified ways to run the current stack are
[`scripts/run-local-stack.md`](../scripts/run-local-stack.md) on one machine, and, for the live services,
[`chain/DEPLOY-RAILWAY.md`](../chain/DEPLOY-RAILWAY.md) (from `chain/Dockerfile`) and
[`services/qor-auth/DEPLOY-RAILWAY.md`](../services/qor-auth/DEPLOY-RAILWAY.md).

## 9. Risks being carried

- **The base layer is in transition.** `chain/` produces and finalises blocks, but M3 is not complete:
  M3.2, the chain specification at the base supply, is open. Above the base layer, DRC-369 with royalties,
  a settled sale and nesting, and ARQ Wallets exist; agent rails and the economics do not.
- **The economics are not built.** There is no transaction payment, no issuance and no treasury, because
  each needs a value OPEN-1, OPEN-2 or OPEN-4 leaves undecided. A chain that charges nothing is not a
  chain anyone has paid to attack, and none of the pressure those mechanisms carry has been observed.
- **QOR ID is deployed** (`https://id.qorsync.dev`). The §7.1 defects are fixed, all seven items ticked;
  what remains is named in `SECURITY.md`, among it that an access token alone can still register an agent
  until L5.1.
- **Irreversible choices are still open.** The genesis split, issuance, decay curve, key scheme and
  address format are all foreclosed by mainnet genesis. The DRC-369 wire format is foreclosed by SDK
  publication.
- **Seven products compete with the chain for the same weeks.** ARQADE is live on Vercel and Market has a
  first slice in the launcher, beside Qontrol's and QFX's; QOR Engine, GNOSIS and Stream have not started.
  Every product track has items that wait on M4, M5 or M8. M2.1 and M2.3, which held all chain work back,
  were both ticked on 22 September 2026.
- **Unaudited.** Nothing here should hold real value.
- **No production network.** A public devnet exists at `wss://rpc.qorsync.dev`; it holds test CGT only, and
  nothing on it holds value.
