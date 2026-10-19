# ADR-103: The launcher's screens move to Qt in stages, over the Rust host that keeps the keys

**Status:** **Accepted in direction**, 18 October 2026, by the owner. **Decision 2's bridge is superseded by
[ADR-104](ADR-104-the-launcher-on-android-on-qt.md)** (18 October 2026), which starts stage 2 on Android through a small C
bridge of the core's own, since CXX-Qt does not support Android; the rest of this record stands. The owner asked to "consider using QT for all aspects of
the QOR experience with the most graphically cutting edge options possible" and chose the Arcade in Qt now, with the
launcher's own screens following in stages, each stage its own decision. **Amends** the standing position that the
launcher stays Tauri unless the owner decides (it is now decided in direction). **Relates to** ADR-046 (the product host),
ADR-083 (QQ on Qt), ADR-094 (QOR Media on Qt) and ADR-102 (ARQADE on Qt). It decides **no economic value**.

## Context

The launcher is a Tauri 2 application: a Rust host (`src-tauri/`) that alone holds the vault's keys and signs, behind
dialogs it draws itself, and a React interface in a webview that can ask but never read a key. Around that boundary sit
302 host tests and fourteen browser checks. QQ, QOR Media and now ARQADE are Qt 6. The owner wants the whole QOR
experience on Qt, at the edge of what it can draw.

## Decision

1. **The Rust host stays, and stays the only thing that holds a key.** The vault, QOR ID's client, the chain client,
   the Mesh, the profiles of ADR-099 and the host dialogs are Rust, tested as they are. What moves to Qt is the
   interface over them.
2. **The move is in stages, and each stage is its own decision, built and checked before the next:**
   - **Stage 1, now:** new surfaces are Qt applications docked inside the launcher under the product host: QQ Studio,
     QOR Media and ARQADE.
   - **Stage 2:** a Qt shell (Qt Quick, Qt Quick 3D for the living background) that links the Rust host as a library
     through a typed bridge (CXX-Qt), so the host and its tests are unchanged and the shell can never reach a key.
   - **Stage 3 and on:** each React surface is rebuilt in the Qt shell, one at a time (Vault, Inventory, Market, Social,
     Library, Mesh, Settings), each with its own checks before the React one is retired.
3. **The boundary is the test.** At every stage, the interface can ask the host to sign and can never read a key, and a
   check proves it, as `check-vault-gate.mjs` does today.

## Consequences

- Stage 2 needs its own record before it starts: the bridge, the build on Windows, macOS and Linux, and how the
  fourteen browser checks are carried over.
- Until a surface is rebuilt, its React version stays the one people use.
