# ADR-119: Renting a DRC-369 asset for a number of blocks, paid in CGT

**Status:** **Accepted**, 26 October 2026, under the owner's delegation of engineering choices (the owner's words of 24
October 2026: "I authorize full automation and decisions permissable. You are approved to continue development ops
with best recommendations and highest quality when possible."), after the owner chose this as the next work on 26
October 2026 ("Asset rental (M4.3)"). **It amends [ADR-061](ADR-061-royalties-and-the-settled-sale.md) decision 1**:
`pallet-drc369-royalties` holds the terms, the sale **and the rental**. It invents no economic value: every price is
set by an asset's holder, every share by its creator; no platform share is taken (U-15), no fee is charged (OPEN-4), no
deposit amount is chosen (U-14) and no CGT is created. No gate or check is loosened.
**Four decisions are product-shaped** — rent pays royalties (decision 4), the holder cannot end a rental early
(decision 7), no extension (decision 8) and progress counts while rented (decision 5). Each is recommended with its
reason, and each is the owner's to reverse **before the wire format freezes** (`beta.wire-format-frozen`).
**Carries out:** the rental third of roadmap item **M4.3** ("Rental, fractional ownership and fixed-point physics").
Fractional ownership and physics are not part of it, so M4.3 stays unticked.
**Depends on:** [ADR-047](ADR-047-the-object-model.md) (decision 13 row 9: the rental time unit is the block number),
[ADR-061](ADR-061-royalties-and-the-settled-sale.md) and [ADR-062](ADR-062-royalty-terms-a-creator-can-correct.md) (the
terms, `split`, a part that cannot be received refuses the whole), [ADR-066](ADR-066-buying-what-was-seen-and-the-market.md)
(what was seen is what is paid for), [ADR-065](ADR-065-nesting-and-the-cycle-rule.md) (an asset held in place by the `Locker`),
[ADR-113](ADR-113-progress-records-kept-by-each-game.md) (progress records), [ADR-035](ADR-035-eighteen-decimals.md),
[ADR-002](ADR-002-value-from-spending.md) and [ADR-006](ADR-006-demand-sinks.md).

## Context

A creator's work is used in more ways than it is sold: a game lends a car for a race, a musician licenses a stem for a
month, a player borrows a friend's sword for an evening. Today the chain can only hand an asset over for good. The
custom devnet had a rental library that nothing called: durations in seconds, never enforced, no ownership check and no
CGT moved (inventory F-D6). ADR-047 fixed one thing about rental, its time unit: the block number, which needs no
clock and cannot drift. Everything else is decided here.

Three facts shape it. **A rental the chain does not enforce is a promise anyone can break**, so while it lasts the
asset must be held in place as firmly as a nested asset is. **A rental is paid, or it is a loan**, and either way the
creator's royalty question arises exactly as it does for a sale. **Expiry must cost nothing**: a rental that needed a
transaction to end, or a hook walking every rental each block, would make time itself a fee or a weight.

## Decision

1. **A rental is a right to use, not ownership.** The holder keeps the asset. For a number of blocks a renter is its
   **user**: the account games, the Library and entitlements treat as the one who may use it, read through
   `user_of`. The chain decides who the user is; what "use" means is each product's.

2. **It ends at a block, by itself.** A rental names the block it ends at (`until`). From that block on it is over
   wherever it is read, with no transaction and no hook: every check compares `until` with the current block. A rental
   that has ended leaves a record until something clears it (decision 7), and that record decides nothing.

3. **First an offer, then a rental.** The holder offers an asset on terms: a **price per period** in CGT (zero lends
   it), a **period** in blocks (at least one), **at most how many periods** (at least one), and, if they choose, **the
   one account** it is offered to. One offer per asset; offering again replaces it. An offer outlives the rentals it
   serves, so an asset can be rented again and again on it. It is withdrawn by whoever made it or by the holder, and by
   anyone once void, because whoever made it no longer holds the asset.

4. **Renting is one transaction, and rent pays royalties as a sale does.** The renter names how many periods, the most
   they will pay in all, and the content they expect the asset to carry (ADR-066: what was seen is what is paid for).
   The rent, the price per period times the periods, is paid up front and divided by `split` exactly as a sale at that
   price: the source's remix share, then the asset's own royalties, then the holder. *Why:* a creator is paid when
   their work is used (AGENTS.md §6), and a rental is use; the terms a creator set are the terms of every payment for
   the asset, not of resales alone. *Product-shaped:* rent could instead go to the holder whole; the owner may reverse
   this before the freeze. A part that cannot be received refuses the whole rental (ADR-061 decision 7). The rent is a
   checked multiplication of an amount by a count, refused rather than wrapped if it would overflow; every fraction is
   still `split`'s.

