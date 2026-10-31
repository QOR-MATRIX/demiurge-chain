# ADR-129: Ever Rift's and Darkveil's rules join ARQADE's shared rules

**Status:** **Accepted**, 31 October 2026, by the owner, who named the two games and so sent their rules here, as QOR OS's
QOR-0017 and QOR-0019 say a named game's rules come. The words are the assistant's. It decides **no economic value**.
**Amends [ADR-126](ADR-126-the-device-side-products-are-developed-in-qor-os.md) decision 3**, which names five rule modules:
there are seven. ADR-126's other decisions stand.

## Context

ADR-126 keeps ARQADE's rule modules identical here and in QOR OS, because ARQADE's server and its Node tests here judge runs by
the same files the games play by. QOR OS's QOR-0017 adds seven original games, each rewarded from the start, its rules, Node
test and fixture shared in the same way, and none published until the owner names it.

The owner has named the first two (QOR OS's QOR-0019):

- **Ever Rift**, a dungeon hack-and-slash: three floors carved from the run's seed, down to a warden.
- **Darkveil**, a 3D action RPG: one valley, three wardens over three beacons, and a king behind a gate. Its rules keep the
  world's state apart from each player's, so a server could one day step one valley for many. No such server is decided here.

The games themselves, their scenes, logic and assets, are QOR OS's, in its private repository, and stay there (ADR-126).

These were checked on 31 October 2026 against QOR OS's `main`: both Node tests pass here as they do there, and each fixture is
a run QQ recorded as ARQADE's games suite played the game, which replays in Node to the outcome QQ reached.

## Decision

1. **Seven rule modules are shared** (amends ADR-126 decision 3): the five it names, and
   `products/arqade/games/ever-rift/logic/rules.mjs` and `products/arqade/games/darkveil/logic/rules.mjs`, kept identical in
   both repositories and changed in both at once, with the fixtures they change and the digests the Node tests pin. They are
   under Apache-2.0, like the rest of this repository, and say so in their first line.
2. **Their tests run here.** `products/arqade/tests/ever-rift.test.mjs`, `darkveil.test.mjs` and the pilot it plays with,
   `darkveil-pilot.js`, are part of ARQADE's `npm test`. Their fixtures join `products/arqade/tests/fixtures/`.
3. **Nothing else of them comes here.** `products/arqade/games/ever-rift/` and `darkveil/` hold only their `logic/rules.mjs`.
   Neither game is in ARQADE's website or the Windows launcher, whose frozen copies (ADR-126 decision 2) are unchanged and do
   not read these files.
4. **What a confirmed run earns is not decided here.** Their rules let a run be replayed and confirmed, as the others' do
   (ADR-102, ADR-116). No value, reward or payout is set, and gold and score inside a game are not CGT.

## Consequences

- ARQADE's `npm test` takes about twenty seconds longer: Darkveil's test plays three whole runs by its pilot.
- A change to either game's rules now has to come here too, as a pull request (ADR-126 decision 3).
- **A sign this is wrong:** rules that cannot stay identical, or a test here that cannot pass without QOR OS. Then the module
  stops being shared, by a later record.
