# ADR-039: Derivation paths, and what the key-scheme change does to accounts that already exist

**Status:** Accepted, 19 September 2026. **Amended by** [ADR-099](ADR-099-each-qor-id-is-its-own-profile.md) (18 October 2026): a profile's wallet holds one
account, so further accounts are no longer derived in the launcher. The derivation paths stand, for wallets from before
0.1.13, whose other accounts become profiles of their own. [ADR-023](ADR-023-sr25519-with-ecosystem-derivation.md) left the form of
further accounts "to that launcher work"; this is that work, and this record is the rest of what carrying it out
decided.
**Accepted under the owner's standing delegation for engineering choices.** Nothing here is a value
[`OPEN_QUESTIONS.md`](../economics/OPEN_QUESTIONS.md) leaves open, and nothing here is reserved to the owner. The
mainnet address prefix stays the owner's, as a Public Release criterion (ADR-024).
**Completes:** [ADR-023](ADR-023-sr25519-with-ecosystem-derivation.md) (Sr25519, derived as the ecosystem derives),
[ADR-024](ADR-024-ss58-addresses-prefix-42-until-mainnet.md) (SS58 addresses) and
[ADR-017](ADR-017-password-accounts-no-chain-identity-without-a-key.md) (no chain identity without a proven key),
which all said they would land with the Substrate work.
**Refines:** ADR-024's migration specification on one step, and says so below.

## Context

ADR-023 chose Sr25519 keys derived from the recovery phrase's entropy, exactly as `sp-core` and Polkadot.js derive
them, so that one phrase opens the same accounts in the launcher, Polkadot.js, Talisman and Nova (ADR-009). It fixed
the first account as the phrase with no derivation path and left three things to the work that carried it out: the
form of further accounts, the signing context to confirm, and what happens to keys already bound under the old
scheme.

ADR-024 specified how QOR ID accepts SS58, including a backfill of the new column from the old one.

What the code did before this change (verified in the repository on 19 September 2026):
- the launcher derived Ed25519 keys by SLIP-0010 at `m/44'/369'/account'/0'/index'` and displayed `0x` and 64 hex
  characters;
- QOR ID verified Ed25519 strictly, stored `primary_pubkey` as hex and `on_chain_address` as `0x` plus the same hex,
  and derived an address for password-only accounts by hashing the name, the discriminator and the clock.

## Decision

1. **Further accounts are the hard junctions `//0`, `//1`, …**, from the phrase's root. The first account is the
   bare phrase, as ADR-023 fixed, so the launcher's second account is `//0`.
   - **That offset is Talisman's**, read from its source on 19 September 2026
     (`apps/extension/src/ui/domains/Account/DerivedFromMnemonicAccountPicker.tsx`): `return index === 0 ? "" :
     \`//${index - 1}\``, under the comment "for substrate, first account should have an empty derivation path".
     Matching it means a phrase imported into Talisman lists the launcher's accounts in the same order with none in
     between; numbering them `//1`, `//2` instead would have left Talisman showing an extra account, `//0`, that the
     launcher never shows and a person could easily pick by mistake.
   - Each junction is built from the path string a person would type, and a test pins every account against
     `sp_core::sr25519::Pair::from_string("<phrase>//n")`.
2. **The signing context is `sp-core`'s, confirmed rather than inferred.** ADR-023 recorded `b"substrate"` as
   Inferred. `sp_core::sr25519::Pair::sign` signs under schnorrkel's `substrate` context at the pinned release, and
   both sides use `sp-core` itself rather than reimplementing it, so there is nothing left to infer. QOR ID and the
   launcher take `sp-core` at the version the chain is pinned to (ADR-033 rule 1).
3. **Nothing is backfilled. Every key and address already stored is cleared.** This refines ADR-024's migration step
   1, which specified a backfill from `on_chain_address`.
   - Every stored key is an Ed25519 public key. Under Sr25519 those same 32 bytes name an account nobody holds a
     secret for, so a backfilled row would bind a QOR ID to an account it can never prove again — and replacing a
     linked account is refused by design, so that account could not be corrected.
   - ADR-023 already said these links "must be linked or registered again". Clearing them is what makes that
     possible.
   - **What it costs:** an account created with a key alone cannot sign in afterwards. Its password is a random value
     nobody knows and its key no longer verifies, so it has to be created again. Nothing is deployed, no network
     holds value, and ADR-023 recorded that this is the cheapest this change will ever be.
