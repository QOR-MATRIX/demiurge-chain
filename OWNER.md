# Where Demiurge stands

For Andrew, 7 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`: two validators —
  *machines that make blocks* — and a public node on Railway — *a server host*. Near block
  41,000; version 8 since 4 October (ARQ Wallet — *a game's payout account* — rounds). You
  hold its admin key. 150 tests pass.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`: unique names, `/account`, sign-in
  for other websites, levels, XP — *experience points* — and avatars. 166 tests pass.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app` since 5 October, on
  Vercel — *a website host*: QOR ID sign-in, solo and multiplayer games, rankings, chat,
  levels, and tips paid through the launcher (you tipped on 5 and 6 October). 48 tests pass.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*: keys, sending, minting, selling,
  the Market, `qor://pay` — *a website's payment request* — since 0.1.7, and a level bubble.
  204 tests and 9 screen checks pass. **On main, for the next build:** your avatar.
- **CI** — *automatic build and test* — is green on main. Two runs failed on 6 and 7 October
  when Chrome started slowly on GitHub's machines; the checks now wait 30 seconds for it.

**Not built:** fees, new CGT, a production network, Market search, paying the welcome
grants QOR ID records as owed, avatars in ARQADE, ARQADE inside the launcher, agent rails
— *spending limits for AI agents* — and the Mesh — *players hosting what they own*.

## Decided 6 October

**QQ is QOR Engine** (ADR-081 — *decision record 81*). QQ is an engine and editor for small games
built on visual effects, in the launcher and on ARQADE. An AI can drive it, and it generates
assets from a description. The Godot plan and its promise that projects open in plain Godot are
withdrawn, along with the release check that enforced that promise. Its builder makes QQ's
technical choices without stopping, and records them as decisions. CGT, wording and key-safety
rules still apply.

## Next work

1. **QQ**: its blueprint, roadmap items and gate, then the engine itself.
2. **The QFX reactive-backdrop library** — *launcher visuals that respond* — you asked for.
3. **Avatars in ARQADE**, then **paying owed welcome grants** and drawing level unlocks.
4. **ARQADE inside the launcher**, payments approved in the launcher's own window.

## What only you can do

1. **After merging this documentation pull request, re-run the audit** against `main`.
2. **Install the next launcher build** after 0.1.8 to get your avatar on its glowing ring.
3. **Add an email address** at `id.qorsync.dev/account`, so a lost password can be reset.
4. **Replace the exposed key.** A recovery phrase was pasted into a chat on 5 October;
   treat that key as exposed and replace it. If it is devnet admin, move admin (`sudo.setKey`).
5. **Delete `QOR-MATRIX/arqade`**, the stray copy Vercel's clone flow made, if not done.
6. **Bounce reports:** a Resend — *QOR ID's email sender* — webhook to
   `https://id.qorsync.dev/api/v1/webhooks/resend`, its secret into Railway as
   `RESEND_WEBHOOK_SECRET` (unconfirmed whether set; until then reports are refused).
7. **In Cloudflare**, delete the `ci` record and the tunnel.
8. **Rotate the nine old credentials** if those services ever ran; revoke SSH key
   `admin@pleroma`.
9. **Decide the nine frozen tiles** on the Nexus — *the launcher's home map*: none works.
10. **Before commercial use:** a paid Vercel plan, a legal review (backing, prizes, welcome
    grant, task rewards, exchangeable CGT) and trademark clearance.
11. **Open questions:** OPEN-1 to OPEN-4 (issuance, genesis split, decay, burn share), U-4
    (fee classes), U-14 (deposits), U-15 (sale share), U-16 (paid play), U-17 (backing),
    U-18 (who funds grants), Q-19 and Q-20 (what an asset fingerprint covers).

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. No spending cap (your decision, 1 October).
