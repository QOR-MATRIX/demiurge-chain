# The chain's address type: `IdentityLookup` or `AccountIdLookup`

**Status:** **Decided, 20 September 2026.** The owner chose `AccountIdLookup`, the recommendation in §5, and it
was carried out the same day. The decision is recorded in
[ADR-041](../decisions/ADR-041-multiaddress-and-accountidlookup.md), which also records what it actually cost
against §4's estimate. Question Q-17 is resolved and finding F-Q10 is closed.

The rest of this document is kept as the record of the proposal, with one correction marked in §4.

- **What was asked.** `chain/`'s runtime sets `frame_system::Config::Lookup` to
  `IdentityLookup<AccountId>`, so an extrinsic's address field is a bare `AccountId32`. The SDK's own
  default for a solochain, and every relay chain runtime, use `AccountIdLookup<AccountId, ()>`, whose
  address is `MultiAddress`. **Which should Demiurge use?**
- **Why it is being asked now.** It is a wire-format choice. Changing it after the format is frozen at
  **M5.1** breaks every signed transaction built against the old shape, including any SDK already in
  someone's hands. Before M5.1 it costs a runtime change and a test pass. After, it costs a
  coordinated release of everything that signs.
- **Why it is the owner's.** Writing this up turned up something worth the owner's attention on its
  own: **`IdentityLookup` here is not a template default.** It overrides one (§1). AGENTS.md §7 says a
  departure from a standard component needs a written reason, and there is no written reason on file.
  The owner asked for this not to be decided by default when the wire format freezes.
- **Recommendation: change to `AccountIdLookup`** (§5). It is the ecosystem's shape, and the cost of
  matching it is at its lowest right now.

This is the write-up of migration-inventory finding **F-Q10** and question **Q-17**. It is a proposal,
in the same sense as [`SPONSORSHIP.md`](SPONSORSHIP.md) and
[`AGENT_KEY_CUSTODY.md`](AGENT_KEY_CUSTODY.md): the owner decides, and the decision becomes an ADR.

Labels: **V** means verified in the code or the SDK for this document; **I** means inferred.

---

## 1. What the code does today

`chain/runtime/src/lib.rs`:

```rust
type Lookup = IdentityLookup<AccountId>;                                            // :181

pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, TxExtension>;    // :91
```

The first type parameter of `UncheckedExtrinsic` is the **address**, and it is `AccountId`. **V.**

Two consequences follow, both observed while building the launcher's client (ADR-040):

- **The address field is 32 bytes**, with no variant byte in front of it. A client that assumes
  `MultiAddress` writes 33 and the node refuses the transaction. **V** (a test in
  `tools/qor-launcher/core/src/chain/config.rs` pins the 32).
- **`Balances`' `dest` argument is a bare account too**, not a `MultiAddress`. **V** (a live transfer
  against `demiurge-node --dev`, 2026-09-20).

### This line overrides the SDK's default; it does not inherit one

This was assumed to be a template default when finding F-Q10 was first written, on the day the
launcher's client was built. Checking it for this document showed otherwise. At the pinned
`polkadot-stable2606-1` (all **V**, fetched from the tag):

- `frame_system::config_preludes::SolochainDefaultConfig` — the prelude the SDK's **`solochain` and
  `minimal` templates both derive from, and the one a new solochain is meant to start from — sets
  `type Lookup = AccountIdLookup<Self::AccountId, ()>`
  (`substrate/frame/system/src/lib.rs:399`, under `pub struct SolochainDefaultConfig` at `:382`).
- The prelude that sets `IdentityLookup` is `TestDefaultConfig`
  (`substrate/frame/system/src/lib.rs:337`), whose `AccountId` is `u64`. It is for pallet unit tests,
  not for a chain.
- The Westend and Rococo runtimes declare `pub type Address = sp_runtime::MultiAddress<AccountId, ()>`
  (`polkadot/runtime/westend/src/lib.rs:1936`, `polkadot/runtime/rococo/src/lib.rs:1642`).

`chain/runtime/src/lib.rs:181` writes `type Lookup` explicitly, so it **replaces** the
`SolochainDefaultConfig` value the runtime derives everything else from. Whatever the reason was, it
was a choice, and it is not recorded anywhere. **V.**

## 2. What `MultiAddress` actually is

```rust
enum MultiAddress<AccountId, AccountIndex> {
    Id(AccountId),          // 0: the account, 32 bytes
    Index(AccountIndex),    // 1: an index from pallet-indices
    Raw(Vec<u8>),           // 2
    Address32([u8; 32]),    // 3
    Address20([u8; 20]),    // 4
}
```

(`substrate/primitives/runtime/src/multiaddress.rs:28`. **V.**)

In practice only variant 0 is used unless `pallet-indices` is mounted, which Demiurge does not mount
and has no plan to — it appears in neither `chain/runtime/src/lib.rs` nor its manifest. **V.** So the
difference on the wire, for every transaction anyone will actually send, is **one byte**. **V.**

That is the whole technical difference. Everything below is about who has to know.

## 3. The case for each

### Keep `IdentityLookup`

1. **It is one byte smaller per transaction**, and one enum match cheaper per dispatch. At any volume
   this project will see before mainnet, neither is measurable. **I.**
2. **It does not offer variants the chain will never honour.** `Raw`, `Address20` and `Index` are
   dead weight on a chain with no `pallet-indices`: a sender can encode them and the lookup will fail.
   A type that cannot express a wrong thing is a real virtue.
