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

1. **PDFs in QOR Media — *the launcher's media app* — are done**, now that you merged #86
   and #87: read, pages changed, marked up, signed with a picture, and forms filled. Tried on
   the real USCIS I-9 form. The IRS's forms are shown but not filled: they keep their answers
   in **XFA** — *an older Adobe form format* — which Acrobat would show instead.
2. **CI's security check went red on `main`** on a flaw published yesterday in a crypto
   library the Polkadot SDK brings in. No fix can reach us yet, and none of our code uses
   the part it concerns. **You chose the guarded ignore**: it is written up as ADR-115, CI
   proves its conditions every run, and it is **#88** — *a pull request, a proposed change
   for you to merge*. Merging it turns the check green.
3. **Next is P7.20** — *ARQADE's four games rebuilt in 3D* — starting with Orbital.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
   Lose them and testers reinstall by hand.
2. **Merge #88**, which makes CI's security check pass again.
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

**Next in engineering:** P7.20, Orbital first, then Void Runner, Synapse and Rift Survivor.
At your word: a launcher release carrying QOR Media's PDF work and assets' levels, filling
the IRS's forms (by removing their XFA, as other tools do), certified signatures (ADR-114),
games other than the home game keeping records (ADR-113's second phase), or rental and
fractions (M4.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
