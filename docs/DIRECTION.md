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
| QOR Launcher (`tools/qor-launcher`) | Version 0.1.14 (`tauri.conf.json`), published for testers on 18 October 2026 as the GitHub Release `launcher-v0.1.14`; unsigned, with no update channel (L6). A Tauri 2 desktop application whose Rust host alone holds keys. Since ADR-104 that host is moving into a core shared with the Android launcher (L13.2): Sparks, errors, the vault and profiles so far. Built: a profile for each QOR ID on a PC, opened with a PIN (ADR-099); the wallet's key in the operating system's keychain with no lock screen (ADR-056); QOR ID sign-in by key; amounts in Sparks (ADR-101); sending, minting, the Inventory, trading, selling and buying (L4.2, L4.4, L4.5); the Market (no search, no indexer); Projects (Qontrol, P1.1, P1.2); the gates dashboard; `qor://pay` behind the host dialog (ADR-076, ADR-077); levels and avatars; Social (ADR-091 to ADR-093); the Library with QOR Media (ADR-094); the Mesh, paid on the devnet's runtime 9 (ADR-095 to ADR-097); ARQADE's Qt app docked over the launcher (ADR-102); QQ Studio started and answered through the product host (ADR-046, ADR-088); six QFX backdrops. 270 host tests (suite `launcher`), 73 core tests (suite `launcher_core`) and fifteen browser checks (`npm run check`), all run in CI. |
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

1. [ ] The Mesh: seeder work verification (U-6) and seeder payment. Carried by L10 (ADR-095): seeders paid from the
   uploader's hosting escrow, for work proven under the scheme the owner accepts (`architecture/MESH_PROOFS.md`).
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
3. [x] Amounts never pass through JavaScript numbers; no raw ember counts in user-facing views; no
   2-decimal fallback.
4. [ ] Host-side confirmation before any signature, except the QOR ID sign-in or name claim that directly
   follows unlocking or creating the vault, which opening the vault approves (ADR-016; since ADR-056 nothing is asked to open it). The webview cannot
   repoint the RPC or QOR ID
   endpoints without that confirmation. Implemented and unit-tested; it stays unchecked until the native
   dialogs are exercised in a running launcher, against the list in `HANDOFF.md`.

   **Partly exercised, 5 and 6 October 2026:** the owner approved `qor://pay` tips in the host dialog of an
   installed launcher against Demiurge Devnet (P7.4).

   **The transfer and the RPC endpoint, both ways, 10 October 2026** (HANDOFF §4 item 72): in the installed launcher
   0.1.8, against a local `demiurge-node --dev`, the owner approved and declined a transfer and approved and declined a
   change of RPC endpoint, to the local node and back to Demiurge Devnet. The approvals are read back from the chains
   (the account's transfers finalised, its nonce, and the launcher's saved endpoint); the declines are the owner's
   account, since a decline leaves nothing to read. Still not exercised by a person: the confirmation before the QOR ID
   endpoint changes, and a key sign-in after QOR ID was unreachable at unlock. The item stays unchecked until they are.
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

1. [ ] Library: install, delta patch and launch. Its local half, a person's own files and QOR Media, is L9 (ADR-094),
   which needs nothing in M8; owned work joins it here.
2. [ ] Market.
3. [ ] Social (named Agora until 2026-09-28, by the owner). **Brought forward of M8 by ADR-091** (15 October 2026),
   since nothing it needs is in M8; its steps are L8, and it is ticked when L8.1 to L8.7 are.
4. [ ] Mesh seeding. Carried by L10 (ADR-095).
5. [ ] Studio inside the launcher.

#### L8: Social (ADR-091, ADR-092, ADR-093; depends on nothing in M8)

Rooms, direct messages, presence and profiles per QOR ID, in a service of its own (`services/social/`) with the
launcher as its first client; World, friends and circles, and translation on the reader's PC (ADR-092). Design:
[`blueprints/social.md`](blueprints/social.md). L8.1 to L8.7 and L8.11 to L8.15 are release 1.

1. [x] QOR ID for Social: token introspection for registered services (`POST /oauth/introspect`); an account on an
   automatic `key_…` name chooses its username once; another person's public profile by username. Each flow joins the
   log check. Done 2026-10-15: merged with pull request #31, CI green on `main` at `76ffbf4`, and live at
   `id.qorsync.dev` (deployed from `main`; migration 023 applied; checked by an unauthenticated call answered
   `invalid_client`). 176 tests; each new one failed against a planted fault first.
2. [x] The Social service: rooms (public, and private by invitation) with owner and moderator roles; direct messages;
   messages sealed at rest, with replies, reactions, edits and deletes; read positions and unread counts; mentions;
   blocks; reports. One WebSocket per client, typed frames, versioned; rate limits and size caps; tests against
   Postgres, the log check and bound SQL from the first commit. Done 2026-10-15: `services/social/`, merged with pull
   request #31, its CI job green on `main` at `76ffbf4`; 29 tests, each rule failing against a planted fault first. Not
   deployed: that is L8.6.
3. [x] Profiles per QOR ID: a display line, a bio, an accent, a banner from the QFX backdrops, a showcase of DRC-369
   assets the account holds, and a card opened from any message; username, avatar and level as QOR ID returns them.
   Done 2026-10-16 with L8.5.
4. [x] Presence: online and away, typing, a status line, and what someone is playing when a product says so. Done
   2026-10-16 with L8.5.
5. [x] The launcher's Social surface, its first client: the socket held by the Rust host, events to the interface; rooms,
   conversations, composer, profiles and notifications, in the launcher's themes and QFX; the username claim on entry,
   and the existing QOR ID claim for a person without one. Done 2026-10-16: merged as #33 and #34, and the owner's first
   conversation, in a launcher built from #34 against the deployed Social, answered "Successful. Looking great so far".
