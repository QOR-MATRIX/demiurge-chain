# Where Demiurge stands

For Andrew, 24 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 10,
two peers; **QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** —
*the gaming website*. **Launcher 0.1.16** is the tester download.

## This session

1. **PDFs in QOR Media — *the launcher's media app* — are done** and on your PC: read, pages
   changed, marked up, signed with a picture, and forms filled. The IRS's forms are shown but
   not filled: they use **XFA** — *an older Adobe form format*.
2. **CI's security check is green again.** You chose the guarded ignore for a flaw in a crypto
   library none of our code uses (ADR-115), and merged it with the rest.
3. **All four of ARQADE's games are rebuilt in 3D and merged (P7.20 done):** Orbital, Void
   Runner, Synapse and Rift Survivor. They play inside ARQADE's app, which is now **on your
   PC**, from `main`: open ARQADE in the launcher and press a game's card. Each game's rules are one small
   file that the game and a plain script both run, so a recorded game replays to the same
   score. That is how ARQADE's server will confirm a run before paying Sparks (P7.22).
4. **Sound now comes from where things happen,** as P7.20 asked: a lock from the satellite,
   Synapse's notes from left to right. You merged it (#93), and it is on your PC.
5. **You authorised full automation.** Claude Code's own safety check still refuses to merge
   without a person's review, so merges stay with you.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Play the four games** in the launcher and tell me what feels wrong.
2. **No pull request is waiting.** To let me merge future ones, add a permission rule for
   `gh pr merge`.
3. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
4. **For rewards in ARQADE:** decide **ADR-116** — *how a run is confirmed before it is
   paid* — and set the reward pool's numbers (largest payout, budget per epoch, most per player).
5. **Install 0.1.15 yourself, then take 0.1.16 when it offers it** (ticks L6.2). Then let a
   test agent ask you: in `X:\REPOZ\qq-interface\tools\qor-launcher\src-tauri` run
   `cargo run --example agent_link` and press *Allow agent* (ticks L5.1).
6. **Terms of Service and a privacy policy, with a lawyer,** and **the legal opinion** on
   rewards, prizes and **KYC** — *identity checks* — before anything carries value.
7. **The economic numbers** — *issuance, the genesis split, the decay curve, fee and burn
   shares* (OPEN-1 to OPEN-4, U-4, U-11). Most of the Beta gate waits on them.
8. **Code-signing certificates**, an **external security review**, and **names** for the
   indexer, the SDK and the **MCP server** — *how AI tools reach Demiurge*.
9. **Still open, as HANDOFF §4.0 records:** godmode's 100,000,000 CGT (its address); Social's
   report rules; the Android app on your phone; whether #46's gate change reads to you as
   loosening; Search Console and the GitHub preview image; the welcome-grant payer; Qt for CI;
   the email webhook; the key pasted on 5 October; `social-secrets.txt` on your Desktop and
   `archive-bZLIQp/` in the old checkout, each to keep or delete.

**Next in engineering:** the two arenas in QQ (P7.24), and a launcher release carrying QOR
Media's PDF work, assets' levels and the four games. At your word: filling the IRS's forms,
certified signatures (ADR-114), records kept by games other than an asset's home game
(ADR-113), or rental and fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
