# ADR-127: The homepage offers QOR Mobile and restates QOR OS

**Status:** **Accepted**, 30 October 2026, by the owner, who asked the assistant to "update the website QORSYNC.DEV with
all current info and ensure that all download links are updated. Users should be able to learn about all things QOR OS
while provided an extremely high quality mobile viewing experience. Be sure that you also include the APK download for
the QOR Mobile app. Change the animated touch reactive background to something more graphically mindblowing too." The
structure, the words and the look below are the assistant's, under that instruction. It decides **no economic value**.
**Supersedes [ADR-123](ADR-123-the-homepage-leads-with-qor-os.md)** decision 2 (what the page says about QOR OS) and
the second point of its decision 4 (the living canvas), and **amends** its decisions 3 and 5. **Starts DIRECTION's
L13.8** (a signed APK on a GitHub Release, with a download page) ahead of L13.3 to L13.7, at the owner's word, for the
first screen L13.1 built. **Names** the Android launcher on the phone; [ADR-104](ADR-104-the-launcher-on-android-on-qt.md)
is otherwise unchanged, its permanent ID above all.

## Context

- **The homepage** has led with QOR OS since ADR-123, whose decision 2 lists what it may say about QOR OS, checked
  against QOR OS's private tree on 29 October 2026. Since then QOR OS has moved on: QQ, ARQADE and QOR Media are
  developed there (ADR-126), Prism starts and hosts all three, every window opens at the size its app asks for, and QQ
  reads QOR OS's own controller service. ADR-123 says such a change is a later record.
- **The downloads.** The page linked the launcher's release page, not its installer, and offered nothing for Android.
  The QOR Launcher for Android (ADR-104, L13) has built a first screen over the core since 18 October 2026, its arm64 APK
  signed only with Android's debug key and never released. The owner calls it **QOR Mobile** and asked for its APK on
  the page.
- **On a phone** the page hid its navigation below 760 pixels wide, with no menu, and several links were 20 to 31 pixels
  tall. The living canvas (ADR-123 decision 4) was a 2D drawing of rings and a lattice.

The facts in decision 3 were checked on 30 October 2026 against QOR OS's `main` at `dd169a7`: its `ROADMAP.md`,
`ARCHITECTURE.md`, `docs/design/SHELL.md`, `docs/CONTROLLERS.md` and its decision records. They are written here so a
reader of this repository can check the page against them.

## Decision

1. **QOR Mobile.** The owner's name for the QOR Launcher for Android. On the phone its label and its wordmark say QOR
   Mobile; its ID stays `app.qorsync.dev` (ADR-104 decision 1). The records keep calling the system the QOR Launcher
   for Android, and the page calls it both.
