# ARQADE in the launcher (Qt)

ARQADE's app: the QOR Launcher's arcade, in Qt 6, C++20, Qt Quick and Qt Quick 3D. It is docked inside the launcher
over its ARQADE screen, under the launcher's product host, as QOR Media is docked over the Library.
[ADR-102](../../../docs/decisions/ADR-102-arqade-in-the-launcher-on-qt.md) decided it, and roadmap item P7.19 in
[`docs/DIRECTION.md`](../../../docs/DIRECTION.md) tracks it.

This is the shell: the home, the games, the arenas' rankings and the player's Sparks. The games themselves are being
rebuilt in QQ (P7.20). Until a game's rebuild arrives, its card opens today's version in the launcher's ARQADE window.

## What it does

- **Docks.** The launcher starts it with `--launcher-channel <name>` and a one-time token on its stdin. The app
  connects, says hello, and keeps one request waiting, `arqade.next`. The launcher answers that request with its next
  command:
  - show at a rectangle of the launcher's window, in physical pixels, with the window's native handle;
  - hide;
  - wear the launcher's colours;
  - who is playing (QOR ID, level, and Sparks, written out by the launcher);
  - the person's preferences (reduced motion);
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
- **Opens the website** in the launcher's guarded ARQADE window (`arqade.web`), where today's versions of the games are
  played.
- **Says what rewards are** wherever Sparks are shown (ADR-102 decision 7): paid from ARQADE's reward pool for runs its
  server confirms. Rewards arrive with the rebuilt games. On the devnet they are test CGT, with no value.

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
pwsh products/arqade/qt/build.ps1              # build and run the three Qt Test suites
pwsh products/arqade/qt/build.ps1 -Run         # build, then open ARQADE in a window of its own
pwsh products/arqade/qt/build.ps1 -Deploy      # and make the self-contained copy the launcher opens
pwsh products/arqade/qt/build.ps1 -Shots dir   # and save what the interface test drew, to look at
```

- `tst_arqadelink`: the launcher link against a stand-in launcher. It covers every command, a sign-in waiting its turn,
  refusals, oversized frames and the launcher going away.
- `tst_arqadeserver`: the sign-in and the rankings, against a stand-in server and a stand-in launcher. An answer that
  leads elsewhere is not followed. A session that ends is renewed once, never in a loop.
- `tst_interface`: the module loads without a warning and draws on the graphics card. It checks the player and their
  Sparks, every game and arena, reduced motion holding it still, and a narrow window; and the one look: Theme equal to
  `tokens.json` value by value, every text the window shows in the look's faces, scale and colours (Barlow carried
  and drawn at its Regular), the primary's dark ink on ember and its focus ring. It needs a desktop session.

The launcher's side is in `tools/qor-launcher/src-tauri/src/arqade/app.rs`. Its tests include one that starts the real
app, deployed by `-Deploy`, and is run only when asked:

```powershell
cargo test --lib arqade::app::tests::arqade_started -- --ignored
```

The launcher finds the app where `ARQADE_APP` points, then beside itself (`arqade\arqade.exe`, where the installer puts
it), then under `%LOCALAPPDATA%\arqade`. `tools/qor-launcher/scripts/stage-arqade.mjs` builds and stages it for the
Windows installer. `ARQADE_SERVER` points the app at another ARQADE server, for tests. The launcher's own checks
accept a sign-in only for ARQADE's real address.
