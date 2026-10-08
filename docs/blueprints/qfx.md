# QFX

**Status:** Blueprint, 21 September 2026. Describes intent, except for layer one, which landed the same day. The research below was done against a tree with no QFX in it at all — no canvas, no `requestAnimationFrame`, no WebGL, no shader, no audio anywhere in `tools/qor-launcher/`. Commit `75b2ead` built the first slice of layer one: `src/qfx/Canvas.tsx` (one WebGL2 fragment stage on a canvas behind `#qor-root`), `src/qfx/contrast.ts` (the guarantee, shared with `scripts/check-contrast.mjs`), the `.qfx-scrim` in `src/styles/qor.css`, and an Ambience setting with three values. There is still no theme format, no authoring path, no shader validation and no audio: **layers two and three are entirely unbuilt, and everything this document says about them is intent.**

**Since 22 September 2026:** ADR-047, the object model and DRC-369's wire format, is accepted and M2.3 is ticked, and M2.1 was ticked the same day against the owner's review of a ten-line summary of the inventory's DRC-369 section. Wherever this document says something waits on "M2.1 and M2.3", it now waits on neither, and where it calls the content fingerprint undecided, it is a BLAKE3-256 manifest root with its algorithm tagged.

**ADR-051 was accepted on 22 September 2026, as amended to fit layer one.** Where this document proposes a luminance clamp, a separate pause control or no canvas behind the Gate, the accepted record replaced them: the scrim is the guarantee (0.90), Still is the pause, and only the built-in ambience runs behind the Gate. The owner's answers to every open question are at the end.

## What it is

QFX is the part of Demiurge that lets a creator author how the software itself looks and moves — the living ground behind the interface, the way a panel answers when you touch it, and eventually a room you can stand in. What a creator makes is a theme: one file another person can install, tune with sliders, and build their own version on top of. When someone uses that theme, the person who made it is paid, and when someone remixes it, the original author is paid too.

## Who it is for

- **The few who write the look.** Shader writers, motion designers, and people who already make demoscene or TouchDesigner work. They author scenes and motion sets.
- **The many who tune it.** Everybody else, working entirely through sliders over parameters the scene's author chose to expose. A tuned result is publishable on its own terms, as a preset. This is the majority path and it is a success, not a shortfall.
- **Engine creators.** Layer three is a scene editor inside QQ Studio, the QOR Engine (ADR-083: Qt 6, scenes in QML), so a person who builds worlds in QQ is already an author here.
- **Everyone who opens the launcher.** They get the default look, unchanged, and one switch that turns all of this off.

## The foundation, and why

ADR-001 says to build on proven foundations wherever failure is silent, and to invent only at the creator-economy layer. Every foundation below was chosen against a failure that is silent — a GPU context that succeeds while running on a software rasteriser, text that drops under 4.5:1 for one frame in six hundred, a shader that hangs a driver on someone else's machine.

| Layer | Foundation | Alternatives rejected, and why each lost |
| --- | --- | --- |
| **1. Canvas** | **WebGL2, one fragment stage, one canvas as a sibling behind `#qor-root` in the existing webview.** | **WebGPU:** absent from WebKitGTK entirely, uncontracted in WebView2, macOS 26+ only in WKWebView. It cannot be a baseline on two of three platforms. **A second always-behind transparent window:** macOS transparency needs `macOSPrivateApi`, which forecloses the App Store; Windows transparency in Tauri 2 is reported broken; two windows never move or resize atomically, so dragging the launcher shows the chrome sliding off its own backdrop. **A native `wgpu` surface under a transparent webview:** technically the best answer and beyond twelve months — it needs per-platform window internals Tauri does not expose and collides with `unsafe_code = "forbid"` in `src-tauri/Cargo.toml`. |
| **2. Binding** | **The CSS cascade the launcher already uses.** Themes write one rule on `:root[data-theme]`; the accessibility overrides sit outside every cascade layer and therefore outrank any theme; reduced motion is `!important` inside `@layer base`, which beats `!important` in every later layer. | Inline custom properties on `<html>`. The repository already made that mistake once and wrote down the result: contrast settings changed nothing, because nothing could outrank an inline property. |
| **3. Scene editor** | **QQ Studio, the QOR Engine** (ADR-083: Qt 6, Qt Quick 3D, scenes in QML), with the QFX scene editor **inside the Studio**, never a second tool. *First planned as a plugin in a custom Godot build; that plan was withdrawn by ADR-081.* | A scene editor is written once, in QQ, and QFX uses it rather than building its own. |
| **Scene format** | **`.tscn` is the precedent, and the requirement.** Text, diffable, mergeable — therefore hashable, therefore mintable under DRC-369 and versionable by Qontrol without inventing a format. GNOSIS's project format takes the same three properties as a requirement and cites the same precedent, so this is one decision across two products. | A binary scene container. It cannot be reviewed, cannot be merged, and turns a remix into an opaque blob instead of a readable diff. |
| **Shader validation** | **`naga`, in the Rust host, before a shader reaches the webview.** It gives a typed IR to walk: reject unbounded loops, dynamic indexing without bounds, undeclared uniforms, calls past a small depth. | Regex-scanning GLSL text. A shader with an input-dependent `while` across two million pixels is a GPU denial of service; on Windows the driver reset blanks every accelerated window on the machine. `naga` is a correctness checker, not a security boundary — so it is layer one of four, not the answer. |
| **Contrast model** | **WCAG 2.x, 4.5:1, as the gate.** Deterministic, closed-form, already what the accessibility panel reasons in. | APCA. WCAG 3 is a Working Draft and its contrast algorithm is still undetermined as of April 2026. Gating a release criterion on an unsettled method is indefensible. APCA Lc is computed as an advisory second opinion in the same report. |

