# ADR-116: Confirming an ARQADE run by replay: what a run is, how a rule version is named, what the service accepts

**Status:** **Proposed**, 24 October 2026, for the owner to accept, amend or reject. Nothing is built on it until the
owner decides. It decides **no economic value**: the pool's numbers (the largest payout, the budget per epoch, the most
one player receives) stay the owner's, in DIRECTION P7.21, and whether a reward of this kind may be paid at all stays
U-16's.

**Relates to:** [ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md) decision 5, which this fills in: a reward is paid
only for a run the server has confirmed by replaying its inputs with the same rules, once, within the pool's policy.
[ADR-070](ADR-070-game-vaults.md) decisions 6 and 7, whose policy names the rule versions payouts may name and whose
outcome id makes a payout happen once. [ADR-105](ADR-105-qq-makes-real-games-in-3d-and-2d.md) decision 6, QQ's fixed
steps and run records. DIRECTION P7.20 (done), P7.21 and P7.22.

## Context

The four solo games are rebuilt in QQ and merged (P7.20). Each game's rules are one plain module,
`products/arqade/games/<game>/logic/rules.mjs`, which QQ's logic runs in the game and Node runs as well. A run QQ
records is a JSON file of format 1: the seed, the ticks, and each tick's presses, releases and screen-button clicks.
The rules' `replay(run)` plays such a file to the outcome the game reached, and tests show it does for a run of each
game recorded in ARQADE (`products/arqade/tests/fixtures/`).

ADR-102 decision 5 says what the service does with a run, but not five things P7.22 needs:

- **What one run is.** ARQADE keeps one record from the moment a game's card opens until the player leaves, and a
  game's own "Play again" starts a new game inside the same scene, on the same seed's later numbers. So one record can
  hold several games.
- **How a rule version is named,** so the pool's policy (ADR-070 decision 6) can list the versions it pays for.
- **Where the seed comes from,** and for how long it is good.
- **What the service accepts,** so a run file cannot cost it more than a run is worth.
- **What a replay does not prove.**

## Decision (proposed)

1. **One run is one game, from its first step to its end, on a seed the service issued for it.** Pressing Play asks the
   service for a seed, and the game starts on it. Inside ARQADE, "Play again" starts a new run: ARQADE asks for a new
   seed and starts the scene afresh (`qml/GameView.qml`), rather than letting the game continue on the old seed. A run
   left before its end is abandoned and confirms nothing.

   *Rejected:* one run for each visit, from the card to Esc. It would make one outcome of several games, and a player
   could retry on one seed until it went well.

2. **A rule version is the SHA-256 of the game's `rules.mjs`,** its bytes with line endings made `\n` (a checkout on
   Windows writes `\r\n`), named `<game>:<hex>`, for example `orbital:3f2a…`. Any change to the rules is a new version,
   by construction; nobody has to remember to number it, and a change to a comment makes a new version too, which costs
   only a line in the policy. The pool's policy lists the versions it pays for (ADR-070 decision 6). Each game's Node
   test pins the rules' whole state at the end of a kept run, so a change that moves that run fails until it is meant.

3. **The service issues the seed.** `POST /api/runs` with the game's name, by a player signed in with QOR ID, returns a
   run id, a seed (a whole number from 1 to 2³¹−1, from the server's own source of randomness) and the rule version the
   service serves. The run is good for **two hours**. A player has at most one open run per game; asking for another
   abandons the first. Not signed in, or with the service out of reach, the game plays on a seed of its own and says the
   run is practice, not confirmed.

4. **The app sends the run when the game ends,** to `POST /api/runs/{id}`: QQ's run file as recorded. The service refuses
   it, and confirms nothing, if:
   - it is not that player's open run, has expired, or was sent before;
   - its seed is not the one issued, or its scene is not the game's;
   - it is larger than **1 MiB**, or longer than **an hour** (216,000 ticks). Ticks count while the game is paused, and
     a paused tick costs a replay next to nothing, so the limit is the same for every game and generous; the rules end a
     run where they end it (Void Runner after 2,700 steps, Rift Survivor after 5,400);
   - it names an action or a button the game does not have;
   - the service no longer serves the version it issued the run under.

5. **The service replays it with the rules of the version it issued, and confirms only a run that ended.** It runs
   the game's `replay` on the file. If the outcome says the game is over, the run is confirmed: its outcome is stored
   against the run id, once, and shown to the player. Otherwise it is refused as unfinished. The run id is the outcome id
   a payout names (ADR-070 decision 7), so even a payout retried after a crash pays once.

6. **What a replay does not prove.** It proves the inputs reach the outcome under the rules. It does not prove a person
   pressed them: a script could play Orbital perfectly. What bounds that is the pool's policy, which the chain enforces
   (the most one player receives per epoch, the budget per epoch), and goals a run must take time to reach (Void Runner
   and Rift Survivor pay nothing before their end). Telling a script from a person is not attempted here; it waits until
   rewards carry value (ADR-102 decision 7).

7. **Nothing here moves CGT.** Paying for a confirmed run is P7.22's second half, after the pool exists (P7.21) and its
   policy is set. Until then a confirmed run is stored and shown, and nothing is paid.

## Consequences

- **What is built, once accepted:**
  - A table of runs (migration `0004`): the id, the player, the game, the rule version, the seed, when it was issued
    and sent, its ticks and its outcome, and whether it is open, confirmed, refused or abandoned.
  - The two routes, each importing the games' own `rules.mjs`.
  - In the Qt app, asking for a seed on Play, starting a new run on Play again, and sending the run at its end (through
    the session the app already holds, `src/arqadeserver.h`).
  - Tests: Node against PGlite, a stand-in server for the Qt app, and the games suite extended.
- **What changes for a player:** a game asks for its seed as Play is pressed, so the service must be reachable to play a
  confirmed run. Practice is always possible.
- **What stays the owner's:** the pool's numbers (P7.21), whether a reward may be paid at all (U-16), and accepting the
  limits above: two hours, 1 MiB, an hour of ticks.
- **Open after this:** telling scripts from people, and keeping more than one version's rules on the service, so a run
  issued just before a deploy can still be confirmed after it.
