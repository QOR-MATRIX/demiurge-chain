# ARQADE: master implementation brief

**What this is.** The corrected form of the ARQADE master implementation prompt the owner handed over on 4 October
2026, reconciled the same day with the code and the accepted decisions. It is a blueprint (`ECOSYSTEM.md` §8): the
design and the brief for whoever implements it. **It is not a roadmap**: ARQADE's steps are P7 in
[`../DIRECTION.md`](../DIRECTION.md), and its decisions are [ADR-069](../decisions/ADR-069-arqade-the-gaming-platform.md)
(accepted by the owner on 4 October 2026, with ADR-043). Read [`ECOSYSTEM.md`](ECOSYSTEM.md) first.

**Status:** planned. The existing arcade runs on OpenAI Sites and its source is outside this repository. Nothing in
this repository implements ARQADE yet.

---

## 0. What changed from the 4 October handoff, and why

The owner's mandate is unchanged. These are the places where the handoff did not match the ecosystem, each checked
against the tree on 4 October 2026.

| # | The handoff said | What is true | What this brief does instead |
| --- | --- | --- | --- |
| 1 | "Integrate the real QOR service using its implemented flows"; "validate token claims through the service's supported contract" | QOR ID has **no browser sign-in** (no `/authorize`, no PKCE; ADR-043 is Proposed) and **no way for another server to verify a token**: HS256 with a shared secret, no introspection | Sign-in waits on ADR-043 plus a verification path (ADR-069 decision 4). ARQADE never holds QOR ID's secret. Until then the Sites alias is a labelled practice identity |
| 2 | Connect "the official Vault/launcher integration", or "implement the minimal approved ecosystem bridge" | **No bridge exists** and none is approved. ADR-011 says web surfaces sign with **delegated keys** (ADR-010, ADR-026, M5.2), unbuilt | Three options and a recommendation in ADR-069 decision 5; each needs its own record before code. Until then ARQADE reads and does not sign |
| 3 | "Reconcile its location with the web-surface decisions" | ADR-011 allows exactly two web surfaces and keeps publishing in the launcher; ADR-050 says where products live | ARQADE is a product in `products/arqade/` (P7, gate `arqade`) and a third web surface amending ADR-011 (ADR-069 decisions 2 and 3) |
| 4 | Preserve the "cinematic portal, terminal aesthetic" | `DESIGN_SYSTEM.md` rules out glows, particles, animated backgrounds, neon and cyberpunk **on every surface** | Kept, but only once the owner gives ARQADE a design record of its own (as ADR-051 did for QFX). Reduced motion, contrast and focus rules apply regardless |
| 5 | "Verifiable allocation" for packs; "inspect existing verifiable randomness" | **The chain has no randomness source** (Aura, no VRF, no randomness pallet) | Gap **G-14**. No pack is sold for CGT until it is decided |
| 6 | Account-bound trophies "with actual protocol support"; "edition enforcement" | Nothing locks an item's transfer; `Drc369::mint` mints to the signer in one singles collection, with no supply cap | Gaps **G-15** (collections and editions) and **G-16** (account-bound). Until decided, trophies are transferable and say so; no edition cap is claimed |
| 7 | "Build missing state/XP primitives as bounded extensions" | State and XP are **M4.2**, the chain's item, and unstarted (`chain/pallets/drc369/src/lib.rs:28-30`) | Evolution waits on M4.2. ARQADE builds no chain code of its own |
| 8 | "Provide an authorized, rate-limited faucet" | The devnet's faucet account is the owner's; a faucet page is the ecosystem's next item (`OWNER.md`) | One ecosystem faucet. ARQADE never runs one or embeds its key |
| 9 | "Implement bounded, checkpointed event indexing" if no indexer exists | ADR-028 puts provenance in an archive node and **the** indexer (M5.4) | ARQADE reads chain state directly for what it needs now, and uses the indexer when it exists; it does not build a second one |
| 10 | Ship an SDK "in a documented canonical location" | M5.1 is the ecosystem SDK; ADR-009 holds publication until the wire format is frozen | ARQADE's SDK is a game layer over M5.1, unpublished before `beta.wire-format-frozen` (ADR-069 decision 9) |
| 11 | A release pipeline with its own catalogue | Market is "one listing model for games, tools, …" (P5.3) | Third-party games are Market listings of kind "game"; ARQADE plays the web-playable ones |
| 12 | Paid play, CGT prizes, "approved budgeted funds" | No treasury (OPEN-2), no issuance (OPEN-1); a prize is not payment for work (ADR-008) | Devnet test values only. Production funding and whether paid entry may win anything is **U-16** |
| 13 | Optional "Energy" entitlements | ADR-031 keeps fungible game items out; "Energy" was the deleted chain's fee model (D-001); the site shows USD prices for it | Energy is retired. No USD prices, no CRGT, no DMRG |
| 14 | "Use the official Sites workflow" | The arcade is on OpenAI Sites (Cloudflare Workers, D1); this ecosystem is on Railway and `qorsync.dev` / `demiurge.cloud` | Sites stays the prototype host until the owner chooses (ADR-069). An implementing agent without Sites tooling does not deploy there |
| 15 | Existential deposit to be "addressed" | Decided: 100 CGT (ADR-036), and a sponsor pays it (ADR-029, mechanics U-4, M4.4) | Payouts below what an account needs accrue until payable; ARQADE subsidises no account outside ADR-029's mechanism |

