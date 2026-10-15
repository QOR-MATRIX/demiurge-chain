# Where Demiurge stands

For Andrew, 15 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this afternoon)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8, block
  170,917. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`, healthy. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*.

## Publishing a game (P3.4): your tries today

**Publish** in QQ Studio turns your game into a **Cartridge** — *the game as one on-chain
asset* — after the launcher asks you.

- **Cause found:** publishing signs with your vault's **first** account, `5Gx19…5sPwt`,
  which held nothing on the test chain; I had funded a different one. Your second try
  proved it: the launcher now says which account is empty, before asking you anything.
  That account now holds **35,000** test CGT, read from the chain this afternoon (my notes
  said 5,000; I can't say who sent the rest). Nothing is held back yet, so no publish has
  gone through.
- **Pull request #29** is open with all of it. **Its checks:** chain, QOR ID, ARQADE and the
  launcher pass (the launcher's first run on Linux). **Security fails**, from security
  notices published this week, not from this work. I updated the two libraries that had
  fixes. The third, `ruzstd` — *a decompression library* — has no fix and sits in a part
  of a library the launcher never uses. Silencing it is your call:
  **ADR-089** — *a written decision* — lists the options.
- **Re-checked this afternoon, with the library updates:** chain 150 tests; launcher 240,
  plus its live tests against a test chain, publishing included; QOR ID's 62 tests that need
  no database (its other 110 need Postgres and Redis, which this PC isn't running: GitHub
  runs them). The only audit failure left is `ruzstd`. QQ is unchanged since its last commit;
  its 73 tests are recorded as passing then, not re-run (they open windows on your screen).
- **Committed** on this PC as a third commit on the branch, not yet on GitHub.

## What only you can do

1. **Publish again**: QQ page → **Open in QQ Studio** → **Publish** → approve
   **"Approve this mint"**. Tell me what the Studio says.
2. **Decide ADR-089** (`docs/decisions/ADR-089-…`). I recommend **A**: ignore that one
   notice, with an automatic check that fails if the launcher ever uses that code — the
   same arrangement you approved for QOR ID on 14 September. Until you decide, the
   security check stays red on every branch, `main` included.
3. **Decide:** should publishing sign with the **first** account, or the one in the rail?
4. **Push** today's third commit: `git push matrix session/qq-p3.4-publish` (pull request #29
   picks it up).
5. **Update QQ Studio** with today's fixes: with it closed, run
   `pwsh products/qq/build.ps1 -Deploy`.
6. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
7. **CI for QQ:** add your Qt account to GitHub's secrets, or let your PC run CI (a
   *self-hosted runner*). Check with Qt that your licence allows it.
8. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
9. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
10. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`) in the project folder —
    delete it or tell me to; delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci`
    record and tunnel in Cloudflare; rotate nine old credentials; the nine frozen
    *home-map* tiles. Before commercial use: a paid Vercel plan, legal review, trademark
    clearance. Open questions: OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
