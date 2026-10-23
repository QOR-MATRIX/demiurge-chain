# Where Demiurge stands

For Andrew, 23 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 9;
**QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the gaming
website*. **Launcher 0.1.16** is the tester download.

## This session

1. **Runtime 10 is rehearsed, and passed.** Runtime — *the chain's program* — 10 is what
   lets assets level up. On this PC I started a copy of the devnet running its own live
   runtime 9, left a payment and an asset on it, then upgraded it exactly as you will
   sign it. All 35 checks passed: nothing was lost, and wallets, the Mesh and the new
   level and **XP** — *experience points* — records all work after it. The file is in
   your Downloads: `demiurge-runtime-v10.compact.compressed.wasm`.
2. **PDFs open in the Library** (#83). A PDF now opens inside the launcher's media app,
   **QOR Media**, instead of another program: zoom, find with every match marked, copy
   text, follow links (only to the web or email, never to a program), see the pages and
   contents beside it, and open a locked PDF with its password. Tested (51 tests; seven of
   eight planted bugs caught; the eighth is guarded twice) and tried on a real 72-page PDF.
   Marking up, filling, signing and rearranging pages come next.
3. **For your merge: #82 and #83**, in either order. #82 teaches assets to level up
   (ADR-113, your approval): a level, XP and stats written by the game the collection's
   owner picks; only whoever holds the game's **Cartridge** — *the game's own asset* — can
   correct a mistake, and nothing changes while an asset is for sale.

## The release gates (measured last session)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
   Lose them and testers reinstall by hand.
2. **Merge #82, then upgrade the devnet to runtime 10**, as you did for 9: in polkadot.js
   Apps, Developer → Sudo, `sudoUncheckedWeight`, then `system.setCode` in the second
   dropdown (the field must read `code`, not `remark`), and pick the file above. Its
   fingerprint starts `6192023b`. Tell me when it is done and I will read it back.
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

**Next in engineering:** changing PDFs in QOR Media (the rest of L9.5): marking up,
filling, signing and rearranging pages. At your word: games other than the home game
keeping records (ADR-113's second phase), or rental and fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
