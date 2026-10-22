# Where Demiurge stands

For Andrew, 21 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 9;
**QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the gaming
website*, now with free play and no Energy. Launcher 0.1.14 is the tester download.

## Done this session, and merged by you

1. **#56: the wallet moved into the core** — *the Rust part both launchers share* (L13.2).
2. **#54: CGT handled like money that matters.** The chain's audit came out sound. QOR ID's
   old donation scheme is retired (no records lost), "1,5" is refused instead of read as 15
   CGT, ARQADE has no Energy, and CI fails on investment language. QOR ID redeployed.
3. **QQ makes real games** — *QQ is our game engine* (ADR-105). Merged: gameplay, triggers
   and timers (#55); menus, scores and pause screens (#57); rebindable controls (#59); 2D
   with sharp pixel art and levels from **Tiled** — *the free level editor* (#60);
   **fair, repeatable play** (#61): 60 steps a second, seeded random numbers, and runs that
   **replay** to the same result, so a server can check a score before any reward;
   **animation** (#62); **sound** (#63); **materials** (#64): pictures on any wall, spot
   lights, **HDR skies** — *360° images that light the world*; and the Studio's **editing**
   (#65): undo and redo, duplicate, snapping; **groups** and **prefabs** (#66) — *a group
   saved once and placed many times*; the Studio's tools (#67); and **saving** (#68) — *a
   game keeps scores and settings between plays*. Every step of the plan is done.
4. **#65 also fixed three security advisories** — *published flaws in libraries we use*;
   one, in a wrapper never built, is ignored while CI proves it, as you chose (ADR-107).
5. **#69: one look everywhere** (ADR-108, as you chose) — one shared source of colours,
   faces and motion, checked in CI. The homepage is on it and live, its fonts its own.
6. **#70: QQ Studio in the same look**, and game menus a keyboard can use.

**For your merge: #71, ARQADE in the look**, its website and its app in the launcher, held
there by a CI check, and the website's welcome screen fixed (live, it sits in a corner).

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
