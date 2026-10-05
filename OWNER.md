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
  work after you looked. **143 tests pass** (4 October, the ARQ Wallet's included).
- **Sign-in (QOR ID)** at `id.qorsync.dev`, beside its database in Virginia. **136 tests pass.**
- **The launcher 0.1.6.** Keys, sign-in, sending, minting, Inventory, trading, selling, buying, and a
  **Market** of everything listed. Its next build points at the devnet by default. **195 tests pass.**

**What it cannot do yet:** charge a fee, create new CGT, search the Market, set royalties from the launcher,
or give anyone test CGT except by you sending it from the faucet.

## What changed (2 to 4 October)

- **The devnet went live on Railway** (ADR-068), with both Alpha checks for it met.
- **Buying is safer**, the launcher shows the chain's own payout sum, and the **Market screen** exists.
- **ARQADE, your gaming platform** — *a website where people play games with their QOR ID, own assets and pay
  in CGT*. You accepted **ADR-069** and **ADR-043** (*how a website signs people in to QOR ID*). Its code is in the
  repository (`products/arqade/`), reads the live devnet, and no longer mentions dollars, CRGT or DMRG; **the
  website itself still shows the old version** until redeployed. You also accepted **ADR-070, the ARQ Wallet** (your
  name) — *each game's own payout account, with no key at all, paying only within limits on the chain* — and
  **ADR-071, the app store** — *developers publish themselves once a checklist is met, games up to 10,000 CGT,
  backers who never get a share of sales, a developer's own AI building the game*. Guides: `products/arqade/sdk/docs/`.

## Next, in order

1. **Fund your launcher account on the devnet** and try minting, selling and the Market there.
2. **Get CI green**: your choice in `RUST_TOOLCHAIN.md`, then the chain job runs its tests again.
3. **A faucet page**, so testers can get test CGT without you sending it by hand.
4. **The indexer** — *a service that reads the chain so the Market can search and show history*.
5. **The ARQ Wallet is built on the chain** (not yet on the devnet): next, prizes held for paid rounds, the SDK calls,
   and putting it on the devnet — which needs your admin key once, when it is ready.

## What only you can do

1. **Send yourself test CGT:** open `https://polkadot.js.org/apps/?rpc=wss://rpc.qorsync.dev`, allow your wallet,
   then Accounts → **Devnet faucet** → Send → to your launcher address `5DMPEX…qLxK`.
2. **CI runs now, and the chain job fails** on new Rust (1.99) linting the SDK's own generated code. Fixing it
   weakens a check, so it is your call: `docs/architecture/RUST_TOOLCHAIN.md` (I recommend option A).
3. **Get a legal opinion** on backing (crowdfunding) and on paid games with prizes before either goes live.
   Still open for ARQADE: its look, where it is hosted, and the rest of **U-16** — *whether paying to enter may win a
   prize, and the platform's minimum waiting times on an ARQ Wallet*.
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
