# QQ — the QOR Engine

**Active blueprint, 7 October 2026.** This replaces the withdrawn Godot plan (`docs/blueprints/qor-engine.md`). The
architectural decisions are in [ADR-082](../decisions/ADR-082-qq-architecture.md), accepted under the delegation of
[ADR-081](../decisions/ADR-081-qq-is-qor-engine.md). Nothing here overrides AGENTS.md's money, language or
key-safety rules.

---

## What it is

QQ is a game engine and editor built directly into the QOR Launcher's webview. A creator authors a scene in a split
editor surface — viewport on one side, component inspector on the other — commits through Qontrol, and publishes as
a DRC-369 Cartridge. The same game bundle runs in the launcher and in ARQADE's browser without a second window,
without a separate process and without a second runtime.

The target is **small-scale, effects-led, interactive experiences**: games that look extraordinary, play tight and
load instantly. Think: a particle-physics puzzle, a procedurally-lit dungeon crawler, a reactive music visualiser
with a score. Not AAA polygon counts. Not a substitute for Unreal.

The design philosophy is **radical quality over radical scope**. One effect executed perfectly beats ten effects done
mediocrely. Every screen QQ produces should be immediately recognisable as Demiurge's — not as the output of a
template.

---

## Why a from-scratch WebGL2 engine

The Godot plan (withdrawn by ADR-081) required a sibling OS window, a C++ build tracking an upstream that changes
fundamentals each minor release, and a Tauri IPC seam between the engine process and the vault. A WebGL2 engine in
the launcher's webview eliminates all three costs:

| Requirement | Godot plan | QQ |
|---|---|---|
| Runs inside the launcher window | ✗ (sibling OS window) | ✓ (same webview) |
| Runs in ARQADE's browser | ✗ (would need a port) | ✓ (same bundle) |
| Vault access from the editor | IPC across a process boundary | the launcher's own host commands (`src/lib/ipc.ts`), same process |
| Reduced motion / contrast obligations | Requires a custom Godot module | CSS `prefers-reduced-motion` + WebGL2 clear colour |
| Upstream rebase cost per minor release | One per Godot 4.x minor | Zero (we own the code) |

The cost paid is writing a renderer. That cost is paid once, and the resulting renderer is exactly as small and fast
as the target scope requires — not a 200 MB runtime that runs Jolt physics for a tile puzzle.

---

## Architecture

```
QOR Launcher (Tauri 2 / Rust host)
│
├── WebView2 webview  ─────────────────────────────────────────
│   │                                                          │
│   ├── QQ Editor (React surface, tools/qor-launcher/src/qq/) │
│   │   ├── Viewport (WebGL2 canvas, owned by QQ runtime)     │
│   │   ├── SceneTree inspector                               │
│   │   ├── Component inspector                               │
│   │   ├── Asset browser (reads launcher's Inventory)        │
│   │   └── Qontrol panel (commit, branch, history)           │
│   │                                                          │
│   └── QQ Runtime (TypeScript/WebGL2 modules)                │
│       ├── Renderer (WebGL2; WebGPU optional per platform)   │
│       ├── Scene graph (entity–component, no ECS overhead)   │
│       ├── Input (pointer, keyboard, gamepad)                │
│       ├── Audio (Web Audio API)                             │
│       ├── Physics (2D: AABB + circles; 3D: simple AABB)     │
│       └── QOR bridge (sign requests → vault via host command)│
│                                                              │
└── Rust host ─────────────────────────────────────────────────
    ├── Vault (signs; never reachable from engine directly)
    ├── subxt client (chain calls)
    └── Qontrol sidecar (libgit2, never in the host)

Published game bundle (standalone JS + WebGL2)
└── Deployed to ARQADE via Vercel — same bundle, no recompile
```

**One canvas, not two.** QQ's viewport canvas lives inside the editor surface, not behind the chrome like the QFX
backdrop. When a published game runs in ARQADE, the QFX canvas is not present. There is no Z-fighting.

---

## The renderer

