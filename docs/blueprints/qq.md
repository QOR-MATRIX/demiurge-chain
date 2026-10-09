# QQ — the QOR Engine

**Active blueprint, 8 October 2026.** QQ is QOR Engine ([ADR-081](../decisions/ADR-081-qq-is-qor-engine.md)). It is
built on **Qt 6** ([ADR-083](../decisions/ADR-083-qq-on-qt.md)), which superseded the first, browser-only architecture
of [ADR-082](../decisions/ADR-082-qq-architecture.md) in part. Nothing here overrides `AGENTS.md`'s money, language or
key-safety rules. **Status:** P3.1 done on 9 October 2026 on Qt 6.12 (Enterprise), in `products/qq/`: QQ Studio edits a
lit 3D world and keeps it as canonical QML in a Qontrol project, opened from the launcher
([`products/qq/README.md`](../../products/qq/README.md)). Next, P3.2: worlds that play.
A TypeScript/WebGL2 preview from ADR-082 runs in the launcher meanwhile (below).

---

## What it is

QQ is an engine for **virtual worlds and experiences**: a place to build a game that looks extraordinary, plays tight
and opens instantly, in 3D or 2D, alone or **with an agent that can design and build it for you**. A creator works in
**QQ Studio**, plays in the same window, versions the work with Qontrol, and publishes it as a DRC-369 Cartridge. A
player runs it from the QOR Launcher or in a browser on ARQADE, signed in with QOR ID.

The ambition is set by the target, not by the size of the engine: small-scale, effects-led, finished games and
worlds, where one effect executed perfectly beats ten done adequately. Every world QQ produces should be recognisable
as made with it — not as the output of a template.

---

## Why Qt

| Need | What Qt 6 gives QQ |
| --- | --- |
| Worlds that look the part | **Qt Quick 3D**: physically based materials, image-based lighting, real-time shadows, baked lightmaps, instancing, and `ExtendedSceneEnvironment` (tonemapping, bloom, ambient occlusion, depth of field, fog, lens effects) |
| Bring your own models | glTF 2.0 loaded at runtime (`RuntimeLoader`); `balsam` to convert at import |
| Things that move like things | **Qt Quick 3D Physics** (PhysX): bodies, colliders, joints, a character controller |
| Sound in space | **Qt Spatial Audio**: sources, listeners, room acoustics |
| Light and life | 3D particles with emitters, attractors and trails; custom materials and effects in Qt's shader language |
| A language people and models both write | **QML and JavaScript**: declarative, readable, one property per line, reloaded while running |
| One game, launcher and browser | Native builds for the launcher; **Qt for WebAssembly** for ARQADE |
| Later | Qt Quick 3D XR for headsets; Android and macOS builds of the Player |

Writing any one of these rows on raw WebGL2 is a project of its own. Qt has them, and its licence is the owner's.

---

## Architecture

```text
products/qq/
├── runtime/        QQ Runtime — QML module "QQ" + C++ backing
│     World, Entity, Body, Collider, Emitter, Light, Model, Camera rigs,
│     Input (keyboard, mouse, gamepad), Audio, Behaviour (QML/JS), Save state
├── studio/         QQ Studio — the native editor (Qt Quick + C++)
│     Viewport (orbit/fly camera, gizmos, grid, selection outline)
│     Scene tree · Inspector (from QML property metadata) · Assets
│     Play / Stop (a second engine instance; the edited scene is untouched)
│     Qontrol panel · Agent panel · Log
├── player/         QQ Player — runs a published game
│     native (started by the launcher) · WebAssembly (inside ARQADE)
├── agent/          The MCP server and the design loop
└── tests/          Qt Test suites: runtime, studio, scene format, agent tools
```

**One runtime.** The Studio and the Player both run the Runtime, so what a creator sees while editing is what a player
gets. Play in the Studio starts a fresh instance of the scene; Stop discards it and the edited scene is exactly as it
was.

**The launcher stays Tauri.** Its QQ surface shows the preview and an **Open in QQ Studio** button that starts the
Studio as its own process. The Studio never sees a key (below).

---

## The scene

A QQ scene is a QML file in a Qontrol project, `scenes/<name>.qml`, written by the Studio in one canonical layout
(built 9 October 2026; `products/qq/tests/fixtures/scenes/sample.qml` is a whole one):

```qml
import QQ

Scene {
    name: "first-light"
    skyTop: "#0b1430"
    skyHorizon: "#c7623a"
    ...
    bloom: true

    Sun {
        name: "Sun"
        elevation: 38
        azimuth: -35
        color: "#fff1dc"
        brightness: 1.15
    }

    Prop {
        name: "Orb"
        position: Qt.vector3d(0, 1.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/orb.gltf"
    }
}
```

