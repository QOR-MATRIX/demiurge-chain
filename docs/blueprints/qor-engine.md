# QOR Engine

**Status:** Blueprint, 21 September 2026. Describes intent, not code that exists. No line of QOR Engine has been written, and nothing below is a commitment to a date.

**Since 22 September 2026:** ADR-047, the object model and DRC-369's wire format, is accepted and M2.3 is ticked, and M2.1 was ticked the same day against the owner's review of a ten-line summary of the inventory's DRC-369 section. Wherever this document says something waits on "M2.1 and M2.3", it now waits on neither, and where it calls the content fingerprint undecided, it is a BLAKE3-256 manifest root with its algorithm tagged.

## What it is

QOR Engine is the place you build the thing — a 3D and 2D creation tool where a scene, a character, a level or a piece of interactive art is made, played and shipped. It is a custom build of Godot with Demiurge's own layer added: sign in as yourself, pull an asset in and have its creator paid when you use it, version your project, and publish without leaving the editor. **QOR Engine, built on Godot.**

## Who it is for

- **The creator who already uses Godot** and wants the part Godot has never had: a way to be paid when their work is used, and a way to use someone else's work with the payment and the permission handled for them.
- **The creator arriving from Blender, Maya or 3ds Max** with a folder of models. They need import that works (glTF, FBX-read) and export that is honest about what survives.
- **The Demiurge creator who is not a game developer** — a musician building a reactive visual, a designer making a launcher theme that is a scene rather than a stylesheet, an artist assembling a remixable world from other people's parts.
- **Not, in the first year, the studio shipping a console title.** See "what full support does not mean".

## The foundation, and why

**The foundation is Godot 4.6, MIT, as a custom build that tracks upstream.** QOR's additions ship as engine modules, a Rust GDExtension and editor plugins, never as a hard fork. Godot's MIT licence permits a rebranded, commercially redistributed derivative; the obligation is to carry the copyright notice and licence text. Rebranded derivatives already exist (Redot, Blazium), and commercial creation tools built in Godot already ship (PixelOver, RPG in a Box, Material Maker, Pixelorama).

**Why this and not the alternatives** — ADR-001 says Demiurge stays boring on purpose where failure is silent and permanent, and spends the innovation budget where failure is loud and recoverable. A renderer, a scene graph, an import pipeline, a skinning solver, an audio mixer and an editor are all places where a bug is discovered months later by someone whose work is already wrong, and where nobody adopts Demiurge because the implementation is ours.

| Alternative | Why it lost |
| --- | --- |
| **From scratch on wgpu** | wgpu is a GPU abstraction, roughly the bottom sliver of what "engine" means. Everything above it — render graph, GI, animation, import, editor — is, on the research's own unmeasured estimate, several person-years to a demo and 15–20 to something a stranger would use, with the editor alone at 3–5. **Those figures are estimates; nothing measures them.** The only calibration available is public engine history: Godot in 2026, years into maturity, is still changing fundamentals — 4.6 made D3D12 the Windows default and Jolt the default 3D physics engine. Choosing this makes Demiurge an engine company that also has a chain. |
| **rend3** | Archived read-only, 7 June 2025. Recommending an unmaintained dependency for the most expensive subsystem in the product is not a trade-off, it is an error. |
| **Bevy** | Genuinely good Rust and ECS, and a good fit with the launcher's Rust host. But through 0.19 (June 2026) there is still no production editor — `bevy_editor_prototypes` remains a prototype with a public roadmap — and a shared scene editor is precisely what QFX layer three requires. (The "no official editor" characterisation is from secondary 2026 write-ups; the prototype roadmap is primary.) Bevy is the right answer to "I want a Rust game framework" and the wrong answer to "I need a scene editor next year." Reconsider only if QOR Engine's scope collapses to a headless runtime with no authoring. |
| **Unity or Unreal as the base** | Closed source, cannot be rebranded, cannot be embedded. Unreal's EULA restricts distribution of "Engine Tools" to end users and routes public distribution through Epic's channels, which forbids the shape of the product outright. Also fails ADR-015's infrastructure-ownership posture. |
| **Godot beside the launcher, never embedded** | Rejected as the end state, **accepted as step one.** It is what 4.6 supports cleanly today; it cannot satisfy "a scene can be a theme", so it is a stage, not an answer. |

