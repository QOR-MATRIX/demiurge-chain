# Where Demiurge stands

For Andrew, 18 October 2026 (late night). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight, from outside)

- **The homepage** at **qorsync.dev**. **Devnet** — *the test chain on servers* — on runtime 9.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — and **launcher 0.1.12** for testers.
- In `main` since you merged #46 and #47: **0.1.13's profiles** and **Sparks** (1 CGT = 1,000).

## Built tonight, waiting for your word

**ARQADE inside the launcher**, stage 2 of six. It is a Qt app — *the toolkit QOR Media uses* —
**docked** — *drawn inside the launcher's window* — over the ARQADE screen:

- a 3D gilt astrolabe that follows the pointer;
- the six games as cards, each with its own little 3D scene;
- your Sparks;
- the arenas' rankings.

It signs in to ARQADE through the launcher, so no password is typed; tested against stand-ins,
not yet by a person in a running launcher. It draws on your RTX 4060, not the weaker Intel chip.
Every test passes, and the launcher started the real app.

The games themselves are rebuilt next (stage 3); until then each card opens today's version.

**Nothing is committed yet:** say "commit it" and I'll open a pull request.

## What only you can do

1. **Say "commit it"** for the ARQADE app, and "publish 0.1.13" if testers should have profiles.
2. **Google Search Console:** add `qorsync.dev`, verify with the DNS record in Cloudflare,
   submit `qorsync.dev/sitemap.xml`; Bing can import it from Google.
3. **GitHub social preview:** repository Settings → Social preview → `site/og.png`.
4. **Rule on one gate change** (in #46): ADR-098's check replaced by ADR-099's. A loosening?
5. **The reward pool's numbers**, at ARQADE's stage 4: the most per run, per player per day,
   and in total.
6. **Still open:** godmode's 100,000,000 CGT (I need its address); Social's report rules
   (L8.7); `social-secrets.txt` on your Desktop; `archive-bZLIQp/`; QQ Studio's update; the last
   two L1.4 checks; CI for QQ; the welcome-grant payer; an email; the key pasted on 5 October.

## Cost per month

The homepage and ARQADE: Vercel's free plan. GitHub downloads: free. The Mesh, QOR Media,
ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free plan.