- **Canonical:** every entity declares its `kind` and its `fields`, and `SceneIO` writes them in that order, one per
  line: numbers to four decimal places with trailing zeros dropped, colours as `#rrggbb`, vectors as `Qt.vector3d`,
  model files relative to the scene. The same scene is the same bytes; one changed value is one changed line in
  Projects. A model from outside the project is copied into its `assets/` when the scene is saved.
- **Entities today:** `Sun`, `Lamp`, `Ground`, `Shape` (cube, sphere, cylinder, cone, plane, with a physically based
  surface and glow), `Prop` (a glTF 2.0 model). Bodies, emitters and behaviours arrive with P3.2.
- **Readable by people and models.** Behaviours (P3.2) will be plain QML and JavaScript, reloaded while the world runs.
- **Imported:** `.qq.json` scenes of format 1 from the preview become QML scenes; what has no 3D form yet (motion, the
  pointer follow, emitters) is named, not silently dropped.

---

## The agent: autonomous design and build

QQ Studio exposes its editing operations to language models through an **MCP server**, so any capable model can drive
it, and a **design loop** in the Studio's Agent panel:

1. **Brief.** From a description ("a neon rooftop chase at night, one minute long"), the agent writes a design brief:
   the fantasy, the core loop, the controls, the look, the scope.
2. **Build.** It creates the scene through the tools — `scene.read`, `entity.add`, `entity.set`, `behaviour.write`,
   `asset.import` — never by writing files around the Studio.
3. **Play and look.** It plays the world, captures frames (`play`, `frame.capture`, `log.read`) and judges them
   against the brief: readability, composition, motion, whether the loop is fun.
4. **Revise** until the brief is met or it needs the creator.
5. **The creator approves** what is committed. Nothing reaches Qontrol, and nothing is published, without them.

The model provider is chosen when the agent is built (ADR-081's delegation) and recorded in an ADR. A provider key
lives in the operating system's keychain and never reaches a log. Generative assets (models, textures, sound from a
description) arrive with P3.6.

---

## QOR ID, the vault and publishing

- **The Studio never holds a key.** A Cartridge mint is asked of the launcher's host over a local channel the launcher
  opens for the Studio it started, and approved in the host dialog (L1.4), as every signature is.
- **Publishing** commits the scene through Qontrol and mints that commit as a **Cartridge**, a DRC-369 asset
  (ADR-082 decision 5).
- **Players** sign in with QOR ID: the launcher's session natively, ADR-073's OAuth in the browser. Without it a game
  still plays; collectibles and CGT are simply absent.

---

## ARQADE

ARQADE hosts the WebAssembly Player and a game's bundle (QML, assets). Play loads the Player with the game and shows
progress while it loads; the Player is cached. Size and first-frame time are measured when it is first built
(Qt Quick 3D's WebAssembly build is tens of megabytes; that number is the one to beat).

---

## Licensing

QQ uses Qt under the owner's **commercial licence** (ADR-083). The repository is public under Apache-2.0: anyone else
building `products/qq/` needs their own Qt licence, commercial or GPLv3 (under which Qt Quick 3D and its physics are
offered). Whether the owner's agreement covers CI runners and AI assistants working on the licensed machine is a
question for the agreement, recorded in ADR-083's consequences.

---

## The preview that runs today

Built under ADR-082 on 8 October 2026 and kept until the Qt Player plays in the launcher and ARQADE (ADR-083
decision 6): `tools/qor-launcher/src/qq/`, a WebGL2 2D renderer (shapes as signed-distance fields with glow, particles
drawn additively), a deterministic world (motion, a pointer follow, emitters) and an editor surface on the rail that
saves `scenes/<name>.qq.json` through the host into a Qontrol project. It is checked by `check-qq-view.mjs` (41
checks, proven to fail by two planted faults) and the host's `qq::` tests. Its scene format is what the Qt Studio
imports.

---

## Phases

The roadmap items are DIRECTION P3.1 to P3.6; in short:

1. **QQ Studio exists** — 3D viewport, a glTF model, scene tree, inspector, gizmos, canonical QML scenes in Qontrol,
   started from the launcher.
2. **Worlds play** — physics, input, particles, spatial audio, live behaviours; the Player native and in a browser.
3. **The agent designs and builds** — MCP server and the design loop, proven on a scene built from a description.
4. **Publishing** — Qontrol commit and a Cartridge minted through the launcher's vault.
5. **ARQADE delivery** — the WebAssembly Player in ARQADE; the preview retired.
6. **Collectibles, generation and the Mesh** — DRC-369 items in games, generative assets, Mesh delivery, settlement
   in CGT.

---

## What QQ is not

- **Not a AAA engine.** No film-grade simulation, no open worlds the size of a country. Worlds a small team — or one
  person and an agent — can finish.
- **Not a Unity or Unreal importer.** Assets move (glTF 2.0). Projects do not.
- **Not a second identity system or payment rail.** QOR ID is the only identity; CGT the only currency; the engine
  never calls a payment API and never holds a key.