5. **While rented, the asset is held in place.** Nobody can transfer it, sell it, list it for sale, revise it, nest it,
   or take it or anything from inside it: `pallet-drc369`, the ledger's `Locker`, answers that it is locked, as it does
   for a nested asset (ADR-065), so `pallet-nfts` refuses every transfer whatever call it arrives in. **Progress
   records stay writable** (ADR-113): a record never moves an asset, and a renter's play counts on the asset they
   play. *Product-shaped:* records could be held still as while listed (ADR-113 decision 7); the owner may reverse it.

6. **Never listed and rented at once.** An asset listed for sale cannot be rented, and a rented one cannot be listed.
   An offer to rent and a listing for sale may stand together; whichever is taken first decides, and the other waits.

7. **The holder cannot end a rental early, a loan included; the renter can.** The renter paid for those blocks, or was
   promised them, and a promise the holder could withdraw at will is no rental. The renter may hand it back early with
   `end_rental` and is refunded nothing. Anyone may call `end_rental` on a rental that has ended, which clears its
   record. *Product-shaped:* a loan the lender can recall is a different agreement and is not built.

8. **No extension.** Once a rental ends, its renter rents again as anyone may. *Product-shaped:* extending in place
   needs a rule for terms changed meanwhile, which can come later.

9. **No deposit of its own (U-14).** One offer and one rental per asset at most, each a small entry, bounded by the
   asset's own deposits, as ADR-061 reasoned for a listing.

10. **Every event names everything an indexer needs** (ADR-047 decision 12): `RentalOffered`, `RentalOfferWithdrawn`,
    `Rented` (the parts of the rent as `Sold` names them, and `until`) and `RentalEnded` (who ended it, and whether
    early).

11. **Asked before paying.** The runtime API answers who uses an asset and until when, an asset's offer, what an
    account is renting, and what renting would pay, part by part, and whether it could settle: run and undone, as
    `sale_preview` is, so a client carries no copy of the rules.

## The spend test and the sink (ADR-002, ADR-006)

Rent is **access gating**: an entitlement to use a work for a time, settled only in CGT. A renter needs CGT to use
something they already want to use, and a creator is paid in it when their work is used. If CGT could never be traded
for dollars, both would still need it: the mechanic passes. Nobody is paid for holding CGT.

## What this record does not decide

- **Fractional ownership**, M4.3's second third, which waits for the legal review: a fraction of an asset sold for
  CGT can look like a security (U-15 to U-18). The inventory points it at `pallet-nft-fractionalization`.
- **Physics**, M4.3's last third, in ADR-047's fixed-point encoding.
- **Fees (OPEN-4), deposits (U-14), a platform share (U-15)** and any sponsored first rental (M4.4).
- **The launcher's renting surface and ARQADE's SDK reading `user_of`**: product work on top of this.

## Consequences

- `pallet-drc369-royalties` gains `offer_rental`, `withdraw_rental_offer`, `rent` and `end_rental` (call indices 5 to
  8), the storage `RentalOffers`, `Rentals` and `RentalsOf`, and a runtime API of its own, `Drc369RentalApi`
  (`user_of`, `rental_of`, `rental_offer_of`, `rentals_of`, `rent_preview`), separate from `Drc369RoyaltiesApi` because
  its answers carry block numbers; `sale_preview` is untouched. `pallet-drc369` gains
  `Config::Rented`, which the runtime answers from `pallet-drc369-royalties` as it answers `ListedForSale`, and refuses
  `revise`, `nest` and `unnest` on a rented asset. Existing calls keep their encoding, so `transaction_version` stays;
  `spec_version` moves to 11, deployed by the owner's sudo key after a rehearsal against the devnet's live runtime.
- A rented asset's holder sees it in their Inventory and cannot sell or move it until `until`; the renter sees it as
  theirs to use. The launcher shows neither until its surface is built.
- Weights are placeholders owed to M7.2, like every weight in both pallets; `rent`'s is sized for its worst case, the
  same seventeen payments as `buy`.
- Reversing a product-shaped decision after `beta.wire-format-frozen` is a format change.