**Layer 1 (shipped first):** A 2D sprite and shape renderer.
- Batched quad rendering (sprites, rectangles, circles drawn as SDF quads).
- A signed-distance-field text renderer for in-game UI.
- A particle system with per-particle position, velocity, lifetime and colour. **Built on the CPU** (P3.1): one flat
  array of at most 4,096 particles, which costs well under a millisecond a frame; a GPU ping-pong through float
  textures is for when a scene needs more.
- Pointer-reactive shader uniforms passed per-frame (same pattern as the QFX backdrop).
- Reduce motion: Play draws one frame and holds it; nothing animates until the creator presses Step, which advances
  the world a quarter of a second. Asked at the moment Play is pressed, so a change of the system setting counts.

**Layer 2 (after P3.1):** A 3D renderer, forward pass, no GI.
- Perspective projection, free-look camera.
- glTF 2.0 import (positions, normals, UVs, joints, weights, morph targets).
- PBR shading: albedo, metallic, roughness, AO, emissive.
- Up to 8 point lights + 1 directional, all realtime.
- Bloom post-process (dual-pass Kawase blur, the cheapest that looks correct).
- No shadow maps in the first 3D slice; they arrive with the first game that needs them.

**Upgrade path to WebGPU:** The renderer is written against an abstract `GpuDevice` interface.  The WebGL2
implementation ships first; a WebGPU shim that adapts the same interface replaces it per platform once the Tauri
webview's WebGPU support is confirmed on the owner's hardware. No game code changes.

---

## The editor

The QQ editor is a launcher surface — a new entry on the rail alongside Projects, Inventory and Market. It opens
the QQ runtime in "edit mode" inside a split layout:

```
┌─────────────────────────────────┬──────────────────────────────┐
│  Viewport (WebGL2, edit mode)   │  Inspector                   │
│                                 │  ├── Scene tree               │
│  [ ▶ Play ] [ ■ Stop ]          │  ├── Selected entity props    │
│                                 │  ├── Material editor          │
│  Drag entities here             │  └── Qontrol status           │
└─────────────────────────────────┴──────────────────────────────┘
│  Asset browser (launcher Inventory + local project files)       │
└─────────────────────────────────────────────────────────────────
```

Play mode runs the game bundle inside the same viewport — press Stop, and the editor state is restored. A creator
never needs to leave the launcher to play-test.

**Entity–Component system (simple, not ECS):** Each entity is a plain JavaScript object `{ id, components: Map }`.
Components are records with typed fields. No archetype arrays, no bitset queries — the target scene size is hundreds
of entities, not millions.

**Scene format:** A QQ scene is a `.qq.json` file. It is plain JSON: an array of entity records, each with a
component list and a name. It is diffable, mergeable with Qontrol's semantic tools, and hashable by BLAKE3 for
DRC-369 minting. The format is deliberately small: there is no binary section and no engine-private id.

---

## QOR ID and the vault

- Sign-in: the editor reads the launcher's existing QOR ID session from its store, as every surface does. The
  editor never calls QOR ID directly and never holds a token.
- Signing: through the launcher's host commands (`src/lib/ipc.ts`), behind the host dialog (L1.4). The engine sees
  a result, never a key.
- Publishing: the launcher's existing mint (`qontrol_mint`, from Qontrol's Mint panel) is reused. The QQ editor
  will have a "Publish" button that commits the scene, then mints that commit.

Neither `window.__qor_invoke__` nor `window.__qor_session__` exists, nor will: the first draft of this blueprint
named them, and they are corrected here (8 October 2026).

---

## ARQADE integration

A published QQ game is a JS bundle at `dist/game.js`. ARQADE loads it in an `<iframe>` with a QOR bridge injected
as `window.QOR = { session, invoke }`. If the player is signed in through QOR ID OAuth, collectibles and CGT
interactions are live; if not, the game runs with those features absent.

The iframe's `allow` attribute gates what the game can do: `allow="camera 'none'; microphone 'none'"` by default,
plus `autoplay` for Web Audio. No additional permissions without a written reason per game.