6. [x] Deployed beside QOR ID on Railway (devnet stack), with CI jobs (format, lints, tests with Postgres) and a
   dependency audit. Done 2026-10-16: live at `wss://social-production-e05c.up.railway.app/v1/socket`, its database
   `Postgres-Yj3Q`, registered with QOR ID as a service; checked from outside (`/ready` 200, and a hello with a token that
   is not live answered `unauthorized`, which needs QOR ID to accept Social's credential). The CI job (#31) and the audit
   (#32) are in `main`; the setup is recorded in `services/social/DEPLOY-RAILWAY.md`.
7. [ ] The owner's decisions before it is open to others: how long messages are kept, and the content policy reports
   are judged against.
8. [ ] Release 2: end-to-end encrypted direct messages keyed from the vault, designed in a record of its own first.
9. [ ] A web client on the same service, and ARQADE's chat moved onto Social.
10. [ ] Voice, feeds, and search across rooms.
11. [ ] World (ADR-092 decision 1): one room everyone is in from their first hello, opened when Social is, kept in full;
    its moderators named in `SOCIAL_WORLD_MODERATORS`, who delete and mute for ten minutes to a week; its own sending
    pace; World making nobody a contact. Merged as #35 (`9a3895f`) and live since 17 October 2026, 02:14 UTC; the owner's first look is to come.
12. [ ] Friends and circles (ADR-092 decision 2): requests by username, answered by the one asked; friends see each
    other's presence; a block ends a friendship; circles, private to their owner, open a private room in one step.
    Merged as #35 (`9a3895f`) and live since 17 October 2026, 02:14 UTC; the owner's first look is to come.
13. [ ] Translation on the reader's PC (ADR-092 decision 3): World and public rooms in the reader's language by
    Mozilla's Bergamot engine and models, downloaded once and checked against Mozilla's hashes; each message carrying the
    language it was written in; the original one click away. The real engine translated Mozilla's real Spanish model on
    the owner's PC. Merged as #35 (`9a3895f`) and live since 17 October 2026, 02:14 UTC; the owner's first look is to come.
14. [ ] Each author's avatar, level and username in chat (ADR-092 decision 4). Merged as #35 (`9a3895f`) and live since 17 October 2026, 02:14 UTC; the owner's first look is to come.
15. [ ] Room levels and standing (ADR-093): Owner, Admin, Mod, VIP and Member, each acting only below itself; Elite
    accounts, staff in every room they are in, made only by godmode; godmode above every Owner; badges beside names;
    mutes in any room but a direct one. `SOCIAL_GODMODE` and `SOCIAL_WORLD_MODERATORS` are `godmode`, in effect since
    that deploy. Merged as #35 (`9a3895f`) and live since 17 October 2026, 02:14 UTC; the owner's first look is to come.

#### L9: The Library and QOR Media (ADR-094; depends on nothing in M8)

Everything a person has in one SteamOS-like view inside the launcher, searched instantly, and opened in QOR Media, a
Qt 6 player, viewer and editor docked inside it. Design: [`blueprints/library.md`](blueprints/library.md). L9.1 to L9.5
are release 1.

1. [ ] The index: folders chosen (Videos, Music, Pictures and Documents offered), scanned and watched, with metadata and
   full-text search, on this PC only. Merged as #37 (`95e554e`) on 17 October 2026, 12:59 UTC; the owner's first look is to come.
2. [ ] The Library view: a hero, shelves, folders and search, with keyboard and controller navigation. The controller
   through the Gamepad API. Merged as #37 (`95e554e`) on 17 October 2026, 12:59 UTC; the owner's first look is to come.
3. [ ] QOR Media docked in the launcher: the player for every format FFmpeg reads, and the image viewer. In
   `tools/qor-media/`; started through the launcher's product host by its own test, not yet opened from the Library on
   screen. Merged as #37 (`95e554e`) on 17 October 2026, 12:59 UTC; the owner's first look is to come.
4. [ ] The editor: plain text, Markdown and code. Merged as #37 (`95e554e`) on 17 October 2026, 12:59 UTC; the owner's first look is to come.
5. [ ] PDF: viewed, annotated, highlighted, filled, signed, pages reordered, rotated and deleted (Qt PDF and PDFium).
   Can start: the owner's Qt 6.12 has Qt PDF (`Qt6Pdf`, `Qt6PdfQuick`, the `QtQuick.Pdf` module) since 27 September 2026.
6. [ ] Owned games and work in the Library (L7.1; depends on entitlements, M8).
7. [ ] Release 2: Office documents viewed and edited through LibreOffice's engine, its own record first.
8. [ ] Pictures beyond Qt's plugins: HEIC, AVIF, JPEG XL and camera RAW.

#### L10: The Mesh (ADR-095; carries M8.1 and L7.4)

A peer-to-peer network inside every launcher, on iroh, holding published work and each person's private backup;
seeders paid in CGT from the uploader's hosting escrow, for proven work only, with no CGT created. Design:
[`blueprints/mesh.md`](blueprints/mesh.md); proofs proposed in [`architecture/MESH_PROOFS.md`](architecture/MESH_PROOFS.md).
No item sets a price, rate, pool or burn share. **Order (ADR-096 decision 5):** L10.3 and L10.4, the chain's half, before
L10.2, because finding who holds a root reads the escrow's assignments on chain.

1. [ ] The network, and one's own published work seeded: an iroh node in the launcher's host, seeding opt-in with disk
   and upload limits, the temporary content store's roots served and fetched verified. No CGT moves. Built 2026-10-17 on
   branch `session/mesh`, not merged: `src-tauri/src/mesh/` and the Mesh page, between two launchers on one PC; not yet
   between two PCs over the internet.
2. [ ] Seeding others' published work: finding who holds a root, fetching and verifying it, holding it within the
   person's limits, and the Mesh page showing what is seeded and served. **Built 2026-10-17 on branch `session/mesh`, not
   merged:** a root is found through the chain (an escrow's seeders, then its uploader, each dialed by the node it bound)
   and fetched verified, from the Mesh page by its fingerprint; while seeding with the Mesh grant (ADR-097) the launcher
   takes on copies escrows still ask for, within the disk limit. Shown between three launchers on one PC against a
   development chain; not yet between PCs over the internet. Live on the devnet since its upgrade to runtime 9
   (17 October 2026). **A file is published to the Mesh from the Library's file menu** (launcher 0.1.11): minted as an
   asset whose manifest names it alone, after a dialog saying it becomes public.
3. [ ] The proofs: storage challenges and serving receipts, as ADR-096 accepts them (U-6). QOR ID attesting receivers on
   chain is a later step of its own. **Both halves built 2026-10-17 on branch `session/mesh`, not merged:**
   `pallet-mesh` checks answers with `mesh-proof` and pays receipts; the launcher answers each period's challenge with a
   proof read from its store a piece at a time, signs receipts for what it downloads, and claims those it is sent, all
   under the Mesh grant (ADR-097). Seen paid for both on a development chain, whose period is a minute.
4. [ ] The hosting escrow on the devnet: deposits, the uploader's prices, assignments, payment for proven work, unspent
   escrow returned; the Mesh page shows what was paid. In `pallet-mesh` (ADR-096). Depends on L10.3. **Built 2026-10-17
   on branch `session/mesh`, not merged, at `spec_version` 9:** the pallet, and in the launcher an escrow opened for
   one's own work at one's own prices after a dialog, and the Mesh page showing what was paid. **Not on the devnet**: the
   runtime upgrade is the owner's to sign. Closing an escrow from the launcher is not built.
5. [ ] Private backup: encrypted on the PC with keys from the vault, erasure-coded across seeders, repaired, and
   restored from the recovery phrase on a new PC. Depends on L10.4.
6. [ ] Demiurge's own relays, in place of n0's public ones.
7. [ ] The takedown list seeders refuse, kept by the root origin (ADR-096).

#### L11: Sparks and embers (ADR-101; depends on nothing)

1. [ ] A Spark is a thousandth of a CGT and the ember the atomic unit, everywhere: the chain's documentation and
   constants (`SPARK`, embers in every comment), the launcher's parsing and display (CGT to three decimals or Sparks,
   exact beyond that, embers on the wire with the old `sparks` name read for one release), QOR ID, ARQADE and its SDK,
   QQ, and every current document. No amount changes.

#### L12: The launcher on Qt, in stages (ADR-103)

1. [ ] Stage 1: new surfaces are Qt applications docked under the product host (QQ Studio, QOR Media, ARQADE).
2. [ ] Stage 2: a Qt shell linking the Rust host through a bridge, the host and its tests unchanged, the shell unable to
   reach a key and a check proving it. Its record is ADR-104, which starts it on Android (L13) through a small C bridge
   of the core's own instead of CXX-Qt.
3. [ ] Stage 3: each React surface rebuilt in the Qt shell, one at a time, each with its own checks before the React one
   is retired.

#### L13: The launcher on Android, on Qt (ADR-104)

The owner, 18 October 2026: "I'd like to build a android varient of the QOR Launcher as well". Qt 6 for Android over
the Rust core, app ID `app.qorsync.dev`, Android 9 and later, delivered as a signed APK on GitHub. In this order:

1. [ ] The toolchain, the bridge and a running APK. The Android toolchain is installed by
   `tools/qor-launcher/shell/android-toolchain.ps1`. `tools/qor-launcher/core/` is the core with its C bridge, and
   `tools/qor-launcher/shell/` is the Qt shell. It builds an arm64 APK that starts and answers through the core.
   **Built, 18 October 2026, and merged (#50, at `15cfa12`):**
   - The toolchain is installed: JDK 21, SDK 36, NDK r27c, Rust's Android targets, the emulator.
   - The core's C bridge is tested.
   - The shell's arm64 and x86_64 APKs build and are debug-signed.
   - The x86_64 build ran on an Android 16 emulator: the core answered through the bridge, and read "1.25 CGT" exactly
     as 1,250 Sparks.

   **Not yet:** on a phone.
2. [ ] The core moved out of the Tauri crate, module by module with its tests: Sparks, errors, the vault, profiles, QOR
   ID's client, the chain client, Social's client, the Library's index and the Mesh. The Windows launcher keeps every
   test green throughout.

   **Moved so far:**
   - Sparks and errors (18 October 2026, #50).
   - The vault, profiles and the confirmation they ask through (19 October 2026, merged in #56 at `c4972b0`):
     - Their 52 tests moved with them. The core has 73 tests and the Windows launcher 270, and both suites pass.
     - The arm64 APK still builds, with sp-core, schnorrkel and secp256k1's C library compiled for Android.
     - On Android the core has no keychain, because `keyring` would keep a wallet's key in memory there. L13.3 gives
       the vault Android's Keystore instead.

   **Next:** QOR ID's client, then the chain client.
3. [ ] Keys on Android: the wallet's key sealed with its profile's PIN and wrapped by Android Keystore. The Gate, the
   profiles and the Vault with Sparks in Qt, and a check that the shell cannot read a key.
4. [ ] QOR ID's sign-in and Social.
5. [ ] ARQADE: its Qt module in the same process, signed in through the core. Depends on P7.19.
6. [ ] The Library: the phone's own photos, music and videos through MediaStore, each opened in the phone's app.
7. [ ] The Mesh: off until turned on, Wi-Fi only unless the person chooses otherwise.
8. [ ] A signed APK on a GitHub Release, `android-v<version>`, with a download page. The test key is kept off the
   repository; the Google Play key is the owner's.

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
Its eight Qt Test suites run on the owner's machine, not in CI, which has no licensed Qt.

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
   1.4 s, 229 to 345 frames a second; corrected 10 October: that game was missing its one model, which had failed to
   load, and with it the first frame is about 1.7 s, HANDOFF §4 item 71); in a browser, `build.ps1 -Web` with `player/measure-web.mjs` (headless Chrome,
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
   dialogs being exercised; the signing dialog it uses was exercised both ways on 10 October 2026 (L1.4, HANDOFF §4
   item 72), and what L1.4 still lacks concerns QOR ID, not signing. The QQ Player's lockdown it waited on (ADR-087 decision 2) is built (10 October 2026,
   HANDOFF §4 item 70).

   **Built 10 October 2026** (HANDOFF §4 item 73), under ADR-088: a Cartridge holds the manifest of the project's whole
   tree at the commit published, with that commit; the first Publish mints it and each later one revises it. The local
   channel is ADR-046 built for its first product: the launcher's `host` module and its `qor-host-proof` crate, the QOR
   host protocol (`tools/qor-launcher/host-protocol/`), and the Studio's `LauncherLink`. Evidence on the owner's
   machine: the launcher's host checks (14, and one against a development node that commits, mints, revises, leaves an
   unchanged project alone and declines, through the real channel), `tst_publish` (9 cases), and eight planted faults,
   each caught. **Stays unchecked until the owner has published from QQ Studio** started by a launcher built with this
   change, approving in its dialog.
5. [ ] **ARQADE delivery.** Published games appear in ARQADE; Play loads the WebAssembly QQ Player with the game,
   signed in through QOR ID when the player is; the TypeScript preview and its check are retired in this change
   (ADR-083 decision 6). Depends on P3.4 and P7 (ARQADE running). Under ADR-087 strangers' games play only here, in a
   sandboxed frame, from a site of their own (a separate `*.vercel.app` project, a registered domain before launch),
   asking ARQADE for anything through a narrow message interface.
6. [ ] **Collectibles, generation and the Mesh.** DRC-369 items as in-game collectibles; generative assets (models,
   textures, sound) from a description through the agent; games delivered over the Mesh; in-game purchases settled in
   CGT; access gating on paid collectibles. Depends on M4.2, M5.2, M5.3, M6.4 and M8. Generation is paid by the creator
   who uses it, on their own provider account (ADR-086); nothing is generated on an account of the project's.

**Real games, in 3D and in 2D (ADR-105, 19 October 2026).** One runtime: 2D is the same world through an orthographic
camera. The agent's tools learn each new kind in the same change. Each item is proven by a sample game in
`products/qq/samples/`, made only from kinds and logic files and played by the Qt Test suites. These items depend on
P3.2 and on nothing on chain, and run in this order:

7. [x] **Gameplay in logic.** Contacts on bodies and a **Trigger** kind that reports what enters and leaves it;
   raycasts; making and removing entities while the world plays; timers; forces and impulses; kinematic bodies.

   **Done 19 October 2026, merged (#55, at `ae11498`):**
   - Logic gains `onStarted`, `onContact`, `onEntered`, `onExited`, `ray`, `spawn`, `remove`, `after`, `every` and
     `cancel`. Timers run on the logic's own clock.
   - A Shape can be `kinematic`, led by its position, and a dynamic one has `push`, `applyForce`, `setVelocity` and
     `setSpin`. `Trigger` is a new kind, in the Studio's Add menu and the agent's instructions.
   - The proof is **Orb Run** (`products/qq/samples/orb-run/`): one scene and one logic file. Orbs appear on a timer,
     are gathered by walking through them, and are replaced, and a boulder is rolled at the player.
   - Evidence, on the owner's PC: `tst_play` passes 18 cases (7 new, the sample among them), and the other seven
     suites pass. Five planted faults were each caught: the contact relay cut, a removed body left behind, timers
     never run, the player unheard by triggers, and rays with no view.

8. [x] **The game's interface.** A **Screen** kind drawn over the world, with text, bars, images and buttons that logic
   sets; title, pause and message screens; the design system's tokens; reduced motion.

   **Done 19 October 2026, merged (#57, at `f31443f`):**
   - ADR-105's "Screen kind" is built as five kinds on one base (`ScreenPart`): `ScreenText`, `ScreenBar`,
     `ScreenButton`, `ScreenImage` and `ScreenPanel`. Each is a plain entity the Studio's inspector and the agent's
     tools already edit, and each belongs to a named screen (`hud`, `title`, `pause`, …).
   - They are drawn on the World's overlay, placed by an anchor and insets in points of a screen 720 tall.
   - The scene's `screens` field names those shown at the start.
   - Logic has `showScreen`, `hideScreen`, `onlyScreen`, `pause`, `resume` and `onClicked`. A paused world holds
     physics, logic and its timers, the player and particles still, and its screens still answer.
   - `Look` carries the interface's style:
     - Cinzel and Barlow Semi Condensed, the homepage's and ARQADE's faces, carried in the runtime under the SIL Open
       Font License;
     - the launcher's inks with ARQADE's gilt;
     - the system's reduced-motion setting, read from Windows' own animation setting, under which screens appear at
       once.
   - Orb Run now opens on its title with the world paused, keeps its score on a HUD, and pauses behind a pause screen.
   - Evidence, on the owner's PC: `tst_play` passes 23 cases, 5 of them new, with Orb Run's title, HUD and pause
     screens among them. All eight suites pass in one run of `build.ps1`: 88 cases. Five planted faults were each
     caught: a button that answers on any screen, physics that ignores a pause, logic that ignores a pause, reduced
     motion ignored, and a part drawn whatever its screen.
9. [x] **Input actions.** Named actions with default bindings for keyboard, mouse, gamepad and touch, rebound by the
   player; the pointer picking what it is over. Move, look, jump and run stay as built-in actions.

   **Done 19 October 2026, merged (#59, at `b483e7a`):**
   - **Declaring actions.** A scene names an action with an **InputAction** entity: keys and mouse buttons ("E,
     Mouse Left") and gamepad buttons ("X, RT"). Every pad button is read, the triggers among them. A
     `ScreenButton` with an `action` holds it under a finger, which is how touch controls work.
   - **Built-ins.** Jump and run are built-in actions with their old bindings, so a scene or a player can rebind
     them. Move and look stay fixed.
   - **Logic** gains `onPressed`, `onReleased`, `held`, `rebind` (the player's next press becomes the binding for
     that device, and that press does nothing else), `bindingOf`, and `pointing()`, the entity under the pointer.
   - **The agent's** `input_set` holds actions by name.
   - **Input events are read once, as they reach a window.** Qt then sends the same key on to the item with focus,
     where it would have been seen twice.
   - **Orb Run** pauses on P, on the pad's Start, or on its Pause button.
   - **Evidence, on the owner's PC:**
     - `tst_play` passes 26 cases, 3 of them new: an action from each device and a finger, rebinding keys, pad and
       jump, and the pointer picking a shape, the sky and the ground.
     - All eight suites pass in one run: 91 cases.
     - Five planted faults were each caught: a press never announced, the naming press also firing, a finger
       holding nothing, picking ignoring the pointer, and every event read twice.
10. [x] **2D.** `view: "2d"` with whole-number pixel scaling; **Sprite** (images and sheets, frame animation, lit or
    unlit); **TileMap** (instanced, imported from Tiled's JSON); bodies locked to the plane; a kinematic 2D player for
    platform and top-down games. The launcher preview's 2D scenes import as 2D QQ scenes.

    **Done 19 October 2026, merged (#60, at `6b58167`; CI passed every job on it, run 39357583640),** with
    [ADR-106](decisions/ADR-106-2d-scenes-say-whether-their-art-is-pixel-art.md) amending ADR-105 decisions 1 and 5:
    - **The view.** A 2D scene is the same world through an orthographic camera on the plane z = 0, over a backdrop of
      its sky's colours.
      - **Pixel art** (`pixelArt`, on by default) is scaled by whole numbers of device pixels to an art pixel, as many
        as fit `viewHeight` units, with the camera on whole device pixels, and shown in exactly its painted colours: no
        bloom, tonemapping or dithering.
      - **Drawn art** fills `viewHeight` exactly and keeps the 3D look.
    - **Sprite:** an image or a frame of a sheet, played from `first` for `count` frames at `fps` and held by a pause.
      Crisp or smooth, lit or not, flipped, and a static, dynamic or kinematic body that logic pushes as a Shape's.
    - **TileMap:** a level drawn in Tiled and saved as JSON, its tileset external (`.tsj` or `.tsx`) or inside it, its
      tiles as CSV or zlib-compressed Base64, flips kept.
      - One layer each, drawn as one mesh.
      - A solid map collides through the fewest rectangles that cover its tiles.
      - The map marked `bounds` is where a following eye stops.
      - A map copied into a project brings its tileset and image, and one whose tileset is outside its folder is
        refused with the reason.
    - **Bodies on the plane:** a 2D scene's dynamic shapes and sprites move only on it and turn only about its depth
      axis.
    - **Player2D:** a platformer's hero or a top-down one, under the same input, carrying a Sprite as its look and
      saying what it is doing (`activity`), so logic plays the frames. `respawn()` puts it back. Its capsule is built
      PhysX's 0.1 contact offset smaller all round, so it stands on the floor, not a pixel and a half above it.
    - **Importing:** the preview's scenes import as 2D scenes of drawn art, framed as they were made, their background
      as the backdrop.
    - **The Studio** edits a 2D scene flat: a drag pans, the wheel zooms, and the handles move things on the plane and
      turn them about its depth axis. Its Add menu has a 2D submenu, and the agent's instructions name the kinds.
    - **Ember Climb** (`samples/ember-climb/`) is a pixel-art platformer made only of kinds and logic.
      - Its art and Tiled level are written by a script, standard library only.
      - The hero runs and jumps through eight embers spawned by logic. A pit puts the hero back at the start, and the
        last ember lights the forge.
      - It has title, HUD, pause and win screens.
    - **Evidence, on the owner's PC:**
      - `tst_play` passes 32 cases, 6 new: a crisp sprite drawn pixel for pixel at a whole zoom (square art pixels,
        sharp edges, painted colours, flipped); a sheet playing its frames and holding when paused; a Tiled level drawn
        tile for tile and stood on, run to its wall and jumped from; a top-down player carrying its look; a dynamic
        body kept on the plane; and Ember Climb played to its end and again.
      - `tst_scene` gains five cases (ten checks), `tst_studio` and `tst_player` one each, and `tst_lockdown` checks
        that a game measures no picture and reads no map or tileset outside its package.
      - All eight suites pass in one run of `build.ps1`: 109 cases.
      - Seven planted faults were each caught:
        - a sheet read upside down;
        - pixel art scaled to fit;
        - Tiled's mirror flag ignored;
        - PhysX's skin not allowed for;
        - a dynamic body free to drift in depth;
        - a tileset read from outside the package;
        - the eye shown past the level's edges.
    - **Found and fixed on the way:**
      - bloom brightened every painted colour;
      - the Studio's handles were not drawn after the view went flat;
      - Again in Ember Climb gathered the last ember where the hero stood.
    - **Not yet read from Tiled:** object layers, tile animations, one-way platforms and slopes.
11. [x] **Fixed steps and replay.** Logic at 60 steps a second whatever the frame rate; seeded random numbers; a run's
    inputs recorded and replayed with no window to the same outcome. A rewarded outcome never depends on PhysX
    (ADR-102 decision 5).

    **Done 19 October 2026, merged (#61, at `6b03027`; CI passed every job on its head and on the merge, runs
    39361627242 and 39363830555):**
    - **The clock.** A playing scene moves in ticks of 1/60 s, whatever the frame rate.
      - Each frame's time is paid in whole ticks, at most eight a frame: beyond that the game slows rather than leaps.
      - Each tick takes the player's hands (`Input.takeStep`: move, look, the actions held, and every press and release
        since the tick before) and the screen buttons clicked.
      - Then, unless the world is paused, it steps: 2D players move, triggers hear them, and logic steps.
      - What logic removes during a tick is gone when the tick ends.
      - Paused, ticks go on without steps, so screens still answer.
    - **Logic** has `onStep` for gameplay, timers counted in steps, `random()` and `randomInt()` from the scene's
      `seed` (a new field; 0 draws a new seed each play), `overlaps()` by QQ's own boxes, and an `outcome`. `onFrame`
      stays, for what is only looked at. Presses, releases and clicks reach logic at the next tick, never between.
    - **The numbers** come from xoshiro128\*\*, seeded by splitmix32, in whole numbers: the same seed gives the same
      numbers on every machine, pinned against an independent implementation.
    - **QQ's own kinematics** (`kinematics.h`): boxes swept against boxes, in doubles.
      - Player2D now moves by them rather than by PhysX's character controller, against solid tile maps, static and
        kinematic shapes and sprites, and the ground.
      - A kinematic body follows it, so PhysX's bodies still feel it.
      - Triggers hear it by QQ's own measure.
      - It is drawn between steps, so it moves smoothly at any frame rate.
      - The build turns off fused multiply-add on GCC and Clang, so their arithmetic is MSVC's.
    - **Runs.** A `RunRecord` keeps a run's seed and what each tick took, keeping only what changed. `replayRun` plays
      it again with no window and no physics world, as fast as it goes, and says whether it reaches the outcome kept.
      - A replay never runs PhysX, so an outcome it reproduces cannot depend on PhysX: ADR-105 decision 6 holds by
        construction.
      - The Player gains `--seed`, `--record` (with `--ticks`) and `--replay`. A replay exits 0 when it matches and 4
        when it does not.
    - **Ember Climb** runs on steps, and its outcome (embers gathered, steps taken) replays. The Studio's starter logic
      and the agent's instructions use `onStep`.
    - **Not part of a replay:** the 3D Player, still PhysX's character controller (QQ's own 3D kinematics are later
      work); PhysX contacts; `pointing()` and `ray()`. An outcome that must replay keeps to these limits and to plain
      arithmetic, since `Math.sin` and its kin come from each system's library.
    - **Evidence, on the owner's PC:**
      - All eight suites pass in one run of `build.ps1`: 117 cases.
      - `tst_play` passes 36 cases, the new ones among them:
        - 120 steps for two seconds drawn at 30, 60, 144 and uneven frame rates, eight a long frame, none paused;
        - the same dice for the same seed;
        - a run played with the keyboard on a Tiled level, replayed with no window to the same outcome, the hero's
          place to the last bit, and not with another seed or idle hands;
        - Ember Climb's first part replayed;
        - a trigger hearing a 2D player with no physics world.
      - `tst_scene` pins the generator, checks the kinematics and reads a run back.
      - `tst_player` keeps a run with the real Player and replays it (exit 0); with another seed it does not match
        (exit 4).
      - Seven planted faults were each caught.
12. [x] **Animation.** glTF skeletal and morph playback, tweens on any field, and a state machine.

    **Done 19 October 2026, merged (#62, at `dd5a29d`; CI passed every job on its head and on the merge, runs
    39368661482 and 39370514734):**
    - **glTF clips.** A Prop plays its file's animations, skeletal and morph alike, by name: `clip`, `clipSpeed`
      (backwards too) and `clipLoops`. `clips` lists them, `clipTime` and `clipDone` say where it is, and `part(name)`
      finds a joint or a mesh.
      - Qt's runtime loader makes each animation a Timeline and plays the first on the wall clock. `Clips` (C++) stops
        that and poses the chosen clip at the game's time instead: once each step, exactly, so logic reading a joint
        reads the same in play and in a replay; and once each drawn frame, between steps, so it moves smoothly.
      - Changing clips puts back what the last one moved, which a switched-off Timeline does not do.
      - Play waits for models to load before its first tick, so no clip's steps depend on how fast a file loads.
    - **Tweens.** `tween(entity, field, to, seconds, easing, done)` eases a number, a vector or a colour field over
      whole steps, with nine easings. A new tween on the same field takes over, `cancel()` stops one, `done` runs on
      the step it arrives, and an entity removed takes its tweens with it.
    - **State machines.** `machine({ start, <state>: { enter, step, exit, on } })` steps with its logic and gives
      `state`, `previous`, `time` and `steps` in the state, `go()` and `send()`.
    - **Orb Run** gains two lanterns, an animated glTF model written by `make_lantern.py` (a skinned pole whose head
      sways, and a core that breathes by a morph target), which flare as each orb is taken. Its orbs grow in and
      shrink away by tweens, its score swells, and its boulder is a state machine.
    - **Evidence, on the owner's PC:**
      - All eight suites pass in one run of `build.ps1`: 121 cases.
      - `tst_play` passes 40 cases, 4 new:
        - a rig (`make_rig.py`) bent and puffed step by step, by its joint's angle, its morph weight and the pixels
          drawn; looped, paused, sped up, and put back at rest;
        - a joint read by logic in a replay with no window;
        - tweens of a vector, a colour and a number at their exact steps, a take-over and a cancel;
        - a machine's enters, steps, exits and events.
        Orb Run's case checks its lanterns and its tweens.
      - Six planted faults were each caught: what a clip moved not put back, a clip running on while paused, a tween's
        start following its field, a tween counted in the wrong steps, a machine never leaving a state, and steps that
        do not pose the model.
      - A seventh, Qt's own timeline left running, was not caught: each step and each frame poses the model over it.
        Looking into why found that a replay, which draws no frames, never posed the model at all; each step now does.
13. [x] **Sound.** One-shots from logic, music, and volume groups, panned in a browser where it can be.

    **Done 19 October 2026, merged 20 October (#63, at `6e37363`; CI passed every job on its head and on the merge, runs
    39376083521 and 39399914355):
    - **One-shots.** Logic's `play(source, { at, volume, group, reach })` plays a file once: placed in the world where
      logic says, or heard everywhere alike. It is put away when it has played through, measured from the file's
      header, since Qt's spatial sounds do not say when they have finished. A scene keeps its one-shots out of its
      tree, so nothing of them is written to its file.
    - **Music.** A new kind, `Music`: a track heard everywhere from when play starts, and on through a pause. Logic
      changes it by setting `source`, and the old track fades out on one voice as the new one fades in on the other,
      over `fade` seconds, after which the old one is let go.
    - **Volume groups.** `Mixer` keeps a level for each group, master among them. Every sound is in one (`effects`
      unless named otherwise, `music` for music), and what is heard is its own volume times its group's level and the
      master level, changed at once. Logic has `setVolume` and `volumeOf`. The Studio's Stop puts every level back, as
      it puts back everything else a play changed.
    - **In a browser.** `WebVoice` is now the page's own Web Audio: each file decoded once and kept for the next voice
      that plays it, then played through a gain and a stereo panner. Sounds and placed one-shots are quieter with
      distance from the eye and panned to the side of it they are on; music is heard alike on both sides. The page's
      first touch or key starts its audio, as browsers require.
    - **A replay makes no sound.** There `play()` returns nothing, so what it returns must not decide an outcome.
    - **Orb Run** has music, the title's giving way to the arena's when play starts; a chime where an orb is taken; a
      rumble where the boulder is rolled; and a switch on its pause screen that turns the music group off and on. Its
      sounds are written by `make_sounds.py`, standard library only, the same bytes each time.
    - **Evidence, on the owner's PC:**
      - All eight suites pass in one run of `build.ps1`: 126 cases.
      - `tst_play` passes 44 cases, 4 new:
        - a one-shot placed where logic said and one heard everywhere, at their groups' levels as they change, put
          away once played through;
        - music fading from one track to the next and letting the first go;
        - as a browser hears it, a sound on the eye's right panned right and one on its left panned left, quieter with
          distance, a sound ahead in the middle, and a one-shot the same;
        - no sound in a replay.
        Orb Run's case checks its music, its chime and rumble, and its music switch.
      - `tst_scene` reads a sound file's length, again when the file is replaced, and checks the Mixer's levels;
        `tst_studio` checks that Stop puts them back.
      - In headless Chrome (`build.ps1 -Web`), the playground's sound plays on the page's Web Audio, decoded and
        through a stereo panner, the audio running.
      - Eight planted faults were each caught: one-shots deaf to their group's level, a one-shot never put away,
        music never letting its old track go, music deaf to its group's level, logic sounding in a replay, a browser
        panning to the wrong side, the Studio's Stop keeping a play's levels, and a replaced file keeping its old length.
14. [x] **Materials.** Images on shapes, spot lights and HDR skies.

    **Done 19 October 2026, merged 20 October (#64, at `f6d230a`; CI passed every job on its head, on a rerun, and on the
    merge, runs 39422156020 and 39432677902):**
    - **Pictures on shapes.** A Shape gains `image`, `imageSize`, `normalMap` and `crisp`; the Ground gains `image` and
      `imageSize`.
      - A picture lies one copy every `imageSize` units on every face, whatever the shape's size, so a long wall has as
        many bricks as it is long, not a few stretched ones.
      - It is projected along the shape's own three axes by a small custom material (`runtime/shaders/surface.vert`
        and `surface.frag`), so it moves and turns with the shape. Qt's own lighting, shadows and sky light shade it,
        exactly as they shade a plain shape.
      - `colour` tints it (white shows it as painted); a `normalMap` gives the light fine relief to pick out; `crisp`
        keeps pixel art's pixels sharp.
    - **Spot lights.** A new kind, `Spotlight`: a cone of light (`cone`, `softness`) shining down unless turned, falling
      off with distance, with shadows. The Studio adds one, and shows where it is and which way it points while
      editing.
    - **HDR skies.** The Scene gains `sky`, an equirectangular `.hdr` image seen behind everything and lighting it, and
      `skyTurn`. Under a sky image, bloom glows only from what is brighter than white.
    - **Saving** brings the scene's own files (its sky) into the project, as it brings its entities'.
    - **The lockdown** (ADR-087): Qt Quick 3D reads pictures and models itself, past the engine's interceptor, so
      `SceneIO.resolvedUrl`, through which every file a scene names is found, gives nothing outside a confined game's
      package.
    - **Orb Run** gains walls of dressed stone with their relief, a slate floor, two spot lights and an HDR dusk sky
      whose sun stands where its Sun shines from, written by `make_looks.py`. The playground has stone, a spot light
      and the sky too, so the browser's check draws them.
    - **Found and fixed on the way:** the Ground was drawn a hundred times as wide as its solid slab (Qt's built-in
      rectangle is a hundred units across), so whatever walked off its edge fell through ground that was still drawn.
    - **Evidence, on the owner's PC:**
      - All eight suites pass in one run of `build.ps1`: 133 cases.
      - `tst_world` passes 9 cases, 6 new, each judged by the pixels drawn: a picture counted copy by copy, sharp or
        blended; a pictured shape lit as a plain one; a relief turning a face to the light; a spot light's pool, moving
        as it turns; an HDR sky seen, lighting what is under it, turned, and not washed out by bloom; and the ground as
        wide as its slab.
      - `tst_lockdown` passes 8 cases: a picture, a relief, a sky, a model and a sound named from outside the package
        are nothing.
      - `tst_studio` saves a picture and a sky chosen from outside the project into it; Orb Run's case checks its looks.
      - `build.ps1 -Web` draws the playground's stone, relief, spot light and dusk sky in headless Chrome (WebGL), with no
        problem on the page. Its first frame comes at 7.2 s, where P3.13's took 5.7 s: the sky's lighting is prepared
        and the new shaders made in the browser first.
      - Eleven planted faults were each caught: pictures laid by the mesh rather than the shape's size, crisp ignored, a
        picture's colours not made linear, the relief never reaching the light, the relief turned by Qt's normal matrix,
        a spot light's cone ignored, the sky never turned, the ground drawn a hundred times its size, bloom washing a
        sky image out, a confined game reading pictures from outside its package (by `tst_lockdown`), and saving
        bringing in only the entities' files (by `tst_studio`).
15. [ ] **The Studio's basics.** Undo and redo, duplicate, snapping, a hierarchy with parenting and prefabs, an asset
    browser, a logic editor, and a statistics overlay with physics shown.

    Built in three parts, each its own pull request: editing (undo and redo, duplicate, snapping); the hierarchy with
    parenting and prefabs; and the asset browser, the logic editor and the statistics overlay.

    **Part one, editing, built 20 October 2026, merged 21 October** (#65, at `f75d082`, with the fix for three dependency
    advisories, ADR-107; CI passed every job on its head and on the merge, runs 39508315025 and 39521483702):
    - **Undo and redo** (Ctrl+Z; Ctrl+Y or Ctrl+Shift+Z; toolbar buttons). Each step is the scene's canonical text after
      an edit (`studio/History.qml`, up to 200), noted wherever an edit ends (the window's `refresh()`), so every edit
      is covered whatever made it: a field in the inspector, a handle let go, an entity added, duplicated or deleted, a
      tool the agent used. Quick edits of one field (a slider dragged, a name typed) are one step. Saving is not a step:
      what it changes (the name, files brought into the project) joins the step it saved. Undoing keeps the file, what
      was last saved, the view and the selection, by name.
    - **Duplicate** (Ctrl+D, or the inspector's button): a copy beside the selection, a unit along X, every field the
      same but its name ("Crate 2"), each value copied so the two stay apart. A scene's one player is not duplicated.
    - **Snapping** (the toolbar's choice: off, one art pixel, 0.1, 0.25, 0.5 or 1 unit): a handle moves along that grid
      on the axis dragged, turns by 15 degrees, and scales by tenths.
    - **Evidence, on the owner's PC:**
      - `tst_studio` passes 14 cases, 3 new: every edit undone and redone with the keys and the buttons, back to the
        saved scene and no further, quick edits as one step, the selection kept, and an edit after an undo leaving
        nothing to redo; a duplicate, apart from its original and undone; and snapping on all three handles.
      - All eight suites pass in one run of `build.ps1`: 136 cases.
      - Eight planted faults were each caught: quick edits of one field never merged, an edit after an undo keeping
        what could be redone, undo forgetting what was last saved, undo losing the selection, saving made a step to
        undo, a copy of a copy named after the copy, turning not snapped, and redo made ambiguous by naming a key the
        platform's redo already has.

    **Part two, the hierarchy and prefabs, built 20 October 2026** (branch `session/qq-hierarchy`, on `main` at `f75d082`
    since #65's merge, whose tree it was built and checked on; pull request #66, not yet merged):
    - **Groups.** A new kind, `Group`: entities under one place, turn and scale, moved as one, and written inside it in
      the scene's file. What is placed can be in a group (shapes, lamps, spot lights, models, emitters, sounds,
      triggers, sprites, groups, prefabs, behaviours); players, the ground, the sun, tile maps and what has no place stay
      at the scene's top. A group passes the scene's state on to what is in it (playing, paused, a replay, a 2D view).
      - `SceneIO.entities` walks the tree in file order; logic's `entity`, `spawn` (a third argument puts what it makes
        in a group) and `remove` (a group goes whole) see it.
      - Bodies are made at the scene's top from an entity's place in the world; a kinematic one, and a trigger's
        volume, follow it as its group moves.
      - QQ's kinematics measure an entity through its groups in doubles, so a replay stays the same everywhere; a
        group's turn about the depth axis counts.
    - **Prefabs.** A group kept in a file of its own, `prefabs/<name>.qml` (its root a Group), placed by the new kind
      `Prefab` (`source`) as often as wanted. The scene keeps where each stands and which file; its parts come from
      the file, play (bodies, triggers, logic: a Behaviour inside finds its target among its own prefab's parts first),
      and are found by `part(name)`, but are not the scene's own entities (`SceneIO.everyEntity` lists them for play).
      Logic places one with `spawn("Prefab", { source, position })`. Saving brings a prefab file into the project's
      `prefabs/`.
    - **The Studio**: the list is the tree, indented; Ctrl+G groups the selection where it stands, Ctrl+Shift+G
      ungroups, and the inspector moves the selection between groups keeping its place. **Keep as prefab** writes a
      group to `prefabs/` and places a prefab where it stood; **Add → Prefab…** places another; **Unpack** turns a
      prefab into a group of its parts to change them, and keeping it back writes over its file and makes the scene's
      other prefabs of it again. A click on a prefab's part picks the prefab.
    - **The agent**: `entity_add` takes a `group`, and `scene_read` says which group each entity is in.
    - **Orb Run**: its orbs are a prefab (`prefabs/orb.qml`) placed by logic, and its walls a group.
    - **The lockdown**: a prefab file is found through `SceneIO.resolvedUrl` like any other, so a confined game makes
      nothing of one outside its package.
    - **Found and fixed on the way:** the QQ README's table of the runtime had been split over two lines since P3.13,
      so the table showed broken; a switch in the inspector read a field the newly selected kind lacked.
    - **Evidence, on the owner's PC:**
      - All eight suites pass in one run of `build.ps1`: 142 cases.
      - `tst_scene` 26 cases, 3 new; `tst_play` 46, 2 new and Orb Run's changed; `tst_studio` 15, 1 new; the lockdown's
        package check takes a prefab.
      - `build.ps1 -Web` plays the playground in headless Chrome with no problem on the page (first frame 7.1 s, 318 frames a second, 12.06 MB brotli): kinematic and static bodies made the new way, from the world place.
      - Ten planted faults were each caught: the entity list not walking into groups, a prefab's parts written into the scene, a group not passing on that the scene plays, a body made inside its group, kinematics measuring only the entity's own place, a prefab's logic finding its target in the scene first, moving between groups not keeping the world place, a click on a prefab's part picking the part, logic unable to remove what is in a group or a prefab, and an unpacked prefab kept as a new file. Two did not fail at first: the crate's rest on the deck was checked as it fell past (it is now checked again after a wait), and logic's refusal to remove a part did no harm in Orb Run's walk (logic now removes a part directly in the prefab case).
16. [ ] **Saving.** One small store per game, the one the lockdown allows.

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
   balance in integer embers, and its DRC-369 inventory from finalised state, shown stale rather than zero on loss.

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
    a glowing ring from level 1 and a wider glow from level 6, in builds from 0.1.9 (17 October 2026). Themes as
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

19. [ ] ARQADE in the launcher on Qt (ADR-102): `products/arqade/qt/`, docked under the product host, signed in with the
    launcher's QOR ID, Qt Quick and Qt Quick 3D throughout, holding no key. Its home, the games, rankings and the
    player's Sparks. **Built, 18 October 2026, in pull request #48 (merged at `6cb96fb`) and in the tester release launcher 0.1.14:**
    - The app has its home: a 3D astrolabe portal, the four games and two arenas as cards with 3D emblems, the player's
      Sparks, and the arenas' rankings from ARQADE's server.
    - The launcher's ARQADE screen starts the app and docks it, telling it where to sit, the launcher's colours, who is
      playing and the person's motion preference. It falls back to the website's window when the app is absent or
      ends.
    - The app signs in to ARQADE's server through the launcher (`arqade.signin`), under the same checks as the web
      window, and opens the website for today's games (`arqade.web`).
    - Tests: 32 Qt tests, 13 host tests (three seen to fail against planted faults), 17 browser checks, and an opt-in
      test that starts the real app.

    **Not yet:** a person signing in through it in the running launcher, against QOR ID's deployed handoff route.
20. [ ] The four solo games rebuilt in QQ (ADR-102): Rift Survivor, Void Runner, Synapse and Orbital, in 3D with
    physically based materials, bloom, particles and spatial sound, keyboard, mouse and gamepad, reduced motion
    honoured, each deterministic for a seed and its inputs.
21. [ ] The platform reward pool on the devnet (ADR-102): ARQADE's own Cartridge and ARQ Wallet, funded with test CGT by
    the owner, whose policy (largest payout, epoch budget, per-player cap, the games' rule versions) the owner sets.
22. [ ] Runs confirmed by replay (ADR-102): a seed from the ARQADE service, the run's inputs replayed by the service with
    the same rules, and Sparks paid from the pool for the outcome the replay reaches, once. Depends on P7.20 and P7.21.
23. [ ] The website without Energy (ADR-102): the Energy store, Energy Tokens and local token wallet removed; its own
    games point to the launcher.

    **Energy removed, 19 October 2026** (merged in #54 at `d93c3c5`, live on Vercel):
    - No Energy store, entry cost, token draw or token wallet. Play is free, and the browser keeps only sessions and
      best scores.
    - The terminal's `energy`, `rewards` and `wallet` are one `stats`. The intro and notice say nothing is bought,
      staked, won or lost.
    - `check-phone.mjs` counts six tabs (GATES change log).

    **Not yet:** the games pointing to the launcher, which waits on P7.20's rebuilt games.
24. [ ] The two multiplayer arenas rebuilt in QQ (ADR-102), on the ARQADE service. Depends on P7.19.
25. [ ] ARQADE's site on phones and by touch (the owner, 18 October 2026: "optimize Arqade for its mobile browser
    users and screen tapping"). **Built, merged (#49, at `943815d`) and live on Vercel:**
    - Below 760 pixels the side rail is a tab bar along the bottom, and the content takes the whole width.
    - Games and dialogs fill the screen, every target is 44 by 44 pixels or more, and the chat's button is round,
      above the tab bar.
    - Void Runner and Orbital are played by tapping their playfields, and the page reaches the screen's edges inside
      its safe area.
    - Held to it by `scripts/check-phone.mjs`, 17 checks, in CI (GATES `arqade.phone`).

    **Not yet:** tried on the owner's own phone.

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
