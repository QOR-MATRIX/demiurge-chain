# ADR-126: QQ, QOR Media and ARQADE's Qt app and games are developed in QOR OS

**Status:** **Accepted**, 29 October 2026, by the owner. The owner asked to "move forward with their development
transition to this repo entirely" (QOR OS's repository), and was asked three questions. For each, the owner chose the
recommended answer:

- the products move to QOR OS, while the core and both launchers stay here;
- new work on QQ is closed until QOR OS's public SDK decides whether its source is published;
- the Windows launcher ships the copies frozen here.

It decides **no economic value**, and loosens no evidence. **Supersedes decision 2 of
[ADR-120](ADR-120-qor-os-carries-the-device-side-products.md)** for `products/qq/`, `tools/qor-media/`,
`products/arqade/qt/` and `products/arqade/games/`. QOR OS records the same cut-over as its QOR-0015.

## Context

ADR-120 (26 October 2026) had QOR OS, the owner's operating system in the private `QOR-MATRIX/qor-os`, carry these four
paths as `git subtree`s, and kept this repository their upstream "until a later record moves them". It asked that
record to say what becomes of their gates here. On 29 October 2026:

- **Nothing changed here.** No commit touched those paths, `products/arqade/tests/fixtures/` or
  `docs/design/tokens.json` in the 23 commits from `28d9e97`, where QOR OS carried them, to `db98ac0`.
- **QOR OS made every change.** Each change to the products since was made there:
  - building and testing them on Linux;
  - the agent's key kept in Linux's Secret Service;
  - their windows placed by QOR OS's compositor.
- **Only QOR OS's CI can test them.** CI here has no licensed Qt, so their Qt Test suites run here only on the owner's
  PC (GATES.toml says so of QQ, QOR Media and ARQADE's app). QOR OS's CI builds and tests them with the licensed kit.
- **The Windows launcher ships them.** The QOR Launcher for Windows builds and ships them from these paths
  (`tools/qor-launcher/scripts/stage-arqade.mjs`, `stage-qor-media.mjs`, `src-tauri/src/qq.rs`).
- **ARQADE's rules are shared.** Five modules in the games hold ARQADE's rules and nothing else, importing nothing:
  - `logic/rules.mjs` of Orbital, Void Runner, Synapse and Rift Survivor;
  - the arenas' `logic/arena.mjs`.

  ARQADE's Node tests (`products/arqade/tests/`, in CI's `arqade` job) import them and replay the runs in
  `products/arqade/tests/fixtures/`, and ARQADE's service is to confirm runs by them (ADR-102 decision 5; ADR-116,
  proposed).

## Decision

1. **QQ, QOR Media, ARQADE's Qt app and its games are developed in QOR OS from 29 October 2026.**
   - Their copies here are frozen as they are at `db98ac0`: `products/qq/`, `tools/qor-media/`, `products/arqade/qt/`
     and `products/arqade/games/`.
   - They take no new work here, beyond the note this record puts at the top of each one's README.
   - This supersedes ADR-120 decision 2 for them.
2. **The QOR Launcher for Windows keeps building and shipping the frozen copies**, the same code as its 0.1.17.
   - Only the owner releases a fix to one of them for the launcher's users, by a pull request here that names the QOR
     OS change it comes from.
   - That fix is then Apache-2.0, like the rest of this repository.
3. **ARQADE's rules and recorded runs stay shared**, because ARQADE's server and its Node tests here judge runs by the
   same files that the games in QOR OS play by.
   - **The five rule modules** are kept identical in both repositories and change in both at once. A change arrives
     here as a pull request, with the fixtures it changes and the digests the Node tests pin.
   - **`products/arqade/tests/fixtures/`** stays this repository's, and QOR OS still pulls it.
   - **A rule change published here** also reaches the launcher's frozen games at the launcher's next release, so it
     has to suit them.
4. **The rest stays here.**
   - The QOR core stays here, the only source of the code that handles keys (ADR-120 decision 3, unchanged).
   - So do both launchers and the network (ADR-120 decision 4).
   - `products/arqade/` stays active for the website, its server's code, its Node tests, its SDK and its fixtures.
5. **The gates stay as they are, and count what is true here.**
   - **Their tests stay.** `qor-engine`, `library` and `arqade` measure their tests through the launcher's, the core's
     and ARQADE's own suites, which stay here and stay required.
   - **Items the launcher's frozen copies can still prove stay here**, and can still be ticked: P3.4 (publishing from
     QQ Studio started by the launcher), L9.1 to L9.5 (the owner's first look) and P7.19.
   - **Items that need new work on the products move to QOR OS's roadmap**, marked so in DIRECTION: P3.3's proof,
     P3.5, QQ's side of P3.6, L9.7 and L9.8. The chain's work P3.6 depends on stays in M4 to M8.
   - **The `qor-engine` gate is therefore not met here.** It counts P3.3, P3.5 and P3.6, so its criteria stay as they
     are: that is the truth while the engine's work is in QOR OS. A later record may re-scope it.

## Consequences

- **Scope.** AGENTS.md section 4 and DIRECTION section 8 list the four paths as frozen here by this record.
  SYSTEMS.md's rows keep their status words, because **frozen** there means pre-realignment code (D-011).
- **CI.** The security job's language scan still reads `products/arqade/qt` and `products/arqade/games`, and the
  `arqade` job's design check still reads ARQADE's Qt QML. Both pass on the frozen copies, and neither is loosened.
- **Licence in QOR OS.** QOR OS lists the files that came from here, which stay Apache-2.0. Its new work on them is its
  own, except the rule modules, which it keeps Apache-2.0 so that they can stay identical here.
- **A sign this is wrong:** the launcher needs fixes to the products often. Then a later record has it take QOR OS's
  builds, or moves it.
