# Where Demiurge stands

For Andrew, 18 October 2026 (late night). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight, from outside)

- **The homepage** at **qorsync.dev**. **Devnet** — *the test chain on servers* — on runtime 9.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — on Vercel.
- **Launcher 0.1.14 for testers**, published tonight:
  `github.com/QOR-MATRIX/demiurge-chain/releases/tag/launcher-v0.1.14`.

## New in 0.1.14

- **ARQADE inside the launcher.** It is a Qt app — *the toolkit QOR Media uses* — **docked**
  — *drawn inside the launcher's window*:
  - a 3D gilt astrolabe that follows the mouse;
  - the six games as cards;
  - your Sparks;
  - the rankings.

  It signs in through the launcher, so nothing is typed. It draws on your RTX 4060. The games
  themselves are rebuilt next; until then each card opens today's version.
- **Sparks:** 1 CGT = 1,000 Sparks, shown and typed.
- **Profiles:** each QOR ID has its own wallet and PIN (from 0.1.13, never published alone).

Signing in to ARQADE through a running launcher is the one part not yet tried by a person.

## What only you can do

1. **Install 0.1.14 and try ARQADE:** open ARQADE from the menu while signed in.
2. **Merge #48** — *this work and the homepage's new download link*. CI checks it first.
3. **Google Search Console:** add `qorsync.dev`, verify with the DNS record in Cloudflare,
   submit `qorsync.dev/sitemap.xml`; Bing can import it from Google.
4. **GitHub social preview:** repository Settings → Social preview → `site/og.png`.
5. **Rule on one gate change** (in #46): ADR-098's check replaced by ADR-099's. A loosening?
6. **The reward pool's numbers**, at ARQADE's stage 4: the most per run, per player per day,
   and in total.
7. **Still open:** godmode's 100,000,000 CGT (I need its address); Social's report rules
   (L8.7); `social-secrets.txt` on your Desktop; `archive-bZLIQp/`; QQ Studio's update; the last
   two L1.4 checks; CI for QQ; the welcome-grant payer; an email; the key pasted on 5 October.

## Cost per month

The homepage and ARQADE: Vercel's free plan. GitHub downloads: free. The Mesh, QOR Media,
ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free plan.