4. **One column, not two.** `primary_pubkey` and `on_chain_address` held the same fact in two encodings and could
   disagree. They are replaced by `chain_account_id BYTEA`, 32 bytes, unique, null until a key is proven. For
   Sr25519 the account ID *is* the public key, so the key and the account are one value. ADR-024 asked for
   `on_chain_address` to be dropped "once no client reads it": every reader in the repository was changed in the same
   commit and nothing is deployed, so both columns go in migration 018 rather than in a later one.
5. **An account is named in one of two fields, never one that takes both forms.** `address` is SS58 at the chain's
   prefix; `account_id` is `0x` hex, the advanced form ADR-024 asks for. Exactly one is given: hex in the address
   field is refused, and so is SS58 in the account-ID field, so an interface that means to take an address cannot
   quietly take a raw account ID.
6. **The prefix is configuration in QOR ID and a constant in the launcher.** QOR ID reads `chain.ss58_prefix`,
   defaulting to 42, as ADR-024 requires. The launcher carries one constant that mirrors the runtime's `SS58Prefix`,
   because it must render an address with no node reachable; when the chain client is rebuilt (L3.1) it reads the
   prefix from the node's `system_properties` and the constant becomes the offline fallback.
7. **The challenge is not yet bound to the network.** ADR-024 specified that the signed challenge carry the chain's
   genesis hash and network name. It is not implemented, because there is no deployed network whose genesis hash
   could be configured, and a value invented now would be wrong for the chain eventually run. The domain tag
   (`demiurge:qor-id:challenge:v1:`) is unchanged, so a challenge signature proves possession of a key and says
   nothing about a network. That is inventory finding F-Q9, still open, and it is recorded in the code beside the
   tag rather than only here.
8. **The launcher's chain client refuses to sign for the custom devnet.** That chain verifies Ed25519 and its account
   ID is an Ed25519 public key, so no signature this vault makes can be accepted by it. A transfer or a starter claim
   now refuses with that reason, before a person is asked to approve anything, rather than asking for a signature the
   node will reject. The read-only calls are unchanged. Roadmap L3.1 replaces the module.

## Alternatives rejected

- **A BIP-44-shaped path (`//44//369//n`).** Nothing in the ecosystem opens accounts that way except Ledger, and a
  person could not retype it anywhere useful. `//n` is what wallets show and what `subkey` documents.
- **Building the junction from a value instead of from the path string.** `DeriveJunction::hard("0")` encodes the
  *string* `"0"`, length prefix and all, and is a different account from `//0`, which encodes the number. The two are
  indistinguishable by eye and produce plausible-looking keys either way. Building each junction from the string a
  person would type sends it through `sp-core`'s own parser, and a test asserts the two are not the same account, so
  the mistake cannot be made quietly.
- **Backfilling the old keys anyway** (ADR-024 as written). It preserves a row that can never authenticate again and
  cannot be corrected, which is worse than asking for one re-link on data nobody holds value in.
- **Keeping `primary_pubkey` beside the new column.** Two representations of one fact is the thing ADR-024's "bytes,
  not a string" reasoning is against.
- **Inventing a genesis hash or a network name for the challenge now.** A challenge bound to the wrong network is
  worse than one bound to none, because it looks like the protection it is not.

## Consequences

- The launcher's vault, QOR ID's routes and the migration all move together; a client that sends the old `pubkey`
  field gets a validation error naming the two fields that exist.
- **Registration creates no address at all** (ADR-017). `AuthService::hash_to_address` is removed, so nothing can
  derive one again, and the profile reports `null` until a key is proven.
- **ADR-023's open confirmation is closed** (19 September 2026). It asked that Talisman and Nova be checked before
  this work shipped.
  - **Talisman: Verified from source**, as quoted above. Its Sr25519 derivation applies the junctions of a
    derivation path to the mini secret from the phrase, and an empty path is the bare phrase.
  - **Nova: Verified from documentation.** Its wiki calls the account generated from the phrase with no derivation
    path the "Root Key", and the Polkadot support article on Nova's advanced options states that the default
    derivation path for Substrate accounts is blank (Ethereum accounts use `//44//60//0/0/0`).
  - **Polkadot.js: Verified**, as ADR-023 recorded.
  - What is *not* verified is any of this against a running build of either wallet with a real phrase. That is a
    five-minute check for whoever next has one in front of them, and it is worth doing before anything of value is
    held.
- The Ed25519 forgery test stays, as ADR-023 point 5 requires. It now measures both halves: the forgery is refused by
  the service, and strict Ed25519 verification refuses the same bytes, so a change that reintroduces Ed25519
  verification cannot reintroduce the forgery unnoticed.
