# Every system this repository names

**What this is:** one table of everything Demiurge calls a system or a surface, what it actually is, and
whether it exists. **Last brought up to date on 2 October 2026**, from that day's test runs (the chain, QOR ID and the
launcher's host, each re-run in that session). Rewritten on 26 September 2026, after Qontrol's git layer (P1.2), against the tree
rather than against the other documents, and brought up to date on 28 September, when the vault lost its
lock screen, first to Windows Hello (ADR-055) and then to the keychain (ADR-056), and the launcher's tests
and screen checks were run again, and the operations stack started. Updated on 29 September, when the chain
gained royalties and a sale settled in CGT (ADR-061) and was tested again. Later that day the code went public as
`ALaustrup/demiurge-chain`, CI moved back to GitHub Actions and QOR ID began moving to Railway (ADR-063). QOR ID's count says when it was last
run, because it did not change. Updated on 30 September: the launcher's gates dashboard reads CI from the public
repository by name, its default QOR ID address is `id.qorsync.dev`, and the PC stack was found down; the launcher's
tests and the chain's were run again. Checked on 1 October: on Railway, Postgres and Redis are online and hold the
owner's restored account, and QOR ID is deployed and running there (its log shows the migrations accepted and both
stores connected), answering at `https://id.qorsync.dev` (`/ready` 200, measured), with email not configured. **Updated on 4 October 2026** with one row, ARQADE (P7, ADR-069; source imported into `products/arqade/` that day), checked against the
arcade's source, the runtime and QOR ID that day; no other row was re-measured.

**How to read the status column.** These words mean exactly one thing each:

| Status | Meaning |
| --- | --- |
| **built** | It exists, it runs, and tests cover it |
| **partial** | Some of it is real; the rest is named in the roadmap and not written |
| **placeholder** | A page exists that states what it will be and why it cannot work yet. No mock content |
| **planned** | Not written. It has a roadmap item, so it is scheduled rather than wished for |
| **named only** | You named it. The repository does not define it. **No scope has been invented for it** |
| **frozen** | Pre-realignment code. Not maintained, not a reference, not deleted (D-011) |
| **dead** | Built by no workspace. Kept only because deleting it is its own decision |

Roadmap items (M-numbers for the protocol, L-numbers for the launcher, P-numbers for the six products) are
in [`DIRECTION.md`](DIRECTION.md). Decisions are the ADRs indexed in [`DECISIONS.md`](DECISIONS.md).

---

## The things that actually run today

