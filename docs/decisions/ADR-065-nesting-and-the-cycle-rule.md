# ADR-065: Nesting, and what a nested asset may do

**Status:** Accepted, 1 October 2026, under the delegation of engineering choices. **Choices 1, 2 and 5 are product
choices** a person will feel, and are the owner's to confirm or reverse, as ADR-062 did for ADR-061.
**Builds on:** [ADR-025](ADR-025-drc369-on-pallet-nfts.md) (the ledger and its `Locker`),
[ADR-047](ADR-047-the-object-model.md) (decision 7, decision 12's events, decision 13's bounds) and
[ADR-052](ADR-052-assets-in-the-runtime.md) (the call filter).

## Context

M4.2 asks for "authorised nesting with bounded depth", and M4.5 for requirement R-2: a nesting cycle refused within a
bounded depth. ADR-047 fixed the bounds (depth 8, 64 children) and the event shapes. It did not say what a nested
asset, or the asset holding it, may do while nested. Twelve questions were left to the build.

## Decision

`pallet-drc369` gains `nest(collection, item, parent)` and `unnest(collection, item)`, signed only, for the owner of
both assets. `ParentOf` maps a child to its parent and `ChildCount` counts a parent's children. The pallet is
`pallet-nfts`'s `Locker`. `spec_version` is 5; no existing call's encoding changed.

**R-2.** `nest` walks `ParentOf` from the proposed parent to its root: at most eight reads. It refuses with
`NestingCycle` if it meets the child, and with `NestedTooDeep` past the bound.

The twelve choices, each the conservative one where the documents were silent:

1. **An asset that holds a nested asset is held in place too, not only the child.** Moving a parent would strand its
   children under the old owner, or need a tree walk inside a transfer, which inventory F-D3 forbids. To move a
   tree: take the contents out, move, put them back; one `batch_all` can carry that.
2. **An asset that holds assets cannot itself be nested** (`HoldsAssets`). Trees are built from the root down, which
   keeps the depth bound exact without storing or measuring a subtree's height. The cost: moving a sword with a gem
   in it into a chest means taking the gem out first.
3. `unnest` may detach an asset that still holds others. Depth is walked, never stored, so nothing goes stale.
4. `parent` and `depth` are not fields of `Asset`, which departs from ADR-047 decision 7's sketch. Appending them
   changes the encoding of every stored asset and of the runtime API the launcher decodes. They can be folded in
   before the wire format is frozen (`beta.wire-format-frozen`).
5. **A nested asset, or one holding others, cannot be bought; it can still be listed.** `buy` fails whole with
   `ItemLocked` and no CGT moves. A listing survives a nest and becomes buyable again when the asset is taken out.
   Refusing at `list` would be clearer to a seller and needs a new error in `pallet-drc369-royalties`.
6. `revise`, `make_permanent` and `set_terms` stay allowed on a nested asset: nesting does not change who owns it.
7. Nesting takes no deposit: one small entry per asset, bounded by the asset's own deposit (ADR-061's reasoning for
   listings).
8. There is no on-chain list of a parent's children and no runtime API change. Which children a parent holds comes
   from events and the indexer (ADR-028).
9. Any two DRC-369 assets the signer holds may be nested, across collections.
10. Nesting creates no royalty relationship.
11. Root cannot nest or un-nest.
12. `unnest` then `transfer` in one `batch_all` by the owner is allowed: the asset is no longer nested when it moves.

## Consequences

- R-2 is met and the migration inventory's last open requirement is closed in code. M4.2 and M4.5 stay unticked:
  state and XP are not built.
- The lock lives inside `pallet-nfts`, so it holds for a bare transfer, an approved account, a batch and a sale
  without the call filter changing.
- `pallet-nfts`'s reference weights do not count the `Locker`'s two reads, and `nest` and `unnest` carry placeholder
  weights. Both join the M7.2 debt.
- Evidence: 10 pallet tests and 3 runtime tests, each shown to fail against a planted fault first, and 109
  workspace tests with the wasm built. One path, a child that is not a DRC-369 asset, is asserted but was not seen
  to fail.
- Nothing has exercised nesting against a running node, and the launcher has no surface for it.
