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

1. **Two launchers out.** 0.1.15 (22 October) is the first that updates itself; testers
   install it by hand once. 0.1.16 (23 October) is the first update it offers: they get
   it inside the launcher by pressing *Update and restart*. Each was downloaded back from
   GitHub and checked against our signing key before I called it done.
2. **An AI agent can act for you only once your own key signs for it** (L5.1), in the
   launcher, after a dialog saying what it may do. QOR ID was redeployed from your merge
   of #79, so this is live: a stolen sign-in token is no longer enough.
3. **QQ Studio on this PC is current**: rebuilt, all eight test suites passed.
4. **Found and fixed on the way:** gaps in the Mesh dialog's sentences since 0.1.10 — *the
   Mesh: our paid file-sharing network* — now held by a test; agent names that broke QOR
   ID's rules; a release build that could not run from the current checkout; a GitHub
   cache hang that stalled **CI** — *GitHub's automatic tests* — now capped at ten minutes.

**For your merge: #80** — 0.1.16's version, the homepage's links to it, and these records.

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
2. **Install 0.1.15 yourself, then take 0.1.16 when it offers it**: the first update
   through the channel, which ticks L6.2.
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

**Next in engineering:** the asset work (M4.2, M4.3, M4.5), then PDF in QOR Media (L9.5).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
