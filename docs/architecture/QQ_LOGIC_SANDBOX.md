# Confining a stranger's game: a design proposal (QQ, before P3.4 and P3.5)

**Status:** **Decided 10 October 2026: ADR-087**, the owner accepting the recommendation in §4 and the advice on §5's
four questions (browser only at first; the Player locked down in every mode, built first; native play of others' games
deferred; games served from a site of their own, a separate `*.vercel.app` project now and a registered domain before
launch). This document is the write-up ADR-087 was decided from, as ADDRESS_TYPE.md was for ADR-041. **Option C, the
Player's lockdown, is built** (10 October 2026: `products/qq/runtime/confinement.h`, checked by `tst_lockdown`), with two
known limits: URLs are refused to the operating system scheme by scheme, and Qt Multimedia cannot be refused natively,
because Qt's spatial audio library registers it.

**What it answers:** how QQ keeps a game made by one person from harming the person who plays it, once games are
published (DIRECTION P3.4) and played by others in ARQADE and the launcher (P3.5). Until then every game is played only
by the person who made it, and nothing here is urgent; afterwards it is the condition for P3.4 and P3.5 to ship.

**Decided already, and not reopened here:**

- **ADR-083:** QQ is Qt 6; a game's scenes and logic are QML, and logic is plain QML and JavaScript, "reloaded while the
  world runs". The Player runs natively from the launcher and as WebAssembly in ARQADE.
- **ADR-084:** the browser Player is Qt's threaded WebAssembly build, served cross-origin isolated.
- **ADR-086:** nothing generative runs on the project's account.
- **The Studio never holds a key** (ADR-083 decision 5); signing is the launcher's, behind its own dialog.

---

## 1. What a game's logic can do today

From the code on `main` (10 October 2026), not from experiment:

- **Logic runs in the same QML engine as the game, with nothing taken away.** `LogicFile` (`products/qq/runtime/
  logicfile.cpp`) compiles a logic file into the engine the Player or Studio created (`qmlEngine(this)`). Neither
  `player/main.cpp` nor `studio/main.cpp` installs a URL interceptor, a network-access policy or an import restriction.
- **QQ's own module offers file operations by path.** `SceneIO` (`runtime/sceneio.h`), which every logic file reaches
  through `import QQ`, has `save`, `writeLogic` and `adopt`, which write or copy files at locations their caller names.
  They exist for the Studio and the agent; nothing stops a game's logic calling them.
- **Qt's QML environment is a general-purpose one.** It offers network requests, opening URLs with the system's
  handlers, loading further QML from URLs, and every QML module shipped beside the program (the deployed Studio carries
  Qt Multimedia, among others).

