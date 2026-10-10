# QQ, the QOR Engine

An engine for virtual worlds and experiences, built on **Qt 6** under the owner's commercial licence
([ADR-083](../../docs/decisions/ADR-083-qq-on-qt.md)). The blueprint is [`docs/blueprints/qq.md`](../../docs/blueprints/qq.md);
the roadmap items are DIRECTION P3.1 to P3.6.

**Status, 10 October 2026: P3.1 and P3.2 done.** QQ Studio edits a 3D world, keeps it in a Qontrol project, and plays
it; the QQ Player plays a scene on its own, natively and in a browser.

- **The world:** a procedural sky that also lights the scene, a sun with soft shadows, lamps, filmic tonemapping, bloom,
  ambient occlusion and multisampled edges; shapes with physically based surfaces; glTF 2.0 models loaded at runtime.
- **Play (P3.2):** shapes with bodies (`body`: none, static or dynamic; mass, bounce, friction) that fall, collide as
  their shapes and come to rest, under the scene's gravity; a **Player**, a character controller that walks, runs, turns
  and jumps, is stopped by walls and climbs low steps, with a camera that follows; **input** from the keyboard (WASD or
  the arrows, Space, Shift), the mouse (a drag looks around) and a gamepad (XInput: sticks past a dead zone, A, the
  shoulders), read as one set of intentions (`Input`: move, look, jump, run) so every game works with every device and
  the agent can play one too; **particles** (`Emitter`: soft points of light that rise, spread and fade); **sound**
  (`Sound`: placed in the world, louder nearer, panned, silent beyond its reach, heard through the camera; in a browser
  not panned, below); and **logic** (`Behaviour` names a `logic/<name>.qml` file whose root is a `Logic`; it runs only
  in play and is rebuilt each time the file is saved, without stopping the world).
- **The Studio:** the scene's entities in a list, the selected one's fields in an inspector (built from what each type
  declares), click to select in the viewport, handles to move along an axis, turn about the vertical or scale evenly,
  orbit and zoom, add (cube, sphere, cylinder, cone, lamp, model, player, emitter, sound, behaviour) and delete.
  **Play** (F5) plays a copy built from the scene's canonical text; **Stop** (Esc) throws the copy away, so the scene,
  its unsaved state and the selection are exactly as they were. **New behaviour** writes a starter logic file into the
  project, never over one that is there.
- **The files:** a scene is `scenes/<name>.qml` in the project, in one canonical layout: read and written back it is
  the same bytes, and one changed value is one changed line in Projects. Models and sounds are copied into `assets/`
  and logic into `logic/`, so the project is whole on its own. 2D scenes from the launcher's preview (`.qq.json`) are
  imported, emitters included; motion and the pointer follow, which in 3D are logic, are named as not carried over.
- **The QQ Player:** plays one scene as a game, full window, natively (`qq-player [scene.qml]`, the bundled playground
  without one) and as a WebAssembly build in a browser, and measures how long its first frame takes.
- **The launcher:** **Open in QQ Studio** on the launcher's QQ surface starts the Studio on the project open there.

**P3.3 under way (10 October 2026): the Studio's tools for agents.** A running QQ Studio offers fourteen tools to
language models over MCP (the Model Context Protocol): read the scene and what can be added, add, change and remove
entities, write logic, play, stop, hold the controls, wait, place the eye, capture a frame, read the log. Any MCP client
reaches them through `qq-mcp` (below); what an agent changes shows in the Studio as unsaved, and only the creator saves
and commits. **And the design loop**, in the Studio's **Agent** panel: describe a game, and Claude (Opus 5.5, ADR-085)
writes a brief, builds it with the same tools, plays it, looks at captured frames, judges them and revises, every step
shown as it happens; the creator then saves and commits it, or undoes the run. The provider's key is kept in Windows
Credential Manager and never logged. **Generation is paid for by whoever uses it** (ADR-086): the key is the creator's
own, Anthropic bills it directly, the panel says so before a key is given and shows each run's token use, and Demiurge
holds no provider key and pays for no one's generation. Still to come in P3.3: its proof, a scene built from a
description with no human edit, made by whoever runs it with their own key.

## What is here

