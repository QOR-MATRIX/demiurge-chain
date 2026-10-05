# ARQADE — implementation state

Renamed on screen from "DEMIURGE · The Sovereign Arcade" to **ARQADE, powered by Demiurge** on 5 October 2026; browser storage keys keep their `demiurge.` names so no visitor loses progress. The control-by-control record is `INTEGRATION_INVENTORY.md`.

Imported into the Demiurge repository on 4 October 2026 from the arcade checkout at `4f02baf` (P7.1, ADR-069). The design and the steps are `docs/blueprints/arqade.md` and P7 in `docs/DIRECTION.md`; where this file and those disagree, those win.

## Working

- Cinematic glitch intro with the requested two messages and an informational preview notice.
- Responsive console, original generated eclipse artwork, procedural canvas game illustrations.
- Void Runner: perspective lane survival, keyboard/touch movement, obstacles, collectibles, timer and hull.
- Rift Survivor: WebGL procedural background plus a bounded Canvas action renderer, 90-second daily seeded enemy patterns, auto-fire, movement, telegraphed rift hazards, escalating enemies, combo chains, collectible fragments, and three mid-run upgrade choices. Keyboard and pointer controls, visibility pause, reduced-motion support, and Canvas fallback.
- Flux Four and Rift Reversi: persistent two-player strategy matches, invitation links, spectators, reconnect, legal-move hints, 90-second turns, concessions, rematch creation, and server-authoritative rules.
- Global top-100 and per-game rankings from completed multiplayer matches. Combined rankings aggregate before limiting. Win 30 / draw 10 / loss 5 points; fewer than six moves does not rank. Solo browser scores never submit to global rankings.
- Floating world chat with authenticated author identity, persistent messages, two-second send limits, own-message deletion, local mute, report storage, unread indicator and draft preservation on connection failure.
- Shared D1 data: players, matches, standings, messages and reports. Versioned schema migration. Match state and awards settle in one atomic, revision-guarded batch, preventing duplicate results from racing requests.
- Live lobby, presence, chat and ranking updates every two seconds while visible; active match reads every 1.2 seconds. This is near-real-time polling for turn-based games, not a WebSocket action-game transport.
- Server-derived Explorer aliases use the Sites authenticated identity. A nullable verified QOR mapping is reserved; client-provided usernames cannot impersonate QOR IDs.
- Synapse: random memory sequences, keyboard/touch input, eight progressive rounds.
- Orbital: timed capture arcs, increasing orbital speed, ten locks and three misses.
- Optional synthesized audio, pause controls, local best scores and completion results.
- Device-local energy, complimentary refills, session ledger and unbiased Web Crypto collectible draws (0–3, 25% each).
- Practice Energy storefront: free packages, no cash prices. Paid play will be priced in CGT only (ADR-069).
- Browser terminal with history, completion and navigation/game commands.
- Configurable local game creator, saved draft, source view, sandboxed preview and standalone HTML export.
- Read-only Demiurge Devnet reader (`lib/chain.ts`, `/api/chain`): standard Substrate JSON-RPC at the fixed `https://rpc.qorsync.dev`, refusing any chain whose genesis is not `0x934e2caa…254a`; returns the finalized and best block numbers, or 503.
- Account lookup (`lib/account.ts`, `/api/chain/account?address=`, the QOR Identity screen): any address's test CGT (free, reserved, frozen, spendable while keeping the account open) and DRC-369 assets at the finalized block, decoded from the runtime's own encoding; a refused address returns 400, an unreadable chain 503; the last answer stays on screen marked stale. (P7.2.)
- WebMCP read-local-state and navigate tools. No purchases, signing, or reward mutations exposed.

## Not implemented or activated


This is a private playable explorer build, not a live gambling, payment, AI-generation or blockchain settlement system.

- QOR authentication/session verification and on-chain profile synchronization.
- A QOR linking/verifier service. Until supplied, the interface clearly labels Explorer aliases. Private site access is preserved; invitations do not admit otherwise unauthorized visitors.
- Automated report triage or an operator moderation dashboard; reports are persisted in the reports table. Message reads show up to 60 nondeleted messages from the last seven days; this is a display window, not a data-deletion retention policy.
- AI model/provider integration. The creator currently uses a local configurable template and says so.
- Real Energy purchases, payment webhooks, paid entitlements, refunds, and durable server ledger.
- CGT payouts, transaction signing and confirmation. CGT is the only currency; Energy and practice tokens are never convertible to it.
- Final Terms of Service, official promotion rules, operator identity, jurisdiction/age eligibility, purchase-free entry, and approved prize economy.
- Native installed CLI. The available CLI is a browser command console.

Do not connect client-local scores, energy, reward draws or storage to real value. A live implementation needs server-authoritative game sessions and economy, replay protection, idempotent payments/redemptions, verified QOR sessions, persistent audit records and chain confirmation handling.

## Protocol references

- The chain is `chain/` in `QOR-MATRIX/demiurge-chain`, a Substrate L1; the reader uses only standard Substrate RPC methods.
- Demiurge Devnet: `wss://rpc.qorsync.dev` (HTTP JSON-RPC on the same host), genesis `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a` (`chain/DEPLOY-RAILWAY.md`). Its CGT is test CGT.
- The earlier `chain_getBlockNumber` adapter at `rpc.demiurge.cloud` named a method of the deleted custom chain and an address that never served this one; it is gone.

## Verification

TypeScript check and local production build passed. Browser checks exercised Orbital and Synapse completion with exactly one debit and reward result, runner rendering/movement/pause, terminal game launch, local ledger, creator save and runnable generated game. Mobile overview fit the viewport without horizontal overflow. WebMCP valid navigation/read and invalid navigation rejection were exercised.

Polish pass: `node --test tests/arena.test.mjs` passes seven test groups including 100 complete Reversi games, Flux wins/illegal moves, identity and origin checks, simultaneous-move contention, exactly-once awards, early concessions, timeouts, SQL transaction rollback, chat limits/ownership, and aggregate rankings. An additional production-worker HTTP check exercised two isolated local identities through an entire Flux match, persistent rankings and cross-player chat delivery/deletion; unauthenticated requests returned 401. All test players/messages exist only in local test storage and are not deployment seeds.

Browser verification also exercised local sign-in, lobby create/cancel, invitation copy feedback, chat send/delete, Rift Survivor WebGL rendering, upgrade selection and pause, and desktop/mobile layouts. New match results and global rankings remain server-side; practice scores and the Energy cache remain device-local.

User-facing labels deliberately distinguish local state from real chain balances. No payment details, passwords, wallet secrets, or authentication tokens are collected.
