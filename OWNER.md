# Where Demiurge stands

For Andrew, 22 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 9;
**QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the gaming
website*. Launcher 0.1.14 is the tester download. **CI** — *GitHub's automatic tests* — is
green on `main`.

## This session

1. **The launcher updates itself** (L6.2, ADR-111): it checks for a new version, says so,
   and installs only when you press *Update and restart*, each installer checked twice
   against our own signing key. Proven end to end on this PC with a test copy.
2. **The records audited, as you asked.** Three checkers read every status line against
   the code, and I re-checked each finding before changing anything. All 42 finished
   roadmap items have their code; nothing claimed was missing. What was wrong was the
   records, mostly out of date: the **Mesh** — *our paid file-sharing network* — called
   unmerged, though live since 17 October; the chain's and QOR ID's numbers from early
   October; the launcher listed at 0.1.9; ARQADE's avatars called missing; a roadmap item
   (L4.7) that does not exist; the project rules calling the Spark the smallest unit (the
   ember is, per ADR-101). All corrected; SYSTEMS is rewritten short.
3. **A CI failure that kept coming back now explains itself.** Chrome sometimes opened no
   page, nothing said why, and a re-run always passed. Chrome now picks its own port, and
   a failure prints Chrome's own words.

**#76 is merged**, ADR-110 kept for later as you asked. **For your merge: #77**, with 1 to 3.

## The release gates (measured this session)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Terms of Service and a privacy policy, with a lawyer.** Three live services hold
   emails and messages, and none has either. Account deletion and export follow from them.
2. **The legal opinion** on rewards, prizes, the welcome grant and backing, and on
   **KYC** — *identity checks* —, sanctions and age limits, before anything carries value.
3. **The economic numbers** — *issuance, the genesis split, the decay curve, fee and burn
   shares* (OPEN-1 to OPEN-4, U-4, U-11). Most of the Beta gate waits on them.
4. **Code-signing certificates** for Windows and Apple, and an **external security review**.
5. **Back up the update key now:** `pwsh tools/qor-launcher/scripts/updater-key.ps1 -Backup`
   says which file and password to keep offline; lose them and testers reinstall by hand once.
6. **Say when to release 0.1.15**, the first launcher that updates itself. Testers install
   it by hand once; every release after reaches them from inside the launcher.
7. **Names** for the indexer and explorer, the SDK, and the **MCP server** — *how AI tools
   reach Demiurge*.
8. **Still open, as HANDOFF §4.0 records:** godmode's 100,000,000 CGT (its address); Social's
   report rules; the Android app on your phone; ARQADE inside 0.1.14; whether #46's gate
   change reads to you as loosening; the reward pool's numbers; Search Console and the GitHub
   preview image; the welcome-grant payer; Qt for CI; the email webhook; the key pasted on
   5 October. On this PC: QQ Studio is the copy of 15 October, before every step of
   ADR-105 (say the word and it is redeployed); `social-secrets.txt` on your Desktop and
   `archive-bZLIQp/` in the old checkout, each to keep or delete.

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
