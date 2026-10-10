# Where Demiurge stands

For Andrew, 10 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`; signs you in to ARQADE's window. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. QQ's first milestone (P3.1) is merged.

## Done today: QQ worlds play (P3.2)

- **Play** (F5) in QQ Studio runs the scene; **Stop** (Esc) puts it back exactly as it was.
- Crates fall and land; a **player** walks, runs and jumps with the keyboard, the mouse or a
  gamepad, with a camera that follows; sparks and glows; sounds placed in the world.
- **Game logic** lives in small files you can edit while the game runs: save, and it changes.
- The **QQ Player** runs a game on its own: on your PC its first picture comes in 1.4 s; in
  Chrome in 6 s, a 12 MB download once compressed.
- 42 automated checks pass; five deliberate faults were each caught.
- A crash in the new checks turned out to be the test's own bug, found and fixed.

Try it: `pwsh products/qq/build.ps1 -Run`, Add a Player, press F5.

**Limits:** in a browser, sounds get quieter with distance but do not pan left and right (a Qt
limit, recorded as ADR-084 — *a decision record*). Game logic is full code, so before strangers
play each other's games we must decide how to fence it in.

## Next work

**P3.3:** an AI agent that designs and builds a game from your written description, inside the
Studio, with you approving before anything is saved.

## What only you can do

1. **Merge P3.2** when its pull request is opened.
2. **CI** — *automatic build and test* — **for QQ:** GitHub cannot build Qt without your licence. Add your Qt account to the
   repository's secrets, or let your PC run CI jobs (a *self-hosted runner*).
3. **Check with Qt** that your licence covers CI, AI assistants and shipping the browser Player.
4. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
5. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
6. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and
   tunnel in Cloudflare; rotate nine old credentials; decide the nine frozen *home-map* tiles.
7. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
8. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-18, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. No spending cap (your decision, 1 October). Qt: your Enterprise licence.
