# Where Demiurge stands

For Andrew. One page. Rewritten at the end of every session — if it is stale, the session is not done.

## What Demiurge is

A blockchain where people who make things get paid when their work is used, and a desktop application
that is how anyone touches it. The currency is **CGT**, and it exists to be spent, not held. Six products
are planned on top of it, all sharing one sign-in, one asset format and one currency.

## What exists and works today

The code is public at `github.com/QOR-MATRIX/demiurge-chain` (your organisation, since 1 October).
Tests last run 30 September; Railway checked 1 October.

- **The chain.** Produces blocks and finalises them — *finalise: agree a block can never be undone* —
  holds assets, sends them, and **sells them for CGT and pays royalties**. Two machines running it agree (last checked 29 September).
  **96 tests pass** (with the runtime built, 30 September).
- **Sign-in (QOR ID).** Accounts, email, backup codes, admin controls. **125 tests passed** when last run
  (21 September). **Running on Railway since 1 October at `id.qorsync.dev`.**
- **The launcher.** Your keys, sign-in, sending CGT, minting from a project, an Inventory of cards, sending
  assets, a listing you can draft, and Projects. **163 tests pass**, plus three against a running chain (last run 29 September).

**What it cannot do yet:** charge a fee, create new CGT, **show a listing to anyone who doesn't already know
the asset**, sell from the launcher, or let an asset nest or change state.

## What changed (30 September and 1 October)

- **1 October: QOR ID runs on Railway**, with your account in it. It first refused to start (the copied
  database had Windows line endings on record); you applied the correction and it went green.
  **`https://id.qorsync.dev` answers from Railway** (checked: ready, database and cache ok), with your
  PC off the path. Email works: a test sign-up's message was delivered to Resend's test
  inbox (a real inbox is untried). The temporary database door is closed, and signing in from the launcher is untried.
- **CI runs at last.** *CI: the robot that checks every change.* You made the organisation QOR-MATRIX
  and pushed the code there; its jobs started, which your locked personal account never allowed
  (ADR-064). Whether they pass is not known yet.
- **The PC stack is retired**: its containers are removed (data volumes kept) and `infra/ops/` is deleted.
- **The progress dashboard was reading the wrong CI**: the private archive's, not the public one's. Fixed;
  it now names `demiurge-chain` outright.
- **The launcher looked for sign-in at `demiurge.cloud`**, where nothing answers. It now looks at
  `id.qorsync.dev`, the address that will be Railway's.
- **The hosting plan** now matches the move to Railway.

## Next, in order

1. **Finish the move**: your items 2 to 5 below, then I send a test email and check the launcher signs in.
2. **You try the onboarding**: claim a name from the bubble, click the glowing notification, read the story.
3. **Selling in the launcher**: Sell publishes to the chain, and Buy appears on a card.
4. **The indexer** — *a service that reads the chain so everyone sees the same listings*.

## What only you can do

1. **Tell me how the first CI run ended** (github.com/QOR-MATRIX/demiurge-chain/actions), and **let
   Railway see the organisation** (Railway → Account → Integrations → GitHub → add QOR-MATRIX), or a
   push no longer redeploys sign-in. The billing dispute on your personal account blocks nothing now.
2. **Bounce reports**: in Resend add a webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`
   and paste its `whsec_…` secret into Railway as `RESEND_WEBHOOK_SECRET`.
3. **Sign up with your own email** once, to see a real inbox receive the message.
4. **Cap Railway at $10 a month**, if you have not: Railway → Workspace → Usage → hard limit.
5. **In Cloudflare**, delete the `ci` record and the tunnel. Then try signing in from the launcher
   (Settings → QOR ID address `https://id.qorsync.dev/api/v1`).
6. **Revoke the old SSH key**, if you have not: ED25519, `admin@pleroma`,
   `SHA256:+BzEaLt1Mjurz+jp8b+SIT7yCdz/pgqQuAQCjjwccEA`.
7. **Rotate the nine credentials** in `SECURITY.md` (they're only in the private archive now).
8. **The 15% for selling**, **deposit amounts** (U-14), **Q-19 and Q-20**, **ADR-043 and 044**, and what
   Demiurge Exchange, QOR Wallet, relays and agentic synchronisation are.

## What it costs per month

**Railway: expected about $5–15 a month** for sign-in, its database and cache (usage-based; an estimate,
not an invoice). You chose a **$10 cap**; it is not set until you set it (item 4). GitHub Actions is free on a public repository.
