# ADR-102: ARQADE rebuilt inside the launcher on Qt, with Energy replaced by Sparks paid from a platform reward pool

**Status:** **Accepted**, 18 October 2026, by the owner: "Can we rebuild Arqade into the launcher as a new and improved
version? Id like to remove the energy system and replace it with Sparks. The games should be rebuilt as high quality
versions and incorporate an earning mechanic which rewards players with Sparks (fractions of CGT) … with the highest
degree of quality and graphical capability, consider using QT for all aspects of the QOR experience with the most
graphically cutting edge options possible." Asked, the owner chose **a platform reward pool** to pay players over each
game's own ARQ Wallet, **ARQADE** as the name of the launcher's Arcade with its code in `products/arqade/qt/`, and Qt for
the Arcade now with the launcher's own screens moving to Qt in stages ([ADR-103](ADR-103-the-launcher-moves-to-qt-in-stages.md)).
**Extends** [ADR-069](ADR-069-arqade-the-gaming-platform.md) and [ADR-070](ADR-070-game-vaults.md); **uses**
[ADR-083](ADR-083-qq-on-qt.md) (QQ on Qt) and [ADR-046](ADR-046-the-launcher-is-an-app-host.md) (the product host). It
decides **no economic value**: every amount, cap and period is the pool's governor's, set when the pool is funded, and
production waits on U-16.

## Context

ARQADE is a website (`products/arqade/`, Next.js on Vercel) with four solo games drawn on 2D canvases and two
multiplayer arenas, played for **Energy**: a device-local practice currency, spent to start a run and refilled for free,
with random "Energy Token" drops of no value. The launcher opens the website in a window of its own (P7.18). QQ, the QOR
Engine on Qt 6 (ADR-083), already renders physically based 3D worlds with soft shadows, filmic tonemapping, bloom,
ambient occlusion, particles, physics, spatial sound and logic, and plays natively and in a browser.

## Decision

1. **ARQADE in the launcher is a Qt 6 application,** `products/arqade/qt/` (executable `arqade`), docked inside the
   launcher under the product host as QOR Media is (ADR-046, ADR-094): started by the launcher with a one-time token,
   signed in with the launcher's QOR ID, holding no key, and asking the launcher for anything that signs. Its interface
   is Qt Quick, with Qt Quick 3D for every scene, on the RHI's Direct3D 12 or Vulkan backend.
2. **Its games are rebuilt as QQ games,** played by the QQ runtime inside ARQADE: Rift Survivor, Void Runner, Synapse and
   Orbital first, then the two multiplayer arenas. Each is rebuilt as a 3D game with physically based materials, bloom,
   particles and spatial sound, at a steady frame rate, with keyboard, mouse and gamepad, and with reduced motion
   honoured. Each game's rules are deterministic for a given seed and inputs, so a run can be replayed exactly.
3. **Energy is removed,** in the launcher and on the website. Play costs nothing.
4. **Players are rewarded in Sparks** (ADR-101) for play that meets each game's published goals, paid from **one platform
   reward pool**: the ARQ Wallet of ARQADE itself, a Cartridge of its own, so the pool is a keyless account the chain
   already knows how to limit (ADR-070). Its governor, the owner's Vault, funds it and sets its policy. Its payout
   authority is the ARQADE service. Nothing is minted: the pool pays only what it was funded with.
5. **A reward is paid only for a run the server has confirmed.** The game starts from a seed the ARQADE service issues;
   the player's ARQADE sends the run's inputs when it ends; the service replays them with the same game rules and pays
   only the outcome its own replay reaches, once (the outcome id of ADR-070), within the pool's policy. The chain itself
   enforces that policy (`pallet-arq-wallet`): the largest single payout, the pool's budget per epoch, the most one
   player receives per epoch, and which games' rule versions it pays for, so one pool serves every game. A run the
   service cannot replay pays nothing. The chain enforces the limits; the replay is what stops a forged score.
6. **The website stays** as ARQADE's public page and the place a stranger's game is played in a browser (ADR-087),
   without Energy. Its own games point players to the launcher, where they are rewarded.
7. **Devnet only.** The pool holds test CGT. Paying players real value waits on U-16 and the owner's legal review, and
   this record says so wherever rewards are shown.

**The spend test.** A reward paid to a player is neither payment for work nor licensing (ADR-008), and fails the spend
test (ADR-002) unless what funds it is itself demand to spend; U-16 records exactly that. On the devnet the pool is the
owner's promotion, funded by the owner, and names no demand sink; before production, U-16 decides whether a reward of
this kind is allowed at all and what may fund it.

## Consequences

- Roadmap items P7.19 to P7.24 in `DIRECTION.md` carry the work, in this order: the Arcade shell; the four solo games;
  the pool and replay confirmation; the website without Energy; the multiplayer arenas.
- The ARQADE service gains a replay of each game's rules, which means the rules live in one place both the QQ runtime and
  the service can run.
- The website's Energy store, Energy Tokens and the "Rewards & Wallet" view of local tokens go.
