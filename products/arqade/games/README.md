# ARQADE's games, rebuilt in QQ

ARQADE's four solo games are being rebuilt as QQ games (ADR-102 decision 2; DIRECTION P7.20). They are played in
ARQADE's Qt app ([`../qt/`](../qt/README.md)) by QQ's runtime ([`products/qq/`](../../qq/README.md)). Each game here is a
QQ package, laid out as QQ Studio keeps a project, so it also opens in QQ Studio and plays in the QQ Player as it is.

| Game | State |
| --- | --- |
| Orbital | Rebuilt, 24 October 2026: plays in ARQADE, its rules replay a run in QQ and in Node |
| Void Runner, Synapse, Rift Survivor | Not started. Their cards open today's version on the website |

## A game's package

```text
orbital/
  scenes/orbital.qml   the scene: sky, light, the core, the orbit's group, the screens and the actions
  logic/orbital.qml    the logic: hands each event to the rules, and draws what they say
  logic/rules.mjs      the rules, and nothing else: the one place they live
  assets/              the sky (an HDR image) and the sounds, written by make_assets.py
  make_assets.py       writes assets/ with the standard library only, the same bytes every time
```

ARQADE carries each package in its executable, at `qrc:/games/<name>/` (`../qt/CMakeLists.txt`), so the scene's paths,
which are relative, mean the same there as on disk.

## Rules in one place

A game's rules are a plain JavaScript module, `logic/rules.mjs`, with no QQ or Qt in it. The game's logic hands it each
event as QQ's tick delivers it (a screen button's click, an action's press, a step of 1/60 s) and draws what the rules
say. Nothing the logic draws decides anything.

So a run QQ records (its seed, and what each tick took) is the rules' input exactly. The rules' `replay(run)` plays a
run file of QQ's (`qqRun` 1) to its outcome, in Node or anywhere else that runs JavaScript. That is what ARQADE's
service will run before it confirms a run and pays for it (ADR-102 decision 5; DIRECTION P7.22, not built yet).

The rules use only arithmetic every machine computes alike: adding, subtracting, multiplying, dividing, `Math.floor`
and whole numbers. No sine, cosine, power or logarithm decides anything, because each system's maths library may differ
in its last bits. Their random numbers come from the run's seed through xoshiro128**, in 32-bit whole numbers. A test
fails if the rules' source uses one of those functions.

## Orbital

A satellite circles a gilt core. Lock the orbit while it crosses the amber arc: Space, Enter, the pad's A, or the Lock
button. Ten locks complete it, and each lock moves the arc and speeds the satellite up. Three misses lose it. P, or the
pad's Start, pauses.

Rebuilt from the website's 2D game in 3D: physically based gilt and emissive beads under bloom, embers rising from the
core, an HDR sky of dust and 260 stars, and stereo sound: two bell strikes for a lock, a falling tone for a miss, bells
climbing a chord when the orbit completes, three falling tones when it is lost, and a slow looping pad. With reduced motion the core stops turning and the words do not swell. The
satellite still moves, because it is the game.

## Tests

- `../tests/orbital.test.mjs` (Node, in `npm test` and CI): the rules are deterministic for a seed, a lock holds within
  the arc and across the turn, perfect play completes the orbit, three misses lose it, nothing moves on the title or
  while paused, and a run replays to the outcome of the same events played live. It also replays
  `../tests/fixtures/orbital-run.json`, a run QQ recorded as ARQADE's games suite played Orbital, to the outcome QQ
  reached. The same run with another seed, or with its first lock pressed 40 ticks later, reaches another.
- `../qt/tests/tst_games.cpp` (Qt Test, on the owner's PC): Orbital's card opens the game in ARQADE's window, and it is
  played by mouse, keyboard and pad to a miss, a pause and ten locks. The run ARQADE kept replays in QQ to its outcome, and so
  does the fixture above. The launcher's reduced motion reaches the game, and Esc or Back returns to the home.

Two faults were planted to check that these tests catch a change. Changing the rules' speed by 0.01 failed the
fixture's replay in Node and in QQ (7 locks and 3 misses in place of 10 and 1). Not passing the record to the game
failed the run test.

## Change a game

- Run it on its own: `qq-player products/arqade/games/orbital/scenes/orbital.qml` (QQ's README says how to build the
  Player), or open the folder in QQ Studio.
- Rewrite the assets: `python products/arqade/games/orbital/make_assets.py`.
- **A change to `rules.mjs` changes what every kept run replays to.** Make a new fixture with ARQADE's games suite:
  `ARQADE_RUN=<file>` writes the run it played. Once ARQADE's service confirms runs (P7.22), a change to the rules is a
  new rule version (ADR-102 decision 5).
