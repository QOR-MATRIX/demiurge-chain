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

## Publishing a game (P3.4): tonight

**Publish** in QQ Studio turns your game into a **Cartridge** — *the game as one on-chain
asset* — after the launcher asks you.

- **Merged:** pull request #29 passed every check, security included, and you merged it.
  ADR-089 (the security notice) and ADR-090 (which account publishes) are in `main`.
- **Your try:** minting "QQTest1" from the launcher worked, on the test chain on this PC.
  **Publishing from QQ Studio did not:** the Studio never asked the launcher anything,
  and its message wasn't recorded, so the cause isn't known yet. Most likely: your
  installed QQ Studio is from 10 October and lacks two fixes. The launcher now logs every
  request a product makes, so the next try leaves a trace either way.
- **Re-checked tonight:** chain 150 tests; launcher 241, its ten screen checks and its live
  tests; QOR ID's 62 that need no database (GitHub runs the other 110).

## What only you can do

1. **Update QQ Studio**, then publish again: with it closed, run
   `pwsh products/qq/build.ps1 -Deploy`; then **Publish**, and send me its message word for word.
2. **Review pull request** for tonight's records, when I open it.
3. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
4. **CI for QQ:** add your Qt account to GitHub's secrets, or a *self-hosted runner* — *your
   PC running GitHub's checks*. Check with Qt that your licence allows it.
5. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
6. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
7. **Housekeeping:** the GitKraken download (`archive-bZLIQp/`) in the project folder;
   delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and tunnel in
   Cloudflare; rotate nine old credentials; the nine frozen *home-map* tiles. Before
   commercial use: a paid Vercel plan, legal review, trademark clearance. Open questions:
   OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
