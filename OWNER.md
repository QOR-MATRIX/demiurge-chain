# Where Demiurge stands

For Andrew, 25 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 10,
two peers; **QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** —
*the gaming website*. **Launcher 0.1.17 is published**; the homepage links it once you
merge **#102**, and offers 0.1.16 until then.

## This session

1. **Launcher 0.1.17 is out** (`launcher-v0.1.17` on GitHub). It carries everything merged
   since 0.1.16: ARQADE's four games and two arenas rebuilt in 3D, the `qor://arqade` link,
   the Dusk and Nocturne themes, an asset's level and XP, and QOR Media's PDF reading,
   marking and form filling. With 0.1.16 installed, Settings › Updates offers it.
2. **The release build caught a real bug, fixed before release:** a rematch in an arena could
   be lost. The app takes only the answer to its own request now, and a new test makes the
   problem happen deliberately on every run.
3. **Test windows on your PC lose the keyboard** to your Linux app "QTOS · Prism" from **WSL**
   — *Linux programs running inside Windows* — even when you are away. Six release attempts
   stopped on it. The tests now take the keyboard back; I left QTOS and its runner alone.
4. **Earlier today, all merged:** the arenas in 3D (P7.24), the website pointing to the
   launcher (P7.23), the `qor://arqade` link (ADR-118) and the level themes (**P7.16 done**).
5. **You authorised full automation.** Claude Code's own safety check still refuses to merge
   without a person's review, so merges stay with you.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Merge #102**, so the homepage offers 0.1.17; then **#103**, so each game on ARQADE's
   website offers itself in the launcher, beside the download.
2. **Take 0.1.17 through Update and restart** (ticks L6.2), then **play an arena with a
   friend:** both sign in to the launcher, one opens a match in ARQADE, the other joins it.
   That is the one test I cannot run.
3. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
4. **For rewards in ARQADE:** decide **ADR-116** — *how a run is confirmed before it is
   paid* — and set the reward pool's numbers (largest payout, budget per epoch, most per player).
5. **Let a test agent ask you:** in `X:\REPOZ\qq-interface\tools\qor-launcher\src-tauri` run
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

**Next in engineering, built and waiting for you:** #103, ARQADE's website offering each game
in the launcher now that 0.1.17 answers the link. At your word: filling the IRS's forms, certified
signatures (ADR-114), records kept by games other than an asset's home game (ADR-113), or
rental and fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
