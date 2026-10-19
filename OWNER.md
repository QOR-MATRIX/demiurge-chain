# Where Demiurge stands

For Andrew, 18 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

- **The homepage** at **qorsync.dev**. **Devnet** — *the test chain on servers* — on runtime 9.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — on Vercel.
- **Launcher 0.1.14 for testers**, with ARQADE inside it: `releases/tag/launcher-v0.1.14`.

## Waiting for you to merge (every check passes on both)

- **#48, ARQADE in the launcher:** its 3D app docked in the launcher, and 0.1.14's records.
- **#49, ARQADE on phones:** a tab bar along the bottom, games filling the screen, played by
  tapping. Its preview opens once you are signed in to Vercel; after merging, it is live.

## Android, started

Built in Qt — *the toolkit QOR Media uses* — over the same Rust core — *the part that holds
keys* — as the Windows launcher (ADR-104). It runs on an Android 16 emulator — *a phone simulated
on this PC*. Tonight it only shows the core answering and reads amounts in Sparks; the wallet,
Social, ARQADE, the Library and the Mesh come next.

To try it: copy **`QOR-Launcher-Android-0.1.0-test.apk`** from your Desktop to your phone, open
it, and allow installs from that app when asked.

## What only you can do

1. **Merge #48, then #49, then #50** (Android, app ID `app.qorsync.dev`, as you confirmed).
2. **Google Search Console** for `qorsync.dev`, and the **GitHub social preview** (`site/og.png`).
3. **Rule on one gate change** (in #46): ADR-098's check replaced by ADR-099's. A loosening?
4. **The reward pool's numbers**, at ARQADE's stage 4: the most per run, per player per day,
   and in total.
5. **Still open:** godmode's 100,000,000 CGT (I need its address); Social's report rules
   (L8.7); `social-secrets.txt` on your Desktop; `archive-bZLIQp/`; QQ Studio's update; the last
   two L1.4 checks; CI for QQ; the welcome-grant payer; an email; the key pasted on 5 October.

## Cost per month

The homepage and ARQADE: Vercel's free plan. GitHub downloads: free. The Mesh, QOR Media,
ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free plan.