---

## 1. The mandate (the owner's, unchanged)

ARQADE is the official starting point for Demiurge's on-chain gaming platform: a futuristic, social gaming universe
whose collectible economy is built **exclusively on DRC-369** and whose cost and payout currency is **exclusively
CGT** (Creator-God Token). DRC-369 is a foundational game system, not a gallery added after the games. Players use
their own Vault-controlled accounts to pay CGT, receive CGT, own trophies, collect and play trading-card universes,
and carry evolving assets between compatible games. Independent creators build such assets and publish their own
games through a developer devnet and SDK. The deliverable is a platform for players and developers.

This is implementation work, done in bounded, tested slices. A polished simulation is never substituted for an
integration that is missing; the missing piece is named instead.

## 2. The starting point

- **Site:** `https://demiurge-arcade.quick-crab-0610.chatgpt.site/`, OpenAI Sites project
  `appgprj_6aafb4a6de848191a3d57f456f231b2a`, publication version 2, audience `custom` (keep it unless the owner
  says otherwise).
- **Source:** `C:\Users\Godmode\Documents\Codex\2026-09-20\sites-plugin-sites-openai-curated-remote`, revision
  `4f02baff7b0d843c6fdf0f4552582be0c28addaa`. Its `.git` belongs to another Windows user, so git refuses it until the
  owner adds it as a safe directory; read the files regardless. `outputs/death-cap-media` is unrelated.
- **Stack:** vinext on Cloudflare Workers, D1 through drizzle (`sqlite-core`), identity from the host's
  `oai-authenticated-user-*` headers. No `test` script: `node --test tests/arena.test.mjs`;
  `tests/worker.smoke.mjs` needs a worker on `127.0.0.1:8787`.
- **Ecosystem:** `QOR-MATRIX/demiurge-chain`, observed at `ea706fba287dada38d1418baba73a729d262962b` on 4 October 2026.

Verified on 4 October 2026: four solo games (Void Runner, Synapse, Orbital in `app/games.tsx`; Rift Survivor in
`app/rift-survivor.tsx`) and two arenas (Flux Four, Rift Reversi, `lib/arena-store.ts:11`), revision-guarded on the
server. Solo scores, Energy (20 a session, a free refill to 1000) and 0 to 3 tokens a game are in `localStorage`
(`app/page.tsx:15-21`). The alias is `Explorer-` plus eight hex characters of a hash of the host's user id. `qor_id`
is read and displayed and never written. Polling, not WebSockets. The creator is a template and the terminal runs in
the browser. `app/api/chain/route.ts` calls the deleted chain's `chain_getBlockNumber` at `https://rpc.demiurge.cloud`.
The store shows USD prices for Energy and names CRGT and DMRG.

