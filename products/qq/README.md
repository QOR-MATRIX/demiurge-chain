# QQ, the QOR Engine

An engine for virtual worlds and experiences, built on **Qt 6** under the owner's commercial licence
([ADR-083](../../docs/decisions/ADR-083-qq-on-qt.md)). The blueprint is [`docs/blueprints/qq.md`](../../docs/blueprints/qq.md);
the roadmap items are DIRECTION P3.1 to P3.16.

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
- **2D (P3.10, built 19 October 2026; ADR-105, ADR-106):** a scene with `view: "2d"` is the same world seen flat,
  through an orthographic camera on the plane z = 0, over a backdrop of its sky's colours. **Pixel art** (`pixelArt`, on
  by default) is scaled by whole numbers of screen pixels to an art pixel, as many as fit `viewHeight` units, with the
  camera on whole device pixels, and shown in exactly the colours it was painted (no bloom, tonemapping or dithering);
  drawn art fills its view exactly and keeps the 3D look. **Sprite**: an image or a frame of a sheet, played from
  `first` for `count` frames at `fps`, crisp or smooth, lit or not, flipped, and a body if it needs one. **TileMap**: a
  level drawn in [Tiled](https://www.mapeditor.org) and saved as JSON (`.tmj`, its tileset beside it or inside it, tiles
  as CSV or zlib-compressed Base64, flips kept), one layer each, drawn as one mesh; a solid one collides through the
  fewest rectangles that cover its tiles, and the one marked `bounds` is where a following eye stops. **Player2D**: a
  platformer's hero or a top-down one, under the same input as everything else, carrying a Sprite as its look and
  saying what it is doing (`activity`) so logic plays the right frames; `respawn()` puts it back. Shapes and sprites
  with bodies stay on the plane and turn only about its depth axis.
- **The Studio:** the scene's entities in a list, the selected one's fields in an inspector (built from what each type
  declares), click to select in the viewport, handles to move along an axis, turn about the vertical or scale evenly,
  orbit and zoom, add (cube, sphere, cylinder, cone, lamp, model, player, emitter, sound, behaviour, the screen kinds,
  actions, and in 2D a sprite, a tile map and a 2D player) and delete. A 2D scene is edited flat: a drag pans, the wheel
  zooms, and the handles move things on the plane and turn them about its depth axis.
  **Play** (F5) plays a copy built from the scene's canonical text; **Stop** (Esc) throws the copy away, so the scene,
  its unsaved state and the selection are exactly as they were. **New behaviour** writes a starter logic file into the
  project, never over one that is there.
- **The files:** a scene is `scenes/<name>.qml` in the project, in one canonical layout: read and written back it is
  the same bytes, and one changed value is one changed line in Projects. Models and sounds are copied into `assets/`
  and logic into `logic/`, so the project is whole on its own; a Tiled map brings its tileset and image, and one whose
  tileset is outside its folder is refused with the reason. 2D scenes from the launcher's preview (`.qq.json`) import
  as 2D scenes of drawn art (100 pixels to a unit, framed as they were made, their background as the backdrop),
  emitters included; motion and the pointer follow are named as not brought in.
- **The QQ Player:** plays one scene as a game, full window, natively (`qq-player [scene.qml]`, the bundled playground
  without one) and as a WebAssembly build in a browser, and measures how long its first frame takes.
