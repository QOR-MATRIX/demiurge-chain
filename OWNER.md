# Where Demiurge stands

For Andrew, 24 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 10,
two peers, still finalising; **QOR ID** — *the one sign-in*; **Social** — *the chat
service*; **ARQADE** — *the gaming website*. **Launcher 0.1.16** is the tester download.

## This session

1. **PDFs in QOR Media — *the launcher's media app* — are done** and on your PC: read, pages
   changed, marked up, signed with a picture, and forms filled. The IRS's forms are shown but
   not filled: they use **XFA** — *an older Adobe form format*.
2. **CI's security check went red** on a flaw in a crypto library the Polkadot SDK brings in;
   none of our code uses that part. **You chose the guarded ignore** (ADR-115). It is **#88**
   — *a pull request, a change for you to merge* — and every check on it passes.
3. **Orbital, the first of ARQADE's four games rebuilt in 3D (P7.20), plays inside ARQADE's
   app.** Its rules are kept in one small file. The game and a plain script both run that
   file, so a recorded game replays to the same score in each. That is how ARQADE's server
   will confirm a run before paying Sparks (P7.22). A test plays it by mouse, keyboard and a
   simulated gamepad. It is **#89**, built on #88. ARQADE's own games run without the lockdown that
   strangers' games get, because we wrote them; strangers' games still play only in a browser.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
2. **Merge #88, then #89.** #88 makes CI's security check pass again.
3. **Install 0.1.15 yourself, then take 0.1.16 when it offers it** (ticks L6.2). Then let a
   test agent ask you: in `X:\REPOZ\qq-interface\tools\qor-launcher\src-tauri` run
   `cargo run --example agent_link` and press *Allow agent* (ticks L5.1).
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

**Next in engineering:** Void Runner, then Synapse and Rift Survivor, as Orbital was. At your
word: a launcher release carrying QOR Media's PDF work, assets' levels and Orbital; filling
the IRS's forms; certified signatures (ADR-114); records kept by games other than an
asset's home game (ADR-113); or rental and fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