## What is new

Everything above exists somewhere. The part that does not exist anywhere is this:

**A preset is a mechanically verifiable remix.** A scene declares a typed, bounded parameter surface; a preset is a values-only override of that surface and nothing else. So a remix is *literally a diff against a named parent* — not a claim, not a metadata field somebody typed. That is the cleanest demonstration of DRC-369 remix royalties in the whole roadmap, because the derivation can be checked by machine and settled by the chain.

**The guarantee travels with the theme, and the creator cannot weaken it.** A theme ships a shader and a parameter table. It never ships JavaScript, never registers a frame callback, never gets a timer. The chrome owns the loop, the clock, the luminance clamp and the scrim. A theme can only ever ask for *more* readable, never less. Nobody else ships user-authored chrome with an arithmetic floor under it.

**A look is a thing you spend on, not a thing you hold.** A licence to use a scene is access gating, the second demand sink in ADR-006, and it settles in CGT. The creator is paid when their work is used, licensed or remixed (ADR-002). Nothing in QFX is scarce, numbered or editioned; no copy anywhere will suggest a theme is worth owning rather than worth using, and nothing in QFX says anything about what CGT is worth.

## Architecture

### The rule

> **The design system governs the default theme and the chrome. QFX governs the canvas.**

Operationally that is four token classes. The first two exist today; the last two are the layer-two build.

| Class | Owner | Themeable by QFX | Today | Examples |
| --- | --- | --- | --- | --- |
| **Guarantee tokens** | Design system, fixed | **No** | Partly tokens, partly hard-coded | Type scale, tracking scale, status colours, radius, focus-ring width and offset, minimum hit target |
| **Chrome tokens** | Design system; a theme picks values from a validated set | Values only | **Exists** — the fifteen tokens in `src/styles/themes.ts` | `--void`, `--base`, `--surface`, `--raised`, `--well`, the accent trio, `--counter`, the ink and edge ramps |
| **Motion and material tokens** | **Shared.** The design system sets the envelope; QFX sets the value inside it | **Clamped** | **Does not exist.** Fourteen hard-coded durations, one easing curve, one blur/saturate pair, one 72% surface mix, one press depth | `--dur-control`, `--dur-view`, `--ease-*`, `--blur-panel`, `--sat-panel`, `--panel-fill`, `--press-depth`, `--rise-distance` |
| **Canvas tokens** | QFX | **Yes, freely** | Does not exist | Scene id, preset id, canvas opacity, parallax amount, pointer gain, audio gain |

Three mechanisms make that structural rather than aspirational:

1. **QFX writes one stylesheet rule on `:root[data-qfx]`**, exactly as `applyTheme` writes `:root[data-theme]`. Never inline properties.
2. **No chrome component reads a QFX token directly.** Every motion and material token is consumed through a `clamp()` in `qor.css` — `--dur-control: clamp(120ms, var(--qfx-dur-control, 200ms), 320ms)`. A theme cannot set a four-second button transition and cannot set zero.
3. **The existing kill switches gain one declaration each.** `:root[data-contrast='maximum']` and `:root[data-solid]` already force panels opaque with `backdrop-filter: none`; both also set canvas opacity to zero. The guarantee becomes structural instead of sampled.

Three chrome surfaces a theme never reaches: **the Gate** (the canvas mounts inside the signed-in branch only, so a passphrase field never has motion behind it — **amended by the owner on 22 September 2026:** the built-in default ambience may run behind the Gate, as layer one does (`src/App.tsx:90`), and an installed theme never may), **the host-drawn signature dialog** (native, outside the webview, safe by construction — stated here so nobody later "unifies" it), and **status and toasts** (status colours are the same in every theme by design; a failed transfer must look identical everywhere).

### The default stays as chosen

Architect — ember on carbon — with no creator's scene and no preset. **What the default does have, by the owner's decision of 21 September 2026, is layer one's own backdrop, live at a restrained amplitude:** a slow drift between the theme's `--base` and `--accent`, lifted near the pointer, under the chrome's scrim (`src/qfx/Canvas.tsx`, `DEFAULT_AMPLITUDE = 0.35`). No neon, no glow, no gradients, no particle background: a choice for the default, not a rule, since ADR-080 allows all of them. The dreamlike look is a creator's choice a person opts into, **never the default.** This section's first draft said the default would have no canvas at all; that was written before the owner scoped layer one, and the owner's scope supersedes it.

