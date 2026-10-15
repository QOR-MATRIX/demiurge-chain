# Where Demiurge stands

For Andrew, 15 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this morning)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`, healthy. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*.

## Where publishing stands (P3.4)

**Publish** in QQ Studio saves your game, records it as a version in Qontrol and turns it
into a **Cartridge** — *the game as one on-chain asset* — through the launcher, which asks
you first. Publishing again updates the same Cartridge.

- **Your first try (10 October) found a fault:** Publish was greyed out because the Studio
  was open on a folder Qontrol doesn't track, and the reason was only in a hover tip. Now
  Publish can always be pressed and says what's missing. A new automatic check fails on
  the old behaviour.
- **Re-checked today**, after your PC restarted and wiped the test chain: all launcher tests
  (240) and all QQ tests (73) pass.
- **Ready for you now:** a local test chain is running, your account holds 5,000 test CGT
  on it, and the test launcher is open.

## What only you can do

1. **Publish once, for real**, in the open launcher:
   1. **Chain → Local node**, and approve.
   2. **Projects:** open or create a project folder.
   3. **QQ page:** choose that same folder, then **Open in QQ Studio**.
   4. In the Studio, add something, press **Publish**, and approve **"Approve this mint"**.
   5. Change something, **Publish** again, and approve **"Approve this revision"**.
   6. Tell me what each window said. I'll check the chain.
2. **Update QQ Studio on your PC** with today's fix (my tools aren't allowed to): close the
   Studio, then run `pwsh products/qq/build.ps1 -Deploy`.
3. **The last two L1.4 checks** — *the launcher's approval dialogs*: change the QOR ID address
   (decline once, approve once); sign in after QOR ID was unreachable at start-up.
4. **Optional: prove P3.3** — your own Anthropic key in QQ Studio's Agent panel, billed to you.
5. **CI** — *automatic build and test* — **for QQ:** add your Qt account to GitHub's secrets,
   or let your PC run CI (a *self-hosted runner*). Check with Qt that your licence allows it.
6. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
7. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
8. **Housekeeping:** a GitKraken download (`archive-bZLIQp/`) appeared in the project folder
   this morning — delete it or tell me to; delete `QOR-MATRIX/arqade`; the Resend bounce
   webhook; the `ci` record and tunnel in Cloudflare; rotate nine old credentials; decide the
   nine frozen *home-map* tiles.
9. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
10. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
