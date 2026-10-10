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

## Done today: your rule on AI costs (ADR-086 — *a decision record*)

**Demiurge pays for no one's AI generation.** Whoever uses it pays their AI provider directly,
on their own account; we never hold a provider key or pay a bill, and take no share.
Until someone switches a feature on with their own key, it is simply off.

- QQ Studio's **Agent** panel now says this before asking for a key, and after each run shows
  how much it used. A check proves it never picks up a key it was not given.
- The old **Sophia** chat — *a frozen AI chat paid by the server* — cannot return in that form;
  its key settings are removed.
- **Selling AI generation through Demiurge** (say, for CGT) is not built. Whether to, and at
  what price, is a new open question, **U-19**.
- **Proving P3.3** (one game built from a description, no human edit) no longer waits on you:
  whoever runs it uses their own key.

## Also fixed

Pull request **#22 (P3.3) merged into #21's branch, not into main**: they were merged 16 s
apart, before GitHub could retarget it. **#23** brought it to main, then **#24** (this rule);
both are merged and automatic checks pass on main.

## What only you can do

1. **Optional: prove P3.3** — type your own Anthropic key into QQ Studio's Agent panel and
   describe a game; the run is billed to your key. Anyone with their own key can do it instead.
2. **CI** — *automatic build and test* — **for QQ:** GitHub cannot build Qt without your
   licence. Add your Qt account to its secrets, or let your PC run CI (a *self-hosted runner*).
3. **Check with Qt** that your licence covers CI, AI assistants and shipping the browser Player.
4. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
5. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
6. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and
   tunnel in Cloudflare; rotate nine old credentials; decide the nine frozen *home-map* tiles.
7. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
8. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-19, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. AI generation: $0, by rule. No spending cap (your decision, 1 October).
Qt: your Enterprise licence.
