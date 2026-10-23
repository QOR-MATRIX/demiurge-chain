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

1. **Assets can now level up** (ADR-113, your approval). Each asset keeps a level, **XP** —
   *experience points* — and stats, written by the game its collection's owner picks. That
   game's servers can only add; only whoever holds the game's **Cartridge** — *the game's
   own asset* — can correct a mistake; nothing changes while an asset is for sale, and no
   record moves an asset or currency. The launcher's Inventory and Market cards show it.
   Built and tested (190 chain tests, seven planted bugs caught); it reaches the devnet
   when you sign the upgrade below. This finishes the asset roadmap items M4.2 and M4.5.
2. **Two launchers out.** 0.1.15 is the first that updates itself; 0.1.16 is the first
   update it offers, taken inside the launcher with *Update and restart*.
3. **An AI agent can act for you only once your own key signs for it** (L5.1); live in
   QOR ID since your merge of #79.
4. **Found and fixed on the way:** the chain's documents said the devnet ran runtime 8 —
   *the chain's program version* — when it has run 9 since 17 October; a security advisory
   in a Mesh library, fixed in #81.

**#81 is merged.** **For your merge: #82**, the progress records and these records.

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
2. **Upgrade the devnet to runtime 10** after merging #82, as you did for 9: on the Sudo
   page pick `system.setCode`, with the field reading `code`, and the file I give you
   once I have rehearsed it against the devnet's own runtime.
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

**Next in engineering:** rehearsing runtime 10; then, at your word, games other than the
home game keeping records (ADR-113's second phase) or rental and fractions (M4.3); PDF in
QOR Media (L9.5).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
