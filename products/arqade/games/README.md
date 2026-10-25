# ARQADE's games, rebuilt in QQ

ARQADE's four solo games and its two arenas are rebuilt as QQ games (ADR-102 decision 2; DIRECTION P7.20 and P7.24).
They are played in ARQADE's Qt app ([`../qt/`](../qt/README.md)) by QQ's runtime ([`products/qq/`](../../qq/README.md)).
Each game here is a QQ package, laid out as QQ Studio keeps a project, so it also opens in QQ Studio and plays in the QQ
Player as it is (an arena there has nobody to ask, so it stays in its lobby).

| Game | State |
| --- | --- |
| Orbital | Rebuilt, 24 October 2026: plays in ARQADE, its rules replay a run in QQ and in Node |
| Void Runner | Rebuilt, 24 October 2026: the same |
| Synapse | Rebuilt, 24 October 2026: the same |
| Rift Survivor | Rebuilt, 24 October 2026: the same |
| Flux Four and Rift Reversi | Rebuilt, 24 October 2026: play in ARQADE against another person, on ARQADE's server; a run replays in QQ and in Node |

## A game's package

```text
orbital/
  scenes/orbital.qml   the scene: sky, light, the core, the orbit's group, the screens and the actions
  logic/orbital.qml    the logic: hands each event to the rules, and draws what they say
  logic/rules.mjs      the rules, and nothing else: the one place they live
  assets/              the sky (an HDR image) and the sounds, written by make_assets.py
  make_assets.py       writes assets/ with the standard library only, the same bytes every time
```

The two arenas are one package, `arenas/`, with a scene for each (`scenes/flux-four.qml`, `scenes/rift-reversi.qml`),
one logic that draws either board (`logic/arena.qml`), and in place of rules their conversation with ARQADE
(`logic/arena.mjs`).

ARQADE carries each package in its executable, at `qrc:/games/<name>/` (`../qt/CMakeLists.txt`), so the scene's paths,
which are relative, mean the same there as on disk. Each package keeps its own rules whole, random numbers and all: a
game reaches nothing outside its package (ADR-087 decision 2), so two games never share a file.

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
fails if the rules' source uses one of those functions. Rift Survivor's rules also take square roots, for distances and
aim: `Math.sqrt` is IEEE 754's correctly rounded square root, which ECMAScript specifies exactly, unlike `Math.hypot`
and the trigonometric functions. Where a shot's spread turns, the cosines and sines of its angles are written in as
numbers.

**Every change to the rules shows.** A small change can leave one run's outcome as it was, so each game's test also pins
the rules' whole state at the end of its kept run, as a SHA-256 of it. Any change to the rules fails that test until the
fixture and the digest are made again. Once ARQADE's service confirms runs (P7.22), such a change is a new version of
the rules (ADR-102 decision 5), and runs kept under the old ones replay under those.

## Orbital

A satellite circles a gilt core. Lock the orbit while it crosses the amber arc: Space, Enter, the pad's A, or the Lock
button. Ten locks complete it, and each lock moves the arc and speeds the satellite up. Three misses lose it. P, or the
pad's Start, pauses.

Rebuilt from the website's 2D game in 3D: physically based gilt and emissive beads under bloom, embers rising from the
core, an HDR sky of dust and 260 stars, and stereo sound: two bell strikes for a lock, a falling tone for a miss, bells
climbing a chord when the orbit completes, three falling tones when it is lost, and a slow looping pad. With reduced
motion the core stops turning and the words do not swell. The satellite still moves, because it is the game.

## Void Runner

A ship runs a track of three lanes into the collapse on the horizon, and what the collapse throws rushes down the lanes
at it. A red hazard that reaches it in its lane breaches its hull; a gold gem is caught, for 100 points. Forty-five
seconds survived completes the run, and three breaches end it; after a breach the hull holds against the next for 0.8
seconds. The rush quickens as the run goes on. A run also scores ten points a second.

Change lane with A and D, the arrows, the pad's d-pad or shoulders, or the Left and Right buttons on the screen. P, or
the pad's Start, pauses.

Rebuilt from the website's 2D game in 3D: a dark track between ember rails whose bars run toward the eye as fast as what
is thrown, the collapse half-risen on the horizon under bloom, red slabs and turning gold, a ship with an engine's wake
of embers, an HDR sky of the collapse's light and 180 stars, and stereo sound: a bell pair for a gem, a thud for a
breach, a glancing tick when the hull holds, a rush of air for each change of lane, and a pulsing loop at 120 beats a
minute. What is thrown is drawn from two pools made as play starts, so nothing is made or removed during a run. With
reduced motion the gems do not turn, the collapse does not pulse, the hull's warning is steady rather than blinking, and
the words do not swell; the track still runs, because it is the game.

