# Where Demiurge stands

For Andrew, 8 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`. 172 tests on the branch.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. **CI** — *automatic build
  and test* — is green on main and passed on pull request #17.

## Decided today

**QQ is built on Qt 6** (ADR-083 — *decision record 83*), with your commercial licence: a
native **QQ Studio** for building 3D worlds, a **Player** for the launcher and for ARQADE's
browser, and an **agent** that designs and builds a game from a description. Bob had
written documents only; the 2D engine in the launcher was built this session and stays
as a preview until the Qt version plays.

## Done today (pull request #17, waiting for your merge)

- **QQ preview**: shapes, glow and particles you place, drag, play and save into a project.
- **ARQADE from the launcher** opens in its own window, **already signed in** with your
  launcher's QOR ID, even if you have no password; tips are approved in the launcher.
- **Six backdrops** — *the moving launcher background* — in Settings, each tested.
- **A flaky screen check fixed at its cause**, and a hidden startup cost removed.
- **Your avatar on ARQADE's sidebar.** A new README for the public repository.

## What only you can do

1. **Install Qt** with your Qt account (online installer): Qt 6.8 or newer, MSVC 2022 64-bit,
   Qt Quick 3D, Quick 3D Physics, Shader Tools, Spatial Audio, WebAssembly, Qt Creator.
   QQ's build starts then.
2. **Merge pull request #17.** QOR ID redeploys from main, which turns on the launcher's
   ARQADE sign-in.
3. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT* —
   and create and fund its Welcome account on the devnet.
4. **Check with Qt** that your licence covers CI runners and AI assistants on your machine.
5. **Install a launcher built from main** and try QQ and ARQADE's window.
6. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on
   5 October.
7. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook and its
   secret; the `ci` record and tunnel in Cloudflare; rotate the nine old credentials;
   decide the nine frozen Nexus — *home map* — tiles.
8. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
9. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-18, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. No spending cap (your decision, 1 October).
