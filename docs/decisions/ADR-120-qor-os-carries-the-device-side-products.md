# ADR-120: QOR OS carries the device-side products

**Status:** **Accepted**, 26 October 2026, by the owner, who asked for QOR OS to be formed from QTOS and this
repository's device-side work and for the plan recorded here to be carried out ("Together this QTOS and my other
project, will form 'QOR OS'"; then, of the plan, "Go with your best and most recommended decisions"). It decides **no
economic value**, and changes nothing in the network or in this repository's scope.

**Relates to:** [ADR-069](ADR-069-arqade-the-gaming-platform.md) (ARQADE in scope),
[ADR-083](ADR-083-qq-on-qt.md) (QQ on Qt 6), [ADR-094](ADR-094-the-library-and-qor-media.md) (the Library and QOR
Media), [ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md) (ARQADE's Qt app),
[ADR-103](ADR-103-the-launcher-moves-to-qt-in-stages.md) (the launcher moves to Qt in stages),
[ADR-104](ADR-104-the-launcher-on-android-on-qt.md) (the Rust host becomes a core library that alone holds a key), and
[ADR-105](ADR-105-qq-makes-real-games-in-3d-and-2d.md) (QQ makes real games).

## Context

The owner has a second project, **QTOS**: an operating system built entirely on Qt 6.12, in the private repository
`QOR-MATRIX/QTOS`. It has a Wayland compositor and shell (Prism), a design system, system services over Qt Remote
Objects, controller input, a signed store and CI with a licensed Qt. On 26 October 2026 the owner decided to merge it
with this repository's work into one operating system for players, creators and developers, **QOR OS**.

Four things in this repository run on a person's device and are written in Qt or link to Qt:

- **QQ**, the engine, its Studio and its Player (`products/qq/`, ADR-083, ADR-105);
- **QOR Media** (`tools/qor-media/`, ADR-094);
- **ARQADE's Qt app** (`products/arqade/qt/`, ADR-102);
- **the launcher's core** (`tools/qor-launcher/core/`, ADR-104). It holds the keys and talks to the chain, QOR ID,
  Social, the Library and the Mesh. DIRECTION L13.2 finishes moving it out of the Tauri crate in the same change as
  this record.

Everything else here is the network, or a client of it: the chain, QOR ID, Social, ARQADE's website, the homepage,
and the Windows launcher, which ships today.

## Decision

1. **QOR OS is the owner's operating system, built in the private repository `QOR-MATRIX/qor-os`.** It starts as a
   mirror of QTOS with its full history. QTOS stays as it is, as the base QOR OS grew from.
2. **QQ, QOR Media and ARQADE's Qt app are carried in QOR OS as `git subtree`s, with their history.**
   - This repository stays their upstream until a later record moves them.
   - A fix that is not specific to QOR OS lands here first, under this repository's rules and CI, and QOR OS takes it
     with `git subtree pull`.
   - Work that is QOR OS's alone lives in QOR OS: building and testing on Linux, running as clients of its compositor,
     its theme tokens and its controller service.
   - The Windows launcher keeps building and shipping these products from here as it does today.
3. **QOR OS uses the launcher's core at a pinned commit, never a copy.**
   - QOR OS builds `tools/qor-launcher/core` from this repository at a commit it names, through Corrosion, as the
     Android shell builds it.
   - The code that handles keys therefore has one source, here, under ADR-104's rule that only the core holds a key.
   - QOR OS builds it with the `desktop` feature, as the Windows launcher does (DIRECTION L13.2).
   - A change QOR OS needs in the core is made here, and QOR OS moves its pin.
4. **The network stays here, public, under these rules.** QOR OS talks to it as services, as the Windows launcher
   does: the chain, QOR ID, Social, ARQADE's website and the homepage. Nothing in QOR OS becomes a second chain, a
   second identity service or a second economic model.
5. **The licence travels with the files.** The carried directories have no licence file of their own. This
   repository's `LICENSE` (Apache-2.0) covers them, so QOR OS keeps a copy of it with them and says where they came
   from.

## Consequences

- This repository's scope (AGENTS.md section 4) is unchanged. `products/qq/`, `tools/qor-media/` and
  `products/arqade/` stay active here, with their gates, suites and CI.
- A change here reaches QOR OS only when QOR OS pulls it, and a change made in QOR OS reaches here only by a pull
  request here. Both directions are deliberate acts, never a sync that runs by itself.
- The core has a third user, after the Windows and Android launchers. Its modules from L13.2 are behind features that
  the Android shell leaves off, so the phone's build is as it was.
- A record that moves QQ's, QOR Media's or ARQADE's development out of this repository supersedes decision 2. It also
  says what becomes of their gates here.
- The crates `qor-launcher-core` and `qor-launcher` say `license = "MIT"` in their manifests, while the repository's
  `LICENSE` is Apache-2.0. Both permit QOR OS's use. Which of the two governs the crates is a question for the owner,
  noted here and not decided.