## Synapse

Four nodes of a mind, Origin, Echo, Pulse and Aether, light one after another in a signal; then the player repeats it,
node by node. Each signal repeated grows by one node, and a signal of eight repeated masters the game for 1,500 points.
One node out of turn loses it, scored 150 for each whole signal repeated and 20 for each node of the last. While the
signal shows, a press is no answer. The keys 1 to 4, the pad's X, Y, B and A (or its d-pad), or the four buttons along
the foot of the screen press the nodes; P or Start pauses.

The rules count the signal in steps: a lead of 30, then each node lit for 28 steps, 42 apart (the website's 500, 460 and
700 ms). Rebuilt in 3D: four glass nodes in a gentle arc, each on a ring of its own colour, with dendrites reaching out
and gilt filaments joining them, under an HDR sky of faint teal and violet cloud and 160 stars. A node lit, or pressed,
glows, swells and throws sparks, and sounds its note (C, E, G and C, struck glass); a signal repeated, a node out of
turn and the game mastered each have a sound, under a quiet drone. With reduced motion the nodes glow and sound but do
not swell.

## Rift Survivor

One ship in an arena for ninety seconds. Enemies come in from its edges and close on it: drones that one shot destroys,
and later armoured wardens that take three. The ship fires on the nearest by itself; the player moves it. Each kill
scores, more in a chain of kills, and leaves a fragment to collect for 15. After five seconds, rift pulses open where the
ship is, arm 1.35 seconds later and breach the hull of a ship still inside. Every so many kills the signal evolves: the
world holds while the player chooses a perk (Overclock, fire a quarter faster; Fracture, a wider burst; Second wind,
two hull back and fragments drawn from further). Four breaches end the run, ninety seconds survived completes it, and
the score adds five a second. Move with W, A, S and D, the arrows, the pad's d-pad, or the screen's Up, Left, Right and
Down; choose a perk with 1, 2 or 3, the pad's X, Y or B, or its button; P or Start pauses.

The rules are the website's, step by step, in its units (an arena of 1000 by 620): the same speeds, reaches, timings
and scores, with time counted in steps and movement taken from four held actions, pressed and released, so a replay
moves exactly as the run did. Rebuilt in 3D: the arena a dark board tilted toward the eye under a teal grid, teal drones
and ember wardens turning, white shots, gold fragments, rifts that fill as they charge and glow when armed, a ship with
an ember wake, under an HDR sky of teal spiral arms around an ember heart. Everything the rules hold is drawn from pools
made as play starts, and only pieces that stop showing are put away each frame.

**Sound from where it happens.** A sound that happens somewhere is played from there: the logic passes QQ's `play` a
place (`at`), and QQ plays it through Qt's spatial audio, heard through the eye, panned to its side and quieter further
off. Orbital's lock and miss come from the satellite, and Void Runner's from the ship's lane. Synapse's notes come from
their nodes, so a signal moves left to right as it is heard. Rift Survivor's kills come from the enemy, its rifts from
where they open, and its shots, fragments and breaches from the ship. Music and what belongs to the screen (a run's end,
a signal repeated, a perk chosen) are heard everywhere. Qt's spatial sounds take one channel and its ambient sounds two,
each warning of the other, so each `make_assets.py` writes the sounds its game places in mono and the rest in stereo.

**Colour under bloom.** QQ's tonemapping turns a glow much past 1 toward white or yellow: Orbital's amber arc washed out
to the satellite's white, and Void Runner's red hazards to white and its collapse to yellow. Each glow that carries a
meaning is kept near or under 1, which keeps its hue. Synapse's lit node is capped at 0.77 for the same reason.

## The arenas: Flux Four and Rift Reversi

Two people play one board on ARQADE's server, which keeps every match and decides every move, as it does for the
website's arcade (`../lib/arena-store.ts`, `../lib/arena-engine.ts`). The arenas have no rules here. Flux Four drops
pulses into seven columns six deep, and four in a row, across, up or diagonally, win. Rift Reversi places discs on six
by six, turning over the opponent's discs each one brackets, and the most discs at the end win. A move has 90 seconds.
The server ranks every match, as it does the website's.

