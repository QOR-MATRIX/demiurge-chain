# ADR-087: A stranger's QQ game plays in the browser, from a site of its own, on a locked-down Player

**Status:** **Accepted**, 10 October 2026, by the project owner, who agreed to the recommendation in
[`../architecture/QQ_LOGIC_SANDBOX.md`](../architecture/QQ_LOGIC_SANDBOX.md) and to the advice given on its four
questions: "Yes I agree with recommended next steps."

## Context

A QQ game's logic is QML and JavaScript (ADR-083). Played natively today it reaches whatever the player can: the same
engine as the game with nothing taken away, QQ's own file operations by path, and Qt's general-purpose QML environment
(the write-up, §1). That is right for a creator's own game and wrong for a stranger's. Publishing (DIRECTION P3.4) and
ARQADE delivery (P3.5) put strangers' games in front of players, so how a game is confined is decided first.

## Decision

1. **A stranger's game plays only in the browser, at first.** In ARQADE, on the WebAssembly Player (ADR-084), inside a
   sandboxed frame in ARQADE's page; whatever the game asks for (a score, a purchase) is asked of ARQADE through a
   narrow message interface and answered by ARQADE's own dialogs or the launcher's signing, never by the game. The
   launcher may show this browser Player in a window of its own.
2. **The QQ Player locks down what any game's logic can reach, in every mode** (native and in the browser): it loads
   QML and files only from the game's own package and QQ's and Qt's modules, imports only an allowed set of modules,
   refuses every network request, offers no file operation outside the package, and does not hand URLs to the
   operating system. This is built first, before P3.4. It is a second wall, not the boundary: the boundary is the
   browser.
3. **The native Player plays the person's own projects**, and a game they open from a folder of their own. Native play
   of other people's games waits for an operating-system sandbox, decided on its own when players ask for it or the
   browser's limits matter.
4. **Games are served from a site of their own**, never a subdomain of `qorsync.dev`, `demiurge.cloud` or ARQADE's
   address, which would share cookies with them. To start, a separate Vercel project on its own `*.vercel.app` address
   (a separate site to the browser), sending the cross-origin-isolation headers the Player needs (ADR-084); its file-size
   limits are confirmed when it is set up (the Player is a 43.8 MB file). **Before commercial launch, a registered domain
   for games**, its name the owner's to choose. Each game's saved progress is kept under its own name; a subdomain per
   game can isolate storage later if needed.

## Consequences

- The Player lockdown (decision 2) is the next work, in `products/qq/`, with checks that show each refused reach.
- The games site, the sandboxed frame and the message interface are built with ARQADE delivery (P3.5).
- Vercel's free plan is not for commercial use: the games site, like ARQADE, moves to a paid plan or another host before
  launch (already on the owner's list).
- Native play of strangers' games is not offered. The native Player's advantages (first frame in 1.4 s against 6 s, sound
  panned, no download) are kept for a creator's own games.
- Nothing here touches CGT, wallet keys or the chain.