From 21 September 2026 `scripts/check-design.mjs` exempted exactly one directory, `src/qfx/`, from exactly two rules — the canvas rule and the pointer rule — and nothing else in `src/` could contain `getContext(`, `requestAnimationFrame`, `pointermove` or `onPointerMove`. That narrowing was made with layer one and logged in `GATES.toml` as a scope change (`42d00c3`), and the owner approved it retroactively the next day (decision 1 below). **ADR-080 (6 October 2026) removed those rules, and three more effect rules, from every directory, and the exemption with them.** `check-design.mjs` now checks only that colours come from tokens and that sizes and letter-spacing come from the scales; canvas, pointer-reactive light and looping animation are allowed anywhere, still bound by reduced motion and readability as painted. The five tightenings this section proposed alongside the narrowing were not made, and stand as proposals: shader files become scannable; the default theme is asserted to register no scene; `--qfx-*` may appear only inside a `clamp()` in `qor.css`; at most one frame-loop call site exists in the tree; colour literals stay banned in scene registries. CI runs `check-design.mjs` with the launcher's other checks (`npm run check`), and CI on `main` is green since 5 October 2026.

### The contrast guarantee, as checks

Two mechanisms, both owned by the chrome, neither reachable from a theme. **The luminance clamp:** the theme's shader draws to an offscreen target; a fixed final pass owned by the runtime tone-maps it, clamping relative luminance to a cap and tinting toward `--base`. A theme's shader cannot produce white. **The scrim:** text already sits on `.surface`/`.glass`, a 72% mix of `--surface`; that number has simply never had to carry a guarantee before. Composed, the worst-case background under any text is closed-form: `A·scrim + (1−A)·B`, with `B` bounded by the clamp. At a cap of Y ≤ 0.10 and today's 72% panel, every shipped theme clears 4.5:1 for `--ink-body` and `--ink-muted` against an adversarial shader. Without the clamp, `--ink-muted` would need 84–86% and today's panel would fail.

| Id | Requirement | Pass condition a script checks | Owner |
| --- | --- | --- | --- |
| **C1** | The panel floor is structural | From computed style, read `--ink-body`, `--ink-muted`, the panel fill alpha and the runtime's luminance-cap constant; compute the worst case in Node; assert ≥ 4.5:1 for body and muted text, ≥ 3:1 for `--edge` hairlines. Runs with no GPU and cannot be flaky | `check-contrast.mjs` |
| **C2** | ~~`--ink-faint` never appears over the canvas~~ **Superseded on 22 September 2026:** `--ink-faint` was fixed and holds 4.5:1 over any backdrop, so it may | It measured 2.87–3.26:1 before the fix; `check-contrast.mjs` now holds every ink step, faint included, to the floor over any backdrop and on every opaque background | `check-contrast.mjs` |
| **C3** | Worst frame, not mean frame | Render a scene under a reference chrome layout across ≥ 600 frames spanning its declared period × pointer at nine grid positions × audio at zero and full; screenshot each; compute the pixel-level ratio under every text run; **pass on the worst frame** | `check-contrast.mjs --render` |
| **C4** | The clamp is wired, not merely declared | A fixture theme `__test_whiteout` whose shader outputs `vec4(1.0)` still passes C3 | `check-contrast.mjs --render` |
| **C5** | The switches dominate | At `data-contrast='maximum'` and at `data-solid`: canvas computed opacity is `0`, and the frame-callback count does not increase over one second | `check-accessibility.mjs` |
| **C6** | Pause, stop, hide (WCAG 2.2 SC 2.2.2) | A keyboard-reachable pause control exists, is in the tab order, and activating it stops the frame-callback count increasing. A living backdrop starts automatically, lasts over five seconds and is never essential, so this is a requirement, not a preference | `check-accessibility.mjs` |
| **C7** | Three flashes (SC 2.3.1) | From the C3 captures, per-frame mean relative luminance over the central 10% of the viewport crosses the general flash threshold no more than three times per second | `check-contrast.mjs --render` |
| **C8** | A scene cannot be exported without C3 and C7 passing, and the results travel in its manifest | Editor-side gate; the manifest carries the measured numbers and the runtime version they were measured under | QOR Engine plugin |

**The one unverified thing in that table:** C3, C4 and C7 need a real WebGL2 context in a headless run, and it is **not verified that the existing CDP harness can get one** — Chromium's SwiftShader fallback is being removed, `--enable-unsafe-swiftshader` is a stopgap, and a Mesa path has not been tried. If neither works, C3, C4 and C7 run on a developer machine and say so, and C1, C2 and C5 carry CI alone. That is a weaker guarantee, stated rather than hidden.

### The frame-time budget, as checks

The RAIL baseline is 16 ms per frame with about 6 ms of it already spent by the browser, hence 10 ms. The launcher is not a game. The backdrop is decoration the person chose, and it loses every contest against the interface. GPU timer queries are disabled in Chromium as a side-channel mitigation, so the budget is measured on the CPU from frame intervals — which is the thing that actually matters, because ambience and interface share one compositor.

