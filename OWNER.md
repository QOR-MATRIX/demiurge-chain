# Where Demiurge stands

For Andrew. One page. Rewritten at the end of every session — if it is stale, the session is not done.

## What Demiurge is

A blockchain where people who make things get paid when their work is used, and a desktop application
that is how anyone touches it. The currency is **CGT**, and it exists to be spent, not held. Six products
are planned on top of it, all sharing one sign-in, one asset format and one currency.

## What exists and works today

The code is public at `github.com/QOR-MATRIX/demiurge-chain` (your organisation). Counts are dated; each is the last time it was run.

- **The devnet is live** — *devnet: the chain running on servers for anyone, not on your PC* — at
  `wss://rpc.qorsync.dev`. Two validators — *the machines that make and agree on blocks* — and a public node,
  on Railway. Measured: 100 blocks finalised in ten minutes, both validators taking turns, all three
  agreeing. You hold its admin key and its faucet — *the account that hands out test CGT* — in your wallet.
- **The chain.** Makes and finalises blocks — *finalise: agree a block can never be undone* — holds assets,
  sends, sells with royalties, nests one asset in another, and refuses a purchase if the seller changed the
  work after you looked. **150 tests pass** (4 October, the ARQ Wallet's included).
- **Sign-in (QOR ID)** at `id.qorsync.dev`, beside its database in Virginia. **149 tests pass** (5 October). Your QOR ID is just your name, with no `#0001`, and no two
  accounts share a name (ADR-075, live). **`id.qorsync.dev/account`** — *QOR ID's own account page* — changes
  your password or adds an email address (waiting on your merge). Since
  5 October it also signs people in to **other websites** on its own page, and ARQADE is registered with it.
- **The launcher 0.1.6.** Keys, sign-in, sending, minting, Inventory, trading, selling, buying, and a
  **Market** of everything listed. Its next build points at the devnet by default. **195 tests pass.**

**What it cannot do yet:** charge a fee, create new CGT, search the Market, set royalties from the launcher,
or give anyone test CGT except by you sending it from the faucet.

## What changed (4 to 5 October)

- **QOR ID signs people in to other websites** (ADR-073), merged by you and live.
- **ARQADE moves to Vercel** (ADR-074, your choice) — *Vercel: a website host; v0: its screen designer* — because the
  Sites copy could not be published or checked from here. It is now an ordinary Next.js site with its own Postgres
  database — *where matches, rankings and chat are kept* — and **players sign in with QOR ID only**: no ChatGPT sign-in,
  no "Explorer" nicknames. 41 tests pass, and the whole sign-in and a match were tried in Chrome against a local QOR ID.
  It is not online yet: the steps below put it there.
- Earlier: the ARQ Wallet (ADR-070) and the app store (ADR-071), accepted by you; guides in `products/arqade/sdk/docs/`.

## Next, in order

1. **Levels, XP and the welcome grant** (ADR-078). **Built, waiting on your merge:** the level bubble and XP bar in the
   launcher and ARQADE, and XP for the tutorial, your email, your key, signing in to ARQADE, a first match and a first
   tip. Next: drawing the unlocks, and paying the 100 test CGT welcome grants that QOR ID now records as owed.
2. **ARQADE inside the launcher**: a Play section, signed in, payments approved in the launcher's own window.
3. **CGT rewards for multiplayer wins**, from ARQADE's ARQ Wallet — *the game's own payout account* — on the devnet.
4. **A faucet page**, and **the indexer** — *a service that reads the chain so the Market can search*.
## What only you can do

1. **Decide whether avatar rings may glow** (ADR-078, which you accepted with a 100 CGT welcome grant). Until you do,
   rings are colour and pattern only.
2. **Add your email address** at `id.qorsync.dev/account` — *QOR ID's own account page* — so a forgotten password
   can be reset. **Tips through the launcher work** (ADR-076): you tipped a creator from launcher 0.1.7 on 6 October.
3. **Replace the key behind the 24-word phrase** pasted into a chat on 5 October: treat it as exposed. If it is the
   devnet's admin key, move admin to a new key (`sudo.setKey`).
4. **Get a legal opinion** on backing (crowdfunding), paid games with prizes, the welcome grant and task rewards, and
   making CGT exchangeable, before any of them goes live.
   Still open for ARQADE: its look (v0), its web address, and the rest of **U-16** — *whether paying to enter may win a
   prize, and the platform's minimum waiting times on an ARQ Wallet*.
5. **Take your Resend key out of this PC's environment**; Railway holds it now.
6. **Bounce reports**: a Resend webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`, its `whsec_…`
   secret into Railway as `RESEND_WEBHOOK_SECRET`.
7. **In Cloudflare**, delete the `ci` record and the tunnel.
8. **Revoke the old SSH key** (`admin@pleroma`) and **rotate the nine credentials** in `SECURITY.md`.
9. **Open economic questions**: the 15% for selling, deposit amounts (U-14), Q-19 and Q-20, U-16, and **U-18** — *who funds the welcome grant and any task rewards with real CGT*.

## What it costs per month

**Railway: not yet measured** for the three devnet nodes (estimated $11–22) on top of sign-in's $5–15. **Vercel and
Neon: $0** on their free plans.
**No spending cap**, by your decision on 1 October. GitHub Actions is free. The real figure replaces these
after a week.
