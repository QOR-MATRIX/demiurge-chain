# ADR-098: Each QOR ID on a PC keeps its own account, and the launcher says whose account is on screen

**Status:** **Superseded** by [ADR-099](ADR-099-each-qor-id-is-its-own-profile.md) the same day, by the owner: "QOR IDs need to be separate entirely." Each QOR
ID is now a profile with its own wallet, its own data and a PIN; the labelled accounts of one wallet below no longer
exist. It was built and published as launcher 0.1.12. Accepted, 18 October 2026, under the owner's standing delegation
for engineering choices, after the owner found one wallet's balance shown under two QOR IDs on one PC. **Amends** [ADR-016](ADR-016-sign-in-with-unlock.md):
the arrival grant covers the account launch signs in with, which is no longer always the vault's first. **Amends**
[ADR-090](ADR-090-the-account-a-publish-signs-with.md)'s "Unchanged" note: QOR ID sign-in and `qor://pay` no longer use
the first account. It decides **no economic value**.

## Context

CGT belongs to an account's key, and a QOR ID is a name bound to one key: QOR ID keeps one QOR ID per chain account. A
launcher's vault holds one recovery phrase and any number of accounts derived from it (ADR-039).

Until launcher 0.1.11 three things used the vault's first account, whatever else was true: opening the vault signed in
to QOR ID with it (ADR-016), the interface showed it after every launch, and `qor://pay` paid from it. On a PC where two
QOR IDs had been used, one of them bound to a second account, launch always signed in as the first account's QOR ID; a
sign-in as the other left the first account on screen, so the other QOR ID's name sat over the first account's balance,
with nothing saying so. A transfer from that screen would spend the first account's CGT, and a tip from ARQADE signed in
as the second QOR ID would have been paid from the first account.

## Options

**A. Keep the first account, and only warn.** *Costs:* a person with two QOR IDs can never launch as the second; every
launch shows the warning.

**B. Separate vaults per QOR ID.** *Costs:* a second phrase to write down and keep, a vault chooser before the shell
(ADR-056 has no lock screen to put it on), and every command made vault-aware. Large, and it buys nothing a Windows user
account does not already give.

**C. One vault, each account labelled with its QOR ID; the account in use follows sign-in and is remembered.** *Costs:*
a small file of public facts beside the vault; the webview can ask for a sign-in with any account of the open vault.

## Decision

**Option C.**

1. **The launcher remembers which QOR ID each account was signed in as**, and the account last in use, in
   `account-owners.json` beside the vault (`identity::owners`). It holds addresses and names, both public, and is sent
   nowhere. A QOR ID seen bound to a different account is forgotten on the old one: a QOR ID has one account.
2. **Launch signs in with the account last in use** while the vault holds it, else the first. The arrival grant
   (ADR-016) covers that account. `qor_sign_in` takes an account, so Settings can sign in as any account's QOR ID. It
   opens the grant for the account chosen, as it has opened it for the first account since ADR-056, so Settings'
   sign-in asks nothing; what the webview could do for the first account it can now do for each. Every other key
   signature keeps its host dialog.
3. **Signing in makes the QOR ID's own account the one in use**, when the vault holds it. One it does not hold leaves
   the account in use as it was, and the interface says so.
4. **The interface says whose account is on screen.** The Vault labels every account with its QOR ID; when the account
   on screen is not the signed-in QOR ID's, the Vault says whose it is and that sending spends its CGT, and offers the
   QOR ID's own account in one click; the rail's account card says so too.
5. **A name is claimed only for the account that has none**, never for the first by default.
6. **A new QOR ID is made on a new account** (`qor_new_identity`): derived, then registered in one host dialog, so it
   starts with its own empty balance and never shares another QOR ID's CGT.
7. **`qor://pay` pays from the account in use** (ADR-090 left this open). After sign-in that is the signed-in QOR ID's
   own account. The payment dialog names the paying account, as before.

**What this is not:** a separation between people. Every account of one vault comes from one phrase, and whoever uses
the launcher can sign with any of them. Two people who share a PC and must not reach each other's CGT need separate
Windows user accounts, each with its own keychain entry and data directory, and so its own vault.

## Consequences

- Built in launcher 0.1.12: `src-tauri/src/identity/owners.rs` (with its test), `arrive`, `adopt`, `qor_sign_in`,
  `qor_new_identity`, `vault_owners` and `pay_link` in `src-tauri/src/lib.rs`; `src/state/store.ts` (`setAccounts`,
  `setSession`), `src/views/VaultView.tsx` (`Whose`), `src/views/SettingsView.tsx` (`SignIn`) and the rail.
- A new criterion, `alpha.accounts-view`, on the suite `accounts_view` (`scripts/check-accounts-view.mjs`, 21 checks),
  logged as a tightening in `docs/GATES.toml`. Two planted faults, the first account chosen at arrival and a name claimed
  for the first account, were each caught.
- A new QOR ID's account is derived before its dialog. Declined, or refused by QOR ID, it stays in the vault with no
  QOR ID, and Settings offers it a name like any other such account.
- The Mesh grant (ADR-097) stays with the account it was given for, whichever QOR ID is signed in.
- Nothing on the chain or in QOR ID changes.
