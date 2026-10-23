# ADR-111: The launcher's update channel: a signed manifest on GitHub, checked by the host, installed only when the person says

**Status:** **Accepted**, 22 October 2026, under the owner's delegation of engineering choices (`HANDOFF.md`, the
owner's direction of 13 September 2026), when the owner asked to "proceed with our current roadmap objectives" and
L6.2 was the next of them (HANDOFF §4 item 99's list). The owner may amend it. It carries out **L6.2** in
`docs/DIRECTION.md`. It decides **no economic value**.

**Relates to:** [ADR-046](ADR-046-the-launcher-is-an-app-host.md) (the host holds what matters; the interface asks),
[ADR-056](ADR-056-no-lock-screen.md) (nothing typed, nothing blocking),
[ADR-103](ADR-103-the-launcher-moves-to-qt-in-stages.md) (the Tauri launcher stays while the Qt shell grows), and L6.1
(code signing), which this does not replace.

## Context

Testers install the launcher from a GitHub pre-release (`launcher-v0.1.14`) and have no way to learn of a fix except
being told; each fix means a new download and a new install. DIRECTION's L6.2 asks for "an update channel that verifies
signatures before installing".

The launcher is Tauri 2.11. Tauri's updater plugin (`tauri-plugin-updater` 2.12, the release that fits this Tauri;
2.14 would bring a newer Tauri, a change of its own) fetches a JSON manifest, downloads the installer it names, checks
the installer's minisign signature against a public key built into the app, and on Windows runs the installer and
exits. Read in its source before choosing (22 October 2026):

- The signature is checked against the downloaded bytes before anything runs.
- Only a newer version is offered; downgrades are refused unless configured.
- Its check that the version a signature was made for is the version the manifest announces is opt-in
  (`requireSignedVersion`), and the Tauri CLI that builds this launcher (2.11.4) records no version in its signatures,
  only the file's name (`file:QOR Launcher_0.1.14_x64-setup.exe`). Turning the option on would refuse every update.

GitHub's `releases/latest` link skips pre-releases, and every launcher release so far is one.

## Decision

1. **The channel is one fixed address:** `https://github.com/QOR-MATRIX/demiurge-chain/releases/download/launcher-channel/latest.json`,
   an asset of a pre-release tagged `launcher-channel` that holds nothing else and is replaced at each release. Each
   version's installer stays in its own pre-release, `launcher-vX.Y.Z`, which the manifest names.
2. **Signed with a key of the project's own, checked twice.** The public key is in `tauri.conf.json`; the plugin
   verifies each download against it. Then the host verifies again, refusing an old-style (legacy) signature, and
   requires the signed file name to be exactly the announced version's installer (`QOR Launcher_<version>_x64-setup.exe`):
   the same protection `requireSignedVersion` gives, from what this CLI does sign. Only HTTPS; no "dangerous" option is
   set in any configuration a release is built from, and a test holds the configuration to it.
3. **The private key never enters the repository or a log.** It lives on the PC that builds releases, outside the
   repository (`%APPDATA%\Demiurge\updater\`), encrypted with a random password that is itself protected by Windows
   (DPAPI) beside it, so nobody types it (ADR-056's spirit). The owner keeps an offline copy of both: losing them means
   testers must reinstall by hand once; a leak alone is not enough to ship to anyone, since the manifest is published
   only by someone who can write this repository's releases.
4. **The host checks; the person decides.** The Rust host checks fifteen seconds after start and every six hours while
   running, unless the person turns that off in Settings (on by default, as fixes are). An update is never installed
   on its own: the launcher says a new version is ready, with its notes, and installs only when the person presses
   *Update and restart*. Windows then asks once for permission, since the launcher installs for every user of the PC,
   and the installer runs without further questions and reopens the launcher. The interface can only ask the host to
   check and to install; it can never choose the address, the key or the file.
5. **What rides with it.** QOR Media and ARQADE's app are in the installer, so they update with the launcher.
6. **Releases are made by a script,** `tools/qor-launcher/scripts/release.mjs`: it builds with the key, checks the
   signature it produced against the public key in `tauri.conf.json` and the file name against the version, writes the
   manifest, and, only when asked to publish, creates the version's pre-release and replaces the channel's manifest.
7. **Not this record:** Windows and Apple code signing (L6.1, the owner's certificates), so Windows' first-install
   warning remains; macOS and Linux updates, until those builds are released; a choice of channels (beta, stable).

## Consequences

- Testers on a launcher with this change get every later release from inside the launcher. Testers on 0.1.14 or older
  install the first such release by hand, once.
- The key is a new thing to keep: HANDOFF and OWNER say where it is and what to back up.
- A release becomes a script run; the manifest it writes is checked before it is published.
- When the Qt shell (ADR-103 stage 2) replaces the Tauri launcher, its updater keeps this channel and this key, or a
  later record says otherwise.
