# Where Demiurge stands

For Andrew. One page. Rewritten at the end of every session — if it is stale, the session is not done.

## What Demiurge is

A blockchain where people who make things get paid when their work is used, and a desktop application
that is how anyone touches it. The currency is **CGT**, and it exists to be spent, not held. Six products
are planned on top of it, all sharing one sign-in, one asset format and one currency.

## What exists and works today

The code is public at `github.com/QOR-MATRIX/demiurge-chain` (your organisation). All counts below are
from 1 October.

- **The chain.** Produces blocks and finalises them — *finalise: agree a block can never be undone* —
  holds assets, sends them, sells them for CGT with royalties, and **now nests one asset inside
  another**. **109 tests pass** with the runtime built. Two machines agreeing was last checked 29 September.
- **Sign-in (QOR ID).** Accounts, email, backup codes, admin controls. **130 tests pass.** Running on
  Railway at `id.qorsync.dev`; you are signed in to it from the launcher.
- **The launcher.** Your keys, sign-in, sending CGT, minting from a project, an Inventory of cards,
  sending assets, Projects, and **now selling: Sell puts a price on the chain, and Buy pays for an asset
  you look up by its number.** **182 tests pass**, plus five against a running chain.

**What it cannot do yet:** charge a fee, create new CGT, **show a listing to anyone who wasn't given the
asset's number**, set royalties from the launcher, or change an asset's state. The chain runs only on
your PC: there is no public network.

## What changed (1 October)

- **QOR ID moved to Railway** and answers at `https://id.qorsync.dev`. Email works (delivered to
  Resend's test inbox). The PC stack is retired.
- **CI runs at last.** *CI: the robot that checks every change.* Its jobs start in your organisation
  QOR-MATRIX, which your locked personal account never allowed (ADR-064). No result seen yet.
- **Three agents built in parallel:** nesting on the chain (ADR-065), selling and buying in the
  launcher, and two QOR ID faults closed (the line-ending fault that stopped Railway, and tests that
  sent two real emails in September).
- **Selling has never been used by a person**, only by tests. It is yours to try.

## Next, in order

1. **Get CI green**: fix whatever its first full runs show.
2. **A public test network** — *devnet: the chain running on servers, not your PC*. The biggest thing
   between here and Alpha. Needs your word on hosting and cost.
3. **You try selling and the onboarding** in the real launcher.
4. **The indexer** — *a service that reads the chain so everyone sees the same listings*.

**Alpha is about a week away**, not an hour: it waits on 1 and 2, on you clicking through the launcher's
confirmation dialogs (L1.4), and on your call about two roadmap items (M3.1, M3.2).

## What only you can do

1. **Tell me how the CI run ended** (github.com/QOR-MATRIX/demiurge-chain/actions), and **let Railway
   see the organisation** (Railway → Account → Integrations → GitHub → add QOR-MATRIX), or a push no
   longer redeploys sign-in.
2. **Approve or refuse three things the agents chose:** how a screen-reading check measures scrolled
   text; that a container asset cannot move until emptied (ADR-065, choices 1, 2, 5); buying by asset
   number as the first shop.
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
