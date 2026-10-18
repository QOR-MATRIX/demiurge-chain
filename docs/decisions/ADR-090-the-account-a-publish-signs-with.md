# ADR-090: The account a publish signs with

**Status:** **Accepted**, 15 October 2026, by the owner, who chose the recommendation put to them ("Go with best
recommendation"). Built the same day. **Amended by [ADR-098](ADR-098-each-qor-id-keeps-its-own-account.md)**
(18 October 2026) in its "Unchanged" note only: QOR ID sign-in and `qor://pay` no longer use the first account. Since
[ADR-099](ADR-099-each-qor-id-is-its-own-profile.md) a profile's wallet holds one account, which signs in, pays and publishes.

**Relates to:** [ADR-088](ADR-088-a-cartridge-holds-a-commit.md) (QQ Studio publishes a project as its Cartridge through
the launcher; one game, one Cartridge), [ADR-046](ADR-046-the-launcher-is-an-app-host.md) (the launcher hosts products),
[ADR-016](ADR-016-sign-in-with-unlock.md) and [ADR-076](ADR-076-arqade-hands-payments-to-the-launcher.md) (the first
account signs in to QOR ID and pays `qor://pay` requests; unchanged here).

## Context

ADR-088 decides what a Cartridge holds and that QQ Studio publishes through the launcher; it does not say which of the
vault's accounts signs. As built on 10 October 2026, a publish always signed with the vault's **first** account, and
looked for the project's Cartridge only among that account's assets.

Every other thing the launcher signs from its own screens (sending CGT, minting from Projects, selling, buying, trading)
signs with the **active** account: the one the rail shows, which the person picks in the Vault. It is the first until
the person picks another; accounts beyond the first last for the session (`Vault::add_account`), so the two differ only
when the person has added an account and picked it.

On 15 October 2026 the owner's first publishes were refused because the account that signed held nothing. The launcher
now says which account that is before anything is asked (HANDOFF §4 item 74), but a publish that signs with an account
other than the one on screen is the trap that refusal reports. The owner was asked which account publishing should use.

Two facts constrain the answer. **Only the holder of an asset can revise it**, so once a Cartridge exists the account
that revises it is fixed by the chain, not chosen. And **one game has one Cartridge** (ADR-088 decision 2): a publish
that looked for the Cartridge under the wrong account would find none and mint a second.

## Options

**A. The first account, always** (as built). *Costs:* a publish signs as an account the person may not be looking at,
while everything else in the launcher signs as the one they are; a Cartridge minted from Projects under another account
is not found, and Publish mints a second one. *If it changes later:* only the launcher changes.

**B. The account the rail shows, always.** *Costs:* picking another account in the Vault and then publishing finds no
Cartridge under it and mints a second one, splitting the game's history across two assets, which ADR-088 forbids.

**C. The holder revises; a first publish signs as the account the rail shows (recommended).** The launcher looks for the
project's Cartridge among **all** the vault's accounts. If one holds it, that account revises it, whatever the rail
shows. If none does, the first publish signs as the active account, as every other launcher action does. *Costs:* one
chain read per account in the vault, a handful at most; and the host must be told which account is active, since until
now only the interface knew. *If it changes later:* only the launcher changes; no Cartridge on the chain is affected.

## Decision

**Option C.**

1. **A revision is signed by the vault account that holds the project's Cartridge.** The Cartridge is the asset any of
   the vault's accounts holds whose commit is nearest HEAD in the project's history (ADR-088's rule, over every account
   rather than one).
2. **A first publish is signed by the active account**, the one the launcher's rail shows. The interface tells the host
   whenever it changes (`vault_set_active`), the host accepts only an account its open vault holds, and a vault that is
   locked and opened again starts from the first account, as the interface does.
3. The launcher's dialog already names the signing account (`Owner:`), so the person sees which account signs before
   approving.

## Consequences

- Built in `tools/qor-launcher/src-tauri/src/host/studio.rs` and `vault/mod.rs` (`Vault::set_active`,
  `Vault::active_address`), with the command in `lib.rs` and the interface's side in `src/state/store.ts`. The live
  publish test (`host::tests::a_project_is_minted_then_revised_then_left_alone_through_the_channel`) publishes from a
  second, picked account, then revises with the first account picked; it failed against both wrong signers planted in
  turn (the first account for a first publish, the active account for a revision).
- **Unchanged:** QOR ID sign-in (ADR-016) and `qor://pay` (ADR-076) still use the first account. Whether a payment
  should follow the rail as well is not decided here.
- Nothing on the chain changes, and no Cartridge already minted is affected.
