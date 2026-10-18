# Where Demiurge stands

For Andrew, 18 October 2026 (night). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this evening, from outside)

- **The homepage** at **qorsync.dev**; demiurge.cloud and demiurge.guru forward to it.
- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime 9.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — and **launcher 0.1.12** for testers.

## Newest: ARQADE inside the launcher, in Qt (stage 1 of 6 built)

As you chose: **1 CGT = 1,000 Sparks**, with the chain's tiny unit, the **ember**, underneath
(nothing on chain changes). ARQADE becomes a **Qt** — *the 3D engine QQ is built on* — app
inside the launcher, its games rebuilt in 3D, Energy gone, players paid in Sparks from **one
reward pool** you fund with test CGT. Our server replays each run before it pays, so a faked
score earns nothing.

- **Stage 1, built (pull request #47):** Sparks everywhere. Your Vault will show "37,000,000
  Sparks" under 37,000 CGT, and the send box and an asset's sale price take "250 Sparks" as
  well as CGT. Every test passes.
- **Next:** ARQADE's Qt home in the launcher; the four solo games rebuilt; then the pool and
  replay; the website without Energy; the multiplayer arenas; then the launcher's own screens.

Rewards stay on the devnet until your legal review: paying players real value is regulated
in many places.

## Open pull requests

- **#45** the homepage (already live), **#46** launcher 0.1.13 (separate profiles with PINs),
  **#47** Sparks. Merge them in that order.

## What only you can do

1. **Merge #45, #46 and #47.** Then say "publish 0.1.13" if you want testers to have it.
2. **Google Search Console:** add `qorsync.dev`, verify with the DNS record in Cloudflare,
   submit `qorsync.dev/sitemap.xml`; Bing can import it from Google.
3. **GitHub social preview:** repository Settings → Social preview → `site/og.png`.
4. **Rule on one gate change** (in #46): ADR-098's check replaced by ADR-099's. A loosening?
5. **The reward pool's numbers**, when we reach stage 4: the most per run, per player per day,
   and in total. They're yours to set; I won't invent them.
6. **Still open:** godmode's 100,000,000 CGT (I need its address); Social's report rules
   (L8.7); `social-secrets.txt` on your Desktop; `archive-bZLIQp/`; QQ Studio's update; the last
   two L1.4 checks; CI for QQ; the welcome-grant payer; an email; the key pasted on 5 October.

## Cost per month

The homepage and ARQADE: Vercel's free plan. GitHub downloads: free. The Mesh, QOR Media and
translation: $0, on each PC. Railway is not yet measured. Neon: free plan.