| Path | What it is |
| --- | --- |
| `runtime/` | The QML module `QQ`: `World` (the viewport, its sky, light, look and hearing), `Scene` (the root a file holds), `Game` (a world playing one scene file: what the Player shows), the entities `Sun`, `Lamp`, `Ground`, `Shape`, `Prop`, `Player`, `Emitter`, `Sound`, `Behaviour`, and `Logic` (what a logic file is). In C++: `SceneIO` (canonical writing, loading, saving, playing a copy, adopting files, importing 2D scenes, and the editing operations the Studio uses and the agent will, P3.3), `Input` (keyboard, mouse and gamepad as intentions), `LogicFile` (builds a logic file and rebuilds it on each save), `SoftDot` (the particles' sprite, made in memory) `gpu` (draw with the faster of two GPUs), `Confinement` (what a game may reach in the Player, ADR-087) and `WebVoice` (a sound in a browser, so a game never needs Qt Multimedia's QML) |
| `studio/` | QQ Studio. `qq-studio [model.gltf] [--project <folder>] [--capture <frame.png>]` |
| `agent/` | The agent's side of the Studio (P3.3): `McpServer` (MCP's JSON-RPC, served on a local pipe only this user can open), `StudioTools` (the fourteen tools, working the Studio's own window functions, and `StudioLog`, what the Studio has said), `DesignLoop` (a description designed, built, played and judged by Claude through those tools; ADR-085), `keychain` (the provider's key in Windows Credential Manager) and `qq-mcp` (the program an MCP client starts, relaying its standard input and output to the running Studio) |
| `player/` | The QQ Player. `qq-player [scene.qml] [--capture <frame.png>] [--measure <result.json>]`; its playground (a small game with everything P3.2 plays) bundled inside it; `measure-web.mjs`, which plays the browser build in a headless browser and measures it |
| `tests/` | `tst_world` (rendering, judged by pixels), `tst_scene` (scene files), `tst_studio` (the real Studio window, worked by clicks, drags and keys), `tst_play` (physics, the player, input, particles, sound and logic, in real frames), `tst_player` (the real Player executable), `tst_lockdown` (what a game's logic may reach in the Player, ADR-087), `tst_agent` (the tools over MCP, in the real Studio and through the real `qq-mcp`) |
| `tests/fixtures/` | `orb.gltf` (written by `make_orb.py`), `chime.wav` (written by `make_chime.py`), `logic/spin.qml`, `scenes/sample.qml` (every entity type, canonical), `first-light.qq.json` (the 2D preview's starter scene) |
| `build.ps1` | Builds with the owner's Qt, MSVC 2022 and Ninja and runs every test; `-Deploy` makes a self-contained Studio; `-Web` builds the Player for the browser and measures it |

## Build, test, deploy

Needs Qt 6.12 (MSVC 2022 64-bit kit, with Qt Quick 3D, Quick 3D Physics, Spatial Audio and Multimedia) and Visual
Studio Build Tools 2022 with the C++ workload. The browser build also needs Qt's WebAssembly kit (multithreaded), the
Emscripten SDK that kit names (5.0.5 for Qt 6.12, in `%LOCALAPPDATA%\emsdk` by default, `-Emsdk` to change it), Node.js
and a Chromium-family browser.

```powershell
pwsh products/qq/build.ps1           # configure, build, run every test; non-zero if anything fails
pwsh products/qq/build.ps1 -Run      # build, then open QQ Studio
pwsh products/qq/build.ps1 -Deploy   # build, test, then put a self-contained QQ Studio where the launcher looks
pwsh products/qq/build.ps1 -Web      # build the QQ Player for the browser, play it in one and measure it
```

The build goes to `%LOCALAPPDATA%\qq-build` (`-Build` to change it); the deployed Studio to `%LOCALAPPDATA%\qq-studio`
(`-DeployTo`), with the Qt libraries, plugins and QML modules it uses and the Microsoft C++ runtime installer: about
1,520 files and 182 MB (113 MB before P3.2 brought physics, audio and Qt Multimedia), runnable on a machine with no
Qt. The launcher's **Open in QQ Studio** starts that copy, or the one the `QQ_STUDIO` environment variable names.

The browser build goes to `%LOCALAPPDATA%\qq-wasm` (`-WebBuild`): the page, `player/qq-player.html`, with
`qtloader.js`, `qq-player.js` and `qq-player.wasm` beside it. **Whatever serves it must send
`Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`**: the build uses threads,
and a page gets the shared memory threads need only when it is cross-origin isolated (ADR-084).

The tests open real windows and render on the GPU, so they need a desktop session. `QQ_SAVE_FRAMES=<folder>` keeps
every frame `tst_world` judges.

## The Player, measured (10 October 2026)

On the owner's laptop (RTX 4060 Laptop GPU), playing the bundled playground at 1280 × 720:

| | Native | In a browser (Chrome, headless) |
| --- | --- | --- |
| First frame proper (the scene playing, every model loaded) | 1.4 s from the process starting (1,371 and 1,438 ms in two runs) | 5.8 to 6.0 s from the page starting to load, cold, served from this machine (three runs: 5,953, 5,812 and 5,779 ms; `main()` begins at about 1.9 s) |
| Frames a second over the next three seconds | 229 and 345 | 220 to 227 |
| Size | the executable, 0.55 MB, with Qt's libraries beside it | 44.1 MB as served (the `.wasm` 43.8 MB); 15.9 MB gzip, 11.7 MB brotli |

The browser's first frame includes compiling 43.8 MB of WebAssembly; across a network it is longer by the download.
Chrome must be given the faster GPU (`measure-web.mjs` asks for it): on the laptop's integrated Intel GPU the same build
drew about 12 frames a second.

## The tests, and how they were proven

- **`tst_world`** (3 cases): a glTF model loads at runtime and is drawn lit in its own colour; bloom lights pixels past
  the glowing band; an unreadable model is an error, not a crash. Proven to fail by two planted faults (bloom ignored;
  a prop that never loads).
- **`tst_scene`** (14 cases): numbers are written one way only; a scene file reads and writes back byte for byte; one
  changed value is one changed line; a saved scene keeps its model relative and loads again; a scene saved elsewhere
  still points at its model (including across drives, where no relative path exists); adopting a model brings its
  files and never overwrites; sounds are adopted into `assets/` and logic into `logic/`; playing is a copy, and the
  scene is not touched; what is not a scene is refused with a reason; entities are added and removed and the file
  follows; a project lists its scenes; the bundled sample can be adopted; a 2D scene imports as a canonical one; a
  file of another format is not imported.
- **`tst_studio`** (10 cases, the real window): a new Studio opens a lit scene in its project; a click on the model
  selects it and a click on the sky selects nothing; dragging the X handle moves along X only; dragging the centre
  handle scales evenly and leaves the position alone; saving writes a canonical scene and carries the model into the
  project, and one edit saved again is one changed line; a saved scene opens again as it was; entities are added,
  named uniquely and deleted; a 2D scene is imported with what cannot come yet said; **Play then Stop**: Play from its
  button, a crate falls and rests while the scene's crate stays put, W walks the player and the eye follows, a drag
  turns the player and tilts the eye, Esc leaves the scene, its unsaved state and the selection exactly as they were,
  and Play starts again from the same place; a new behaviour is written into the project and never over a file.
- **`tst_play`** (11 cases, real frames): a dynamic shape falls and comes to rest on the ground; a static one holds
  what lands on it and one with no body lets it through; a cylinder and a cone collide as their shapes; a fall takes
  the time the scene's gravity says; the player walks, runs, turns and jumps at the keyboard, with the camera behind
  and above; a gamepad moves and turns it past its dead zone and is let go when unplugged (the pad's state is given,
  as the agent will give it: no pad is attached in the run); typing into a field is not playing; an emitter fills the
  air with its colour, counted in the frame; a sound is set up where it is, with its reach, only in play, and no sound
  device is opened while editing (what is heard is not judged); logic runs, is rebuilt when its file is saved without
  the world stopping, keeps running through a save that does not build and says why, and does not run while editing.
- **`tst_lockdown`** (7 cases, 12 checks with the imports' five; ADR-087): a game that keeps to its own files (logic
  importing Quick 3D Physics, a model from its `assets/`) plays with nothing refused; imports of `QtQuick.LocalStorage`,
  `QtCore`, `QtQuick.Dialogs` and `Qt.labs.settings` refused with the module named (and `QtMultimedia` an expected
  failure, the known limit); `SceneIO`'s writing, saving, copying, loading and listing outside the package refused and
  nothing on disk; a `Loader` and an `Image` aimed outside the package, by whole URL and by a relative path climbing
  out, both errors; a network request to a listener on this computer never arriving; `Qt.openUrlExternally` refused for
  a file and a web address, with nothing handed to the system. **Proven to fail** by four planted faults, each caught by
  its own check: no URL interceptor; no network policy; `SceneIO` left unconfined; every module allowed.
- **`tst_player`** (4 cases, the real executable): the playground plays and its start is measured; its first frame is
  the game (hundreds of colours, not an empty window); a scene file from disk plays too; a scene that is not there is
  said, not hidden.
- **Proven to fail** (10 October 2026) by five planted faults, each caught by its own check: no gamepad dead zone (the
  gamepad case); a logic file not watched (the rebuild); Play on the scene itself, not a copy (`tst_scene` and
  `tst_studio`); a sound active while editing (the sound case); a player that ignores the mouse (the drag in
  `tst_studio`, "turned 0").
- **In a browser:** `build.ps1 -Web` (`player/measure-web.mjs`) serves the build cross-origin isolated, plays it in
  headless Chrome, reads the Player's measurement, captures the first frame and fails unless it is the game, with no
  error on the page or in any of its workers.
- **`tst_agent`** (15 cases, one of them run only when asked): the protocol alone (a version agreed, or the server's newest; instructions given; not
  JSON, a batch, an unknown method or tool each answered with its JSON-RPC error; a notification not answered; a
  tool's failure a result with its id); in the real Studio, through the protocol: every tool listed with a name and
  schema clients accept; the scene read as its canonical text and fields, and every kind with its types, defaults,
  ranges and choices; entities added (shown in the Studio, unsaved, the file untouched), changed (one value, one
  line), refused with nothing of the call kept, named uniquely and removed; logic written, attached once, and code
  that does not build written but reported; play, a crate fallen to the ground as `wait` reports, the agent's
  controls walking the player, a captured frame that is the world, Stop restoring the scene and letting go of the
  controls, edits refused while playing; logic that throws every frame in the log once, with a count; and the real
  programs: `qq-mcp` with no Studio says so, finds one opened later, relays requests and results, and a second Studio
  on the same pipe stays out and says why. The design loop, against a stand-in for Anthropic's API on this computer: a
  description built, played, looked at and left for the creator (every request carrying the key in its header and
  nowhere else, the API version, the fallback opt-in, the model, its effort and thinking, and the fourteen tools; the
  conversation append-only; a turn's three results sent back together in order, the frame as a PNG; the crate built,
  unsaved, play stopped, the key in no log; the run undone back to the scene before it); a busy provider waited out (a
  429 and a 529, as `retry-after` says), a refused key said and not retried, a refusal ending the run, an answer cut
  off not run, a stop dropping the request in flight and anything after it; what a fallback replaced neither run nor
  sent back; the Agent panel asking for a key, never showing it again, then for a description; each run's token use
  counted from the provider's answers; and a key in the environment (`ANTHROPIC_API_KEY`, as a server might set) never
  used: with no key of the creator's own nothing is sent, the panel saying first who pays. **Proven to fail** by nine
  planted faults, each caught by its own check: a refused change half kept; edits allowed in play; a bridge that
  never tries the Studio again; a log that keeps every repeat; tool calls a fallback replaced run anyway; a busy
  provider not waited out; a cut-off answer's tool call run; the key put in the request's body; a key taken from the
  environment when the creator has given none. With
  `QQ_LIVE_ENDPOINT=1`, one more case reaches Anthropic's real endpoint over TLS with a key that is not one and is
  refused (10 October 2026: refused, as it should be).
- **With an independent client** (10 October 2026, once): the official MCP Python SDK (`mcp` 2.3.0) started `qq-mcp`,
  agreed protocol 2025-06-18, listed the fourteen tools, built a crate, an emitter and spinning logic, played, captured
  a frame of the result, stopped, and was refused a mass out of range.
- In the launcher: `check-qq-view.mjs` checks that **Open in QQ Studio** asks the host to start the Studio on the open
  project and nothing more; the host's `qq::` tests check where the Studio is found, that the project is one argument,
  and that a folder that is not a project is refused; `qq_studio_starts_on_a_project` (ignored by default: it opens a
  window) starts the deployed Studio on a real repository and sees it in the process list.

Learned on the way, and recorded in `HANDOFF.md` §5: Qt Quick 3D's `Material` type shadows the Material style's attached
`Material.*` when it is imported after it; Qt 6 keeps a URL property as written, so relative model URLs are resolved
against the entity's own file; across drives there is no relative path, and a bare `X:/...` is read back as a URL with
scheme `x`; a lambda counting `frameSwapped` must not take the window as its context while capturing a local (the
signal is queued from the render thread, and a late one writes into a stack frame that has gone); Qt for WebAssembly's
single-threaded kit has no Qt Multimedia, and in the threaded one Qt's spatial audio makes its sound on a worker, where
the web has no audio, so in a browser `Sound` is a plain sound effect.

## Known limits

- **In a browser, sounds are not panned.** Their volume follows the distance from the scene's player (full within a
  metre, halving with each doubling, none beyond `reach`); a scene with no player hears them at their own volume.
- **Logic is code, and the Player locks it down** (ADR-087). In the Studio a creator's logic has the engine's full
  reach, as it should. In the QQ Player, natively and in a browser, every game is confined (`runtime/confinement.h`):
  it loads QML and files only from its own package and Qt's and QQ's module folders, imports only an allowed list of
  modules (no storage, settings, dialogs, sockets or devices), makes no network request, writes and copies no file
  (`SceneIO::confineTo`), and opens nothing outside QQ. It is a second wall: a stranger's game plays only in the
  browser (P3.5), whose sandbox is the boundary. **Two limits, both shown by the checks:** Qt refuses URLs to the
  operating system only scheme by scheme, so a scheme off the list still reaches it; and Qt Multimedia (its camera and
  microphone types among them) cannot be refused natively, because Qt's spatial audio library registers it when it
  loads. Natively only a person's own games play; in a browser the browser asks before any camera or microphone.
- **Gamepads are read through XInput, so on Windows only**, and only Xbox-style pads; the checks give the pad's state
  rather than reading a real one.
- **One Studio at a time offers its tools.** A second Studio on the same pipe says so and stays out; `QQ_MCP_PIPE`
  gives one another name (for `qq-mcp` too).
- **An agent works with the creator's trust.** The pipe is open only to the user running the Studio, and the agent can
  write logic, which is code; it cannot save the scene or commit. The creator reviews what it did (the scene shows as
  unsaved, each change is said in the notice line) and saves and commits it.

## The Agent panel

**Agent**, in the Studio's header, opens it. Type an Anthropic API key once (it goes into Windows Credential Manager for
your account, and is not shown again), describe the game, and press **Design and build**. The panel shows what the agent
says, each tool it uses and each frame it captures; **Stop** ends the run at once. When it is done, look the scene over:
save and commit it as any other, or press **Undo the run** to put the scene back as it was (logic files it wrote stay in
`logic/`).

**You pay for what it uses, and Demiurge pays nothing** (ADR-086). The agent runs on your own Anthropic account: Anthropic
bills that account for each run (Claude Opus 5.5: $4 per million input tokens, $20 per million output tokens, cached
input $0.20; every captured frame is an image it reads). Demiurge does not pay for it, does not charge for it and does
not see the bill. Without your key the agent is simply off: none is built in, and none is taken from the environment.
After a run the panel shows the tokens it used, as Anthropic counted them.

## Connect an agent

Open QQ Studio (from the launcher's QQ screen, or `qq-studio`), then point any MCP client at `qq-mcp.exe`, which
`build.ps1 -Deploy` puts beside the Studio in `%LOCALAPPDATA%\qq-studio`. In Claude Code:

```powershell
claude mcp add qq -- "$env:LOCALAPPDATA\qq-studio\qq-mcp.exe"
```

In Claude Desktop's `claude_desktop_config.json`, with the path written out:

```json
{ "mcpServers": { "qq": { "command": "C:\\Users\\<you>\\AppData\\Local\\qq-studio\\qq-mcp.exe" } } }
```

The Studio says in its notice line when an agent connects, and each change it makes. The client's model runs on the
client's own account, as everything generative does here (ADR-086).

## Not in CI yet

GitHub Actions has no licensed Qt. ADR-083: until the owner adds Qt's installer and account to the repository's
secrets, or a self-hosted runner, QQ is built and tested on the owner's machine and its runs are recorded in
`HANDOFF.md`. The `qor_engine` suite in `docs/GATES.toml` is still the TypeScript preview's check until this replaces
it (ADR-083 decision 6).
