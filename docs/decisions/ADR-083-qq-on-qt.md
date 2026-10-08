# ADR-083: QQ is built on Qt 6, as a native studio and player, with an agent that can design and build

**Status:** **Accepted**, 8 October 2026, by the project owner: "Utilize QT framework because i have a licence to build
with it", "Id like for us to at least have our own game engine that is capable of creating virtual worlds and
experiences" and "Autonomous game design and dev seems like a good addition". Asked how Qt reaches the machine, the owner
chose to install **commercial Qt** with their own account; asked about the TypeScript engine built for P3.1, the owner
chose to **keep it until the Qt build plays** in the launcher and in ARQADE's browser. It **supersedes ADR-082 in part**
(decisions 1, 3, 6 and 7: a TypeScript/WebGL2 engine and a React editor inside the launcher's webview, at
`tools/qor-launcher/src/qq/`); ADR-082's decisions 2, 4 and 5 stand (below). It decides no economic value.

## Context

ADR-082 put QQ in the launcher's webview as a TypeScript/WebGL2 engine, and its first slice was built and checked on
8 October 2026 (HANDOFF §4 item 61). It draws 2D shapes and particles well. It is not a path to what the owner now asks
for: **virtual worlds** — 3D with physically based materials, imported models, physics, spatial audio and, later, VR —
and an editor and runtime that an AI can drive to **design and build a game on its own**. Writing those on raw WebGL2
is years of renderer work. The owner holds a commercial Qt licence.

Qt 6 supplies what the target needs and nothing has to be written twice:

- **Qt Quick 3D**: a physically based forward renderer with image-based lighting, shadows, baked lightmaps,
  instancing, glTF 2.0 loaded at runtime (`RuntimeLoader`), 3D particles, and a post-processing environment
  (`ExtendedSceneEnvironment`: tonemapping, bloom, ambient occlusion, depth of field, fog). **Qt Quick 3D Physics**
  (PhysX) for bodies, joints and character control; **Qt Quick 3D XR** later; **Qt Spatial Audio** for sound in space.
- **QML and JavaScript** as the scene and behaviour language: declarative, readable, diffable by Qontrol, reloadable
  while running, and a language large models write well. That is what makes autonomous building practical: an agent
  writes QML, the studio reloads it, plays it, captures the frame, and the agent judges what it sees.
- **Qt for WebAssembly** runs the same player in a browser, which keeps ADR-081's promise that one game runs in the
  launcher and on ARQADE.
- **cxx-qt** binds Rust to Qt, so the launcher's Rust code (Qontrol, the chain client) can be reused rather than
  rewritten, should QQ need it directly.

## Decision

1. **QQ is built on Qt 6** (6.8 LTS or newer), in C++20 and QML, using Qt Quick 3D, Qt Quick 3D Physics, Qt Shader
   Tools and Qt Spatial Audio, under the owner's commercial Qt licence. Its source lives at **`products/qq/`** and is
   built with CMake. Nothing in it is written against Qt modules that the owner's licence does not cover.
2. **Three parts, one runtime.** The **QQ Runtime** is a QML module (`QQ`) and its C++ backing: worlds, entities,
   bodies, emitters, lights, models, input, audio and behaviours. **QQ Studio** is the native editor: a 3D viewport
   with gizmos, the scene tree, an inspector, assets, Play and Stop, and the agent. The **QQ Player** runs a published
   game: natively when the launcher starts it, and compiled to WebAssembly inside ARQADE's browser. Studio and Player
   both run on the Runtime, so what a creator sees while editing is what a player gets.
3. **A QQ scene is QML** (`scenes/<name>.qml` in a Qontrol project), written by the Studio in one canonical layout:
   readable, one property per line, so a change is a small, true diff in Projects. `.qq.json` scenes of format 1
   (ADR-082's) are imported.
4. **The agent (autonomous design and build).** QQ Studio exposes its editing operations to language models through
   an **MCP server** (read the scene, add and change entities, write behaviours, play, stop, capture a frame, read the
   log) and a design loop that turns a description into a design brief, builds it, plays it, looks at the frames and
   revises. A creator approves what the agent proposes before it is committed. The AI provider is chosen under
   ADR-081's delegation and recorded when the agent is built; an API key is kept in the operating system's keychain
   and never logged.
5. **The launcher stays a Tauri application.** Its QQ surface keeps the TypeScript preview and gains **Open in QQ
   Studio**, which starts the Studio as a process of its own. **The Studio never holds a key:** anything to be signed
   (a Cartridge mint, ADR-082 decision 5) is asked of the launcher's host over a local channel the launcher opens for
   the Studio it started, and approved in the host dialog (L1.4), as every signature is.
6. **The TypeScript engine stays until the Qt build plays** in the launcher and in ARQADE's browser (the owner's
   choice), then it is retired, with its check, in the change that proves the replacement.
7. **ADR-082's decisions 2, 4 and 5 stand:** one game in the launcher and the browser; signing only through the
   launcher's vault and host dialog; a published game is a DRC-369 Cartridge.

## Consequences

- DIRECTION P3's six items are rewritten for the Qt engine (the same day). The first, P3.1, is now the Studio with a
  3D viewport; the TypeScript slice built under ADR-082 no longer meets it.
- `docs/blueprints/qq.md` is rewritten for Qt.
- **Building waits on Qt being installed** on the development machine with the owner's account (an owner step).
  Nothing under `products/qq/` is written before it can be compiled and tested there; untested engine code is not
  merged.
- **CI:** building Qt code on GitHub Actions under a commercial licence needs Qt's installer and the owner's Qt account
  on the runner (as repository secrets) or a self-hosted runner. Until one exists, `products/qq/` is built and tested
  on the owner's machine and its runs recorded in HANDOFF; it is not a CI job, and no gate reads it from CI.
- **Licensing, for the owner to confirm with Qt:** the repository is public under Apache-2.0. The owner's commercial
  licence covers the owner's builds and distribution; anyone else building `products/qq/` needs their own Qt licence
  (commercial, or the GPLv3 under which Qt Quick 3D and its physics are offered). Whether the commercial agreement
  covers builds on CI runners and work done by AI assistants on the licensed machine is a question for the agreement,
  not for this record.
- Qt for WebAssembly builds of Qt Quick 3D are large (tens of megabytes) and use WebGL2; ARQADE shows a loading
  progress and caches the player. Measured when built.
- No money rule changes. The Studio and the Player create, price and charge no CGT.
