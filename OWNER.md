# Where Demiurge stands

For Andrew, 22 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 9;
**QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the gaming
website*. **Launcher 0.1.15** is the tester download, the first that updates itself.

## This session

1. **Launcher 0.1.15 is out** (GitHub: `launcher-v0.1.15`). Testers install it by hand
   once; after that, each release reaches them inside the launcher when they press
   *Update and restart*. It also brings Obsidian, the one look, everywhere. I downloaded
   it back from GitHub and checked it against our signing key before calling it done.
2. **QQ Studio on this PC is current**: rebuilt from today's code, all eight test suites
   passed, and the launcher opens it. The old copy was from 15 October.
3. **Found and fixed on the way:** the Mesh's permission dialog — *the paid file-sharing
   network's one question* — has shown gaps in its sentences since 0.1.10, and a test now
   refuses any dialog text like it; the release could not build from the current
   checkout; a GitHub cache hang stalled **CI** — *GitHub's automatic tests* — for 30
   minutes, now capped at ten.
4. **Your merge of #77** brought the update channel and the corrected records to `main`.

**For your merge: #78** — 0.1.15's version and fixes, these records, and the homepage's
download links to 0.1.15, which go live when you merge.

## The release gates (measured this session)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together (a
   password manager or an encrypted drive). Your earlier try ran from your Documents
   folder, where the short path does not exist. Lose them and testers reinstall by hand.
2. **Install 0.1.15 yourself** if your launcher is older: your last install by hand.
3. **Terms of Service and a privacy policy, with a lawyer.** Three live services hold
   emails and messages, and none has either. Account deletion and export follow from them.
4. **The legal opinion** on rewards, prizes, the welcome grant and backing, and on
   **KYC** — *identity checks* —, sanctions and age limits, before anything carries value.
5. **The economic numbers** — *issuance, the genesis split, the decay curve, fee and burn
   shares* (OPEN-1 to OPEN-4, U-4, U-11). Most of the Beta gate waits on them.
6. **Code-signing certificates** for Windows and Apple, and an **external security review**.
7. **Names** for the indexer and explorer, the SDK, and the **MCP server** — *how AI tools
   reach Demiurge*.
8. **Still open, as HANDOFF §4.0 records:** godmode's 100,000,000 CGT (its address); Social's
   report rules; the Android app on your phone; ARQADE inside the launcher; whether #46's
   gate change reads to you as loosening; the reward pool's numbers; Search Console and the
   GitHub preview image; the welcome-grant payer; Qt for CI; the email webhook; the key
   pasted on 5 October; `social-secrets.txt` on your Desktop and `archive-bZLIQp/` in the
   old checkout, each to keep or delete.

**Next in engineering:** L5.1 — an AI agent can act for you only once your own key, in the
launcher, has signed for it (today a stolen sign-in token is enough).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
