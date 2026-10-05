# ARQADE integration inventory

**What this is.** Every control, statistic, status and terminal command ARQADE shows, with where its value comes
from, who may use it, where it persists, what happens when it fails, and how it was verified. Required by P7.1
(`docs/DIRECTION.md`) and the blueprint's §4 (`docs/blueprints/arqade.md`). Written 5 October 2026 against
`products/arqade/` as it is in the tree; checked in a running build (the site's own workerd runtime, driven in headless
Chrome) the same day.

**Read with.** `IMPLEMENTATION.md` (what is built), the SDK's `README.md`. When a control changes, its row changes in
the same commit.

## Sources of truth, in short

| Source | What lives there | Who can change it |
| --- | --- | --- |
| **The browser** (`localStorage`) | `demiurge.local.v1`: practice Energy, practice tokens, plays, best scores, session ledger. `demiurge.portal.v1`: intro seen. `demiurge.project.v1`: the creator's draft. `demiurge.muted`: muted chat players. Keys kept from before the rename, so nobody loses progress | The visitor, on this device only. **None of it has value or reaches the chain** |
| **The arcade service** (Cloudflare D1 behind the Sites host) | Players (an `Explorer-…` alias from the host's signed-in user), matches, standings, chat messages, reports | The server, from authenticated requests; rules in `lib/arena-engine.ts`, writes in `lib/arena-store.ts` |
| **Demiurge Devnet** (`https://rpc.qorsync.dev`, read-only) | Finalized and best block, any account's test CGT and DRC-369 assets | Nobody, from here. ARQADE signs nothing; the genesis is checked before every read |
| **The code** | The four solo games, their rules and copy | A release |

## Shell and navigation

| Control or display | Source | Permission | Persists | On failure | Verified |
| --- | --- | --- | --- | --- | --- |
| Brand "ARQADE · powered by Demiurge", page title | Code | — | — | — | Chrome, 5 Oct: title "ARQADE — powered by Demiurge" |
| Sidebar views (Overview, The Arcade, Multiplayer, Leaderboard, Creation Engine, Terminal, Energy Store, Rewards & Wallet) and URL `#hash` | Code; the view is read from the hash | Anyone | The hash | Unknown hash stays on the current view | Chrome: navigation by sidebar and by hash |
| "Your cybercade · Season Zero" | Static copy | — | — | — | Read in the code |
| Top bar Energy count | Browser (`demiurge.local.v1`) | Visitor | Browser | Unreadable storage resets to the initial 1,000 practice Energy | `page.tsx` validates every stored field |
| **Connect QOR ID** | Opens the QOR Identity view, where **Sign in with QOR ID** starts the sign-in on QOR ID's own page (P7.3) | Anyone | — | — | Chrome: opens the view |
| Live strip ("network … online", open lobbies, Global rankings) | Arcade service (`/api/live`, polled every 2 s while visible) | Host-authenticated visitor | — | Shows "connecting" and "—" when the service is unreachable or the visitor is not signed in to the host | `tests/arena.test.mjs`; locally it shows "connecting" (no host sign-in) |
| Sidebar panel "Demiurge Devnet · Read-only · see QOR Identity" | Static pointer. **Was "Chain connection pending", which never read anything; replaced 5 Oct** | — | — | — | Chrome |
| "Unbound explorer · Explorer alias" | Arcade service: alias derived from the host's user id. **Not a QOR identity** | Host-authenticated visitor | Server | Unauthenticated: no alias | `tests/arena.test.mjs` (identity is server-derived) |
| Portal intro and its notice | Code; seen flag in browser | Anyone | Browser | Storage blocked: the intro shows every visit | Chrome |
| Footer "Connected arcade · No financial transactions · Demiurge Protocol · Portal notice / Replay intro" | Code | — | — | — | Read in the code |

## Overview and stats

| Display | Source | Notes | Verified |
| --- | --- | --- | --- |
| Hero, "Launch Rift Survivor" | Code | Opens the game | Chrome |
| Energy cache | Browser | Practice only, labelled local | Chrome |
| Play rewards ("Energy tokens") | Browser, 0–3 per finished game, from Web Crypto | **Practice tokens, no value**, labelled "local play · no cash value". Never become CGT (ADR-069 decision 7) | Chrome |
| Worlds available | Code | Four solo worlds, two shared arenas | — |
| Creator engine status | Browser (draft exists or not) | — | — |

## Solo games (Void Runner, Synapse, Orbital, Rift Survivor)

| Control | Source | Permission | Persists | On failure | Verified |
| --- | --- | --- | --- | --- | --- |
| Open game, **Begin session** (costs 20 practice Energy) | Browser | Visitor | Best score and ledger in browser | Under 20 Energy: "Refill your local energy" | Chrome, 5 Oct: each of the four opened and began |
| Play (keys, pointer, touch), pause, sound, reduced motion | Code | Visitor | — | Hidden tab pauses | Chrome: Void Runner scored, Synapse advanced a round on a real key press, Orbital ended after three misses, Rift Survivor scored with a combo; no page errors |
| Score and result | Browser | **Never submitted, never ranked, never paid** | Browser | — | `IMPLEMENTATION.md`; nothing in the code sends a solo score |

## Multiplayer arenas (Flux Four, Rift Reversi), leaderboard and chat

| Control | Source | Permission | Persists | On failure | Verified |
| --- | --- | --- | --- | --- | --- |
| Create lobby, invitation link, close lobby, join, cancel | Arcade service | Host-authenticated visitor; origin checked | Server (D1) | Error with **Retry**; revision guard refuses a stale write | `tests/arena.test.mjs` (lifecycle, racing moves) |
| Board moves, legal-move hints, 90-second turns, **Concede match**, rematch (**Find another match**), **Back to lobby** | Server rules (`lib/arena-engine.ts`) | The two players; spectators read | Server | A refused move leaves the board unchanged | Tests: 100 Reversi games, Flux wins and illegal moves, exactly-once results, timeouts |
| Leaderboard (global top 100, per game) | Server standings | Anyone signed in to the host | Server | "Offline" when unreachable | Tests: aggregation before the limit |
| World chat: send, **Delete** own message, **Mute player**, **Report**, unread count | Server; mute list in browser | Host-authenticated; delete only one's own | Server; mute in browser | Draft kept on failure; two-second send limit | Tests: identity, rate limits, ownership |

**Not yet:** results are bound to the host alias, not a QOR identity (P7.3); polling, not WebSockets.

## Creation Engine

| Control | Source | Notes | Verified |
| --- | --- | --- | --- |
| Name, concept, colour, speed, duration; Save; Preview; Source; Export HTML | Browser (`demiurge.project.v1`), a local template | **Not an AI**: the "AI" note says a provider is not connected. The preview runs sandboxed; export is a standalone file marked "Made in ARQADE · Free local game · No prizes or transactions" | Read in the code; the stored draft is validated on load |

## Energy Store, Rewards & Wallet

| Control | Source | Notes | Verified |
| --- | --- | --- | --- |
| Practice Energy packages, **View package**, refill | Browser | **Free and practice only**. No cash prices since 4 Oct; the package dialog says checkout is not connected | Chrome |
| Practice-token balance, meter, session ledger, ledger download (`arqade-local-ledger.json`) | Browser | Labelled device-local | Read in the code |
| "Practice tokens stay practice" panel, **Not convertible** (disabled) | Code | States that Energy and practice tokens are never CGT | Chrome |

## QOR Identity

| Control | Source | Permission | Persists | On failure | Verified |
| --- | --- | --- | --- | --- | --- |
| QOR authentication / Verified account / Chain account / CGT settlement rows; **Sign in with QOR ID**, **Sign out** (P7.3) | QOR ID, through ARQADE's server (`/api/auth/me`, which asks `/oauth/userinfo`); CGT settlement is still **Not connected** | Anyone may sign in; sign-out from this site's own pages only | Server-side session (D1, keyed by the cookie's hash), one HttpOnly cookie | Unconfigured site: "Not configured", no button. QOR ID unreachable: says so. A refused sign-in: "did not complete. Nothing was shared" | `tests/qor-session.test.mjs`; end to end in Chrome against a local QOR ID, 5 Oct |
| **Open Demiurge portal** | Link to `demiurge.cloud` | — | — | — | — |
| **Check chain connection** | Devnet (`/api/chain`): genesis and name checked, finalized block | Anyone | — | "Wrong network" or "could not be read" (503) | `tests/chain.test.mjs`; route in workerd |
| Account **Look up** | Devnet (`/api/chain/account`): test CGT and DRC-369 assets at the finalized block | Anyone; read-only | — | Refused address: the reason (400). Unreadable: the last answer stays, marked stale (503) | `tests/account.test.mjs` (bytes from a real node); live devnet matched `@polkadot/api`; Chrome lookup and refusal |

## Terminal (`arqade — local session`, "NO CHAIN SIGNER")

| Command | Does | Source | Verified |
| --- | --- | --- | --- |
| `help`, `clear`, `worlds` | Lists commands, clears, lists the four worlds | Code | Read in the code |
| `play void\|synapse\|orbital\|rift` | Opens a game | Code | Read in the code |
| `energy`, `rewards`, `wallet` | Reads practice Energy and tokens; opens the ledger | Browser | Read in the code |
| `create`, `qor`, `intro` | Opens the creator, the identity view, the intro | Code | Read in the code |
| `chain` | Reads the devnet's finalized block. **Fixed 5 Oct**: it read a field the devnet reader no longer returns ("block #undefined") | Devnet | Chrome: "Demiurge Devnet · finalized block #20897 · Verified by its genesis" |

## Machine interfaces

| Interface | Does | Permission | Verified |
| --- | --- | --- | --- |
| `GET /api/chain` | Devnet heads, genesis-checked | Anyone | Tests, workerd |
| `GET /api/chain/account?address=` | One account's test CGT and assets | Anyone | Tests, workerd |
| `GET /api/auth/login`, `GET /api/auth/callback`, `GET /api/auth/me`, `POST /api/auth/logout` | Start a QOR ID sign-in, finish it (state checked against the browser's cookie, code exchanged with PKCE and the site's secret), who is signed in, sign out | Anyone; logout same-origin only | Tests; Chrome against a local QOR ID |
| `/api/live`, `/api/matches`, `/api/matches/[id]`, `/api/chat` | Arenas, standings, chat | Host-authenticated; origin checked | `tests/arena.test.mjs`, `tests/worker.smoke.mjs` (needs a running worker) |
| WebMCP tools `read_local_arcade`, `navigate_demiurge` | Read practice state; open a section. Names kept for agents already using them | The page | Read in the code |

## What is deliberately absent

No signing, no payment, no CGT payout, no asset award and no ranking from solo play. Each is a later P7
step, named in `docs/DIRECTION.md`.
