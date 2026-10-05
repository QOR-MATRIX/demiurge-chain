# ADR-027: Identity stays off chain in QOR ID; only the account is on chain

**Status:** Accepted, 15 September 2026, by the project owner, who took option C.
**Resolves:** migration inventory question Q-10 ([`MIGRATION_INVENTORY.md`](../architecture/MIGRATION_INVENTORY.md)
§3.4, F-Q7).
**Follows:** [ADR-017](ADR-017-password-accounts-no-chain-identity-without-a-key.md), under which a chain identity
exists exactly where a key the holder proved exists.

**Identity form changed, 5 October 2026, by [ADR-075](ADR-075-one-name-per-qor-id.md):** a QOR ID is the username
alone, unique on its own; the `#0001` discriminator below is retired. The text is kept as it was written.

## Context

The custom chain's `qor-identity` module holds DID records and handles. Nothing dispatches it, it is not in
the state root, and it checks no authorisation on create, transfer or reserve (inventory §3.4; Verified repo).
Nothing in it is carried.

The inventory set out three options for QOR ID on the new chain:
- **A. `pallet-identity`.** A registrar-and-judgement model with deposits, sub-accounts and username authorities.
  It holds no keys and delegates no signing (Verified SDK, 14 September 2026).
- **B. A custom handle pallet.**
- **C. Identity stays off chain** in `services/qor-auth`, with only the account on chain.

## Decision

**Option C.** A person's identity (username, discriminator, email, profile) lives in QOR ID. The chain holds
accounts and nothing that names the person behind one. No runtime pallet records QOR ID names, handles or
judgements.

## Alternatives rejected

- **A. `pallet-identity`.** Its model is registrars that judge identity claims for a fee, and deposits held
  against identity fields. QOR ID is neither. To use it, QOR ID would have to become a registrar issuing
  judgements it does not make, or a username authority granting names under rules built for another system.
  Bending the pallet that way would be worse than either alternative: a standard component used against its
  own model, where its behaviour stops matching what its name promises.
- **B. A custom handle pallet.** A bespoke on-chain name registry has to settle squatting, deposits, transfer,
  expiry, and the permanent public link between a name and an account, before anyone needs handles on chain.
  The first release does not.

## Consequences

**What the public viewer loses** (ADR-011) by not having handles on chain:
1. **A creator's name is not chain state.** The viewer can read the creator's account from the chain, but the
   name beside it comes from QOR ID, a service the chain does not attest. ADR-011 says the creator must be on
   chain for a page to be honest. The account is; the name is not. The viewer labels a name as QOR ID's.
2. **The viewer depends on QOR ID for names.** If QOR ID is unreachable, pages show accounts only.
3. **Names have no on-chain history.** A renamed or deleted QOR ID account changes what an older page shows. The
   provenance chain stays exact, because it is made of accounts.
4. **Other explorers and wallets show addresses only.** Third-party tools read `pallet-identity`, and would find
   nothing, so they show the SS58 address (ADR-024).
5. **A public lookup from account to name is new.** Today QOR ID has no unauthenticated route that maps a chain
   account to a username. The viewer needs one, and it publishes which person holds which account. Whether a name
   is shown by default or only when the holder opts in must be decided before the viewer is built.

**What adding handles later would take**, if Demiurge decides it wants them:
- **Either** `pallet-identity` with QOR ID as a username authority, **or** a custom handle pallet. Each is its own
  ADR, and a custom pallet also needs its name approved.
- **A runtime upgrade,** not a hard fork. The pallet is added under governance (ADR-021).
- **Each holder signs their own claim.** Names cannot be copied from QOR ID onto accounts wholesale: the chain
  would then trust QOR ID's say-so about who holds which account, which ADR-017 and R-3's reasoning keep out.
- **Rules settled then:**
  - uniqueness against QOR ID's `username#discriminator` form;
  - deposits and whether sponsorship covers them (ADR-029);
  - transfer, expiry and squatting;
  - which of QOR ID and the chain is the source of truth for a name.
- **Nothing in the first release blocks it,** provided the DRC-369 wire format and the SDK do not assume a name
  field on chain.

**The inventory's `qor-identity` row** becomes Not carried.
