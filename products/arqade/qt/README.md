# ARQADE in the launcher (Qt)

ARQADE's app: the QOR Launcher's arcade, in Qt 6, C++20, Qt Quick and Qt Quick 3D. It is docked inside the launcher
over its ARQADE screen, under the launcher's product host, as QOR Media is docked over the Library.
[ADR-102](../../../docs/decisions/ADR-102-arqade-in-the-launcher-on-qt.md) decided it, and roadmap item P7.19 in
[`docs/DIRECTION.md`](../../../docs/DIRECTION.md) tracks it.

This is the shell: the home, the games, the arenas and their rankings, and the player's Sparks. The four games and the
two arenas are rebuilt in QQ (P7.20, P7.24), in [`../games/`](../games/README.md), and all six play here: the arenas
against another person, on ARQADE's server.

## What it does

- **Docks.** The launcher starts it with `--launcher-channel <name>` and a one-time token on its stdin. The app
  connects, says hello, and keeps one request waiting, `arqade.next`. The launcher answers that request with its next
  command:
  - show at a rectangle of the launcher's window, in physical pixels, with the window's native handle;
  - hide;
  - wear the launcher's colours;
  - who is playing (QOR ID, level, and Sparks, written out by the launcher);
  - the person's preferences (reduced motion);
  - open a game, when a `qor://arqade` link names one (ADR-118): it opens as its card would, and a name no card plays
    opens nothing;
  - quit, when the person signs out of the launcher.

  The app's window becomes a child of the launcher's (`src/dock.h`, the same as QOR Media's). F11 lifts it to full
  screen; Esc puts it back. It exits when the launcher closes the channel.
- **Signs in to ARQADE's server with the launcher's QOR ID** (`src/arqadeserver.h`):
  1. The app asks ARQADE's server to start a sign-in. The server answers with QOR ID's sign-in page and sets its PKCE
     state on the app's cookie jar.
  2. The app asks the launcher, `arqade.signin`. The launcher checks the address is QOR ID's sign-in page on the QOR ID
     it uses. It asks QOR ID with its own session, checks the answer lands on ARQADE over https, and returns it.
  3. The app follows only an answer on its own server. That sets ARQADE's session cookie.

  The cookie jar lives in memory, so the session ends with the app. The app holds no key and no token of the person's,
  and logs no address, code or cookie.
- **Shows the rankings** from `/api/live`: all games, Flux Four, or Rift Reversi, with the player's own row marked. It
  asks every 15 seconds at most, and only while it shows.
- **Plays the arenas on ARQADE's server**, as the website plays them, with the session it signed in with: the live
  arcade's open matches, and a match opened, joined, moved in, conceded or closed, and asked after (`ArqadeServer`'s
  `createMatch`, `actOnMatch`, `fetchMatch`). A post carries the server's own origin, as the website's pages do, which
  its arena routes require. A match's id goes into an address, so only an id shaped as the server makes them is ever
  sent. A refusal comes back in the server's words.
