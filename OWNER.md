# Where Demiurge stands

For Andrew, 15 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this morning)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`, healthy. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*.

## Publishing a game (P3.4): your try this morning

**Publish** in QQ Studio turns your game into a **Cartridge** — *the game as one on-chain
asset* — after the launcher asks you.

- **Your publish was refused** before reaching the chain, and the message was cut off.
  The likely cause: publishing signs with your vault's **first** account, and the test
  CGT went to a different one, so the chain saw an empty account. Its own wording,
  "unable to pay fees", is misleading: this chain has no fees.
- **Fixed:** the launcher now checks first and says *which account* is empty *on which
  chain*, before asking you anything. QQ Studio shows the whole message on hover. Both
  have automatic checks, each proven to catch the old behaviour.
- **Re-checked today:** launcher 240 tests, every live chain test against a test chain,
  all 73 QQ tests.
- **Saved:** P3.4 is committed on this PC (two commits), not yet on GitHub (below).

## What only you can do

1. **Push to GitHub** — *my tools aren't allowed to*. In the project folder, run
   `git push matrix session/qq-p3.4-publish`. Then tell me, and I'll open the pull request.
2. **Retry the publish:** close the test launcher (tray icon → Quit; QQ Studio closes with
   it), tell me, and I'll start the new one. Then QQ page → **Open in QQ Studio** →
   **Publish**. If it names an empty account, I'll fund that one.
3. **Decide:** should publishing use the **first** account, or the one shown in the rail?
4. **Update QQ Studio** with today's fixes: with it closed, run
   `pwsh products/qq/build.ps1 -Deploy`.
5. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
6. **CI** — *automatic build and test* — **for QQ:** add your Qt account to GitHub's secrets,
   or let your PC run CI (a *self-hosted runner*). Check with Qt that your licence allows it.
7. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
8. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
9. **Housekeeping:** a GitKraken download (`archive-bZLIQp/`) appeared in the project folder
   this morning — delete it or tell me to; delete `QOR-MATRIX/arqade`; the Resend bounce
   webhook; the `ci` record and tunnel in Cloudflare; rotate nine old credentials; decide the
   nine frozen *home-map* tiles.
10. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
11. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20. Optional: prove
    P3.3 with your own Anthropic key.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
