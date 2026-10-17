# Where Demiurge stands

For Andrew, 17 October 2026 (afternoon). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this afternoon, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime version 8.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`.
- **Social** — *the chat service* — on Railway, with World, friends and circles,
  translation on each PC, and room levels with Elite and godmode. Your test is to come.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.9**, unsigned — *Windows warns on install* — built today with Social,
  the Library and avatars; the installer is on your Desktop.

## Merged today: the Library (#37)

- **One view of your own files** — *SteamOS-style* — videos, music, pictures and
  documents from folders you choose: resume, shelves, tabs, instant search. Nothing
  leaves your PC. A **game controller** works in it.
- **QOR Media** — *our Qt player, viewer and editor* — sits inside the launcher's window.
  It plays every format FFmpeg reads, shows pictures, and edits text, Markdown and code,
  saving files exactly as they were. It's installed on your PC.
- **CI is green on `main`.** Amazon's image server kept refusing our test database, so
  CI now uses Google's copy of the same images; no account needed.
- **A correction:** I told you Qt PDF was missing. It isn't; you've had it since
  27 September. Nothing to install: **PDF is next**, and I can start it.

## What only you can do

1. **Look at the Library** in a launcher built from `main`, when you're ready; I'll tick
   it done after your look.
2. **Test World, translation, friends and levels**; I'll tick them done.
3. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
4. **Keep the `godmode` account safe:** whoever signs in as it holds every room.
5. **Your Desktop file** `social-secrets.txt`: move it to where you keep credentials,
   or delete it. Its message key can't be recovered if lost.
6. **Update QQ Studio**, then publish again: `pwsh products/qq/build.ps1 -Deploy`.
7. **The last two L1.4 checks** — *the launcher's approval dialogs*; **CI for QQ**;
   **name the welcome-grant payer**; **add an email**; **replace the key** pasted
   into a chat on 5 October.
8. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`, still here); the old
   `QOR-MATRIX/arqade` repository.

## Cost per month

QOR Media and translation: $0; both run on each PC. Railway is not yet measured. Vercel
and Neon — *ARQADE's database host*: free plans, per earlier notes.
