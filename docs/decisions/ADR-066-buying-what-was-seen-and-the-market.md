# ADR-066: Buying what was seen, asking what a sale pays, and the Market's first slice

**Status:** Accepted, 3 October 2026, by the project owner.
**Builds on:** [ADR-061](ADR-061-royalties-and-the-settled-sale.md) (the settled sale) and
[ADR-065](ADR-065-nesting-and-the-cycle-rule.md) (nesting). Changes neither.

## Context

Between 1 and 3 October three things were built that put new names in public surfaces, which AGENTS.md §8 reserves
to the owner (ADR-032's naming convention):

- **A purchase that holds the work.** `buy(collection, item, max_price)` holds the price a buyer agreed to, not the
  work: `pallet-drc369`'s `revise` can change a listed asset's content between the buyer looking and the block. A
  new call refuses that sale.
- **A question any client can ask.** The launcher carried a second copy of the pallet's payout arithmetic to show
  what a sale would pay. A runtime API answers it from the chain's own function, and the launcher now uses it and
  carries no copy.
- **QOR ID says whether its mail leaves the machine.** Its end-to-end scripts register addresses nobody holds; run
  against a service sending real mail, each bounces and counts against the sending domain, which happened on 22
  September. `GET /health` now answers that one fact, and every script refuses to run unless it is `false`.

The launcher's Market screen, the first slice of L7.2, also needed one product choice that the chain does not make.

## Decision

1. **`Drc369Royalties::buy_exact(collection, item, max_price, content)`**, call index 4, is `buy` that also refuses
   the whole sale with the error **`ContentChanged`** if the asset's current content reference is not `content`.
   `buy` keeps its index and encoding. The launcher sends `buy_exact`.
2. **The runtime API is `Drc369RoyaltiesApi`, with one method, `sale_preview(collection, item, price, buyer)`**,
   answering what a sale would pay part by part and the error it would fail with, if any. It moves nothing.
3. **QOR ID's `/health` carries `email_leaves_this_machine`**, a boolean: true only when email is configured and goes
   anywhere but a stand-in on the same machine. It is the only configuration fact `/health` gives.
4. **On the Market, "Clear" on a void listing is offered only to the asset's holder.** The chain lets anyone clear a
   listing whose seller no longer holds the asset; the screen offers it to the one person it concerns.
5. The launcher's internal names for the Market (`chain/market.rs`, the command `drc369_market`, the surface
   `market`) and its labels (Everyone's, Others', Yours; the eyebrow "Exchange") stand as built.

## Consequences

- The names in 1 and 2 are in runtime metadata from `spec_version` 6. Renaming them later is a breaking change for
  every client, so they are treated as fixed.
- A client that still sends `buy` keeps working and is not protected against a revision; only `buy_exact` is.
- `/health` tells anyone whether a deployment sends real mail. That was already inferable from whether
  `forgot-password` answers 503. Production answers `true`, so the end-to-end scripts refuse to run against it.
- Anyone can still clear a void listing directly on chain; decision 4 is about what the launcher offers, not what
  the chain allows.
