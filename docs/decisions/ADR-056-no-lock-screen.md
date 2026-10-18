# ADR-056: No lock screen: the vault's key is in the keychain, and QOR ID never blocks the launcher

**Status:** Accepted, 28 September 2026, by the project owner.
**Supersedes:** [ADR-055](ADR-055-the-vault-opens-with-windows-hello-alone.md) (Windows Hello as the vault's only
seal) and, with it, what remained of [ADR-054](ADR-054-unlocking-with-windows-hello.md).
**Amends:** [ADR-016](ADR-016-sign-in-with-unlock.md). Opening the vault still opens the arrival grant for QOR ID
challenges, but opening it no longer asks the person anything, and the launcher no longer waits for QOR ID.
**Amended by** [ADR-099](ADR-099-each-qor-id-is-its-own-profile.md) (18 October 2026, the owner): each QOR ID on a PC is a profile with its own wallet. The profile
signed in last still opens with nothing asked; once it signs out, its key leaves the keychain and the launcher shows
*Who's playing?*, where a profile opens with its PIN.

## Context

Hours after ADR-055 the owner moved their vault to Windows Hello and met the next wall: the launcher showed the
shell only when the vault was open **and** QOR ID had signed in, and QOR ID was not running. The default QOR ID
endpoint is `demiurge.cloud`, which is not deployed, so without a local service no one could get past the Gate.
The owner: "GET RID THE FUCK OF THE FUCKING VAULT AND WINDOWS HELLO … CLEARLY IT IS NOT FUCKING DOING ANYTHING BUT
MAKING IT IMPOSSIBLE TO LOGIN."

Removing the vault would not have signed anyone in: QOR ID needs its service whatever the launcher does. And the
keys cannot go, because they prove the QOR ID and sign every transaction. What could go was every prompt and every
wait. That is what this decides.

## Decision

1. **The vault's key is in the operating system's keychain.** `vault.qor` holds the recovery phrase sealed
   (Argon2id, then XChaCha20-Poly1305, as before) under a random 32-byte key, and the file names the keychain
   entry, `vault-key:<id>`, under the service `cloud.demiurge.qor-launcher` beside QOR ID's tokens. The key is read
   with no prompt: Windows Credential Manager, the macOS Keychain or the Secret Service. No passphrase, no Hello.
2. **There is no lock screen.** The launcher opens the vault before its first frame. There is no idle lock and no
   Lock button; the vault stays open while the launcher runs.
3. **QOR ID never blocks the launcher.** The shell shows once the vault is open. Opening it tries to sign in (the
   arrival grant, ADR-016); a service that is down stops nothing. Settings offers *Sign in to QOR ID* with the
   vault's key, and a name to claim when the key has none. The launcher has no password sign-in.
4. **A first run is one button.** *Begin* makes the phrase and the vault. The phrase is not shown then; Settings
   shows it on request.
5. **Showing the recovery phrase asks in a host dialog**, which the webview can neither draw nor answer.
6. **Older vaults move once.** A Windows Hello vault (ADR-055) asks Hello one last time; a passphrase vault asks
   for its passphrase one last time, or moves with Hello alone if its ADR-054 copy belongs to it. The old file is set
   aside, never deleted.
7. **The recovery phrase is the way back** when the keychain loses the key (a new computer, a reset Windows
   profile).

## What this gives up, which the owner accepted

- **Anyone using the person's signed-in computer account can open the vault**, and so can any program running as
  that person. Under ADR-055 each opening needed the person's face, fingerprint or PIN. The owner's words, and
  the Gate being the only thing between them and the launcher, decide this.
- **Nobody is made to write the recovery phrase down.** A person who never opens Settings has no backup if the
  keychain loses the key. This is the owner's trade for a first run with nothing to do; it is the first thing to
  revisit before anything of value is held.
- **What still holds:** a copied `vault.qor` opens nothing without the account's keychain; every transfer, trade,
  mint, endpoint change and test run still asks in a host dialog (L1.4), and so does showing the phrase.

## Alternatives considered

- **A plain key file, or the phrase unsealed on disk.** Rejected: a file that opens the vault, once copied by a
  backup, a sync folder or malware, holds the keys on any machine for ever. The keychain gives the same
  zero-prompt opening without that.
- **DPAPI through Windows' WinRT `DataProtectionProvider`.** Equivalent on Windows, but Windows-only and a new
  surface; the `keyring` crate was already a dependency and covers macOS and Linux too.
- **Starting QOR ID locally instead.** Offered to the owner and not chosen: it would have signed them in only while
  a local service ran, and left the Gate waiting on it.

## Consequences

- `vault/keychain.rs` is the `KeyStore` (the OS keychain, and an in-memory one for tests, plus an ignored test
  against the real keychain). `vault/hello.rs` is read only to move ADR-055's vaults. `VaultStatus` is
  `Absent`, `Locked { sealed_with: keychain | hello | passphrase }` or `Unlocked { accounts }`.
- Commands: `vault_create(phrase)`, `vault_restore(phrase)`, `vault_unlock()`, `vault_move_to_keychain(passphrase)`,
  `vault_export_phrase()` (host dialog) and `qor_sign_in()`. `touch_vault` and `vault_hello_available` are gone.
- The launcher works on macOS and Linux again, which ADR-055 had removed.