- **The launcher:** **Open in QQ Studio** on the launcher's QQ surface starts the Studio on the project open there.
- **Publishing (P3.4, built 10 October 2026):** **Publish** saves the scene, then asks the launcher that started the
  Studio to commit the project through Qontrol and publish it as its **Cartridge** (ADR-088): the first time a DRC-369
  asset is minted, holding the manifest of every file at that commit; each time after, the same Cartridge is revised
  to the new commit; with nothing new, nothing is asked. The launcher's own window asks first and names QQ Studio; the
  Studio holds no key and never sees a transaction. It asks over a channel only the process the launcher started can
  use (ADR-046: a one-time token on its stdin and the operating system's word on who connected), and exits when the
  launcher closes it. A Studio started any other way works as before but cannot publish, and its Publish button says
  why.

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
| `runtime/` | The QML module `QQ`: `World` (the viewport, its sky, light, look and hearing), `Scene` (the root a file holds), `Game` (a world playing one scene file: what the Player shows), the entities `Sun`, `Lamp`, `Ground`, `Shape`, `Prop`, `Player`, `Emitter`, `Sound`, `Behaviour`, `Trigger` (a volume that hears what enters and leaves it, P3.7), the screen kinds `ScreenText`, `ScreenBar`, `ScreenButton`, `ScreenImage` and `ScreenPanel` (the game's interface, drawn over the world on the World's overlay, each in a named screen; built on `ScreenPart`, P3.8), `InputAction` (an action a player can take, with its keys, mouse buttons and pad buttons, P3.9), the 2D kinds `Sprite`, `TileMap` and `Player2D` (P3.10), and `Logic` (what a logic file is, with the gameplay API of P3.7: `onStarted`, `onContact`, `onEntered`, `onExited`, `ray`, `spawn`, `remove`, `after`, `every`, `cancel`; and for the interface, P3.8, `onClicked`, `showScreen`, `hideScreen`, `onlyScreen`, `pause` and `resume`; and for actions, P3.9, `onPressed`, `onReleased`, `held`, `rebind`, `bindingOf` and `pointing`). `Look` is the interface's style: the faces Cinzel and Barlow Semi Condensed (carried in `runtime/fonts/`, under the SIL Open Font License), the launcher's inks and ARQADE's gilt, and the system's reduced-motion setting. In C++: `SceneIO` (canonical writing, loading, saving, playing a copy, adopting files, importing 2D scenes, and the editing operations the Studio uses and the agent will, P3.3), `Input` (keyboard, mouse and gamepad as intentions), `LogicFile` (builds a logic file and rebuilds it on each save), `tilemap` (Tiled's maps read, their solid tiles merged into colliders, and `TileGeometry`, a layer as one mesh), `SoftDot` (the particles' sprite, made in memory) `gpu` (draw with the faster of two GPUs), `Confinement` (what a game may reach in the Player, ADR-087) and `WebVoice` (a sound in a browser, so a game never needs Qt Multimedia's QML) |
| `studio/` | QQ Studio. `qq-studio [model.gltf] [--project <folder>] [--capture <frame.png>] [--launcher-channel <name>]`; `LauncherLink`, its side of the QOR host protocol (publishing through the launcher that started it, ADR-088) |
| `agent/` | The agent's side of the Studio (P3.3): `McpServer` (MCP's JSON-RPC, served on a local pipe only this user can open), `StudioTools` (the fourteen tools, working the Studio's own window functions, and `StudioLog`, what the Studio has said), `DesignLoop` (a description designed, built, played and judged by Claude through those tools; ADR-085), `keychain` (the provider's key in Windows Credential Manager) and `qq-mcp` (the program an MCP client starts, relaying its standard input and output to the running Studio) |
| `player/` | The QQ Player. `qq-player [scene.qml] [--capture <frame.png>] [--measure <result.json>]`; its playground (a small game with everything P3.2 plays) bundled inside it; `measure-web.mjs`, which plays the browser build in a headless browser and measures it |
| `tests/` | `tst_world` (rendering, judged by pixels), `tst_scene` (scene files), `tst_studio` (the real Studio window, worked by clicks, drags and keys), `tst_play` (physics, the player, input, particles, sound, logic and gameplay in logic, 2D pixel for pixel, in real frames, and the Orb Run and Ember Climb samples played), `tst_player` (the real Player executable), `tst_lockdown` (what a game's logic may reach in the Player, ADR-087), `tst_publish` (publishing through a stand-in launcher, P3.4), `tst_agent` (the tools over MCP, in the real Studio and through the real `qq-mcp`) |
| `tests/fixtures/` | `orb.gltf` (written by `make_orb.py`), `chime.wav` (written by `make_chime.py`), `logic/spin.qml`, `scenes/sample.qml` (every entity type, canonical), `first-light.qq.json` (the 2D preview's starter scene), and the 2D fixtures `make_2d.py` writes: `tiles.png`, `tiles.tsj`, `level.tmj` (a Tiled map with a mirrored tile and two layers) and `sheet.png` (eight frames) |
| `samples/` | Games made only from kinds and logic files, each proving a step of P3 and played by the suites (ADR-105): `orb-run/` for P3.7 to P3.9 and `ember-climb/`, a pixel-art platformer, for P3.10 ([`samples/README.md`](samples/README.md)) |
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

On the owner's laptop (RTX 4060 Laptop GPU), on mains power, playing the bundled playground at 1280 × 720, locked down
(ADR-087):

| | Native | In a browser (Chrome, headless) |
| --- | --- | --- |
| First frame proper (the scene playing, every model loaded) | about 1.7 s from the process starting (five runs: 2,023, 1,709, 1,729, 1,791 and 1,711 ms) | 5.5 to 5.7 s from the page starting to load, cold, served from this machine (two runs: 5,683 and 5,537 ms; `main()` begins at about 0.3 to 0.4 s) |
| Frames a second over the next three seconds | 265 to 346 | 313 |
| Size | the executable, 0.55 MB, with Qt's libraries beside it | 44.1 MB as served (the `.wasm` 43.8 MB); 16.0 MB gzip, 11.7 MB brotli |

The lockdown costs nothing measurable: its parts switched off, one at a time or all together, give the same first
frame. The 1.4 s first measured (P3.2) was for a game missing its one model, which had failed to load; the Player now
names any model that fails (`unloaded` in its measurement), and its checks fail on one.

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
  file of another format is not imported. **Since P3.10, 19 cases (24 checks):** a Tiled map is read with its external
  tileset, its mirrored tile and its solids merged, and another layer by name; a map of zlib-compressed Base64 with its
  tileset inside reads the same; six maps that cannot be read are refused, each with its reason; solid tiles become the
  fewest rectangles; and a map adopted into a project brings its tileset and image, while one whose tileset is outside
  its folder is refused with nothing copied.
- **`tst_studio`** (10 cases, the real window): a new Studio opens a lit scene in its project; a click on the model
  selects it and a click on the sky selects nothing; dragging the X handle moves along X only; dragging the centre
  handle scales evenly and leaves the position alone; saving writes a canonical scene and carries the model into the
  project, and one edit saved again is one changed line; a saved scene opens again as it was; entities are added,
  named uniquely and deleted; a 2D scene is imported with what cannot come yet said; **Play then Stop**: Play from its
  button, a crate falls and rests while the scene's crate stays put, W walks the player and the eye follows, a drag
  turns the player and tilts the eye, Esc leaves the scene, its unsaved state and the selection exactly as they were,
  and Play starts again from the same place; a new behaviour is written into the project and never over a file.
  **Since P3.10, 11 cases:** a 2D scene opens framed as it plays, its handles see what the world sees, its X handle moves
  along X on the plane, a drag on nothing pans with the pointer, and the wheel shows fewer units, larger.
- **`tst_play`** (11 cases, real frames): a dynamic shape falls and comes to rest on the ground; a static one holds
  what lands on it and one with no body lets it through; a cylinder and a cone collide as their shapes; a fall takes
  the time the scene's gravity says; the player walks, runs, turns and jumps at the keyboard, with the camera behind
  and above; a gamepad moves and turns it past its dead zone and is let go when unplugged (the pad's state is given,
  as the agent will give it: no pad is attached in the run); typing into a field is not playing; an emitter fills the
  air with its colour, counted in the frame; a sound is set up where it is, with its reach, only in play, and no sound
  device is opened while editing (what is heard is not judged); logic runs, is rebuilt when its file is saved without
  the world stopping, keeps running through a save that does not build and says why, and does not run while editing.
  **Since 19 October 2026, 18 cases (P3.7, ADR-105):**
  - logic hears its target's body meet another entity, once as they meet;
  - a Trigger hears a falling body enter and leave, and lets it through;
  - a Trigger hears the player walk in;
  - a ray meets the first entity in its way, passing through triggers and starting inside its own target;
  - logic makes three bodies, refuses an unknown kind, removes one with its body, and keeps timers on its own clock,
    `every` and `after` and a cancel;
  - an impulse moves a dynamic body, a kinematic one is not pushed, and a kinematic lift rises with its rider;
  - the Orb Run sample is canonical and plays: an orb gathered, removed with its glow and replaced, and the boulder
    rolled.

  Five planted faults were each caught: the contact relay cut, a removed body left behind, timers never run, the
  player unheard by triggers, and rays with no view.
  **Since 19 October 2026, evening, 23 cases (P3.8):**
  - a screen draws what logic sets, over the world, where its anchor and insets put it;
  - screens show and hide, and a pause holds the world, logic's clock and its timers while screens still answer;
  - a button answers only while its screen shows and the world plays, and never while editing;
  - with reduced motion a screen appears at once;
  - an image from the project is drawn crisp;
  - Orb Run opens on its title, Play starts it, the score is on its HUD, and Pause and Resume hold and release it.

  Five planted faults were each caught: a button that answers on any screen, physics or logic that ignores a pause,
  reduced motion ignored, and a part drawn whatever its screen.
  **Since 19 October 2026, night, 26 cases (P3.9):**
  - an action is held from its key, a mouse button, a pad button and a finger on a button;
  - jump and run are actions with their old bindings;
  - a player rebinds a key, a pad button and the built-in jump by pressing what they want, and that press does
    nothing else;
  - the pointer picks a shape, the sky and the ground;
  - Orb Run pauses and goes on with P.

  Five planted faults were each caught: a press never announced, the naming press also firing, a finger holding
  nothing, picking ignoring the pointer, and every event read twice.
  **Since 19 October 2026, late night, 32 cases (P3.10):**
  - a crisp sprite is drawn pixel for pixel at a whole zoom: every art pixel a square of whole device pixels, sharp
    edges, the painted colours, the sheet's second row the right way up, and flipped;
  - a sheet plays its frames, from the first of them, and holds them while the world is paused;
  - a Tiled level is drawn tile for tile with its mirrored tile and only its first layer, and the hero stands on its
    floor to within half an art pixel, runs to its wall, jumps a unit and a half and turns;
  - a top-down player moves across the plane with no gravity and carries its look;
  - a dynamic body in a 2D scene, pushed into the depth, stays on the plane and turns only about its depth axis;
  - Ember Climb is canonical and plays: its title, an ember gathered, a pit and back to the start, pause, the forge lit
    and Again; and its eye stops at the level's edges.

  Seven planted faults were each caught: a sheet read upside down, pixel art scaled to fit, Tiled's mirror flag
  ignored, PhysX's skin not allowed for, a body free to drift in depth, a tileset read from outside the package (by
  `tst_lockdown`), and the eye shown past the level's edges.
- **`tst_lockdown`** (7 cases, 12 checks with the imports' five; ADR-087): a game that keeps to its own files (logic
  importing Quick 3D Physics, a model from its `assets/`) plays with nothing refused; imports of `QtQuick.LocalStorage`,
  `QtCore`, `QtQuick.Dialogs` and `Qt.labs.settings` refused with the module named (and `QtMultimedia` an expected
  failure, the known limit); `SceneIO`'s writing, saving, copying, loading and listing outside the package refused and
  nothing on disk; a `Loader` and an `Image` aimed outside the package, by whole URL and by a relative path climbing
  out, both errors; a network request to a listener on this computer never arriving; `Qt.openUrlExternally` refused for
  a file and a web address, with nothing handed to the system; and (P3.10) a picture measured, a map read, or a
  map's tileset read from outside the package, each refused, while a game's own map and sheet play. **Proven to fail** by four planted faults, each caught by
  its own check: no URL interceptor; no network policy; `SceneIO` left unconfined; every module allowed.
- **`tst_publish`** (9 cases; P3.4, ADR-088), against a stand-in launcher speaking the QOR host protocol: the hello
  carries the token and the Studio's id; Publish asks for the commit and then the publish, with the project, and
  reports the receipt; a decline is said and nothing more is asked; a refused handshake leaves publishing unavailable
  with the reason; frames a byte at a time and two in one write read alike; a frame over 1 MiB is not read; the
  launcher closing the channel is told (and the Studio exits); and the real Studio window, not started by the launcher,
  cannot publish, then, linked, saves an unsaved scene first and says what was published, or that it was declined,
  and a refusal too long for the toolbar is shown whole when the notice is hovered;
  and, linked but on a folder that is not versioned, Publish can still be pressed, says what is missing and asks the
  launcher nothing. **Proven to fail** by four planted faults: publishing without the commit (three cases);
  publishing without saving first; Publish greyed out on a folder that is not versioned (the owner's first run, 10
  October 2026); the whole notice not shown on hover (15 October 2026). The launcher's side, the channel's proofs and a run against a development node, is in the launcher's tests.
- **`tst_player`** (5 cases, the real executable): the playground plays and its start is measured; its first frame is
  the game (hundreds of colours, not an empty window); a scene file from disk plays too, with every model loaded in it
  and in the playground; a 2D game (Ember Climb) plays, measured and captured; a scene that is not there is said, not
  hidden. **Proven to fail** (10 October 2026) by putting
  back `Prop`'s old loader line: both playing cases fail with `Orb: IO Error: File not found`.
- **Proven to fail** (10 October 2026) by five planted faults, each caught by its own check: no gamepad dead zone (the
  gamepad case); a logic file not watched (the rebuild); Play on the scene itself, not a copy (`tst_scene` and
  `tst_studio`); a sound active while editing (the sound case); a player that ignores the mouse (the drag in
  `tst_studio`, "turned 0").
- **In a browser:** `build.ps1 -Web` (`player/measure-web.mjs`) serves the build cross-origin isolated, plays it in
  headless Chrome, reads the Player's measurement, captures the first frame and fails unless it is the game, with every
  model loaded and no error on the page or in any of its workers.
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

- **2D, as built in P3.10:** a Tiled map is read one tile layer per TileMap, from its first tileset, which must be one
  image (a collection of images is refused, with the reason); object layers, tile animations, one-way platforms and
  slopes are not read yet. A Player2D is PhysX's character controller, built 0.1 unit smaller all round because PhysX
  keeps it that far from what it touches (Qt Quick 3D Physics leaves PhysX's contact offset at its default). It steps
  up on its own only what is lower than a quarter of its capsule's straight part, which Qt sets: a sixteenth of a unit
  for the default figure, so a platformer's hero jumps onto anything higher.

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
