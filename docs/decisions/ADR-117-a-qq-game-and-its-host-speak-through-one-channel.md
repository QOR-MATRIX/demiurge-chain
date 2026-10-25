# ADR-117: A QQ game and its host speak through one channel, kept in the run

**Status:** **Accepted**, 24 October 2026, under the owner's delegation of engineering choices (the owner's words that
day: "I authorize full automation and decisions permissable. You are approved to continue development ops with best
recommendations and highest quality when possible."), as the first step of the arenas in QQ (DIRECTION P7.24), the
next roadmap item that waits on no owner decision. The owner may amend it. It decides **no economic value**.

**Relates to:** [ADR-087](ADR-087-strangers-games-are-confined.md), whose decision 1 has a stranger's game ask ARQADE
for what it needs "through a narrow message interface", answered by ARQADE and never by the game, and whose decision 2
keeps a game from reaching anything outside its package. [ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md) decision 2:
the arenas are rebuilt as QQ games played inside ARQADE. [ADR-105](ADR-105-qq-makes-real-games-in-3d-and-2d.md)
decision 6: a run is its seed and what each tick took, so a replay reaches the same outcome.

## Context

A QQ game reaches nothing outside its package: no network, no file but its own store (ADR-087 decision 2). That is
right for every game, ARQADE's own included. But the arenas are played against another person on ARQADE's server, and
a game that may not call the server cannot play them. Nor can a stranger's game ask ARQADE for a score or a purchase,
as ADR-087 decision 1 says it will, through an interface not yet built.

What a game needs is not reach but a host to ask. The host is the application that plays the scene: ARQADE's app for
its games, the browser page for a stranger's (P3.5). It decides what to answer.

## Decision

1. **One channel, both ways, of JSON values.** A game's logic asks with `askHost(message)` and hears what the host tells
   it in `onTold(message)`. The host hears the game in the `asked` signal of the `Game` (or `Scene`) playing it, and
   tells it with `tell(message)`. A message is a JSON value of at most 16 KiB, copied as JSON copies it, so neither side
   holds anything of the other's. At most 64 told messages wait for a tick; anything else is refused and says so. The
   logic hears a JSON value as JavaScript has one, in play and in a replay alike: its lists are arrays. (A QML signal's
   own argument would make them Qt's lists, which `Array.isArray` does not know, so the runtime hands the value on
   itself.)
2. **What the host tells is heard on the game's clock and kept in the run.** It waits for the next tick, which delivers
   it to every logic after that tick's screen-button clicks and before its presses and releases. The run record keeps
   it with that tick (`told`, in format 1's file; a run kept before this has none), and a replay delivers it at the same
   tick. So a game's outcome may rest on what its host told it, and its run still replays to that outcome.
3. **What the game asks goes out at once, and never from a replay.** Asking is output: it changes nothing in the game
   until an answer is told, and an answer is in the record. A replay asks nothing.
4. **The host decides.** The channel gives a game no reach of its own. A host answers only what it chooses to: ARQADE's
   app answers its arenas from its own server session, and a browser host will answer a stranger's game through
   ARQADE's dialogs and the launcher's signing, as ADR-087 decision 1 says. The QQ Player hosts nothing; a game asking
   there is not answered.
5. **The channel is QQ's, not ARQADE's.** It lives in QQ's runtime (`Logic.qml`, `Scene.qml`, `Game.qml`, `run.h`), so
   every host uses the same one: the browser Player will relay it to its page when P3.5 is built.

## Consequences

- A game played inside ARQADE can be multiplayer without any reach of its own. The arenas' QQ scenes, built in the same
  change, ask ARQADE's app for their lobby, a match and a move, and are told the match as it changes (P7.24;
  `products/arqade/games/arenas/logic/arena.mjs` lists the messages).
- Runs keep a new list, `told`, which older readers ignore. The four solo games tell and ask nothing, so their runs and
  their rules' replays in Node are unchanged.
- Tested in QQ's play suite: a game asks as it starts, hears what it is told one tick later, its lists, however deep,
  arrays, a message over 16 KiB is refused, and its run replays to the same outcome, from its file too; without what it
  was told, it does not. A record that kept nothing told was seen to fail it, and so was the message handed on through
  the signal's own argument.
- Not built here: the browser Player's relay to its page (P3.5), and any message a stranger's game may send, which is
  for P3.5 to list and ARQADE to answer.
