# Where Demiurge stands

For Andrew. One page. Rewritten at the end of every session — if it is stale, the session is not done.

## What Demiurge is

A blockchain where people who make things get paid when their work is used, and a desktop application
that is how anyone touches it. The currency is **CGT**, and it exists to be spent, not held. Six products
are planned on top of it, all sharing one sign-in, one asset format and one currency.

## What exists and works today

The code is public at `github.com/QOR-MATRIX/demiurge-chain` (your organisation). All counts below are
from 2 October, re-run that day.

- **The chain.** Produces blocks and finalises them — *finalise: agree a block can never be undone* —
  holds assets, sends them, sells them for CGT with royalties, and **now nests one asset inside
  another**, and a buyer can insist on getting exactly the work they looked at. **118 tests pass** with the runtime built. Two machines agreeing was last checked 29 September.
- **Sign-in (QOR ID).** Accounts, email, backup codes, admin controls, and each session now shows when it was last used. **136 tests pass.** Running on
  Railway at `id.qorsync.dev`; you are signed in to it from the launcher.
- **The launcher.** Your keys, sign-in, sending CGT, minting from a project, an Inventory of cards,
  sending assets, Projects, and **now selling: Sell puts a price on the chain, and Buy pays for an asset
  you look up by its number, and a Market screen lists everything for sale.** If the seller changes the work after you look, the chain refuses your purchase and nothing moves. **195 tests pass**, plus seven against a running chain.

**What it cannot do yet:** charge a fee, create new CGT, **search the Market**, set royalties from the launcher, or change an asset's state. The chain runs only on
your PC: there is no public network.

## What changed (2 and 3 October)

- **Unfinished work from the last session was found, finished and tested.** It had been left half done:
  four sentences in the launcher were broken, one QOR ID function was a stub that always answered "safe",
  and nothing was recorded. All fixed; every suite passes.
- **Buying is safer.** A purchase now carries the exact work you saw, and the chain refuses it if the
  seller swapped the work in between. Proven on a running chain.
- **The chain answers "what would this sale pay?"**, and the launcher now shows that answer instead of
  doing the sum itself, so the two can never disagree.
- **The test scripts for sign-in refuse to run** if they would send real email, so the September bounces
  cannot happen again from a test.
- **The Market screen** — *a list of everything for sale* — is in the launcher: filter, sort, page, and
  Buy. It lists what this chain holds, with no search yet. Built by two agents working in parallel,
  then everything was re-tested together.

## Next, in order

1. **Get CI green**: fix whatever its first full runs show.
   Then **you try the Market and selling** in the real launcher.
2. **A public test network** — *devnet: the chain running on servers, not your PC*. The biggest thing
   between here and Alpha. Needs your word on hosting and cost.
3. **You try selling and the onboarding** in the real launcher.
4. **The indexer** — *a service that reads the chain so everyone sees the same listings*.

**Alpha is about a week away**, not an hour: it waits on 1 and 2, on you clicking through the launcher's
confirmation dialogs (L1.4), and on your call about two roadmap items (M3.1, M3.2).

## What only you can do

0. **QOR ID's region**: it moved to Oregon on 3 October while its database stayed in Virginia, so every
   sign-in is slower. If that wasn't meant: Railway → qor-auth → Settings → Regions → back to `iad`.
1. **Tell me how the CI run ended** (github.com/QOR-MATRIX/demiurge-chain/actions), and **let Railway
   see the organisation** (Railway → Account → Integrations → GitHub → add QOR-MATRIX), or a push no
   longer redeploys sign-in.
2. **Approve or refuse three things the agents chose:** how a screen-reading check measures scrolled
   text; that a container asset cannot move until emptied (ADR-065, choices 1, 2, 5); buying by asset
   number as the first shop. (Names and Market choices: approved 3 October, ADR-066.)
3. **Take your Resend key out of this PC's environment**: Railway holds it now, and here it lets a
   local test send real mail.
4. **Bounce reports**: in Resend add a webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`
   and paste its `whsec_…` secret into Railway as `RESEND_WEBHOOK_SECRET`.
5. **In Cloudflare**, delete the `ci` record and the tunnel.
6. **Revoke the old SSH key** (`admin@pleroma`) and **rotate the nine credentials** in `SECURITY.md`.
7. **The 15% for selling**, **deposit amounts** (U-14), **Q-19 and Q-20**, **ADR-043 and 044**, and what
   Demiurge Exchange, QOR Wallet, relays and agentic synchronisation are.

## What it costs per month

**Railway: expected about $5–15 a month** for sign-in, its database and cache (an estimate, not an
invoice). **No spending cap**, by your decision on 1 October. GitHub Actions is free.
