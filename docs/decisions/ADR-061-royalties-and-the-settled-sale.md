# ADR-061: Royalties, remix royalties, and the sale settled in CGT they bind to

**Status:** Accepted, 29 September 2026. **Decision 2 superseded the same day by
[ADR-062](ADR-062-royalty-terms-a-creator-can-correct.md)**, on the owner's ruling: terms can be corrected while the
creator holds the asset. Decision 4, one level of remix royalty, was confirmed by the owner in the same ruling.
**Decision 1 amended on 26 October 2026 by [ADR-119](ADR-119-renting-an-asset-for-blocks-paid-in-cgt.md):** the pallet
holds the terms, the sale and the rental, and rent is divided by `split` as a sale is.
**Accepted under the owner's standing delegation for engineering choices.** It invents no economic value: every share
is set by a creator, not by the protocol; no platform share is taken (U-15 is open), no fee is charged (OPEN-4), no
deposit amount is chosen (U-14) and no CGT is created. The pallet's name, `pallet-drc369-royalties`, was confirmed by
the owner with ADR-032. No gate or check is loosened.
**Two decisions are product-shaped** — one level of remix royalty (decision 4) and terms that never change
(decision 2). Each is recommended here with its reason, and each is the owner's to reverse **before the wire format
freezes** (`beta.wire-format-frozen`), after which reversing it is a format change.
**Carries out:** the royalty half of roadmap item **M4.2** — "royalties with remix royalties settled in CGT". Nesting
and state and XP, M4.2's other half, are not part of it.
**Depends on:** [ADR-047](ADR-047-the-object-model.md) (the record, the bounds, decision 11's rule that no settlement
walks a graph, decision 12's events), [ADR-057](ADR-057-eight-royalty-recipients.md) (eight recipients),
[ADR-052](ADR-052-assets-in-the-runtime.md) (one way in, and the call filter), [ADR-035](ADR-035-eighteen-decimals.md)
(the SDK's helpers for every fraction), [ADR-002](ADR-002-value-from-spending.md) and
[ADR-006](ADR-006-demand-sinks.md).

## Context

The owner's next item after onboarding was real selling, and it needs royalties first: a creator is paid when their
work resells. ADR-047 fixed the fields and bounds a royalty would use — `derived_from` and `remix_depth` on the
record, a `Sold` event, eight recipients, shares as a `Permill` — and said explicitly that it did **not** decide
"royalty mechanics beyond the fields and bounds above".

One fact shapes everything, and it holds on every chain: **a plain transfer names no price, so nothing can be taken
from it** (ADR-047's constraint table, inventory F-D5). A royalty can bind only to a sale the chain itself settles.
`pallet-nfts` has a sale of its own, `set_price` and `buy_item`, and it pays no royalty; the call filter already
refuses both (ADR-052). So a royalty pallet that stored terms and nothing else would enforce nothing. It has to own
the sale as well.

## Decision

1. **`pallet-drc369-royalties` holds the terms and the sale, and nothing else.** Two storage maps, one entry of each
   per asset at most: `RoyaltyTerms` and `Listings`. Four calls: `set_terms`, `list`, `unlist` and `buy`. The sale is a
   fixed-price listing bought outright: the smallest sale a chain can settle, and the primitive Market's listings
   (P5.5) will sit on. Auctions, offers and bundles are not built.

2. **Terms are set once, by the asset's creator, while they hold it, and never change.** The creator is the owner of
   the collection the asset was minted into — under ADR-047's singles collection, the account that minted it.
   *Why:* a buyer paid a price in the knowledge of what each resale would owe. Terms that could be changed afterwards
   would let a creator raise their share on work already sold, and terms that a later holder could set would let any
   buyer redirect a creator's royalty to themselves. *Product-shaped:* the alternative is terms editable until the
   first sale, which is no less safe; it was not chosen only because "until the first sale" is a second piece of
   state for one convenience. The owner may reverse this before the freeze.

3. **A terms record is up to eight recipients, each with a share of every sale, and a remix share.** No recipient
   twice, no share of zero, shares summing to at most one whole. The **remix share** is what a sale of any asset
   derived from this one owes this one's recipients, divided between them in proportion to their shares. A remix
   share needs at least one recipient to pay.

4. **Remix royalties are one level deep: a sale pays its direct source only.** When the asset sold was derived from a
   source with terms, the source's remix share of the price is paid first. The source's own source is paid nothing
   from that sale. *Why:* paying every ancestor means reading every ancestor's terms inside the sale, which is the
   graph walk ADR-047 decision 11 forbids — at sixteen levels, 128 payments in one transaction — and flattening the
   ancestry into each new asset at mint would need a bound on the flattened list that the eight-recipient bound
   cannot give. *Product-shaped:* a creator who wants a share of their work's remixes' remixes gets it through the
   remix in between, which itself pays upstream from its own sales, not from its descendants'. The owner may reverse
   this before the freeze; the cost is a second bound in the wire format.

5. **The price is divided in one pure function, `split`, and every CGT amount the pallet moves comes out of it.** For a
   price `p`: the upstream pool is the source's remix share of `p`, rounded down, divided in proportion to the source's
   shares, each part rounded down; each of the asset's own recipients receives their share **of what the upstream
   payments left**, rounded down; the seller receives everything else. Own shares apply to the remainder rather than
   to `p`, so the two can never together exceed the price, whatever order the terms were set in. The parts always sum
   to exactly `p`, and rounding favours the seller by less than one Spark per recipient. Every fraction uses
   `Permill::mul_floor` or `multiply_by_rational_with_rounding` (ADR-035); a test pins `p = u128::MAX`.

6. **A sale is one transaction, and nothing in it is partial.** `buy` checks that the seller still holds the asset,
   that the buyer is not the seller and that the price is at most the `max_price` the buyer signed; pays upstream, then
   royalties, then the seller; hands the asset over through `pallet-nfts`; and removes the listing. A failure anywhere
   undoes all of it. A listing whose seller no longer holds the asset is void: it can never be bought, and anyone may
   clear it.

7. **A part that cannot be received refuses the whole sale.** A payment that would leave its recipient below the
   existential deposit (100 CGT, ADR-036) — an account that does not exist yet receiving a small royalty — cannot be
   made. The sale is refused with `PaymentCannotBeReceived` rather than paying that part to someone else or holding it
   for later. *Why:* either alternative decides who gets another person's money, and holding it needs a claim path and
   a pot of CGT owned by nobody. The cost is that a small sale naming a recipient with no account fails, which the
   launcher can say before it is attempted.

8. **Remix provenance is written by `pallet-drc369`, as ADR-047 decision 7 placed it.** `mint` gained a
   `derived_from: Option<(CollectionId, ItemId)>` argument; the record gained `derived_from` and `remix_depth`;
   `Minted` gained `derived_from`. Naming a source is open to anyone, because it obliges the remix and not the source.
   The source must be a DRC-369 asset, and depth is checked at mint against `MaxRemixDepth` (16, ADR-047), so a remix
   cannot name itself and nothing later walks the graph. Remix *rights* — whether a creator may refuse a remix, or sell
   the right to make one — are access gating under ADR-006 and are not decided here.

9. **`Sold` carries more than ADR-047 decision 12 listed, not less.** `Sold { collection, item, from, to, price,
   source, remix, royalties, seller_received }`: the source and the upstream payments are separate from the asset's own
   royalties, so an indexer can attribute every Spark without re-deriving the arithmetic. This extends decision 12's
   `Sold`; it removes nothing from it.

10. **No deposit is taken for terms or a listing.** Each is bounded to one per asset, and only an asset's holder can
    write either, so the number that can exist is bounded by the number of assets, each of which already holds its
    deposits. U-14's sizing should count them: an asset can now carry up to two more storage entries than the four
    the placeholder `ITEM` deposit prices.

## The spend test and the sink (ADR-002, ADR-006)

A sale settled in CGT is **access gating**: an entitlement — ownership of a work — that settles only in CGT. Royalties
are the reason a creator is paid when their work is used and resold, which is the demand ADR-008 describes. Neither
pays anyone for holding CGT. If CGT could never be traded for dollars, a creator would still need it to buy the work
they build on and would still be paid in it when their own work is bought: the mechanic passes.

## What this record does not decide

- **The platform's share of a sale (U-15).** None is taken. If one is decided, it is a further payment inside
  `split`, and what it is a share *of* is part of that decision.
- **Fees (OPEN-4), deposits (U-14), and any sponsorship of a buyer's first purchase (M4.4).**
- **Remix rights and licences**, which are access gating (ADR-006) and a later item.
- **Nesting, state and XP**, M4.2's other half.
- **The launcher's selling surface.** Sell drafts a listing on the owner's machine today (L4.6, half built); wiring it
  to `list` and `buy` is launcher work, and a listing everyone can see needs the indexer (M5.4).

## Consequences

- `spec_version` 4 and `transaction_version` 2, because `Drc369::mint`'s encoding changed. The launcher's mint passes
  `None` for the new argument in the same change.
- `beta.value-pallet-coverage` already named `pallet-drc369-royalties`; it now exists, so CI reports its coverage.
- The weights are placeholders assembled like `pallet-drc369`'s and owed to M7.2; `buy`'s is sized for its worst
  case, seventeen payments.
- Reversing decision 2 or 4 after `beta.wire-format-frozen` is a format change.
