# Where Demiurge stands

For Andrew, 9 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`, redeployed after your merge: the
  launcher now signs you in to ARQADE's window. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. **CI** — *automatic build
  and test* — green on main.

## Done today

**QQ's own engine is running**, on the Qt 6.12 Enterprise you installed. **QQ Studio** opens
a 3D world: a sky that lights the scene, a sun with soft shadows, filmic colour, bloom, and
a 3D model loaded straight from a file. You can orbit, zoom, and set the sun and exposure.
Three tests render real frames and judge them by their pixels; all pass, and each was shown
to fail when the engine was deliberately broken.

To see it: `pwsh products/qq/build.ps1 -Run` from the repository folder.

## Next work

The rest of P3.1: the scene tree, an inspector, move-rotate-scale handles, saving scenes
into a project, and **Open in QQ Studio** from the launcher. Then worlds that play
(physics, input), then the agent that designs and builds.

## What only you can do

1. **Merge the next pull request** when it is opened (QQ's engine foundation).
2. **CI for QQ:** GitHub cannot build Qt without your licence. Either add your Qt account to
   the repository's secrets, or let your PC run CI jobs (a *self-hosted runner*). Until then
   QQ is tested on this PC.
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