Preserve the experience: the layout, games, controls and mobile behaviour. Evolve the branding to **ARQADE, powered by
Demiurge** across metadata, intro, navigation, wallet, terminal and notices. Keep old links where practical, keep the
intro skippable, and respect reduced motion. The cinematic styling stays pending the design record in ADR-069.

## 3. The ecosystem truth to build against

Read, in order: `HANDOFF.md`, `docs/DIRECTION.md`, `docs/DECISIONS.md`, `docs/economics/CGT.md`,
`docs/economics/OPEN_QUESTIONS.md`, `docs/protocol/PROTOCOL.md`, `chain/README.md`, `.cursorrules`, then `OWNER.md`,
`docs/SYSTEMS.md`, `SECURITY.md` and `docs/GATES.toml`. Follow `AGENTS.md`, above all §1 (old documentation does not
exist) and §5 (money).

Checked on 4 October 2026; recheck against the runtime you connect to:

- **One chain, `chain/`.** Aura authors, GRANDPA finalises. The retired custom chain and its RPC vocabulary are not
  revived. Pallets: System, Timestamp, Aura, Grandpa, Balances, Session, ValidatorSet, Nfts, Drc369, Utility (only
  `batch_all`), Drc369Royalties, and Sudo on development and test networks. `spec_version` 6, `transaction_version` 2.
- **No fees** (no transaction-payment pallet), **no treasury**, **no issuance**, **no randomness**, **no proxy**, **no
  sponsorship**. Fees are never estimated or shown, because there are none.
- **CGT:** 18 decimals, integer Sparks, `1 CGT = 10^18 Sparks`, existential deposit 100 CGT, SS58 prefix 42
  (`chain/runtime/src/denomination.rs`). Devnet CGT is **test CGT**; say which network wherever value moves.
- **Accounts:** AccountId32, SS58, Sr25519, derived as the ecosystem derives them (ADR-023). No key is ever made from
  a username or an email address.
- **The devnet:** `Demiurge Devnet` (`demiurge_devnet`, Live), `wss://rpc.qorsync.dev`, genesis
  `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a`, health at
  `https://rpc.qorsync.dev/health/readiness`. Upgraded in place where possible, reset with notice otherwise
  (`docs/architecture/DEVNET_PLAN.md`, `chain/DEPLOY-RAILWAY.md`).
- **QOR ID:** `https://id.qorsync.dev/api/v1`. Bearer JWTs (HS256, shared secret) whose `sub` is the account's UUID,
  CORS by allowlist, challenge signing domain-separated as `demiurge:qor-id:challenge:v1:`, keys linked through
  `GET /auth/challenge` and `POST /auth/link-keypair`. A password-only account has no chain identity until it proves
  a key (ADR-017).
