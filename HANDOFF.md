# Handoff

**Current state, 6 October 2026.** The code is the public repository `QOR-MATRIX/demiurge-chain` (ADR-063, ADR-064).
`main` is at `9f8a818` (PR #13), and Pleroma CI is green on `main`: every completed run on `main` since the merge of #6 (5 October 2026) passed. Before that the merges of #2 to #4 failed, and PR #8 was merged with its launcher job red (a flaky browser-check wait, fixed in #9). Work goes on a `session/*` branch and
reaches `main` by pull request. `origin` is the private archive, not the current tree. Live, devnet-only and with test
CGT: Demiurge Devnet (`wss://rpc.qorsync.dev`), QOR ID (`https://id.qorsync.dev`) and ARQADE
(`https://qor-arqade-tau.vercel.app`). §1 has the detail.

**Newest: §4 item 58 (6 October 2026): documentation brought up to date with the tree after the IBM Bob audit.**
Before it: item 57 (6 October 2026): avatars (ADR-079) and effects allowed (ADR-080), merged to `main` (PR #13); item 56
(6 October 2026): levels and tasks built (ADR-078), merged (PR #11), then migration 021 and launcher 0.1.8 (PR #12); item
55 (6 October 2026): ADR-078 accepted, levels, tasks and a 100 CGT welcome grant (PR #10), with "Play Now" and the fix
for a flaky wait in the launcher's browser checks (PR #9); item 54 (5 October 2026): P7.4 built, tips through the QOR
Launcher (ADR-076, ADR-077), merged (PR #8); item 53: QOR ID account page, merged (PR #7); ADR-076 accepted (cap 100,000
CGT); item 52: one name per QOR ID, no #0001 (ADR-075), merged (PR #5); item 51: QOR ID sign-in live for apps; ARQADE
moved to Vercel with Postgres (ADR-074), merged (PR #2, then PRs #3, #4 and #6); item 50: P7.3 built, QOR ID signs people
in to ARQADE, merged (PR #1); item 49: P7.1 done, ARQADE renamed, inventory written, lint a gate; item 48: P7.2 done,
ARQADE reads any account from the devnet; item 47: Demiurge Devnet at spec_version 8, rounds live; item 46: rounds with
held prizes and the SDK's wallet module; item 45: Demiurge Devnet upgraded to spec_version 7 by the owner, the ARQ Wallet
live; item 44: the upgrade rehearsed; item 43: the chain pinned to Rust 1.98.1 (ADR-072, the owner's option A); item 42:
merged to `main`; the chain CI job red since Rust 1.99, the owner's call; item 41: `pallet-arq-wallet` built and tested,
before it reached the devnet; item 40: ADR-070 and ADR-071 accepted, the wallet named ARQ Wallet; item 39: ARQADE as a
self-publishing store (ADR-071); item 38: Game Vaults designed (ADR-070) and the ARQADE SDK started; item 37: ARQADE in
`products/arqade/`, reading the devnet; item 36 the same day: ARQADE, the gaming platform the owner named, reconciled
with the tree — a corrected brief (`docs/blueprints/arqade.md`), ADR-069, track P7, gate `arqade`, U-16 and gaps G-14 to
G-16. Before those, §4 item 33 (2 October 2026): half-finished, unrecorded work found in the tree was finished and
verified (chain `spec_version` 6 with `buy_exact`; the launcher's Market host read; QOR ID's last-used sessions and the
e2e email guard). It is on `main`, and the chain has since moved to `spec_version` 8.

**History, 13 to 30 September 2026.** §4 item 32 (29 and 30 September) made the code public, moved CI back to GitHub
Actions and began moving QOR ID to Railway; all three are done (ADR-063, ADR-064), and QOR ID is live on Railway. Before
it, §4 item 31: M4.2's royalty half — royalties, remix royalties and a sale settled in CGT (ADR-061), proven live on a
development node. Before it, §4 items 27 to 30: CI on Woodpecker and the operations stack, both since retired. Before
those, §4 item 25 (28 September): onboarding — a QOR ID bubble, a glowing notification and a tutorial, an intro splash
and a first-run slot, Agora renamed Social, explanations behind information icons, and QOR ID and a chain running on
this PC. Before it, §4 item 24: no lock screen. The vault's key is in the OS keychain, nothing is asked to open it, and
QOR ID never blocks the launcher (ADR-056, the owner's decision, superseding ADR-055 the same day). Before it, §4 item 23:
Windows Hello as the vault's only seal (ADR-055). Before that, §4 item 22: a view check for the Gate's restore screens
and Windows Hello. Before that, §4 item 21: unlocking the vault with Windows Hello as an opt-in (ADR-054). Before it, §4
item 20: restoring a vault from its recovery phrase when the passphrase is lost. Before that, §4 item 19: Qontrol's git
layer completed — branch, switch, discard, per-file commit, a guard before staging, and the helper bundled with the
launcher — which ticks P1.2. Before it, §4 item 18: Qontrol's line-by-line diffs, reading line endings and
`.gitattributes` as git does, which ticked P1.1. Before that, §4 item 17: the trade window reworked into two sides with a
lane, "recently traded with", and Sell drafting a listing that publishes nothing. Before that, §4 item 14: M4.1, the
first asset pallet, and the first asset minted from a Qontrol project and shown in the launcher's Inventory. M2.1 was
ticked the same day against the owner's review of a ten-line summary.

**Read this first, especially §2.** Then:
1. [`docs/DIRECTION.md`](docs/DIRECTION.md): what Demiurge is, and the only roadmap, including the launcher
   track.
2. [`docs/DECISIONS.md`](docs/DECISIONS.md): the only decision log, with the ADR index.
3. [`docs/GATES.toml`](docs/GATES.toml): the accepted release gate criteria. Read its rule on changing
   evidence before touching it.
4. [`docs/architecture/MIGRATION_INVENTORY.md`](docs/architecture/MIGRATION_INVENTORY.md): **all but
   Q-19 and Q-20 of its twenty questions are decided** — Q-1 to Q-16 by ADR-017 to ADR-032, confirmed by
   the owner on 2026-09-17, **Q-17 by ADR-041 on 2026-09-20**
   ([`docs/architecture/ADDRESS_TYPE.md`](docs/architecture/ADDRESS_TYPE.md) is the write-up it was
   decided from), and **Q-18 by ADR-057 on 2026-09-28** (eight royalty recipients stay). **Q-19 and Q-20
   are open until before the freeze** (§4 item 14). **M2.1, the owner's
   review of it, was ticked on 2026-09-22** against a ten-line summary, kept verbatim in the document.
5. [`docs/audit/RECONCILIATION.md`](docs/audit/RECONCILIATION.md): the code measured against the
   direction, with a remediation table.

---

## 1. Status at a glance

| Area | State |
| --- | --- |
| Direction | Settled. Demiurge is a creative engine on a purpose-built Substrate L1, with four primitives. The launcher is a first-class deliverable with its own track (L0 to L7). **The active product track is P7, ARQADE** (ADR-069, active scope since 4 October 2026). |
| Base layer | Migration to the Polkadot SDK decided (ADR-013) and complete since M3.5 (20 September 2026). **Q-1 to Q-16 decided** (ADR-017 to ADR-032, 15 September 2026), confirmed by the owner on 2026-09-17; Q-17 (ADR-041) and Q-18 (ADR-057) decided since; Q-19 and Q-20 open before the freeze. Pinned to `polkadot-stable2606-1` (ADR-022), and the chain builds with Rust 1.98.1 (ADR-072). |
| The Substrate chain (`chain/`) | **`spec_version` 8. 150 workspace tests pass with the wasm built (4 October 2026).** Pallets: `pallet-drc369` with `pallet-drc369-royalties` and nesting (ADR-052, ADR-061), `pallet-nfts` (ADR-052), `pallet-utility` with only `batch_all` reachable (ADR-053), and `pallet-arq-wallet` at index 11, rounds with held prizes included (ADR-070). The validator set comes from governance through `pallet-session` (ADR-020); an address is a `MultiAddress` (ADR-041). **No transaction payment (OPEN-4), no treasury (OPEN-2) and no issuance (OPEN-1)**; deposits are placeholders (ADR-052, U-14). **Live as Demiurge Devnet** at `wss://rpc.qorsync.dev`, genesis `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a`: two validators and an RPC node on Railway (ADR-068), and the owner holds the sudo key. `chain/` last changed on 4 October 2026. History: first finality 17 September (M3), M4.1 22 September, royalties 29 September, the ARQ Wallet 4 October (§4 items 14, 31, 41 to 47). |
| The custom devnet (`framework/`) | **Retired and deleted at M3.5, 2026-09-20.** 150 files, 49,058 lines, after everything that referred to it had been rewritten and verified with the tree still present. Its last run passed 285 tests. Untrusted, and nothing of it is carried forward (§2.0). **M3 is not complete:** M3.2 waits on OPEN-2. |
| Security track | Items 1 to 7 done. **The owner confirmed credential rotation on 2026-09-14:** the Postgres password and both JWT secret pairs. **Nine credential-shaped values** (from `docker/n8n/docker-compose.yml` and `docker/docker-compose.testnet.yml`) are not in the current tree or in the public repository's history; they exist only in the private archive's history, and rotating them is still advised if those services ever ran (an owner step in `OWNER.md`). CI's security job scans for committed credentials. **A recovery phrase was pasted into a chat on 5 October; treat that key as exposed and replace it.** Sessions record a placeholder IP address. §3. |
| `services/qor-auth` (QOR ID) | **Live at `https://id.qorsync.dev`**, the Railway service `qor-auth`, deployed from `main`. **166 tests pass** (6 October 2026, against Postgres 16 and Redis 7.4, `--include-ignored`), the log and SQL checks among them. Latest migration: 022. Password and key sign-in (Sr25519 and SS58; ADR-023, ADR-024, ADR-039); email verification and reset through Resend; the account page `/account` (change the password, add or change the email); sign-in for other apps by OAuth 2.1 with PKCE (ADR-073), the apps registered in `QOR_OAUTH_CLIENTS`, ARQADE among them; one name per account, no `#0001` (ADR-075, migrations 019 and 021); levels, XP and tasks, the welcome grant recorded as owed, and three sign-ups per network address read from Railway's `X-Real-IP` (ADR-078, migration 020); avatars (ADR-079, migration 022). Registration creates no CGT and calls no chain (R-3). Agent keys are authorised, never created (ADR-014). **The Resend bounce webhook is an owner step, and delivery to a real inbox is unconfirmed** (§4.0). One known gap (§2). History: §4 items 5 to 8, 33 and 50 to 57. |
| `tools/qor-launcher` | **Version 0.1.8** (`tauri.conf.json`). **204 Rust tests pass (8 ignored without a node), and the nine browser checks pass** — design 3, accessibility 41, gates view 34, Projects 104, Inventory 238, Market 127, vault gate 65, contrast 54, readability 361 — all on 6 October 2026. The 0.1.8 installer was built locally and is **unsigned, with no update channel** (L6); 0.1.7 added `qor://pay`. Avatars and rings reached `main` after 0.1.8 and need a new build to ship. The default chain endpoint is `wss://rpc.qorsync.dev` (`DEFAULT_RPC` in `src-tauri/src/chain/mod.rs`; `LOCAL_RPC` is `ws://127.0.0.1:9944`). Features: the vault's key in the OS keychain with no lock screen (ADR-056); QOR ID sign-in by key; send, mint, Inventory, trade, sell and buy; the Market view (no search, no indexer); Projects and Qontrol; the gates dashboard; `qor://pay`, approved in the host dialog (ADR-077); the level bubble and XP bar; the avatar with its ring; one QFX backdrop shader. The Nexus still shows nine frozen-app tiles marked "local" or "forming". L1.1, L1.2, L1.3, L1.5, L1.6, L1.7, L3.1 and L3.2 are done (L1.6 and L1.7 ticked on 6 October 2026 against CI). **L1.4 is partly exercised:** the owner approved real `qor://pay` tips in the host dialog on 5 and 6 October; a decline and an endpoint change are not yet exercised (§4.0). History: §4 items 9, 14 to 24 and 54 to 57. |
| Products | **Seven, each with a blueprint, a track and a gate**: P1 to P6 (§4 item 12) and **P7, ARQADE** (ADR-069). The shared substrate decisions ADR-046 to ADR-051 are all accepted: ADR-047 and ADR-051 on 2026-09-22, ADR-046, ADR-048, ADR-049 and ADR-050 on 2026-09-28. Built inside the launcher: Qontrol's Projects (P1.1, P1.2), QFX's backdrop (P2.1) and the Market view. ARQADE is its own site (row below). QOR Engine, GNOSIS and Stream have not started. |
| Release gates | **`GATES.toml` accepted (2026-09-14).** Coverage: cargo-llvm-cov, 80% of lines, only on pallets that move CGT or own DRC-369 semantics. Public Release gained `public-release.name-clearance` on 2026-09-17, a tightening logged in the file. **On 2026-10-06 (ADR-080) `check-design.mjs` dropped its five effect rules and the QFX and ceremony exemptions**, keeping colour tokens, the type scale and the tracking scale; it is logged in the change log with the owner's approval. **Loosening any evidence rule needs the owner's separate decision** (the rule is in the file). |
| CI | **"Pleroma CI", GitHub Actions on the public `QOR-MATRIX/demiurge-chain` (ADR-063, ADR-064), and green.** Every completed run on `main` since the merge of #6 (5 October 2026) passed. Before that the merges of #2 to #4 failed, and PR #8 was merged with its launcher job red (a flaky browser-check wait, fixed in #9). Jobs run in the `QOR-MATRIX` organisation, where the billing lock that refused the personal account's jobs in late September does not apply (ADR-064), and `probe.yml` is gone: the workflows are `ci.yml` and `devnet-image.yml`. Jobs: `chain` (format, lints and tests, without `SKIP_WASM_BUILD`); `chain-newest-clippy` (the newest Rust, report-only, scheduled); `two-validators` (scheduled); `qor-auth` (with Redis, `--include-ignored`); `launcher` (with the nine browser checks); `arqade`; `coverage`; and `security` (dependency audits, the committed-credential scan, the `[patch]` guard and the `framework/` guard). A flaky wait loop in the launcher's browser checks was fixed on 6 October (PR #9). Woodpecker and `ci.qorsync.dev` are retired; deleting the `ci` DNS record in Cloudflare is an owner step. Scope changes are logged in `GATES.toml`. History: §4 items 1, 27 to 32 and 42. |
| Infrastructure | **Railway** (project `demiurge`): Postgres, Redis, QOR ID (`id.qorsync.dev`) and Demiurge Devnet (two validators and an RPC node, ADR-068). **Vercel**: ARQADE, project `qor-arqade` (team Astra Matrix, Hobby plan), with Neon Postgres (ADR-074). Fly.io is not used: ADR-015's Fly plan is superseded by ADR-063 and ADR-068. |
| Production | No production or test network. Live, devnet-only and with test CGT: Demiurge Devnet, QOR ID and ARQADE. |
| ARQADE (`products/arqade/`) | **Live at `https://qor-arqade-tau.vercel.app`**: standard Next.js 16 on Neon Postgres, migrations 0001 to 0003 applied on production builds (`scripts/migrate.mjs`). **48 tests** (6 October 2026). QOR ID is the only sign-in; Play Now, the solo games, Flux Four and Rift Reversi multiplayer, rankings, chat, devnet reads, tips through the launcher (ADR-076, ADR-077; the owner tipped successfully on 5 and 6 October), and the level bubble and level row. P7.1 to P7.4 are ticked (P7.4 on 6 October 2026). |
| Decisions | 80 ADRs (ADR-001 to ADR-080); ADR-069 to ADR-080 are accepted. The owner's open questions are in §4.0. |
| Not live | Avatars in ARQADE (ADR-079 step 3, not built), paying the welcome grant (grants are recorded as owed, never paid), ARQADE inside the launcher (P7.18), the indexer, the Mesh, fees and issuance. |

## 2. Known gaps: do not build on these

### 2.0 There is one chain

**`chain/` is the chain.** `framework/`, the custom Rust devnet it replaced, was retired at M3.5 on 20
September 2026: everything that referred to it was rewritten first and verified with the tree still
present, then the directory was deleted in a commit of its own.

Everything below about two chains is history, kept because a lot of this file and a lot of muscle memory
were shaped by it.

| | `chain/` |
| --- | --- |
| What | The Substrate L1 (ADR-013), located and named by ADR-032 |
| Chain identity | `demiurge_dev` / "Demiurge Development", `demiurge_local` / "Demiurge Local Testnet", or `demiurge_devnet` / "Demiurge Devnet", the live devnet (ADR-068; `chain/node/src/chain_spec.rs`) |
| Finality | GRANDPA (ADR-018) |
| RPC | Standard Substrate. The launcher speaks it over WebSocket (ADR-040); its default is `wss://rpc.qorsync.dev`, and `ws://127.0.0.1:9944` is `LOCAL_RPC`, for a local node |
| Crate and binary | `demiurge-node` |
| Address type | `AccountIdLookup`; an extrinsic's address is a `MultiAddress` (ADR-041) |

**What went, and what remains of it.**

- The custom Rust devnet in `framework/`, its `demiurge-node-legacy` binary, its private RPC vocabulary
  (`chain_getHealth`, `chain_getBlockNumber`, `account_getTransactionNonce`, …), its 285 tests and its
  six exempted advisories went together, in one commit.
- **It was untrusted, and that is why nothing of it is carried forward.** Its transaction validation used
  non-strict Ed25519 verification, and a test on 2026-09-14 showed it accepting a signature nobody made
  for small-order public keys: the identity-point key on 64 of 64 messages, the all-zero key on 13 of 64.
  Any account with such a key could be spent from by anyone.
- **What it established is not lost.** Its behaviour is in `chain/`'s acceptance tests and the
  two-validator script; its module-by-module mapping is in the migration inventory; what it did on the
  wire is in `PROTOCOL.md`'s history. The inventory's requirements outlived it: **R-1** is closed as met
  by standard behaviour (ADR-038), **R-2** is met in `chain/` (`pallet-drc369` refuses a nesting cycle with
  `NestingCycle`), **R-3** is
  closed in QOR ID, and **R-4** is met by `chain/` having been written with `CGT` from the start.
- **Do not bring it back.** CI fails if the directory returns. Reading it in git history is not forbidden
  the way the pre-realignment documentation is (AGENTS.md §1) — it is code, not a contradicting plan —
  but nothing in it should be copied forward.

**Two traps it left behind, which will outlive it by a while.**

- **`demiurge-node` means one thing.** For a few days two crates built a binary of that name, and from
  2026-09-17 the old one was renamed `demiurge-node-legacy` to stop it. Both names now resolve to one
  place or to nothing: `cargo build -p demiurge-node` in `chain/`, and `demiurge-node-legacy` nowhere.
- **Port 9944 is no longer contested**, so a node answering there is `chain/`. `system_chain` is still the
  way to be sure, and the launcher asks automatically and shows the answer.

Two records mention the old chain and are deliberately unchanged: `PLATFORM_REALIGNMENT.md`, which
records what was launched on a date, and every ADR that referred to it. An ADR is never edited.

**The key scheme changed on 19 September 2026, and no key stored before it still works.**
- Accounts are Sr25519, derived as the ecosystem derives them (ADR-023), and addresses are SS58 (ADR-024).
- Every key QOR ID held was an Ed25519 key. Those same 32 bytes now name an account nobody holds a secret
  for, so migration 018 cleared them rather than binding a QOR ID to an account it could never prove again
  (ADR-039).
- **A password account links its new key** through `link-keypair`. **An account created by key sign-in
  before that day cannot sign in at all** and has to be created again: its password is a random value
  nobody knows. Nothing was deployed on 19 September, so the accounts this touched were development accounts.
- **A vault created before that day opens different accounts.** The phrase is unchanged and nothing is
  lost; the addresses it derives are new ones, which is what ADR-023 accepted while nobody holds anything.
- **The wire changed with it.** `pubkey` is gone from every request; an account is named by `address`
  (SS58) or by `account_id` (hex), exactly one of the two.

**An access token alone can register an agent** (ADR-014, "Known gap").
- Until the controller's launcher vault signs each agent authorisation (L5.1), agent registration is
  authorised by the controller's session.
- Anyone holding a controller's access token can register an agent for that controller.
- It is **not a design choice and must not become permanent.** L5.1 is a Beta gate criterion, so the
  public testnet cannot open with the gap in place.

**Other gaps, as of 6 October 2026.**
- **Welcome grants are recorded as owed and never paid** (ADR-078). Who funds them with real CGT is U-18.
- **No fees and no issuance.** The chain has no transaction payment (OPEN-4) and no issuance (OPEN-1).
- **The Market view has no search and no indexer**, and transaction history waits on the indexer (ADR-028).
- **Sessions record a placeholder IP address** (`0.0.0.0`, "not yet taken from the request", in
  `src/handlers/auth.rs`).
- **The launcher is unsigned and has no update channel** (L6). A new version is installed by hand.
- **Avatars are not in ARQADE** (ADR-079 step 3, not built). The launcher's avatars are on `main` and wait for a
  new build.
- **The Nexus still shows nine frozen-app tiles** (Explorer, Staking, Sophia, Scatter, Resonance, Worlds, Agent
  Foundry, Developers, Bounties), marked "local" or "forming".
- **The three-sign-ups-per-address limit trusts Railway's `X-Real-IP`** (ADR-078). If QOR ID ever runs behind
  another proxy, the header, and the limit with it, must be checked again.

## 3. Security track and other open findings

Details are in [`SECURITY.md`](SECURITY.md).

| # | Item | State |
| --- | --- | --- |
| 1 | Committed secrets removed from configuration | Done; **rotation confirmed by the owner on 2026-09-14** |
| 2 | Seeded admin removed (migrations 008, 010) | Done; verified on Postgres 16 |
| 3 | Balances self-transfer created CGT | Done, with tests |
| 4 | DRC-369 mint authorisation; parent-owner check on nesting | Done, with tests |
| 5 | QOR ID created and held agent private keys | Done (ADR-014, migration 011); verified end to end |
| 6 | Agent endpoints unscoped; wallet link unbound and unproven | Done; verified end to end |
| 7 | Domain-separated QOR ID challenge signing (launcher and `qor-auth`) | Done; also L1.5. Unit tests on both sides, and the e2e harness shows a bare-challenge signature refused at login and at key link (28 of 28 checks) |

**Migration requirements:**
- R-1: strict signature verification. Closed as met by standard behaviour (ADR-038; §5).
- R-2: refuse nesting cycles. Met in `chain/`: `pallet-drc369` refuses one with the error `NestingCycle`, tested
  in the pallet and the runtime. The custom chain's `do_nest` never refused them, and it is gone.

**Other open findings:**
- The secrets removed on 2026-09-14 are rotated. **Nine credential-shaped values** (from
  `docker/n8n/docker-compose.yml` and `docker/docker-compose.testnet.yml`) are not in the current tree or in the
  public repository's history; they exist only in the private archive's history. Rotating them is still advised if
  those services ever ran (`OWNER.md`). CI's security job fails on committed key material and on credentials in
  configuration files.
- **A recovery phrase was pasted into a chat on 5 October 2026; treat that key as exposed and replace it**
  (`OWNER.md`).
- **New surfaces since October:** avatar upload (an image parser on untrusted input, under size, frame and
  dimension limits; ADR-079) and the OAuth client secrets in `QOR_OAUTH_CLIENTS` (stored as SHA-256 hashes;
  ADR-073). Both are in the log check.
- **The per-address sign-up limit trusts `X-Real-IP`**, which Railway sets (ADR-078); `X-Forwarded-For` is not
  trusted.
- ~~The RPC reports `finality: 2000` and every transaction as "finalized".~~ **Gone with `framework/` at
  M3.5.** `chain/` has real finality (ADR-018), and the launcher reports a transfer only once GRANDPA has
  finalised it.
- There is no limit on concurrent QOR ID sessions; the unenforced setting was removed.
- **QOR ID weaknesses that remain** (2026-09-15; details in `SECURITY.md`). Closed on 2026-09-15:
  - a reset ends every session;
  - backup codes are single-use and hashed, and can be regenerated;
  - sign-in discloses nothing;
  - an account whose verification link lapsed can request a new one;
  - an account without an email can add a verified one (§4, item 7);
  - no token, key, link or address reaches a log. This is a standing check at runtime and in CI, GATES
    `alpha.no-secrets-in-logs`. Token material had reached the logs three times, from three directions.

  Still open:
  - some accounts cannot be recovered; they are named in `SECURITY.md` as accepted lockouts;
  - sessions record their last activity, but their IP address is a placeholder (`0.0.0.0`);
  - ~~a password-only account holds an `on_chain_address` no key can sign for~~. **Closed 2026-09-19**
    (ADR-017, ADR-039): registration stores no address, `hash_to_address` is removed, and migration 018
    cleared the derived addresses already stored.
- Agent capabilities and spend caps are recorded, not enforced; enforcement is on chain (Q-9).

## 4. What is next

### 4.0 What is blocked, and on whom

Read this before the detail below. "Ready" means it can start today with nothing outstanding.

| Track | State | Waiting on | What unblocks it |
| --- | --- | --- | --- |
| **M3, the Substrate chain** | **Unblocked, started 2026-09-17** | Nobody | The owner confirmed ADR-018 to ADR-032 and supplied all four inputs: eighteen decimals (ADR-035), an existential deposit of 100 CGT (ADR-036), the five pallet names, and `pallet-sudo` on development and test networks only (ADR-037). |
| **U-1, the decimal places** | **Decided 2026-09-17** | Nobody | Eighteen, on both recorded conditions (ADR-035). Removed from `OPEN_QUESTIONS.md`, so the gate reads it as decided. |
| **The existential deposit** | **Decided 2026-09-17** | Nobody | 100 CGT (ADR-036), `10^20` Sparks. Revisitable before mainnet only, and only if OPEN-2's sponsorship budget changes the arithmetic. |
| **L3.1**, the launcher's chain client | **Done, 2026-09-20 (ADR-040)**, which completes M3.4 | Nobody | The client is `subxt` at 0.51, under ADR-033 rule 2, with every call built from the metadata the node serves. Proven against a development node: an approved transfer signed in the vault and finalised by GRANDPA, a declined one moving nothing, and two sent back to back. Two calls still refuse, and for reasons that live elsewhere: **transaction history** waits on ADR-028's indexer, and **the starter claim** waits on the economics (OPEN-1, OPEN-2). |
| **Q-17**, the runtime's address type | **Decided and done, 2026-09-20 (ADR-041)** | Nobody | The owner chose `AccountIdLookup`, and it was carried out the same day: runtime and launcher in one commit, because a node and a client on opposite sides of the change refuse each other. 51 chain tests, 13 two-validator checks and the launcher's live test all pass on it. |
| **L4**, the studio | **L4.2, L4.4, L4.5 done**; **half of L4.6 built 2026-09-23, unticked**; L4.1 started | Nobody, to continue; **the owner** for the listing vocabulary | Owned assets are listed from on-chain enumeration (L4.2), traded (L4.4, L4.5), and since 2026-09-23 a listing can be drafted for one — saved on this machine and published nowhere, so **L4.6 stays unticked** until M4.2, P5.5 and M5.4 give it somewhere to publish. The six categories it offers are a proposal for the owner to confirm (§4 item 17). Minting any file, and collections beyond a creator's singles, are L4.1's remainder. |
| **L1.4**, host-side confirmation | **Partly exercised** | Nobody | The owner approved real `qor://pay` tips in the host dialog on 5 and 6 October 2026. Still needed in a running launcher: a decline, and an endpoint change approved once and declined once (item 4 below). |
| **L2.2**, the gates dashboard | Part verified, 2026-09-17 | Nobody | The numbers are now covered from both sides: the host against the real `GATES.toml`, and the view against a known fixture in a real rendering engine (`scripts/check-gates-view.mjs`, 34 checks, wired into CI and `npm run check`). What is left needs a person: running a suite with one approval and one refusal, and the `gh` readings (item 4 below). |
| **L1.6**, launcher tests in CI | **Ticked 6 October 2026** | Nobody | The `launcher` job runs and passes in CI on `main` (every completed run since the merge of #6, 5 October 2026; on the merge of #13, run `37496555161`, 204 host tests passed and 7 were ignored). Ticked in `DIRECTION.md` against that evidence. |
| **L1.7**, CI runs at all | **Ticked 6 October 2026** | Nobody | CI starts and passes on `main` (the merges of #12 and #13), and the `coverage` job uploads the `coverage` artifact `GATES.toml` reads. Ticked in `DIRECTION.md` against that evidence. |
| **L6, distribution** | Partly done | **Owner**, for signing | Unsigned installers 0.1.7 and 0.1.8 were built locally and installed by the owner. **Signing and an update channel are not done.** Signing needs a Windows code-signing identity and an Apple Developer ID, which the owner was obtaining on 2026-09-17. |
| **The landing page** | **Deferred, 2026-09-17** | Nobody, until the owner wants it | The owner's decision: do not start it and do not extend frozen scope. If it is wanted it gets a roadmap item and its own decision first (item 11 below). |
| **Live email to a real inbox** | Blocked | **Owner** | The Resend webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`, with its signing secret in Railway as `RESEND_WEBHOOK_SECRET` (item 7 below). Delivery to a real inbox is unconfirmed. |
| **Deployment** | **Devnet stage done** | Nobody, for the devnet | QOR ID on Railway (ADR-063), Demiurge Devnet on Railway (ADR-068) and ARQADE on Vercel (ADR-074), all devnet-only with test CGT. No production network exists. |
| **M2.1**, the owner's review of the inventory | **Ticked 2026-09-22** | Nobody | Against the owner's acknowledgement of a ten-line summary of the DRC-369 section, kept verbatim in the inventory. M2 is done. |
| **M4.1**, the asset primitive | **Done 2026-09-22** (ADR-052) | Nobody | §4 item 14. Its deposits are placeholders (U-14) and its weights are placeholders owed to M7.2. |
| **M4.2, M4.3, M4.5** | **Ready**; M4.2's royalty half **built 2026-09-29** (ADR-061) | Nobody; the owner ruled on ADR-061's two choices the same day (ADR-062) | None needs an economic value. M4.4 waits on U-4 and OPEN-4. |
| **The freeze** (`beta.wire-format-frozen`) | Blocked | **Owner**, by an ADR each | Q-19 (whether an asset's root depends on its files alone) and Q-20 (whether the media-type table is part of an asset's identity). `beta.manifest-identity` and `beta.media-types` count them. Q-18 was answered by ADR-057 on 2026-09-28: eight royalty recipients stay. |
| **ADR-047** (the asset format) and **ADR-051** (QFX) | **Accepted 2026-09-22** | Nobody | M2.3 is ticked. Deciding is not freezing: the open items before the freeze are Q-19 and Q-20 (Q-18 was answered by ADR-057). |
| **ADR-046, ADR-048, ADR-049, ADR-050** | **Accepted 2026-09-28** | Nobody | Accepted by the owner. |
| **The QFX check narrowing** | **Approved 2026-09-22, retroactively**; widened by ADR-080 on 2026-10-06 | Nobody | Since ADR-080, `check-design.mjs` keeps only its colour-token, type-scale and tracking-scale rules, with no QFX or ceremony exemption; reduce motion and readability as painted still bind. Logged in `GATES.toml`'s change log with the owner's approval. |
| **The product tracks** | Ready where an item names no dependency | Nobody | **P1.1 is done (2026-09-23)** and **P1.2 is done (2026-09-26)** (§4 items 18 and 19). P1.3 to P1.5, P2.2 to P2.5, P3.1, P4.1, P5.1 and P5.2 can start today, and P6.1 is a decision the owner can take today. P1.6 is half built. **P7 (ARQADE):** P7.1 to P7.4 are ticked (P7.4, tips, on 6 October 2026); P7.16 to P7.18 (ADR-078) are partly built — levels and tasks exist, and ring styles are drawn in the launcher on `main`, but themes as unlocks are not built, no grant is paid, ARQADE is not inside the launcher and there is no abuse watcher; ARQADE's avatars are not built. |
| **Open owner questions** | Open | **Owner** | OPEN-1 to OPEN-4, U-4, U-14, U-15, U-16, U-17, U-18, Q-19 and Q-20 (`docs/economics/OPEN_QUESTIONS.md`, the migration inventory). |
| **Legal review** | Open | **Owner** | A legal opinion on backing, paid games with prizes, the welcome grant, task rewards, and making CGT exchangeable, before any of them goes live. |
| **Trademark clearance** | Open | **Owner** | An attorney's clearance opinion on the project name (`public-release.name-clearance`). |
| **Vercel plan** | Hobby | **Owner** | Vercel's Hobby plan is for non-commercial use. Before ARQADE takes a real payment or pays out real CGT, it moves to a plan that allows commercial use (ADR-074). |
| **The exposed key** | Open | **Owner** | A recovery phrase was pasted into a chat on 5 October 2026; treat that key as exposed and replace it (`OWNER.md`). |
| **The nine credentials** | Open | **Owner** | Rotate them if those services ever ran (§3, `SECURITY.md`). |
| **Cloudflare clean-up** | Open | **Owner** | Delete the `ci` record and the tunnel left from the retired operations stack. |
| **A new launcher build** | Ready | Nobody | Build and install a launcher after 0.1.8, so the avatars and rings on `main` reach the owner. |
| **The QFX backdrop library** | Ready | Nobody | The owner's request (§4 item 57): reactive backdrops (Drift, Aurora, Nebula, Lattice, Starfield, Liquid), combinable pointer effects, presets, and backdrops that evolve with the level. |

Nothing above waits on another track finishing. L1, L2 and L6 run in parallel with M3, and always could.


1. **Resolved: CI runs** (ADR-063, ADR-064). Until late September 2026 no job in the private repository's workflow
   ever executed: every run failed at startup, and a trivial `probe.yml` failed the same way, which put the cause
   above workflow parsing, at the account. CI now runs on the public `QOR-MATRIX/demiurge-chain`, `probe.yml` is
   deleted, and CI is green on `main`.
2. **L1.7's decisions are applied** (2026-09-14, logged in `GATES.toml`):
   - `framework/` audit: six named advisories exempt until M3.5. **Expired with the directory on
     2026-09-20.** CI now fails if `framework/` returns at all.
   - `qor-auth` audit: RUSTSEC-2023-0071 ignored. CI fails if `rsa` enters the compiled graph.
   - `build-blockchain.yml` removed.
3. **ADR-018 to ADR-032 are confirmed** (owner, 2026-09-17): they record the decisions the owner made, for the
   reasons they made them. **M3 is unblocked and started.** All four inputs arrived with the confirmation:

   | Input | Decided | Record |
   | --- | --- | --- |
   | Precision (U-1) | Eighteen decimals, on both conditions: the stale headroom comment corrected, and a written rule that scaled arithmetic uses the SDK's helpers | ADR-035 |
   | Existential deposit | 100 CGT, `10^20` Sparks; revisitable before mainnet if OPEN-2's sponsorship budget changes the arithmetic | ADR-036 |
   | Pallet names | `pallet-drc369`, `pallet-drc369-royalties`, `pallet-sponsorship`, `pallet-agent-caps`, `pallet-validator-set`. **Checked 2026-09-17: none collides.** All five are unregistered on crates.io, which also rules out an SDK collision, since every SDK pallet is published there | ADR-032, confirmed |
   | `pallet-sudo` | Development and test networks only; the owner holds the key; **absent** from any mainnet runtime, with a removal path written now and a gate criterion of its own | ADR-037, `public-release.no-sudo` |

   Open points that remain the owner's, none of them blocking M3:
   - **The address prefix.** Prefix 42 on development and test networks (the owner, correcting Q-7: the registry is
     archived). ADR-024 reports what was found: no registration process and no successor registry. A forum question
     is drafted there and waits for the owner to post it. The mainnet prefix is a Public Release criterion.
   - **The currency's ticker: settled and done (ADR-034).** On 17 September 2026 the owner decided the name stays
     Creator-God Token and the ticker becomes **CGT**, subject to clearance research. That research found nothing
     that bars it: no live trademark on the string anywhere in any class; no crypto asset on any aggregator or chain,
     so a symbol lookup resolves to nothing; absent from the ss58 registry, Talisman's chain data and icon slots, and
     SubWallet's chain list, all three of which carry `CGT`; no regulatory or exploit history; and no collision with
     anything this system already names. The swap is made: the declared symbol, the launcher's nine hard-coded
     fallbacks and the sixteen prose strings. Identifiers, commands, RPC names and the unit constant deliberately
     still read `cgt`, because M3 replaces them; **R-4 in the migration inventory requires the Substrate
     implementation to write `CGT` into names from the start, with no second rename pass.** U-13 has left
     `OPEN_QUESTIONS.md`, and all of its research is the appendix to ADR-034.
     **Carried consciously:** the project's name, Demiurge, has live trademark exposure in classes 9, 41 and 42 (a US
     application by Demiurge Studios, Inc., past its notice of allowance). It does not touch the ticker and no one
     holds the string in class 36, the class a currency occupies. The screening covered identical marks only, by a
     non-lawyer, so an attorney's clearance opinion on the project name is now a Public Release gate criterion,
     `public-release.name-clearance`.
   - **Linking an address is not chain identification (F-Q9).** Prefix 42 is shared, and the same account bytes are
     valid on every Substrate chain, so ADR-024's prefix check validates a shape and nothing more. The inventory
     records the options; the recommendation is that no route stores an address without proof of possession, over a
     challenge bound to the chain's genesis hash. Not implemented: it lands with the SS58 migration.
   - **Sponsorship (U-4):** the design proposal in `docs/architecture/SPONSORSHIP.md`.
   - **U-10** (voting power) is written up in `OPEN_QUESTIONS.md`, to be decided before OpenGov. It carries a known,
     unresolved tension with ADR-020: nominated proof of stake at mainnet could still be secured by genesis-pool holders.
   - **Settled on 17 September 2026:**
     - the fifth pallet is `pallet-validator-set`, the session manager of ADR-020 (instance name `ValidatorSet`);
     - prefix 42 for development and test networks stands, and the forum draft in ADR-024 is the owner's to post.
   - **Settled on 15 September 2026:**
     - `pallet-assets` for fractionalization only, not a reversal of ADR-031;
     - the Ed25519 forgery test stays;
     - the pin stays on `polkadot-stable2606-1`;
     - dependency versions follow ADR-033.
   - **M2.1** (the owner's review of the inventory) and **M2.3** (the DRC-369 wire format) were both ticked on
     22 September 2026 (items 13 and 14).
4. **L2.2, the development dashboard, is implemented and unit-tested.**
   - The host (`src-tauri/src/gates.rs`) applies `GATES.toml`'s counting rules and nothing else. An
     unreadable signal is not met, never unmeasurable, unless the file says so for that kind.
   - The view is the Gates surface, on the rail and in the Nexus.
   - Running a suite is approved in a host dialog first, because it builds and runs repository code.
   - CI and coverage readings need `gh` installed and signed in.

   **The first of the three checks below is now automated** (2026-09-17). `scripts/check-gates-view.mjs`
   opens the built Gates surface in a real rendering engine, answers `gates_report` with a fixture whose
   numbers are known, and reads back what was drawn: the counts, the passed state, every criterion and
   unit, and that no bar appears while anything is unmeasurable. 34 checks. It never reads `GATES.toml`,
   because a check reading the same file as the host would pass if both were wrong together. It was
   proven to fail first, by altering the report handed to the page. It runs in CI and in `npm run check`.

   What still needs a person at a running launcher:
   - run the `launcher` suite, approving once and declining once;
   - read the CI units with `gh` signed in, and again with it signed out.

   Security item 7, L1.1, L1.2, L1.3 and L1.5 are done.
   **Owner:** the Veridian and Abyss themes lean neon, and the Veridian and Sanguine descriptions are not
   restrained (`docs/design/DESIGN_SYSTEM.md` §2). They are unchanged, pending your call.
   L1.4 (host-side confirmation) is implemented and unit-tested. **Since 5 and 6 October 2026 it is partly
   exercised:** the owner approved real `qor://pay` tips in the host dialog. It stays unchecked until the rest of
   its native dialogs are exercised in a running launcher: a transfer and an endpoint change, each approved once and
   declined once, against a local `demiurge-node --dev` (L3.1 gave the transfer dialog a real subject
   again; the starter claim now refuses before any dialog, so it is no longer one of these).
   Since ADR-016, unlocking also signs in to QOR ID with no dialog, and a
   key with no QOR ID goes straight to claiming a name. Exercise both, and a key sign-in after QOR ID was
   unreachable at unlock, which still asks.
5. **The key scheme and the address format landed on 2026-09-19** (L3.2, half of M3.4; ADR-039).
   - The launcher derives Sr25519 keys with `sp-core` at the chain's pinned version: the first account is
     the bare phrase, further accounts are `//0`, `//1`, … as Talisman enumerates them. Sealing, custody
     and the idle lock are untouched.
   - Addresses are SS58 at prefix 42 everywhere a person sees one; the raw account ID appears only in the
     vault's Derivation panel.
   - QOR ID verifies Sr25519, takes `address` (SS58) or `account_id` (hex), stores 32 bytes, and gives a
     password-only account no chain identity until it proves a key.
   - **What a person should try in a running launcher**, since none of it has been: create a vault, add a
     second account, claim a QOR ID, sign in with the key, and link the key to a password account. Then
     import the same phrase into Talisman or Polkadot.js and check that the first two accounts are the
     same two addresses. That last check is the one thing tests cannot do, and it is what ADR-009 is for.
   - **What is deliberately not done:** the challenge still names no network (ADR-039 point 7, inventory
     F-Q9). The launcher's two state-changing chain calls refused until L3.1, which landed the next day
     (item 9 below).
6. *(Retired on 6 October 2026: work reaches `main` by pull request on `QOR-MATRIX/demiurge-chain`, §7.)*
7. **Owner: Resend on `demiurge.cloud`.** QOR ID sends verification and reset messages through Resend's API,
   tested end to end against a local stand-in.
   - `demiurge.cloud` is verified and authorised in Resend (owner, 2026-09-15). Checked in public DNS the same
     day:
     - DKIM (`resend._domainkey`) resolves and matches;
     - SPF (`send`, TXT and MX) resolves and matches;
     - DMARC (`_dmarc`) is published at `p=none`, monitoring only.
   - Demiurge mail is never sent from `vybz.cloud`: separate projects do not share a sending reputation.
   - The owner set the sending-only key on 2026-09-15, in their user environment. None was created from here.
     A key pasted into chat earlier was revoked. The variables:
     - `RESEND_API_KEY`: the key. Resend refuses it anything but sending;
     - `EMAIL_FROM`: `Demiurge-Cloud <noreply@demiurge.cloud>`;
     - `BASE_URL`: QOR ID's own origin. **Since 1 October 2026 QOR ID is at `https://id.qorsync.dev`** on
       Railway, which holds its settings now; the `demiurge.cloud` subdomain planned here was not used.
   - `RESEND_API_URL` stays unset in any live environment.
   - **QOR ID serves the pages its links open on its own subdomain** (owner, 2026-09-15; recorded as a
     clarification of ADR-015).
     - Vercel serves marketing and the web surfaces only, and does not proxy or rewrite QOR ID's paths.
     - Routing paths through Vercel to QOR ID's host would put a proxy and a rewrite layer between people
       and one-use tokens. (The Fly.io plan of the time is superseded: QOR ID runs on Railway, ADR-063.)
     - `BASE_URL` has no default. Email stays unconfigured unless it is an origin: HTTPS, or HTTP to this
       machine, with no path.
   - **Resend webhooks, set up by the owner.** QOR ID has had its public HTTPS address since 1 October 2026:
     1. In Resend, open Webhooks and add an endpoint: `https://id.qorsync.dev/api/v1/webhooks/resend`.
     2. Select the events `email.bounced`, `email.complained` and `email.suppressed`. Others are recorded and
        ignored, so they are harmless but unnecessary.
     3. Copy the endpoint's signing secret (`whsec_…`) into Railway's settings for `qor-auth` as
        `RESEND_WEBHOOK_SECRET`, never in chat.

     Until then every delivery is refused with 503, and the service logs a warning at start. Whether the
     secret is set today is unconfirmed here; it is listed in `OWNER.md` as still to do.
   - **Live run, 2026-09-15, to Resend's test address `delivered@resend.dev`:**
     - registration sent a verification message, which Resend marked delivered, and its link verified the
       address (a second use was refused);
     - `forgot-password` sent a reset link, and it reset the password (a second use was refused). Resend
       marked it sent for at least two minutes, and delivered when checked again later. It was delivery lag,
       not a failure. Both messages go through the same code path, sender and key. Resend's per-message
       status carries no event times, and no webhook is configured, so the exact lag is not known;
     - the new password signs in and the old one is refused;
     - no link and no key reached the service log.
   - **Still to do:** the same run with a real inbox the owner names, now through the link pages. Receiving
     mail at `demiurge.cloud` is not set up, and is not to be as part of a test (§5). A live bounce report
     cannot be part of it until the webhook is configured.
   - **Templates rewritten; copy approved by the owner as drafted** (2026-09-15).
     - All four messages share one layout in the Architect palette: tables and inline styles, no images,
       declared `color-scheme: dark`.
     - Each message has a plain-text part.
     - Each link is shown in full as well as behind the button.

   The lockout for accounts with an unverified email is closed. An account registered while email is
   configured is sent its link. Any account whose link lapsed or never arrived requests a new one through
   `resend-verification`. Neither needed loosening the reset rules.
8. **Recovery for an account without an email address: decided and built** (owner, 2026-09-15).
   - Such an account adds a verified address through `POST /api/v1/profile/email`, with its password.
   - The address counts for recovery only once confirmed, and the account keeps its backup codes.
   - Changing an address follows the same rules and tells the old address.
   - No existing rule was loosened.
   - The accounts that still cannot be recovered are named in `SECURITY.md` ("Accepted lockouts in QOR
     ID"):
     - no verified email, a forgotten password and no code;
     - a lost inbox with a forgotten password;
     - a lost key on a keypair account.

9. **L3.1 landed on 2026-09-20 (ADR-040), and with it M3.4.** This is the first time anything outside
   `chain/` has used the Substrate chain.
   - The client is `subxt` 0.51, chosen under ADR-033 rule 2 and verified before it was decided: it
     resolves alongside the pinned `sp-core =43.0.0` with a single copy of every shared crate, including
     the same `frame-metadata 23.0.1` the chain builds against, and it depends on no SDK crate, so it
     cannot drag the pin forward.
   - Every call is addressed against the metadata the connected node serves. No generated interface is
     checked in and no metadata file is committed, so a runtime upgrade does not need a launcher release.
   - The launcher declares its own `subxt::Config`. On the day, it had to: the runtime still used
     `IdentityLookup`, so an extrinsic's address was a bare `AccountId32` rather than the `MultiAddress`
     every relay chain uses. That became inventory question **F-Q10**, written up for the owner as
     **Q-17** in [`docs/architecture/ADDRESS_TYPE.md`](docs/architecture/ADDRESS_TYPE.md), and **it was
     decided and carried out later the same day** (ADR-041, item 10 below). **So the reason in this
     sentence no longer holds:** every value in that `Config` is now the SDK's own default, and it stays
     as the one place a future divergence would be declared.
   - `subxt`'s `Signer` is deliberately not implemented: it is synchronous and infallible, so a locked
     vault or a declined prompt could not have said so. The client reads the bytes to be signed and hands
     those to the vault, which draws the host dialog and signs inside its own lock.
   - **Proven against `chain/target/release/demiurge-node --dev`:** a fresh vault funded from Alice, an
     approved transfer signed and finalised, a declined transfer that moves nothing and leaves the nonce
     where it was, exact balance arithmetic on both sides (there are no fees yet, OPEN-4), and two
     transfers back to back. It is `cargo test --lib chain::live -- --ignored`, ignored without a node.
   - **What is deliberately still refused:** transaction history, which waits on ADR-028's indexer because
     a Substrate node serves none, and the starter claim, which waits on the economics because the chain
     has no issuance and OPEN-1 and OPEN-2 are undecided. Both say exactly that when called.
   - **`cargo audit` on the launcher: no vulnerability**, and 13 warnings, of which `subxt` brings two.
     `smallstr` (RUSTSEC-2026-0215, unmaintained) arrives through `scale-info-legacy` and `frame-decode`
     and is compiled. `lru` (RUSTSEC-2026-0253, unsound) arrives through `smoldot-light` under
     `subxt-lightclient`, which is an optional feature the launcher does not enable, so it is in the
     lockfile and not in the compiled graph. Neither fails the audit, and **nothing was exempted to make
     that so**: the launcher still has no `audit.toml`.
   - **The live test is not in CI, and is not meant to be yet.** It is ignored without a node, so
     `cargo test` — which is what the `launcher` suite in `GATES.toml` runs — passes with or without one.
     Putting it in CI means building and running a chain node in the job, which is a tightening worth
     making once the node's build cost in CI is known.
   - **What still needs a person:** the native dialog itself. The live test answers the prompt in code.
10. **Q-17, the runtime's address type: decided and done** (2026-09-20). Written up in
    [`docs/architecture/ADDRESS_TYPE.md`](docs/architecture/ADDRESS_TYPE.md), decided by the owner the same
    day, recorded in [ADR-041](docs/decisions/ADR-041-multiaddress-and-accountidlookup.md).
    - **The runtime uses `AccountIdLookup<AccountId, ()>` and an address is a `MultiAddress`**, which is the
      SDK's own `SolochainDefaultConfig` value and every relay chain's. The launcher moved in the same
      commit.
    - **What decided it was a correction, not the original finding.** F-Q10 said `IdentityLookup` was the
      minimal template's default. It is not: both SDK templates derive `SolochainDefaultConfig`, which sets
      `AccountIdLookup` (`substrate/frame/system/src/lib.rs:399`); the prelude that sets `IdentityLookup` is
      `TestDefaultConfig`, whose `AccountId` is `u64`. So the runtime line overrode the SDK's default with no
      reason on record, which is what AGENTS.md §7 asks to be justified.
    - **The write-up's cost estimate was wrong in one row, and says so.** It said `acceptance.rs` needed no
      change. Eight call sites needed it: a balances call's `dest` **argument** is `AccountIdLookupOf<T>`,
      which changes with `Lookup` whether or not an extrinsic address is involved. Corrected in place, struck
      through rather than quietly fixed, because the owner decided on that table.
    - **Evidence:** 51 chain tests with the wasm built, 13 two-validator checks, and the launcher's live test
      against a node built from this runtime. The old shape is refused by the new node, observed rather than
      assumed — the live test's funding helper still passed a bare account on the first run and failed with
      `CannotEncodeCallData(… WrongShape)`.
11. **The landing page: deferred by the owner** (2026-09-17). Do not start it, and do not extend frozen scope.
   - The roadmap has no landing-page item, and there is no gate for one. The nearest item is **M5.4, the public
     viewer**, the shareable page per published work, which depends on M5 and on the provenance source chosen in
     M2. A marketing landing page is not that deliverable.
   - `apps/marketing-site` exists but is **frozen scope** (D-011, amended by ADR-011), so building there would extend
     frozen scope, which is the one thing that must not happen by accident.
   - **If it is wanted it gets a roadmap item and its own decision first.** Until then this entry is the whole of its
     status, so it is deferred rather than undefined.
12. **The six products, 2026-09-21.** The owner named QOR Engine, Qontrol, GNOSIS and QFX, expanded Market
    and Library, and gave music streaming the placeholder name Stream. What exists now:
    - **Six substrate decisions, all Proposed on the day** (ADR-047 and ADR-051 accepted on 22 September, item 13): ADR-046 (the launcher hosts products as separate
      processes), ADR-047 (the object model and the DRC-369 wire format, which would carry out M2.3),
      ADR-048 (interchange formats), ADR-049 (a creator's data under their QOR ID directory), ADR-050
      (where products live, and a gate each) and ADR-051 (QFX's rendering layer).
    - **Seven blueprints** in `docs/blueprints/`: `ECOSYSTEM.md` first, then one per product.
    - **A product track each in `DIRECTION.md` §7 (P1 to P6, 34 items) and a gate each in `GATES.toml`**,
      appended after Public Release. Alpha, Beta and Public Release are unchanged. The launcher's gate
      parser reads `P` headings since `d4be885`.
    - **Two first slices, both inside the launcher, both the owner's scope:** Qontrol's Projects surface
      (P1.1, `5a44207`) and QFX layer one's backdrop (P2.1, `75b2ead`). **P1.1 is not ticked:** the owner's
      scope asked for changes with diffs, and the view and the host's `qontrol_diff` show only which files
      changed. A line-by-line diff that honours `core.autocrlf` and `.gitattributes` is the next Qontrol
      work, ahead of the rest of P1.2.
    - **The owner decided the two foundations** before the build: QOR Engine is a custom Godot build tracking
      upstream, never a hard fork ("QOR Engine, built on Godot"); Qontrol is gitoxide for every read and
      libgit2 for staging only, in a sidecar process, never linked into the host that holds the vault.

    **Found and fixed when the work was finished, after an API failure had stopped the session:**
    - **Ambience Off left the backdrop on screen with the scrim removed**, so text sat outside the contrast
      guarantee, and Off could not be turned back on without a restart. Fixed in `6e5bb28`.
    - **Six of the seven QFX accessibility cases could not fail**: they wrote the expected answer and read it
      back. Replaced by 21 that drive the app and count frame callbacks, proven to fail first.
    - **Qontrol's three proven tests would never have run in CI**: nothing built the helper, and a test
      without it returned early and reported ok. `qontrol-no-skips` makes that a failure, and CI builds,
      lints, tests and audits the helper (`7a3c8ea`). CI also runs `npm run check` now, which it had not,
      so the Projects and contrast checks were never in the workflow.
    - **The drafts described the tree before Phases 4 and 5 landed**, and were corrected before their first
      commit (`df3ccce`, `e998faf`).

    **What needs a person:**
    - **Open the two screens.** They sit behind sign-in, so run the stack as `scripts/run-local-stack.md`
      §4 and §5 describe, and build the helper first (the same file says how). Then **Projects** on the
      rail: New (Code, Music or Game, then "Choose a folder and create") or "Open a folder"; type a message;
      Commit. And **Settings → Accessibility → Ambience**: Live, Still or Off. The backdrop is visible from
      the Gate onwards.
    - **Answer ADR-047 yes or no**, after M2.1.
    - **Confirm or reject the check-design narrowing** (§4.0).
    - **Known divergences from the Proposed records**, not fixed because the records are not accepted:
      the canvas mounts behind the Gate (ADR-051 says never), the budget is one step rather than tiers, and
      there is no pause control; the helper is not bundled, so an installed launcher could not commit
      (P1.2). *(The first three were resolved by the owner on 22 September by amending ADR-051: item 13.)*
13. **The owner's acceptances and decisions, 22 September 2026.**
    - **ADR-047 accepted**, with its three owner questions answered as recommended: an asset is revisable
      until a one-way switch makes it permanent; one singles collection per creator; bytes in a store
      labelled temporary until the Mesh. **M2.3 ticked.** Deciding is not freezing:
      `beta.wire-format-frozen` stays unmet, and GNOSIS's requirements check flags `MaxRoyaltyRecipients`
      (8) as possibly too small before the freeze.
    - **The check-design narrowing approved, retroactively**, with the rule: the design system governs the
      default theme and the chrome; QFX governs the canvas; the default must still pass `check-design.mjs`
      unchanged.
    - **ADR-051 amended and accepted:** only the built-in default ambience may run behind the Gate, never
      an installed theme; the scrim is the guarantee, with the values recorded; the one-step budget is
      accepted for v0 and the tiers are P2.2; Still is the pause. `DESIGN_SYSTEM.md` §1, §6 and §7 and
      `DIRECTION.md` §5 are amended with it.
    - **`--ink-faint` fixed** in all five themes (`320be99`), and the scrim raised from 0.86 to 0.90 so faint
      text also holds 4.5:1 over any backdrop. `check-contrast.mjs` now applies every theme through the app:
      54 checks, proven to fail first against the old tokens and against the old scrim.
    - **Every open question in the six blueprints answered**, each recorded in its blueprint as the owner's
      and reversible. QOR Engine's promise that projects open in stock Godot is P3.6 and a criterion of
      its gate; Market's gate counts U-7, deferred to M6; P6.1 is deferred to M6.
    - **P1.1 stays unticked until in-file diffs exist**, and the diff is the next Qontrol item after M4.1.
    - **Next:** the owner says "acknowledged" to the ten-line summary of the inventory's DRC-369 section,
      which ticks M2.1. Then M4.1, and the first asset minted from a Qontrol project, shown in the launcher.
      No M4 code before that. **Both happened the same day: item 14.**
14. **M4.1: the asset primitive, and the first mint, 22 September 2026.**
    - **M2.1 ticked** against the owner's acknowledgement of the ten-line summary, kept verbatim in the
      inventory. **The eight-recipient royalty bound carried forward** by the owner: ADR-047 open item 1,
      inventory Q-18, and `beta.royalty-recipients`, so the format cannot be frozen with it unexamined.
    - **The chain** (ADR-052, accepted under the delegation): `pallet-nfts` mounted with every value's
      source recorded (ADR-047 for identifiers and strings; Asset Hub Westend at the pinned tag for the other
      bounds, fetched and read), and `pallet-drc369` with only M4.1: the 41-byte content reference, the pinned
      commit, revise, one-way make-permanent, a mint signed to the signer's own singles collection, events
      carrying the reference, and owner enumeration from storage and a runtime API. **A call filter makes it
      the only way an asset is created.** Deposits are placeholders derived from the existential deposit
      (U-14, opened; `public-release.economics-decided` counts it). Weights are placeholders, M7.2 debt.
    - **`chain/scripts/dev-fund.mjs`** sends CGT from Alice on a `--dev` node only, creates nothing, and
      refuses any other chain type.
    - **The launcher:** the object model (manifest, BLAKE3 root, 41-byte reference, temporary content store
      in the launcher's data directory), Projects' Mint panel, and an Inventory read from the chain with Make
      permanent. A declined mint moves, sends and writes nothing.
    - **Found and fixed on the way:** `alpha.chain-tests` could never be met, because four tests kept names
      from the old ticker (`5a0378f`); the Projects surface's controls named classes the stylesheet does not
      define and rendered as browser defaults (`3f67870`).
    - **Found and recorded, not decided:** ADR-047 says an asset's identity is "a function of bytes and
      nothing else" while its decisions 5 and 8 put the commit and branch inside the manifest. The launcher
      follows the decisions; **Q-19** and `beta.manifest-identity` hold it open for a later ADR.
    - **Found and recorded, not fixed:** one QFX case in `check-accessibility.mjs` is intermittent on this
      machine and predates this session's changes (§5). *Since fixed, by item 15.*
15. **Readability, 22 September 2026, from the owner's smoke test.**
    - **The launcher was unreadable with the backdrop live.** The canvas and the scrim were rendered inside
      `#qor-root` beside the interface, and the rule meant to lift the interface above them lifted all three
      together, so the scrim was painted over everything but Nexus. Fixed with `.qfx-interface`
      (`1f5f4ed`).
    - **`check-readability.mjs`** measures every run of text as painted on every screen in every theme, with a
      hostile backdrop and with none; it failed first on every theme, and fails 15 of 15 when the bug is put
      back. It joined `npm run check`.
    - **Three colours raised** by the few lightness points AA needed over a bright backdrop: `--color-bad`,
      and the sanguine and numen accents (`67a083d`).
    - **Also that day, from the same smoke test:** a new vault showed QOR ID's raw refusal instead of asking
      for a name, because the launcher matched two refusals by wording the service had changed (`26f2521`).

16. **Assets as cards, and trading them, 22 September 2026, both asked for by the owner.**
    - **The card shell** (`5d8ad74`): every asset the Inventory reads is a card that leans towards the
      pointer, opens at size on its title, and carries a mark drawn from its own fingerprint until it has a
      preview. In `src/qfx/`, under the carve-out **the owner widened that day** from the backdrop to a
      QFX-owned surface — the same directory, the same two rules, logged in `GATES.toml` with its approval.
      **No sheen**: a sheen is a gradient, gradients are forbidden in every directory including that one, and
      the way to have one is for the single canvas to show *through* the card (ADR-051), which is the next
      slice.
    - **The owner's asks written into the roadmap** (`ebb93f3`), with one correction: the chain stores a
      fingerprint, a commit, a name and an ownership record — **not the files, and not the manifest** — so it
      proves which bytes an asset is and cannot produce them until the Mesh (M8). Said in `PROTOCOL.md` §12.
      An asset may be any bytes, and the media-type table is widened at L4.1. Trade became L4.4/L4.5, Sell
      L4.6, the atomic batch M4.6, and the owner's proposed 15% platform share and USD rail **U-15, open**.
    - **M4.6 and ADR-053**: `pallet-utility` mounted, **only `batch_all` reachable**, `spec_version` 3. Four
      runtime tests, each shown to fail first. A batch cannot smuggle a filtered call: for a signed origin
      the inner calls go through this runtime's own base filter, and only root bypasses it (ADR-037).
    - **L4.4 and L4.5**: an asset's menu on right click *and* from the keyboard, Sell offered and refused
      with its reason, and a trade of up to sixteen assets with an optional public message, a warning screen,
      and then the host's own dialog. 2 host tests and 24 view checks, each proven to fail first; a live
      trade against a development node moved two of three minted assets and the chain was asked afterwards.
    - **The development chain was upgraded in place** with `sudo.setCode` rather than purged, so the owner's
      minted asset survived: `spec_version` 2 to 3, verified by reading it back. The first runtime upgrade
      performed in this repository.
    - **The media-type table was widened** — video, archives, documents, models and fonts are named, a
      project's own formats are still `application/octet-stream` — because the owner asked for "an asset can
      be anything" to be clear now. **Doing it found a wire-format question**: a file's media type is inside
      the hashed manifest, so the table is part of an asset's identity, two clients with different tables
      compute two roots for the same bytes, and the format names no table. **Q-20**, ADR-047 open item 3,
      counted by `beta.media-types`. Nothing minted before is affected — the chain never recomputes a root —
      but re-minting the same project now produces a different one.

17. **The trade window, Sell, and three faults found by running it, 23 September 2026.**
    - **The work was already written when this session started, and none of it had been run.** It was in the
      working tree uncommitted. Running it is what this session did, and all three faults below were invisible
      to reading.
    - **L4.5 reworked at the owner's request**: the trade window is two sides with a lane between them — what
      leaves this account, where it goes — and a mark that crosses the lane **once per change**, never a loop,
      and not drawn at all under reduced motion. A loop would fail `check-design.mjs`, which forbids an endless
      decorative animation in every directory; `src/qfx/`'s carve-out is two rules wide and this is not one of
      them.
    - **"Recently traded with"** (`src-tauri/src/partners.rs`): the accounts this machine has sent assets to,
      written only after the chain finalised a trade, capped at twelve, **local and sent nowhere**. A file that
      cannot be read is an empty list rather than an error, because a convenience must never break a trade;
      the failure to write one is a `warn!` that carries the error and not the address. Reaching someone by
      QOR ID needs a directory lookup that does not exist (L4.7), so `label` is carried and always empty today.
    - **Half of L4.6, and the item stays unticked** (`src-tauri/src/listings.rs`, `src/views/SellDialog.tsx`).
      Sell drafts a listing — title, kind, that kind's questions, price, description — into a file in the
      launcher's data directory. **Nothing is published**, and the form says so in the product twice: a banner
      before anything is typed, and a line after saving. L4.6 is "a listing other people can see", which still
      needs M4.2, P5.5 and M5.4, so **ticking it would be a lie**. The price goes through `cgt::parse_cgt`:
      exact, excess precision refused rather than rounded, no float (AGENTS.md §5). **No economic value is
      invented** — nothing takes a share, because U-15 is undecided.
    - **The vocabulary is a proposal, and the owner's to confirm** (AGENTS.md §8). Six categories, of which the
      owner named three and said "etc.". It is deliberately not Market's payload-kind axis: a payload kind says
      what a buyer's machine does with the bytes, a category says what the thing is. **Physical (offline)
      carries a warning the others do not**, shown before it is chosen, because the chain moves the record and
      not the object and nothing here holds money, confirms delivery or settles a dispute.
    - **Fault 1: `check-inventory-view.mjs` could not be parsed**, so it had never run. It declared `sent`
      twice and Node refused the file. The `GATES.toml` entry written beside it claimed the run was 110 checks;
      no run had produced that number. Logged as a correction in that file rather than edited away.
    - **Fault 2: the Sell checks passed by doing nothing.** They open an asset's menu after the trade block,
      and used the asset the trade had just sent away — so the card was gone, `?.click()` was a no-op, and no
      dialog opened. Six checks then read an absent dialog. **The `?.` is what made it silent**; those clicks
      no longer use it, and the block uses the asset still held. The real run is **126 checks**.
    - **Fault 3, in the host: three `sort_by` calls failed `clippy -D warnings`.** Fixed to `sort_by_key` with
      `Reverse`, which is the same stable order.
    - **Proven to fail first**, three planted faults, one failure each and the right one each time: a banner
      reading "Your listing is live"; a category appended in the view instead of taken from the host; and a
      form that answered a question the person had not, by filling every field with its first option — that
      last one is the failure the check exists for, and it caught a listing silently claiming "Players: One".
    - **Two roadmap items no gate counted**, found by running §5's gate-coverage check rather than by reading:
      **L4.6** and **P2.8**, both appended to their tracks on 22 September after the gate lists that would have
      counted them. Added to `beta.launcher` and `qfx.roadmap`, logged as a tightening. **This is the third
      time**; the check exists because of the first two. Only `L2.1` is uncounted now, which is deliberate.
    - **Evidence:** 125 host tests, `cargo fmt --check` and `cargo clippy --all-targets -D warnings` clean,
      `npm run build` (including `tsc --noEmit`) clean, and `npm run check` 8 / 41 / 34 / 43 / 126 / 54 / 181,
      exit 0, read from the file rather than through a pipe.
    - **Not done, and needing a person or the owner:** nobody has opened the Sell form in a running launcher —
      every check above drives the built page in a headless browser against a fixture host, so the Tauri
      commands `listing_save`, `listing_drafts`, `listing_discard`, `listing_vocabulary` and `trade_partners`
      have unit tests but **have never been called from the real interface**. The trade lane and the partner
      list have not been seen by a person either.

18. **Qontrol's diffs, and P1.1 ticked, 23 September 2026.** The owner's sequencing put the diff next after
    M4.1; it is built, in `src-tauri/src/qontrol/diff.rs`, with the view in `Projects.tsx`.
    - **What it compares:** the index's blob against the working-tree file **converted to what git would
      store**, through gitoxide's filter pipeline in `Mode::ToGit`, under the repository's own
      `core.autocrlf` and `.gitattributes`. That is `git diff`'s comparison. A raw byte comparison would show
      every line of every file changed on a Windows machine that converts line endings, which is why the
      first slice shipped without a diff rather than with that one.
    - **What it returns:** a structured diff per file — lines with their kind and the file's own line
      numbers, or binary with both sizes, or "same", "empty", or not a file (a folder of new files, a
      conflict, a submodule) — not a string the view would parse back apart. A moved file names where it
      came from. Past 4,000 lines it stops and says so.
    - **It never runs a program.** A diff driver's text conversion and an external diff command both run
      whatever configuration names, and Qontrol opens folders from strangers, so neither is applied.
    - **Evidence:** 10 new host tests, 24 for Qontrol and 135 in all, passing under `qontrol-no-skips` with
      the helper built. **Five failed against a planted raw-byte diff** — the autocrlf test (which also asks
      `git diff --numstat` for a second opinion), the `text` attribute test with autocrlf explicitly off, the
      `-diff` test, a NUL byte, and an empty new file — and passed restored. Three of them are now required
      by `qontrol.tests` in `GATES.toml`, logged as a tightening. `check-projects-view.mjs` went from 43 to 67
      checks and **failed against three planted view faults**, one each: the late-answer guard removed (the
      first file's lines drawn under the second file's name), a line drawn with the wrong text, and the diff
      kept after a commit. `check-readability.mjs` measures an open diff in every theme and backdrop, 181 to
      191. `cargo fmt --check` and `clippy --all-targets -D warnings` clean; `npm run check` exit 0.
    - **Found on the way:** `check-readability.mjs` failed its first run on the diff, on a screen-reader-only
      "added: " label — invisible text it measured as painted. The check was not changed (that would be a
      loosening, and the owner's); the rows use `<ins>` and `<del>` instead, which say the same thing to
      assistive technology with no hidden text. And the new Projects checks first failed to parse, on a
      duplicate `const` — the same fault that hid the Inventory check for a day (item 17). Caught by running
      it, not by reading.
    - **Not done:** nobody has opened a diff in a running launcher; every check above drives the built page
      against a fixture host, and the host tests drive real repositories. *(The helper was bundled on
      2026-09-26, item 19.)*

19. **Qontrol's git layer, and P1.2 ticked, 26 September 2026.**
    - **Branch** is made at HEAD by gitoxide and not switched to. **Switch** and **discard** are libgit2's
      safe checkout in the `qontrol-git` helper, because gitoxide 0.87 can only check out a whole index into
      a folder. A switch carries uncommitted changes across and refuses, touching nothing, when one would be
      overwritten. A discard puts back a modified or removed file and **refuses a new one**, because that
      would be deleting the only copy.
    - **Per-file commit** stages only the ticked changes, matched by exact name: libgit2's pathspec treats
      `take[1].wav` as a glob even under `DISABLE_PATHSPEC_MATCH`, so the helper decides itself.
    - **The guard** (`src-tauri/src/qontrol/guard.rs`) reads what the helper would stage, under the same
      ignore rules as the commit, and holds back any file over 50 MiB, or named or shaped like a credential,
      until the person says yes to it by name. Its warnings never carry the text that matched, and the host
      runs it again at commit rather than trusting the view.
    - **Bundling:** `npm run app:build` runs `scripts/build-helper.mjs`, which builds the helper, places it
      where Tauri takes a sidecar from, and copies libgit2's licence from the exact vendored source; then it
      applies `src-tauri/tauri.bundle.conf.json`, which declares the sidecar. It is declared there and not in
      `tauri.conf.json` because tauri-build refuses to compile when a declared sidecar is missing, which would
      break `cargo test` on a clean checkout. A Windows installer was built:
      `src-tauri/target/release/bundle/nsis/QOR Launcher_0.1.0_x64-setup.exe`.
    - **Evidence, re-run on 2026-09-26 in a fresh session:** 150 host tests pass under `qontrol-no-skips`
      with the helper built (39 of them Qontrol's), three live ones ignored without a node; the helper's 16
      pass; `cargo fmt --check` and `clippy --all-targets -D warnings` clean. The session that built it
      recorded each new refusal failing first against a planted fault where a single fault can reach it
      (`GATES.toml`, change log of 2026-09-26), and three new tests are required by `qontrol.tests`, logged
      as a tightening. `check-projects-view.mjs` 104, `check-readability.mjs` 231.
    - **Two intermittent view-check failures, found while re-running the evidence (2026-09-26), neither in
      P1.2's code:**
      - **Readability** failed one measurement in two of its first three runs. The second named it: the
        trade warning's "Send them for good" at 1.22:1 in `rgb(6, 7, 10)`, the primary button's text colour,
        which Send never has when settled. React reconciled the compose step into the warning and reused
        Review's button as Send, so Send cross-faded out of the primary look for 200ms, and the check's 250ms
        wait sometimes caught it. **Fixed in the product, not the check:** `TradeDialog.tsx` keys its two
        steps so each mounts fresh. `check-inventory-view.mjs` gained a timing-free check (126 to 127) that
        failed with the keys removed and passed restored; logged in `GATES.toml` as a tightening. Readability
        then passed 231 of 231. The first failure's line was not kept, so it is assumed, not shown, to be the
        same one.
      - **Accessibility** failed "stored off, then live: the backdrop starts" once, counting 3 frames in a
        second where it needs 20, then passed 41 of 41 on the next run. It looks like the headless browser
        being starved of frames under load. **The threshold was not touched** (loosening is the owner's);
        if it recurs on a quiet machine, it is a real fault in turning the backdrop back on.
    - **Not done:** nobody has installed that installer and committed from it; the macOS and Linux bundles
      have not been built; every view check drives the built page against a fixture host. **Next in P1:**
      P1.3, scaffolds completed (Unity and Unreal variants, `qontrol.toml` and the lock set).

20. **Restoring a vault from its recovery phrase, 26 September 2026, asked for by the owner** after losing
    the passphrase to the vault on this machine.
    - **Before this, a lost passphrase had no way back inside the launcher.** The host's `vault_create` took
      any phrase, but the Gate only ever offered a freshly generated one, and `create` refuses when a vault
      exists. The only route was renaming `vault.qor` by hand.
    - **Now:** the Gate's unlock and welcome screens offer *Restore from your recovery phrase*. The host first
      returns the address the typed phrase opens (`vault_preview_phrase`, which stores nothing), so the person
      recognises the account before anything changes. `Vault::restore` then refuses a bad phrase, a weak
      passphrase or an open vault **before** any dialog; asks in a host dialog, naming the set-aside file and
      the account; renames the old file to `vault.qor.set-aside-<date>` (then `-2`, `-3`, …), never deleting
      it; seals the new vault; and puts the old file back if sealing fails. A new error kind, `vault_open`.
    - **Evidence:** five new host tests, 155 in all, fmt and clippy clean. Two were proven to fail first: moving
      the file before the dialog failed the decline test and the refuse-before-asking test, and destroying the
      old file instead of renaming it failed the set-aside test (which checks it is byte-identical and still
      opens with its old passphrase). The view checks all pass unchanged — design 8, accessibility 41, gates
      34, Projects 104, Inventory 127, contrast 54, readability 231. **None of them opened the two new screens;
      since 2026-09-28 `check-vault-gate.mjs` does (item 22)**, but the host dialog has not been seen in a
      running launcher. The putting-back on a failed
      seal is not tested: nothing in a test can make the write fail after the rename.
    - **Why it grants nothing new:** anyone at the machine could already rename the file; nothing in the old
      vault becomes readable, and it stays on disk.
    - **For the owner's own vault:** the file from 22 September already set aside by hand,
      `vault.qor.set-aside-2026-09-22`, is left where it is; a restore today names its file
      `vault.qor.set-aside-2026-09-26`.

21. **Windows Hello unlock, 26 September 2026, the owner's decision (ADR-054).**
    - **Asked for** as "a keyfile stored in the client's directory", because two passwords to log in is
      ridiculous. The second password was QOR ID's fallback sign-in, shown because QOR ID was not running
      (below); ADR-016 already makes the passphrase sign in. Three options were put to the owner with their
      risks — Windows Hello, DPAPI auto-unlock, a plain key file — and **the owner chose Windows Hello**.
    - **Built:** `vault/hello.rs` (the `Presence` interface, its Windows implementation on `windows` 0.62 and a
      test stand-in); `Vault::enable_hello`, `unlock_with_hello`, `disable_hello`, `hello_enabled`; four
      commands; a Settings panel; and the Gate's *Unlock with Windows Hello*, asked once on arrival with the
      passphrase underneath. The copy is bound to `vault.qor` by BLAKE3 and moves aside with it on a restore.
    - **Evidence:** seven new vault tests, 162 host tests in all; fmt and clippy clean. With the binding check
      removed, `a_hello_copy_never_opens_a_replaced_vault` failed; restored, it passes. The one ignored live test
      asked the real Windows on this machine and got `available: true`.
    - **`cargo audit` on the launcher: no vulnerability, 14 warnings**, none from `windows`. The one more than the
      13 recorded on 2026-09-20 is `faster-hex` (RUSTSEC-2026-0306, unsound), which arrives through gitoxide
      (`gix-hash`), not through this change: an advisory published since, against a dependency Qontrol already
      had. Nothing was exempted; whether a newer `gix` clears it is for the next Qontrol session.
    - **Not done:** nobody has turned Hello on or unlocked with it in a running launcher, so the Windows
      implementation's prompts, the determinism it relies on, and whether the prompt opens behind the window
      are unobserved. The helper that would raise the prompt needs `unsafe`, which the launcher forbids; it was
      removed rather than the rule loosened. ~~No view check opens the Hello button or panel~~ — **closed
      2026-09-28 by item 22.**
    - **Also found:** the restore dialog's text (item 20) had runs of spaces where line continuations were lost;
      fixed. Its test only checked that the address appeared.
    - **QOR ID is not running on this machine**, which is why sign-in failed after the restore. Docker Desktop was
      started this session and `qor-smoke-redis` is up; `qor-smoke-pg`, which most likely holds the owner's
      account, cannot bind 55432 because Windows now reserves 55376–55475. Starting a second container on its
      volume was refused by the session's permission check and is waiting on the owner (the reply that turn
      has both routes).

22. **The Gate's restore screens and Windows Hello, checked in a browser, 28 September 2026.** Items 20 and
    21 each recorded that no view check opened their screens.
    - **`scripts/check-vault-gate.mjs`, 64 checks**, joins `npm run check` (so CI's launcher job, once CI runs).
      A stand-in host records every command. It asserts that the typed phrase reaches the host normalised; the
      account it opens is shown before anything is restored, and editing the phrase withdraws it; restore is
      asked once, only after *That is my account*, with that phrase and the passphrase typed; replacing a vault
      says it is set aside, not deleted; with Hello on, Hello is asked once on arrival, a cancel is not re-asked,
      the passphrase stays underneath; no Hello button when Hello is off or the host cannot say; Settings turns
      Hello on only with the passphrase typed, and redraws from the host.
    - **Proven to fail first** against five faults planted in the view one at a time (listed in the script's
      header and in `GATES.toml`'s change log of 2026-09-28, logged as a tightening). Each was caught by its
      own checks; the view was restored byte-identical from a backup afterwards and the check passed again.
    - **`check-readability.mjs` 231 → 251**: the unlock screen with Hello on and just cancelled, and a checked
      recovery phrase, in every theme and backdrop. Its fixture host now answers `vault_hello_status`, so the
      Settings surface it measures includes the Hello panel. All AA.
    - **Its first runs were flaky, and the cause was the check, not the app** (§5): a headless page that draws
      no frames left "Opening the gate…" on screen, and a step could act before the next screen was drawn.
      Fixed by keeping the page in front and waiting for each target; the failing walk now reports where it
      stopped. No assertion was loosened.
    - **Still not done:** Hello and the restore dialog have not been used in a running launcher (items 20, 21).

23. **The vault opens with Windows Hello alone, 28 September 2026, the owner's decision (ADR-055).** The owner:
    every visit ran into "my password or the vault key", and Hello "apparently isn't implemented". It was
    implemented (item 21) but opt-in, behind a Settings panel that itself asked for the passphrase, so every
    arrival was still the passphrase screen; and when QOR ID was unreachable the Gate offered *Use a password
    instead*. Three options were put — Hello only, a silent DPAPI unlock, both — and **the owner chose Hello only.**
    - **The host.** `vault.qor` is now sealed under the Hello key's signature and nothing else (`HelloVault`,
      `"unlock": "windows-hello"`). `Presence::ensure` makes the key only if the Windows account has none, so a
      set-aside vault still opens; ADR-054 replaced it each time. **Sealing signs twice and refuses if the
      signatures differ**, because as the only seal a non-deterministic Hello would make a vault that never opens
      again; ADR-054 assumed determinism and nobody had confirmed it on a device. `unlock`, `restore` and
      `export_phrase` take a `Presence`; `move_to_hello` moves a passphrase vault with its passphrase; a vault with
      an ADR-054 copy bound to it moves with Hello alone. `VaultStatus::Locked` carries `passphrase`. New error
      kind `passphrase_vault`. Commands: `vault_create(phrase)`, `vault_restore(phrase)`, `vault_unlock()`,
      `vault_move_to_hello(passphrase)`, `vault_export_phrase()`, `vault_hello_available()`; the four
      `vault_hello_*` commands are gone. The chain client's three live tests seal with the stand-in.
    - **The Gate.** No passphrase step: create is phrase, confirm, *Seal with Windows Hello* (a button, so the
      prompt follows a click). Unlock asks Hello once on arrival. A passphrase vault goes straight to *Move your
      vault to Windows Hello*. Sign-in is the vault key only; the password form is gone (`qor_login` stays in the
      host, unused). A device without Hello is told before any words are written down. Settings loses the Hello
      panel; *Reveal with Windows Hello* replaces the passphrase field. `declined` now reads "You cancelled, so
      nothing was signed, changed or opened."
    - **Evidence.** 165 host tests pass (vault tests rewritten: 26 in `vault::tests`, including a Hello that signs
      differently each time, a passphrase vault moved with and without its ADR-054 copy, and a copy of another
      file ignored); clippy clean. `check-vault-gate.mjs` rewritten, **84 checks**, proven to fail against four
      planted faults; `check-readability.mjs` 251, all AA; all eight view checks pass. `cargo test ... --ignored`
      confirms Windows reports Hello set up on this PC. Logged in `GATES.toml` as a **scope change**, not a
      tightening, because assertions for removed screens were deleted: **the owner is asked whether that is a
      loosening** (`OWNER.md`).
    - **Found by the check, fixed:** a passphrase vault was sent to the unlock screen, refused at once, and the
      swap to the move screen was lost mid cross-fade in 3 of 5 runs, leaving the person stuck. The locked
      status now names such a vault and the Gate goes straight to the move.
    - **Still not done:** nobody has sealed or opened a vault with the real Hello prompt, or moved the owner's
      own passphrase vault. The prompt may open behind the window (`unsafe` is forbidden, as under ADR-054).
      Other platforms can no longer create a vault.

24. **No lock screen, 28 September 2026, the owner's decision (ADR-056).** After item 23 the owner moved their
    vault to Hello and was stopped by the Gate's sign-in: the shell required a QOR ID session, and QOR ID was not
    running (its default endpoint, `demiurge.cloud`, is not deployed; Docker was not running). They demanded the
    vault and Hello be removed. The keys stay; every prompt and every wait went.
    - **Host.** `vault/keychain.rs`: `KeyStore`, the OS keychain via the existing `keyring` dependency, service
      `cloud.demiurge.qor-launcher`, entry `vault-key:<id>`; an in-memory one for tests; an ignored live test that
      passed against this PC's Credential Manager. `vault.qor` v3 (`"unlock": "os-keychain"`) seals the phrase
      under a random key held there. No idle lock, no `touch`. `unlock` opens a keychain vault with nothing asked,
      moves an ADR-055 Hello vault with one last gesture, and a passphrase vault via `move_to_keychain` (or its
      ADR-054 copy). `export_phrase` asks in a host dialog. `VaultStatus::Locked { sealed_with }`. Commands
      `vault_move_to_keychain`, `qor_sign_in`; `touch_vault`, `vault_hello_available`, `vault_move_to_hello` gone.
    - **Frontend.** The store opens a keychain vault before the first frame; `App` shows the shell once the vault
      is open, session or not. The Gate is first run (one button), restore, and the two one-time moves. Settings
      signs in to QOR ID with the vault key and claims a name. No Lock button, no countdown.
    - **Evidence.** 162 host tests pass; clippy clean; tsc clean; all eight view checks pass (Gate 65, readability
      251). `check-vault-gate.mjs` caught the keychain error reading "The system keychain refused the request."
      and the wording was fixed. **It has not been proven to fail against planted faults**: the first plant made
      every check wait out its timeout and the run was stopped to ship the installer. Logged in `GATES.toml` as a
      scope change; the owner is asked whether it is a loosening. Installer rebuilt 28 September 15:18.
    - **Owed:** the planted-fault proof; a person installing and opening it; the keychain message is shared with
      QOR ID's token errors, whose wording ("the vault's key") fits only the vault.

25. **Onboarding, the intro, and a local stack, 28 September 2026, the owner's asks.** Items 20-24 were committed
    first, on the owner's word (`a3109e2`, `928cab5`, not pushed).
    - **Local stack.** `tools/qor-launcher/scripts/start-local.ps1`: Postgres `qor-local-pg` (54329, volume
      `qor-local-pgdata`) and Redis `qor-local-redis` (56389) in Docker, QOR ID on 8080, `demiurge-node --dev --tmp`
      on 9944; secrets in `%LOCALAPPDATA%\qor-local`, never printed. Running as of this session. Captured output
      makes the caller wait on the servers' inherited pipe (§5).
    - **Launcher.** Agora renamed Social (id `social`; DIRECTION L7.3). `InfoTip` replaces view-header bodies,
      field hints and the accessibility and appearance explanations; kept outside headings so a heading's name
      stays its title (the Projects and Inventory checks caught it inside). `Onboarding.tsx`: the claim bubble and a
      card checking names as typed (350 ms debounce, latest answer only), claiming after `qor_sign_in` so no dialog
      appears. `src/qfx/ceremony/`: `Awaken.tsx` (neochrome halo), `Tutorial.tsx` (six chapters, claims inside
      CGT.md), `Intro.tsx` (2 s splash each open, first-run once per version from `src/assets/first-run/`, reduced
      motion skips both), `ceremony.css`.
    - **The owner's decision:** `check-design.mjs` exempts `src/qfx/ceremony/` from the glow, gradient and looping
      rules only (DESIGN_SYSTEM.md; GATES.toml evidence-rule entry). Proven not to leak.
    - **Evidence.** tsc clean; design 8, accessibility 41 (its known intermittent case failed once of three runs),
      gates 34, Projects 104, Inventory 127, Gate 65, contrast 54, readability 281 (onboarding and tutorial measured
      in every theme). Host unchanged (162).
    - **The owner tested it in the installed launcher and accepted it, 28 September 2026.** Owed: the owner's
      first-run file; not committed.

26. **The owner's rulings of 28 September 2026.** VYB merged into Social and its name dropped (`systems.ts`, SYSTEMS,
    ECOSYSTEM and stream blueprints). ADR-046, ADR-048, ADR-049 and ADR-050 accepted. Q-18 decided, eight royalty
    recipients (ADR-057; `beta.royalty-recipients` now reads met). The two check-scope changes ruled not loosenings
    (GATES.toml). **CI must leave GitHub Actions**, which tried to charge; the platform is not yet chosen, and nothing
    in `.github/workflows` was changed. The readability check's tutorial case now waits for the last chapter to be
    fully drawn (it caught chapter V mid-fade once); 57 of 57 on one theme, twice. Committed on
    `session/realignment-2026-09-21`; new work continues on `session/trading-cards-2026-09-28`.

27. **CI moves to Woodpecker; qorsync.dev is the operations domain, 28 September 2026 (ADR-058, ADR-042).**
    - **Found:** `qorsync.dev` was bought through Vercel, its DNS is Vercel's and verified, and its zone held only
      Vercel's managed defaults (`*` and apex ALIAS to Vercel; CAA for Let's Encrypt, Sectigo, Google). Every name
      resolved and none had a certificate. The Vercel team has **no projects**. `demiurge.cloud`'s DNS is still at
      GoDaddy. Vercel cannot host Woodpecker, QOR ID, Postgres, Redis or a node.
    - **Done:** ADR-042 accepted. `.woodpecker/` holds the six former Actions jobs command for command (chain,
      two-validators nightly, qor-auth with Postgres and Redis services, launcher with its view checks run as a
      non-root user for Chromium, coverage to an agent volume, security as always-run steps). `infra/woodpecker/`
      holds the Fly server config, the PC agent's compose file and a seven-step README. `ci.yml` runs only by
      hand; `probe.yml` deleted. GATES `kinds.ci` now reads Woodpecker (evidence-rule entry, owner-approved).
      AGENTS.md §7's CI sentence updated.
    - **Owed:** the owner's GitHub OAuth app; then `fly deploy`, the `ci` A/AAAA records in Vercel, the agent, and
      trusting the repository for volumes; the gates dashboard's CI and coverage readers rewritten for Woodpecker.
      No pipeline has run, so none of `.woodpecker/` is proven yet.

28. **The operations server on Oracle Cloud Always Free, 28 September 2026 (ADR-059).** The owner asked for a free
    option to ADR-015's $50–70 a month and chose Oracle. `infra/oracle/`: `compose.yaml` (Caddy, QOR ID in
    production mode with no email, Postgres, Redis, Woodpecker server), `Caddyfile` (`id`, `ci`, `ci-grpc`),
    `bootstrap.sh` (Docker, the machine's iptables, secrets made on the machine), `deploy.ps1`,
    `set-github-oauth.ps1` (the secret typed by the owner, sent on SSH standard input), and a README for the
    owner's five steps. ADR-058 decision 1 superseded; `infra/woodpecker/fly.toml` deleted; the agent points at
    `ci-grpc.qorsync.dev:443`. `.gitattributes` pins LF for `infra/`. SSH key `~/.ssh/qor-oracle` made on the
    owner's computer; its public half is in the README. `bash -n` and PowerShell's parser pass; **nothing has run
    against a machine**, because none exists yet.
    - **The public RPC node is withheld on purpose:** `--dev`'s sudo is `//Alice`. Owed: a test chain specification
      with a privately generated sudo key in `chain/`, then the node in the stack and `rpc.qorsync.dev`.

29. **Oracle rejected; the operations stack runs on the owner's computer behind a Cloudflare Tunnel, 28 September
    2026 (ADR-060).** `infra/oracle/` became `infra/ops/`: `compose.yaml` (cloudflared, QOR ID production, Postgres,
    Redis, Woodpecker server and agent, no host ports), `start-ops.ps1` (secrets in `%LOCALAPPDATA%\qor-ops\.env`,
    the tunnel token and GitHub app typed by the owner), README. Caddy, `bootstrap.sh`, `deploy.ps1`,
    `set-github-oauth.ps1` and `infra/woodpecker/agent/` deleted; `~/.ssh/qor-oracle` is unused. `docker compose
    config` accepts the stack; the QOR ID image was built. **Waits on the owner:** Cloudflare account, nameservers,
    tunnel, GitHub OAuth app.

30. **The operations stack is running, 28 September 2026 (ADR-060).** `cloudflared` 2026.9.3 installed by winget
    (its MSI reports 1603 at `RegisterProduct` without elevation, but the binary is installed and on the machine
    PATH). The owner made the Cloudflare account and added `qorsync.dev` (Free); Cloudflare assigned
    `georgia.ns.cloudflare.com` and `porter.ns.cloudflare.com`, which were set at the registrar through Vercel's API
    (`PATCH /v1/registrar/domains/qorsync.dev/nameservers`, team `astramatrix`). Before the switch, Vercel's zone
    held only its own system records (ALIAS, CAA, HTTPS for the apex and `*`) and no MX or TXT; Cloudflare's
    import already answered for the apex, `www` and `id`. `cloudflared tunnel login` done; the owner made the
    GitHub OAuth app and ran `start-ops.ps1`.
    - **Found and fixed:** the owner's terminal predated the install, so `cloudflared` was not on its PATH, the
      script carried on, and wrote `config.yml` with an empty tunnel ID; the container restarted for ever and the
      names answered 525 from Cloudflare's imported records. `start-ops.ps1` now finds cloudflared by its install
      path, throws on an empty ID or a failed DNS route, and rewrites a config with no ID. Re-run: tunnel `qor-ops`
      made, both names routed, four connections registered (sjc).
    - **Measured afterwards:** all six containers up and healthy; `https://id.qorsync.dev/health` 200
      `{"service":"qor-auth","status":"healthy","version":"0.1.0"}`; `https://ci.qorsync.dev/healthz` 204 and `/`
      200.
    - **Step 6 done by the owner:** repository active, webhook on GitHub, Trusted → Volumes only (Network and
      Security were ticked at first and unticked), cron `nightly` `0 3 * * *` on `main`.
    - **CI's first runs, 29 September, manual on the session branch.** Pushes to it start nothing: every pipeline
      runs on push to `main`/`develop`, pull requests to `main`, or manual. Run 1 (`73bc20a`): chain passed
      (55 min cold), QOR ID passed; four faults, none in product code: coverage's loop ended at the first
      unwritten pallet in `GATES.toml`; the fork check matched `GATES.toml`'s own description of it; the
      two-validator script looked in `chain/target` while the build used `CARGO_TARGET_DIR`; the launcher lost
      Docker's DNS mid-download (transient). Fixed in `d50c924` and `d9ba806`. Run 2 (`d9ba806`): chain (89 s
      warm), QOR ID, coverage, security and two validators (13 of 13) passed; the launcher failed at `npm ci`
      with no lockfile, fixed by tracking `tools/qor-launcher/package-lock.json`. Run 3 (`80d72d5`): the same five
      passed; the launcher installed, built and passed the design check, then no browser check could start
      Chromium: "No usable sandbox!". Docker's default seccomp refuses its namespaces as an ordinary user and
      with `chromium-sandbox` too (both measured in `node:24-bookworm`). The browser checks now pass
      `--no-sandbox` only when `QOR_CHECK_NO_SANDBOX=1`, which only `.woodpecker/launcher.yaml` sets; the
      alternative, a privileged step, would need Trusted → Security, which the owner turned off. **The owner chose this option on 2026-09-29.** Locally, without
      the variable, `check-accessibility.mjs` passed 41 of 41. Run 4 (`61ff33c`): chain, QOR ID, coverage,
      security and two validators passed, and so did every launcher frontend check (714 across eight scripts, 0
      failed); the host step then lost Docker's DNS during `apt-get install`, the second time in four runs, both
      in a large download while other pipelines ran. Every `apt-get` in `.woodpecker/` now retries
      (`Acquire::Retries=5`). **Run 5 is owed** for the launcher's host format, lints and tests.
    - Woodpecker reports a temporary `WOODPECKER_GRPC_SECRET` at each start; the agent re-authenticates, so it
      is harmless, but persisting one in `start-ops.ps1` would quiet it. Cloudflare still holds the imported apex and `www` records pointing at Vercel;
      nothing is served there.

31. **M4.2's royalty half: royalties, remix royalties and a sale settled in CGT, 29 September 2026 (ADR-061).**
    The owner's list put real selling next, which needs royalties first. ADR-047 had fixed the fields and bounds and
    left the mechanics undecided; ADR-061 decides them under the engineering delegation and invents no value.
    - **`pallet-drc369`:** `mint` gained `derived_from: Option<(collection, item)>`; the record gained `derived_from`
      and `remix_depth`, checked at mint against `MaxRemixDepth` = 16 (ADR-047); `Minted` carries `derived_from`.
      Errors `UnknownSource`, `RemixTooDeep`. 2 new tests, 15 in all.
    - **`pallet-drc369-royalties`** (new crate, `chain/pallets/drc369-royalties/`, mounted as `Drc369Royalties` at
      index 10): `set_terms` (creator only, while holding it, once; up to 8 recipients, `Permill` shares summing to at
      most one whole, and a remix share), `list`, `unlist`, `buy(max_price)`. `split` is one pure function every amount
      comes from: remix pool to the direct source's recipients pro rata, own shares of what is left, the rest to the
      seller; the parts sum to the price. **One level of remix royalty and immutable terms are product-shaped and the
      owner's to reverse before the freeze.** A part a recipient cannot receive (below the existential deposit, no
      account) refuses the whole sale. No platform share (U-15), no fee (OPEN-4), no deposit of its own (noted
      under U-14). 18 tests.
    - **Faults planted, each caught, then removed** (`diff` against the saved file confirmed the restore): own shares
      taken of the price rather than the remainder (4 tests failed); no check that the seller still holds the asset
      (1); terms settable twice (1); no receivability check (1); upstream payments not subtracted (6); and in
      `pallet-drc369`, the remix depth bound removed (1).
    - **Runtime:** `spec_version` 4, `transaction_version` 2 (the mint's encoding changed). `Nfts::buy_item` joins the
      filter test's refused list; the royalty calls pass the filter; bounds test pins 16 and 8; a runtime test runs a
      sale in CGT. 11 runtime asset tests.
    - **Evidence:** `cargo fmt --all`, `cargo clippy --workspace --all-targets -D warnings` clean; **`cargo test
      --workspace` 95 passed** without `SKIP_WASM_BUILD` (the runtime's wasm rebuilt during the run, 09:35). Release
      node with `sudo`: serves `spec_version` 4, metadata **97,710 bytes** with `Sudo` and `Drc369Royalties`; the
      mainnet shape, built without it: **95,802 bytes**, no `Sudo` (`chain/README.md`). **Two validators: 12 of 13, then
      13 of 13** — the first run reached block 3, not 4, inside the 120-second window and passed everything after; the
      re-run passed all. **Live on a `--dev` node, by a throwaway script since deleted:** Alice minted, set terms
      (Charlie 10%, remix 5%), listed at 1,000 CGT; Bob bought: Alice +900, Charlie +100, Bob −1,000, Bob holds it.
      Dave minted a remix of it (`derivedFrom` recorded, depth 1), listed at 200; Eve bought: Charlie +10 (the remix
      share), Dave +190, Eve −200.
    - **Launcher:** the mint passes `None` for `derived_from`. 162 host tests pass (`--features qontrol-no-skips`),
      and **all three live tests pass against the spec-4 node** (transfer, mint, trade), so reading records and events
      that now carry more fields works. The launcher does not call `list` or `buy` yet: that is L4.6's next half.
    - **Documents:** ADR-061 and both index rows; `PROTOCOL.md` §12–13; `DIRECTION.md` M4.2 (unticked, progress
      recorded) and L4.6; `GATES.toml` (`alpha.pallets` gains the pallet and 14 tests, a tightening logged);
      `OPEN_QUESTIONS.md` U-14 and U-15; `chain/README.md`; `SYSTEMS.md`; `OWNER.md`.
    - **The ticker, on the owner's word the same day: "it's CGT and we are NOT going with DMRG."** The code already
      said CGT (ADR-045). Stale `DMRG` prose was rewritten in `MIGRATION_INVENTORY.md` (R-4 now states CGT, with a
      history note correcting its old claim that the check "looks for `dmrg`"), `DECISIONS.md`'s index descriptions
      and `audit/RECONCILIATION.md`. Left as history: ADR-034 and its index row, ADR files and filenames (never
      edited), and `GATES.toml`'s change log.
    - **The owner's ruling on ADR-061's two choices, the same day (ADR-062):** "the one that ensures the best results
      and least amount of friction for users while also ensuring platform stability." One level of remix royalty is
      kept (the stability choice). Terms that never change is **reversed**: a creator may change their terms whenever
      they hold the asset, never while anyone else does; and once any remix names the asset its remix share may fall
      but not rise (`RemixShareLocked`, from a new `Drc369::RemixCount` written at a remix's mint). `TermsAlreadySet`
      is gone. Kept inside `spec_version` 4, since nothing was committed or deployed. Faults planted and caught: remix
      lock removed (1 test failed), creator-*and*-holder weakened to creator-*or*-holder (1), remix count never
      incremented (1). Pallets 15 and 19 tests. `GATES.toml` swaps the old terms test for its replacement and adds the
      remix-lock test (logged). **Evidence after the change:** clippy clean; `cargo test --workspace` **96 passed** with
      the wasm rebuilt (10:45); metadata 98,230 bytes with `Sudo`, 96,322 without (so the 97,710 / 95,802 above is
      superseded); two validators 13 of 13; and live on a `--dev` node, by a throwaway script since deleted: a
      correction to terms accepted, `RemixCount` 1 after a remix, raising the remix share then refused
      (`RemixShareLocked`), the 1,000 CGT sale paying Charlie 100 and Alice 900, Alice's edit after the sale refused
      (`NotCreator`), and the remix's 200 CGT sale paying Charlie 10 and Dave 190. The machine paused for about
      seventeen minutes mid-run (the node's log jumps from 10:48 to 11:05); nothing failed. The launcher's three live
      tests were not re-run on this last runtime; the mint they build is unchanged by ADR-062.
    - **Not done:** CI run 5 (Docker Desktop was off, `ci.qorsync.dev` answered 530; the owner started it after this
      session's work); the launcher's selling surface; nesting and state and XP.

32. **The repository goes public as `demiurge-chain`; CI back to GitHub Actions; QOR ID moving to Railway, 29 September
    2026 (ADR-063). HALF DONE — the owner has three steps, then the assistant has five (below).**
    *Since 1 October 2026 this is done: the repository is `QOR-MATRIX/demiurge-chain` (ADR-064), CI runs there and
    is green, QOR ID is live on Railway at `https://id.qorsync.dev`, and work reaches `main` by pull request. The
    owner steps and the branch names below are history.*
    The owner travels, so nothing may depend on their computer. The owner chose Railway for what must stay online and
    GitHub Actions for CI, with the repository public so Actions is free, published **without history** (option 1).
    - **Why not just flip the old repository public:** old credentials remain in its history, nine values in two
      tracked files were never rotated (`SECURITY.md`), and the deleted pre-realignment docs are in its history.
    - **Done — the public repository.** `ALaustrup/demiurge-chain`, public, default branch `main`: `e611c99` (the
      import: the private tree at `d9bb19a` less `docker/n8n/docker-compose.yml` and
      `docker/docker-compose.testnet.yml`) and `2681a51` (CI and records). Before publishing, gitleaks 8 (installed by
      winget) scanned the whole tree: 6 findings, all false positives (empty keys in `.env.example`, where gitleaks
      flagged the neighbouring region and index names; a storage-key name; a `YOUR_TOKEN` placeholder; a `0123…` test
      token; a deliberately wrong test secret). Verified after the push: public, one root commit, neither credential
      file in `git ls-files`. **The owner ran the first commit, `gh repo create` and the first push by hand**: Claude
      Code's auto-mode classifier refused them twice, even on the owner's explicit instruction. The owner's `printf`
      to `.gitignore` did not take, so `2681a51` adds the two ignore entries (checked with `git check-ignore`).
    - **Local clone:** now on branch **`public-main`**, tracking **`public/main`** (remote `public` =
      `demiurge-chain`). Remote `origin` is still the private archive `demiurge-cloud`; its branches, including
      `session/trading-cards-2026-09-28` at `d9bb19a`, are untouched. **Push new work to `public`, never `origin`.**
      The two credential files are still on disk, ignored. `archive-7gUg78/` (a GitKraken download) is untracked.
    - **Done — CI.** `.github/workflows/ci.yml` runs on push and pull request to `main`, nightly (two validators) and
      by hand, keeps the name "Pleroma CI" so the gates' `workflow` fields still match, and carries Woodpecker's fixes
      (coverage records a pallet not yet written as absent; apt retries). Chromium keeps its sandbox
      (`sysctl kernel.apparmor_restrict_unprivileged_userns=0` on the runner; `QOR_CHECK_NO_SANDBOX` is not set).
      `.woodpecker/` and `infra/woodpecker/` deleted. `GATES.toml` `kinds.ci` reads Actions again (evidence-rule entry,
      owner-approved). **The first run, `36617755694`, did not start: "your account is locked due to a billing
      issue".** That is also what every `startup_failure` since 21 September was. **It is not the workflow.**
    - **Done — Railway** (MCP connector, account `alaustrup`). Project **`demiurge`**
      `449a0842-d46d-416f-bf4c-c418f72c1d79`, environment `production` `7dfe7274-8af4-4202-aafb-38b475b9d123`, region
      `iad`. **`Postgres`** `4986dae9-7cd8-45ea-9a39-1ea3391cf690` and **`Redis`** `f5d7f670-73e2-4a8b-bfc2-e23573696047`
      from Railway's templates (passwords generated by Railway), both deployed SUCCESS. **`qor-auth`**
      `10e620ca-030e-40d4-9eca-d5c09d2ff78c` is **configured, with no source attached and never built**: root
      `/services/qor-auth`, Dockerfile, healthcheck `/ready` 120 s, restart on failure ×3, watch `/services/qor-auth/**`,
      and every non-secret variable (`DEPLOY-RAILWAY.md` lists them). **A temporary TCP proxy on Postgres**,
      `tokaido.proxy.rlwy.net:30413` (proxy `14c665a7-dd5a-4ed7-91d4-f20ea47a2e48`), exists only for the data copy and
      must be removed after it. Railway has deprecated `railway.json`, so it was deleted and the settings recorded in
      `DEPLOY-RAILWAY.md`. The owner's other Railway project, `fractal-node-room`, was not touched.
    - **Data.** The operations stack's QOR ID held **0 users**; the launcher's local one (`qor-local-pg`, started and
      stopped again for this) held **1, `godmode`**, with a chain account and 18 migrations. Its dump is
      `%LOCALAPPDATA%\qor-ops\qor_auth.dump`. The PC's operations QOR ID was stopped for a dump and restarted; the PC
      stack (`qor-ops-*`) is still running and still serves `id.qorsync.dev` and `ci.qorsync.dev` through the tunnel.
    - **Waiting on the owner, in any order:**
      1. **Unlock the GitHub account:** Settings → Billing and plans (a failed payment or a balance). Then re-run CI
         (`gh run rerun 36617755694 -R ALaustrup/demiurge-chain`, or push).
      2. **Paste QOR ID's two JWT secrets** from `%LOCALAPPDATA%\qor-ops\railway-qor-auth-secrets.txt` into Railway →
         `demiurge` → `qor-auth` → Variables, then delete the file. Generated locally and never printed or sent through
         the chat (`DEPLOY-RAILWAY.md`'s rule).
      3. **Run `& "$env:LOCALAPPDATA\qor-ops\restore-to-railway.ps1"`** in PowerShell. It asks for Railway's
         `POSTGRES_PASSWORD` (Postgres → Variables) and should print "1 user(s), 18 migrations". It mounts the folder
         into a `postgres:18` container, so it works in Windows PowerShell 5.1 too.
    - **Then the assistant, in order:** (a) `connect-service-source` on `qor-auth` with repo `ALaustrup/demiurge-chain`,
      branch `main`; watch its first Railway build; check the logs show the migrations already applied and `/ready`
      answers. (b) Delete the Postgres TCP proxy. (c) `generate-domain` `id.qorsync.dev` on `qor-auth` and give the owner
      the DNS record: in Cloudflare the `id` record is the tunnel's today, so the tunnel route is removed and the record
      replaced (the assistant can do it only once the Cloudflare connector is authorised through `/mcp`); wait for the
      certificate (`.dev` is HSTS-preloaded) and check `https://id.qorsync.dev/ready`. (d) Stop the PC's operations
      stack, delete `ci.qorsync.dev`'s record and the tunnel, and delete `infra/ops/` in a commit. (e) Agree a Railway
      usage limit with the owner (expected $5–15 a month) and rewrite `OWNER.md`, `SYSTEMS.md` and this file.
    - **Also owed, done 30 September:**
      - **The gates dashboard read the wrong repository.** It called `gh` without `-R`, and with two remotes `gh`
        picked `origin`: measured, `gh run list` in this clone returned a `demiurge-cloud` run. `[ci]` in
        `GATES.toml` now names `repository = "ALaustrup/demiurge-chain"` and every `gh` call passes it (run list, the
        branch head through `repos/<repository>/branches/<branch>`, the coverage download); a `[ci]` without it fails
        the reading instead of guessing. Logged as a tightening in `GATES.toml`'s change log; pinned by
        `ci_is_read_from_the_named_repository_only`. Checked against the live API: it now reads run `36695500285` on
        `4023451`, not met, the billing lock.
      - **`docs/architecture/HOSTING.md`** now says what ADR-063 changed (QOR ID, Postgres and Redis on Railway;
        CI free; ADR-042 accepted) and leaves the nodes, the indexer and the archive on ADR-015's Fly plan, which
        ADR-063 did not reopen. `docs/README.md`'s row for it said ADR-042 was Proposed; fixed.
      - **The launcher's default QOR ID address was `https://demiurge.cloud/api/v1`, which answers 404** and never
        served QOR ID. It is now `https://id.qorsync.dev/api/v1` (ADR-042, ADR-063), and so is the Settings
        placeholder. A saved `settings.json` still wins, and the owner's points at `127.0.0.1:8080`. The chain
        default `wss://rpc.demiurge.cloud` is unchanged: public RPC is withheld (ADR-063 decision 6) and its name is
        a HOSTING §2 proposal.
      - Launcher: fmt, clippy `-D warnings`, **163 host tests** (one new), frontend build and `npm run check` (281
        view checks) all pass.
    - **Found 30 September:** Docker Desktop was off, so the PC stack (`qor-ops-*`) and its tunnel were down and
      `id.qorsync.dev` answered **530**. It was not restarted: the stack is being retired, and Railway is the fix.
      Owner steps 1 and 2 were not done (the scheduled run `36695500285` at 09:20 UTC was refused for billing; the
      secrets file is still on disk); step 3 could not be checked, since this session had no Railway connector.
      The Railway steps (a) to (c) need that connector back.
    - **Checked 1 October, with the Railway connector back:** `Postgres` and `Redis` online; `qor-auth` offline, no
      source, no deployment, and **no JWT secret among its variable names** (read with `describe-service`, which
      returns names only; `list-variables` returns values in plaintext and was not called). The secrets file is still
      on disk, so owner step 2 is not done. Step 3 was not verified: checking needs the Postgres password. The TCP
      proxy `14c665a7…` is still ACTIVE, as the restore needs. CI: three runs, all refused, none since
      `36695500285`. **Step (a) was deliberately not started:** a first deploy would run the 18 migrations into the
      empty database, and the restore must go in first.
    - **Later on 1 October: the owner did steps 2 and 3; step (a) ran and the service does not start.** Both JWT
      secret names are among `qor-auth`'s variables, and the owner reported the restore printed "1 user(s), 18
      migrations". The source is attached (`ALaustrup/demiurge-chain`, `main`). Deployment `d1f3297f` at `4023451`:
      **the image built** (`cargo build --release --locked`, 1m 03s), then the container exited four times on
      `Error: migration 1 was previously applied but has been modified` and the `/ready` healthcheck failed.
      **Cause, measured, all 18 migrations:** the checksum in the dump equals the SHA-384 of the file with CRLF
      endings (the working tree, `core.autocrlf=true`, a Windows build) and never the LF form git holds and Railway
      builds. The SQL is unchanged. **Not fixed:** correcting the 18 checksums is a write to Railway's Postgres; the
      auto-mode classifier refused the command that generates the `UPDATE`s, so it waits on the owner's say.
      `%LOCALAPPDATA%\qor-ops\fix-migration-checksums.ps1` exists and its `.sql` does not, so it refuses to run.
      The secrets file is still on disk. The TCP proxy stays until the fix is in. Steps (b) to (e) not started.
      **Fixed the same day, and QOR ID RUNS ON RAILWAY:** the owner approved it; `fix-migration-checksums.sql` (18
      `UPDATE`s, each guarded by the CRLF checksum) was applied by the owner through the proxy, and the owner
      redeployed from the dashboard (the classifier refuses `redeploy` from here). Deployment `f0658467` is SUCCESS;
      its log reads "Database migrations applied", "Connected to Redis", "Listening on 0.0.0.0:8080". The owner's
      pasted run showed 18 × `UPDATE 0`, which is the script's second run; the first run's output was not seen, and
      the service starting is the evidence the checksums are right. **Email is not configured there**
      (`RESEND_API_KEY`, `EMAIL_FROM`, `BASE_URL`, `RESEND_WEBHOOK_SECRET` unset; the log warns), so no mail is sent.
      **Step (b) NOT done:** the owner declined the proxy's deletion when asked, so `tokaido.proxy.rlwy.net:30413`
      is still open to the internet, behind the Postgres password. **Step (c) NOT done:** the classifier refuses
      `generate-domain` ("DNS / Domain / Cert Changes"), so the owner adds `id.qorsync.dev` in the dashboard (port
      8080) and enters Railway's record in Cloudflare. The service has no public address yet, so `/ready` has been
      checked only by Railway's healthcheck. (d) and (e) not started.
      **Step (c) DONE later that day, by the owner's hands:** custom domain `id.qorsync.dev` → port 8080 on
      `qor-auth` (domain `711a8beb…`, certificate VALID), and in Cloudflare the `id` record is now a DNS-only CNAME
      to `esk0jahg.up.railway.app`. Measured from this PC: it resolves so at 1.1.1.1, and
      `https://id.qorsync.dev/ready` answers **200** `{"checks":{"cache":"ok","database":"ok"},"status":"ready"}`
      with `server: railway-hikari`; `/health` 200. (Railway's own `domain-status` still read "requires update"
      at that moment; it lags.) Sign-in with the restored account was not exercised from the launcher.
      **Step (d), mostly done on the owner's instruction:** `docker compose -p qor-ops down` removed the six
      `qor-ops-*` containers and the network; **the four `qor-ops_*` volumes are kept** (delete them when nothing in
      them is wanted). No scheduled task or startup entry restarts it. `infra/ops/` is `git rm`'d, **staged and not
      committed**. `id.qorsync.dev/ready` still 200 afterwards; `ci.qorsync.dev` answers 530. **Left for the owner
      in Cloudflare:** delete the `ci` DNS record and the tunnel (Zero Trust → Networks → Tunnels); no connector.
      **Email, half:** `EMAIL_FROM` and `BASE_URL=https://id.qorsync.dev` set on `qor-auth` with `skipDeploys`, so
      they take effect at the next deploy. The owner pastes `RESEND_API_KEY` (which redeploys), then adds the
      Resend webhook `https://id.qorsync.dev/api/v1/webhooks/resend` (bounced, complained, suppressed) and pastes
      its `whsec_…` as `RESEND_WEBHOOK_SECRET`. Not yet done, and no mail has been sent from Railway.
      **Step (b) STILL not done:** `delete-tcp-proxy` was cancelled at the approval prompt twice more.
      **Step (e):** the owner chose a **$10 a month** cap; it is set in Railway's dashboard (Workspace → Usage →
      limits), which no tool here reaches. **The secrets file** was emptied by the owner, not deleted.
      **Checked again later on 1 October:** **step (b) is DONE** — the owner removed the proxy, `list-tcp-proxies`
      on Postgres returns none. **`RESEND_API_KEY` is set** and deployment `87b5ae96` (SUCCESS) logs "Email service
      configured"; the only warning left is `RESEND_WEBHOOK_SECRET`. **No message has been sent from Railway**, so
      live delivery there is unproven. `/ready` 200. **GitHub is still locked:** `gh run rerun 36695500285` created
      six jobs, each refused in 3 s, "your account is locked due to a billing issue". `ci.qorsync.dev` still
      resolves, so the Cloudflare leftovers are not deleted. Whether the $10 cap is set is unknown from here.
      **Committed and pushed on the owner's word: `7e154e1` on `public/main`.** `/ready` 200 after Railway's rebuild.
      **Live mail from Railway is proven to Resend's test address:** `POST /api/v1/auth/register` as
      `mailtest_20261001` with `delivered@resend.dev` answered 201, and Resend lists "Confirm your email address for
      QOR ID" to that address as **delivered** (09:16 UTC, email `01a0f6bf…`). The link was not opened. **That test
      account now exists in the live database**, with a random password nobody kept; delete it or leave it. A real
      inbox is still untried. **Seen in the same Resend listing, not investigated:** two verification messages
      **bounced** on 22 September, to `hasemail@` and `withemail@example.invalid` — some local test run sent real
      mail through the owner's key. Bounces cost the sending domain reputation; find the run and point it at the
      stand-in (`RESEND_API_URL`) or unset the key for it.
    - **CI RUNS: the repository moved to the organisation `QOR-MATRIX` (ADR-064, 1 October).** The owner disputes
      the charge behind the personal account's lock and will not pay it, created the organisation, and by hand
      (`gh repo create QOR-MATRIX/demiurge-chain --public`, `git push matrix public-main:main`) pushed `7e154e1`
      there. **The owner saw its first run in progress**, the first job of this workflow ever to execute. **Nobody
      has seen a run finish**, and its Linux-only steps had never run before, so expect faults.
      - **The classifier refuses the assistant everything about that repository**: creating it, pushing, even
        `gh run list -R QOR-MATRIX/demiurge-chain` ("Data Exfiltration"), and `git remote set-url` ("Remote
        Repoint"). So the owner reads CI and runs the git commands, until they allow them in Claude Code's settings.
      - **Remotes, unchanged by the assistant:** `matrix` = `QOR-MATRIX/demiurge-chain` (push here now);
        `public` = `ALaustrup/demiurge-chain` (no longer pushed to; the owner archives it); `origin` = the private
        archive. `public-main` still tracks `public/main`.
      - **Changed in the tree, uncommitted:** ADR-064 and both indexes; `GATES.toml` `[ci].repository` and
        `kinds.ci` with an evidence-rule entry; the gates test's expected name; the README badge, the issue-template
        links, both `Cargo.toml` `repository` fields, `ci.yml`'s comment, `DIRECTION.md`, `HOSTING.md`,
        `DEPLOY-RAILWAY.md`, `SYSTEMS.md`, `OWNER.md`.
      - **Railway still deploys from `ALaustrup/demiurge-chain`.** Its GitHub app must be given the organisation
        (owner), then `connect-service-source` to `QOR-MATRIX/demiurge-chain`. Until then a push redeploys nothing,
        and `DEPLOY-RAILWAY.md` already names the new source: it is ahead of the service.
      - **Committed `34f21c1`, pushed to `matrix` by the owner.** Its CI run's result is not known yet.
      - **Step (e) closed by the owner: no Railway usage cap.** The $10 figure given earlier that day is withdrawn
        ("I am not capping railway"). Do not set one, and do not raise it again unless the bill does.
    - **Three agents, 1 October, on the owner's instruction** (chain M4.5 nesting; the launcher's Sell and Buy on
      chain; QOR ID's two faults). The QOR ID one is done and spot-checked by the lead:
      - **Line endings: closed.** `.gitattributes` pins `services/qor-auth/migrations/*.sql` to LF (`i/lf w/lf`,
        checked); `src/migration_hygiene.rs` fails the tests on a carriage return in a migration, on disk or
        embedded, seen failing first. `qor-local-pg` was corrected with
        `services/qor-auth/scripts/correct-migration-checksums.ps1` (18 rows; a second run changes nothing) and
        `target/release/qor-auth.exe` rebuilt. Any other local database migrated on Windows before today needs the
        script once (`scripts/run-local-stack.md` §4), then a rebuild. This closes the "Trap for later" above.
      - **The two bounced emails of 22 September: cause found and closed.** `EmailConfig::default()` read the
        environment, seven test helpers used it, and this PC has the real `RESEND_API_KEY`, `EMAIL_FROM` and
        `BASE_URL` set, so `registering_with_an_email_leaves_it_unverified` and
        `an_account_with_an_email_address_is_refused` sent real mail. `Default` is gone, only `main` calls
        `EmailConfig::from_env()`, a test build refuses any API base not on this machine, and two guard tests pin it.
        **Still open:** a *running* local QOR ID started with the ambient key and no `RESEND_API_URL` sends whatever
        the e2e scripts register; `start-local.ps1` inherits the user environment. Owner's call: clear the key for
        that child, or take it out of the user environment now that Railway holds it.
      - **130 tests** (125 + 5) with Postgres, Redis and `--include-ignored`; fmt and clippy clean; log and SQL
        hygiene pass. The five are required by `alpha.qor-auth-tests` (tightening, logged). `migration_hygiene` is a
        new test-only module name, after `sql_hygiene` and `log_hygiene`: AGENTS §8 asks the owner about names.
        Not yet run in CI.
      - **Chain: nesting and R-2 built (ADR-065), `spec_version` 5.** `Drc369::nest` / `unnest` in
        `pallet-drc369`, which is now `pallet-nfts`'s `Locker`: a nested asset and the asset holding it cannot be
        transferred, sold or burned. Depth 8, 64 children (ADR-047). **109 workspace tests with the wasm built**
        (was 96), fmt and clippy clean, as the agent reported; the lead did not re-run the suite. 13 new tests, each
        seen to fail first, are required by `alpha.pallets` and `alpha.chain-tests` (tightening, logged).
        **M4.2 and M4.5 stay unticked: state and XP are not built.** ADR-065 records twelve choices; **1, 2 and 5
        are product choices for the owner** (a parent holding assets cannot move or be nested; a nested asset can
        be listed but not bought). **Not done:** the release node was not rebuilt, so the running
        `demiurge-node.exe` still serves `spec_version` 4 and nesting has never run on a live node;
        `chain/README.md`'s metadata sizes were not re-measured; the launcher has no nesting surface and was not
        built against the new metadata.
      - **Launcher: Sell publishes on chain and Buy settles (L4.6, still unticked).** `src-tauri/src/chain/sales.rs`
        and five commands (`drc369_sale`, `_sale_preview`, `_list`, `_unlist`, `_buy`), each through the host
        dialog; `BuyDialog.tsx`, `SaleBreakdown.tsx`, a rewritten `SellDialog.tsx`. Buying is by an asset's number,
        inside Inventory: there is no catalogue, because there is no indexer. As the agent reported: **182 host
        tests** (was 163), fmt and clippy clean, **5 live tests** against the spec-4 node including
        `a_sale_pays_every_part_and_hands_the_asset_over` (a remix sold at 2,000 CGT + 7 Sparks, every part exact to
        the Spark, declined prompts moving nothing), and `npm run check` exit 0 **on its second run** — Inventory
        238, readability 341, the rest unchanged. The first run failed on the accessibility case "stored off, then
        live: the backdrop starts", the known intermittent one.
        - **Owner's decisions owed:** (1) **`check-readability.mjs`'s measuring rule changed** — text clipped inside
          a scrolling dialog no longer counts as visible. Logged in `GATES.toml` as evidence-rule, **NOT YET
          APPROVED**. (2) The module name `chain/sales.rs` and the five command names (AGENTS §8). (3) Buying by
          asset number as the surface.
        - **Known weaknesses (the first two removed on 2 October, items 33 and 34):** the payout preview is a **second copy of the pallet's `split`** in the host
          (`sp-arithmetic =28.0.1`), pinned by the pallet's vectors and the live `Sold` event; a runtime API would
          remove it. **A revision landing between the dialog and the block is not caught**: the host compares
          fingerprints, the chain's `buy` has only `max_price`. One view check (the re-read after a purchase)
          cannot fail, and the README says so. `market.md` decision 9 allows a zero price and the pallet refuses
          it. Royalty terms and remix mints have no launcher surface. No person has used any of it.
        - **Traps, to move to §5:** a view check waiting for "the chain was asked again" passes on an unrelated
          re-read (Inventory reloads when the active account object is replaced, which may also flicker in the
          real app); `cargo test --lib chain::live -- --ignored` also runs `vault::keychain::live`.
      - **A devnet plan exists, undecided: `docs/architecture/DEVNET_PLAN.md`** (an agent's research, 1 October). It
        recommends Railway with two validators and a separate RPC node, which needs an ADR superseding ADR-015 for
        the nodes; cost is NOT measured (an illustration of about $11-22 a month from Railway's unit prices).
        **Missing in `chain/node/`, read in the code:** no specification without the well-known Alice/Bob keys, no
        `key` subcommand mounted, no container image for `chain/` (the root `Dockerfile` and `fly.toml` still
        describe the deleted `framework/`), no first-boot key script. **The hard question is the owner's:** a
        non-dev genesis with no CGT means nobody can mint and the sudo key cannot sign (it needs the existential
        deposit), and AGENTS §5 forbids creating CGT outside `--dev`. Two validators also stop finalising whenever
        either restarts. Nine owner decisions are listed in the plan.
      - **Launcher 0.1.5 built, on the owner's instruction** (was 0.1.0 everywhere, the installed copy included):
        `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `Cargo.lock` and `tauri.conf.json`.
        `npm run app:build` produced `src-tauri/target/release/bundle/msi/QOR Launcher_0.1.5_x64_en-US.msi` and
        `bundle/nsis/QOR Launcher_0.1.5_x64-setup.exe`, **unsigned** (no code-signing identity yet, L6), carrying
        the selling surface. Not installed and not run by anyone; no check was run on the packaged build.
      - **Integration, done:** the release node was rebuilt at `spec_version` 5 (2m 34s) and restarted on a fresh
        `--dev` chain, and the launcher's five live tests passed against it (378 s), the sale among them
        (`%LOCALAPPDATA%\qor-ops\integration-spec5.log`). So the launcher's sale works on the runtime that has
        nesting. Nesting itself has still not been called on a live node. A `--dev` node keeps its chain in a
        temporary directory, so every restart is a new chain.
      **Trap for later:** nothing pins these files' line endings, so a Windows build and a Linux build of QOR ID
      cannot share a database; a `.gitattributes` `eol=lf` on `services/qor-auth/migrations/*.sql` would end it, and
      would in turn need the same checksum correction on the launcher's local database (`qor-local-pg`).

33. **2 October 2026: unrecorded, half-finished work found in the tree, finished and verified. UNCOMMITTED.** *(Since
    committed; it is on `main`.)* The
    owner said "proceed". The working tree held a previous session's changes in three areas that `HANDOFF.md` did not
    mention, cut off mid-work. What they are, what was wrong, and what was run:
    - **Chain, `spec_version` 6.** `Drc369Royalties::buy_exact(collection, item, max_price, content)`, call index 4:
      `buy` that also refuses with `ContentChanged` if the asset's `current` reference is not `content`. `buy` keeps
      its index and encoding (pinned in `runtime/tests/assets.rs`), so `transaction_version` stays 2. And the runtime
      API `Drc369RoyaltiesApi::sale_preview(collection, item, price, buyer)` — the parts through the same function
      `buy` uses, and `refusal`: with a buyer the real settlement is run inside `with_transaction` and rolled back;
      without one, zero price, unreceivable parts and `ItemLocked` only. **Wrong when found:** one clippy error in
      `runtime/tests/assets.rs` (redundant closure). **Run:** `cargo fmt --check`, `clippy --workspace --all-targets
      -D warnings`, **`cargo test --workspace` 118 passed** without `SKIP_WASM_BUILD` (was 109: royalties 26, runtime
      asset tests 16). Release node rebuilt (no `sudo` feature) and **running now** as `--dev --tmp` (pid 33028 at
      the time, log `%LOCALAPPDATA%\qor-ops\node-spec6.log`); it reports `spec_version` 6 and serves **99,901
      bytes** of metadata without `Sudo` (the mainnet shape; the `sudo` shape was not re-measured). `PROTOCOL.md`
      and `chain/README.md` updated.
    - **Launcher.** `chain/market.rs` (787 lines) and the command `drc369_market`: every listing read by walking
      `Drc369Royalties::Listings` at the finalised block, paged, bounded, details only for the window asked
      (L7.2's host half). Buy now knows nesting (`Drc369::ParentOf` / `ChildCount`; `ItemLocked` in words).
      **Wrong when found:** `market.rs` never formatted, and **four user-facing sentences in `sales.rs` broken** — a
      lost `\` continuation left a run of spaces inside each string. Likely cause: **a heredoc through the Bash tool
      turns `\` + newline into nothing**; it happened again in this session and was fixed with Edit. Trap for §5:
      never write Rust string continuations through a shell heredoc. **Added this session:** Buy sends **`buy_exact`**
      with the content reference that was on screen (`ContentRefArg::try_from(&ContentView)` in `assets.rs`, refusing
      an unknown algorithm or a root that is not 32 bytes), and `ContentChanged` is put in words; this closes the
      "revision landing between the dialog and the block" weakness in item 32. **Run:** fmt, clippy `-D warnings`,
      **193 host tests** (was 182), and **the five live tests against the spec-6 node, all passed (377 s,
      `%LOCALAPPDATA%\qor-ops\integration-spec6.log`)**, the sale through `buy_exact`. A launcher now needs a node at
      spec 6 or later to buy. **Not done:** no Market screen (no `.tsx` change, nothing in `ipc.ts`);
      `drc369_market` has never run against a node; the host still carries its own copy of `split` instead of
      calling `sale_preview`; `npm run check` was not run (no view changed).
    - **QOR ID.** Sessions record `last_activity` at each refresh (`SessionService::record_use`, Redis `SET … XX
      KEEPTTL`, so a revoked session is not revived and expiry never moves; refresh refuses if the session is gone),
      shown as `last_used_at`; `GET /health` gains `email_leaves_this_machine`; every e2e script imports
      `scripts/e2e/_guard.mjs` and exits 2 unless that field is exactly `false`; `start-local.ps1` gained an email
      stand-in mode. The refresh and the session list joined the log check (AGENTS §9). **Wrong when found:**
      `mail_leaves_this_machine` was a stub returning `false` — the unsafe answer, which would have let every e2e
      script run against a real-mail service — with its test failing and `on_this_machine` unused. Implemented as
      `sends && !on_this_machine(api_url)`. **Run** against throwaway `qa1002-pg` (Postgres 16, port 55435) and
      `qa1002-redis` (7.4, 56381), both `--rm`: fmt, clippy `-D warnings`, **136 passed with `--include-ignored`**
      (was 130). **Not run:** the e2e scripts themselves, including the five new `sessions.mjs` checks.
    - **Owner's decisions owed (AGENTS §8, names):** the call `buy_exact`, the error `ContentChanged`, the runtime
      API `Drc369RoyaltiesApi` / `sale_preview`, the launcher module `chain/market.rs` and command `drc369_market`,
      the health field `email_leaves_this_machine`. Also: `/health` now tells anyone one bit of configuration
      (whether a deployment sends real mail); the code comment argues it is already inferable from `forgot-password`.
    - **Documents:** `PROTOCOL.md`, `chain/README.md`, the launcher README, `SYSTEMS.md`, `OWNER.md`. `DIRECTION.md`
      not changed: no roadmap item is ticked by this.
    - **Next:** the launcher's preview through `sale_preview` (removes the host copy of `split`), then the Market
      screen; then CI and the devnet (item 32's `DEVNET_PLAN.md`, waiting on the owner's nine answers).
    - **Committed `2ba0a85`** on the owner's instruction ("commit all and push and deploy when satisfactory").

34. **2–3 October 2026: a sprint with two agents, on the owner's instruction. The preview from the chain, and the
    Market screen.** Each agent worked in its own git worktree from `2ba0a85` (both worktrees were first created at
    an unrelated commit, `7bfcec6`, and each agent reset its own to `2ba0a85`; the main tree was not touched).
    - **`fbe447d`, the launcher asks the chain what a sale pays.** `sales.rs` calls `Drc369RoyaltiesApi_sale_preview`
      at the finalised block and draws the breakdown from the answer; the host's copy of `split`, its four vector
      tests and the `sp-arithmetic` dependency are gone. The answer is refused if its parts do not sum to the price
      (checked add). A refusal is named through the metadata and put in the same words as a refused transaction. A
      node without the API (spec_version below 6) is refused in words, with no fallback. IPC shapes unchanged.
      Eight faults planted, each caught. **Behaviour change:** looking up a listed asset, previewing, listing and
      buying now need spec 6; withdrawing does not.
    - **`09ad4b5`, the Market screen (L7.2's first slice).** A `market` surface on the rail and a Nexus tile;
      `src/views/Market.tsx`; `AssetCard` gained a `listed` mode; IPC `market.page` over `drc369_market`. Filter
      (everyone's, others', yours), order (cheapest, dearest, by number), paging by the host's window, the truncation
      and unreadable notices, an InfoTip saying the list is one node's view with no search, history or "newest".
      Buy re-reads the asset and opens the existing BuyDialog; Withdraw and, for the holder, Clear void listing.
      Copy that said "no storefront" now names the Market. **`check-market-view.mjs`, 127 checks; 24 planted faults,
      all caught.** Two `#[ignore]` live tests. **Product choices the agent made, for the owner:** Clear offered only to
      the holder (the chain lets anyone clear a void listing); the labels Everyone's / Others' / Yours, the eyebrow
      "Exchange", "No search yet" on the Nexus tile; a page past the end jumps to the last page. **Not covered by the
      check:** that jump, and the Refresh spinner.
    - **`3de3d46` merges both** (one README conflict, the test count). Chain and QOR ID untouched by either.
    - **Verified on the merged tree:** host fmt, clippy `-D warnings`, **195 passed, 8 ignored**. **Live, against a
      spec-6 `--dev` node, all seven chain tests passed**: six in one run (296 s: the market read, a listing read back
      by the Market, mint, transfer, trade, keychain) and the sale alone (305 s). **`npm ci`, `npm run build` and all nine view checks passed first time**: design 8, accessibility 41, gates 34, Projects 104, Inventory 238, Market 127, Gate 65, contrast 54, readability 361. The agent saw the accessibility case "stored off, then live" fail more often on its build (7 of 10 alone, against 3 of 8 for `2ba0a85`); cause not established, threshold not touched.
    - **Two false failures on the way, both environmental:** the PC entered Modern Standby 15:31–01:05 during the
      first live run (Kernel-Power 506/507), so two tests timed out on finality after 9.6 hours; and a node relaunched
      through `Win32_Process.Create` with `cmd /c` received Ctrl+C when a shell command ended (`^C` in its log) and
      died mid-test, which surfaced as subxt's "`chainHead_follow` emitted 'stop'". **Trap for §5:** start the
      development node with `Start-Process -WindowStyle Hidden` (its own console), never through `cmd /c` from a tool
      shell, and keep the PC awake during live runs.
    - **QOR ID's end-to-end scripts, run for the first time since item 33** against throwaway Postgres 16 and Redis
      7.4 (removed after) and the release build: **all six pass** — account-and-admin 24 (2 skipped, email off),
      sign-in-enumeration 11 (1 skipped), sessions 26 (the last-used checks included), registration 8, logout 15 (the
      Redis restart included), email-via-resend 50 (2 skipped) through the stand-in. **The guard was seen to refuse:**
      started with a placeholder key and no stand-in, `/health` said `true` and `sessions.mjs` exited 2 before
      registering anything. Trap: a Redis started with `--rm` is deleted by `logout.mjs`'s `docker stop`.
    - **Railway, read on 3 October:** `qor-auth`'s source has **no repository connected** (root directory
      `/services/qor-auth` only), and its live deployment is `7e154e1` (1 October). So a push redeploys nothing until
      the service is connected to `QOR-MATRIX/demiurge-chain`, which needs Railway's GitHub app to see the
      organisation (owner).
    - **Committed `26e3636`. The push was refused** by Claude Code's classifier (`git push matrix public-main:main`),
      and on 3 October so was `git fetch matrix`: the owner pushes. `matrix/main` was `34f21c1`, an ancestor, so the
      push is a fast-forward.
    - **Railway, read again on 3 October:** the owner redeployed `qor-auth` at 09:09 UTC, but it rebuilt the same
      `7e154e1` (no repository is connected), and **`sleepApplication: true` is now set** on the service, so QOR ID
      sleeps when idle and the first request wakes it. `/ready` and `/health` answered 200 (0.44 s, 0.27 s); `/health`
      has no `email_leaves_this_machine`, so the new code is not live. Whether sleeping suits "nothing may block on
      QOR ID" (ADR-056) is the owner's call; the launcher already does not wait on it.
    - **GATES.toml tightened** (two `change_log` entries, 3 October): six chain tests and one QOR ID test join the
      Alpha criteria, each seen to fail against a planted fault first, and the Market check joins `npm run check`'s
      record. Planting showed **the pallet's mock has no `Locker`**, so a preview that ignored nesting passed every
      pallet test and failed only the runtime's. Not listed, not yet seen failing: `buy_exact_keeps_every_rule_buy_has`,
      `a_preview_at_the_largest_price_cannot_overflow`, `buy_exact_is_a_new_call_and_buy_keeps_its_encoding`, and QOR
      ID's health and session last-used tests. The launcher's 15 gates tests pass against the new file.
    - **Deployed, 3 October, by the owner:** pushed to `matrix` and connected `qor-auth` to `QOR-MATRIX/demiurge-chain`
      (branch `main`, root `/services/qor-auth`). Deployment `ca92e0f4` of `342b6c5` **SUCCESS** at 09:53 UTC.
      Measured: `/health` 200 with `email_leaves_this_machine: true` (right for production: it sends real mail, so the
      e2e scripts refuse it), `/ready` 200 with database and cache ok (~0.35 s from here), a forged refresh token 401.
      Production Redis is 8.2, so `SET … XX KEEPTTL` is supported. `sleepApplication` is off again. **The service
      moved region to `us-west2` while Postgres and Redis stay in `iad`**: every query crosses the continent. Owner's
      call whether that was meant; if not, set qor-auth's region back to `iad`.
    - **The owner's decisions, 3 October (ADR-066):** `buy_exact`, `ContentChanged`, `Drc369RoyaltiesApi::sale_preview`
      and `email_leaves_this_machine` approved as named; the Market's Clear on a void listing stays holder-only; its
      labels and internal names stand. **`my-app/` deleted on the owner's word**: an untouched `create-next-app`
      starter that appeared at 02:00 on 3 October, never committed, origin unknown.
    - **The owner took every recommended option (ADR-067):** ADR-065 choices 1, 2 and 5 confirmed; buying by number
      kept beside the Market; the readability measuring rule approved (`GATES.toml` `approved_by`). **qor-auth moved
      back to `iad`** through Railway's `update-service` on the owner's word; the classifier then refused even
      read-only checks of the live service ("Production Deploy"), and **the owner confirmed `/ready` answered
      `{"checks":{"cache":"ok","database":"ok"},"status":"ready"}`**. Launcher bumped to **0.1.6** for an installer
      that upgrades 0.1.5.

35. **3 October 2026: the devnet is decided (ADR-068), and its engineering has started.** The owner funded nothing
    yet; their account `5DMPEXVcNq6ik8jzW4S7TracpEb1mnabmKbge76Xcd75qLxK` got 10,000 CGT on the local `--dev` chain
    through `dev-fund.mjs` (issuance unchanged) to try launcher 0.1.6. An "Agent IQ" idea (private simulated agents in
    the launcher) was raised and **scrapped by the owner the same day**; nothing was built. The owner then took every
    recommended answer to `DEVNET_PLAN.md` §9:
    - **ADR-068:** Railway; two private validators and a public RPC node (accepts ADR-044; supersedes ADR-015 for the
      nodes); `Demiurge Devnet` / `demiurge_devnet` / `Live`; genesis holds **only** a sudo account (existential
      deposit) and a separate faucet account (the development endowment's marked placeholder of test CGT), both the
      owner's, addresses not yet given; `alpha.devnet-live` will read `https://rpc.qorsync.dev/health/readiness`; a
      new **`alpha.devnet-finality`** check (tightening, logged); the assistant creates Railway services where
      permitted, the owner clicks where not.
    - **An agent is building, in a worktree:** the `key` subcommand, a `devnet_chain_spec` function (no raw spec until
      first boot gives real keys), `chain/Dockerfile`, a first-boot script that prints only public values, and a
      manual `devnet-image.yml` workflow publishing to `ghcr.io/qor-matrix/demiurge-node`, verified with three local
      containers. **Owed by the owner:** the sudo and faucet addresses, from a browser wallet.
    - **Merged `4366b81` (agent commit `6c9808c`).** Chain **124 tests** with the wasm built (was 118: five devnet-spec,
      one CLI), fmt and clippy clean, eight planted faults caught. `chain/Dockerfile` (context `chain/`, image 236 MB),
      `chain/docker/entrypoint.sh` (role from `DEMIURGE_ROLE`; drops to user `demiurge` uid 10001 with `setpriv`; keys
      made with `umask 077`, phrases only in 600 files then in the keystore; prints `DEMIURGE_PUBLIC` lines; waits
      without a spec at `DEMIURGE_CHAIN_SPEC`), `.github/workflows/devnet-image.yml` (manual, to
      `ghcr.io/qor-matrix/demiurge-node:devnet` and `:sha-…`), `.gitattributes` forcing LF on both. **Local evidence:**
      first-boot log carried peer id and both public keys and **0 of 27 secret patterns** (positive control 1); a second
      boot reused the keys; two validators and an RPC node on a Docker network authored and finalised (best 19,
      finalised 16), `system_chain` "Demiurge Devnet", `author_rotateKeys` and `author_insertKey` refused, readiness 200.
      **Two departures from the plan, both measured:** `--allow-private-ip` on every node (a `Live` chain refuses
      private addresses, and Railway's network is private), and **`--rpc-cors all` on the RPC node only** (the default
      403s any public Host; plan §2 corrected). **Not yet tried:** the workflow, a Railway volume's ownership, libp2p
      over Railway's private network. The root `Dockerfile`, `.dockerignore` and `fly.toml` still describe the deleted
      chain and were left, as `DIRECTION.md` records; removing them is a separate decision.
    - **THE DEVNET IS LIVE, 3 October 2026 (record: `chain/DEPLOY-RAILWAY.md`).** Owner's sudo
      `5HN6PZA4…Zadn` and faucet `5GuugU7t…4T9` (polkadot{.js} extension); genesis `0x934e2caa…254a` from
      `chain/specs/demiurge_devnet.raw.json` (`f836075`, 125 chain tests), baked into `sha-f836075`; three Railway
      services in `iad` with volumes; `rpc.qorsync.dev` on `devnet-rpc` only. **Measured:** readiness 200, two peers,
      unsafe methods refused, ten minutes finalising #55 → #155 with authors 50/51 and all three nodes agreeing on
      #155. **`alpha.devnet-live` has its URL and `alpha.devnet-finality` is done (tightening + evidence, logged).**
      The launcher's default endpoint is now `wss://rpc.qorsync.dev` and its chain preset "Mainnet" (a dead address)
      became "Devnet"; 195 host tests, and the read-only Market live test passed against the devnet. **Traps, for
      §5:** Railway labels every standard-error line "error"; an image's `sha-` tag is the short commit, so a workflow
      run before a push builds the old commit (it happened once); the organisation had to allow public packages
      before the image could be made public. **Not done:** no faucet page (the owner sends test CGT by hand from
      polkadot.js Apps); the launcher's signing paths have not run against the devnet; RPC rate limits unset
      (unmeasured); a week's cost not yet read.

36. **4 October 2026: ARQADE reconciled with the tree. Documents only.** The owner handed over a master implementation
    prompt for **ARQADE**, the ecosystem's future gaming platform (QOR ID, DRC-369 only, CGT only, a developer SDK),
    starting from an existing arcade on OpenAI Sites, and asked for whatever did not fit the ecosystem to be corrected.
    Three read-only audits, each with file and line evidence: the arcade's source (its claims held: four solo games, two
    server-checked arenas, `localStorage` Energy and tokens, a `qor_id` column nothing writes, `chain_getBlockNumber` at
    `rpc.demiurge.cloud`; and it shows USD prices for Energy and names CRGT and DMRG), the chain, and QOR ID with the
    launcher. **What the prompt got wrong**, all recorded in `docs/blueprints/arqade.md` §0: QOR ID has no browser
    sign-in (ADR-043 Proposed) and **no way for another server to verify a token** (HS256, shared secret, no
    introspection); nothing connects a browser to the vault, and ADR-011 says web surfaces sign with delegated keys
    (M5.2, unbuilt; no `pallet-proxy`); ADR-011 allows two web surfaces; `DESIGN_SYSTEM.md` rules ARQADE's look out on
    every surface; the chain has **no randomness, no account-bound asset and no enforced editions** (new gaps G-14 to
    G-16 in `ECOSYSTEM.md`); state and XP are M4.2; one faucet, one indexer, the SDK is M5.1 and unpublished before the
    freeze; third-party games list through Market (P5.3); prize funding has no source (new **U-16**). **Written:**
    the brief, ADR-069 (decision 1 is the owner's direction; 2 to 9 await the owner), P7.1 to P7.10, the `arqade` gate
    (a tightening, logged), and the indexes. The owner's copy of the prompt (`Downloads/ARQADE.txt`) was replaced by
    the corrected brief, the original kept beside it. **Found, not fixed:** the runtime's `TOKEN_NAME` is
    `"Creator God Token"` (`chain/runtime/src/denomination.rs:55`) where every record writes Creator-God Token;
    changing it changes the chain's properties, so it is the owner's call. **P7.2 (read-only devnet reads in the
    arcade) can start today**; everything else in P7 waits on ADR-069.
    **Later the same day the owner accepted ADR-069 and ADR-043 as written** ("I accept ADR-069 and ADR-043 as
    written"); a first, one-word "yes" was not recorded, because the assistant's tooling refused to mark decisions on
    so little. ADR-069's three further choices (design, hosting, name) and U-16 stay open. ADR-043's items 1 to 3
    (redirect sign-in, rotating browser tokens, per-client sessions) may now start; item 4 was already done. Both
    `arqade.decisions` units are met.

37. **4 October 2026: ARQADE enters active scope (P7.1 started, P7.2's first read). On branch
    `session/arqade-2026-10-04`.** With ADR-069 accepted, the arcade's tracked tree at `4f02baf` was imported verbatim
    into `products/arqade/` (`f647c82`; 114 files, `tsconfig.tsbuildinfo` left out; its two vendored files MIT, from
    OpenAI and shadcn; no credentials). **Baseline before any change:** 7 of 7 tests, `tsc` clean, build succeeds,
    lint fails with 13 errors and 6 warnings already present (React Compiler rules in minified code). **Then:** the
    deleted chain's `chain_getBlockNumber` at `rpc.demiurge.cloud` is replaced by `lib/chain.ts`, a read-only reader of
    Demiurge Devnet over standard Substrate RPC at the fixed `https://rpc.qorsync.dev`, which refuses any chain whose
    genesis or name is not the devnet's and reports finalized and best blocks (`/api/chain`; the QOR panel shows the
    finalized number). Five new tests against a scripted node; two planted faults (genesis check removed, finalized
    past best allowed) each failed one. Read live: finalized #12,028, genesis matched. CRGT, DMRG, the USD prices and the
    "Energy Tokens → DMRG" panel are gone from the copy; Energy is labelled practice and never CGT. `npm test` (12 pass)
    and `npm run typecheck` scripts added. **Scope (ADR-050 decision 10):** `AGENTS.md` §4 and `DIRECTION.md` §8 list
    `products/arqade/`; `[suites.arqade]`, `arqade.tests`, and an `arqade` CI job (install, type check, tests, build)
    in `[ci].quality_gates`; lint under `[ci].reports`, not failing the run — all logged as a tightening. **P7.1 is not
    ticked:** the integration inventory is unwritten and the CI job has not yet run. Not done: the ARQADE rebrand,
    the lint errors, balance and inventory reads (the rest of P7.2), and the arcade site on OpenAI Sites still serves
    the old copy, because its deploy needs the owner's Sites tooling.

38. **4 October 2026: Game Vaults designed, and the ARQADE SDK started. On `session/arqade-2026-10-04`.** The owner:
    games pay CGT rewards from a specialised wallet per game project, available to developers through the SDK, with
    an innovative way to unlock it. **ADR-070 (Proposed)**: a keyless account per game (`PalletId` sub-account, no
    key to leak) in a pallet of its own (placeholder `pallet-game-vaults`; ADR-026's proxies were weighed and
    rejected, because a proxy filter cannot see amounts and a pure proxy's spawner holds an `Any` proxy); the game's
    build minted as a DRC-369 **Cartridge**, whose holder governs the Vault; a payout authority key bounded by an
    on-chain policy; outcome ids paid once; loosening and withdrawals delayed, tightening instant; prizes held before
    paid rounds open; payouts below the existential deposit accrued; and "ignition", the SDK's unlock walk ending in a
    finalised devnet payout. U-16's "what funds a prize" now records the owner's answer; its bounds and paid entry stay
    open. **Written:** the ADR, `products/arqade/sdk/` (README, `docs/game-vaults.md`, `src/amount.ts`,
    `src/vault-policy.ts`, 4 tests), P7.11, the gate's two new units (a tightening, logged), the indexes. `npm test`
    16 of 16; `tsc` clean; two planted faults (excess precision truncated instead of refused; a shorter epoch read as
    tightening) each failed a test. **Nothing of the pallet is built**, and no number is set: every cap is the
    developer's, every bound U-16's.

39. **4 October 2026: ARQADE as a self-publishing store (ADR-071, Proposed). On `session/arqade-2026-10-04`.** The
    owner: an app-store-like process where developers self-publish once requirements are met, a project profile from a
    template as each game's store page, onboarding at any stage to build interest, CGT support from backers (Kickstarter
    and Patreon), **a price of up to 10,000 CGT per game** (the owner's number, recorded as decided), in-game purchases
    as DRC-369 assets, and agentic, "vibe coded" creation through an external LLM. **ADR-071:** the Cartridge's profile
    manifest is the store page and its revisions the devlog; four stages that publish themselves when the SDK's checks
    find nothing missing, people only for payouts, large campaigns and reports; licences and in-game items as DRC-369
    (non-fungible only, ADR-031); backing gives the work and never its proceeds, with all-or-nothing Campaign Vaults and
    milestone release, memberships per period until M5.2; agents through ADR-010's MCP server under an agent key, the
    developer signing anything public or costing CGT; all code sandboxed by build hash; and one interim content store
    for the ecosystem rather than ARQADE's own (ADR-050 decision 11). **New:** gap G-17 (no primary sale), U-17
    (backing), P7.12 to P7.15, seven gate units (a tightening, logged). **Built and tested:** `sdk/src/profile.ts`
    (`checkReadiness`, `readyStage`, `priceProblem`), the template, 5 tests; the first run caught a real gap (the
    template's all-zero QOR id passed) and two planted faults (ceiling doubled; sandbox not tied to the build) each
    failed a test. `npm test` 21 of 21, `tsc` clean. **Legal review now covers backing as well as paid chance.**

40. **4 October 2026: the owner accepts ADR-070 and ADR-071 and names the wallet "ARQ Wallet".** In the owner's words:
    "I accept ADR-070 and ADR-071 as written. The game wallet should be called 'ARQ Wallet'". Statuses and indexes
    updated; each record's status says that where it reads Game Vault the name is ARQ Wallet, and neither record's
    decision changed. **The pallet is `pallet-arq-wallet`** (`ArqWallet` in the runtime), its name taken from the
    owner's. Renamed in current documents and code: the guide is `products/arqade/sdk/docs/arq-wallet.md`, the
    module `src/arq-wallet-policy.ts`, the type `ArqWalletPolicy`. ADR-070's filename and the history in items 38 and
    39 and in `GATES.toml`'s change log keep the old words. `arqade.decisions` now has all five of its units met.

41. **4 October 2026: `pallet-arq-wallet` built — ARQ Wallets on chain (ADR-070, P7.11's first slice). On
    `session/arqade-2026-10-04`. Not on the devnet.** Mounted as `ArqWallet` at index 11, `spec_version` 7
    (`transaction_version` stays 2: no existing call changed). Keyless accounts from `PalletId(*b"dmg/arqw")` and the
    Cartridge's `(collection, item)`; the governor is `Nfts::owner` at each call; `create` (at least the existential
    deposit), `set_policy` (tightening at once, loosening pending until `apply_policy` after `loosen_delay`),
    `cancel_pending`, `set_authority`, `set_paused`, `payout` (authority only; max payout, rule version, outcome
    window, each outcome once, epoch budget and count, cap per recipient, nothing owed or the deposit spent; a payout
    that cannot reach its player accrues), `claim`, `expire_accrual`, `schedule_withdrawal` / `execute_withdrawal`
    (delayed, re-checked against what is owed), `prune_outcome`. Every amount checked, overflow refused. **Not built:**
    held prizes for paid rounds (ADR-070 decision 8), a runtime API, the SDK calls. **Placeholders:** the four U-16
    bounds (600, 600, 432,000, 1,000 blocks) in `runtime/src/assets.rs`, pinned by a runtime test as placeholders, and
    the weights (M7.2). **Evidence:** 15 pallet tests and 3 runtime tests; three planted faults (outcome check removed:
    1 failed; owed total ignored: 3 failed; loosening applied at once: 1 failed); `cargo fmt --check` and `clippy
    -D warnings` clean; **143 workspace tests pass with the runtime's wasm built** (125 before). One `clippy` allow,
    `too_many_arguments` on `payout`, with its reason beside it. The gate gains `arqade.wallet-tests`, eleven of them,
    counted by ARQADE's gate and not Alpha's (ADR-050 decision 8; a tightening, logged). **Trap found:** drive X: was
    completely full (0 bytes of 954 GB), and the linker failed with LNK1318/LNK1201 rather than saying so; 48 GB of
    incremental caches under three `target/debug/incremental` directories were removed and the run repeated with
    `CARGO_INCREMENTAL=0`. **To reach the devnet** the runtime must be built **with the `sudo` feature** (the devnet's
    runtime has it; a wasm without it would remove sudo from the devnet on upgrade) and set by the owner's sudo key.

42. **4 October 2026: ARQADE merged to `main`; the chain's CI job is red, and has been since Rust 1.99.** `main`
    fast-forwarded to `e5b5f7c` (six commits) and pushed; `public-main` now tracks `matrix/main`. CI runs on the
    organisation (no longer blocked): QOR ID and Security pass; **the chain job fails at Lints**, as it has on every
    run since 28 September, because `dtolnay/rust-toolchain@stable` installed Rust 1.99.0, whose clippy flags
    `clone_on_copy` in code `#[pallet::event]` and `#[pallet::call]` generate (reproduced locally: 36 hits in
    `pallet-drc369`, 3 in `pallet-validator-set`). Every fix weakens a gated check, so it is written up for the owner:
    `docs/architecture/RUST_TOOLCHAIN.md`, recommending a 1.98.1 pin with a non-failing newest-clippy report. Nothing
    was changed in CI or in code. `archive-7gUg78/` (a GitKraken installer zip) was left uncommitted on purpose.
    **The ARQADE job failed at first**, at setup: the root `.gitignore` excludes every `package-lock.json` and every
    `build/`, so the import had silently left out the arcade's lockfile and `build/sites-vite-plugin.ts`. Both are now
    tracked through documented exceptions (`6a4c566`); a clean copy of the commit installed, type-checked, passed 21
    tests and built, and **CI run `37260506426` passed the ARQADE job**. Trap: an import into this repository must be
    checked with `git status --ignored`, because the root rules drop files without a word.

43. **4 October 2026: the owner chose option A — the chain builds with Rust 1.98.1 (ADR-072).** "go with option A".
    `chain/rust-toolchain.toml` pins 1.98.1 with `rustfmt`, `clippy` and `wasm32v1-none`; the chain, two-validators and
    coverage jobs install 1.98.1 (coverage's `llvm-tools-preview` for that version, since the toolchain file decides the
    version in `chain/`); a new `chain-newest-clippy` job lints with the newest stable on scheduled and manual runs,
    `continue-on-error`, `SKIP_WASM_BUILD`, and no criterion reads it. ADR-072 amends ADR-033 rule 2 for `chain/` only.
    `GATES.toml`: `[ci].quality_gates` names the pin, `[ci].reports` the new job, and an `evidence-rule` change-log
    entry carries the owner's approval. Verified locally: rustup switched `chain/` to 1.98.1 from the file; fmt and
    clippy `-D warnings` clean. **CI run `37260938168` on `main` (`f3a684e`) concluded success: every job passed** —
    chain (fmt, clippy and every test with the wasm built, its first pass since 28 September), QOR ID, launcher,
    ARQADE, coverage and security; two-validators and the newest-clippy report are scheduled-only and were skipped.

44. **4 October 2026: the ARQ Wallet's runtime upgrade rehearsed; the devnet waits on the owner's signature.** Live
    devnet read first: `demiurge` spec_version 6, `Sudo::Key` set. Built `cargo build -p demiurge-node --release
    --features sudo --locked` at `715f0ee`: the runtime is `demiurge_runtime.compact.compressed.wasm`, **516,222 bytes,
    SHA-256 `0db8455c018df88ad8aac04c86016629467b506aef3c203bb582dac8ba81a901`**, copied to the owner's Downloads as
    `demiurge-runtime-v7.compact.compressed.wasm`. **Rehearsal** (`chain/scripts/rehearse-upgrade.mjs`, new): a
    development specification with its genesis `:code` replaced by the devnet's own (2,614,632 bytes, read with
    `state_getStorage(":code")`), one local validator; a transfer and a DRC-369 mint left behind; then
    `sudo.sudoUncheckedWeight(system.setCode(wasm))` exactly as the owner will sign it. **Passed:** spec_version 7,
    spec_name kept, Sudo kept with the same key, the transfer and the asset kept, `ArqWallet` in the metadata, and a new
    ARQ Wallet paid a player exactly 5 CGT. The client logged one decode warning for the upgrade block's events, read
    with the old metadata; expected at an upgrade. Traps: a `--tmp` validator needs `--node-key`. **Next:** the
    owner signs the upgrade on the devnet in polkadot.js Apps; then spec_version is read back.

45. **4 October 2026: the owner upgraded Demiurge Devnet to `spec_version` 7 — the ARQ Wallet is live.** Signed in
    polkadot.js Apps (`Developer → Sudo`, `system.setCode` with weight override, the wallet's own password) with the
    sudo account `5HN6PZA4…Zadn`. **Read back from `wss://rpc.qorsync.dev`:** `demiurge` spec_version 7,
    `transaction_version` 2; `:code` is 516,222 bytes with SHA-256 `0db8455c…a901`, the rehearsed file; `Sudo::Key`
    unchanged; `ArqWallet` at index 11 with 12 calls; finality advancing (#19,792 → #19,798 in 36 s) with both
    validators authoring; readiness 200. The node image (`sha-f836075`) was not rebuilt and need not be: the runtime
    runs as wasm from state. Trap: Apps showed no admin account on the Sudo page until the wallet's accounts appeared
    under Accounts; the wallet asks for its own per-account password, not QOR ID's.

46. **4 October 2026: rounds with held prizes (ADR-070 decision 8) and the SDK's ARQ Wallet module. `spec_version` 8,
    rehearsed; the devnet still runs 7 until the owner signs.** `pallet-arq-wallet` gains `open_round` (authority;
    prize at most the epoch budget, held at once in `Held`, at most `MaxOpenRounds` 64 open), `settle_round` (authority,
    from close to `OutcomeWindow` after; at most `MaxWinners` 64 distinct winners; pays, accrues what cannot reach,
    releases the rest) and `cancel_round` (authority or governor any time, anyone once unsettleable). `available()`
    subtracts the hold, so payouts and withdrawals cannot touch it. Held totals live in their own map, so **no stored
    encoding changed and no migration is needed**; `transaction_version` stays 2. **Evidence:** 21 pallet tests (6 new,
    one pinning the wallet account's bytes beside the SDK's) and 4 runtime tests; three planted faults caught (hold
    ignored: 2 failed; prize limit removed: 1; anyone may cancel: 1); fmt and clippy clean; **150 workspace tests with
    the wasm built** (143 before). Five more tests in `arqade.wallet-tests` (a tightening, logged). **SDK:**
    `sdk/src/arq-wallet.ts` — `arqWalletAccountId`, `arqWalletAddress`, `ss58` (checked against Alice's known address),
    `outcomeId` and `roundId` (BLAKE2b-256, length-prefixed fields, separate domains), `toChainPolicy`, `payoutArgs`;
    4 tests, `npm test` 25 of 25. New dependency `@noble/hashes` 2.4 (ADR-033 rule 2: current). **Upgrade rehearsal**
    against the devnet's live `:code` (516,222 bytes, the v7 the owner set): spec_version 8, Sudo kept, state kept, a
    payout, and a round holding 20 CGT then settling 15 to a winner — passed. The runtime, built with `--features sudo`
    at this commit, is **516,949 bytes, SHA-256 `fbd85089f7095e0604cd2d1aa07e0aecf32035e63e40845b572e72eed7db0074`**,
    in the owner's Downloads as `demiurge-runtime-v8.compact.compressed.wasm`.

47. **4 October 2026: the owner upgraded Demiurge Devnet to `spec_version` 8 — rounds with held prizes are live.**
    Signed in polkadot.js Apps as before. **Read back:** `demiurge` spec_version 8; `:code` 516,949 bytes, SHA-256
    `fbd85089…0074`, the rehearsed file; `Sudo::Key` unchanged; `ArqWallet` at index 11 with 15 calls, `open_round`
    among them; finality advancing (#20,370 → #20,376 in 36 s) with both validators authoring; readiness 200. CI run
    `37269092178` on `75e8ba8` passed every job before the upgrade.

48. **4 October 2026: P7.2 done — ARQADE reads any account's CGT and DRC-369 assets from the devnet.** `lib/account.ts`:
    `System::Account` from storage (key `twox128(System) ++ twox128(Account) ++ blake2_128_concat(id)`) and the runtime
    API `Drc369Api_assets_of`, both at the finalized head, decoded by a bounded SCALE reader; spendable is
    `free − max(frozen − reserved, existential deposit)`, as `pallet-balances` computes with `Preserve`. The SDK's
    `ss58Decode` refuses another network's prefix and a broken checksum before any read. `/api/chain/account` returns
    amounts as decimal strings (400 for a refused address, 503 for an unreadable chain); `app/account-lookup.tsx` on the
    QOR Identity screen shows them grouped, labelled test CGT, and keeps the last answer marked stale on failure.
    **Evidence:** test vectors captured from a local `--dev` node at `spec_version` 8 after two named mints (one a
    remix, one with a non-ASCII name, both commit kinds), where `@polkadot/api` decoded the same balance; the reader
    against the live devnet matched `@polkadot/api` on `5DMPEX…qLxK` (100,000 test CGT, 0 assets); the route answered
    in the local workerd runtime; headless Chrome drove the panel (a lookup showing 99,900 / 100,000; a refused checksum).
    `npm test` 31 of 31, type check and build clean; lint unchanged at 13 inherited errors. Trap: `npm run build` fails
    with EPERM on `dist` while `npm start` (workerd) is running. **The live Sites website was not redeployed.**

49. **5 October 2026: P7.1 done — ARQADE renamed on screen, its inventory written, its lint a gate.** The 13 React
    Compiler errors fixed without changing behaviour: latest-callback refs (`callback`, `doneRef`, `cb`, the Orbital
    action, Rift Survivor's props) written in an effect after render; the per-game reset as React's previous-value
    pattern; the first "nothing lit" in Synapse scheduled with its timers; three mount-time reads of browser storage and
    the arenas' first poll deferred one tick; Rift Survivor's run created by `useState`; the home link through
    `next/link`; an unused import removed. Lint 0 errors, 4 warnings (three effect dependencies, one expression), and
    now a CI gate (a tightening, logged). **Verified in headless Chrome** against the built worker: each of the four solo
    games opened, began and played with no page exception or console error (a control run showed Synapse's synthetic
    click was the check's fault, and a real key press advanced its round). **Renamed** "ARQADE · powered by Demiurge":
    title, brand, intro, notice, terminal, ledger file, exported games; `localStorage` keys and the `navigate_demiurge`
    tool name kept. **Two more false displays fixed:** the sidebar's "Demiurge Protocol · Chain connection pending",
    which read nothing, and the terminal's `chain`, which printed "block #undefined" after the devnet reader's fields
    changed. **Inventory:** `products/arqade/INTEGRATION_INVENTORY.md`, every control with its source, permission,
    persistence, failure and verification; it and `IMPLEMENTATION.md` are now in `docs/README.md`. `npm test` 31 of 31,
    type check and build clean.

50. **5 October 2026: P7.3 built — QOR ID signs people in to ARQADE (ADR-043 carried out; ADR-073). On branch
    `session/p7.3-qor-sign-in`, not merged: merging deploys QOR ID.** QOR ID: `handlers/oauth.rs` with `/oauth/authorize`
    (its own page; unknown app or unregistered redirect answered there, never redirected), `/oauth/token` (S256 PKCE
    always; client secret by SHA-256 hash; codes single-use in 60 s; refresh tokens rotate, reuse ends the session),
    `/oauth/userinfo`, `/oauth/revoke`; `QOR_OAUTH_CLIENTS` registry validated at start; sessions and tokens carry the
    app (`client_id`, `cid`, `refresh_jti`/`jti`), serde defaults so nothing stored breaks; the password check shared
    with `/api/v1/auth/login` (`authenticate_password`); `/api/v1/auth/refresh` refuses an app's token; the sign-in page's
    CSP `form-action` adds the app's origin, because browsers apply it to the post-submit redirect. **143 tests** (7 new)
    against Postgres 16 and Redis 7.4 in throwaway containers `qor-p73-pg`/`qor-p73-redis`, fmt and clippy clean; four
    planted faults caught; **the log check drives the whole flow** and caught a planted log of the code. ARQADE:
    `lib/qor-session.ts`, `/api/auth/{login,callback,me,logout}`, migration `0001` (`qor_logins`, `qor_sessions`, the
    session keyed by the cookie's SHA-256), `app/qor-panel.tsx` replacing the static rows; 9 tests, two planted faults
    caught; 40 in all. **End to end in Chrome** against a local QOR ID (port 8099) and the local worker: the button, QOR
    ID's page "Sign in to ARQADE · QOR ID", back signed in as `arqtester#0001`, `document.cookie` empty (HttpOnly),
    sign-out revoking at QOR ID. A bug that check found: the return hash must be `#qor-identity`. **To go live:** merge;
    set Railway's `QOR_OAUTH_CLIENTS` for ARQADE's production callback with the hash of a new secret; set
    `QOR_CLIENT_ID`, `QOR_CLIENT_SECRET`, `QOR_REDIRECT_URI` in the site's host; apply ARQADE's migration `0001`;
    redeploy the site.

51. **5 October 2026: P7.3 live on QOR ID's side; ARQADE moved to Vercel (ADR-074). On branch
    `session/arqade-vercel`, not merged then (merged as PR #2).** The owner merged P7.3 (PR #1, `788d3ab`; QOR ID redeployed, `/oauth/authorize`
    answering) and set Railway's `QOR_OAUTH_CLIENTS` for ARQADE (read back: the live page is titled "Sign in to ARQADE
    · QOR ID"; an unknown app still gets 400). The secret is in the session scratchpad, never in the repository. Sites
    then could not be published or checked from here (every route behind its sign-in wall; it moved itself to
    `demiurge-arcade.andyithink.chatgpt.site`), so the owner chose **Vercel, with v0 for screens**. Done: vinext, Vite,
    wrangler, drizzle, the Sites plugin, `.openai/`, `app/chatgpt-auth.ts` and `tests/worker.smoke.mjs` removed; standard
    Next.js 16; `lib/db.ts` (the D1-shaped statement interface over `pg`, `?` numbered, batches as transactions; PGlite in
    tests); `db/migrations/0001_arcade.sql`; `scripts/migrate.mjs` (production builds only, advisory lock); players are QOR
    ID accounts (id = SHA-256 of `sub`); the arcade accepts QOR ID's answer for 30 s (`qor_sessions.checked`); sign-in
    errors name their cause; origin checks use Host, redirects are relative. **41 tests** (two planted faults caught),
    type check, lint (0 errors), `next build`; migrations twice on Postgres 16; **Chrome end to end** against Postgres and
    a local QOR ID: signed in as `arqtester#0001`, a match created by that player, 401 after sign-out. The Vercel
    connector cannot create projects (403), so **the owner creates the project** in Vercel's dashboard: import
    `QOR-MATRIX/demiurge-chain`, root `products/arqade`, add Neon. Then: QOR ID's registry gets the Vercel callback, and
    Vercel gets `QOR_CLIENT_ID`, `QOR_CLIENT_SECRET`, `QOR_REDIRECT_URI`. **An incident:** a `taskkill /IM node.exe`
    and later a kill of whatever listened on port 3000 stopped Docker Desktop twice (port 3000 is a container's); the
    owner restarted it and the `vyb-*` containers came back healthy. Kill only recorded PIDs of processes you started.
    **Later the same day:** the owner created the Vercel project `qor-arqade` (team Astra Matrix), **live at
    `https://qor-arqade-tau.vercel.app`**: the page loads and `/api/chain` reads Demiurge Devnet (genesis checked). Its
    first build failed because Vercel saw the root `turbo.json` and installed the frozen workspace;
    `products/arqade/vercel.json` (PR #3) fixes install and build to the folder. Vercel at first copied the repository into
    a new private `QOR-MATRIX/arqade`; the project now builds from `demiurge-chain`, and deleting the copy is the owner's
    call. With no database or settings the site now says so (not configured, sign in, being connected) instead of a
    generic outage. Still to do: Neon, the Railway callback, the three `QOR_*` settings in Vercel.

52. **5 October 2026: ADR-075, one name per QOR ID, no `#0001` (the owner's rule). On branch
    `session/unique-usernames`, not merged then (merged as PR #5): merging redeploys QOR ID and runs migration 019 on the live database.**
    The owner's three ARQADE sign-ins as `Godmode` were refused by QOR ID's password check (Railway HTTP log, 21:10,
    `POST /oauth/authorize` 401); every sign-up path for people always refused a taken name, so this was the password or
    the account's state, not a shared name (I said otherwise first, and corrected it). QOR ID: `User::qor_id()` and every
    response give the name alone; `find_by_username` has no `ORDER BY discriminator`; `generate_discriminator` removed;
    agent sign-up refuses a taken name; an insert race maps `users_username_unique` to "Username already taken";
    `QorId` reads an old `name#0001` and drops the number; profile returns `renamed_from`; start-up logs the renamed
    count. Migration 019 (oldest keeps the name, later `name_N`, `renamed_from`, discriminator 1, unique index) checked
    on a database holding shared names. 145 tests, fmt, clippy; a planted `#0001` failed a test. Launcher: shows the
    name alone, its name rules now match QOR ID's (no hyphen), a tutorial finished under `name#0001` stays finished;
    196 tests, clippy, build. ARQADE's tests use plain names (41). **A rename flow is not built**: only agent accounts
    could have shared a name; read the count logged at deploy.

53. **5 October 2026: QOR ID's account page; ADR-076 proposed (P7.4). On branch `session/account-page`, not merged then (merged as PR #7).**
    The owner was locked out because their account was created from a key (the launcher signs in with
    `keypair-login`), so it had no password and no email, and could neither reset nor change one; a password was set by
    hand in the database (the owner ran the SQL; a script made the hash locally) and the stored name had a stray space,
    fixed the same way. Built: `handlers/account.rs`, `GET /account` (two forms, each needing the current password),
    `POST /account/password`, `POST /account/email` (reuses `request_email_change`; its limits are now said on the page,
    not "Something went wrong"), `POST /api/v1/profile/password`; a change ends every session and cancels a pending
    email change. All three joined the log check. 149 tests, clippy, fmt; a planted fault (no sign-out) failed a test.
    ARQADE's identity card links to the page (`accountUrl` from `/api/auth/me`) and shows the QOR ID once.
    **ADR-076, accepted by the owner the same day with a cap of 100,000 CGT per request**: a signed `qor://pay` link, the launcher's host dialog, the result read from finalised blocks,
    devnet only. **Also:** the owner pasted a 24-word recovery phrase into the
    chat; whatever account it controls must be treated as exposed and replaced (OWNER.md).

54. **5 October 2026: P7.4 built, tips through the QOR Launcher (ADR-076 with ADR-077). On branch
    `session/p7.4-qor-pay`, not merged then (merged as PR #8).** ADR-077 corrects ADR-076 from the launcher's code: the host dialog approves
    (the owner chose it over Windows Hello, which no signature has used since ADR-056), no fee exists to show (OPEN-4),
    "devnet only" is a genesis check. The owner chose tips to a game's creator as the first use. **Launcher**: `src/pay.rs`
    (parse and check a `qor://pay` link: ARQADE's Ed25519 key in `KNOWN_APPS`, genesis, the 100,000 CGT cap, account,
    label, id, expiry within fifteen minutes, paid-once in `paid-requests.json`), `src/chain/pay.rs` (`pay_request`:
    genesis re-checked, `batch_all` of `transfer_keep_alive` and `remark_with_event`, through `sign_and_finalise`, prompt
    built from checked values), `lib.rs` (deep-link and single-instance plugins, `handle_pay_link`, a native outcome
    message), `QorError::PaymentRefused`. 204 tests (8 new); a planted fault (no signature check) failed one. **ARQADE**:
    `lib/pay.ts` (request in the launcher's field order, Ed25519 by WebCrypto, `paymentIn`: the remark's event and the
    transfer from its sender, to the recipient, of the amount, in one extrinsic that did not fail), `lib/pay-chain.ts`
    (`@polkadot/api` over HTTP, genesis-checked; **`@polkadot/api` renames the event field `hash` to `hash_`**, which
    the first real-chain run caught), `lib/tips.ts`, `/api/pay/tip`, `/api/pay/[id]`, `app/tip-panel.tsx`, migration
    `0002_tips`. 47 tests; a planted fault (no same-extrinsic check) failed one. **The two are pinned together**: a
    link made by ARQADE's code with a test key is a constant in the launcher's tests, and ARQADE's test checks it is
    still that link. ARQADE's signing key was generated locally; its public half is in `KNOWN_APPS`, its private half in
    the session scratchpad until the owner sets `QOR_PAY_SIGNING_KEY` in Vercel. **To go live**: merge; Vercel
    `QOR_PAY_SIGNING_KEY` and `QOR_PAY_TIP_ADDRESS`; build and install the launcher (the installer registers `qor://`).

55. **6 October 2026: ADR-078 accepted, with a welcome grant of 100 CGT; U-18 opened.** One level and XP per QOR ID
    across every app, granted only by QOR ID or an app's server for checked tasks, each once (level L at 50·L·(L+1) XP;
    tutorial 50, email 25, key 25 make level 1); a level bubble and XP bar; cosmetic unlocks (ring styles colour and
    pattern only until the owner decides on glow, which the design system forbids outside `src/qfx/ceremony/`); ARQADE
    inside the launcher; 100 CGT once per new QOR ID when tutorial, email and key are confirmed, from an uncapped
    Welcome account, devnet test CGT first; three accounts per network address (a keyed hash kept 30 days); an abuse
    watcher that flags. **The owner set 100 knowing an account at exactly the existential deposit has nothing to spend.**
    Roadmap P7.16 to P7.18. Also on 5 and 6 October: tips worked end to end (the owner tipped from launcher 0.1.7 after
    funding the vault's own account: the 100,000 test CGT sit in `5DMPEX…qLxK`, which is not this vault's account);
    "The Arcade" renamed "Play Now" (PR #9); the launcher's browser checks stopped giving up before the page existed
    (the flaky "did not expose a page to inspect"). The owner wants CGT to become exchangeable; no current record says
    so yet, and it was offered as a record of its own with a legal review.

56. **6 October 2026: ADR-078's first build, levels and tasks. On branch `session/levels`, not merged then (merged as PR #11).** QOR ID:
    migration 020 (`progress_events` one row per account and task; `welcome_grants` unique per account, email and chain
    account, 100 CGT in Sparks as text; `signup_addresses` keyed hashes); `handlers/progress.rs` (tasks and XP, level
    50·L·(L+1), unlocks named, `award` once and records an owed grant when tutorial, verified email and linked key are all
    there; `summary`; `/api/v1/profile/progress`, `/api/v1/profile/progress/tutorial`; `/oauth/progress` for an app's
    own tasks with its secret and a live token issued to it; `register_limited`/`keypair_register_limited` wrap sign-up
    with the three-per-address limit, the address from Railway's **`X-Real-IP`** (Railway's documentation;
    `X-Forwarded-For` is not trusted), as an HMAC). Hooks: verify-email in both verification paths, link-key in
    `link_verified_key` and key sign-up, sign-in-arqade in `authorize_submit`; userinfo carries `progress`. 157 tests;
    the log check drives the three routes; a planted fault (a grant without a verified email) failed a test. Launcher:
    `components/chrome/Level.tsx` (bubble, XP bar), `qor_progress`, `qor_tutorial_done`, the tutorial reports itself.
    ARQADE: `reportTask` (`lib/qor-session.ts`), a first match (players.first_match_reported, migration 0003) and a
    first paid tip reported; the bubble and the level row. 48 tests. **Checked in Chrome** against a local QOR ID and
    Postgres: signing in to ARQADE granted 10 XP; the bubble showed 0 and the card "Level 0 · 10/100 XP · Next: Ring
    style: Ember". **Not built yet:** drawing ring styles and themes as unlocks, paying owed grants, ARQADE inside the
    launcher, the abuse watcher.
    **Then:** migration 021 credits accounts created before levels with the verify-email and link-key XP they had
    already earned (checked on sample accounts, twice, no change the second time); launcher 0.1.8 built
    (`QOR Launcher_0.1.8_x64-setup.exe`) to show the level.

57. **6 October 2026: avatars (ADR-079) and effects allowed (ADR-080). On branch `session/adr-079`, not merged then (merged as PR #13).**
    ADR-079 accepted ("I accept ADR-07.", replying to it). QOR ID: migration 022 (`avatars` with a still first frame for
    GIFs, `avatar_reports`, `avatar_removals`); `avatar_image.rs` (the `image` crate, pure Rust, under limits: PNG, JPEG,
    WebP, GIF up to 4 MB, orientation applied, centre square, 256 x 256, re-encoded PNG or GIF, at most 120 frames and
    2 MB; metadata does not survive, tested with a planted text chunk); `handlers/avatar.rs` (upload with a 4 MB body
    limit, delete, serve by content hash with immutable caching, `nosniff`, a sandbox CSP and cross-origin resource
    policy, the still route, report once per reporter, the owner's queue and removal, which clears every account showing
    it, tells them and is audited); profile and userinfo carry `avatar_url`. 166 tests; the log check drives upload,
    serve, report, the queue and removal. Launcher: `components/chrome/Avatar.tsx` (picture or letter, ring, level
    bubble, first frame under reduce motion), `qor_choose_avatar`, `qor_remove_avatar`, `qor_avatar` (the offline copy),
    the rail at 34 px and the Nexus header at 64 px. **The header first grew** and pushed three Nexus tiles under the
    fold; `check-readability.mjs` caught it (50 failures) and the header was brought back to its height. **ADR-080**, the
    owner's: glow, neon, gradients, shadows, canvas, pointer-reactive light and looping animation are allowed everywhere;
    the rule is distinctiveness, never a generic template; reduce motion, readability as painted, and theme tokens and
    scales still bind. `check-design.mjs` dropped its five effect rules and the QFX and ceremony exemptions (logged in
    GATES.toml's change log as the owner's loosening). Rings glow from level 1. **Next, asked for by the owner:** a library
    of reactive QFX backdrops (Drift, Aurora, Nebula, Lattice, Starfield, Liquid), combinable pointer effects, presets,
    and backdrops that evolve with the level; and ARQADE's avatars.
    *Merged to `main` the same day (PR #13, `9f8a818`), as were items 51 to 56 (PRs #2 to #12).*

58. **6 October 2026: documentation brought up to date with the tree after the IBM Bob audit.** On branch
    `session/docs-current`. **What was stale:** the status sections of the current documents had been written
    before the work of ADR-063 to ADR-080 — the public repository and CI on GitHub Actions, QOR ID and the devnet on
    Railway, ARQADE on Vercel, sign-in for apps, unique names, tips, levels and avatars — and still said so. This
    file's §1 called CI blocked by a billing lock, QOR ID configured but not deployed, the launcher at 91 host tests
    and production offline; its opening called item 32 half done, `id.qorsync.dev` down and the work on
    `public-main`; items 51 to 57 said "on a branch, not merged". `SYSTEMS.md` called ARQADE not yet live on Vercel,
    `SECURITY.md` placed the nine credentials in the tree, and QOR ID's `main.rs` module comment still described
    `username#discriminator` names. **What was fixed:** all current documents were checked against the tree at
    `9f8a818` and against live state, and corrected. Here: the opening and the reading list; §1 rewritten, with rows
    for ARQADE, the decisions and what is not live; §2's chain identity, endpoint and a list of current gaps; §3's
    R-1, R-2, credentials, sessions and new surfaces; §4.0 (L1.4 partly exercised; L1.6, L1.7 and P7.4 ticked in `DIRECTION.md` with their evidence; the
    pull-request row removed; the owner's open decisions added); §4 items 1 to 11; §5's traps; §6 and §7. No
    behaviour or ADR decision changed, and no gate criterion's requirement changed. **Next:** the owner re-runs the audit against `main` on GitHub once this
    work is merged.
    **Owner confirmations, 6 October 2026** (recorded here as `GATES.toml`'s `kinds.check` evidence rule allows):
    "Release gates file confirmed" — the two reworded criterion texts above (`alpha.no-dependency-patching`,
    `public-release.single-provider-revisited`) stand; "Mark Dependency-patching gate done" —
    `alpha.no-dependency-patching` is `done = true`, evidence main's run `37496555161`, whose security step
    passed; "Allowed browser origins confirmed" — `server.allowed_origins` stays unset in production and is
    never set from the environment (it stops QOR ID starting; `services/qor-auth/DEPLOY-RAILWAY.md`).

## 5. Traps, so nobody re-learns them

**Launcher checks**
- `npm run check` runs nine checks: design, accessibility, the gates view, the Projects view, the Inventory
  view, the Market view (`check-market-view.mjs`), the Gate and Windows Hello (`check-vault-gate.mjs`), QFX
  contrast and readability. It needs `npm run build` first, because eight of them serve `dist`, and a
  Chromium-family browser (set `BROWSER_PATH` for one that is not Edge or Chrome in the usual places).
  Readability takes about four minutes; `READABILITY_THEMES=numen` runs one theme while iterating.
- **`?.` in a check's click turns a missing element into a silent pass.**
  `document.querySelector(sel)?.click()` does nothing when `sel` matches nothing, and the checks after it then
  read an absent dialog and fail with no hint of why — or, worse, read nothing and pass. A Sell block shipped
  that way on 2026-09-23: it opened the menu of an asset the trade block before it had already traded away.
  **In a check, an element that must be there is addressed without `?.`**, so the run throws at the line that
  is wrong. Keep `?.` for reading something that is legitimately optional.
- **A fixed sleep in a view check is a flake waiting for a loaded machine.** `check-vault-gate.mjs` failed one
  run in three under `npm run check` (2026-09-28) and never alone: a click or a field was addressed while the
  previous screen was still drawn. Wait for the target (`until`), and for a headless page, send
  `Page.bringToFront` and `Emulation.setFocusEmulationEnabled` after every reload, or it may draw no frames
  and a framer-motion cross-fade never finishes.
- **A later block runs against the state the earlier ones left.** `check-inventory-view.mjs` mints, makes
  permanent, trades and then sells, all in one page. After the trade only one asset is held, so a block added
  at the end cannot use the fixtures the earlier blocks used. Read what the previous block left before
  choosing an asset to act on.
- **A check file that does not parse reports nothing, and looks like nothing ran.** A duplicate `const` made
  Node refuse `check-inventory-view.mjs` entirely on 2026-09-23; the `GATES.toml` entry beside it had already
  been written with a check count worked out by arithmetic. **A count in `GATES.toml` is evidence and comes
  from a run**, never from adding up the checks you think you wrote.
- **A pipeline's exit code is the last command's, not npm's.** `npm run check | grep RESULT` exited 0 while
  `check-readability.mjs` had **crashed**, because grep succeeded. The crash was invisible: its per-check
  lines were filtered out and its `RESULT` line never came. Read the whole output, or redirect to a file and
  check the exit code of the command you care about.
- **A runtime upgrade beats purging a development chain.** `sudo.sudoUncheckedWeight(system.setCode(wasm))`
  against a `--dev` node keeps every account, asset and block; the node binary need not match, because the
  runtime executes as wasm from state. Check the chain type is `Development` first, and read `spec_version`
  back afterwards. Restarting the node with the same `--base-path` and no upgrade serves the OLD runtime,
  which is also how the previously recorded metadata size was confirmed to the byte.
- **A running node holds its own binary open on Windows.** `cargo build --release` fails with "Access is
  denied" while the development node is running; stop it first. To measure both runtime shapes, copy the
  first binary aside before building the second, because they are the same path.
- **A token check cannot see a painting mistake.** `check-contrast.mjs` does the colour arithmetic and assumes
  the layers are canvas, scrim, interface. On 2026-09-22 they were not — the scrim was painted over most of
  the interface — and every check passed while the launcher was unreadable. `check-readability.mjs` measures
  the painted result. Keep the interface in `.qfx-interface`; anything rendered outside it, beside the canvas
  and the scrim, paints beneath them.
- **A headless screenshot can wait for a frame that never comes.** `Page.captureScreenshot` hung for 60 s
  and killed two full runs of `check-readability.mjs` on the first theme, after the check grew three more
  screens. It is the test browser, not the app: `shoot()` now brings the page to front, restores focus
  emulation and asks once more, and the run says how often it had to. A second stall still fails the run.
- **Headless Edge needs `--disable-backgrounding-occluded-windows`, `--disable-renderer-backgrounding` and
  `--disable-background-timer-throttling` for any long run.** Without them the page can be marked hidden,
  stop producing frames, and a screenshot waits for ever; `check-readability.mjs` stalled that way once.
- **A class name that names nothing fails silently.** The Projects surface shipped with `button`,
  `button-quiet` and `input`, none of which exists in `qor.css`, and rendered browser defaults while every
  check passed. The stylesheet's classes are `btn btn-primary`, `btn btn-ghost` and `field`. The Projects
  and Inventory checks now fail on any class the stylesheet does not define (lucide's icon markers
  excepted).
- **`check-accessibility.mjs`'s case "stored off, then live: the backdrop starts" used to fail about half
  the time**, with 200 to 800 ms frames after that boot. Since the interface was lifted above the canvas and
  the scrim (`1f5f4ed`) it has passed 7 runs of 7. The likely cause was the interface being repainted beneath
  every backdrop frame; that is inferred, not measured. If it returns, **do not lower the threshold**; that
  is a loosening.
- **The two live tests share Alice.** They run in parallel, and two fundings that read her next nonce at
  once collide ("priority is too low"). `fund_from_alice` takes a lock; a new live test that funds must use
  it.
- **The repository ignores every `package-lock.json` except the launcher's**, which is tracked since
  2026-09-29 because CI installs it with `npm ci` and all of its dependencies are ranges. Woodpecker's
  second run failed at `npm ci` for want of it.
  `chain/scripts/package.json` pins its two direct dependencies exactly and is installed with `npm install`,
  not `npm ci`.

**Chain**
- **A `sp_api::decl_runtime_apis!` trait's runtime-side module is snake-cased with digits split:**
  `Drc369Api`'s is `runtime_decl_for_drc_369_api`, and its trait is `Drc369ApiV1`.
- **A mint on a development chain needs about 1,100 CGT free** at the placeholder deposits: 500 for the
  singles collection, 400 for the asset, 100 plus 0.625 a byte for the name, and the existential deposit.
  `chain/scripts/dev-fund.mjs` sends 10,000 by default.
- **Only `pallet-drc369` can create an asset.** Of `pallet-nfts`'s calls the base call filter lets through
  only `transfer`, `approve_transfer`, `cancel_approval` and `clear_all_transfer_approvals`, and its
  `CreateOrigin` refuses everyone. A test that dispatches any other `Nfts` call gets `CallFiltered`.
- **CI runs `npm run check` itself, not a list of scripts** (since `7a3c8ea`). It used to name three, and
  two checks added later were never in the workflow. Add a check to `package.json` and CI has it.
- **The QFX cases in `check-accessibility.mjs` count frame callbacks.** "Still" means no callback is
  scheduled, which a screenshot cannot show. A positive control proves the backdrop is drawing first;
  without it the still cases would pass on a dead canvas. Headless Edge gets a WebGL2 context even under
  `--disable-gpu`.

**Products (P1 to P6)**
- **Build Qontrol's helper before its tests:** `cargo build --manifest-path tools/qor-launcher/qontrol-git/Cargo.toml`.
  Without it the tests that commit skip and say so; with `--features qontrol-no-skips`, as CI and the
  `qontrol` gate's suite run them, they fail. A skipped test reports ok, which is why the feature exists.
- **The helper is found through `QONTROL_GIT_BIN`, beside the launcher executable, or in
  `qontrol-git/target/`.** Since P1.2 the installer bundles it (`src-tauri/tauri.bundle.conf.json`,
  `externalBin` `binaries/qontrol-git`).
- **Write "Accepted" plainly at the start of an ADR index's Status cell.** The gates parser does not strip
  Markdown, so a cell reading `**Accepted**` counts as not accepted. ADR-045's row is written that way; no gate
  counts it today. Stripping emphasis in the parser would widen what counts as met, which is loosening.
- **A product item is `P<n>.<m>` under `#### P<n>:`** in `DIRECTION.md`. The launcher's parser has read
  `P` headings only since `d4be885`; before it, every product item would have read as unmeasurable.
- **Product gates may count chain items; chain gates never count product items.** They are appended after
  Public Release, because a `gate` criterion resolves against earlier gates only.
- **`check-design.mjs` has no effect rules and no exemptions since ADR-080** (6 October 2026). Glow, canvas,
  pointer-reactive light and looping animation are allowed on every surface; the check enforces theme colour
  tokens, the type scale and the tracking scale only. Reduce motion and readability as painted still bind, through
  `check-accessibility.mjs` and `check-readability.mjs`.
- **Ambience Off hides the canvas and the scrim in one rule.** Hiding only the scrim leaves a frame on
  screen with nothing over it, which was layer one's first defect.

**The chain (`chain/`, M3)**
- `chain/` is the only chain (ADR-013, ADR-032). `framework/` was retired and deleted at M3.5.
- **The pin is `polkadot-sdk = "=2606.1.0"`**, exact, for `polkadot-stable2606-1` (ADR-022). Resolving the
  workspace on 2026-09-17 locked 1,559 packages and reported that **2606.2.0, 2606.3.0 and 2606.4.0 now
  exist**. **The owner's decision on 2026-09-17: stay on 2606.1.0**, noted and deferred rather than
  stale. Moving is its own task with its own verification pass under ADR-033 rule 3, and mid-M3 is the
  wrong moment; ADR-022 already declined 2606-2 once. **A move is proposed once the node and
  `pallet-validator-set` are in and the chain produces blocks under a governance-chosen validator set,
  not before.** Also recorded in `chain/README.md`, where someone building the chain will see it.
- **`SKIP_WASM_BUILD=1` gives a fast compile of the Rust**, without building the runtime's wasm. Use it
  while iterating; never use it to claim the runtime builds. A full release build takes about four minutes.
- **The wasm target is `wasm32v1-none`**, installed on 2026-09-17. The SDK builds with
  `wasm32-unknown-unknown` if it is the only one present, and warns that the newer target is preferred;
  switching targets changes the runtime blob, so it was settled before anything depended on the blob.
  Changing it again means `cargo clean` first, as the SDK's own warning says.
- **No transaction payment, no issuance, no genesis allocation.** Fee classes and burn shares are OPEN-4,
  the issuance rate OPEN-1, the genesis split OPEN-2. A `WeightToFee` is exactly the kind of value
  AGENTS.md §5 forbids inventing, so the runtime has no fee pallet yet. The custom chain charges no fee
  either, so the devnet loses nothing.
- **`pallet-sudo` is behind the `sudo` feature** (ADR-037), on for development and test networks and off
  for mainnet, so removal is a build configuration and not a code edit under pressure.

**R-1 on the new chain: closed as met by standard behaviour (ADR-038)**
- Writing the acceptance tests measured it. The forgery the custom chain accepted (R = the identity
  point, s = 0) is **refused by Sr25519 on 64 of 64 messages** for both small-order keys, and **accepted
  by Ed25519 on 64 of 64**.
- **It is not the custom chain's defect repeated.** `sp-core` verifies Ed25519 with `ed25519-zebra`,
  which implements ZIP-215: a precisely specified rule chosen so that every node agrees on validity.
  Consensus needs that agreement more than strictness. Changing it means departing from the SDK's
  standard verification, which ADR-013 does not allow without a written reason.
- **What it costs:** an account whose address is a small-order Ed25519 point can be spent from by anyone.
  Nobody holds a secret for such an address, so nothing is taken from a person; funds sent there are lost
  to whoever claims them, as with any unowned address.
- **ADR-023 chose Sr25519 for account keys**, so the scheme this chain actually uses is not exposed.
- **Decided on 2026-09-18 (ADR-038): the runtime is not restricted to Sr25519 signatures.** Doing so
  would own a consensus-critical divergence permanently, in exchange for closing a hole that harms
  nobody, and ADR-001 names consensus and key handling as where not to be clever. R-1 is closed as met by
  standard behaviour, not left half-met.
- **ADR-023 therefore stays a convention, not an enforced rule.** Account keys are Sr25519 by decision and
  by what the launcher generates, not because the runtime refuses anything else.
- Three tests pin all of this, including one that **fails if the SDK ever becomes stricter**, so the day
  that changes is the day someone revisits R-1 rather than never noticing.

**Two validators, by script rather than by cargo test**
- `chain/scripts/check-two-validators.mjs` starts two real nodes on the `local` specification and checks
  13 things: they find each other, both author, they hold the same block at a common height, GRANDPA
  finalises, they agree on what is finalised, and a killed validator restarts, catches up and agrees
  about the blocks made while it was away. Run on 2026-09-18, 13 of 13.
- **It needs a release build first** (`cargo build -p demiurge-node --release --features sudo`) and takes
  about two minutes, which is why it is not in `cargo test`. Criterion `alpha.multi-validator` reads it.
- **The owner accepted that run as evidence on 2026-09-18**, so `alpha.multi-validator` is met, and it was
  re-run on 2026-09-20 after ADR-041 changed the address type. **It is a CI job of its own since
  2026-09-20** (`two-validators`, `[ci].quality_gates`), which fails the run, so nothing sits unwatched
  between two manual runs again. It is a separate job because it needs the release build and the quality
  gates do not. It runs on the nightly schedule and by hand (`ci.yml`), and carries `timeout-minutes: 90`
  rather than a guessed number.
- **A validator will not generate its own network key**, so both nodes are given `--node-key`. Without
  one a validator exits with `NetworkKeyNotFound`.
- **The first validator runs with `--force-authoring`, deliberately.** A node with no peers stops
  authoring, which is Substrate refusing to build a one-sided chain while it might be partitioned. That
  is correct behaviour and it made an earlier version of this check fail when the second validator was
  killed. Forcing it is what lets the chain advance while the other node is away, which is the only way
  catch-up can be tested.
- **Finality stops while one of two validators is down**, because GRANDPA needs more than two thirds of
  the voters. The chain keeps producing blocks and resumes finalising when the other returns. That is
  safety, not a fault.

**pallet-validator-set (ADR-020)**
- **A governance action must not be able to stop the chain.** The validator set is what authors blocks, and
  a halted chain needs a new genesis rather than a governance fix. Three redundant guards: the
  configuration (`MinValidators >= 1`, refused at startup), every call (nothing may take the set below the
  minimum), and the session boundary (`new_session` answers `None` rather than an empty set, keeping the
  previous authors and emitting `PreviousSetKept`).
- **If `PreviousSetKept` is ever emitted, treat it as an incident.** It means a guard above the session
  boundary failed and only the failsafe stood between the chain and a halt.
- **Each guard was verified by deleting it**, not by reading it: removing the session-boundary check fails
  three tests, removing the per-call bound fails four. Doing that found a real gap — the sequence test
  never reached the removal of the *last* validator — which is now covered and commented so it is not
  reintroduced.
- **An empty genesis is allowed**, following `pallet-collator-selection`, which ADR-020 says to mirror. It
  yields a chain with no authorities that never authors block one, which is immediate and unmissable; the
  danger guarded here is a *running* chain governed into a halt.
- **On the mainnet shape there is no path to the governance origin yet.** It is `EnsureRoot`, and root has
  no caller without `pallet-sudo`. That is correct rather than an oversight: ADR-021's collective is not
  built. A mainnet runtime therefore cannot change its validator set, and that is recorded rather than
  papered over with a weaker origin.

**A `cargo tree -i` query without an exact version answers about one copy and reads like both**

- **This is how a wrong claim survived in four files.** `SECURITY.md`, `HANDOFF.md`, `GATES.toml` and
  `ci.yml` all said none of `chain/`'s ten advisories was reachable through a normal dependency edge.
  Four of them are. The query behind the claim was `cargo tree -i -e normal <crate>`, run **per crate**
  and not **per crate version**.
- **A lockfile resolves several versions of the same crate at once.** `chain/Cargo.lock` holds two
  `hickory-proto` (0.24.4 and 0.25.2) and two `tracing-subscriber` (0.2.25 and 0.3.19). An advisory names
  one of them. `cargo tree -i` given a bare name answers about whichever it picks, and prints no warning
  that the other exists, so an answer about the unreachable copy reads as though it covered the compiled
  one too.
- **Query by exact version, always:** `cargo tree --workspace -i <crate>@<version> --target all`.
  `--target all` covers platform-gated edges, and the default edge kinds already include build and dev,
  so a dev-dependency counts as compiled — which is what you want when asking whether something is built.
- **`-e normal` was the second half of the mistake.** It excludes build and dev edges, so a crate that is
  genuinely compiled can answer "not reachable". Use it to ask a narrower question deliberately, never to
  ask "is this compiled".
- **Confirm a negative before trusting it.** A crate the query cannot reach is in the lockfile because a
  lockfile resolves the union of every optional dependency in the graph. Repeat the query with
  `--all-features`; if it is still unreachable, no feature this workspace can enable selects it.
- **`cargo audit` lists the version.** Read it from there rather than from the crate name in the advisory
  title, which is where the two versions blur together.

**Whenever scope moves between milestones, check gate coverage**
- Twice now, work moved out of an item has landed in a milestone whose gate listed only the items that
  existed when that gate was written, so the moved work would have been counted by **no gate at all**:
  M6.4 and M6.5 when M3.1 was narrowed, and M4.5, M5.6 and M6.6 when M3.3 was. A scope decision must not
  become a gating decision by omission.
- The check, run from the repository root:

  ```bash
  python - <<'PY'
  import tomllib, re, io
  d = tomllib.load(open('docs/GATES.toml','rb'))
  counted = set()
  for g in d['gates']:
      for c in g.get('criteria', []):
          if c.get('kind') == 'roadmap':
              counted.update(c.get('items', []))
  text = io.open('docs/DIRECTION.md', encoding='utf-8').read()
  items, cur = set(), None
  for line in text.splitlines():
      # Reset on EVERY heading. Without this, a heading that is not a milestone
      # -- '### 7.1 Security track' -- leaves `cur` pointing at the last
      # milestone, and that section's items are reported as phantom M1.1..M1.7.
      if line.startswith('#'):
          m = re.match(r'^#{3,4} (M\d+|L\d+|P\d+)', line)
          cur = m.group(1) if m else None
          continue
      m2 = re.match(r'^(\d+)\. \[[ x]\]', line)
      if m2 and cur:
          items.add(f'{cur}.{m2.group(1)}')
  print(sorted(i for i in items if i not in counted))
  PY
  ```

- **M1.1 to M1.7 do not exist, and a version of this note used to say they were expected.** M1 is
  prose with no checkbox items. The phantom entries came from the bug fixed in the script above: it
  never reset `cur` on a heading that is not a milestone, so `### 7.1 Security track` left it pointing
  at `M1` and that section's seven items were reported as `M1.1`…`M1.7`. With the reset in place the
  only uncounted item is **`L2.1`**, which is deliberate and explained on the criterion. Anything else
  appearing is a real gap.

**Keys and addresses (ADR-023, ADR-024, ADR-039)**
- **One implementation, not two.** Both the launcher and QOR ID use `sp-core` itself, pinned to
  `=43.0.0`, the version inside `polkadot-stable2606-1`. Do not reimplement the derivation, the SS58
  encoding or the verification anywhere; ADR-033 rule 1 is what puts them on the SDK's version.
- **`full_crypto` is what makes a key able to sign.** The launcher enables it. QOR ID does not, and its
  tests do, through a dev-dependency, so the service binary cannot sign anything.
- **A junction built from a value is not the junction a person types.** `DeriveJunction::hard("0")`
  encodes the *string* `"0"` and is a different account from `//0`. Build junctions from the path string;
  a test in `vault/derive.rs` asserts the two differ, so this cannot be reintroduced quietly.
- **Account `n` is `//(n-1)`, not `//n`.** The first account is the bare phrase. That offset is Talisman's
  enumeration and is deliberate (ADR-039).
- **`pubkey` is gone from every QOR ID request.** An account is named by `address` (SS58 at the chain's
  prefix) or by `account_id` (`0x` hex), exactly one of the two. Hex in `address` is refused on purpose.
- **A prefix check is not chain identification.** Many chains use 42. Do not describe it, or rely on it,
  as evidence of which chain an address came from (ADR-024, F-Q9).
- **The challenge still names no network** (ADR-039 point 7). The reason in the code, beside the domain tag in
  `src/handlers/auth.rs`, is that no network was deployed. Demiurge Devnet has been live since October (ADR-068),
  so binding the challenge to a genesis hash is now an open item, not a wait.
- **A node and a launcher from opposite sides of 2026-09-20 refuse each other.** ADR-041 changed the
  extrinsic's address from a bare `AccountId32` to a `MultiAddress`, so a node built before that commit
  refuses what a launcher built after it signs, and the reverse. Rebuild both. Only the devnet is deployed,
  and it holds only test CGT. The failure is loud: the client cannot even encode the call.
- **The launcher talks to `chain/` and nothing else** (ADR-040). Its default is Demiurge Devnet,
  `wss://rpc.qorsync.dev`; for a local node, point it at `ws://127.0.0.1:9944` (`LOCAL_RPC`) with
  `chain/target/release/demiurge-node --dev`.
- **The launcher's chain endpoint is a WebSocket address now.** `ws://` or `wss://`, because the client's
  transport is WebSocket and nothing else. A stored `http://` endpoint is upgraded on load rather than
  refused, since it names the same node on the same port. QOR ID's endpoint is unchanged and is still
  `http(s)://`: the two are not the same kind of address.
- **A transfer is reported only once GRANDPA has finalised it,** and a timeout waiting for that is
  reported as a timeout naming the transaction hash, never as a failure. A transaction that has been
  submitted may still finalise, and must not be sent twice.
- **Take the nonce from `system_accountNextIndex`, not from `subxt`'s default.** `at_current_block()`
  pins the latest *finalised* block, so a nonce read there is stale the moment a transaction is in a
  block but not yet finalised, and the second of two transfers is refused as "Transaction is outdated".
  This was observed, then fixed; the live test sends two in a row so it cannot come back.
- **A database that ran the old schema needs migration 018**, which drops `users.on_chain_address`,
  `users.primary_pubkey` and `auth_challenges.pubkey`. A build database from before it will not compile
  the crate.

**Working rules**
- Do not extend or patch the custom chain (ADR-013). Record defects as migration requirements.
- Do not invent open values (`docs/economics/OPEN_QUESTIONS.md`).
- "Session keys" means validator keys in Substrate. What QOR ID issues to agents is an **agent
  authorisation** (ADR-014).
- In `GATES.toml`, never write a URL for a service that has not actually been deployed.

**qor-auth**
- **Compiling needs a migrated Postgres,** because `sqlx` checks its `query!` macros at compile time. Create a
  throwaway database, apply `migrations/*.sql` to it with `psql` for the compile schema, and point
  `DATABASE_URL` at it. Run the service itself against a separate empty database, and let it apply its
  own migrations.
- **The flat `DATABASE_URL` wins over `QOR_AUTH__DATABASE__URL`** at runtime (`src/config.rs`, about line 275).
  Unset it before starting the service, or it silently uses the build database.
- **The `vyb-redis` container on 6379 requires a password.** Use a separate Redis for `qor-auth` tests.
- **Registration shapes changed.** `POST /agents/register` and `POST /profile/link-wallet` take `pubkey`,
  `challenge` (from `GET /auth/challenge?pubkey=`) and `signature`. `link-keypair` needs a bearer token.
- **`POST /auth/keypair-register` signs the new account in** (2026-09-14). It returns `access_token`,
  `refresh_token`, `expires_in` and `token_type` alongside the account fields, and accepts an optional
  `device_id`, as `keypair-login` does. Before this it returned no tokens, so claiming a QOR ID from the
  launcher created the account and then failed with "did not return a usable token pair".
- **Challenge signatures carry a domain tag.** Every client signs `demiurge:qor-id:challenge:v1:` followed
  by the challenge. A signature over the bare challenge gets 401. The tag is defined twice, in
  `services/qor-auth/src/handlers/auth.rs` and `tools/qor-launcher/src-tauri/src/identity/mod.rs`, and
  each side's test pins it.
- **Migration 008 changed.** A database that ran the old version needs its sqlx checksum updated once
  (see the 008 header).
- Production configuration takes secrets only from the nested `QOR_AUTH__*` environment variables.
- **The service will not start on the flat variables alone.** `QOR_AUTH__JWT__ACCESS_SECRET`,
  `QOR_AUTH__JWT__REFRESH_SECRET`, `QOR_AUTH__DATABASE__URL` and `QOR_AUTH__REDIS__URL` must be set, or
  it exits with "missing field". The flat forms then override them.
- **Every access token needs its Redis session** (2026-09-14). Logout ends the token at once; with Redis
  down, authenticated routes answer 500.
- **Nested root routes have no trailing slash.** `/api/v1/profile` works; `/api/v1/profile/` is a 404
  that never reaches the auth middleware, so it proves nothing about authentication.
- **`cargo clippy` and `cargo test` do not rebuild `target/debug/qor-auth.exe`.** Run `cargo build`
  before an end-to-end run, or the checks exercise the previous build. It happened on 2026-09-14.
- **End-to-end scripts** are in `services/qor-auth/scripts/e2e/`. Each takes the base URL and needs a
  running service against Postgres and Redis:
  - `account-and-admin.mjs`, `sign-in-enumeration.mjs`, `sessions.mjs` and `registration-mints-nothing.mjs`
    expect email not configured;
  - `logout.mjs` stops Redis for one check only when `REDIS_CONTAINER` is set;
  - `email-via-resend.mjs` runs a stand-in for Resend on port 59925, so start the service with
    `RESEND_API_KEY`, `EMAIL_FROM` and `RESEND_API_URL=http://127.0.0.1:59925`.
- **Registration calls no chain** (R-3, 2026-09-14). `BLOCKCHAIN_RPC_URL` is no longer read, and the
  response has no `starter_cgt_minted` field.
- **The tests need a Postgres they can create databases on.** Handler tests use `#[sqlx::test]`, which
  creates a database per test from `DATABASE_URL`; in CI that user is the container superuser. Migrations
  012 to 014 are applied to the compile database like the others. Tests that need Redis are `#[ignore]`
  locally: run everything with `QOR_AUTH_TEST_REDIS_URL=redis://... cargo test -- --include-ignored`, as CI
  now does.
- **A password reset needs Redis.** Sessions are revoked before the reset commits, so with Redis down
  nothing is reset.
- **`account-and-admin.mjs`** promotes an account to god and plants a reset token with `docker exec psql`.
  It needs `PG_CONTAINER` and `PG_DATABASE`, and reports SKIP without them.
- **Admin tokens issued before the role fix carry `God`** and are refused. Sign in again.
- **Email is Resend, configured only by environment:** `RESEND_API_KEY` and `EMAIL_FROM`, with `BASE_URL`
  for the link targets. Without both, `forgot-password` answers 503 and registration sends no verification
  email. `RESEND_API_URL` may name another API base, but only HTTPS or local HTTP. No email content is logged.
  `config/production/*` and `scripts/setup-email.*` still describe SMTP and do not work with the service.
- **No address at `demiurge.cloud` receives mail.**
  - The domain has no MX record, and receiving is disabled in Resend.
  - `noreply@demiurge.cloud` and `Godmode@Demiurge.Cloud` are sender identities only. A reply to either
    goes nowhere, and mail sent to either bounces.
  - Never use either address as a test inbox, a contact address or a `Reply-To`.
  - Whether the domain receives mail is a separate decision for the owner. It is not to be set up as a
    side effect of other work.
- **Opening a link page changes nothing.**
  - `GET /verify-email` and `GET /reset-password` only read. The token is spent by the form their button
    submits, `POST /verify-email` or `POST /reset-password`, because mail scanners open links on their own.
  - A test or tool that expects a GET to verify is wrong.
  - A used or expired link answers 410, with a form that posts to `/resend-verification` or
    `/forgot-password`.
  - These page routes sit at the root, beside the JSON API under `/api/v1/auth`, and share its handlers.
- **Request log spans record the path, never the query**, because the link pages carry tokens in the query.
  Do not put `uri` back into the span.
- **`POST /api/v1/webhooks/resend` needs `RESEND_WEBHOOK_SECRET`** (`whsec_…`). Without it, 503.
  - A missing, wrong or more-than-five-minutes-old signature is 401.
  - Only a permanent bounce, `email.suppressed` or a complaint marks an address.
  - Marked addresses are kept as SHA-256 hashes in `email_suppressions` (migration 017). Every send checks
    them, through `AppState::new`, and a marked address is refused with 422 `UNDELIVERABLE`.
- **Unmarking an address is admin-only:** `POST /api/v1/admin/email-suppressions/unmark` with
  `{"email", "reason"}`.
  - The address goes in the body, never the path, because paths are logged.
  - A god account cannot unmark an address that is its own, or pending on its own account. No account
    holder can clear their own mark.
  - It is audited under the acting administrator, by hash, never by address.
  - Resend's own suppression list is separate. An address Resend suppressed must also be removed there.
  - Do not delete rows by hand.
- **Nothing secret may reach a log, at any level** (GATES `alpha.no-secrets-in-logs`).
  - `log_hygiene::nothing_secret_reaches_a_log_at_any_level` drives every such flow through
    `crate::router` with trace-level capture. It needs Redis, so it runs with `--include-ignored`. A new
    flow that handles a token, key, password or address belongs in it.
  - CI's security job also fails when a log call interpolates such a value by name, or a request span
    records the URI. Name such values plainly (`token`, `email`, …) rather than hiding them in a
    differently named variable; the scan reads names.
- **The service is built by `router()` in `main.rs`**, not inside `main`, so tests drive exactly what is
  served.
- **A query built from values fails the build** (`sql_hygiene`, GATES `alpha.parameterised-sql`).
  - sqlx logs full statement text at debug level, so every value is bound. `sqlx::query*` and `raw_sql`
    take SQL fixed at compile time: a string literal, `include_str!`, an upper-case constant, or a loop
    variable over one.
  - Import them only through the `sqlx::` prefix, since an unprefixed import fails the check.
  - Use a `QueryBuilder` with `push_bind`, never `push(format!(…))`.
  - Test SQL with literal values still shows up in the log check's process-wide capture. That is why the
    log check matches only markers unique to itself (`logcheck.invalid`).
- **Review requirement (AGENTS.md §9).** A new flow handling a token, key, secret, password, backup code,
  link or address joins `log_hygiene` in the same change, whatever its values are named.
- **Registration returns `backup_codes`**, ten of them, for an account without an email, instead of one
  `backup_code`. `reset-password-backup` answers with `backup_codes_remaining`.
- **`POST /api/v1/auth/resend-verification`** takes `{"identifier"}` and issues a new verification token,
  replacing the old one. It sends at most once every 5 minutes and 5 times in 24 hours per account
  (migration 015), and registration's message counts as the first.
- **`POST /api/v1/profile/backup-codes`** needs a bearer token and `{"password"}`. It works for an account
  without an email address, or one that added an address while it held codes. It replaces every old code.
- **`POST /api/v1/profile/email`** needs a bearer token and `{"email", "password"}`.
  - It stores the address as pending (migration 016) and emails a link to `/verify-email`. The existing
    `verify-email` route confirms it.
  - The account's current address, if any, gets a notice without a link.
  - It shares the verification-link limits, so a second request within 5 minutes is 429.
  - A password reset clears the pending change.
  - A keypair account cannot use it: its password is random and unknown.
- **Every sign-in refusal is 401 "Invalid credentials"**, including locked and banned accounts. Clients
  cannot tell them apart, by design.
- **The password minimum is now enforced** from `security.password_min_length`: 12 by default, 8 in
  `services/qor-auth/config/production.toml`.
- **`server.host` must be an IP address**, and is now honoured.
- **The launcher's lockfile held a vulnerable `rustls` for five days** (0.23.44, RUSTSEC-2026-0285),
  although `SECURITY.md` said both lockfiles had been upgraded. Only QOR ID's had been. Found on
  2026-09-19 by running `cargo audit` rather than by reading the claim, upgraded to 0.23.45 the same day,
  and corrected in `SECURITY.md`. **Run the audit in both projects when either lockfile changes**; it
  takes a minute, and a written claim is not a check.
- **`cargo audit` configuration lives in each project's `.cargo/audit.toml`, and there is now exactly one:**
  `services/qor-auth`, whose ignore holds only while `rsa` stays out of the compiled graph, which CI
  enforces. The `framework/` exemption expired with the directory at M3.5. The launcher has none, and
  `chain/` has none, deliberately — an exemption file is a place for things to go quiet.
- **`cargo audit` in `chain/` reports 10 vulnerabilities. Measured one by one on 2026-09-20: four are
  compiled, six are not.** Every one is transitive through the pinned SDK, and every one of the four is
  held by a version the SDK dictates, so ADR-033 rule 1 does not override it. Each advisory is named,
  with what blocks it and what would close it, in `SECURITY.md`, "Dependency advisories in `chain/`".
  **Correction:** this file previously said none of the ten was reachable through a normal dependency
  edge. Four of them are: `tracing-subscriber` 0.3.19, through `sc-tracing` and `sp-tracing`, and
  `hickory-proto` 0.24.4 and 0.25.2, through libp2p's DNS transport, mDNS and litep2p. The old claim came
  from a `cargo tree -i -e normal` query not made **per crate version**, and two versions each of
  `hickory-proto` and `tracing-subscriber` are in the lockfile, so an answer about the unreachable copy
  read as though it covered both.
  - **Query per exact version**, or the answer is wrong the same way:
    `cargo tree --workspace -i <crate>@<version> --target all`. Default edge kinds already include build
    and dev, so a dev-dependency counts as compiled.
  - **`tracing-subscriber` does not move by moving the pin.** `sc-tracing` 47.0.0 is the newest published
    and every release from 45.0.0 requires `=0.3.19`. It closes upstream or not at all.
  - **The two `hickory` advisories might.** `libp2p-mdns` admits the fixed `hickory-proto` at 0.49.0.
    Whether a later `polkadot-stable2606` patch carries a new enough libp2p or litep2p is **unmeasured**:
    the exact pin means even a dry run needs the manifest edited, and editing it is the move. That
    measurement is the first step of the pin-move task (ADR-033 rule 3), which is a reason to propose one.
  - **RUSTSEC-2026-0118 has no fixed release anywhere**, so it is carried regardless.
  - **The audit now runs in CI as a report that does not fail the run**, registered in `GATES.toml` as
    `[ci].reports`. No criterion reads it and nothing is exempted: `chain/` still has no `audit.toml`, and
    the answer is still not one. Making it a gate is a decision for when those four can close.

**Chain and launcher**
- ~~**The launcher still mirrors a 13,000,000,000 supply.**~~ **Gone on 2026-09-20**, which is the
  removal ADR-003 asks for by name. `cgt.rs` now declares `MAX_AMOUNT_SPARKS`, a ceiling on what a person
  can type, and **it is not a supply**: ADR-004's perpetual issuance means no fixed total exists, and
  `chain/` declares no such constant for the launcher to mirror. The value is ADR-003's base supply
  because a guard needs a number and that one is decided; a test pins it to that record and pins the
  refusal a person reads to not calling it a total supply. The "Total supply" row left the Settings
  surface with it. **The trap it leaves:** do not put a supply figure back into `TokenInfo`. A panel of
  facts about the connected node should hold only what can be pinned to that node.
- **An account has one nonce**, read from `system_accountNextIndex` (ADR-040). The custom devnet's
  separate request nonce and transaction nonce (D-004) went with it.
- **There is no faucet and no mint.** The chain has no issuance mechanism, and will not until OPEN-1 and
  OPEN-2 are decided. Fund a development account by transferring from one endowed in the chain
  specification.
- After moving the repository, run `cargo clean -p tauri` in `tools/qor-launcher/src-tauri`.
- Zustand 5 selectors must return cached values.
- A `cd` inside a shell command persists between commands here. Use absolute paths.

**Operational files**
- **Four Substrate-fork scripts were deleted on 2026-09-18**, by the owner's decision:
  `apply-substrate-fix.ps1`, `setup-substrate-from-zip.ps1`, `test-blockchain-build.ps1` and
  `complete-substrate-fix.ps1`. The first inserted a `[patch.crates-io]` section pointing `sc-cli` at a
  personal Substrate fork (`ALaustrup/substrate.git`), which contradicts ADR-022's pinned release and
  ADR-033 directly: "The SDK's transitive pins are not overridden, with `[patch]` or otherwise." They also
  targeted `blockchain\Cargo.toml`, a path from the pre-realignment layout that no longer exists. The
  owner's rule: nothing that repoints the dependency graph at a personal fork stays in the repository.
- **Four more were deleted the same day**, found by scanning for the hazard rather than the filename:
  `fork-substrate.ps1` and `verify-and-setup-fork.ps1` cloned the fork, `apply-substrate-fix-in-fork.ps1`
  edited `sc-cli` inside it, and `test-node-build-versions.sh` recommended a `[patch.crates-io]`
  override. Seven in total.
- **A standing check now enforces it**, because deleting by hand is a fix and a check is a control. CI
  fails if any tracked manifest holds a `[patch]` section, or if anything tracked references the fork.
  Criterion `alpha.no-dependency-patching`, recorded in `[ci].quality_gates`. It was proven to fail
  before it was trusted: a planted patch section and a planted fork clone were both caught.
  - **Two files are excluded, deliberately:** `.github/workflows/ci.yml`, which has to contain the
    strings in order to search for them, and this file, which records the rule. Widening that list shows
    up in a diff.
  - The `[patch]` half scans manifests only, because a `[patch]` section is a hazard in a `Cargo.toml`
    and prose in an ADR. The fork half scans everything, so no document can quietly reintroduce it.
- About a hundred other pre-realignment files under `scripts/` do not touch dependencies. They are a
  separate, unurgent question.
- `config/`, `testnet/`, `scripts/*.sh` and the root deploy scripts do not work against the current node.
- **Five of them reference a directory that does not exist**, since M3.5 deleted `framework/`:
  `Dockerfile`, `docker/Dockerfile.node`, `config/production/demiurge-node.service`,
  `testnet/systemd/*.service` and `fly.toml` (a sixth, `docker/docker-compose.testnet.yml`, is gone from the
  tree). They were already non-working, so nothing regressed; they were deliberately left out of the deletion
  commit, which was scoped to `framework/` alone. The plan to rewrite `fly.toml` for `chain/` under ADR-015 is
  moot: the devnet runs on Railway (ADR-068, `chain/DEPLOY-RAILWAY.md`). They are a cleanup of their own.

## 6. Local environment

*Recorded between 14 and 21 September 2026 and not re-checked since. Read `docker ps -a` before relying on any
container named here.*

- **Cleaned up on 2026-09-14.**
  - The test containers `qor-db-1` and `qor-redis-e2e` are removed.
  - Their throwaway databases (`qor_auth_build`, `qor_auth_old008`, `qor_auth_e2e`) are dropped.
  - The volume `qor_qor_pgdata` was kept. It still holds the `qor` and `qor_test` databases that existed
    before these sessions.
- **Docker Desktop was running on 2026-09-15,** with the `vyb` stack it auto-started. Nothing in `vyb` was changed.
  It had stopped by the logout round (2026-09-14) and was started again for it, and again for the live email
  run (2026-09-15). It is installed at `%LOCALAPPDATA%\Programs\DockerDesktop`, not under Program Files.
- **For the rounds of 2026-09-14 and 2026-09-15:** the temporary containers `qor-verify-pg` (port 55432) and
  `qor-verify-redis` (port 56379) were created for each round and removed after it, most recently at the end
  of 2026-09-15.
- **Correction:** `qor-ci-check` (5434) and `qor-redis-ci` (6381) were stopped, not removed. They still exist,
  exited, alongside `qor-launcher-test-pg` (5433) and `qor-launcher-test-redis` (6380). None was touched in
  these rounds, by the owner's instruction; remove them with `docker rm` when no longer wanted.
- **For the key-scheme round of 2026-09-19:** two throwaway containers were created and **left running**,
  `qor-sr25519-pg` (port 55434, database `qor_auth_build`, user and password `qor`) and
  `qor-sr25519-redis` (port 56380). They are what `DATABASE_URL` and `QOR_AUTH_TEST_REDIS_URL` pointed at
  for the compile and the tests. Remove them with `docker rm -f` when they are no longer wanted; nothing
  in them matters.
- **For M3, on 2026-09-17:** the `wasm32v1-none` Rust target was installed with
  `rustup target add wasm32v1-none`, because the Polkadot SDK prefers it for runtime wasm. Nothing else was
  installed; `protoc` 36.0 was already present.
- **For the L1.7 round:**
  - the temporary containers `qor-ci-check` and `qor-redis-ci` were created and then removed;
  - `cargo-audit` was installed into `~/.cargo/bin`;
  - PyYAML was installed only into the session scratchpad, to parse `ci.yml`.
- **Measured on 2026-09-21, against `docker ps -a`:** `qor-verify-pg` and `qor-verify-redis` still exist,
  exited, although the entry above says they were removed after 2026-09-15; so do `qor-ci-check`,
  `qor-redis-ci`, `qor-launcher-test-pg` and `qor-launcher-test-redis`. `qor-sr25519-pg` and
  `qor-sr25519-redis` are exited, no longer running. All are throwaway; remove them with `docker rm` when
  no longer wanted. For QOR ID's run of 2026-09-21, `qa6231-pg` (port 55433) and `qa6231-redis` (56380)
  were created with `--rm` and stopped afterwards, so they are gone.

## 7. Repository state

- **Now (6 October 2026): the public repository `QOR-MATRIX/demiurge-chain`** (ADR-064), whose `main` is at
  `9f8a818` (PR #13). Work goes on a `session/*` branch and reaches `main` by pull request; PRs #1 to #13 are
  merged. `origin` is the private archive and is not the current tree. This file's update (§4 item 58) is on
  `session/docs-current`.
- **Earlier (29 September 2026): the public repository's start.** Local branch `public-main` then tracked
  `public/main` on `ALaustrup/demiurge-chain`: `e611c99` (import) and `2681a51` (CI and Railway records). The
  repository moved to `QOR-MATRIX` on 1 October 2026 (ADR-064).
- **Before that (29 September 2026): branch `session/trading-cards-2026-09-28`**, pushed to `origin` up to `3d63b43`, 64
  commits ahead of `main`. §4 items 18 to 24, listed below as not committed, were committed on 28 September
  (`a3109e2`, `928cab5`). **§4 item 31 is committed, on the owner's word, in three concerns:** the chain and the
  launcher's one-line mint change (`f24f6dd`); the `DMRG` prose sweep (`57232da`); and the records —
  ADR-061, ADR-062, both indexes, `PROTOCOL.md`, `DIRECTION.md`, `GATES.toml`, `OPEN_QUESTIONS.md`, `SYSTEMS.md`,
  `OWNER.md` and this file — in the commit this entry is part of. Pushed to `origin` the same day. `archive-7gUg78/`
  at the root is a GitKraken CLI download (`gk_3.1.76_windows_amd64.zip`) left by a plugin install; it is not
  project work and is not committed.
- **Earlier: branch `session/realignment-2026-09-21`**, created from `main` at `7bfcec6` after
  `session/cgt-foundation-and-launcher` was fast-forwarded into `main`; remote `ALaustrup/demiurge-cloud`.
- **Committed on 2026-09-21 on this branch, one commit per concern:** QOR ID's deployment configuration
  (`119ca53`); Projects, Qontrol's first surface (`5a44207`); QFX layer one (`75b2ead`) and its scope-change
  entry (`42d00c3`); then, after an API failure stopped the session and the work was resumed: Ambience Off
  fixed with checks that can fail (`6e5bb28`); ADR-046 to ADR-051 (`df3ccce`); the seven blueprints
  (`e998faf`); the gates parser reading `P` tracks (`d4be885`); Qontrol's no-skips feature and the CI that
  runs what `npm run check` runs (`7a3c8ea`); the product tracks, gates and systems table (`53d1b7c`);
  `OWNER.md` and this file (`4233c10`); and P1.1 unticked, because its diffs were never written (`f1d499b`).
- **Committed on 2026-09-22, one commit per concern:** `--ink-faint` fixed and the contrast check extended to
  every theme (`320be99`); ADR-047 accepted and M2.3 ticked (`d5133d7`); the check narrowing approved and
  ADR-051 amended and accepted (`9a7eeb9`); the owner's product decisions (`a2537e1`); and this file with
  `OWNER.md` and `SYSTEMS.md` (`643c54e`).
- **Committed on 2026-09-22, the M4.1 session, one commit per concern:** M2.1 ticked and the royalty bound
  carried forward (`99bb0be`); `alpha.chain-tests`' four test names corrected (`5a0378f`); M4.1's chain half,
  `pallet-nfts` and `pallet-drc369` with ADR-052 and U-14 (`d11862e`); `dev-fund.mjs` (`6f934c7`); the
  Projects surface's undefined classes (`3f67870`); the accessibility check's visibility precondition
  (`f596a2b`); M4.1's launcher half (`021d1c4`); M4.1 ticked with the roadmap, blueprints and ADR-047's second
  open item (`3d27c69`); and this file with `OWNER.md` and `SYSTEMS.md` (`5648e91`).
- **Committed on 2026-09-22, from the owner's smoke test:** the launcher matching QOR ID's refusals as they are
  worded now (`26f2521`); the interface lifted above the QFX backdrop and its scrim, with
  `check-readability.mjs` (`1f5f4ed`); three colours raised to read AA over any backdrop (`67a083d`); those
  status documents (`53d7ea2`); the DRC-369 card shell with the carve-out the owner widened (`5d8ad74`); the
  owner's asks of that day written into the roadmap, with the correction that the chain holds no files
  (`ebb93f3`); trade — `pallet-utility` with only `batch_all` reachable, an asset's menu, the warning and the
  live proof, plus the widened media-type table and Q-20 (`d5e59f4`); and this file with `OWNER.md` and
  `SYSTEMS.md`.
- **Committed on 2026-09-23, on the owner's word, one commit per concern — four, not five.** The trade half
  and the Sell half **could not be split**: `lib.rs` declares both new modules and `ipc.ts`'s `trade` carries a
  `label`, which binds `TradeDialog` and `SellDialog` to one signature, so splitting them would mean inventing
  an intermediate state that never existed — the same reason the vault and chain-client halves were one commit
  on 2026-09-20. An earlier draft of this entry promised five; that was wrong and is corrected here rather
  than quietly dropped.
  - `9484f23` — the code: the trade window's two sides and lane, `partners.rs`, `Sigil.tsx` lifted out of the
    card, and the listing draft (`listings.rs`, `SellDialog.tsx`, five Tauri commands, `ipc.ts`).
  - `35b82aa` — the Inventory check: the duplicate `sent` that stopped it parsing, the Sell block that read a
    dialog which never opened, and the twenty checks.
  - `a8c22eb` — `GATES.toml`: L4.6 and P2.8 counted, and the correction of the 110 that was never measured.
  - the documents, in the commit this entry is part of: `DIRECTION.md`, the launcher `README.md`, `OWNER.md`,
    `docs/SYSTEMS.md` and this file.
  - **Not pushed, and no pull request, on 2026-09-23.** (Superseded: work now reaches `main` by pull request.)
- **What was already in the tree when the session of 2026-09-23 began, written and never once run:** the
  trade window's two sides and lane, `partners.rs`, `Sigil.tsx`, the listing draft and its five Tauri
  commands, the Sell wiring, and the Inventory check's twenty new checks. Running it is what that session
  did, and it is how all three faults in §4 item 17 were found — none of them was visible by reading. The
  documents and the gate-coverage fix were written that day, after the evidence existed.
- **Not committed when written; committed on 28 September 2026 in `928cab5`, in the private archive:
  §4 item 18**, Qontrol's diffs — `qontrol/diff.rs`, the change to `qontrol/mod.rs` and
  `commands.rs`, ten host tests, `ipc.ts`, `Projects.tsx`, the Projects and readability checks, `GATES.toml`,
  `DIRECTION.md`, the Qontrol blueprint, the launcher `README.md`, `OWNER.md`, `docs/SYSTEMS.md` and this
  file. The owner commits.
- **Not committed when written; committed on 28 September 2026 with item 18: §4 item 19**, Qontrol's git layer — `qontrol/guard.rs`, the changes to `qontrol/mod.rs`,
  `commands.rs`, `sidecar.rs`, `tests.rs` and `lib.rs`, the helper's `main.rs`, `scripts/build-helper.mjs`,
  `src-tauri/tauri.bundle.conf.json`, `package.json`, `.gitignore` (the built helper and licence are not
  tracked), `ipc.ts`, `Projects.tsx`, the Projects and readability checks, and the same documents as item 18.
  Also the trade fix found while re-running it: `src/views/TradeDialog.tsx`, the new check in
  `scripts/check-inventory-view.mjs`, and its `GATES.toml` entry — a separate concern, and a clean commit
  of its own. And §4 item 20, the vault restore: `vault/mod.rs`, `error.rs`, two commands in `lib.rs`,
  `ipc.ts`, `components/gate/Gate.tsx`, and the launcher `README.md` — a commit of its own except for `lib.rs`,
  which also carries item 19's Qontrol command registrations, so that file needs staging by hunk. §4 item 21,
  Windows Hello, touches the same vault, Gate and `lib.rs` files as item 20, plus `vault/hello.rs`,
  `SettingsView.tsx`, `Cargo.toml`, `Cargo.lock`, and ADR-054 with its two index rows. **§4 item 23**, Hello
  alone (ADR-055), rewrites those same files again — `vault/mod.rs`, `vault/hello.rs`, `error.rs`, `lib.rs`,
  `chain/mod.rs`'s three live tests, `ipc.ts`, `Gate.tsx`, `SettingsView.tsx`, `ascent.ts`, both Gate checks —
  and adds ADR-055 with its index rows, the notes on ADR-054 and ADR-016, and edits to `SECURITY.md`,
  `DIRECTION.md`, `MIGRATION_INVENTORY.md`, `GATES.toml` and the launcher `README.md`. §4 item 24 (ADR-056) rewrites
  them once more and adds `vault/keychain.rs`, `App.tsx`, `store.ts`, `TitleBar.tsx`, `VaultView.tsx` and
  `chain/assets.rs`'s tests. Items 20, 21, 23 and 24 cannot be separated cleanly: one commit for the vault is
  the honest shape. Items 18 and 19 share files (`mod.rs`, `tests.rs`, `Projects.tsx`, the checks), so they may not split into
  two commits cleanly. The owner commits.
- Earlier history, kept as it was written:
- The owner confirmed credential rotation on 2026-09-14, which cleared the branch for pushing.
- The QOR ID fixes are committed one commit per fix: those of 2026-09-14 end at `67de317`, and those of
  2026-09-15 at `517fc83`. That covers:
  - resend-verification, backup-code regeneration and ADR-017;
  - adding or changing an email address and the rewritten emails;
  - the log span fix, the link pages and Resend webhooks;
  - `BASE_URL` origin-only, the admin unmark route and the standing log check;
  - ADR-015's clarification, and the records of all of these.
- Committed on 2026-09-15 after `517fc83`, one commit per concern:
  - the SQL parameterisation check;
  - AGENTS.md §9, the review requirement;
  - the base-layer decisions (ADR-018 to ADR-032), with the release re-check and the naming amendment;
  - U-10, U-13 and the other open-question updates;
  - the sponsorship proposal;
  - the dependency-version policy (ADR-033).
- Committed on 2026-09-17, one commit per concern:
  - the record of the ticker research, and three corrections to what U-13 had claimed;
  - a correction of occurrences counted as lines;
  - **the ticker change itself** (ADR-034): the declared symbol, the launcher's hard-coded fallbacks and its
    copy, the migration requirement R-4, the `public-release.name-clearance` gate criterion, the rename of the
    economic model to `docs/economics/CGT.md`, and two defects found while doing it;
  - this documentation alignment pass.
- Committed on 2026-09-18, one commit per concern: the acceptance tests and ADR-038, the block-import
  harness, the two-validator script and its accepted evidence, the narrowing of M3.1 and M3.3, the deletion
  of the seven fork scripts with the check that enforces it, and the rewrite of `PROTOCOL.md`.
- **Committed on 2026-09-20, one commit per concern.** This covers both the key-scheme round of
  2026-09-19 and the chain-client round of 2026-09-20, which were both still uncommitted:
  - **QOR ID's accounts become Sr25519 and SS58** (ADR-039, migration 018), with `SECURITY.md`, the CI
    log scan's new names and `GATES.toml`'s two tightenings for it.
  - **The launcher: an Sr25519 vault and a `subxt` chain client** (L3.2 and L3.1, ADR-040). The vault's
    half and the client's half are in one commit because the client rewrote the files the vault's half
    had changed, and the vault's half was never committed on its own; splitting them now would mean
    inventing an intermediate state that never existed.
  - **The documentation alignment** for both rounds: `DIRECTION.md` (L3.1, L3.2 and M3.4 ticked),
    `DECISIONS.md`, the ADR index, the migration inventory, `docs/audit/RECONCILIATION.md`,
    `chain/README.md` and this file.
  - **Q-17, the runtime's address type, written up for the owner**: `docs/architecture/ADDRESS_TYPE.md`,
    its row in `docs/README.md`, and `GATES.toml`'s `beta.address-type` tightening.
  - **`AccountIdLookup` and `MultiAddress`** (ADR-041), the owner's decision on Q-17, carried out the same
    day: the runtime, the two chain test files, the launcher's `subxt::Config` and its transfer, in one
    commit because a node and a client on opposite sides of it refuse each other.
  - **M3.5's reference work**: AGENTS.md §4, §7 and §8, `.cursorrules`, CI (the `framework/` job replaced
    by a `chain/` job that is a quality gate), `HANDOFF.md` §2.0, `scripts/run-local-stack.md`, `GATES.toml`'s
    CI scope, and DIRECTION's M3.5. **Verified with the tree still present:** `framework/` 285,
    `chain/` 51, the launcher 91, all green.
  - **The deletion of `framework/`**, on its own, after the owner confirmed it: 150 files, 49,058 lines,
    plus the CI guard against its return and M3.5 ticked. Scoped to `framework/` alone, by the owner's
    choice; the six deploy files that referred to it are a separate cleanup (§5).
- **No pull request was opened then.** Since October 2026 work reaches `main` by pull request (the first entry
  above).
- The confirmation that gated M3 arrived on 2026-09-17 (§4 item 3), so nothing in the chain work waits on it any longer.