---

## Substrate consumed

| Substrate | How QQ uses it |
|---|---|
| **QOR ID** | Read-only session in the editor; OAuth bridge in ARQADE. QQ never holds a key. |
| **DRC-369** | A game is a Cartridge asset (ADR-070's derivation). Collectibles inside a game are DRC-369 items the player holds. QQ does not mint directly; it calls the launcher's mint path. |
| **Qontrol** | Scene versioning. The `.qq.json` scene format is designed to be diffed and merged by Qontrol's existing tools. |
| **ARQ Wallet** | Each published game's payout account (ADR-070). QQ does not interact with the wallet; ARQADE's server does. |
| **CGT** | Settlement for collectible purchases and payouts. All amounts are integer Sparks in u128; no float arithmetic. |

**Substrate gaps recorded, not filled locally:**
- **Access gating** for paid collectibles: waits on M4.4 (sponsored fees) and OPEN-4.
- **Mesh delivery:** the game bundle is served by Vercel today; Mesh delivery is M8.
- **Agent generation:** generative AI creates assets from a description (ADR-081 decision 4); this needs M5.2 and M5.3.

---

## Phases

### P3.1 — The runtime and editor surface exist

**Built 8 October 2026, not yet ticked** (DIRECTION.md P3 item 1 has the evidence). Built: the WebGL2 renderer
(shapes as signed-distance fields with glow, particles additively), the world (motion, a pointer follow,
emitters, deterministic per entity), the editor surface, saving and opening `scenes/<name>.qq.json` through the
host, and the canonical form that makes one changed value one changed line. **Not built yet:** the sprite
(ADR-082 decision 7), which needs an image read from the project through the host, and the SDF text renderer
of layer one.

A creator opens QQ from the launcher rail, sees a 2D viewport, drags in a coloured shape, adds a particle emitter, watches it animate, presses Stop and the editor is back. Nothing is published or signed.

Deliverables:
- `tools/qor-launcher/src/qq/runtime/` — WebGL2 renderer, scene graph, input
- `tools/qor-launcher/src/qq/editor/` — React editor surface (viewport + inspector)
- `tools/qor-launcher/src/qq/editor/QQView.tsx` — the launcher surface entry point
- Launcher rail entry: QQ, after Projects
- Qontrol commit of a `.qq.json` scene reads as meaningful text diff in Projects

Does not depend on any chain milestone.

### P3.2 — 3D and glTF import

3D renderer (layer 2 above), glTF 2.0 import, PBR shading, a directional light. A creator imports a `.glb` and it appears in the viewport with correct materials. Depends on P3.1.

### P3.3 — Publishing a game

A "Publish" button commits the scene through Qontrol and mints it as a Cartridge DRC-369 asset. The game bundle is a
standalone JS file the creator can share. Depends on P3.2, M4.1 and L1.4's native dialogs being exercised.

### P3.4 — ARQADE delivery

Published games appear in ARQADE. Pressing Play loads the QQ bundle in an iframe. QOR bridge active. Depends on P3.3 and P7 (ARQADE running).

### P3.5 — Collectibles and agent generation

DRC-369 items as in-game collectibles; generative AI builds assets from a description in the editor. Depends on M4.2, M5.2, M5.3.

### P3.6 — Mesh delivery and settlement

Games delivered over the Mesh; in-game purchases settled in CGT; access gating on paid collectibles. Depends on M6.4, M8.

---

## What QQ is not

- **Not a AAA engine.** No Nanite, no Lumen, no physics-simulation-for-films. The target is a 2D puzzle or a 3D
  effects-led arcade game — something one person can build in a week and that looks extraordinary.
- **Not a Unity or Godot project importer.** Assets move (via glTF 2.0). Projects do not.
- **Not a second identity system.** QOR ID is the only identity. The engine never asks for a login.
- **Not a second payment rail.** CGT is the only currency. The engine never calls a payment API directly.
- **Not a framework others adopt in year one.** The runtime will be open source eventually; in year one it is a
  launcher surface, not a SDK.