| Id | Requirement | Pass condition a script checks | Owner |
| --- | --- | --- | --- |
| **F1** | Ambience Off costs nothing | The canvas is hidden, zero frame callbacks are scheduled, and a launcher that opens with Off never creates a GL context. The first two are checked in a real rendering engine (`check-accessibility.mjs`, since 2026-09-21); the third is in the code and not yet asserted | `check-accessibility.mjs` + `check-frame-budget.mjs` |
| **F2** | Scene CPU cost per frame ≤ 1 ms | The runtime uploads uniforms and issues one draw. More than that is a defect, not a tuning problem | `check-frame-budget.mjs` |
| **F3** | Interacting frames: p95 ≤ 10 ms, p99 ≤ 16 ms | Measured across a scripted Nexus → Vault → Nexus cycle including the view cross-fade | `check-frame-budget.mjs` |
| **F4** | Sustained: no dropped frames over 60 s idle | Frame-interval histogram; dropped means an interval over 20 ms | `check-frame-budget.mjs` |
| **F5** | The ambience never takes more than a named fraction of the frame | ≤ 25% of the frame interval at the top tier, ≤ 15% at the lowest running tier. Pinned in the ADR so the governor enforces a number rather than a feeling | `check-frame-budget.mjs` |
| **F6** | Degradation is automatic, ordered, and visible | Three consecutive seconds over budget → halve render scale → halve the cap → suspend, and **say so in Settings.** Never silently. Promote at most one tier per thirty seconds, or it oscillates | scripted against a deliberately slow fixture |
| **F7** | Preset switching costs zero shader compiles | Assert the compile count is unchanged across a full preset sweep. This is why there are no static switches in the format: they recompile, and permutations make the budget unprovable | `check-frame-budget.mjs` |
| **F8** | Suspended when it cannot be seen | Window unfocused, minimised, occluded, or the Gate showing → no frame callbacks | host focus event → runtime |
| **F9** | On battery or power-save, demote and cap | Power state comes from the host, not `navigator.getBattery()`, which is Chromium-only. Context creation passes `powerPreference: 'low-power'` so a laptop does not wake its discrete GPU for a backdrop | host → runtime |
| **F10** | A bad shader cannot take the machine down | Compile in a worker under a wall-clock cap, cancellable; probation render at 64×64 before full-screen; `webglcontextlost` suspends QFX and leaves the shell running; a theme that loses the context twice is disabled locally with a plain message and never auto-retried | forced-failure test |
| **F11** | Capability detection is empirical | WebKitGTK masks the renderer string and succeeds on a software rasteriser with no error to catch. The only honest probe is a timed calibration render at first run, re-run when the display or driver configuration changes | runtime |

### Reduced motion means still

Not slower. Not gentler. **Still.**

- **R1 — the theme has no mechanism to animate.** It ships a shader and a parameter table; the chrome owns the single frame loop. Everything below is belt-and-braces.
- **R2 — still is one real frame, not a paused loop.** One render with the clock frozen, audio bands at zero, pointer at centre, then the loop is cancelled. The check is binary: is a callback registered.
- **R3 — one resolver, used by the CSS and the runtime**, reading the same `data-motion` states the accessibility panel already writes, so the two cannot drift.
- **R4 — the pixel test.** Under emulated `prefers-reduced-motion: reduce`, two screenshots one second apart are byte-identical. Not "did we set a flag" — did the pixels move.
- **R5 — the manifest has no motion override field.** `tier_min` can only make a theme *stiller*. Unknown manifest keys are rejected, so a future field cannot be hand-edited in.

### A theme cannot make the launcher unusable

The shell renders identically with QFX absent — true today, and the property to keep true. It is also the entire fallback story: the zero tier is the launcher exactly as it is now. A scene failure surfaces as a notice in Settings and nowhere else. The canvas carries `aria-hidden="true"` and `pointer-events: none`, sits outside the main landmark, and is not in the tab order, asserted in the rendering engine rather than in source. And `img-src` in the CSP is narrowed before any scene may load an image: today it would let a scene pull a texture from any https origin, which is a fingerprinting channel keyed to which theme you use. Layer one's shader reads no texture, so this is not yet exposed.

### The theme package

One package, one asset, and **text, diffable, mergeable** — the same requirement the scene format takes from `.tscn`, for the same reason: a remix has to be readable as a diff. The palette block **is** today's fifteen-token `Theme['tokens']`, reused exactly — that is what makes a QFX theme a theme rather than a wallpaper. Around it: the ambience source and its tier table; `[ambience.params]`, every uniform the shader may read, typed and bounded, with the runtime building the uniform block from it and rejecting any shader that reads an undeclared uniform; `[ambience.pointer]`, clamped; `[ambience.audio]`, which **declares intent and grants nothing** — with audio off, every band reads zero and the theme still runs, and the theme cannot tell, which removes any incentive to nag; `[motion]`, drawn from a closed set of curves; and `[contrast]`, which is a *request*: the chrome takes `max(theme.scrim_min, chrome.floor)` and `min(theme.luminance_cap, chrome.cap)`.

**The package carries no author field, no price field and no licence field.** Those are DRC-369 and QOR ID records, not strings in a file nobody can verify.

## Substrate consumed