**Custom build, not fork, is the operative word.** A fork is a debt that compounds every release. A custom build that tracks upstream pays a rebase cost per minor version and keeps every upstream fix — including the ones for driver bugs nobody on this team could have found.

## What is new

Everything Godot does not have, and nobody else has either. **None of the six exists, because no line of QOR Engine has been written.** What they stand on is partly there and not blocked: QOR ID, DRC-369 minting (M4.1) and M4.2's royalties, remix royalties and nesting exist on the chain; payment when a part is used rather than sold (access gating, M8.2) does not, and agent rails are M5.2 and unbuilt. What is new is therefore a plan, not a differentiator anyone can use yet.

1. **An asset is a live, owned thing inside the scene.** A DRC-369 asset dropped into a scene carries its creator, its licence terms and its remix lineage as part of the node, not as a note in a README. Nesting one inside another is the authored act that creates the royalty relationship (M4.2).
2. **The creator of a part is paid when the part is used**, not when a marketplace listing is clicked. That is ADR-002's spend test applied inside an editor: the demand that matters is the demand to spend CGT on using work, never the demand to hold it.
3. **Remix is a first-class editor operation.** Open someone's scene, change it, publish the change, and the lineage and the splits follow the nested nodes — because `.tscn` already records nesting and inherited-scene overrides precisely.
4. **Identity is the creator's, not the tool's.** QOR ID signs in (ADR-016, ADR-043); the vault signs; the engine never holds a key.
5. **Agent rails inside the editor** (M5.2, ADR-014): an agent gets a scoped, revocable delegated key with a protocol-enforced spend cap, and can generate or assemble inside your project without ever being you.
6. **The scene is a publishable, versionable artifact by construction**, because `.tscn` is text — diffable, hashable, and mergeable with a semantic tool. That property is a requirement of GNOSIS's project format too, with `.tscn` cited as the precedent.

## Architecture

```
                 QOR Launcher (Tauri 2, Rust host, unsafe_code = "forbid")
                 ├── Vault ........... the only thing that signs (L1.4 confirmation)
                 ├── QOR ID session
                 ├── Qontrol ......... gitoxide for every read path;
                 │                     libgit2 in a sidecar process for stage and
                 │                     write-tree only, never linked into the host
                 └── subxt chain client (L3.1, ADR-040)
                          │  IPC, out of process. Engine is OUTSIDE the vault's trust boundary.
                          ▼
   QOR Engine = custom Godot 4.6 build ── tracks upstream, additions layered:
     ├── engine modules (C++) ......... only where an editor plugin cannot reach
     ├── GDExtension (Rust, gdext 0.5)  QOR ID identity, sign-requests to the vault,
     │                                   DRC-369 nodes, Mesh fetch, Qontrol status
     └── editor plugins (EditorPlugin / @tool)
           ├── QOR ID panel            ├── DRC-369 asset library
           ├── Qontrol panel           ├── Publish to Market
           ├── Agent rails             └── QFX shared scene editor  ◄── layer three
```

**The shared scene editor for QFX layer three is an editor plugin in this build, not a second tool.** One scene editor serves both products. Building it twice, or building it at all when an MIT one exists, is the clearest ADR-001 violation available.

**Three seams, each stated once:**

