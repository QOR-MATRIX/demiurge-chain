# Where Demiurge stands

For Andrew, 17 October 2026 (early morning). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime version 8.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`.
- **Social** — *the chat service* — on Railway, with World, friends and circles,
  translation on each PC, and room levels with Elite and godmode since #35 merged.
  Your own test of those is still to come.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. Social is in `main`, not yet
  in an installer.

## New tonight: the Library, on branch `session/library` (not merged)

- **One view of your own files** — *SteamOS-style* — videos, music, pictures and
  documents from folders you choose, with a hero to resume, shelves, tabs and instant
  search. Nothing leaves your PC.
- **A game controller works in it**: the D-pad or stick moves, A opens, B goes back.
- **QOR Media** — *our Qt player, viewer and editor* — sits inside the launcher's window
  over the Library. It plays every format FFmpeg reads (MKV, MP4, HEVC, AV1, FLAC…),
  shows pictures, and edits text, Markdown and code, saving files exactly as they were.
- **Tested:** the launcher started the real QOR Media and it played and reported back.
  Nobody has opened a file from the Library on screen yet: that's your first look.
- **PDF is next.** It needs **Qt PDF**, which your Qt doesn't have yet.
- Office documents come in a later release, as you chose.

## What only you can do

1. **Merge #36** — *the notes saying #35 is live* — **then the Library's pull request**,
   once I open it; I'll settle any overlap between them.
2. **Add Qt PDF:** open the *Qt Maintenance Tool*, choose Add or remove components,
   tick Qt 6.12.0 → Additional Libraries → Qt PDF, and apply.
3. **Look at the Library** in a launcher built from the branch, when you're ready.
4. **Test World, translation, friends and levels**; I'll tick them done.
5. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
6. **Keep the `godmode` account safe:** whoever signs in as it holds every room.
7. **Your Desktop file** `social-secrets.txt`: move it to where you keep credentials,
   or delete it. Its message key can't be recovered if lost.
8. **Update QQ Studio**, then publish again: `pwsh products/qq/build.ps1 -Deploy`.
9. **The last two L1.4 checks** — *the launcher's approval dialogs*; **CI for QQ**;
   **name the welcome-grant payer**; **add an email**; **replace the key** pasted
   into a chat on 5 October.
10. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`, still here); the old
    `QOR-MATRIX/arqade` repository.

## Cost per month

QOR Media and translation: $0; both run on each PC. Railway is not yet measured. Vercel
and Neon — *ARQADE's database host*: free plans, per earlier notes.
