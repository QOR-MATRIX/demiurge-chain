# The QOR Launcher on Android: the Qt shell and the core

[ADR-104](../../../docs/decisions/ADR-104-the-launcher-on-android-on-qt.md) decided it, and DIRECTION's **L13** tracks it.

The Android launcher is a Qt 6 application, this folder. It sits over the launcher's Rust core,
[`../core`](../core/), which holds the keys and talks to the chain on every platform. The Windows launcher uses the same
core through Tauri.

## Where it stands (L13.1, and L13.2 done)

- **The core** (`tools/qor-launcher/core`, crate `qor-launcher-core`) holds everything of the Windows launcher's Rust
  host that holds a key or talks to a service, moved out of the Tauri crate one module at a time with its tests
  (DIRECTION L13.2): Sparks and amounts (`cgt.rs`), the error type, the wallet (`vault/`), the confirmation it asks
  through (`confirm.rs`) and the profiles (`profiles.rs`, ADR-099), and since 26 October 2026 QOR ID's client, Social's
  socket, the chain client with the object model, `qor://pay` and `qor://agent`, listings and trading partners, the
  Library's index, the Mesh and Qontrol. The Windows launcher re-exports each at its old path, and every test is green.
  - The modules moved last are each behind a feature (`identity`, `social`, `content`, `chain`, `library`, `mesh`,
    `qontrol`; `desktop` is all of them, which the Windows launcher and QOR OS build, ADR-120). None is on by default,
    and this shell builds the core with its default features, so the phone links what it did before. L13.3 to L13.7
    turn on what the phone needs as each is built.
  - On Android the core has no keychain: `keyring` has no Android store and would keep a wallet's key in memory. The
    shell will give the vault a store wrapped by Android's Keystore (L13.3). Until then nothing on the phone makes a
    wallet.
  - The test stand-ins (an in-memory keychain, a scripted confirmation, the test process's logger) are behind the core's
    `testing` feature, which only test builds turn on.
- **The bridge** (`core/src/bridge.rs`, `core/include/qor_core.h`) is the shell's one way in.
  - The shell sends a JSON request, `{"command", "args"}`, and gets a JSON answer: `{"ok": …}`, or
    `{"error": {"kind", "message"}}` as the Windows interface already reads it.
  - Amounts cross as strings of embers.
  - It is the only code in the core allowed `unsafe`, and only to move strings across the C boundary.
  - It answers `core_version`, `token_info`, `parse_amount` and `format_sparks` so far (`core/src/commands.rs`).
- **The shell** (`src/core.h`, `qml/`) shows the core answering and reads an amount typed in Sparks or CGT. The wallet,
  QOR ID, Social, ARQADE, the Library and the Mesh come in L13's next steps.
- **The app:** ID `app.qorsync.dev`, Android 9 (API 28) and later, arm64-v8a.

## The toolchain

```powershell
pwsh tools/qor-launcher/shell/android-toolchain.ps1
```

It installs, each download checked against its published SHA-256:
- JDK 21 to `%LOCALAPPDATA%\Programs\jdk-21`;
- Android's command-line tools, SDK platform 36, build tools 36 and NDK r27c (27.2.12479018, the NDK Qt 6.12 is built
  with) to `%LOCALAPPDATA%\Android\Sdk`;
- Rust's `aarch64-linux-android` and `x86_64-linux-android` targets.

It sets `JAVA_HOME`, `ANDROID_SDK_ROOT` and `ANDROID_NDK_ROOT` for this user. It also needs the owner's Qt 6.12 with Qt
for Android, MSVC 2022, Ninja and Rust.

## Build and test

```powershell
pwsh tools/qor-launcher/shell/build.ps1              # desktop build, then its Qt Test suite (tst_core)
pwsh tools/qor-launcher/shell/build.ps1 -Run         # desktop build, then open it
pwsh tools/qor-launcher/shell/build.ps1 -Android     # the phone's APK (arm64-v8a), debug-signed for testing
cargo test --manifest-path tools/qor-launcher/core/Cargo.toml --all-features   # the core's own tests, every module (CI runs them)
```

- **The core is built by Corrosion** (pinned v0.6.1, fetched at configure), for whatever the build targets: the host on
  the desktop, `aarch64-linux-android` with the NDK's compilers for the phone.
- **The Android build runs inside MSVC's environment**, because Rust's build scripts and macros are built for the PC
  first.
- **The APK is signed with Android's debug key**, for testing only. A signed release is L13.8, with a test key kept off
  the repository.
- **The shell's Qt Test suites run on the owner's PC.** CI has no licensed Qt; it reads the shell's QML with
  `docs/design/check-qml.mjs`.

## Its look

Demiurge's one look in the tool register (ADR-108), as the Windows launcher's default, Obsidian. `qml/Theme.qml` binds
every value to `qml/Tokens.qml`, which `docs/design/tokens.mjs` generates from `docs/design/tokens.json`. The faces
are the app's own, the same on a phone and a PC (`fonts/`, copied by the same script, loaded by `src/fonts.h`): Cinzel
in gilt for the wordmark alone, Barlow Semi Condensed for labels, Barlow for reading, IBM Plex Mono for numbers.
`tst_core` checks Theme against `tokens.json` value by value and that each face is drawn from its own file.
