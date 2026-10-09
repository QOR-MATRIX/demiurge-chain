# Where Demiurge stands

For Andrew, 9 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`; signs you in to ARQADE's window. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. **CI** — *automatic build
  and test* — green on main.

## Done today: QQ's first milestone (P3.1)

**QQ Studio**, on your Qt 6.12 Enterprise, now edits 3D worlds:

- a list of everything in the scene, and a panel to edit whatever is selected;
- click to select; handles to **move, turn and scale**; orbit and zoom;
- add cubes, spheres, cylinders, cones, lamps, or your own 3D models;
- **Save** keeps the scene in a project folder with version history; one edit shows as one
  changed line; your models are copied into the project so it is complete on its own;
- scenes from the launcher's 2D preview can be imported;
- the launcher's QQ screen has **Open in QQ Studio**.

23 automated checks pass, including ones that click and drag in the real window, and each
check was shown to fail when the Studio was deliberately broken.

Try it: `pwsh products/qq/build.ps1 -Run`.

## Next work

**P3.2, worlds that play:** physics, keyboard and gamepad control, particles, spatial sound,
live game logic, and a Player that runs a game in the launcher and in a browser. Then the
agent that designs and builds a game from a description.

## What only you can do

1. **Merge the pull request** for P3.1 when it is opened.
2. **CI for QQ:** GitHub cannot build Qt without your licence. Add your Qt account to the
   repository's secrets, or let your PC run CI jobs (a *self-hosted runner*).
3. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT* —
   and create and fund its Welcome account on the devnet.
4. **Check with Qt** that your licence covers CI and AI assistants on your machine.
5. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on
   5 October.
6. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook and its secret;
   the `ci` record and tunnel in Cloudflare; rotate the nine old credentials; decide the
   nine frozen Nexus — *home map* — tiles.
7. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
8. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-18, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. No spending cap (your decision, 1 October). Qt: your Enterprise licence.
