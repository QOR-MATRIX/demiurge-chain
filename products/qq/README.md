# QQ, the QOR Engine

An engine for virtual worlds and experiences, built on **Qt 6** under the owner's commercial licence
([ADR-083](../../docs/decisions/ADR-083-qq-on-qt.md)). The blueprint is [`docs/blueprints/qq.md`](../../docs/blueprints/qq.md);
the roadmap items are DIRECTION P3.1 to P3.6.

**Status, 9 October 2026: the first piece of P3.1.** QQ Studio opens a 3D world and shows a glTF 2.0 model loaded at
runtime, lit physically (image-based light from a procedural sky, a sun with soft shadows), with filmic tonemapping,
bloom, ambient occlusion and multisampled edges; the viewport orbits and zooms, and the sun, the exposure and the bloom
are set from the panel. Not yet: the scene tree, the inspector, gizmos, scenes saved as QML in a Qontrol project,
`.qq.json` import, and **Open in QQ Studio** from the launcher — the rest of P3.1.

## What is here

| Path | What it is |
| --- | --- |
| `runtime/` | The QML module `QQ`: `World` (the viewport, its sky, light and look), `Sun`, `Ground`, `Prop` (a glTF model loaded at runtime, reporting `Ready` or `Failed`) |
| `studio/` | QQ Studio, the native editor. `qq-studio [model.gltf] [--capture frame.png]` |
| `tests/` | `tst_world`: renders real frames on the GPU and judges their pixels |
| `tests/fixtures/` | `orb.gltf`, written by `make_orb.py`: an orange sphere with an emissive cyan ring |
| `build.ps1` | Builds with the owner's Qt, MSVC 2022 and Ninja, and runs the tests |

## Build and test

Needs Qt 6.12 (MSVC 2022 64-bit kit, with Qt Quick 3D) and Visual Studio Build Tools 2022 with the C++ workload.

```powershell
pwsh products/qq/build.ps1          # configure, build, run every test; non-zero if anything fails
pwsh products/qq/build.ps1 -Run     # build, then open QQ Studio
```

The build goes to `%LOCALAPPDATA%\qq-build` by default (`-Build` to change it). The tests open real windows and
render on the GPU, so they need a desktop session. `QQ_SAVE_FRAMES=<folder>` keeps every frame a test judges.

## The tests, and how they were proven

`tst_world` builds small worlds from the `QQ` module, renders them, grabs the frame and judges the pixels:

- **a glTF model loads at runtime and is drawn lit in its own colour:** the sphere is orange, the sky is drawn, and the
  sphere's sunlit upper left is brighter than its shaded lower right; no QML warning is raised;
- **bloom lights pixels beyond the glowing band:** the same world rendered with bloom on and off, compared just
  outside the ring's ends;
- **a model that cannot be read is an error, not a crash:** `Prop` reports `Failed` with a reason, and the world
  still draws.

Proven to fail on 9 October 2026 by two planted faults, one at a time: bloom ignored by `World` failed the bloom case;
a `Prop` that never handed its source to the loader failed all three. Learned the same day: restoring a file with
`Copy-Item` keeps its old time, so Ninja does not rebuild it — touch it, or the next run tests the planted fault.

## Not in CI yet

GitHub Actions has no licensed Qt. ADR-083: until the owner adds Qt's installer and account to the repository's
secrets, or a self-hosted runner, QQ is built and tested on the owner's machine and its runs are recorded in
`HANDOFF.md`. The `qor_engine` suite in `docs/GATES.toml` is still the TypeScript preview's check until this replaces
it (ADR-083 decision 6).
