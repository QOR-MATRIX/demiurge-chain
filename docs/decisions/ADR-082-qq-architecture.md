# ADR-082: QQ architecture — a WebGL2 engine and editor built from scratch in the launcher

**Superseded in part, 8 October 2026, by [ADR-083](ADR-083-qq-on-qt.md):** decisions 1, 3, 6 and 7 (the TypeScript/WebGL2
engine and React editor in the launcher's webview) give way to a Qt 6 engine; decisions 2, 4 and 5 stand. The text
below is kept as accepted.

**Status:** **Accepted**, 7 October 2026, under the delegation of ADR-081 (QQ's engineering and product choices are delegated to its builder without stopping for approval; choices are recorded as ADRs accepted under that delegation and reported to the owner).

## Context

ADR-081 makes QQ QOR Engine and authorises building it "by any means necessary." The withdrawn Godot plan depended on a large C++ build tracking an upstream that changes its fundamentals each minor release; the seam between a Godot process and the launcher's Tauri/Rust/WebView2 host was a sibling OS window or an unlanded upstream embedding API, neither of which lets a game run inside the launcher without a second window. The target is small-scale, effects-led 2D and 3D games that run in the launcher **and** in a browser on ARQADE, from one codebase, without a separate process.

A JavaScript/WebGL2 engine compiled to the launcher's WebView2 webview meets all three requirements: it runs in the same process as the launcher UI, it runs unchanged in ARQADE's browser context, and its output is a WebGL2 canvas, which the QFX layer already uses. The launcher's vault and QOR ID session are reachable through Tauri's `invoke` IPC. WebGPU is an enhancement added per-platform as evidence justifies it; WebGL2 is the baseline that exists everywhere Tauri 2 runs today.

## Decision

1. **QQ is a TypeScript/WebGL2 engine and editor embedded in the QOR Launcher's webview.** It does not depend on Godot, Bevy, Unreal or any other game engine. It is built from scratch, leveraging `gl-matrix` for math, `resvg-wasm` for SVG asset loading, and `wgpu`/WebGPU optionally where available.

2. **The runtime runs identically in the launcher webview and in a browser on ARQADE.** A published QQ game is a self-contained JavaScript bundle with a WebGL2 canvas and an optional QOR ID bridge. If the browser has no QOR ID session, the game runs without identity; collectibles and CGT are simply absent, not broken.

3. **The editor is a React surface inside the launcher**, alongside the launcher's existing surfaces (Projects, Inventory, etc.). It does not open a second OS window. It communicates with the QQ runtime through a shared `EditorBridge` interface (not IPC, same process).

4. **Signing, vault access and chain calls go through the launcher's existing paths.** The engine never holds a key. It posts a sign request through `window.__qor_invoke__` (the same Tauri bridge the rest of the launcher uses), which the vault handles behind the host dialog (L1.4).

5. **QQ games are DRC-369 assets.** A game is minted as a Cartridge asset through `Drc369::mint` when the creator publishes it. The engine writes nothing to the chain directly.

6. **QQ's packages live at `tools/qor-launcher/src/qq/`.** They are TypeScript modules bundled by Vite alongside the rest of the launcher. A QQ game also compiles to a standalone bundle for ARQADE delivery.

7. **The first QQ milestone (P3.1 as rewritten in DIRECTION.md)** is a renderer and an editor surface that can open, run and display a simple scene: coloured shapes, a sprite, a particle emitter, realtime pointer input. Nothing is published or signed at this milestone.

## Consequences

- `docs/blueprints/qq.md` replaces `docs/blueprints/qor-engine.md` as the active blueprint for P3.
- The superseded Godot blueprint (`docs/blueprints/qor-engine.md`) stays as a historical record with its superseded header.
- DIRECTION.md's P3 items are rewritten as QQ's items (done in the same commit as this ADR).
- The `qor-engine` gate's `qor-engine.roadmap` and `qor-engine.decisions` criteria count QQ's track items; the old Godot items are superseded by those.
- No `products/qor-engine/` directory is created; QQ lives at `tools/qor-launcher/src/qq/` and its ARQADE bundle at `products/arqade/vendor/qq/` when first shipped.
- The innovation budget (ADR-001) argument was explicitly set aside for QQ by the owner in ADR-081. That acceptance stands.
- No economic value is invented. CGT paths, fee classes and CGT amounts follow the rules in AGENTS.md §5.
