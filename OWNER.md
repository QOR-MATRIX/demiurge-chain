# Where Demiurge stands

For Andrew, 10 October 2026 (afternoon). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`; signs you in to ARQADE's window. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*.

## Decided today: keeping players safe from strangers' games (ADR-087)

Before people play each other's QQ games, a game must not be able to touch the player's files,
accounts or network; today a game's logic can do whatever the player can. **You chose:**

- strangers' games play **only in the browser** (ARQADE) at first, from a web address of their
  own that holds no logins, boxed inside ARQADE's page;
- the QQ Player **locks down** what any game's logic can reach, everywhere (cheap);
- playing others' games **natively** in the launcher waits for an operating-system *sandbox* —
  *a locked box a program runs in* — which is a bigger job, done if and when you want it;
- games are served from a **site of their own**: a separate free Vercel address now, a domain
  of their own (your choice of name) before launch.

**Next:** I build the Player lock-down, then publishing (P3.4).

## Done earlier today

**Your rule on AI costs** (ADR-086 — *a decision record*): Demiurge pays for no one's AI use;
users pay their provider directly, and features stay off until they bring their own key.
P3.3 and the rule are on main; automatic checks pass.

## What only you can do

1. **Merge #25 and #26** (records, and this decision).
2. **Optional: prove P3.3** — your own Anthropic key in QQ Studio's Agent panel, billed to you.
3. **CI** — *automatic build and test* — **for QQ:** GitHub cannot build Qt without your
   licence. Add your Qt account to its secrets, or let your PC run CI (a *self-hosted runner*).
4. **Check with Qt** that your licence covers CI, AI assistants and shipping the browser Player.
5. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
6. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
7. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and
   tunnel in Cloudflare; rotate nine old credentials; decide the nine frozen *home-map* tiles.
8. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
9. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