| Substrate | How QFX uses it | Status |
| --- | --- | --- |
| **QOR ID** | The author of a theme, a scene and a preset. Identity stays off chain; only the account is on chain (ADR-027). QFX gets no login of its own, and neither does QOR Engine | Partial and real |
| **DRC-369** | A theme *is* a DRC-369 asset. Ownership and enumeration from `pallet-nfts`; nesting for sub-scenes and for a theme that references a scene; the content fingerprint for "the theme you received is the theme that was minted"; remix royalties for presets. **QFX needs no new pallet and must not get one** | **M4, started and not blocked.** M2.1 and M2.3 were ticked on 22 September 2026; M4.1 (mint, fingerprint, enumeration) is done, and M4.2's royalties, remix royalties and nesting are built. No theme is minted, because there is no theme package yet |
| **Qontrol** | Version control for scene sources and for `.tscn`, over the settled split: gitoxide for every read path, libgit2 in a sidecar process for stage and write-tree only, both behind Qontrol's interface. A `.tscn` semantic merge driver is what makes two people able to edit one scene | **The split is built:** the launcher's Projects surface and the `qontrol-git` helper (since 21 September 2026, bundled since 26 September). The `.tscn` merge driver is not built |
| **Mesh** | Where the theme's bytes live. The chain holds a fingerprint; the payload does not | **M8** |
| **CGT** | Settlement for a licence to use a scene, under the access-gating sink (ADR-006). The creator is paid when the work is used, licensed or remixed (ADR-002) | The chain has no fees, no issuance and no treasury |

**SUBSTRATE GAP 1 — a theme package has nowhere to live.** There is no content-addressed store until M8. Phases 1 and 2 therefore ship **built-in themes compiled into the launcher and loaded from disk**, which exercises manifest parsing, validation, sandboxing, tiering and the contrast enforcement against real content. **No local theme registry, no download URL, no signature scheme of QFX's own** — that would be a second distribution rail beside the Mesh and a second asset format beside DRC-369. A preset a person saves is a local file they own, not a registry entry, and nobody else can install it until M4 and M8.

**SUBSTRATE GAP 2 — authorship has no record to live in either.** Until M4, "who made this theme" can only be a string in a file, and a string in a file is not provenance. QFX does not add an `author` field to compensate: until DRC-369 exists, every built-in theme is authored by the project itself and the question does not arise; after it exists, the answer is one QOR ID on one asset record.

**SUBSTRATE GAP 3 — shader bytes in the fingerprint.** Whether the DRC-369 wire format covers an executable shader payload is an M2.3 input nobody has raised. If it is not fingerprinted, a theme's behaviour can be swapped under its identity. **Answered on 22 September 2026 (decision 7 below):** ADR-047's manifest covers it by construction, since a theme's shader is one of its entries.

**SUBSTRATE GAP 4 — scene, project and sub-scene naming.** DRC-369 must be able to name a scene, a project and a nested sub-scene. QOR Engine must not mint a project id of its own. Also an M2.3 question.

**SUBSTRATE GAP 5 — entitlement has no check to call.** "This person may use this scene" is DRC-369 entitlement (M8.2, with the royalty path at M4.2), settled in CGT (M6). QFX writes **no local licence file, no entitlement cache and no purchase path of its own**; until those milestones land, every theme a person can use is one that shipped with the launcher.

**Economics not to be invented here.** What a licence costs, what fee class a mint or a licence falls in, and what share burns are **OPEN-4**. Issuance is **OPEN-1**; the genesis split is **OPEN-2**. QFX prices nothing, in code, in a mock, or in a document. Any test that needs such a number uses one marked plainly as a placeholder.

## The first usable slice

**What shipped first is layer one, because the owner scoped it that way on 21 September 2026** (`75b2ead`): one WebGL2 canvas behind the interface; one shader drifting slowly between the active theme's own tokens, lifted near the pointer; an Ambience setting in Settings with Off, Still and Live, Live the default at a restrained amplitude; reduced motion forcing Still; a frame-time budget that stops the canvas on its last frame; no audio. The chrome's contrast guarantee is a full-screen scrim, at 0.86 when it shipped and 0.90 since 22 September, whose opacity is a literal in `qor.css`, not a theme token, and `check-contrast.mjs` solves for the worst colour the canvas could paint beneath it. Off first shipped with the scrim removed and the canvas still showing; that was corrected the same day (`6e5bb28`) and is pinned in `check-accessibility.mjs`.

**It meets ADR-051 as the owner amended and accepted it on 22 September 2026**: the built-in ambience may run behind the Gate, the scrim is the guarantee (raised to 0.90 that day, when `--ink-faint` was fixed), the one-step budget is accepted for v0, and Still is the pause. The tiers and the render-based checks are P2.2.

**The slice this section first proposed is now the next one.** A person opens Settings, picks a theme, moves five sliders, and the whole launcher answers — and none of it can make anything harder to read or slower to use. Concretely: the motion and material tokens exist and are clamped; two or three built-in themes ship a motion set as well as a palette; the sliders write to `:root[data-qfx]`; reduced motion, maximum contrast and reduce-transparency each override everything, provably; and the result is savable as a named local preset. It has no GPU, chain, Mesh or engine dependency of any kind.

## Phases to a full product

**One correction to the stated order, made openly.** The three layers are the three phases, but *within* Phase 1 the guarantee and the tokens land before the first shader. Build the contrast floor, the tier ladder and still-means-still against a placeholder that drifts two colours, then let creators near it. The reverse order produces a beautiful backdrop nobody can turn down and nothing can read text over, and every fix after that is a fight with a theme that already shipped.

