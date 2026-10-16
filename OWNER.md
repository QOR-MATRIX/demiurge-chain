# Where Demiurge stands

For Andrew, 16 October 2026 (afternoon). One page, rewritten every session.

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

## Social is online (ADR-091)

- **Live since today** on Railway, beside QOR ID: the Social service — *the server behind
  the chat* — with rooms, private rooms, direct messages, replies, reactions, edits, unread
  counts, @mentions, blocking, reports, online/away/playing, and profiles. Checked from
  outside: it answers, and QOR ID vouches for it.
- **The launcher's Social screen** now points at it (pull request #34). Your first
  conversation needs a launcher built with that change: I can start a test one for you.
- **Two snags fixed on the way:** QOR ID hadn't been told about Social (now done), and
  Social held a different secret from the one QOR ID expected (you replaced it). The secret
  you pasted in chat was never registered, so it unlocks nothing.
- **Your Desktop file** `social-secrets.txt`: move it to where you keep credentials, or
  delete it. The message key in it can't be recovered if lost.

## Publishing a game (P3.4)

Minting "QQTest1" from the launcher worked; **Publish inside QQ Studio did not**, and the
Studio's message wasn't recorded. Your QQ Studio is from 10 October and lacks two fixes.
The launcher now logs every request a product makes, so the next try shows the cause.

## What only you can do

1. **Merge pull request #34** (the launcher pointed at the live Social) once its checks pass.
2. **Your first Social conversation:** tell me, and I'll start a test launcher.
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
