# ADR-016: Unlocking the vault signs in to QOR ID without a second approval

**Status:** Accepted, 14 September 2026, under the project owner's delegation. The owner asked for sign-in
and sign-up with as little friction as possible, without weakening security, and left the design to the
assistant. **Amended by [ADR-055](ADR-055-the-vault-opens-with-windows-hello-alone.md)** (28 September 2026, the
owner): there is no passphrase, and the approval below is given by the Windows Hello gesture that opens the
vault. **Amended again by [ADR-056](ADR-056-no-lock-screen.md)** the same day: the vault opens with nothing asked,
so the grant rests on the person being signed in to their computer, and the launcher no longer waits for QOR ID.
**Amended by [ADR-098](ADR-098-each-qor-id-keeps-its-own-account.md)** (18 October 2026): the grant covers the
account launch signs in with, the one last in use on the PC, which is no longer always the vault's first. ADR-098 was
superseded the same day by [ADR-099](ADR-099-each-qor-id-is-its-own-profile.md): the grant covers the signed-in profile's one account.
The decision stands otherwise.
**Narrows:** roadmap item L1.4 in [`DIRECTION.md`](../DIRECTION.md), host-side confirmation before any
signature, by one named exception. Every other part of L1.4 stands.

## Context

The launcher has two layers, and until this decision the person met both as separate logins:
- **The vault** is local key custody: the recovery phrase, sealed on this device with Argon2id and
  XChaCha20-Poly1305 and opened by a passphrase. Nothing is sent anywhere when it opens.
- **QOR ID** is the account on the identity service. With a vault key, signing in means answering a
  one-time challenge with a signature, so the service never holds anything reusable.

Under L1.4 every signature needs approval in a dialog the host draws. A returning person therefore typed
the passphrase, pressed "Sign in with your vault key", and approved a dialog: three acts for one intent.
A new person created the vault, went to the sign-in screen, chose "Claim one", entered a name and approved
a dialog.

The owner's question was why there are two logins at all. The layers are both needed. The steps are not.

## Decision

**Typing the correct passphrase is the approval for signing in.**

- Unlocking or creating the vault opens an **arrival grant** for the vault's first account.
- While the grant is open, that account answers QOR ID challenges, at sign-in and when claiming a name,
  without a dialog.
- The grant closes when a QOR ID session is established, when the vault locks, or five minutes after it
  opened, whichever comes first.
- The launcher uses it at once. Unlocking keeps a QOR ID session that is still valid, and otherwise signs
  in with the key. If the key has no QOR ID yet, the only thing left to ask is a name.
- **Nothing else uses the grant.** Transfers, starter-grant claims, wallet links, endpoint changes and test
  suite runs each keep their dialog. An identity signature outside the grant, such as a key sign-in after
  QOR ID was unreachable at unlock, asks as before.

The result: a returning person types a passphrase and is in. A new person writes down the phrase, sets a
passphrase and chooses a name.

## Why this does not weaken security

- **The passphrase is a stronger approval than the dialog.** It is the secret that opens the vault; the
  dialog is a button. The grant exists only because the passphrase was just typed correctly.
- **What the grant can sign is narrow.** A QOR ID challenge carries the domain tag
  `demiurge:qor-id:challenge:v1:`, and the launcher refuses to sign anything that is not in the service's
  challenge format. Such a signature cannot move CGT or authorise anything but a session on the configured
  identity service, and changing that service still needs a dialog.
- **The worst a compromised webview gains is small and bounded.** Inside the five-minute window after an
  unlock, it could sign the person in to their own account, or claim a name for a key that has none. It
  cannot obtain the key, a transfer, or any signature once the window closes.

## Alternatives considered

- **Opening the vault without a passphrase, with the operating system holding the key that unlocks it.**
  Rejected as a default: anyone who can use the person's operating-system account would open the vault.
  An opt-in unlock with Windows Hello or Touch ID, which checks that the person is present, is a possible
  later decision.
- **A PIN for switching accounts.** Rejected. Every account in a vault comes from one recovery phrase and
  unlocks with it, so a PIN between them protects nothing. A PIN that opened a vault would be far easier to
  guess than the passphrase.
- **No dialog for any identity signature.** Rejected: a compromised webview could then obtain identity
  signatures at any time while the vault is unlocked, not only in the moments after the person typed the
  passphrase.

## Consequences

- The Gate shows one field to a returning person, and a name field to a new one. The sign-in screen
  remains only as the fallback when QOR ID cannot be reached at unlock, or for accounts that use a
  password.
- L1.4's roadmap text names this exception. Exercising L1.4 in a running launcher now includes unlocking
  with no dialog, and a key sign-in outside the grant that still asks.
- An account picker would extend the grant to the account the person picks. That is for its own roadmap
  item.