### Phase 1 — Layer one: the living backdrop

*Authoring path:* **knobs first, graph second, source third.** Knobs are sliders over a scene's declared parameter surface, live, in Settings — zero compile, instantly reversible, and the result is a publishable preset. The node editor comes next and compiles to the same scene format, with Godot's rule mandatory: **always show the generated code**, and diff it against the parent when the graph is a fork. Pasted GLSL is third and for the few; a hand-written scene and a compiled one are indistinguishable to the runtime.

*Needs from layer two:* the clamped motion and material tokens, and the canvas-token contract. *From layer three and QOR Engine:* nothing. *From the launcher:* a canvas mount behind `#qor-root` (not inside it — `#qor-root` carries the zoom, and a canvas inside would be rescaled against its own drawing buffer), a monotonic clock, a pointer stream, suspend and resume on focus and power, and the narrowed `img-src`. *From the chain:* nothing. Themes are built in and local.

*Not in this phase:* audio. Audio capture is the most invasive thing this application would ever do; it is a privacy decision, not a rendering detail, and the host owns the capture, never the webview. Off by default, explicit opt-in, a persistent indicator while live, never auto-enabled by a theme.

### Phase 2 — Layer two: the responsive interface

*Authoring path:* **preset composition over motion and material tokens.** A creator picks a palette, a motion set and at most one scene with one preset, names the combination, and that is a theme. No code. This is the phase where "authoring a theme" becomes something a person with taste and no shader knowledge can finish.

*Needs from layer one:* only the canvas-token contract; it works standalone with no scene at all, and should ship that way first. *From layer three:* nothing. *From the chain:* nothing until publishing.

*Shipped first, on 22 September 2026, because the owner asked for it:* **DRC-369 assets are cards.** Every asset the Inventory reads from chain storage is drawn as a card that leans towards the pointer and opens, at size, on its own title. Its face carries what the chain holds and nothing invented — the name, the reference it carries now, the commit it pins, permanent or not — and, until an asset has a preview, a mark derived from the fingerprint itself, so the same asset always draws the same mark and two assets do not look alike. It lives in `src/qfx/`, where the pointer rule then allowed it, under the carve-out the owner widened that day from the backdrop to a QFX-owned surface. **There is no sheen yet.** When the card was built, a sheen was a gradient and gradients were forbidden in every directory, so none was made; since ADR-080 (6 October 2026) gradients are allowed, so a sheen may be one. None is built.

*The slices after it, in order.* **The sheen**, as designed: the one canvas already behind the interface shows *through* the card, rather than a second canvas being started for it (ADR-051 — one canvas, owned by the chrome). Since ADR-080 a gradient on the card itself is also allowed; the design keeps the canvas route, and either way reduced motion stills it. **The preview image**, once a mint names one: ADR-047 has the Preview role and nothing sets it, and the bytes have to be read from somewhere, which is the temporary content store until the Mesh exists (M8). **The model**, a glTF or GLB entry rendered in that same canvas, which is the 3D view the owner asked for. Each is a slice on its own, and none of them may cost readability or stillness.

### Phase 3 — Layer three: the personal space

*Authoring path:* **the scene editor, inside QQ Studio** (ADR-083; the Godot plugin first written here is withdrawn). Not a second tool, not a separate download, not a webview reimplementation. A scene is QML — text, diffable, mergeable — so it is reviewable, hashable, mintable and versionable without inventing anything, and GNOSIS's project format holds itself to the same rule. A QFX layer-one scene becomes a *leaf* in that graph rather than a competing format, which is the reason layers one and two are built as a DAG of typed nodes with a declared parameter surface from day one: if the engine work slips, nothing was wasted.

*Needs from layer one and two:* both, plus scene composition (nesting). *From QOR Engine (QQ Studio, ADR-083):* the Studio, the QOR ID session, and signing asked of the launcher's vault over the local channel the launcher opens. Two hard constraints, stated once: the first shippable integration is a **child or sibling OS window**, not an in-webview canvas; and the engine instance, which executes project content, **is never inside the vault's trust boundary** — QQ Studio is its own process, started by the launcher, and holds no key. *From the chain:* DRC-369 for identity, ownership and remix royalty (M4); the Mesh for the payload (M8); CGT settlement with the fee class open (OPEN-4).

## Honest feasibility

One founder and an agent, working sequentially. Where the research says two pieces of work are each "about three months, in parallel", one founder does not get parallel, and the horizons below reflect that.

### What is reused

| Reused | Where it already is |
| --- | --- |
| Token-swap theming, the `:root[data-theme]` rule, cascade-layer discipline, accessibility overrides that outrank themes | `src/styles/themes.ts`, `src/lib/a11y.ts`, `src/styles/qor.css` |
| The headless CDP check harness — contrast probes, media emulation, driving the built app in a real rendering engine | `scripts/check-accessibility.mjs`; add screenshot capture and a frame-callback counter |
| Tauri IPC, the capability model, the host-owns-privilege posture | `src-tauri/capabilities/default.json` |
| Qt 6 (ADR-083): Qt Quick 3D, QML as the scene format, the Studio as its own process | Under the owner's commercial licence |
| `naga` as a GLSL front end; `pallet-nfts` under DRC-369; gitoxide and libgit2 under Qontrol | Upstream crates, and decisions already taken |

