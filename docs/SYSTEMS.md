# Every system this repository names

**What this is:** one table of everything Demiurge calls a system or a surface, what it actually is, and
whether it exists. **Last brought up to date on 6 October 2026**, every row checked against public `main` (`9f8a818`,
pull request #13 merged) and the live services: QOR ID's, the launcher's and ARQADE's counts are from that day's runs; the
chain's are from 4 October, when `chain/` last changed. Earlier history: brought up to date on 2 October 2026 from that
day's test runs (the chain, QOR ID and the launcher's host). Rewritten on 26 September 2026, after Qontrol's git layer (P1.2), against the tree
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

Roadmap items (M-numbers for the protocol, L-numbers for the launcher, P-numbers for the seven products) are
in [`DIRECTION.md`](DIRECTION.md). Decisions are the ADRs indexed in [`DECISIONS.md`](DECISIONS.md).

---

## The things that actually run today

| System | What it is | Status | Where |
| --- | --- | --- | --- |
| **The chain** | A Substrate L1 on the Polkadot SDK. Aura produces blocks, GRANDPA finalises them, and the validator set comes from governance rather than from a config file. Since 22 September 2026 it also holds assets (DRC-369, below), and moves several of them in one all-or-nothing transaction (`pallet-utility`, ADR-053). **Since 29 September it settles sales in CGT and pays royalties** (`pallet-drc369-royalties`, ADR-061), since 1 October it nests assets (ADR-065), and **since 2 October a buyer can hold a purchase to the content they saw** (`buy_exact`) and any client can ask what a sale would pay (the runtime API `Drc369RoyaltiesApi`; `spec_version` 6 that day). **Since 4 October it holds ARQ Wallets** (`pallet-arq-wallet`, index 11, ADR-070), one keyless payout account per game, with rounds and held prizes. **150 tests pass with the runtime built, fmt and clippy clean (4 October, with `pallet-arq-wallet` and its rounds; 124 on 3 October)**; `chain/` has not changed since. **Since 3 October it runs as `Demiurge Devnet` on Railway** (ADR-068), **at `spec_version` 8 since 4 October** (7 for the ARQ Wallet, then 8 for its rounds, each upgraded in place by the owner's sudo key and read back), near block 41,000 on 6 October: two validators and a public node at `wss://rpc.qorsync.dev`, genesis `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a` holding only the owner's sudo and faucet accounts, measured finalising 100 blocks in ten minutes with both validators authoring (`alpha.devnet-live` and `alpha.devnet-finality` met); two validators agree, finalise, and one recovers after being killed (13 of 13 on the final runtime; one earlier run that day missed its 120-second deadline for block 4 by one block and passed everything after); a sale and a remix sale were run on a development node and paid the right accounts | **partial** — the base layer, assets, royalties, nesting and ARQ Wallets are real and live on the devnet; fees, issuance, a treasury and a production network are not | `chain/` · ADR-013, ADR-068, ADR-070 · M3, M4.1, M4.2 (royalties and nesting), M4.6 |
| **DRC-369** | The asset standard. **Its first part exists:** mint a project's commit as an asset identified by a fingerprint of its manifest (every file, each fingerprinted, and the commit), revise it until a one-way switch makes it permanent, one collection per creator, and a list of what each account holds. Only this pallet can create an asset — inside a batch as well as outside one. **Since 29 September:** a mint can name the asset it remixes (bounded at 16 deep), a creator sets royalty terms and can correct them while they hold the work (up to eight recipients and a remix share that never rises once someone has remixed it, ADR-062), and an asset can be listed and bought for CGT, paying the remix's source, then the royalties, then the seller, in one transaction. **Since 1 October:** an owner places one of their assets inside another and takes it out again; cycles are refused (R-2); a tree is at most 8 deep and each asset holds at most 64; a nested asset and the asset holding it stay put (no transfer, sale or burn) until taken out (ADR-065). 25 pallet tests, 26 royalty tests and 16 runtime asset tests (2 October) | **partial** — M4.1 done, M4.2's royalty half (ADR-061) and nesting half (ADR-065) built; state and XP, physics, rental and fractions are not. Moving a parent together with its contents does not exist, and the launcher has no nesting surface. Without an indexer (M5.4) a list of everything for sale can only be read by walking the chain's listing storage, which the launcher's Market screen does (2 October). **Its deposits and weights are placeholders** (U-14, M7.2) | `chain/pallets/drc369/`, `chain/pallets/drc369-royalties/`, `chain/runtime/src/assets.rs` · ADR-025, ADR-047, ADR-052, ADR-061 · M4 |
| **QOR ID** | The identity service. Username or keypair sign-in, JWTs, sessions, email verification, password reset, backup codes, admin routes, agent registration. **166 tests pass** against real Postgres 16 and Redis 7.4 with `--include-ignored` (6 October 2026; 136 on 2 October); its latest migration is 022. It has **avatars** (ADR-079: an image or GIF per account, re-encoded clean at 256 x 256 and kept in its database, served by content URL with a still first frame for GIFs, Report and the owner's removal), **levels and tasks** (ADR-078: XP per QOR ID, the welcome grant recorded as owed, three sign-ups per network address) and **an account page at `/account`** (change the password with the current one, add or change the email address; every change signs all sessions out) and `POST /api/v1/profile/password` (ADR-075: a QOR ID is the username alone, unique, no `#0001`; migration 019 makes the database enforce it), with sign-in for other apps: `/oauth/authorize`, `/oauth/token`, `/oauth/userinfo`, `/oauth/revoke` (ADR-073), with ARQADE the one registered app. Since 2 October a session records when it was last used (at sign-in and at each refresh, shown in the account's list of sessions), and `GET /health` says, as one boolean, whether its email leaves the machine it runs on: the end-to-end scripts refuse to run unless it says no. Its migrations are LF on every platform and a test fails on a carriage return in one, so a Windows and a Linux build can share a database; no test can send real mail, whatever the machine's environment holds | **partial** — live at `https://id.qorsync.dev`, deployed from `main` on Railway; sign-in is real. Session keys and scoped delegation are not built, an access token alone can still register an agent (L5.1), sessions record a placeholder network address, and the welcome grant is recorded as owed, never paid | `services/qor-auth/` · ADR-014, ADR-016, ADR-017, ADR-073, ADR-075, ADR-078, ADR-079 |
| **Operations stack** | QOR ID, its Postgres and Redis, **on Railway** (ADR-063): project `demiurge`, region `iad`. **Demiurge Devnet's two validators and its RPC node on Railway too** (ADR-068, since 3 October; `chain/DEPLOY-RAILWAY.md`). **ARQADE on Vercel** (project `qor-arqade`, Hobby plan) with its Postgres on Neon (ADR-074, since 5 October) | **partial** — since 1 October QOR ID runs there and answers at `https://id.qorsync.dev` (`/ready` 200, measured), holding the owner's restored account. **Email works** (a verification message from it was delivered to Resend's test address on 1 October; a real inbox is untried), and bounce reports are refused until the webhook secret is set. Its Postgres has no public address. The PC stack (ADR-060, superseded) is retired: its containers are removed and `infra/ops/` is deleted; `ci.qorsync.dev`'s record and the Cloudflare tunnel remain for the owner to delete. The devnet answers at `wss://rpc.qorsync.dev`, and ARQADE at `https://qor-arqade-tau.vercel.app`. Monthly cost is not yet measured | `services/qor-auth/DEPLOY-RAILWAY.md`, `chain/DEPLOY-RAILWAY.md`, `products/arqade/README.md` · ADR-063, ADR-068, ADR-074 |
| **CI (GitHub Actions)** | "Pleroma CI" builds and tests every push and pull request to `main` of the public `QOR-MATRIX/demiurge-chain` (ADR-063, ADR-064; Woodpecker, ADR-058, is retired). Two workflows: `ci.yml`, with jobs for the chain (format, lints, tests, on Rust 1.98.1, ADR-072), QOR ID, the launcher (its host tests and the nine browser checks), ARQADE (install, type check, lint, tests, build), a coverage report, and security (dependency audit and a committed-credential scan, which also fails the run if `framework/` returns); on a schedule, two validators and a report of the newest Rust's clippy lints that never fails the run. `devnet-image.yml` builds the devnet's node image, started by hand only | **built** — green on `main` (6 October 2026): every completed run on `main` since the merge of #6 (5 October 2026) passed. Before that the merges of #2 to #4 failed, and PR #8 was merged with its launcher job red (a flaky browser-check wait, fixed in #9). The chain job failed at Lints from Rust 1.99 (28 September) until the pin (ADR-072, 4 October); a flaky wait in the launcher's browser checks was fixed on 6 October (pull request #9) | `.github/workflows/ci.yml`, `.github/workflows/devnet-image.yml` · ADR-063, ADR-064, ADR-072 |
| **QOR Launcher** | The desktop application, **version 0.1.8** (installer built on 6 October 2026; unsigned, and with no update channel, L6). A vault holding Sr25519 keys whose key is in the OS keychain, opened with nothing asked (ADR-056); QOR ID sign-in by the vault's key, with no password and never blocking the launcher; sending the currency and minting assets against a real node; an Inventory read from the chain, where each asset is a card that can be sent to another account, traded, listed for sale on chain (Sell) and taken off sale (Withdraw), and bought; **the Market screen** (2 October): every listing on the chain, with what a sale pays the chain's own answer (`Drc369RoyaltiesApi`), so the launcher carries no copy of the arithmetic; a chain view; a release-gate dashboard reading CI from `QOR-MATRIX/demiurge-chain` by name; Qontrol's Projects surface with line-by-line diffs, branches and a guard before committing; settings, themes and the QFX backdrop (one shader: a drift and a lift under the pointer). **`qor://pay`, since 0.1.7** (ADR-076, ADR-077): it pays a request a website signs, after the host's own dialog (not Windows Hello), for ARQADE only, on the devnet only, at most 100,000 CGT; the owner used it to tip on ARQADE on 5 and 6 October. **A level bubble and an XP bar** from QOR ID (ADR-078). **On `main` after 0.1.8, so in the next build:** the account's **avatar**, with a ring that glows from level 1, chosen with the native file picker and kept as a copy on the machine so it shows offline (ADR-079, ADR-080). Its default chain is the devnet, `wss://rpc.qorsync.dev`, since 3 October, and its default QOR ID address `https://id.qorsync.dev/api/v1` since 30 September; a saved setting still wins. The Nexus still shows nine frozen-app tiles (see "Frozen" below). **204 Rust tests pass (8 ignored) and nine browser checks pass: design 3, accessibility 41, gates 34, Projects 104, Inventory 238, Market 127, vault gate 65, contrast 54 and readability 361 (6 October 2026)** | **partial** — L0 done; L4.2, L4.4 and L4.5 done; **L1.6 and L1.7 ticked on 6 October 2026** (the host tests and browser checks run and pass in CI on `main`); L1.4 and L2.2 are open. Not signed, no update channel, no Market search (no indexer), royalty terms cannot be set from it, and it has no nesting surface | `tools/qor-launcher/` · L0–L7 |
| **Creator-God Token** | The currency everything settles in. 18 decimals, 1 CGT = 10^18 Sparks, existential deposit 100 CGT | **partial** — the unit is real. **No supply, no issuance, no fee, no treasury** — every one of those needs a number nobody has decided | `chain/runtime/src/denomination.rs` · ADR-002–008 |

**The honest summary of the chain:** it produces and finalises blocks, moves the currency between
accounts, and since 22 September 2026 mints, revises and fixes assets and lists who holds them. It cannot
charge a fee or create currency, because neither is decided, and an asset cannot yet have state.
Since 29 September 2026 it pays royalties on a sale it settles in CGT, since 1 October an asset can be nested
inside another, and since 4 October each game can have an ARQ Wallet. It runs publicly as Demiurge Devnet
(`spec_version` 8); there is no production network.

---

## The launcher's surfaces

The strip down the left of the launcher. `DIRECTION.md` §1 names the same list.

| Surface | What it does | Status |
| --- | --- | --- |
| **Gate** | What shows before the vault is open: a first run, restoring from a recovery phrase, or an older vault's one last Hello or passphrase. A returning person never sees it (ADR-056) | **built** |
| **Nexus** | The map you land on. Every system in one place, with the account's level and XP bar and, from the build after 0.1.8, its avatar | **built** — it still shows nine frozen-app tiles that do not work ("Frozen", below) |
| **Vault** | Hold, send and receive the currency. The only thing that signs. **Since 28 September 2026 there is no lock screen: the vault's key is in the operating system's keychain and nothing is asked to open it** (ADR-056, superseding ADR-055's Windows Hello the same day). The recovery phrase is the way back; restoring sets the old vault aside, never deleted; older vaults move once | **built** — tested against an in-memory keychain and once against this PC's real Credential Manager, and its screens in a browser (`check-vault-gate.mjs`, 65 checks, passing on 6 October 2026, not yet proven to fail against planted faults). The owner has signed tips with it from an installed launcher (5 and 6 October) |
| **Inventory** | The DRC-369 assets an account holds — name, fingerprint, pinned commit, permanent or not — read from the chain every time; **Make permanent**; **an asset's own menu, sending assets to another account, listing one for sale and withdrawing it, and buying by number**; and settled history | **partial** — the assets half is built (L4.2, 238 checks against the built view, 6 October 2026); since 22 September each asset is a card that leans towards the pointer and opens at size (P2.8), and its menu sends several assets in one all-or-nothing transaction (L4.4, L4.5, ADR-053). Since 23 September the trade window is two sides with a lane between them and offers accounts traded with before, held in this machine's own file and sent nowhere. **Since 1 and 2 October Sell lists the asset on chain and Withdraw takes it off sale** (`src-tauri/src/chain/sales.rs`, `list` and `unlist`, each through the host's dialog), and the Market screen shows the listing to everyone; only the description form stays on this machine and publishes nothing, because the chain holds a listing's price and nothing else. History still needs the indexer (ADR-028), and the page says so |
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

## The seven products

Built on one substrate: every product signs in with QOR ID, stores creations as DRC-369 assets, versions
them with Qontrol, distributes through the Mesh and settles in the currency. The decisions they share are
ADR-046 to ADR-051, **all accepted** (ADR-047 and ADR-051 on 22 September 2026, the rest on 28 September).
Each has a blueprint, a track in `DIRECTION.md` and a release gate of its own. **ARQADE is live on the web**
(since 5 October 2026); Qontrol, QFX and Market have a first slice inside the launcher; the rest are not
started.

| Product | What it is | Status | Where |
| --- | --- | --- | --- |
| **Qontrol** | Version control for people who make things. On disk it is an ordinary git repository; the creator sees plain words, never git's vocabulary | **partial** — the Projects surface is built, diffs included (P1.1, 2026-09-23), with branches, discard, per-file commit, a guard before staging and its helper bundled (P1.2, 2026-09-26), and publishes a version as an asset (half of P1.6); P1.3 to P1.5 and P1.7 are not | `tools/qor-launcher/src-tauri/src/qontrol/`, `tools/qor-launcher/qontrol-git/` · [blueprint](blueprints/qontrol.md) · P1 |
| **QFX** | The launcher's living visual layer, in three layers: a backdrop, an interface that responds, a personal space. Creator-made, and the chrome keeps text readable whatever a theme paints | **partial** — layer one's first slice, the backdrop, is built, and layer two's first, the DRC-369 card; the sheen, previews and 3D are not, and layer three is not | `tools/qor-launcher/src/qfx/` · [blueprint](blueprints/qfx.md) · P2 · ADR-051 |
| **QOR Engine (QQ)** | Since ADR-081 (6 October 2026), **QQ**: an engine and editor for small, effects-led 2D and 3D games, a surface of the launcher and playable on ARQADE, signed in with QOR ID, driveable by an LLM through an agent interface, and generating assets and worlds from a description. The Godot build it was planned as is withdrawn | **planned** — P3. Nothing written; QQ's blueprint and roadmap items are not written yet, and P3's six Godot items stand marked superseded | [old blueprint, superseded](blueprints/qor-engine.md) · P3 · ADR-081 |
| **GNOSIS** | Music production where a song is a folder of small text files two people can edit and merge | **planned** — P4. Nothing written | [blueprint](blueprints/gnosis.md) · P4 |
| **Market and Library** | Listing and buying work in the currency; downloading, verifying, installing, patching and launching it. One listing model for games, tools, plugins, scenes, presets and themes | **partial** — P5, carrying L7.1 and L7.2. Market's first slice exists in the launcher (the Market surface above: every listing on chain, buying, no search); Library is a placeholder page | [blueprint](blueprints/market.md) · P5 |
| **Stream** *(placeholder name)* | A music listening service first, with licensing, over the Mesh; a work's royalty split applied when it is paid for | **planned** — P6. Its first item is a decision, deferred to M6; nothing else before M8 | [blueprint](blueprints/stream.md) · P6 |
| **ARQADE** | The gaming platform the owner named on 4 October 2026: games played with a QOR ID, every owned collectible a DRC-369 asset, CGT the only currency, and a developer SDK. It started as an arcade on OpenAI Sites; that copy is no longer ARQADE. **Since 5 October 2026 it is a standard Next.js site on Vercel with its Postgres on Neon (ADR-074)**, at `https://qor-arqade-tau.vercel.app`, its source in `products/arqade/` (active scope, ADR-069) | **partial** — P7. **Live on Vercel since 5 October 2026:** QOR ID sign-in only (ADR-073; players are QOR ID accounts), Play Now, solo games, the Flux Four and Rift Reversi multiplayer arenas, rankings, chat, devnet reads (test CGT and DRC-369 assets at the finalised block, P7.2), tips paid through the QOR Launcher's `qor://pay` (ADR-076, ADR-077; the owner tipped successfully on 5 and 6 October), and the level bubble and level row (ADR-078). Its database migrations 0001 to 0003 run on each production build. **48 tests pass (6 October 2026).** The SDK's first pieces are in `products/arqade/sdk/` (CGT amounts as integer Sparks, ARQ Wallet policy checks, wallet addresses and round ids, the chain calls' arguments, and the project profile with its stage checks and the owner's 10,000 CGT price ceiling). ARQ Wallets, one keyless payout account per game (ADR-070), are built on the chain as `pallet-arq-wallet` and live on the devnet (rounds and held prizes since `spec_version` 8, 4 October). Self-publishing, backing and agentic creation (ADR-071) are accepted and not built beyond the SDK's checks. P7.1 is done (5 October). **Not built:** avatars in ARQADE (ADR-079 step 3), ARQADE inside the launcher (P7.18), paying multiplayer winners from an ARQ Wallet, randomness, editions and account-bound assets. It runs on Vercel's Hobby plan, which is not for commercial use | `products/arqade/` · [blueprint](blueprints/arqade.md) · ADR-069, ADR-074 · P7 |

---

## Planned, with a roadmap item

Not written. Scheduled.

| System | What it is | When | Blocked by |
| --- | --- | --- | --- |
| **The rest of DRC-369** | State and XP, physics, rental, fractional ownership (nesting with cycles refused is built, ADR-065) | **M4.2 to M4.5** | Nothing technical. M4.4, sponsored deposits, waits on U-4 and OPEN-4 |
| **The SDK** | Builds and signs transactions from the runtime's own metadata. Published only after the wire format is frozen | **M5.1** | The freeze, which cannot happen while Q-19 and Q-20 are open (Q-18 was answered, ADR-057) |
| **Agent rails** | Delegated keys with scoped capabilities and spend caps enforced by the chain, not by a database | **M5.2** | M4 |
| **The MCP server** | One server any LLM can drive, instead of a plugin per vendor. Reference capability: generate-and-mint | **M5.3** | M5.2 |
| **The public viewer** | A shareable page per published work — the growth surface. No sign-in | **M5.4** | The indexer |
| **The remote console** | A thin web console. Read, manage, transact. Deliberately never a second product | **M5.5** | — |
| **The indexer** | Reads chain events into a database so history can be shown. A Substrate node serves no history | **M5.4** | An archive node |
| **The archive node** | A node that keeps everything, feeding the indexer. Its disk grows forever | **M5.4** | — |
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

1. **M4.2's royalties (ADR-061) and nesting with cycles refused (ADR-065) are built; its state and XP, M4.3 and
   M4.5 can start now.** None needs an economic value.
2. **M4.4, sponsored fees and deposits, waits on U-4 and OPEN-4**: sponsoring a fee needs a fee.
3. **The deposits a mint holds are placeholders** (U-14). They are derived from the existential deposit so
   a development chain can run, and a public network cannot open on them.
4. **The format cannot be frozen** — and no SDK published — while two questions are open (**Q-18**, eight royalty
   recipients, was answered by the owner on 28 September: eight, ADR-057): **Q-19**, whether an asset's fingerprint depends on its files alone or also on the commit it came
   from; and **Q-20**, whether the table that labels a file's media type is part of that fingerprint, since
   the label is inside the hashed manifest and the format names no table. `beta.royalty-recipients`,
   `beta.manifest-identity` and `beta.media-types` count them.
