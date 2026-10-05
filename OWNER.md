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
- **Sign-in (QOR ID)** at `id.qorsync.dev`, beside its database in Virginia. **143 tests pass** (5 October). Since
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

1. **Fund your launcher account on the devnet** and try minting, selling and the Market there.
2. **CI is green** since your option A (ADR-072); next, the two-validator check on its nightly run.
3. **A faucet page**, so testers can get test CGT without you sending it by hand.
4. **The indexer** — *a service that reads the chain so the Market can search and show history*.
5. **Prize rounds are live on the devnet** — *a game locks a prize on the chain before anyone plays for it*. The SDK
   finds a game's wallet and builds its payouts.

## What only you can do

1. **Put ARQADE on Vercel**, one step at a time with me: merge the `session/arqade-vercel` pull request, import the
   repository in Vercel (folder `products/arqade`), add a free Neon database there, then one Railway setting and three
   Vercel settings. Vercel's free plan is for non-commercial use; it must change before ARQADE takes any payment.
2. **Send yourself test CGT:** open `https://polkadot.js.org/apps/?rpc=wss://rpc.qorsync.dev`, allow your wallet,
   then Accounts → **Devnet faucet** → Send → to your launcher address `5DMPEX…qLxK`.
3. **Get a legal opinion** on backing (crowdfunding) and on paid games with prizes before either goes live.
   Still open for ARQADE: its look (v0), its web address, and the rest of **U-16** — *whether paying to enter may win a
   prize, and the platform's minimum waiting times on an ARQ Wallet*.
4. **Take your Resend key out of this PC's environment**; Railway holds it now.
5. **Bounce reports**: a Resend webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`, its `whsec_…`
   secret into Railway as `RESEND_WEBHOOK_SECRET`.
6. **In Cloudflare**, delete the `ci` record and the tunnel.
7. **Revoke the old SSH key** (`admin@pleroma`) and **rotate the nine credentials** in `SECURITY.md`.
8. **Open economic questions**: the 15% for selling, deposit amounts (U-14), Q-19 and Q-20, U-16.

## What it costs per month

**Railway: not yet measured** for the three devnet nodes (estimated $11–22) on top of sign-in's $5–15. **Vercel and
Neon: $0** on their free plans.
**No spending cap**, by your decision on 1 October. GitHub Actions is free. The real figure replaces these
after a week.
