# Where Demiurge stands

For Andrew, 23 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 10,
two peers, still finalising; **QOR ID** — *the one sign-in*; **Social** — *the chat
service*; **ARQADE** — *the gaming website*. **Launcher 0.1.16** is the tester download.

## This session

1. **PDFs can be marked up in QOR Media** — *the launcher's media app*: highlight,
   underline or strike through words, add notes, draw by hand, and place a picture of
   your signature, drawn or typed in a handwriting face. Marks show at once, undo, and
   are saved into the PDF, so any PDF reader shows them. A PDF's own marks can be taken
   away, and its notes read by a tap.
2. **The signature is a picture, as you chose:** QOR Media says so where it is placed;
   it certifies nothing. Certified signatures still wait on you (ADR-114).
3. **It waits on #85.** It is built on #85's branch, so it becomes a pull request —
   *a proposed change for you to merge* — once #85 is in. Not yet in any release.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
   Lose them and testers reinstall by hand.
2. **Merge #85** when you have looked at it; then the marking up goes up for your merge.
3. **Install 0.1.15 yourself, then take 0.1.16 when it offers it**: the first update
   through the channel, which ticks L6.2. Then let a test agent ask you: in
   `X:\REPOZ\qq-interface\tools\qor-launcher\src-tauri` run `cargo run --example agent_link`
   (or ask me to), and press *Allow agent* in the launcher's dialog. That ticks L5.1.
4. **Terms of Service and a privacy policy, with a lawyer.** Three live services hold
   emails and messages, and none has either. Account deletion and export follow from them.
5. **The legal opinion** on rewards, prizes, the welcome grant and backing, and on
   **KYC** — *identity checks* —, sanctions and age limits, before anything carries value.
6. **The economic numbers** — *issuance, the genesis split, the decay curve, fee and burn
   shares* (OPEN-1 to OPEN-4, U-4, U-11). Most of the Beta gate waits on them.
7. **Code-signing certificates** for Windows and Apple, and an **external security review**.
8. **Names** for the indexer and explorer, the SDK, and the **MCP server** — *how AI tools
   reach Demiurge*.
9. **Still open, as HANDOFF §4.0 records:** godmode's 100,000,000 CGT (its address); Social's
   report rules; the Android app on your phone; ARQADE inside the launcher; whether #46's
   gate change reads to you as loosening; the reward pool's numbers; Search Console and the
   GitHub preview image; the welcome-grant payer; Qt for CI; the email webhook; the key
   pasted on 5 October; `social-secrets.txt` on your Desktop and `archive-bZLIQp/` in the
   old checkout, each to keep or delete.

**Next in engineering:** filling in PDF forms, the last of L9.5. At your word: certified
signatures (ADR-114), a launcher release with QOR Media's PDF work and assets' levels,
games other than the home game keeping records (ADR-113's second phase), or rental and
fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