**So, natively, a game's logic can do what the person running it can do.** That is right for a creator's own game, and
wrong for a stranger's. In the browser the browser's own sandbox applies, but a game served from ARQADE's own origin
could still act as the player there (its requests carry that origin's cookies).

## 2. What must hold

For a game the player did not make:

1. It cannot read, write or delete the player's files, other than its own saved progress.
2. It cannot reach the network, other than its own package and any service it is explicitly allowed.
3. It cannot act as the player anywhere: not on QOR ID, ARQADE, the launcher's vault or the chain. Anything it asks for
   on the player's behalf (a purchase, a signature) goes through a dialog that is not the game's.
4. It cannot start other programs, open the camera or microphone, or read the clipboard.
5. Breaking out needs a flaw in an engine built for this (a browser, the operating system's sandbox), not one missed
   item in a list QQ keeps itself.

The fifth is why "remove the dangerous parts from QML" is not enough on its own: the surface is large, grows with every
Qt release, and one item missed undoes it.

## 3. The options

### A. Strangers' games play only in the browser, from an origin of their own

The Player's WebAssembly build (ADR-084) runs games in ARQADE. Each game is served from a separate origin that holds
no cookies or sessions (for example a dedicated games domain), embedded in ARQADE's page as a sandboxed frame, with a
content security policy that allows network access only to the game's own files. ARQADE and the game talk through a
narrow message interface (a score, a request to buy, handled by ARQADE's own dialog and the launcher's signing).

- **Holds 1 to 5**, with the browser as the boundary, the one engine built for running strangers' code.
- **Cost:** a games origin and its hosting (static files; ADR-086 does not apply, this is not generation); the
  cross-origin-isolation headers ADR-084 requires must also be set on the frame; a message interface to design.
- **Gives up:** native play of other people's games, until B or C exists. The launcher can still open the browser
  Player in its own window.

### B. A native Player inside the operating system's sandbox

The launcher starts the Player for a stranger's game as a sandboxed process: on Windows an AppContainer with no file,
network or device capabilities except read access to the game's own package and a folder for its saved progress. The
Player talks to the launcher over the channel the launcher opens (as ADR-083 decision 5 already plans for signing).

- **Holds 1 to 5** with the operating system as the boundary.
- **Cost:** substantial and per platform: process creation with capabilities from the launcher (Rust), checking the
  Player's rendering, audio and input work inside the container, and the same again for macOS and Linux later.
- **Gives up:** nothing for players; it is the most work.

### C. A confined QML engine for games (defence in depth, not a boundary)

The Player, when playing any game: a URL interceptor that lets the engine load only the game's own package and QQ's
modules; a network-access policy that refuses everything; QQ's file operations missing from the module the Player
gives logic; opening URLs with the system refused; saved progress only through a small API of QQ's.

- **Holds 1 to 4 as far as the list is complete; fails 5.**
- **Cost:** small: a few days, all in `products/qq/`.
- **Gives up:** nothing a game needs.
- **Worth doing under A or B as well**: it makes a mistake elsewhere less likely to matter.

### D. Logic from a fixed set of behaviours instead of code

Stranger's games may use only declared behaviours (spin, follow, collect, trigger, timer, score...) with parameters:
data, not code. A creator's own games keep full logic.

- **Holds 1 to 5** for logic, since there is no code to confine.
- **Cost:** designing a behaviour set rich enough to make games worth playing; the agent (P3.3) would need to build
  with it.
- **Gives up:** much of what makes QQ worth making games in (ADR-083 chose QML for "a language large models write
  well").

### E. Review before publishing

A person or a model reviews each game's logic before it can be published.

- **Holds nothing reliably:** a review misses things, and a game can change what it does after review by loading
  more at run time. Useful for quality and for the rules on what may be published; not as a security boundary.

## 4. Recommendation

**A, with C, now; B later, if native play of other people's games is wanted.**

1. **P3.5 ships strangers' games in the browser only** (A), from a games origin of their own, sandboxed in ARQADE's
   page, talking to ARQADE through a narrow message interface. This is the delivery ADR-083 already plans for ARQADE,
   with the origin and frame rules added.
2. **The Player confines its QML engine in every mode** (C): small, cheap, and it keeps a slip in A's configuration
   from becoming a hole.
3. **The native Player plays only the person's own projects**, and a game they explicitly open from a folder of their
   own, saying what that means. Native play of others' games waits for B, as its own decision when wanted.
4. **D is not needed** while A and B are the boundary; a behaviour library can still come later as a convenience.
5. **E** belongs to publishing rules, not security.

## 5. What the owner decides

1. **Whether strangers' games play only in the browser at first** (A), accepting no native play of others' games
   until an OS sandbox (B) is built.
2. **Whether to build C** in every mode (recommended), and **whether B is wanted** at all, and when.
3. **The games origin:** a domain of its own (recommended: no cookies, no sessions, nothing else served there), and
   where it is hosted (HOSTING.md).
4. **What a game may ask ARQADE for** through the message interface at first (a score, a purchase), each answered by
   ARQADE's or the launcher's own dialog.

**Not decided here:** prices, purchases in CGT, collectibles (P3.6), and the publishing rules themselves (P3.4).