### What is new, and when

| Horizon | What is realistically done | What it is honestly gated on |
| --- | --- | --- |
| **3 months** | The ADR and `GATES.toml` change-log entry narrowing `check-design.mjs` (days of work, and it gates everything else); the motion and material tokens extracted and clamped, with the design-system document updated in the same change; preset composition over them; `check-contrast.mjs` C1/C2/C5 and `check-frame-budget.mjs` F1; the reduced-motion pixel test. **That is the first usable slice, and it is the whole three months. No canvas, no shader** | The owner's decision on the check narrowing, which is question 1 below |
| **12 months** | Layer one for **scenes we wrote**: the WebGL2 runtime, scene contract, uniform plumbing, tier ladder, frame governor, luminance clamp, suspend and resume, the knobs UI, a small hand-written scene library shipped off by default, and the render-based checks. Host microphone audio on the three platforms is a further three months and only after the privacy decision | A WebGL2 context in headless CI, unverified today; and the shader validator, which is itself a twelve-month item — so within twelve months **creators tune scenes, they do not yet write them** |
| **Beyond 12 months** | A creator writing a scene we did not write (the validator, the subset, the probation render, the context-loss blacklist); a node editor good enough to be the authoring story; publishing (M4); distribution (M8); paying (M6, with OPEN-4 undecided); macOS system-audio loopback (L6.1 and a signing identity); layer three | Layer three is **unschedulable**, not late: QOR Engine has a track (P3) and no item started, no accepted ADR and no owner-approved published names |
| **Not planned** | A WebGPU path; a native GPU surface under a transparent webview | Absent on Linux and uncontracted on Windows; and `unsafe_code = "forbid"` plus Tauri's window model |

**Since this table was written, the owner reordered its first rows:** layer one's first slice — a canvas and one shader — shipped before the tokens (`75b2ead`), with the narrowing logged beside it (`42d00c3`). The token slice in the first row is still ahead, and the second row's canvas is started rather than untouched. Since ADR-080 (6 October 2026) the first row's narrowing, and the owner's decision it waited on, are moot: the rules it narrowed are gone.

These are estimates, not commitments. What is not an estimate: **CI runs** on GitHub Actions on the public repository (ADR-063), passes on `main`, and runs the launcher's design, accessibility, contrast and readability checks on every pull request; `main` is green. The render-based checks (C3, C4, C7) and `check-frame-budget.mjs` are not written yet.

### What it cannot start without

| Needs | Milestone | State |
| --- | --- | --- |
| Accessibility overrides that outrank any theme | **L1.2** | **Done.** QFX must not regress it |
| Any check running automatically at all | **L1.6, L1.7** | **CI runs** (GitHub Actions on the public repository, ADR-063), with the launcher's checks in it, and passes on `main` since 5 October 2026 |
| Host-confirmed signing, for publishing from the launcher | **L1.4** | Implemented, unexercised |
| Minting a theme from the launcher | **L4.1, L4.2** → **M4** | Not blocked on the chain: the launcher mints a Qontrol commit since M4.1 (22 September 2026). Waits on the theme package (P2.5) |
| Shader bytes inside the fingerprint | **M2.3** | Answered 22 September 2026 (decision 7): the shader is a manifest entry |
| Ownership, nesting, remix royalties, sponsored mint | **M4.1, M4.2, M4.4** | M4.1 done; M4.2's royalties, remix royalties and nesting built; M4.4 unstarted |
| Anything settling in CGT | **M6**, with OPEN-1, OPEN-2 and OPEN-4 undecided | Unstarted |
| Distributing a theme to another person, and checking an entitlement | **M8.1, M8.2**, surfaced by **L7** | Unstarted |
| macOS system-audio capture | **L6.1** signed installers | Not started. The system keys the grant to a stable signing identity, so there is no macOS loopback before there is a real one |
| Studio inside the launcher, where publishing lives | **L7.5** → M8 | Unstarted |

## What "full support" means, and what it does not

**It means:** a creator authors a look from sliders, a node graph or GLSL; publishes it as a DRC-369 asset with their QOR ID on it; and is paid in CGT when it is used or when someone builds on it, with a preset's parentage verifiable by diff rather than by claim. It means the same scene format serves the launcher's backdrop and the engine's scene editor. It means a person who wants none of it gets the launcher exactly as it is today, with one switch.

**It does not mean a creator can ship arbitrary code.** A theme is a shader plus a parameter table inside a published, enforced subset: no `while`, no `do`, `for` loops with literal bounds under a constant, no recursion, no user textures in the first version, no static switches, no JavaScript, no timer, no network. Creators will complain. The answer is that the subset is the price of a theme that cannot take someone's machine down, and that constraint has produced better work in every scene that ever had one. Expect the validator to reject some legitimate shaders; that is the correct direction to fail in.

**It does not mean the canvas gets to look like anything a creator wants.** The luminance clamp means a shader cannot produce white. Nothing authored can lower the panel floor, unfreeze reduced motion, or survive maximum contrast.