3. **It is a standard component, not a custom one.** `IdentityLookup` ships in `sp_runtime::traits`
   and is a supported `Lookup` implementation. AGENTS.md §7's rule is about departing from standard
   components, and this is not a departure in that sense.
   - **But it is a departure from the standard *default*.** The line overrides
     `SolochainDefaultConfig` (§1), so "we just took the template's word for it" is not available as
     an argument. It was chosen, silently.
4. **Nothing is broken today.** Every client that reads the address type from metadata handles it. The
   launcher does, and it took one line of configuration. **V.**

### Change to `AccountIdLookup`

1. **It is what every wallet, tool and integrator has met before.** Polkadot.js, Talisman, Nova,
   `subxt` and the SDK's own examples all read the address type from metadata and therefore work
   either way — but the *humans and code* around them do not. A third-party integrator writing against
   Demiurge for the first time, copying a working Polkadot example, hits a refusal on their first
   signed transaction, and the error the node gives them does not say why. **I**, but it is the
   failure mode I would bet on: it is the same shape as the "two chains, same port" trap in
   `HANDOFF.md` §2.0, which cost this project time twice.
2. **ADR-009's promise is compatibility with the ecosystem's tools**, and ADR-023 and ADR-039 already
   paid real costs for it — Talisman's derivation offset was matched exactly so that one phrase lists
   the same accounts in the same order. Matching the ecosystem's *address shape* is the same argument,
   one layer up, and it is cheaper than the one already paid.
3. **It keeps the door open for `pallet-indices`**, which is the only thing `MultiAddress` is really
   for. Not planned, and not an argument on its own; it costs nothing to keep.
4. **It removes a thing that has to be explained.** Today the launcher carries a comment explaining
   why it does not use `PolkadotConfig`; the SDK will carry the same explanation; so will the first
   integration guide. Every one of those is a place the explanation can go stale.

## 4. What it costs, and when

| | Before M5.1 (now) | After M5.1 |
| --- | --- | --- |
| Runtime | One line in `chain/runtime/src/lib.rs`, plus `MultiAddress` in `UncheckedExtrinsic` | The same line |
| Storage migration | **None.** No storage is keyed by the address type; it exists only in the extrinsic envelope. **V** | None |
| Chain tests | `chain/runtime/tests/block_import.rs:91` builds signed extrinsics with `UncheckedExtrinsic::new_signed(call, account, …)` and would pass `MultiAddress::Id(account)` instead. **V.** ~~`acceptance.rs` needs nothing: it dispatches through `RuntimeOrigin::signed`, which never sees an address.~~ **This was wrong** — see the correction below | The same |
| The launcher | One line: `type Address = MultiAddress<AccountId32, ()>`, and the `dest` argument becomes `MultiAddress`. Its live test then re-proves it. **V** | The same, **plus** every released launcher in someone's hands stops being able to transact |
| The SDK | Does not exist yet | Published, versioned, and in other people's builds (M5.1). A breaking release, coordinated with the chain's runtime upgrade |
| Anyone else's code | Nobody has any | Whoever has integrated |
| Risk of getting it wrong | A test failure, immediately | Refused transactions in the field, with an error that does not name the cause |

**Correction, 20 September 2026, made while carrying the decision out.** The row above said `acceptance.rs`
needed nothing. It needed eight call sites. Dispatching through `RuntimeOrigin::signed` does mean it never
builds an extrinsic address — that part was right — but the `dest` **argument** of every balances call is
`AccountIdLookupOf<T>`, which changes with `Lookup` regardless. The mistake was reading "address type" as only
the extrinsic envelope. The table is left as written, struck through, rather than quietly fixed, because the
owner decided on it. It would not have changed the decision: eight mechanical call sites against a coordinated
breaking release is not close. It is corrected because a cost estimate that was wrong should say so whichever
way it would have pushed.

**The asymmetry is the whole point.** Today this is an afternoon and a test run. After the wire format
is frozen it is a coordinated breaking change across the chain, the SDK, the launcher and every
integrator, for one byte. That is the kind of change that does not get made, which means the choice
made by default at M5.1 is the choice forever.

## 5. Recommendation

**Change to `AccountIdLookup<AccountId, ()>`, before M5.1.**

- The technical case for `IdentityLookup` is one byte and one match. The technical case against it is
  nothing at all. This is not a performance decision; it is a decision about who has to know something.
- The case for `AccountIdLookup` is that it is what the ecosystem assumes, and this project has already
  decided twice (ADR-009, ADR-023) that matching the ecosystem's assumptions is worth paying for. This
  is the cheapest of those three payments by a wide margin.
- AGENTS.md §7 asks for a written reason to depart. Writing this document was the attempt to find one,
  and it found the opposite: the line overrides the SDK's own solochain default (§1), so there is not
  even an inherited default to point at. "It came from the template" would have been an explanation
  rather than a reason; it is not available either.

**If the owner prefers to keep `IdentityLookup`**, that is a legitimate choice and this document
becomes its written reason. In that case two things should follow, so the choice stays deliberate:

1. The runtime carries a comment at `type Lookup` saying it is chosen, not inherited, and why.
2. The SDK's first integration guide leads with it, because it is the first thing that will bite.

Either way it should be an ADR, so the next person finds a decision rather than a default.

## 6. Decision needed

- [ ] ~~**Keep `IdentityLookup`**, with the comment and the guide note above~~, or
- [x] **Change to `AccountIdLookup<AccountId, ()>`** (recommended), scheduled before M5.1.

**Decided by the owner on 20 September 2026: `AccountIdLookup`.** Their reason was the correction in §1 — that the
runtime line overrides the SDK's own default with nothing on record — which removed the only argument for keeping
it. Carried out the same day, in [ADR-041](../decisions/ADR-041-multiaddress-and-accountidlookup.md).
