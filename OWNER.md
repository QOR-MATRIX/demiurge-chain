# Where Demiurge stands

For Andrew, 16 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime version 8.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`.
- **Social** — *the chat service* — on Railway. Your first conversation worked
  ("Successful"), so the launcher's Social screen, profiles and presence are ticked done.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. Social is in `main`, not yet
  in an installer.

## Built tonight, waiting for your merge (ADR-092)

- **World** — *one room for everyone* — opens when Social does. Kept in full, as you
  chose. Moderators you name delete messages and mute for 10 minutes to a week.
- **Friends and circles** — *your private friend groups* — asked by username; a circle
  opens a group chat in one click.
- **Translation on each PC** — *free, and private* — World and public rooms appear in your
  language, marked "Translated from Spanish", the original one click away. It uses
  Mozilla's engine, the one Firefox uses. Each new language pair downloads about 24 MB
  once. Tested for real tonight: Spanish became "Hi everyone, how are you today? I love
  this game." in a fifth of a second.
- **Chat shows** each person's avatar, level and username.

## What only you can do

1. **Review and merge the pull request** for the above. Railway updates Social from `main`.
2. **Say who moderates World** (your QOR ID username?), and I'll set it on Railway.
3. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
4. **Then your look at World:** I'll start a test launcher built from `main`.
5. **Your Desktop file** `social-secrets.txt` is still there: move it to where you keep
   credentials, or delete it. Its message key can't be recovered if lost.
6. **Update QQ Studio**, then publish again: with it closed, run
   `pwsh products/qq/build.ps1 -Deploy`; then **Publish**, and send me its message.
7. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
8. **CI for QQ:** your Qt account in GitHub's secrets, or a *self-hosted runner* — *your PC
   running GitHub's checks*.
9. **Name the welcome-grant payer**; **add an email** at `id.qorsync.dev/account`; **replace
   the key** pasted into a chat on 5 October.
10. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`, still here); the old
    `QOR-MATRIX/arqade` repository (still there); from earlier notes, not re-checked: the
    Resend bounce webhook, Cloudflare's `ci` record and tunnel, nine old credentials.

## Cost per month

Translation: $0 per message; it runs on each PC. Railway is not yet measured. Vercel and
Neon — *ARQADE's database host*: free plans, per earlier notes. AI generation: $0, by rule.
