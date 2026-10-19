# The QOR Launcher on Android: the Qt shell and the core

[ADR-104](../../../docs/decisions/ADR-104-the-launcher-on-android-on-qt.md) decided it, and DIRECTION's **L13** tracks it.

The Android launcher is a Qt 6 application, this folder. It sits over the launcher's Rust core,
[`../core`](../core/), which holds the keys and talks to the chain on every platform. The Windows launcher uses the same
core through Tauri.

## Where it stands (L13.1)

- **The core** (`tools/qor-launcher/core`, crate `qor-launcher-core`) is being moved out of the Tauri crate one module at
  a time, with its tests. It holds Sparks and amounts (`cgt.rs`) and the error type so far. The Windows launcher
  re-exports both at their old paths, and every test is green.
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
cargo test --manifest-path tools/qor-launcher/core/Cargo.toml   # the core's own tests (CI runs them)
```

- **The core is built by Corrosion** (pinned v0.6.1, fetched at configure), for whatever the build targets: the host on
  the desktop, `aarch64-linux-android` with the NDK's compilers for the phone.
- **The Android build runs inside MSVC's environment**, because Rust's build scripts and macros are built for the PC
  first.
- **The APK is signed with Android's debug key**, for testing only. A signed release is L13.8, with a test key kept off
  the repository.
- **The shell's Qt Test suites run on the owner's PC.** CI has no licensed Qt.