**It does not mean parity across platforms.** WebGL2 is the ceiling on Linux and even that is conditional: it landed in WebKitGTK 2.40, and Ubuntu 22.04 ships 2.36. Some Linux users get the still tier, permanently, and are told so plainly rather than left wondering. No WebGPU path in the first version, and probably never for this workload.

**It does not mean audio on every platform.** Windows loopback works but the permission rule on current Windows 11 is unverified and must be tested on a clean machine before anything is promised. macOS needs 14.2+ and a signing identity that does not exist yet. Linux works today via the monitor source. Microphone and system audio are separate grants and separate decisions.

**It does not mean publishing.** Phases 1 and 2 ship built-in themes only. Until M4 and M8, **nothing is shareable**, and QFX will not grow a local substitute for either.

**It does not mean outside creators inside a year.** Within twelve months the scenes are ours and the creativity is in the tuning. A person writing a scene we did not write waits on the validator, and the validator is the serious engineering in this whole document.

**It does not mean layer three has a date.** QQ on Qt (ADR-083) is a real plan with a track (P3), waiting on Qt being installed. Phase 3 is written down so that phases 1 and 2 are built in a shape that fits it — not because it is scheduled.

## Risks

1. **A generic look.** L1.1 removed a particle field, glows and pointer light, and layer one brought that class of thing back at larger scale — first through an owner-approved exemption, and since ADR-080 (6 October 2026) because effects are allowed on every surface. That return is on the record, so it is not erosion. The risk that remains is the one ADR-080 names: a backdrop that looks like the default output of a template or an AI rather than Demiurge's, or one that costs readability or stillness.
2. **`backdrop-filter` over a moving ground.** Six panels of 20px blur over a static ground rasterise once; over an animated canvas they recompute every frame. This is the single most expensive thing in the design. Measure it before assuming a tier; the honest first answer is that panels drop to a smaller blur or go opaque whenever the canvas is live.
3. **A shader that hangs a driver.** The recovery path blanks every accelerated window on the machine. Four defences, all necessary, and none of them a guarantee.
4. **Linux lies about what it is running on.** Context creation succeeds on a software rasteriser with no error, and the renderer string is masked. Only a timed calibration render is honest.
5. **The render checks may not run in CI.** CI runs, with the launcher's existing checks in it, and is green on `main` since 5 October 2026, but C3, C4 and C7 are not written, and they need a headless WebGL2 context that has not been demonstrated.
6. **The node editor is a twelve-month thing to be good at.** Shipping it early as knobs-plus-source is fine; pretending a rushed graph editor is the authoring story is not.
7. **Idle motion has to stay ambience.** Since ADR-080 the design system allows looping animation on every surface, so a canvas that moves while the interface is idle contradicts nothing. What must hold is what ADR-080 keeps: reduced motion stills it, Still pauses it, and text over it stays readable. An idle slow-down rather than a stop keeps it ambience and not feedback.
8. **Naming.** A launcher-internal directory called `qfx/` is internal. A **published theme-asset type** is read by outside creators and toolmakers, so its SDK- and chain-facing identifiers take plain names under ADR-032 and need the owner's approval.

## Decided by the owner, 22 September 2026

Every question this blueprint asked was answered that day. Each answer is the owner's and is reversible.

1. **The narrowing of `check-design.mjs`** — one directory, two rules, made with layer one (`42d00c3`) — is
   **approved, retroactively**, with the rule restated: the design system governs the default theme and the
   chrome; QFX governs the canvas; the default must still pass `check-design.mjs` unchanged. Logged in
   `GATES.toml`; ADR-051 carries the rule. *Since 6 October 2026 ADR-080 has removed the rules this narrowed,
   so there is no exemption left.*
2. **The canvas may move while the interface is idle**, under that rule. `DESIGN_SYSTEM.md` §6 says so.
3. **No brightness cap; the scrim is the guarantee.** Its level was left to be set so the contrast check passes
   with margin, and recorded: **0.90**, with text over any backdrop at 10.87:1 or better for body, 5.54:1 for
   muted and 4.61:1 for faint, against a floor of 4.5:1 (ADR-051, decision 14).
4. **The frame budget is accepted**: at most 25% of the frame at the top tier, 15% at the lowest. v0's one-step
   budget is accepted for v0; the tiers are P2.2.
5. **`--ink-faint` is fixed now**, as an accessibility defect that predates QFX (`320be99`). It reads at 4.77:1
   or better on every opaque background and 4.61:1 over any backdrop, so it may be used over the canvas.
6. **Audio: system sound first, the microphone later, neither in v0.**
7. **Shader code is covered by the DRC-369 fingerprint**, because the same asset must not run different code.
   ADR-047's manifest does it by construction: a theme's shader is one of its entries.
8. **The theme asset type is "QFX Theme"** until the owner names it.
9. **The two neon-leaning themes, Veridian and Abyss, become QFX themes and are never the default.** They move
   when the theme package exists (P2.5).

Two further rules came with ADR-051's acceptance: **behind the Gate only the built-in default ambience may run,
never an installed theme**, because the Gate is where a passphrase is typed and a theme is code from strangers;
and **Still is the pause** that WCAG 2.2 SC 2.2.2 asks for, so there is no separate pause control. Scene-as-theme
waits until CI runs and the signing dialogs are verified by a person (the owner's QOR Engine decision).