2. **A first release for testers, now** (L13.8, ahead of its turn): the GitHub pre-release `android-v0.1.0`, whose asset
   `qor-mobile-0.1.0-arm64-v8a.apk` is version 0.1.0 (code 1), for Android 9 and later on 64-bit ARM.
   - It is **signed with the project's test key**, as L13.8 says: a PKCS12 key made on the owner's PC on 30 October
     2026, kept off the repository in `%USERPROFILE%\.qor-launcher\android\`, its password beside it sealed by Windows
     DPAPI to the owner's account. Its certificate is `CN=QOR-MATRIX test key, O=QOR-MATRIX`, SHA-256
     `bf3b9fea964990d0f6ea34f0f178555c5138542f3295c70a09fd454add7e8b7d`. `tools/qor-launcher/shell/build.ps1 -TestKey`
     signs with it. Every later release is signed with the same key, so each installs over the last; the Google Play
     key, later, is the owner's.
   - **What it does is the first screen only**: the core answering, amounts read exactly in CGT, Sparks and embers. It
     makes no wallet until Keystore (L13.3).
   - **Checked before release:** built from this record's commit; apksigner verifies its signature (scheme v3, which
     every phone it supports reads); it installed and ran on the Android 16 emulator on the owner's PC, through that
     image's ARM translation, and the core read "1.25 CGT" as 1,250 Sparks. It has not run on a phone, and L13.1 still
     says so.
3. **What the page says about QOR OS is this, and only this** (supersedes ADR-123 decision 2):
   - QOR OS is an operating system for players, game makers and creators: play, make and publish games, apps, themes
     and media on one machine, with one account and one library, driven by a controller as well as a keyboard.
   - **Its parts:** **Prism**, the shell, which is also the compositor, so windows and games are live parts of one
     scene, drawing nothing while nothing changes; **Lumen**, its design language; **QQ**, the QOR Engine, with QQ Studio
     and the QQ Player; **ARQADE** and **QOR Media**, running inside Prism; **its own controller service**, which reads
     the common console pads over USB and Bluetooth in one layout and drives the whole shell and QQ's games, a game in
     front having the pad to itself; the **Store**, of signed packages (games, apps, themes and widgets); **the QOR
     core**, this repository's code for keys, profiles, amounts and the chain, built into QOR OS at a pinned commit
     (ADR-120); and a **vault** that keeps keys in a service of its own, outside the shell and every app, whose approvals
     the compositor draws. The vault is designed and not yet built.
   - **Inside Prism**, as built: Beam, one search for apps, actions, settings and files, with a calculator; the Rail;
     the Shelf; Control Center; windows for its own apps and other Linux apps built for Wayland, each opening at its own
     size; three Spaces and Overview; the controller interface, with a cursor, focus, an on-screen keyboard and the
     glyphs of the pad in use; themes and widgets from the Store; several displays; three-finger gestures on touchpads
     and touchscreens; notifications; and its own apps (Files, Music, Notes, Terminal, Monitor, Settings and the Store).
   - **Where it stands.** Phase 0, done: Prism, Lumen and the shell running nested on Linux with real applications, a
     controller and sound; the Store's first stage; QQ, ARQADE and QOR Media built and tested on Linux. Phase 1, now:
     done, that QQ, ARQADE and QOR Media are developed inside QOR OS, that Prism starts all three, lays out and names
     their windows, that every window opens at the size its app asks for, and that QQ plays with QOR OS's controllers;
     left, controller focus in ARQADE and QOR Media, Lumen's look on every app, and a Home and a Library made for the
     controller; its goal, Orb Run, a QQ game, playing inside Prism on Linux with a controller. Then, in order: one
     sign-in, chat and publishing from QQ Studio to the devnet, with the vault; booting on its own and playing the PC
     games people already own; outside developers shipping QQ games to it with public tools.
   - **There is no image or download yet.** Its code is private while it is built. QOR OS 1.0 moves no money: free
     items and test CGT on the devnet only.
   - It uses Demiurge's network as services (ADR-120 decision 4): QOR ID, DRC-369 and CGT.

   Durations, dates beyond those above, hardware and the names of other companies' products stay off the page, as
   ADR-123 had it, except the platforms a download needs (Windows, Android). A change to any of this is a later record.
4. **Downloads.** The page has a Download section, and the hero and the phone's menu offer both downloads, the one for
   the visitor's platform first. Each link goes to the release's file itself, beside its release notes, and the page
   gives each file's version, size, what it needs, who signed it, its SHA-256 and how to install it: QOR Mobile's
   (decision 2), and the launcher's `QOR-Launcher_0.1.17_x64-setup.exe` (`launcher-v0.1.17`, 88,784,650 bytes, SHA-256
   `6e1081ce0883f5a82dd262af08924cee858d017671bcb840ef6507ab7dbe4c44`, as GitHub records it). ARQADE in the browser is
   offered beside them, and QOR OS is shown with nothing to install. `site/README.md` says what changes with a new
   release of either.
5. **Structured data** (amends ADR-123 decision 3): QOR Mobile is a `MobileApplication` whose `downloadUrl` is its APK;
   the launcher's `downloadUrl` is its installer; QOR OS keeps no download address and no offer.
6. **The living background** (supersedes ADR-123 decision 4's second point): one WebGL shader behind the page, QOR OS's
   mark as a lens at the scale of the sky. Its ring is the edge of a dark mass that bends the light around it; a nebula
   of ember and gilt drifts behind it with stars, all seen through the lens; a tilted disc of burning matter circles
   the mass, its far side bent up over the ring; and the mark's tail, an ember, orbits just outside the ring. A finger or
   the pointer carries a small mass that bends the sky where it goes, and a tap sends a ring of light through it.
   Its rules:
   - it is decoration, and every word stays HTML;
   - reduced motion draws one still frame, and a hidden tab draws nothing;
   - a phone left untouched draws at half the rate, and the resolution drops by itself when frames come late;
   - without WebGL the painted ground stays;
   - the colours and faces are still `tokens.css`'s (ADR-108), and the Content Security Policy is unchanged.
7. **On a phone** the page is designed first for the hand. Below 860 pixels wide the navigation is a sheet opened from
   the bar, holding both downloads. Everything to tap is at least 44 pixels high. Rows of cards swipe sideways. The
   lens has a band of its own above the words.
8. **`site/scripts/check-site.mjs`** (amends ADR-123 decision 5) holds the page to this record as well:
   - QOR Mobile is described with its APK as its download;
   - every download link points to a file of a release of this repository whose tag matches its version;
   - each download's SHA-256 is on the page;
   - the page still says QOR OS has no image or download yet.

## Consequences

- People can install QOR Mobile from the homepage. A release whose APK does not match the page's hash is a fault the
  page shows, since the check ties the page to the release's tag.
- **The test key is a single point of loss**: if it is lost, testers uninstall QOR Mobile before the next release can
  install. The owner backs it up off the PC.
- Once `android-v0.1.0` is published, L13.8 is done for this first build; each later step of L13 ships through it.
- When QOR OS's facts change, the page changes only through a later record that restates decision 3.
