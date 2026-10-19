# ADR-104: The QOR Launcher on Android, on Qt, over the Rust core through a bridge of its own

**Status:** **Accepted**, 18 October 2026, by the owner: "I'd like to build a android varient of the QOR Launcher as
well". Asked, the owner chose:
- **Qt for Android now**, over the same Tauri launcher built for Android;
- a first build holding the wallet, QOR ID and Sparks with **Social, ARQADE, the Library and the Mesh**;
- delivery as **a signed APK on GitHub**;
- the Android toolchain **installed on the owner's PC**;
- **`app.qorsync.dev`** as the app's permanent ID: first typed `app.qorsync.dev`, then confirmed in Android's usual
  lowercase form when asked, before any tester installed it.

**Starts** [ADR-103](ADR-103-the-launcher-moves-to-qt-in-stages.md)'s stage 2, on Android first, and is the record that
stage asked for. **Supersedes** one thing in it: the bridge it named in passing, CXX-Qt (decision 2). Everything else in
ADR-103 stands, the first decision above all: the Rust host alone holds a key. It decides **no economic value**.

## Context

- **The launcher today** is a Tauri 2 application.
  - Its Rust host (`tools/qor-launcher/src-tauri/`) alone holds the vault's keys and signs, behind dialogs it draws
    itself. More than 300 host tests and fourteen browser checks surround that boundary.
  - Most of the host is free of Tauri: the vault, profiles, QOR ID's client, the chain client, Sparks, the Library's
    index, Social's client and the Mesh. What depends on Tauri is the application state and commands in `lib.rs`.
  - Some parts are desktop-only: the product host (named pipes, job objects), QOR Media and ARQADE's docking, Qontrol,
    and the Windows keychain.
- **Android** here means Android 9 (API 28) to 16 (API 36), Qt 6.12's range.
  - Qt 6.12 for Android is installed on the owner's PC for arm64-v8a, armeabi-v7a, x86 and x86_64.
  - `tools/qor-launcher/shell/android-toolchain.ps1` installed what it needs, checking every download's published
    SHA-256: JDK 21, the Android SDK (platform 36, build tools 36), NDK r27c (27.2.12479018, the NDK Qt 6.12 is built
    with) and Rust's Android targets.
- **CXX-Qt**, which ADR-103 named for the bridge, builds for Android only with workarounds.
  - Its maintainers wrote in 2025 that Android "is currently not a platform we actively support" (KDAB/cxx-qt#1251).
  - It picks the wrong `qmake` when cross-compiling (KDAB/cxx-qt#1153, open).
  - The bridge is the one part of the phone app that every screen and every key operation crosses, so it should not
    rest on an unsupported path.

## Decision

1. **The Android launcher is a Qt 6 application**, `tools/qor-launcher/shell/`.
   - Qt Quick throughout, with Qt Quick 3D where a scene is wanted, and touch-first.
   - Its ID is `app.qorsync.dev`, the owner's choice; once testers install it, it never changes.
   - It runs on Android 9 (API 28) and later, built for arm64-v8a, which is phones; x86_64 for the emulator.
2. **The Rust host becomes a core library**, `tools/qor-launcher/core/` (crate `qor-launcher-core`).
   - It is moved out of the Tauri crate one module at a time, with its tests, and nothing about a module changes in the
     move.
   - The Windows launcher uses it through Tauri as before. The Android shell links it as a static library.
   - Only the core holds a key, on either platform.
3. **The bridge is a small C interface of the core's own**, not CXX-Qt.
   - The shell sends a command and its arguments as JSON, and the core answers in JSON, in the shape the Tauri
     interface's `invoke` already uses, so one command function serves both shells. Events come back by a callback.
   - `unsafe` code is denied in the core. The bridge module alone is allowed it, only to move strings across the
     boundary, and it is tested.
   - This supersedes ADR-103's "(CXX-Qt)", for the reason in the context above. CXX-Qt can be weighed again if its
     maintainers come to support Android.
4. **Keys on Android.**
   - The wallet's key is sealed with its profile's PIN, as on Windows (ADR-099).
   - What is kept at rest is wrapped again by a key in Android's Keystore. That key never leaves the device's secure
     hardware where the device has it.
   - The shell reaches Keystore through JNI, behind the core's keychain interface, so the core decides what is sealed
     and the shell only stores it.
5. **What the first build holds**, the owner's scope:
   - **Wallet:** the Gate and profiles with PINs, the Vault with Sparks, and QOR ID's sign-in.
   - **Social.**
   - **ARQADE:** its Qt module in the same process (Android has no product host), signed in through the core as on
     Windows.
   - **The Library:** the phone's own photos, music and videos, through Android's MediaStore, each opened in the
     phone's own app. QOR Media stays Windows-only for now.
   - **The Mesh:** off until the person turns it on, and on Wi-Fi only unless they choose otherwise.

   Not on Android: Qontrol, QQ Studio, QOR Media and the product host.
6. **Delivery.**
   - Each build is a signed APK on a GitHub Release, `android-v<version>`, with a download page, as the Windows builds
     are.
   - The test signing key is made on the owner's PC and kept out of the repository.
   - The key for Google Play, later, is the owner's to hold.
7. **Checks.**
   - The core's tests run on every platform, and CI runs them.
   - The shell's Qt Test suites run on the owner's PC, since CI has no licensed Qt, as QOR Media's and ARQADE's do.
   - A check proves the shell cannot read a key (ADR-103 decision 3).

## Consequences

- `docs/DIRECTION.md` gains **L13, the launcher on Android**, in the order the work must go: the bridge and a running
  APK, the core moved out, keys in Keystore and the wallet's screens, QOR ID and Social, ARQADE, the Library, the Mesh,
  then a signed release.
- ADR-103's index line and status say that its bridge is ADR-104's now.
- Until L13 reaches a signed release, there is no Android launcher for testers. The Windows launcher is unaffected
  throughout, and its tests are the proof.
- On the Mesh, a phone seeds for test CGT as a PC does (ADR-095). Battery, storage and mobile data are the person's, so
  it starts off.