- **Process boundary.** An engine instance executes project content: GDScript, GDExtension binaries, downloaded assets. It must never sit inside the vault's trust boundary, and naïvely linking a C++ engine into the launcher host does not survive `unsafe_code = "forbid"`. Godot's own editor already runs the game out of process so a crash cannot take the editor down; QOR Engine copies that posture toward the vault. It is the same reasoning that puts libgit2 in a Qontrol sidecar rather than in the host that holds the keys.
- **Window composition.** LibGodot (merged 8 October 2025, in the 4.6 milestone) gives a C ABI — `libgodot_create_godot_instance()`, a `GodotInstance` with startup and per-iteration control — with two limits: **one instance per process, and editor functionality is not in the library build.** Rendering a native GPU surface inside a WebView2 webview is not supported. So the first shippable integration is a **native child or sibling OS window composited with the webview**, not an in-webview canvas. Upstream is walking toward the better answer (godot-proposals #14435 host-provided rendering surface; Migeran's `DisplayServerEmbedded` and `Window.set_native_surface()`; the GodotCon Amsterdam 2026 talk names host-UI embedding and restart-without-exit as next patches) — but nothing here is scheduled against an upstream patch that has not landed.
- **Signing.** Every signature request originates in the engine and is answered by the vault behind the host-side confirmation of L1.4 and ADR-016. The engine sees a result, never a key.

**Interchange, as it actually stands:**

| Format | Read | Write | Reality |
| --- | --- | --- | --- |
| **glTF 2.0 / GLB** | Core, runtime (`GLTFDocument`) | **Core, runtime — mandatory** | Khronos, royalty-free, ISO/IEC 12113:2022. The floor for every 3D artifact. A delivery format: it does not carry modifier stacks or procedural history. |
| **FBX** | Core since 4.3 via `ufbx` (MIT, clean-room) | **Never** | There is an open FBX reader and no permissively-licensed writer; the Autodesk SDK is binary-only under its own licence agreement. An FBX that opens wrong is worse than no FBX: the creator believes they left with their work. |
| **OpenUSD** | **Not in Godot core**, for a stated reason: the API is 200+ MB, too large for an editor people expect to download quickly. Three GDExtensions exist (`meshula/usd-godot`, `tefusion/godot-usd`, V-Sekai `idtx-flow`); none is a shipped, supported path | Integration work | Licensed under the Tomorrow Open Source Technology License 1.0 — Apache 2.0 with §6 (Trademarks) changed, so permissive, but never put "Pixar" or "USD" in a product name. AOUSD Core Spec 1.0 is ratified; 1.1 and ISO certification are in progress. This is the scene-composition answer and it is a twelve-month integration, not a checkbox. |
| **Audio samples** | WAV, Ogg Vorbis, MP3 in core | Add **FLAC (RFC 9639)** for archive and **Opus (RFC 6716)** for delivery | Both fully open and agreement-free. Master in WAV/BWF, archive FLAC, deliver Opus. No lock-in anywhere on this row. |
| **MIDI** | **Input only** in core (`InputEventMIDI`); SMF file playback is addon territory today | **Write SMF (MIDI 1.0)** | SMF is the most portable music format in existence and carries notes, timing, tempo and CCs — no audio, no plugin state, no mix. MIDI 2.0/UMP is real on the wire; its *file* story has nothing like SMF's universality. Do not make a MIDI 2.0 clip file an exit format. |
| **Shaders** | GLSL, WGSL, SPIR-V, MaterialX | **SPIR-V + MaterialX + the authored source, together** | `.gdshader` has no portability — it is a Godot-only dialect. The engine format is a build output; it is never the only form of the asset. |
| **Themes** | DTCG 2025.10 JSON | DTCG 2025.10 JSON | The launcher's `themes.ts` token structure maps onto it almost mechanically. Cheapest lock-in elimination available. |
| **Godot `.tscn` / `.scn` / `.res`** | Native | Native | The project format. Round-trip is not a feature we build; it is the file format we already use. |

Two rows carry an honest caveat: **USD's and MIDI's absence from Godot core is an absence of evidence in the 4.6 documentation, not an upstream statement saying "no".** Both are treated as absent and needing integration work.

**Why Godot round-trip is native and Unity or Unreal round-trip is not achievable at any budget:**

A `.tscn` round-trip is not "read the text". It is reconstructing every node type's exported properties, every resource's serialisation, every signal connection, every inherited-scene override and every script with the same meaning. The only implementation of those semantics is the engine. Because QOR Engine *is* the engine, we get it for free, and the third-party semantic merge tooling that already exists for `.tscn` (gdmerge) applies to Demiurge projects unchanged.

Unity and Unreal fail on four independent grounds, any one of which is fatal:

1. **A project is behaviour, not data.** A Unity scene's meaning lives in `MonoBehaviour` subclasses; an Unreal level's lives in Blueprint graphs and C++ classes. Round-trip would mean translating arbitrary user code between runtimes with different lifecycles, tick semantics, physics steps and object models. That is program translation, and it is undecidable in general.
2. **References are engine-private identity.** Unity identifies by `(GUID from the .meta file, fileID inside the YAML)`, where `--- !u!1 &6` means class 1, object 6 — an internal type registry that versions with Unity. Prefab instances serialise as overrides, not contents. Godot identifies by node path, UID and, in 4.6, `unique_id`. Any mapping is lossy both ways and breaks on the next minor release of either engine.
3. **Unreal's project data is undocumented binary.** `.uasset` has no public specification; what exists is community reverse-engineering. A product built on it breaks on every UE release, silently — the exact column ADR-001 says to stay out of.
4. **The licence forbids the product shape.** Unreal's EULA restricts distribution of Engine Tools to end users and routes public distribution through Epic's channels. (Clause detail here is from secondary summaries; the primary text refuses automated fetch, and a lawyer reads it before any public claim.)

**So the interop story, said in public copy from day one: assets in and out, projects never.** The alternative is a promise broken in year two.

## Substrate consumed

| Substrate | How QOR Engine uses it |
| --- | --- |
| **QOR ID** | The only identity. Sign-in happens in the launcher and the session is handed to the engine read-only. **QOR Engine never gets a login screen of its own.** Godot has no account concept, which is convenient: the gap is avoided by not filling it. |
| **DRC-369** | Assets, scenes, sub-scenes and projects are DRC-369 tokens. Nesting a node is what creates the royalty relationship. **M4, started and not blocked:** M2.1 and M2.3 were ticked on 22 September 2026, M4.1 is done, and M4.2's royalties and nesting are built; state and XP are not. |
| **Qontrol** | Versioning for every project, including the `.tscn` semantic merge driver and the large-file side store. The engine calls Qontrol's interface; it never links a git implementation of its own, and it never speaks to libgit2 except through Qontrol's sidecar. |
| **Mesh** | Asset and project delivery, and dependency fetch. **M8.1, and nothing before it.** |
| **CGT** | Settlement for every paid operation, once there is anything to settle in. |

**Substrate gaps — recorded, not filled locally:**

1. **Project and scene identity.** DRC-369 must be able to name a *project*, a *scene* and a *nested sub-scene*, and it must relate a scene's identity to a content fingerprint. Those were wire-format questions for M2.3; ADR-047 (accepted 22 September 2026) answers them as assets and nested assets (decision 7 at the end of this document), and ADR-025 records that token identity is not preserved from the old module. **QOR Engine must not mint a project id of its own.** If it writes an interim local id, that id is marked in the file as a placeholder and carries a stated migration path, or it becomes the format.
2. **Content fingerprint.** ADR-025 assigns the fingerprint to the DRC-369 pallet, and the migration inventory puts content off chain on the Mesh (M8.1). Which algorithm it is was an M2.3 decision, not the engine's: BLAKE3-256 with its algorithm tagged (ADR-047, 22 September 2026). One fingerprint serves the chain identity and the content store; a second scheme invented in the editor is a gap created on purpose.
3. **Distribution and storage.** The Mesh is M8.1 and does not exist. An engine that publishes projects and fetches dependencies needs storage today and cannot have it. **No CDN, no S3 bucket, no package registry of QOR Engine's own.** The honest first slice has no remote at all and says so in the interface.
4. **Exclusive locks.** Art teams need exclusive checkout on files that cannot be merged, and nothing in the substrate holds a short-lived exclusive claim by an identity over a path. QOR ID authenticates the claimant; where the claim lives is unsolved, and chain state is the wrong shape and the wrong latency for a lock taken for twenty minutes. **Name it and stop:** a local advisory lock is acceptable and must be labelled as local.
5. **Licence vocabulary.** Nothing anywhere defines what a licence *term* is. QOR Engine needs remix terms, Market needs resale terms, Library needs entitlement terms, Stream needs per-play terms. Four products inventing one is four vocabularies for one concept. **This needs an ADR before any of the four is built.**
6. **The C2PA assertion shape.** An exported `.glb` or `.flac` carries no signed statement of which QOR ID made it or which account royalties settle to. C2PA is the existing open standard that can carry it, and **the assertion shape is one substrate decision across every product**, related to ADR-028's provenance source — not a per-product choice made inside an editor.
7. **Qontrol's number arrived after its code.** Qontrol's Projects surface and staging sidecar (`5a44207`) landed before it had a roadmap number; since 21 September 2026 it is P1, with a `SYSTEMS.md` entry, and still has no accepted record defining it. QOR Engine's versioning depends on it, and needs that scope document before the engine saves a project twice.
8. **Settlement, and every rate it would need.** The chain's runtime today (`spec_version` 8) mounts System, Timestamp, Aura, GRANDPA, Balances, Session, ValidatorSet, Nfts, Drc369, Utility, Drc369Royalties, ArqWallet and feature-gated Sudo — assets and a sale settled in CGT with royalties, but no fees, no issuance and no treasury. Each paid operation names its ADR-006 sink: Mesh hosting of project dependencies → Mesh hosting; unlocking a licensed asset in the editor → access gating; publishing to Market with reach → staking for distribution (U-7); in-editor agent generation → agent compute (U-9). **The rates are not this blueprint's to pick.** The burn share per fee class and the `WeightToFee` that M6.4 needs are **OPEN-4**; the perpetual issuance rate is **OPEN-1**; the genesis allocation split is **OPEN-2**. All three are undecided, no number for them appears here, and any test that needs one uses a value marked as a placeholder.

## The first usable slice

**A creator opens QOR Engine, is already signed in as themselves, imports a model, builds a scene, and commits it — with the project's history readable and the identity real.**

Concretely, and nothing more:

- A rebranded Godot 4.6 build on Windows, Linux and macOS, with upstream tracking established and one rebase already performed so the cost is measured rather than assumed.
- **QOR ID identity visible in the editor**, read-only, via the Rust GDExtension. No signing yet, no chain writes.
- **glTF and FBX in, glTF out, proven end to end** on a fixture set, with the read-only FBX rule enforced in the UI: the export menu does not offer FBX.
- **Qontrol's panel in the editor**: status, log, diff and commit, through the same interface the launcher's Projects surface uses — gitoxide for the reads, the libgit2 sidecar for stage and write-tree. **The `.tscn` semantic merge driver belongs to Qontrol, not to the engine**, because Godot mints fresh random sub-resource ids per save and two people each adding a texture collide on the same line.
- **The engine launched from the launcher as a separate process with its own window.** Not embedded. Honest, and it works today.

That is worth opening. What it deliberately is not: no minting from the editor, though DRC-369 minting exists in the launcher (M4.1); no Mesh fetch, because the Mesh does not exist; no purchases from the editor, which waits on Phase 3. CI runs on GitHub Actions on the public repository (ADR-063) and is green on `main` since 5 October 2026; an engine build job would join it, and nothing here is verified until one does.

## Phases to a full product

**What is reused, what is new, and what actually lands.** The bands below are the honest frame; the phases after them are the same content in order. The research's figures assume one to two engineers. **This project is one founder plus an agent, and that changes the new column, not the reused one:** reused work is someone else's, so it arrives at full size, but every new item competes for the same single reviewer, and anything needing a person at a running launcher — L1.4's native dialogs, a signed installer, a platform-specific crash — does not parallelise at all. Treat every duration as an estimate with nothing measuring it.

| Band | What lands |
| --- | --- |
| **Reused, zero engine work** | Renderer (Forward+, Mobile, Compatibility over Vulkan, D3D12 and Metal), Jolt physics, audio mixer, skeletal animation, glTF and FBX import at runtime, glTF export, the scene format, the full editor, export templates, XR. All MIT. |
| **New, unavoidable** | Rebrand and upstream-tracking discipline; the Rust GDExtension bridge (`gdext` 0.5) carrying the QOR ID session, vault-mediated signing and later DRC-369; the `.tscn` semantic merge driver (in Qontrol); the Qontrol editor panel; C2PA manifest writing; USD and MIDI if wanted, as integrations. |
| **~3 months, one founder plus an agent** | The rebranded build on three platforms with one rebase performed; the GDExtension exposing read-only QOR ID identity; glTF and FBX in and glTF out proven on fixtures; the engine launched from the launcher as a sibling window. **The Qontrol panel and the merge driver land at the end of this band or the start of the next, not inside it.** |
| **~12 months** | Vault-mediated signing from the editor behind L1.4; the QFX shared scene editor as an editor plugin; scene-as-theme as a signed, hash-pinned artifact that degrades to the CSS themes; DTCG theme read and write; FLAC, Opus and SMF; shader packages; the merge driver exercised in CI; DRC-369 assets as live nodes, on M4.1 and M4.2's built halves; the agent rails panel on M5.2. |
| **Beyond twelve months, or blocked** | Mesh-backed delivery and publish (M8.1); entitlements (M8.2); staking for distribution (M8.3); settlement inside the editor (M6.4, and OPEN-4 before it); host-surface embedding inside the webview (waiting on upstream #14435); consoles (a porting partner, on commercial terms). |
| **Not achievable, at any budget** | Unity or Unreal **project** round-trip. |

**Phase 0 — the build exists (weeks 1–4).** Rebranded Godot 4.6 building reproducibly on three platforms, upstream tracking documented, trademark and attribution text reviewed, one rebase performed. Deliverable: an installer a stranger could run.

**Phase 1 — identity and versioning (the 3-month band).** The slice above. Rust GDExtension on `gdext` 0.5; QOR ID read-only; the Qontrol panel; glTF and FBX proven.

**Phase 2 — the scene as a Demiurge artifact (the 12-month band, first half).** Vault-mediated signing from the editor, behind L1.4's host confirmation. Scene-as-theme as a hash-pinned, signed artifact rendered as launcher chrome — **with a hard requirement that it degrades to the current CSS themes when the GPU path is unavailable or when the contrast or motion preference says no**, or QFX regresses a shipped accessibility commitment (L1.2). DTCG 2025.10 theme read and write. FLAC and Opus added. SPIR-V, MaterialX and source shader packages.

**Phase 3 — assets are owned (the 12-month band, second half; gated on M4).** DRC-369 assets as live nodes in the scene; the asset library panel; nesting that creates a royalty relationship; remix lineage visible in the editor. **None of this starts before M4.1 and M4.2 land.** M4.1 landed on 22 September 2026, and M4.2's royalties, remix royalties and nesting are built; its state and XP are not. Agent rails panel on M5.2 and the single MCP server of ADR-010 and M5.3. **No operation in this phase costs anything, because no fee class exists (OPEN-4).**

**Phase 4 — publishing and delivery (beyond twelve months).** Mesh-backed dependency fetch and project publish (M8.1); entitlements (M8.2); Market publish with staking for distribution (M8.3); settlement inside the editor (M6). C2PA manifests written into every export, to the substrate-wide assertion shape, so an artifact proves its own origin without Demiurge existing.

**Phase 5 — embedding proper.** The engine renders into a host-provided surface inside the launcher rather than a sibling window. **Waits on upstream (#14435); not schedulable.**

## Dependencies on chain and launcher milestones

**Blocking, chain:**
- **M2.1** — the owner reviews `MIGRATION_INVENTORY.md`. **Ticked on 22 September 2026**, so it no longer blocks anything.
- **M2.3** — the DRC-369 wire format: field bounds, token identity, fixed-point encoding, rental time unit. **Project, scene and sub-scene identity and the content fingerprint are decided here**, not in the engine. **Ticked on 22 September 2026** (ADR-047); Q-19 and Q-20 stay open before the format freezes.
- **M4.1, M4.2** — ownership, mint, collections, fingerprint; nesting with bounded depth, and royalties with remix royalties settled in CGT. **Phase 3 is entirely gated on these.** M4.1 is done; M4.2's royalties and nesting are built, its state and XP are not.
- **M4.4** — sponsored fees and deposits, so a creator can publish their first asset without holding CGT first. Without it, the first run requires the creator to acquire CGT before they can do anything, which is the wrong first impression for a tool about being paid for use.
- **M5.2, M5.3** — delegated agent keys with scoped capabilities and spend caps (ADR-014); the MCP server. The editor's agent panel is a client of these, never its own rail.
- **M6.4** — transaction payment with a burn-share handler per fee class. It needs a `WeightToFee`, which is **OPEN-4**. Until then no in-editor operation can cost anything, and none is designed as though it could.
- **M8.1, M8.2, M8.3** — the Mesh, entitlements, staking for distribution.

**Blocking, launcher:**
- **L1.2** (done) — accessibility settings that take effect. Scene-as-theme must honour the contrast override and the motion preference, or it breaks a shipped commitment.
- **L1.4** — host-side confirmation before any signature. Every engine-originated signature routes through this. Implemented and unit-tested; unchecked until the native dialogs are exercised in a running launcher.
- **L1.6, L1.7** — the launcher's host tests in CI, and CI on `main` starting and passing. **CI runs**, on GitHub Actions on the public repository (ADR-063), and CI on `main` is green since 5 October 2026. An engine build job would be evidence once it joins that pipeline.
- **L3.1** (done) — the `subxt` client against runtime metadata (ADR-040). The engine talks to the chain through the launcher's client. It never opens its own.
- **L4** (depends on M4) — Studio and assets. The engine's asset library is L4's surface rendered in the editor, **not a second asset system**.
- **L6.1, L6.2** — signed installers and a verifying update channel. A branded engine distribution without these is unshippable to strangers.
- **L7.5** — Studio inside the launcher, which is where the embedded path lands.

**Not blocking but shaping:** L1.1 removed a particle field and a pointer light, and "a scene can be a theme" invites exactly that class of thing back. Since ADR-080 (6 October 2026) it is allowed: glow, neon, gradients, particle and shader backdrops and pointer-reactive light are permitted on every surface. What a scene-as-theme still owes is what ADR-080 keeps: reduce motion stills it, text over it stays readable as painted, and it is recognisably Demiurge's rather than a template's. That goes into the QFX brief on day one.

## What "full support" means, and what it does not

**It means:** glTF 2.0 in and out, losslessly, as the guaranteed artifact for every 3D thing. FBX in. Open audio in and out. SMF out. Shader packages carrying SPIR-V, MaterialX and the authored source together. Themes in DTCG. Godot projects round-trip natively, because it is the same engine. Three rendering methods over Vulkan, D3D12 (the Windows default in 4.6) and Metal. Jolt physics. XR. All MIT, all reused, zero engine work.

**It does not mean:**

- **Unity or Unreal project round-trip. Not partially, not later, not at any budget.** Assets move. Projects do not. Existing community migration tools plus glTF and USD are the whole story, and they are an assets-only story.
- **That every QOR Engine project opens in stock Godot.** A project using only upstream features does. A project using QOR's nodes opens in stock Godot with those nodes missing unless the GDExtension is installed — which is how every GDExtension behaves, and it is a limit, not a bug. The promise is that your scenes and assets are standard files, not that Demiurge's layer is in upstream.
- **FBX export.** We read FBX as a courtesy to creators arriving from Maya and 3ds Max. We never write it.
- **Unreal's rendering.** Godot's renderer is not Unreal's, and there is no plan for it to become Unreal's. **That is fine, because photorealistic AAA rendering is not the target** — the target is a creator who wants to be paid when their work is used. Anyone who needs Nanite and Lumen should use Unreal and bring their assets to Demiurge as glTF.
- **OpenUSD in the first year.** Not in Godot core, for a stated reason. The GDExtensions are unshipped and unverified for production. Treat USD as a twelve-month integration decision, not a feature.
- **`.tscn` merging itself automatically.** Text and diffable is not the same as mergeable. Godot mints fresh random sub-resource ids per save, and 4.6's new `unique_id` and `owner_uid_path` fields add a fresh diff-noise source when a GLB is reimported. A semantic merge driver is required, it lives in Qontrol, and shipping and testing it is our job.
- **A verified claim about web or mobile exports carrying QOR's additions.** That has not been tested, and until it is, nothing about it is promised.
- **Consoles.** Godot has no first-party console export. That path runs through a porting partner, on commercial terms and platform NDAs.
- **Support for Godot itself.** A user of QOR Engine cannot file an upstream bug for QOR's additions, and we cannot promise fixes in upstream code on our schedule. The branding must not imply Godot endorses, supports or maintains this build. **The line is "QOR Engine, built on Godot" and nothing stronger.**
- **A working in-editor marketplace on day one.** The chain has assets and a sale settled in CGT, but no fees and no issuance. Everything paid is Phase 3 and later, and every rate it would need is OPEN-1, OPEN-2 or OPEN-4 and undecided.
- **An engine that holds your keys.** It never will. The vault signs.

## Risks

1. **Upstream drift.** A custom build tracking upstream costs a rebase per minor release, and the cost grows with every C++ module added that an editor plugin could have done instead. **Mitigation: a standing rule that additions go in editor plugins and GDExtension unless a written reason says otherwise, mirroring ADR-013's rule about departing from a standard component.**
2. **Scheduling against unlanded upstream work.** Host-surface embedding is a proposal, not a patch. If Phase 2 or the QFX brief assumes it, the plan is fiction. **Mitigation: the sibling-window path is the plan; embedding is an improvement, never a dependency.**
3. **The engine is a large attack surface next to a vault.** It executes GDScript, loads GDExtension binaries and opens files from strangers. **Mitigation: out of process, always; a stated policy for executing code in downloaded content; the vault reachable only through a narrow signing IPC.**
4. **Scene-as-theme regresses the accessibility work, or turns the look generic.** Effects are allowed since ADR-080, but L1.2 shipped a contrast and motion commitment that a scene must keep. **Mitigation: CSS themes remain the floor and the fallback; the scene path is additive and must degrade, and reduce motion and readability as painted bind it.**
5. **DRC-369 is only partly built, so the differentiating layer can be validated only in part.** Phases 0 to 2 deliver a good Godot build with identity and versioning and no Demiurge economics at all. M2.1 cleared on 22 September 2026, M4.1 is done and M4.2's royalties and nesting are built; state and XP (M4.2), sponsored fees (M4.4) and the Mesh (M8) are not, and Phase 3 and later wait on them.
6. **Trademark and confusion.** A branded Godot derivative draws bug reports meant for upstream and questions from the Godot community. **Mitigation: the verbatim line, visible attribution and licence text, an About panel that names upstream plainly, and contributing fixes upstream where they belong.**
7. **A green CI that does not build the engine.** CI runs and is green on `main` since 5 October 2026 (ADR-063), but no job builds QOR Engine. Until one does, every engine claim in Phase 1 is unverified, and a green run must not be read as covering it — calling something green that no job checked is a failure mode this project has already had once.
8. **Scope creep into a second engine.** The moment QOR Engine grows its own asset format, its own account, its own storage, its own lock service or its own payment rail, the substrate rule is broken and there are two of everything. **Mitigation: each of those is listed above as a gap, and a gap is recorded, never filled locally.**

## Decided by the owner, 22 September 2026

Every question this blueprint asked was answered that day. Each answer is the owner's and is reversible.

1. **A light rebrand: "QOR Engine, built on Godot".** The editor stays recognisable, so Godot's own documentation
   applies to it.
2. **Track the current stable 4.x line, pinned exactly**, and move the pin deliberately.
3. **A separate process in year one**, under ADR-046: a sibling window, not embedded.
4. **USD is read-only first.**
5. **No scene-as-theme until CI runs (L1.7) and the signing dialogs are verified by a person (L1.4).** P2.6 counts
   both.
6. **Downloaded content runs no native code without published source and a signature.** The details belong in
   Market's ADR, which is not yet written.
7. **No interim project id.** ADR-047 is accepted, so a project, a scene and a sub-scene are DRC-369 assets and
   nested assets when M4 exists; until then the engine writes no identity of its own.
8. **Unlocking an asset in the editor is the access-gating sink** (ADR-006). No rate is set.
9. **Contribute fixes upstream** to Godot, under Godot's terms.
10. **Answered in part:** Qontrol is P1 and QOR Engine P3; a record defining Qontrol is still missing.
11. **Projects must open in stock Godot**, and that promise is a gate criterion: P3.6, and a test the `qor-engine`
    gate requires, `a_project_opens_in_stock_godot`. QOR's additions therefore stay optional at load time, and
    its nodes degrade rather than fail without them.
