# ADR-099: Each QOR ID on a PC is a profile of its own, with its own wallet, its own data and a PIN

**Status:** **Accepted**, 18 October 2026, by the owner: "QOR IDs need to be separate entirely. A new app sign in should
be entirely dependent on the data for that account. If an account logs out, that should separate the amounts
completely." Asked what should keep a signed-out QOR ID from someone else at the same PC, the owner chose **a PIN per QOR
ID** over Windows Hello and over nothing. The design below is the assistant's, under the owner's delegation.
**Supersedes** [ADR-098](ADR-098-each-qor-id-keeps-its-own-account.md). **Amends** [ADR-056](ADR-056-no-lock-screen.md)
(the vault still opens with nothing asked, but only the profile signed in last; a signed-out PC shows the profiles) and
[ADR-039](ADR-039-account-derivation-and-the-scheme-change.md) (a profile's wallet holds one account). It decides **no economic
value**.

## Context

ADR-098 kept one wallet per PC, holding every QOR ID's account, and labelled each account with its QOR ID. Two days of
use showed that was the wrong shape:

- **The amounts were never separate.** Every QOR ID's account came from one recovery phrase, so whoever used the
  launcher could see and spend all of them, and the Vault listed them side by side.
- **A "new" account was an old one.** The vault never stored the accounts it derived: each launch it held only the
  first, so *Add account* and ADR-098's *A new QOR ID* derived the second account again every time. On the owner's PC
  that second account was the one used since early October, holding 57,900 test CGT, so a "new, empty" QOR ID showed
  57,900 CGT.
- **Nothing personal was per QOR ID.** The avatar, trade partners, listing drafts, the Mesh's held work and payments,
  the Library's folders and the record of paid requests all sat in one folder for whoever was signed in.

## Decision

1. **A profile is one QOR ID's everything on this PC.** Each has its own folder (`profiles/<id>/` in the launcher's data
   directory, and the same under its local data directory) holding its wallet (`vault.qor`), its avatar, trade partners,
   listing drafts, record of paid requests, the bytes of its published work, its Mesh node, settings, held work and
   payments, and its Library index. QOR ID's tokens are kept in the keychain under the profile's id. Device settings
   (the chain, QOR ID and Social addresses, the theme, translation models) stay shared. `profiles.json` lists the
   profiles by id, name and address, and names the one signed in.
2. **A profile's wallet holds one account**, which is its QOR ID's. *Add account* is gone. A new profile has a new
   recovery phrase, so it starts empty and shares nothing with any other.
3. **Signing out closes the profile completely**: its QOR ID session ends, its tokens are deleted, the Mesh and Library
   stop, the wallet locks, and the key that opens it silently is removed from the keychain. Nothing of it stays on
   screen or in memory. The launcher then shows **Who's playing?**: this PC's profiles by name, *New QOR ID* and *Restore
   from a recovery phrase*. No balance is shown there.
4. **A PIN per profile**, at least six digits. Opening a profile from *Who's playing?* asks its PIN; the profile signed in
   last opens with nothing asked at launch, as ADR-056 decided. The PIN is chosen the first time the profile signs out,
   so a first run is still one button. The PIN does not merely gate a screen: the key that opens the wallet is kept in
   the profile's entry sealed under the PIN (Argon2id at 64 MiB and three passes, then XChaCha20-Poly1305, the vault's
   own seal), and is in the keychain in the clear only while the profile is signed in. After five wrong PINs each further
   try waits, doubling from 30 seconds. A forgotten PIN is replaced by restoring the profile from its recovery phrase.
5. **The wallet from before 0.1.13 becomes the first profile**, with its first account, signed in, and its personal files
   moved into its folder. Other accounts that phrase has used can each be made a profile of their own from Settings:
   the launcher looks at the phrase's next accounts on the chain and offers those that have been used, with their
   balances, and the new profile is that account alone. Those profiles share a phrase, as they always did; a profile
   made from now on never does.
6. **ADR-098's labelling is removed**: `account-owners.json`, the account labels, the "not this QOR ID's wallet"
   warnings and the per-account sign-in list. They described a shape that no longer exists.

**What this protects, and what it does not.** Someone at the PC who does not know a profile's PIN cannot open it in the
launcher, and cannot read its balance there. Copying the files is not enough either: the wallet's key is sealed under the
PIN. Someone who can run their own programs as the same Windows user, and spends days, can try every six-digit PIN against
the sealed key offline; Argon2id makes each try slow, not impossible. People who must be safe from each other in that
sense need separate Windows logins. A longer PIN raises the cost, and the launcher accepts up to twelve digits.

## Consequences

- Launcher 0.1.13. New: `src-tauri/src/profiles.rs`, the *Who's playing?* step in the Gate, and PIN dialogs; changed: the
  vault (one account, a profile's folder, sealing its key under a PIN), QOR ID's token entries, and every personal path.
- The Mesh grant (ADR-097) belongs to the profile, so a signed-out profile's Mesh stops signing and serving.
- ADR-098's criterion `alpha.accounts-view` and its suite are replaced by `alpha.profiles-view` on `profiles_view`,
  which holds the new screens to this record. The change is logged in `docs/GATES.toml`.
- `qor://pay` pays from the signed-in profile's account, the only one open.