| System | What it is | Status | Where |
| --- | --- | --- | --- |
| **The chain** | A Substrate L1 on the Polkadot SDK. Aura produces blocks, GRANDPA finalises them, and the validator set comes from governance rather than from a config file. Since 22 September 2026 it also holds assets (DRC-369, below), and moves several of them in one all-or-nothing transaction (`pallet-utility`, ADR-053). **Since 29 September it settles sales in CGT and pays royalties** (`pallet-drc369-royalties`, ADR-061), since 1 October it nests assets (ADR-065), and **since 2 October a buyer can hold a purchase to the content they saw** (`buy_exact`) and any client can ask what a sale would pay (the runtime API `Drc369RoyaltiesApi`); `spec_version` 6. **150 tests pass with the runtime built, fmt and clippy clean (4 October, with `pallet-arq-wallet` and its rounds; 124 on 3 October)**; **since 3 October it runs as `Demiurge Devnet` on Railway** (ADR-068), **at `spec_version` 7 since 4 October** (the ARQ Wallet, upgraded in place by the owner's sudo key): two validators and a public node at `wss://rpc.qorsync.dev`, genesis holding only the owner's sudo and faucet accounts, measured finalising 100 blocks in ten minutes with both validators authoring (`alpha.devnet-live` and `alpha.devnet-finality` met); two validators agree, finalise, and one recovers after being killed (13 of 13 on the final runtime; one earlier run that day missed its 120-second deadline for block 4 by one block and passed everything after); a sale and a remix sale were run on a development node and paid the right accounts | **partial** — the base layer, assets and royalties are real; fees, issuance and a treasury are not | `chain/` · ADR-013 · M3, M4.1, M4.2 (half), M4.6 |
| **DRC-369** | The asset standard. **Its first part exists:** mint a project's commit as an asset identified by a fingerprint of its manifest (every file, each fingerprinted, and the commit), revise it until a one-way switch makes it permanent, one collection per creator, and a list of what each account holds. Only this pallet can create an asset — inside a batch as well as outside one. **Since 29 September:** a mint can name the asset it remixes (bounded at 16 deep), a creator sets royalty terms and can correct them while they hold the work (up to eight recipients and a remix share that never rises once someone has remixed it, ADR-062), and an asset can be listed and bought for CGT, paying the remix's source, then the royalties, then the seller, in one transaction. **Since 1 October:** an owner places one of their assets inside another and takes it out again; cycles are refused (R-2); a tree is at most 8 deep and each asset holds at most 64; a nested asset and the asset holding it stay put (no transfer, sale or burn) until taken out (ADR-065). 25 pallet tests, 26 royalty tests and 16 runtime asset tests (2 October) | **partial** — M4.1 done, M4.2's royalty half (ADR-061) and nesting half (ADR-065) built; state and XP, physics, rental and fractions are not. Moving a parent together with its contents does not exist, and the launcher has no nesting surface. Without an indexer (M5.4) a list of everything for sale can only be read by walking the chain's listing storage, which the launcher's Market screen does (2 October). **Its deposits and weights are placeholders** (U-14, M7.2) | `chain/pallets/drc369/`, `chain/pallets/drc369-royalties/`, `chain/runtime/src/assets.rs` · ADR-025, ADR-047, ADR-052, ADR-061 · M4 |
| **QOR ID** | The identity service. Username or keypair sign-in, JWTs, sessions, email verification, password reset, backup codes, admin routes, agent registration. **136 tests pass** against real Postgres 16 and Redis 7.4 with `--include-ignored` (2 October 2026); **145 on 5 October** (ADR-075: a QOR ID is the username alone, unique, no `#0001`; migration 019 makes the database enforce it), with sign-in for other apps: `/oauth/authorize`, `/oauth/token`, `/oauth/userinfo`, `/oauth/revoke` (ADR-073). Since 2 October a session records when it was last used (at sign-in and at each refresh, shown in the account's list of sessions), and `GET /health` says, as one boolean, whether its email leaves the machine it runs on: the end-to-end scripts refuse to run unless it says no. Its migrations are LF on every platform and a test fails on a carriage return in one, so a Windows and a Linux build can share a database; no test can send real mail, whatever the machine's environment holds | **partial** — sign-in is real; session keys and scoped delegation are not | `services/qor-auth/` · ADR-014, ADR-016, ADR-017 |
| **Operations stack** | QOR ID, its Postgres and Redis, **on Railway** (ADR-063): project `demiurge`, region `iad` | **partial** — since 1 October QOR ID runs there and answers at `https://id.qorsync.dev` (`/ready` 200, measured), holding the owner's restored account. **Email works** (a verification message from it was delivered to Resend's test address on 1 October; a real inbox is untried), and bounce reports are refused until the webhook secret is set. Its Postgres has no public address. The PC stack (ADR-060, superseded) is retired: its containers are removed and `infra/ops/` is deleted; `ci.qorsync.dev`'s record and the Cloudflare tunnel remain for the owner to delete. No public RPC until a private sudo key | `services/qor-auth/DEPLOY-RAILWAY.md` · ADR-063 |
| **CI (GitHub Actions)** | Builds and tests every push and pull request to `main` of the public `QOR-MATRIX/demiurge-chain`: chain, QOR ID, launcher, coverage, security, and two validators nightly (ADR-063, ADR-064; Woodpecker, ADR-058, is retired) | **partial** — **jobs execute since 1 October**; on 4 October (measured) QOR ID and Security pass and **the chain job failed at Lints** on every run since Rust 1.99 (28 September) linted the SDK's macro output. The owner chose a pin the same day (ADR-072): the chain builds on Rust 1.98.1 and a report job shows the newest Rust's lints. An `arqade` job joined on 4 October. **On 4 October run `37260938168` passed every job** (measured), the chain's included. when the repository moved to the owner's organisation (on the personal account a billing lock refused every one). The owner saw the first run in progress; **no run has been seen to finish**, so whether it passes is not known | `.github/workflows/ci.yml` · ADR-063, ADR-064 |
| **QOR Launcher** | The desktop application. A vault holding Sr25519 keys whose key is in the OS keychain, opened with nothing asked, QOR ID sign-in by the vault's key with no password and never blocking the launcher, sending the currency and minting assets against a real node, an Inventory read from the chain where each asset is a card that can be sent to another account, a chain view, a release-gate dashboard (reading CI from `QOR-MATRIX/demiurge-chain` by name), Qontrol's Projects surface with line-by-line diffs, branches and a guard before committing, settings, themes and the QFX backdrop. Its default QOR ID address is `https://id.qorsync.dev/api/v1` since 30 September (it was `demiurge.cloud`, which never served QOR ID); a saved setting still wins. Sell lists an asset on chain and Buy settles a purchase, held to the price and the content that were on screen (`buy_exact`, 2 October). **The Market screen** (2 October) shows every listing on the chain, and what a sale pays is the chain's own answer (`Drc369RoyaltiesApi`); the launcher carries no copy of the arithmetic. Its default chain is the devnet, `wss://rpc.qorsync.dev`, since 3 October. **195 host tests pass** (3 October), seven more against a running node at `spec_version` 6, nine screen checks (Market 127 among them), and eight checks in a real rendering engine (2026-09-28), one of which measures every piece of text on every screen as it is painted | **partial** — L0 done, L4.2, L4.4 and L4.5 done, half of L4.6; L1.4/L1.6/L1.7/L2.2 open | `tools/qor-launcher/` · L0–L7 |
| **Creator-God Token** | The currency everything settles in. 18 decimals, 1 CGT = 10^18 Sparks, existential deposit 100 CGT | **partial** — the unit is real. **No supply, no issuance, no fee, no treasury** — every one of those needs a number nobody has decided | `chain/runtime/src/denomination.rs` · ADR-002–008 |

**The honest summary of the chain:** it produces and finalises blocks, moves the currency between
accounts, and since 22 September 2026 mints, revises and fixes assets and lists who holds them. It cannot
charge a fee or create currency, because neither is decided, and an asset cannot yet carry
or have state. Since 29 September 2026 it pays royalties on a sale it settles in CGT, and since 1 October an
asset can be nested inside another.

---

## The launcher's surfaces

The strip down the left of the launcher. `DIRECTION.md` §1 names the same list.

| Surface | What it does | Status |
| --- | --- | --- |
| **Gate** | What shows before the vault is open: a first run, restoring from a recovery phrase, or an older vault's one last Hello or passphrase. A returning person never sees it (ADR-056) | **built** |
| **Nexus** | The map you land on. Every system in one place | **built** |
| **Vault** | Hold, send and receive the currency. The only thing that signs. **Since 28 September 2026 there is no lock screen: the vault's key is in the operating system's keychain and nothing is asked to open it** (ADR-056, superseding ADR-055's Windows Hello the same day). The recovery phrase is the way back; restoring sets the old vault aside, never deleted; older vaults move once | **built** — tested against an in-memory keychain and once against this PC's real Credential Manager, and its screens in a browser (`check-vault-gate.mjs`, 65 checks, 2026-09-28, not yet proven to fail against planted faults). Nobody has yet opened it in an installed launcher |
| **Inventory** | The DRC-369 assets an account holds — name, fingerprint, pinned commit, permanent or not — read from the chain every time; **Make permanent**; **an asset's own menu, sending assets to another account, and drafting a listing for one**; and settled history | **partial** — the assets half is built (L4.2, 127 checks against the built view); since 22 September each asset is a card that leans towards the pointer and opens at size (P2.8), and its menu sends several assets in one all-or-nothing transaction (L4.4, L4.5, ADR-053). Since 23 September the trade window is two sides with a lane between them and offers accounts traded with before, held in this machine's own file and sent nowhere; and **Sell opens a listing form that saves on this machine and publishes nothing** (half of L4.6, which stays unticked — the chain can settle a sale since 29 September, but the launcher does not call it yet, and a listing others can see needs the indexer). History still needs the indexer (ADR-028), and the page says so |
| **Chain** | Node health, block production, which chain you are pointed at | **built** |
| **Onboarding** | Not a rail surface. A bubble offers a QOR ID and checks a name as it is typed; once named, a notification with a neochrome halo opens a six-chapter tutorial; a two-second intro plays on each open and a first-run animation after an install (the owner's own file, or a longer intro while there is none) | **built, 28 September 2026** — measured by `check-readability.mjs` in every theme; tested and accepted by the owner in the installed launcher; the first-run file not yet supplied |
| **Gates** | The release gates, measured from `GATES.toml`: Alpha, Beta and Public Release, and one per product | **partial** — needs a person to exercise it once |
| **Projects** | Qontrol's surface. Open a folder or start one from a code, music or game scaffold; see which files changed and, since 23 September 2026, **what changed inside each one, line by line**, read as git reads it so a converted line ending is never shown as a change; the history; since 26 September 2026, **make a branch, switch to one, put a changed file back, and commit only the files ticked**, with **a file over 50 MiB or shaped like a credential held back until accepted by name**; and since 22 September 2026, **mint the commit** as a DRC-369 asset | **partial** — P1.1 and P1.2 are done (104 checks against the built view). The helper ships inside the Windows installer, which was built; nobody has yet installed it and committed from it, or opened a diff in a running launcher. No merge, push or large-file store (P1.5) |
| **Settings** | Themes, accessibility, the Ambience of the QFX backdrop (Off, Still, Live), identity (sign in to QOR ID with the vault key), and the recovery phrase shown after a host dialog | **built** |
| **Library** | Install, patch and launch titles, with entitlements bound to your QOR ID | **placeholder** — L7.1, stepped out in P5 |
| **Social** (named Agora until 2026-09-28; VYB merged into it the same day) | Rooms, messages, presence, voice, profiles and feeds, under the same identity you play under | **placeholder** — L7.3 |
| **Mesh** | Community-seeded distribution. Players host what they own and get paid for it | **placeholder** — L7.4 |
| **Studio** | Upload any file and mint it | **partial** — a tile pointing outside the launcher. Moves inside at L7.5. Minting from a project is in Projects |
| **Market** | Where creators list work, buyers spend the currency on it, and creators commit currency for visibility | **partial** — L7.2 / M8.3, stepped out in P5. **Since 2 October the launcher has a Market screen**: every listing on the chain, read by walking the chain's listing storage at the finalised block (there is no indexer, so no search, no history and no "newest"), filtered by whose it is, ordered by price or number, paged, with listings that cannot be bought shown with the reason. Buy goes through the same purchase dialog as Inventory, and what a sale pays is the chain's own answer (`sale_preview`). Sell, Withdraw and buying by number are in Inventory since 1 October. It is one node's view, bounded, with no catalogue model for the six kinds of payload (P5.3 waits on the indexer), royalty terms cannot be set from the launcher, and no person has yet run it in the real application. The description form saves on this machine and publishes nothing |

**Where a minted asset's bytes are:** in `temporary-content-store/` inside the launcher's data directory, on
the creator's machine only, labelled temporary until the Mesh (ADR-047, question 13, as the owner answered
it). The chain holds a 41-byte fingerprint of them, never the bytes.

---

## The six products

Built on one substrate: every product signs in with QOR ID, stores creations as DRC-369 assets, versions
them with Qontrol, distributes through the Mesh and settles in the currency. The decisions they share are
ADR-046 to ADR-051. **ADR-047, the asset format, and ADR-051, QFX's rendering layer, are accepted**; the rest
are Proposed. Each has a blueprint, a track in `DIRECTION.md` and a release gate of its own. **None is a
product yet**; two have a first slice inside the launcher.

| Product | What it is | Status | Where |
| --- | --- | --- | --- |
| **Qontrol** | Version control for people who make things. On disk it is an ordinary git repository; the creator sees plain words, never git's vocabulary | **partial** — the Projects surface is built, diffs included (P1.1, 2026-09-23), with branches, discard, per-file commit, a guard before staging and its helper bundled (P1.2, 2026-09-26), and publishes a version as an asset (half of P1.6); P1.3 to P1.5 and P1.7 are not | `tools/qor-launcher/src-tauri/src/qontrol/`, `tools/qor-launcher/qontrol-git/` · [blueprint](blueprints/qontrol.md) · P1 |
| **QFX** | The launcher's living visual layer, in three layers: a backdrop, an interface that responds, a personal space. Creator-made, and the chrome keeps text readable whatever a theme paints | **partial** — layer one's first slice, the backdrop, is built, and layer two's first, the DRC-369 card; the sheen, previews and 3D are not, and layer three is not | `tools/qor-launcher/src/qfx/` · [blueprint](blueprints/qfx.md) · P2 · ADR-051 |
| **QOR Engine** | A 2D and 3D creation tool: a custom Godot build tracking upstream, with Demiurge's layer added as modules and editor plugins. "QOR Engine, built on Godot" | **planned** — P3. Nothing written | [blueprint](blueprints/qor-engine.md) · P3 |
| **GNOSIS** | Music production where a song is a folder of small text files two people can edit and merge | **planned** — P4. Nothing written | [blueprint](blueprints/gnosis.md) · P4 |
| **Market and Library** | Listing and buying work in the currency; downloading, verifying, installing, patching and launching it. One listing model for games, tools, plugins, scenes, presets and themes | **planned** — P5, carrying L7.1 and L7.2. Library is a placeholder page; Market's first slice exists (the row above) | [blueprint](blueprints/market.md) · P5 |
| **Stream** *(placeholder name)* | A music listening service first, with licensing, over the Mesh; a work's royalty split applied when it is paid for | **planned** — P6. Its first item is a decision, deferred to M6; nothing else before M8 | [blueprint](blueprints/stream.md) · P6 |
| **ARQADE** | The gaming platform the owner named on 4 October 2026: games played with a QOR ID, every owned collectible a DRC-369 asset, CGT the only currency, and a developer SDK. It started as an arcade on OpenAI Sites (Cloudflare Workers and D1): four solo games and two server-checked multiplayer arenas. **Since 5 October 2026 it is a standard Next.js site for Vercel with Postgres (ADR-074)**, its live arcade played by QOR ID accounts | **partial** — P7. Since 4 October 2026 its source is in `products/arqade/` (active scope, ADR-069): the arenas and solo games, a read-only devnet reader checked against the genesis and an account lookup on its QOR screen (test CGT and DRC-369 assets at the finalized block, P7.2), and the SDK's first pieces (`products/arqade/sdk/`: CGT amounts as integer Sparks, ARQ Wallet policy checks, a wallet's address and the outcome and round ids, the chain calls' arguments, and the project profile that becomes a store page with its stage checks and the owner's 10,000 CGT price ceiling). ARQ Wallets, one keyless payout account per game (ADR-070), are **built on the chain** as `pallet-arq-wallet` (index 11; **live on Demiurge Devnet since 4 October**, with rounds and held prizes since `spec_version` 8 the same day, each upgrade read back; 25 tests). Self-publishing, backing and agentic creation (ADR-071) were accepted on 4 October 2026 and are not built beyond the SDK's checks. Renamed ARQADE on screen and its control-by-control inventory written (P7.1 done, 5 October). The live site still serves the old copy. ADR-069 and ADR-043 were accepted on 4 October 2026; **QOR ID sign-in is built** (ADR-073, 5 October: QOR ID's own page, then a server-side session in ARQADE; QOR ID's side live and ARQADE registered). **Not live on Vercel yet**: the owner creates the Vercel project and its database. 41 tests. The old Sites copy is no longer ARQADE, signing, randomness, editions and account-bound assets are all missing | [blueprint](blueprints/arqade.md) · ADR-069 · P7 |

---

## Planned, with a roadmap item

Not written. Scheduled.

| System | What it is | When | Blocked by |
| --- | --- | --- | --- |
| **The rest of DRC-369** | Nesting with cycles refused, state and XP, physics, rental, fractional ownership | **M4.2 to M4.5** | Nothing technical. M4.4, sponsored deposits, waits on U-4 and OPEN-4 |
| **The SDK** | Builds and signs transactions from the runtime's own metadata. Published only after the wire format is frozen | **M5.1** | The freeze, which cannot happen while Q-18 and Q-19 are open |
| **Agent rails** | Delegated keys with scoped capabilities and spend caps enforced by the chain, not by a database | **M5.2** | M4 |
| **The MCP server** | One server any LLM can drive, instead of a plugin per vendor. Reference capability: generate-and-mint | **M5.3** | M5.2 |
| **The public viewer** | A shareable page per published work — the growth surface. No sign-in | **M5.4** | The indexer |
| **The remote console** | A thin web console. Read, manage, transact. Deliberately never a second product | **M5.5** | — |
| **The indexer** | Reads chain events into a database so history can be shown. A Substrate node serves no history | **M5.4** | An archive node |
| **The archive node** | A node that keeps everything, feeding the indexer. Its disk grows forever | **M5.4** | — |
| **Public RPC node** | A separate node the world can talk to. **Validators get no public address** | **M5.1 / M7** | Deployment |
| **QOR Installer** | Installs the launcher, sharing its Rust core | **L6.3** | Code-signing identities |
| **Update channel** | Verifies signatures before installing. An installed launcher with no update path cannot be fixed | **L6.2** | — |

---

## Named by you, undefined in the repository

**No scope has been invented for any of these.** Each needs a roadmap item and a decision before it needs
a line of code or a subdomain.

| Name | What the repository knows | What has to happen |
| --- | --- | --- |
| **Relays** | Nothing beyond the name | Define it or drop it |
| **Agentic synchronisation** | Nothing beyond the name. **Not** the agent rails of M5.2, which are a different, defined thing | Define it or drop it |
| **QOR Wallet** | **The launcher's Vault already is the wallet.** There is also a frozen `qor-wallet` library in `apps/hub` that derives chain keys from a QOR ID — which is exactly the design ADR-017 removed | Say whether this is a new name for Vault, or a second place keys live. If the second, it contradicts ADR-001 and ADR-011 |
| **VYB Social** | **Merged into Social by the owner, 28 September 2026**, and the VYB name dropped. The frozen code in `apps/hub` that defined it is not a reference. `vybz.cloud` is recorded as a different project | **merged** |
| **Demiurge Exchange** | No code, no roadmap item and no decision. The name appears only in documents that say it is undefined | **Decide this one first.** See below |

### Why Demiurge Exchange needs deciding before anything is built

The name is ambiguous in a way that matters, because the two readings are opposites under ADR-002:

- **A marketplace where people spend the currency on assets** is already in scope, already passes the
  spend test, and is already called **Market** (L7.2). If that is what you meant, it is a duplicate name.
- **A venue for trading the currency against outside money** is the clearest possible failure of the
  spend test. ADR-002 says value comes from demand to *spend*, not demand to *hold*; a trading venue
  makes holding the point. It would need a decision record that survives ADR-002, and none exists.

---

## Frozen: real code that is not maintained

Pre-realignment. **None of it works against the current chain.** Kept, not deleted, because deleting is
its own decision. 806 files tracked in git, about 1.6 MB, measured on 22 September 2026. The figure this line
used to give, 850 files and about 11 MB, could not be reproduced and is replaced.

`apps/hub` · `apps/wallet-extension` · `apps/sophia` · `apps/nft` · `apps/portal` ·
`apps/marketing-site` · `apps/games` · `apps/guru` · `cli/` · `sdk/` · `packages/` ·
`client/DemiurgeClient`

The launcher's Nexus advertises nine of these as reachable systems — Explorer, Staking, Sophia, Scatter,
Resonance, Worlds, Agent Foundry, Developers, Bounties. **None has a roadmap item and none of those tiles
works.** What to do about that is a decision, and it is in
[`REALIGNMENT-2026-09-21.md`](REALIGNMENT-2026-09-21.md).

## Dead

Built by no workspace: `aeons/` · `archons/` · `syzygies/` · `packages/blockchain-wasm`. About 31 KB tracked.

## Gone

`framework/`, the custom chain that came before this one, was deleted on 20 September 2026. It had no
finality and it accepted forged signatures. Nothing in it is carried forward.

---

## What stands before the rest of the asset work, specifically

M2.1 was ticked on 22 September 2026, against the owner's review of a ten-line summary of the migration
inventory's DRC-369 section, and M4.1 was built and ticked the same day. So:

1. **M4.2's royalty half is built (ADR-061); its nesting, state and XP half, M4.3 and M4.5 can start now.** None
   needs an economic value.
2. **M4.4, sponsored fees and deposits, waits on U-4 and OPEN-4**: sponsoring a fee needs a fee.
3. **The deposits a mint holds are placeholders** (U-14). They are derived from the existential deposit so
   a development chain can run, and a public network cannot open on them.
4. **The format cannot be frozen** — and no SDK published — while two questions are open (**Q-18**, eight royalty
   recipients, was answered by the owner on 28 September: eight, ADR-057): **Q-19**, whether an asset's fingerprint depends on its files alone or also on the commit it came
   from; and **Q-20**, whether the table that labels a file's media type is part of that fingerprint, since
   the label is inside the hashed manifest and the format names no table. `beta.royalty-recipients`,
   `beta.manifest-identity` and `beta.media-types` count them.