- **DRC-369:** mint with a content reference (ADR-047's 41-byte BLAKE3-256 manifest root) and a pinned commit, revise
  until permanent, remix provenance (`derived_from`, depth 16), nesting (depth 8, 64 children; a nested asset is
  held in place, ADR-065), royalties and a sale settled in CGT (`list`, `buy`, `buy_exact`; runtime API
  `Drc369RoyaltiesApi::sale_preview`). **State, XP and physics are not started** (M4.2, M4.3).
- **Frozen:** `apps/`, `cli/`, `sdk/`, `packages/`, `client/`. Nothing there is a library to integrate with.

## 4. Audit by capability

Start with the arcade's `IMPLEMENTATION.md`, then verify the code: `app/page.tsx`, `app/games.tsx`,
`app/rift-survivor.tsx`, `app/live-arcade.tsx`, `app/systems.tsx`, `app/chatgpt-auth.ts`, `lib/arena-store.ts`,
`lib/arena-engine.ts`, `app/api/{live,chat,matches,chain}`, `db/schema.ts`, `drizzle/`, `tests/`.

Write one integration inventory in `products/arqade/` once the source is imported (until then, beside this brief):
for every control, terminal command, statistic and status, its source of truth, permission, persistence, failure
behaviour and verification. Wire what can be real, and disable the rest with the exact missing dependency named. The
first removals need nothing decided: the CRGT and DMRG copy, the USD prices, the Energy-to-token framing, and the
dead chain call.

## 5. Identity

- The account key is QOR ID's immutable `sub`; usernames are profile data.
- **Verified sign-in waits on ADR-043** (redirect, authorization code with PKCE, no 30-day refresh token in a
  browser, per-client sessions) **and on a verification path for ARQADE's server** (ADR-069 decision 4). ARQADE's
  origin joins QOR ID's CORS allowlist then, and not before.
- Until then, the Sites alias is shown as a practice identity. No ranking that claims integrity, no payout and no
  award is bound to it.
- Signing in to QOR ID and controlling a chain account are separate facts. Show a password-only account honestly and
  send the player to the launcher to link a key.
- Server-managed sessions, origin checks, nothing secret in browser storage, URLs or logs. Any new QOR ID flow joins
  `services/qor-auth/src/log_hygiene.rs` in the same change (AGENTS.md §9).
- Explorer records migrate only after the player proves both identities, with collisions handled and history kept.
- Logout, expiry, revocation and bans apply to games, chat and every protected operation.

## 6. Signing and custody

The player's Vault in the launcher is the only custody. ARQADE never asks for a phrase or a key, never creates a
wallet, and never holds a key for a player. Until ADR-069 decision 5 is accepted and the chosen path has its own
record, **ARQADE reads and does not sign**: a player who buys, sells or transfers does so in the launcher's Market and
Inventory, and ARQADE shows the finalised result. When signing arrives, every request shows amount, recipient,
purpose, network and maximum spend before approval, and delegated authority is only ever what the chain enforces
(M5.2), never a limit kept by the frontend.

## 7. Chain reads and CGT settlement

- One typed boundary against the connected runtime's metadata. Check the genesis hash before anything else; detect
  an endpoint or runtime change; refuse a wrong network.
- Integer Sparks end to end, as decimal strings in JSON. Reject excess precision and overflow. No floating point for
  any amount, price, payout or comparison. Spendable balance accounts for reserves, holds, freezes and the
  existential deposit.
- Follow finalised heads with reconnect and backoff; show connection state and data age; keep a last-known value as
  stale rather than showing zero.
- Transaction states stay distinct: awaiting approval, signed, submitted, included, finalised success, failed,
  rejected, expired, unresolved. Inclusion is not success; check the dispatch outcome and events at finality. A
  timed-out submission is reconciled by hash and nonce, never blindly resent. The launcher's
  `src-tauri/src/chain/mod.rs` shows the pattern.
- Where a database action depends on settlement, a durable outbox and a reconciler give at-most-once credit.
- No administrator, faucet, treasury or player key in ARQADE. Operator signing, if ever needed, is a separate,
  least-privilege service with spending limits, approved first.
- History comes from chain state and, once it exists, the indexer (M5.4). No second indexer.

## 8. DRC-369 is the only collectible system

Every trophy, card, pack, cosmetic, equipment item and evolving owned object is a real DRC-369 asset. No parallel
standard, no database ownership ledger, no JSON record presented as an asset. Scores, particles and UI state are not
assets.

An asset is identified by genesis plus collection and item. Ownership is read from finalised chain state; caches are
projections. Content follows ADR-047: retrieved bytes are checked against their reference, unavailable media shows a
retrieval error, and the Mesh is not claimed (M8). Content revision, game state, ownership and presentation stay
separate, and a permanent reference is never changed to fake evolution.

**What can be built now, and what waits:**

| Experience | Now (M4.1, M4.2 halves, M4.6) | Waits on |
| --- | --- | --- |
| **Trophies** | An issuer account mints and transfers in one `batch_all`; the award and its rule version recorded once per eligibility event | **G-16** for account-bound trophies; until then they are transferable and say so. The issuer's authority and how its key is held need a record of their own |
| **Card universes** | Cards as assets whose manifest declares universe, set and role; deck eligibility read from finalised ownership; sales through `buy_exact` with the chain's royalties | **G-15** for creator collections and enforced editions; **G-14** for packs, which are not sold for CGT until it is decided |
| **Evolving assets** | Nothing on chain | **M4.2 state and XP**. A transition contract — asset, expected version, game and rule, evidence, issuer, permitted change, expiry, unique action id — is designed with it, not ahead of it in ARQADE's database |

Recheck ownership and revision when an asset enters a match, is traded or is updated, and say what happens when it
changes mid-session. Nesting and its locks are the chain's (ADR-065). A direct transfer is not a royalty-paying sale.
Cross-game use is opt-in through declared schemas. Programmable behaviour is bounded, versioned and deterministic; no
arbitrary code runs on validators or in the vault. A signature proves who attested to an outcome, not that it was
true: the game authority, its anti-cheat evidence and its limits are written down.

## 9. CGT costs and payouts

CGT is the only cost and payout currency. Every paid entry, pack, purchase and sale is priced in CGT; every currency
payout is CGT to the player's own account. No chips, Energy, CRGT, DMRG, swaps or conversion rates. Local Energy,
points and tokens are practice data and never become claims.

- **Sinks.** Paid entry, packs and card sales are access gating (ADR-006). Name the sink in every mechanic.
- **Payouts come from the game's own ARQ Wallet** (the owner's direction and name, 4 October 2026; ADR-070, accepted): a
  keyless account per published game, funded by its developer, paying only within a policy on chain. See
  [`products/arqade/sdk/docs/arq-wallet.md`](../../products/arqade/sdk/docs/arq-wallet.md).
