# Where Demiurge stands

For Andrew, 19 October 2026 (late night). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 9;
**QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the gaming
website*, now with free play and no Energy. Launcher 0.1.14 is the tester download.

## Done this session, and merged by you

1. **#56: the wallet moved into the core** — *the Rust part both launchers share*: the next
   Android step (L13.2). Windows unchanged: every test passes, and the phone app still builds.
2. **#54: CGT handled like money that matters.** An audit found the chain sound. Fixed:
   - QOR ID's old donation scheme ("staking" bonuses for dollars) is retired, no records lost.
   - "1,5" is refused instead of read as 15 CGT.
   - ARQADE has no Energy: no entry cost, no random token drops.
   - CI now fails on investment language.
3. **#55: QQ makes real games** — *QQ is our game engine* (ADR-105): one engine for 3D and 2D.
   Built: triggers, collisions, rays, spawning, timers, pushes and moving platforms, with
   *Orb Run*, a small game made only of a scene and a script. Next: on-screen menus and
   scores, button mapping, then 2D.

QOR ID redeployed with its cleaned-up database on its own.

**Merged since:** QQ's game interface (P3.8, #57): titles, scores, bars, buttons and pause
screens over the game, in our own typefaces. **Built since:** controls a player can rebind
(P3.9): any key, mouse or pad button, and on-screen buttons for touch. Orb Run pauses on P
or the pad's Start. It is pull request **#59**, waiting for you to merge.

## Toward a worldwide beta: what only you can do

1. **Terms of Service and a privacy policy, with a lawyer.** Three live services hold
   emails and messages, and none has either. Account deletion and export follow from them.
2. **The legal opinion** on rewards, prizes, the welcome grant and backing, and on
   KYC — *identity checks* —, sanctions and age limits, before anything carries value.
3. **The economic numbers** — *issuance, the genesis split, the decay curve, fee and burn
   shares* (OPEN-1 to OPEN-4, U-4, U-11). Most of the Beta gate waits on them.
4. **Code-signing certificates** for Windows and Apple, and an **external security review**.
5. **Names** for the indexer and explorer, the SDK and the agents' MCP server.
6. **Still open:** godmode's 100,000,000 CGT (its address); Social's report rules; the
   Android app on your phone; ARQADE in 0.1.14; the gate change in #46; the reward pool's
   numbers; Search Console and the GitHub preview image; the welcome-grant payer; the last two
   L1.4 dialogs; Qt for CI; QQ Studio's update; `social-secrets.txt`; `archive-bZLIQp/`;
   the email webhook; the key pasted on 5 October.

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
