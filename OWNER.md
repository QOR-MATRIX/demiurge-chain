# Where Demiurge stands

For Andrew, 10 October 2026 (late afternoon). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`; signs you in to ARQADE's window. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*.

## Done today: the lock-down measured, and a hidden fault found

With the laptop on mains power, I timed how fast a game starts in the QQ Player:

- **The lock-down costs nothing measurable.** Switching its parts off, one at a time or all
  at once, gives the same start time.
- **On your PC** the sample game shows its first picture in about **1.7 seconds**; **in a
  browser**, about **5.5 seconds** from the page starting to load. Both draw far more
  frames a second than a screen shows.
- **Found:** before the lock-down, the sample game's one 3D model (the gold orb) never
  loaded in the Player. The game quietly played without it, so it started faster: the
  1.4 seconds recorded earlier today was for a game missing a piece. The lock-down work had
  already fixed the cause; now the Player also **names any model that fails to load**, and
  the automatic checks fail on one. I proved this by putting the old fault back and
  watching the checks catch it.
- **QQ Studio is updated** on your PC, so **Open in QQ Studio** has all of today's fixes.

**Your dialog checks (L1.4) worked:** you approved and declined a send and a change of
chain, on a local test chain; I read the approvals back from the chain. Two QOR ID checks
remain (below). Note: on Devnet you sent 100 test CGT to a public test address, so anyone
can take them; harmless, as test CGT is worth nothing.

**Next:** publishing a game (P3.4), now unblocked.

## What only you can do

1. **Merge #28** (today's measurements and dialog checks).
2. **The last two L1.4 checks**, when convenient: change the QOR ID address in the launcher,
   declined once and approved once; and sign in after QOR ID was unreachable at start-up.
3. **Optional: prove P3.3** — your own Anthropic key in QQ Studio's Agent panel, billed to you.
4. **CI** — *automatic build and test* — **for QQ:** GitHub cannot build Qt without your
   licence. Add your Qt account to its secrets, or let your PC run CI (a *self-hosted runner*).
5. **Check with Qt** that your licence covers CI, AI assistants and shipping the browser Player.
6. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
7. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
8. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and
   tunnel in Cloudflare; rotate nine old credentials; decide the nine frozen *home-map* tiles.
9. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
10. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