**The conversation.** A QQ game reaches no server of its own (ADR-087), so an arena asks its host, ARQADE's app, through
QQ's host channel (ADR-117): `askHost` to ask, `onTold` to hear. ARQADE's `GameView` answers from the server through
`ArqadeServer`, with the player's session. It answers with the arena's open lobbies from the live arcade, read every 3
seconds while the lobby shows, and with each match opened, joined, moved in, conceded or closed. It asks after the
match being followed every 1.2 seconds, as the website does, and tells the game only what changed, so the run keeps
each answer once. A match that could not be joined or opened is not followed, and the lobby is told again. `arena.mjs` is the game's side:
what it asks for each thing a player does, what it makes of each answer (never an older board than the one drawn, nor
another match's), and what the screens say. It decides no move: a move it asks for is one the server listed as legal,
and the server checks it again. It keeps no clock and draws no chance: the seconds left are counted down in QQ's steps
from the server's answer.

**Playing.** From the lobby a player opens a match, joins the newest open lobby of someone else's, or returns to their
own match. Their own lobby waits for a challenger, and can be closed before one comes. In Flux Four, left and right (A
and D, the arrows, the d-pad) choose a column and Space, Enter or the pad's A drops a pulse; the keys 1 to 7 drop one
straight into a column. In Rift Reversi, the arrows (W, A, S and D, the d-pad) move a cell at a time, Q and E (the pad's
shoulders) jump between the legal moves, and Space, Enter or A places a disc. In both, the pointer over the board aims,
and a click moves. Concede asks once more before it concedes. Lobby leaves a match while it goes on, its clock still
running. A move the server refuses says why for five seconds.

**In 3D.** Flux Four is an upright frame of wells between bronze rails on a plinth. Its pulses drop into their column
and come to rest with a bounce, and four in a row pulse when the match is won. Rift Reversi's board is teal tiles in a
pale frame: a disc set down grows into place, and each disc it turns lifts and flips over to the other colour. Both
stand nearly facing the eye, turned a little so their depth shows, and a lamp beside the eye lights them and casts their
shadows. The player's move waits over the board in their colour, the host's gold or the guest's cyan. In Flux Four it is a
pulse held over the column, bobbing, with a ring where it would land. In Rift Reversi it is a see-through disc,
breathing, with each legal cell marked. The last move holds a spark. A pulse landing, a disc set down and the discs
turning are heard from where they happen. The player's move coming round, a challenger arriving, the match's end and
the music are heard everywhere. With reduced motion, pulses arrive and discs turn at once, and nothing bobs, breathes or
pulses.

**Its run.** An arena's run is what the host told it and what the player did, and its outcome is the server's verdict
on the match: `{ match, seat, result, moves }`. `arena.mjs`'s `replay(run, game)` plays a run in Node to the same state.
Where the pointer was belongs to the window, not the run, so a replay makes nothing of a click on the board. The move it
asked for is in the server's answer, which the run keeps. An arena's run is not rewarded: the server ranks its matches.

## Tests

- `../tests/orbital.test.mjs`, `../tests/void-runner.test.mjs`, `../tests/synapse.test.mjs` and
  `../tests/rift-survivor.test.mjs` (Node, in `npm test` and CI), 9, 9, 10 and 11 tests. Each game's rules are deterministic for a seed and replay a run to the outcome of the same
  events played live. Orbital: a
  lock holds within the arc and across the turn, perfect play completes the orbit, three misses lose it. Void Runner:
  something is thrown every 38 steps and the rush quickens, a hazard in the lane breaches the hull and three end the
  run, a pilot who dodges survives forty-five seconds whole, and the lanes stop at the edges. Synapse: the signal's
  timing to the step, a press while it shows is no answer, eight signals master the game, and one node out of turn loses
  it with its score. Rift Survivor: the same run for a seed, enemies' timing and kinds, the ship's movement and the
  arena's edges, shots, chains and fragments, a still ship breached four times with the hold between, the signal evolving
  and each perk, and a pilot who keeps away lasting far longer than a still one. For each, nothing moves on the title or
  while paused, and the rules' source uses no sine, cosine, power or logarithm.
- Each also replays its run kept in `../tests/fixtures/`, a run QQ recorded as ARQADE's games suite played the game, to
  the outcome QQ reached; the same run with another seed, or altered (Orbital's first lock 40 ticks later, Void Runner's
  lanes never changed, Synapse's node out of turn taken away, Rift Survivor's ship never moved), reaches another. And each pins the rules' whole state at
  the end of that run.
- `../qt/tests/tst_games.cpp` (Qt Test, on the owner's PC): each card opens its game in ARQADE's window, from the
  keyboard. Orbital is played by mouse, keyboard and pad to a miss, a pause and ten locks. Void Runner is steered
  away from hazards and toward gems, changing lane by the keys, the arrows, the pad and the screen's buttons in turn,
  until each has been used and a gem caught, then steered into three hazards. Synapse is pressed while its signal shows
  (no answer), then three signals are repeated, each node pressed by the keys, the pad and the buttons in turn, then a
  node out of turn loses it for 450. Rift Survivor is moved each way there is (D, the left arrow, the d-pad's down and
  the screen's Up button), steered clear by the keys until ten kills evolve the signal, a perk chosen by its button while
  the world holds, then left standing until four breaches end it, a perk chosen by its key each time the signal evolves
  meanwhile. Each game's placed sounds are checked where they happen: a lock within a unit of the satellite, a change
  of lane from a placed voice, each Synapse note at its own node, a kill from a place in the arena. The run ARQADE kept
  of each replays in QQ
  to its outcome, and so does each fixture. The launcher's reduced motion reaches the game, and Esc or Back returns to
  the home.
- `../tests/arenas.test.mjs` (Node, in `npm test` and CI), 17 tests of the arenas' conversation, against boards made
  by the server's own rules (`../lib/arena-engine.ts`). It covers the lobby kept to the arena's own matches, signed out
  saying why and asking nothing, a match opened, closed and joined, a move as the cursor and the board last seen, an
  older board or another match's never drawn, the cursor through a full column and onto the nearest legal move, a
  concession asked twice, a Space that pressed a button not moving, a refusal fading, the clock counting down, each
  result's words and outcome, and what is told read and never trusted. Both runs kept in `../tests/fixtures/` replay to
  QQ's outcomes, and not without the server's last answer. The whole state at the end of each is pinned, and
  `arena.mjs` uses no clock or chance of its own.
- `../qt/tests/tst_games.cpp` plays both arenas against a stand-in ARQADE that signs in as the website does, keeps
  matches as `arena-store.ts` keeps them and moves for the opponent. Signed out, an arena says why and offers nothing.
  Flux Four opens a match, a challenger joins, and four pulses go down one column by Space, the key 5, the pad's A and
  the pointer, each heard where it lands, to win. Then a rematch's lobby is closed before anyone joins. In Rift
  Reversi, a lobby someone else takes just before the player joins it is said to be no longer available, and the lobby
  then shows none open. Then Rift Reversi joins someone's open lobby. Its legal moves are marked, the arrows and E move the cursor, Space places a disc whose
  sound comes from where it lands, and the discs it brackets turn to the guest's colour. A move the server refuses says
  why. A disc is placed by the pointer, and the match is conceded only after Concede is asked twice. Each arena's kept
  run, and its fixture, replays in QQ to its outcome.

Faults planted to check these tests catch a change:

- Orbital's speed changed by 0.01: the fixture's replay failed in Node and in QQ (7 locks and 3 misses in place of 10
  and 1).
- Orbital's game not given the record: the run test failed.
- Void Runner's acceleration changed from 0.004 to 0.0041: the fixture's outcome did not move, which is why the state
  is pinned. The pinned state then failed, as did Orbital's for its speed changed by 0.0001.
- Void Runner's logic not handing presses to the rules: the pilot's first change of lane failed.
- Synapse's gap between nodes changed from 42 steps to 41: the pinned state failed. Its time lit changed from 28 to 27,
  which only the drawing reads: the timing test failed. Its logic not handing presses to the rules: the second signal
  never came.
- Rift Survivor's hold after a breach changed from 96 steps to 95: the pinned state failed. Its logic not handing
  releases to the rules: the left arrow did not move the ship (right was still held).
- The arenas' conversation drawing an older board, or another match's; a Space that pressed a button also moving; the
  cursor not set on a legal move; the clock at half speed; a refusal leaving the ask waiting; a call to `Date`: each
  failed `arenas.test.mjs`.
- QQ handing a host's message on through the signal's own argument, which makes its lists Qt's rather than arrays:
  QQ's play test failed (and the arenas' lobby never read as signed in). ARQADE's server posting a move without its
  origin: the server's own suite failed. ARQADE's window never asking after the followed match: the Flux Four test
  never saw its challenger join.

## Change a game

- Run it on its own: `qq-player products/arqade/games/<name>/scenes/<name>.qml` (QQ's README says how to build the
  Player), or open the folder in QQ Studio.
- Rewrite the assets: `python products/arqade/games/<name>/make_assets.py`.
- **A change to `rules.mjs` changes what every kept run replays to.** Make new fixtures with ARQADE's games suite:
  `ARQADE_RUN=<folder>` writes each run it played there, as `<name>-run.json`. Copy the game's into `../tests/fixtures/`,
  set the outcome and the digest its test expects, and say why in the commit. The same goes for the arenas'
  `arena.mjs`, whose replayed state is pinned as the rules' is.
