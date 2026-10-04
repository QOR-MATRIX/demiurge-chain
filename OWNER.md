# Where Demiurge stands

For Andrew. One page. Rewritten at the end of every session — if it is stale, the session is not done.

## What Demiurge is

A blockchain where people who make things get paid when their work is used, and a desktop application
that is how anyone touches it. The currency is **CGT**, and it exists to be spent, not held. Six products
are planned on top of it, all sharing one sign-in, one asset format and one currency.

## What exists and works today

The code is public at `github.com/QOR-MATRIX/demiurge-chain` (your organisation). Counts are from 3 October; nothing was re-run on 4 October, when only documents changed.

- **The devnet is live** — *devnet: the chain running on servers for anyone, not on your PC* — at
  `wss://rpc.qorsync.dev`. Two validators — *the machines that make and agree on blocks* — and a public node,
  on Railway. Measured: 100 blocks finalised in ten minutes, both validators taking turns, all three
  agreeing. You hold its admin key and its faucet — *the account that hands out test CGT* — in your wallet.
- **The chain.** Makes and finalises blocks — *finalise: agree a block can never be undone* — holds assets,
  sends, sells with royalties, nests one asset in another, and refuses a purchase if the seller changed the
  work after you looked. **125 tests pass.**
- **Sign-in (QOR ID)** at `id.qorsync.dev`, beside its database in Virginia. **136 tests pass.**
- **The launcher 0.1.6.** Keys, sign-in, sending, minting, Inventory, trading, selling, buying, and a
  **Market** of everything listed. Its next build points at the devnet by default. **195 tests pass.**

**What it cannot do yet:** charge a fee, create new CGT, search the Market, set royalties from the launcher,
or give anyone test CGT except by you sending it from the faucet.

## What changed (2 to 4 October)

- **The devnet went live on Railway** (ADR-068), with both Alpha checks for it met.
- **Buying is safer**, the launcher shows the chain's own payout sum, and the **Market screen** exists.
- **ARQADE, your gaming platform, is written into the plan** (4 October) — *a website where people play games
  with their QOR ID, win assets and pay in CGT*. Your handoff was checked against the code and corrected:
  `docs/blueprints/arqade.md`, its steps P7.1 to P7.10, and **ADR-069** — *a written decision* — which **you
  accepted**, with **ADR-043** (*how a website signs people in to QOR ID*). Its code is now in the repository (`products/arqade/`), reads the live devnet, and no longer
  mentions dollars, CRGT or DMRG. **The website itself still shows the old version** until it is redeployed.

## Next, in order

1. **Fund your launcher account on the devnet** and try minting, selling and the Market there.
2. **Get CI green**: tell me what its runs show.
3. **A faucet page**, so testers can get test CGT without you sending it by hand.
4. **The indexer** — *a service that reads the chain so the Market can search and show history*.
5. **ARQADE:** finish P7.2 (balances and assets, read-only), write its control-by-control inventory, rename it
   ARQADE on screen, and fix its 13 lint errors.

## What only you can do

1. **Send yourself test CGT:** open `https://polkadot.js.org/apps/?rpc=wss://rpc.qorsync.dev`, allow your wallet,
   then Accounts → **Devnet faucet** → Send → to your launcher address `5DMPEX…qLxK`.
2. **Tell me how the CI runs ended** (github.com/QOR-MATRIX/demiurge-chain/actions).
3. **ARQADE's three open choices** (you accepted ADR-069 and ADR-043 on 4 October): its look (your design rules
   forbid glow and neon), where it is hosted, and **U-16** — *whether paid games may pay out prizes, and from
   whose money*.
4. **Take your Resend key out of this PC's environment**; Railway holds it now.
5. **Bounce reports**: a Resend webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`, its `whsec_…`
   secret into Railway as `RESEND_WEBHOOK_SECRET`.
6. **In Cloudflare**, delete the `ci` record and the tunnel.
7. **Revoke the old SSH key** (`admin@pleroma`) and **rotate the nine credentials** in `SECURITY.md`.
8. **Open economic questions**: the 15% for selling, deposit amounts (U-14), Q-19 and Q-20, U-16.

## What it costs per month

**Railway: not yet measured** for the three devnet nodes (estimated $11–22) on top of sign-in's $5–15.
**No spending cap**, by your decision on 1 October. GitHub Actions is free. The real figure replaces these
after a week.
