# Where Demiurge stands

For Andrew, 25 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 10,
two peers; **QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** —
*the gaming website*. **Launcher 0.1.16** is the tester download.

## This session

1. **All four of ARQADE's games are rebuilt in 3D, merged and on your PC (P7.20 done)**,
   each sound heard from where it happens. Open ARQADE in the launcher and press a card.
2. **The two arenas are rebuilt in 3D too, merged and on your PC (P7.24 done).** Flux Four
   and Rift Reversi play inside ARQADE's app against another person, on ARQADE's own server:
   the same matches and rankings as the website. A game made in **QQ** — *our game engine* —
   cannot reach the internet, so it asks the app it runs in, which answers from the server.
   That is a new **host channel** — *how a game talks to its app* — decided under your
   delegation (ADR-117). Tested against a stand-in server, not yet between two real people.
3. **The website's games now point players to the launcher, live (P7.23 done):** each game's
   first and last screen says it is rebuilt in 3D there, with a link to download it.
4. **A link that opens a game in the launcher is merged (#99), in no release yet:**
   `qor://arqade?play=flux` brings the launcher forward at ARQADE with Flux Four open. It pays,
   signs and signs in nothing (ADR-118, under your delegation). The website can offer it once a
   launcher release carries it; an older launcher would refuse it.
5. **You authorised full automation.** Claude Code's own safety check still refuses to merge
   without a person's review, so merges stay with you.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Play an arena with a friend:** both sign in to the launcher, one opens a match in
   ARQADE (Flux Four or Rift Reversi), the other joins it. That is the one test I cannot run.
2. **Play the four games** too, and tell me what feels wrong. To let me merge pull
   requests myself, add a permission rule for `gh pr merge`.
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

**Next in engineering, at your go:** launcher 0.1.17, carrying QOR Media's PDF work, assets'
levels, the games, the arenas and the new link; then the website offering that link. At your word: filling the IRS's forms, certified signatures (ADR-114), records
kept by games other than an asset's home game (ADR-113), or rental and fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