- **Payouts move existing CGT.** Nothing is created by play. A displayed prize is funded and reserved before a paid
  round accepts entries, with caps, liability and failure handling recorded.
- **On the devnet**, with test CGT and values marked as placeholders, the whole loop is built and tested.
- **In production**, nothing is paid out until **U-16**'s remaining parts are decided (whether paid entry may win
  anything, and the protocol's bounds on an ARQ Wallet). **Paid chance-based prizes stay off** until U-16
  and a legal review; calling something a sweepstake does not make it compliant.
- **The existential deposit** is 100 CGT. A payout too small to open an account accrues as a pending claim until it
  can be paid; ARQADE subsidises no account outside ADR-029's sponsorship (U-4, M4.4).
- **Randomness that decides value** needs G-14's answer. Browser randomness, public daily seeds and client scores
  never decide anything redeemable.
- **No fiat** in the loop (U-15 governs any later rail).
- **Language** follows ADR-008: no cash value, no "earn" framing for play, nothing about what CGT is worth.

The journey, once its dependencies exist: the Vault approves a CGT cost, its finalisation authorises one session, the
server validates the result, and a funded payout or a DRC-369 award reaches the player's account exactly once, with a
finalised receipt.

## 10. Play, rankings and community

Keep the arenas' authoritative validation, revision guards, reconnection, timeouts, spectators, concessions and
exactly-once results. Solo games award nothing ranked or redeemable until they have server sessions or deterministic
replay with issued session ids, rule versions, timing bounds and replay prevention. If action multiplayer moves to
WebSockets, it is measured; if polling stays, it is called polling. Rankings come from validated results only, with
tie-breaks, seasons, collusion checks and corrections written down, and bind to `sub` once sign-in is real. Chat
keeps its rate limits, deletion and reporting, and gains moderation, bans and a retention policy. No fake players, no
fake activity, no coercive streaks, no deceptive odds.

## 11. The developer devnet and SDK

- **Devnet:** Demiurge Devnet and a local node. No second chain. Network identity, RPC, compatibility, health and the
  reset policy are published from `chain/DEPLOY-RAILWAY.md`. Test CGT from the ecosystem's faucet page.
- **SDK:** a typed TypeScript layer over M5.1 for game sessions, outcomes, leaderboards, rooms, manifests, CGT amount
  parsing and formatting, inventory, sales and royalties, with browser and server entry points separated so that a
  browser cannot mint, forge outcomes, spend payout funds or impersonate a player. Names are the owner's. **Not
  published before the wire format is frozen.**
