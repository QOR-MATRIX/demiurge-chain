# QQ, the QOR Engine

An engine for virtual worlds and experiences, built on **Qt 6** under the owner's commercial licence
([ADR-083](../../docs/decisions/ADR-083-qq-on-qt.md)). The blueprint is [`docs/blueprints/qq.md`](../../docs/blueprints/qq.md);
the roadmap items are DIRECTION P3.1 to P3.6.

**Status, 9 October 2026: P3.1 done.** QQ Studio edits a 3D world and keeps it in a Qontrol project:

- **The world:** a procedural sky that also lights the scene, a sun with soft shadows, lamps, filmic tonemapping, bloom,
  ambient occlusion and multisampled edges; shapes with physically based surfaces; glTF 2.0 models loaded at runtime.
- **The Studio:** the scene's entities in a list, the selected one's fields in an inspector (built from what each type
  declares), click to select in the viewport, handles to move along an axis, turn about the vertical or scale evenly,
  orbit and zoom,
  add (cube, sphere, cylinder, cone, lamp, model) and delete.
- **The files:** a scene is `scenes/<name>.qml` in the project, in one canonical layout: read and written back it is
  the same bytes, and one changed value is one changed line in Projects. A model from outside the project is copied
  into its `assets/` first, so the project is whole on its own. 2D scenes from the launcher's preview (`.qq.json`)
  are imported, with what has no 3D form yet (motion, the pointer follow, emitters: P3.2) named.
- **The launcher:** **Open in QQ Studio** on the launcher's QQ surface starts the Studio on the project open there.

Next is P3.2: worlds that play (physics, input, particles, spatial audio, live behaviours) and the QQ Player.

## What is here

| Path | What it is |
| --- | --- |
| `runtime/` | The QML module `QQ`: `World` (the viewport, its sky, light and look), `Scene` (the root a file holds), the entities `Sun`, `Lamp`, `Ground`, `Shape`, `Prop`, and `SceneIO` (C++): canonical writing, loading, saving, adopting models, importing 2D scenes, and the editing operations the Studio uses and the agent will (P3.3) |
| `studio/` | QQ Studio. `qq-studio [model.gltf] [--project <folder>] [--capture <frame.png>]` |
| `tests/` | `tst_world` (rendering, judged by pixels), `tst_scene` (scene files), `tst_studio` (the real Studio window, worked by clicks and drags) |
| `tests/fixtures/` | `orb.gltf` (written by `make_orb.py`), `scenes/sample.qml` (every entity type, canonical), `first-light.qq.json` (the 2D preview's starter scene) |
| `build.ps1` | Builds with the owner's Qt, MSVC 2022 and Ninja, runs every test, and with `-Deploy` makes a self-contained Studio |

## Build, test, deploy

Needs Qt 6.12 (MSVC 2022 64-bit kit, with Qt Quick 3D) and Visual Studio Build Tools 2022 with the C++ workload.

```powershell
pwsh products/qq/build.ps1           # configure, build, run every test; non-zero if anything fails
pwsh products/qq/build.ps1 -Run      # build, then open QQ Studio
pwsh products/qq/build.ps1 -Deploy   # build, test, then put a self-contained QQ Studio where the launcher looks
```

The build goes to `%LOCALAPPDATA%\qq-build` (`-Build` to change it); the deployed Studio to `%LOCALAPPDATA%\qq-studio`
(`-DeployTo`), with the Qt libraries, plugins and QML modules it uses and the Microsoft C++ runtime installer: about 1,500
files and 113 MB, runnable on a machine with no Qt. The launcher's **Open in QQ Studio** starts that copy, or the one
the `QQ_STUDIO` environment variable names.

The tests open real windows and render on the GPU, so they need a desktop session. `QQ_SAVE_FRAMES=<folder>` keeps
every frame `tst_world` judges.

## The tests, and how they were proven

- **`tst_world`** (3 cases): a glTF model loads at runtime and is drawn lit in its own colour; bloom lights pixels past
  the glowing band; an unreadable model is an error, not a crash. Proven to fail by two planted faults (bloom ignored;
  a prop that never loads).
- **`tst_scene`** (12 cases): numbers are written one way only; a scene file reads and writes back byte for byte; one
  changed value is one changed line; a saved scene keeps its model relative and loads again; a scene saved elsewhere
  still points at its model (including across drives, where no relative path exists); adopting a model brings its
  files and never overwrites; what is not a scene is refused with a reason; entities are added and removed and the
  file follows; a project lists its scenes; the bundled sample can be adopted; a 2D scene imports as a canonical one;
  a file of another format is not imported.
- **`tst_studio`** (8 cases, the real window): a new Studio opens a lit scene in its project; a click on the model
  selects it and a click on the sky selects nothing; dragging the X handle moves along X only; dragging the centre
  handle scales evenly and leaves the position alone; saving writes a
  canonical scene and carries the model into the project, and one edit saved again is one changed line; a saved scene
  opens again as it was; entities are added, named uniquely and deleted; a 2D scene is imported with what cannot come
  yet said. Proven to fail by two planted faults (a handle that drags along a diagonal; a save that does not copy the
  model into the project).
- In the launcher: `check-qq-view.mjs` checks that **Open in QQ Studio** asks the host to start the Studio on the open
  project and nothing more; the host's `qq::` tests check where the Studio is found, that the project is one argument,
  and that a folder that is not a project is refused; `qq_studio_starts_on_a_project` (ignored by default: it opens a
  window) starts the deployed Studio on a real repository and sees it in the process list.

Learned on the way, and recorded in `HANDOFF.md` §5: Qt Quick 3D's `Material` type shadows the Material style's attached
`Material.*` when it is imported after it; Qt 6 keeps a URL property as written, so relative model URLs are resolved
against the entity's own file; across drives there is no relative path, and a bare `X:/...` is read back as a URL with
scheme `x`.

## Not in CI yet

GitHub Actions has no licensed Qt. ADR-083: until the owner adds Qt's installer and account to the repository's
secrets, or a self-hosted runner, QQ is built and tested on the owner's machine and its runs are recorded in
`HANDOFF.md`. The `qor_engine` suite in `docs/GATES.toml` is still the TypeScript preview's check until this replaces
it (ADR-083 decision 6).
