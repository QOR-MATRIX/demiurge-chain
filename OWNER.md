# Where Demiurge stands

For Andrew, 15 October 2026 (night). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`. Since you merged #31 it includes
  what Social needs, the one-time name pick among it: 176 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. `main` passes every check.

## Social (ADR-091, your four choices)

- **In `main`:** the Social service — *the server behind the chat* — with rooms, private
  rooms, direct messages, replies, reactions, edits, unread counts, @mentions, blocking,
  reports, online/away/playing, and profiles. Not online yet.
- **Built tonight, waiting for your review (pull request #33):** the **Social screen in the**
  **launcher**. Your conversations with unread counts (also on the rail), messages with
  reactions and replies, @name completion, who is typing, profile cards with a living
  banner and your colour, a profile editor with a live preview, and the name pick on entry.
  Proven by a new screen check (29 points) that caught three planted faults and two real
  ones, fixed before you see it.
- **Tried together for real on this PC:** QOR ID and Social, two people, rooms, messages,
  mentions, a direct message and the one-time name pick: 12 of 12. The launcher against a
  running Social waits on putting Social online; that first conversation is yours.
- **Pull request #32:** Social's library check on QOR ID's terms, as you chose. It missed #31.

## Publishing a game (P3.4)

Minting "QQTest1" from the launcher worked; **Publish inside QQ Studio did not**, and the
Studio's message wasn't recorded. Your QQ Studio is from 10 October and lacks two fixes.
The launcher now logs every request a product makes, so the next try shows the cause.

## What only you can do

1. **Review and merge** pull requests **#30**, **#32** and **#33** when their checks pass.
2. **Put Social online:** the steps are written in `services/social/DEPLOY-RAILWAY.md` (a
   Railway service and database, two secrets made on your PC, one line in QOR ID's settings).
3. **Update QQ Studio**, then publish again: with it closed, run
   `pwsh products/qq/build.ps1 -Deploy`; then **Publish**, and send me its message word for word.
4. **Before Social opens to anyone else:** how long messages are kept, and the rules reports
   are judged against (L8.7).
5. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
6. **CI for QQ:** add your Qt account to GitHub's secrets, or a *self-hosted runner* — *your
   PC running GitHub's checks*. Check with Qt that your licence allows it.
7. **Name the welcome-grant payer**; **add an email** at `id.qorsync.dev/account`; **replace
   the key** pasted into a chat on 5 October.
8. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`); delete `QOR-MATRIX/arqade`;
   the Resend bounce webhook; the `ci` record and tunnel in Cloudflare; rotate nine old
   credentials; the nine frozen *home-map* tiles. Before commercial use: a paid Vercel
   plan, legal review, trademark clearance. Open questions: OPEN-1 to OPEN-4, U-4, U-14 to
   U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37; Social will add about $5–15). Vercel and Neon —
*ARQADE's database host*: $0 on free plans. AI generation: $0, by rule. No spending cap
(your decision, 1 October). Qt: your Enterprise licence.