- **A game manifest** declares its id, creator `sub`, build hash, runtime target, DRC-369 schemas, requested
  permissions, CGT policy reference, service endpoints, network and rule versions. Economic policy and permissions are
  reviewed, not accepted from game code.
- **Three references**, built on the same public SDK: a trophy game, a card game with sets and decks, and a game
  whose asset evolves (after M4.2).
- **Release:** third-party games are listed through Market's listing model (P5.3), play in an isolated origin with a
  validated message bridge, and pin their game and rule versions for every paid session. Developer sign-in for third
  parties needs its own record (ADR-043 excludes it).

## 11a. Self-publishing, backing and agents

The owner's direction of 4 October 2026, decided in ADR-071 (accepted 4 October 2026): developers self-publish once a stage's
requirements are met, the Cartridge's profile is the store page and its history the devlog, a game is free or up to
10,000 CGT, in-game purchases are non-fungible DRC-369 items, backers support projects in CGT and receive the work and
never its proceeds, and an external LLM builds through ADR-010's MCP server while the developer signs. The developer
guides are [`publishing.md`](../../products/arqade/sdk/docs/publishing.md),
[`backing.md`](../../products/arqade/sdk/docs/backing.md) and
[`building-with-agents.md`](../../products/arqade/sdk/docs/building-with-agents.md).

## 12. Console and creator

Every dashboard and terminal read uses the same services as the interface. Commands that spend, sign, publish, delete
or change permissions need a preview and the player's approval. The template creator stays; a real AI provider is
connected only when the owner has configured one with a budget. A native CLI, if ever wanted, is not the frozen `cli/`.

## 13. Slices

The numbered steps are P7 in `DIRECTION.md`; this is the order and what each proves.

1. **Baseline** (P7.1): import, inventory, the false claims removed, tests run. **Done 5 October 2026.**
2. **Live reads** (P7.2): genesis, finalised head, a balance and an account's DRC-369 inventory, read-only. **Done
   4 October 2026.**
3. **Verified identity** (P7.3): ADR-043 and a verification path. **Built 5 October 2026** (ADR-073); live once deployed.
4. **Signing** (P7.4): the path ADR-069 decision 5 chooses, with its own record.
5. **Paid play and awards on the devnet** (P7.5), exactly once, surviving reloads and retries.
6. **Card universes** (P7.6) and **evolution** (P7.7), as G-14, G-15 and M4.2 land.
7. **Developer SDK and references** (P7.8) and **third-party release** (P7.9).
8. **Production payouts** (P7.10), only after U-16 and a legal review.

When a slice is blocked, name the prerequisite and continue with independent work. Do not rewrite stable systems to
look busy.

## 14. Evidence and handoff

Run the arcade's tests before and after each change, and the ecosystem's suites for any component changed (the chain's
with the runtime built, never under `SKIP_WASM_BUILD`). Add tests for the failure cases the handoff listed — two
users, expired and forged identity, wrong genesis, RPC loss, stalled finality, runtime change, the existential deposit,
maximum amounts, excess precision, failed dispatch, ambiguous submission, duplicate completion, forged scores,
exhausted payout budgets, inventory against finalised ownership, refused signatures, malicious embedded-game requests,
accessibility, reduced motion and frame time — as each becomes reachable. Keep unit, local, devnet and production
evidence apart, and never weaken a gate.

Commits, pushes, deployments, migrations, access changes, funded transactions and releases follow the owner's existing
authorisation. Taking ownership does not authorise moving funds, changing governance, issuing currency or replacing
custody.

Every handoff ends with what works, the evidence and environment, what blocks, and the one next owner action. Each
feature is marked verified, implemented but unverified, blocked, or deferred. ARQADE is not done with sign-in and a
balance panel: it needs Vault-funded CGT play and payouts, real DRC-369 trophies, cards and evolution, and a usable
SDK with a proven release. If a protocol capability is missing, the release is reported as partial.