- **Plays the games rebuilt in QQ** (`qml/GameView.qml`), on QQ's runtime, built into the app from
  `products/qq/runtime`. A rebuilt game's card opens it in place of the home, under a bar with Back and the game's name.
  Esc, or Back, puts it away and returns to the home; F11 still fills the screen. The launcher's reduced motion reaches
  the game as well as the home. Each play is kept as a run (QQ's `RunRecord`): its seed, what every tick took, and its
  outcome, which the game's rules replay. ARQADE's service will replay it before it confirms a run (P7.22); nothing is
  sent yet.

  An arena (Flux Four, Rift Reversi) asks the view what only the server knows, through QQ's host channel (ADR-117).
  The view answers from the server: the arena's open lobbies as the live arcade changes (read every 3 seconds while an
  arena's lobby shows, where the home reads it every 15), and the match it follows, asked after every 1.2 seconds as
  the website asks. A match that could not be joined or opened is not followed, and the lobby is told again. It tells the game only what changed, so the run keeps each answer once.
  It decides nothing of a match: the server does.

  The games play in ARQADE's own QML engine, as the QQ Player plays a person's own project. They are ARQADE's own,
  written and reviewed in this repository and carried in the executable. QQ's lockdown (ADR-087 decision 2) is the
  second wall around a stranger's game, and a stranger's game never plays here: it plays in the browser (ADR-087
  decision 1).
- **Opens the website** in the launcher's guarded ARQADE window (`arqade.web`), where the website's own versions of the
  games and the arenas are played.
- **Says what rewards are** wherever Sparks are shown (ADR-102 decision 7): paid from ARQADE's reward pool for runs its
  server confirms. Rewards arrive once runs are confirmed (P7.21, P7.22). On the devnet they are test CGT, with no
  value.

## How it looks

Demiurge's one look in the showcase register (ADR-108, `docs/design/DESIGN_SYSTEM.md`): obsidian, gilt and ember.
Every value comes from `qml/Tokens.qml`, which `node docs/design/tokens.mjs` generates from `docs/design/tokens.json`:
edit that, never this file. `qml/Theme.qml` binds to it, and lays the launcher's theme over its grounds, accent and
inks when the launcher sends one. Cinzel for names and numerals, Barlow Semi Condensed for labels and controls,
Barlow for reading, carried in the executable under the SIL Open Font License (`fonts/`, copied by the generator).
Buttons and tabs take the 8 pixel corner and panels the 12; the primary is ember with dark ink; every text off the
Cinzel titles is a step of the scale; every transition takes the look's durations and easing; the keyboard's focus
ring is the one every surface draws. `products/arqade/scripts/check-design.mjs` reads the QML for anything else, in
CI.

- **The portal** (`qml/Portal.qml`): an astrolabe of physically based gilt rings around an ember core. The homepage's
  triangle and square turn inside it. Embers rise through it and lean toward the pointer, and the camera follows the
  pointer.
- **The rendering:** filmic tonemapping, bloom, a vignette and 4× MSAA, lit by `assets/sky.hdr`.
- **Each game's card** (`qml/Emblem.qml`) carries a small 3D scene of what the game is about. It turns faster when
  hovered.
- **Reduced motion,** from the launcher, holds everything that is only decoration still.

It draws with Direct3D 12 where the graphics card has it, probed first so a card without it falls back to Qt's
Direct3D 11 rather than showing a black window. Where a PC has two cards, it draws on the one Windows calls
high-performance (`src/main.cpp`). `QSG_RHI_BACKEND` and `QT_D3D_ADAPTER_INDEX` override both choices.

**Every 3D view shares one sky image** for its metal to reflect (`assets/sky.hdr`, written by `tools/make_light.py`
with the core's halo). Six views that each built their own sky with Qt's `ProceduralSkyTextureData` crashed Qt 6.12
with heap corruption, in three runs out of three. One shared image is stable.

## Build, test, run

It needs the owner's Qt 6.12 (`C:\Qt\6.12.0\msvc2022_64`, with Qt Quick 3D), MSVC 2022 and Ninja. CI has no licensed Qt,
so these suites run on the owner's PC, as QOR Media's and QQ's do.

```powershell
pwsh products/arqade/qt/build.ps1              # build and run the four Qt Test suites
pwsh products/arqade/qt/build.ps1 -Run         # build, then open ARQADE in a window of its own
pwsh products/arqade/qt/build.ps1 -Deploy      # and make the self-contained copy the launcher opens
pwsh products/arqade/qt/build.ps1 -Shots dir   # and save what the interface and games tests drew, to look at
```

- `tst_arqadelink`: the launcher link against a stand-in launcher. It covers every command, a sign-in waiting its turn,
  refusals, oversized frames and the launcher going away.
- `tst_arqadeserver`: the sign-in, the rankings and the arenas, against a stand-in server and a stand-in launcher. An
  answer that leads elsewhere is not followed. A session that ends is renewed once, never in a loop. A match is created,
  asked after, moved in and joined, each post as JSON from the server's own origin with the session, and a refusal comes
  back in the server's words. A bad id, action or arena sends nothing.
- `tst_interface`: the module loads without a warning and draws on the graphics card. It checks the player and their
  Sparks, every game and arena, reduced motion holding it still, and a narrow window; and the one look: Theme equal to
  `tokens.json` value by value, every text the window shows in the look's faces, scale and colours (Barlow carried
  and drawn at its Regular), the primary's dark ink on ember and its focus ring. It needs a desktop session.
- `tst_games`: every card opens its game in ARQADE's window, from the keyboard. Each is played as a person plays it,
  the pad driven through QQ's input with the PC's own pads not read:
  - Orbital: the mouse on its Play and Resume buttons, Space and the pad's A to lock, and P and the pad's Start to
    pause. One lock outside the arc misses, a pause holds the satellite still, and ten locks in the arc complete the
    orbit with a score of 2,000.
  - Void Runner: steered away from each hazard about to arrive and toward each gem, changing lane by the keys, the
    arrows, the pad and the screen's Left and Right buttons in turn, until each has been used and a gem caught with the
    hull whole; then steered into three hazards, which end the run.
  - Synapse: a node pressed while the signal shows is no answer; three signals repeated, each node pressed by the keys
    1 to 4, the pad's X, Y, B and A, and the screen's buttons in turn; then a node out of turn loses it for 450.
  - Rift Survivor: moved by D, the left arrow, the d-pad and the screen's Up button; steered clear by the keys until
    the signal evolves and a perk is chosen while the world holds; then left standing until the hull breaks.
  - The arenas, against a stand-in ARQADE that signs in as the website does, keeps matches as `arena-store.ts` keeps
    them and moves for the opponent. Signed out, an arena says why and offers nothing. Flux Four opens a match, a
    challenger joins, and four pulses dropped by Space, the key 5, the pad's A and the pointer win it; then a rematch's
    lobby is closed. In Rift Reversi a lobby taken by someone else just first is said to be gone, and the lobby then
    shows none open; then it joins someone's open lobby. Its discs are placed by the keys and the pointer, the
    discs they bracket turn over, a move the server refuses says why, and the match is conceded once the player is sure.

  Sounds that happen somewhere are checked to be heard from there, through QQ's spatial audio (Qt's `SpatialSound`).

  The run ARQADE kept of each replays in QQ, with no window, to the same outcome, and so does each run kept in
  `../tests/fixtures/`, which the game's Node test replays with its rules (an arena's with its conversation,
  `arena.mjs`). The launcher's reduced motion reaches the
  game, and Esc and Back each return to the home. With `ARQADE_RUN` set to a folder, it writes each run it played there
  as `<game>-run.json`, which is how the fixtures were made. It needs a desktop session. A card is opened from the
  keyboard because on Windows the real pointer, wherever it is, can take a synthetic one's hover and press away: one
  run in about fifteen missed the card when it was clicked.

`-Deploy` copies what QQ's runtime and the games import as well as ARQADE's own: Qt Quick 3D's physics, particles and
spatial audio, and Qt Multimedia. On 24 October 2026 the deployed copy was 187 MiB, 32 MiB more than before the games,
and `tst_games` passed when run from inside it with Qt's own folders off the path.

The launcher's side is in `tools/qor-launcher/src-tauri/src/arqade/app.rs`. Its tests include one that starts the real
app, deployed by `-Deploy`, and is run only when asked:

```powershell
cargo test --lib arqade::app::tests::arqade_started -- --ignored
```

The launcher finds the app where `ARQADE_APP` points, then beside itself (`arqade\arqade.exe`, where the installer puts
it), then under `%LOCALAPPDATA%\arqade`. `tools/qor-launcher/scripts/stage-arqade.mjs` builds and stages it for the
Windows installer. `ARQADE_SERVER` points the app at another ARQADE server, for tests. The launcher's own checks
accept a sign-in only for ARQADE's real address.
