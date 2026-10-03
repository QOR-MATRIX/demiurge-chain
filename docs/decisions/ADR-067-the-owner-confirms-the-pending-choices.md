# ADR-067: The owner confirms nesting's product choices, buying by number, and the readability rule

**Status:** Accepted, 3 October 2026, by the project owner, who asked for "the most recommended choices for optimal
experience and overall quality of the platform".
**Confirms:** [ADR-065](ADR-065-nesting-and-the-cycle-rule.md) choices 1, 2 and 5, which that record left to the
owner. Changes nothing in it.

## Context

Three choices made by agents on 1 October were waiting on the owner (`HANDOFF.md` §4 items 32 and 33, `OWNER.md`):
nesting's three product choices, buying an asset by its number as the first way to buy, and a change to what
`check-readability.mjs` measures, logged in `docs/GATES.toml` as an evidence-rule change not yet approved. On 3
October the Market screen (ADR-066) made the second one a second way to buy rather than the only one.

## Decision

1. **ADR-065 choices 1, 2 and 5 stand as built.** An asset holding a nested asset is held in place with it; an asset
   that holds assets cannot itself be nested; a nested asset, or one holding others, can be listed and cannot be
   bought until it is taken out. The Market and the purchase dialog say why, in words, before anyone is asked to
   sign.
2. **Buying by an asset's number stays**, in Inventory, beside the Market. A person given a number can still buy
   without browsing.
3. **The readability check's measuring rule is approved**: text scrolled out of an overflowing ancestor's box is
   not counted as visible, and a scrolling dialog is measured scrolled to its end. Recorded as `approved_by` on the
   2026-10-01 entry in `GATES.toml`.

## Consequences

- Refusing a nested asset at `list` time, which ADR-065 named as clearer for a seller, is not done; it would need a
  new error and a runtime upgrade, and the Market already shows the reason on the listing.
- None of these changes code. The tests that pin them (in `alpha.pallets`, `alpha.chain-tests` and the Market and
  Inventory view checks) are now the owner's accepted behaviour, not an agent's guess.
