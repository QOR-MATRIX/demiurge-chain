# Where Demiurge stands

For Andrew, 15 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this afternoon)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8, block
  170,917. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`, healthy. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*.

## Publishing a game (P3.4): your decisions tonight

**Publish** in QQ Studio turns your game into a **Cartridge** — *the game as one on-chain
asset* — after the launcher asks you.

- **Your first tries failed** because publishing signed with your vault's first account,
  `5Gx19…5sPwt`, which held nothing. It now holds 35,000 test CGT (read from the chain;
  my notes said 5,000). No publish has gone through yet.
- **Which account publishes — done (ADR-090):** a first publish signs as the account in
  the rail, like everything else in the launcher; once a game is published, the account
  holding it publishes its updates, whatever the rail shows. The dialog names the account.
  Tested against a test chain, and proven to catch both wrong accounts.
- **Pull request #29, after your push:** everything passes except security, and that fails
  on `ruzstd` alone, as expected. **ADR-089 is now done** with your permission: that one
  notice is ignored, and an automatic check fails if the launcher ever builds `ruzstd`.
  The audit passes on this PC; GitHub proves it once you push.
- **Re-checked tonight:** chain 150 tests; launcher 241, its ten screen checks and its live
  tests; QOR ID's 62 that need no database (GitHub runs the other 110).

## What only you can do

1. **Push** tonight's two commits (ADR-090, ADR-089): `git push matrix session/qq-p3.4-publish`.
   Security should then pass on #29; merging #29 makes `main` green again.
2. **Publish again** on a launcher with tonight's change: quit the test launcher (tray →
   Quit), tell me, and I'll start the new one. Then QQ → **Open in QQ Studio** → **Publish**.
3. **Update QQ Studio:** with it closed, run `pwsh products/qq/build.ps1 -Deploy`.
4. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
5. **CI for QQ:** add your Qt account to GitHub's secrets, or a *self-hosted runner* — *your
   PC running GitHub's checks*. Check with Qt that your licence allows it.
6. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
7. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
8. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`) in the project folder;
   delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and tunnel in
   Cloudflare; rotate nine old credentials; the nine frozen *home-map* tiles. Before
   commercial use: a paid Vercel plan, legal review, trademark clearance. Open questions:
   OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
