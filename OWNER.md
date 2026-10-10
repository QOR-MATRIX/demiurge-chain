# Where Demiurge stands

For Andrew, 10 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8. 150 tests.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`; signs you in to ARQADE's window. 172 tests.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`. 49 tests.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. QQ's first milestone (P3.1) is merged.

## Done today: QQ worlds play (P3.2)

**Play** (F5) in QQ Studio runs the scene and **Stop** (Esc) restores it exactly. Crates fall;
a **player** walks, runs and jumps by keyboard, mouse or gamepad; sparks, glows, placed sounds;
**game logic** you can edit while it runs. The **QQ Player** runs a game alone: first picture
in 1.4 s on your PC, 6 s in Chrome (a 12 MB download). Try: `pwsh products/qq/build.ps1 -Run`.
**Limits:** browser sounds do not pan (ADR-084 — *a decision record*); game logic is full code,
to be fenced in before strangers play each other's games.

## Also today: AI in QQ Studio (P3.3, all but its proof)

An AI assistant such as Claude can connect to a running Studio and build, play, take pictures
and read errors; it cannot save or commit, and only your Windows account can connect. The
**Agent** panel goes further: describe a game, and Claude designs, builds, plays, looks and
revises, every step shown; you save it or undo it. Your API key stays in Windows Credential
Manager, never in a file or log; runs are billed to it.

**Next:** prove P3.3 with one game built from a description with no human edit. Needs your key.

## What only you can do

1. **Merge pull request #21** (P3.2): all its checks pass. Then #22 (P3.3).
2. **An Anthropic API key** in the Agent panel, so the design loop can be proven (ADR-085 chose
   Claude Opus 5.5: $4 / $20 per million tokens in / out).
3. **CI** — *automatic build and test* — **for QQ:** GitHub cannot build Qt without your
   licence. Add your Qt account to its secrets, or let your PC run CI (a *self-hosted runner*).
4. **Check with Qt** that your licence covers CI, AI assistants and shipping the browser Player.
5. **Name the welcome-grant payer** — *the server that pays new players' 100 test CGT*.
6. **Add an email** at `id.qorsync.dev/account`; **replace the key** pasted into a chat on 5 October.
7. **Housekeeping:** delete `QOR-MATRIX/arqade`; the Resend bounce webhook; the `ci` record and
   tunnel in Cloudflare; rotate nine old credentials; decide the nine frozen *home-map* tiles.
8. **Before commercial use:** a paid Vercel plan, a legal review, trademark clearance.
9. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-18, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. No spending cap (your decision, 1 October). Qt: your Enterprise licence.
