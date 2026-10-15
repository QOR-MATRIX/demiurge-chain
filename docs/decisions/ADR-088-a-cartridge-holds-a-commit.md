# ADR-088: A Cartridge holds the commit a game was published from, and QQ Studio publishes through the launcher

**Status:** **Accepted**, 10 October 2026, by the project owner, who chose both recommendations put to them before P3.4
was built: "The commit (Recommended)" for what a Cartridge holds, and "Approve (Recommended)" for the names below
(AGENTS.md §8). **Amends** [ADR-071](ADR-071-arqade-self-publishing.md) decision 2 on what a Cartridge's content is;
its other decisions, and [ADR-070](ADR-070-game-vaults.md) decision 4, stand.

**Relates to:** [ADR-046](ADR-046-the-launcher-is-an-app-host.md) (the launcher hosts products; this record is its
first product), [ADR-047](ADR-047-the-object-model.md) (the manifest), [ADR-082](ADR-082-qq-architecture.md) decision 5
and [ADR-083](ADR-083-qq-on-qt.md) decision 5 (a game is minted as a Cartridge, asked of the launcher's host),
[ADR-087](ADR-087-strangers-games-are-confined.md) (how a published game is played).

## Context

DIRECTION P3.4 publishes a QQ game: "A Publish action commits the scene through Qontrol and mints that commit as a
Cartridge (a DRC-369 asset), signed by the launcher's vault behind the host dialog, asked over the local channel the
launcher opens for the Studio it started; the Studio never holds a key." Two accepted records describe the Cartridge's
content differently. ADR-070 decision 4: publishing "mints its build as a DRC-369 asset — the Cartridge — pinning the
commit it was built from", and "a revised build keeps the same Cartridge". ADR-071 decision 2: "the content is the
project's profile manifest", revisable until a release is made permanent. What the launcher mints today is a third
thing, which reconciles them: `qontrol_mint` refuses a project with uncommitted changes and mints the ADR-047 manifest
of the whole tree at HEAD, with the commit in `Drc369::mint`'s `commit` field and `revisable` set
(`tools/qor-launcher/src-tauri/src/qontrol/publish.rs`, `chain/assets.rs`). A QQ game's build is its project's files:
the Player plays scenes, logic and assets as they are (ADR-083), so the tree at a commit is the build.

Nothing on the chain is missing: `Drc369::mint` carries a content reference, the commit and a name, and
`Drc369::revise` replaces the content and commit of a revisable asset. What is missing is the launcher's side: no
channel to the Studio it starts (it spawns the Studio and drops the handle), no typed request from a product, a host
dialog that does not name who is asking, and no revise path.

## Decision

1. **A Cartridge's content is the ADR-047 manifest of the project's whole tree at the commit published, and its
   `commit` field is that commit.** Scenes, logic, models, sounds and, once a project carries one, ARQADE's profile
   (`arqade.project/1`, P3.5) are all in the tree, so the one asset pins the build (ADR-070) and the store page
   (ADR-071) together. A project with uncommitted changes is committed first, or not published.
2. **One game, one Cartridge.** The first Publish mints it, revisable. Each later Publish of the same project revises
   that Cartridge to the new commit (`Drc369::revise`), so its history is the game's published history. Making a
   release permanent is the holder's separate act (`Drc369::make_permanent`), not part of Publish.
3. **QQ Studio publishes through the launcher that started it, under ADR-046**, built for this first product as that
   record says: the launcher is the server, on a named pipe (a Unix socket elsewhere) only the current user can open;
   the Studio proves it is the process the launcher spawned with a one-time token on its stdin and the operating
   system's peer credentials; frames are a length prefix and JSON; a handshake agrees a version and capabilities; the
   Studio exits when the channel closes; and it asks for **typed intents** (commit this project; mint or revise this
   commit as a Cartridge), never bytes to sign. The launcher builds every transaction itself and its own dialog says
   which product is asking. A Studio not started by the launcher can save scenes but cannot publish.
4. **Names** (approved by the owner, internal only; nothing here is in runtime metadata or published): the launcher's
   product host is the module `host` (`tools/qor-launcher/src-tauri/src/host/`); the operating-system checks that need
   unsafe code are the crate `qor-host-proof` (`tools/qor-launcher/host-proof/`), since the launcher forbids unsafe
   code; the protocol is the **QOR host protocol**, version 1, its schema in `tools/qor-launcher/host-protocol/`; the
   Studio's side is `LauncherLink` (`products/qq/studio/launcherlink.*`).

## Consequences

- Publishing holds the deposits `pallet-nfts` and `pallet-drc369` reserve, still placeholders (U-14): on a first
  Publish a collection, an asset and a name. Nothing is decided about their size here.
- Q-19 (whether the commit and branch also belong inside the manifest) stays open; the commit is on the chain either
  way.
- The publishing rules a game must meet before ARQADE lists it (ADR-071 decision 3's stages) are P3.5's and ARQADE's,
  not Publish's. Publish puts a game on the chain; it does not list it.
- ADR-046 is built for one product on the platform it runs on, Windows, with the portable rule (exit when the channel
  closes) and the Unix socket's checks in the same abstraction. Code-signature checks at install (ADR-046 decision 6)
  wait on L6.1, as that record says.
