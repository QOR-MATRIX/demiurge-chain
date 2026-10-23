# ADR-113: An asset's progress is kept by each game that earns it

**Status:** **Accepted**, 23 October 2026, by the owner. The owner, weighing four ways to decide who may change an
asset's level, XP and stats, found each lacking ("None of these paths seem to be a very good balance") and left the
choice to the assistant's discretion "while also urging you to consider long term usability alongside developer
friendliness". This record's design was put to the owner, with the question of supporting several paths by use case,
and the owner answered "all is Approved" and "Proceed." It carries out **state and XP**, the part of **M4.2** and
**M4.5** in `docs/DIRECTION.md` still open. It decides **no economic value**.

**Builds on:** [ADR-047](ADR-047-the-object-model.md) (decision 7's `state: AssetState`, decision 12's `StateChanged`,
decision 13's `MaxStats`), [ADR-070](ADR-070-game-vaults.md) (a game is its Cartridge, governed by whoever holds it),
[ADR-078](ADR-078-levels-tasks-and-the-welcome-grant.md) (XP is granted only by a server that can check what it rewards),
[ADR-066](ADR-066-buying-what-was-seen-and-the-market.md) (a buyer is held to what they saw) and
[ADR-112](ADR-112-the-controllers-own-key-signs-every-agent-authorisation.md) (an agent authorised by its controller's
key may later be a writer).

## Context

ADR-047 gives every asset a level, XP and up to 32 numbered stats, and an event when they change, but not who may
change them. The four ways weighed were the asset's holder (anyone could raise their own item), the item's game alone
(every holder trusts one game with everything), the game and the holder co-signing each change (a dialog per level-up),
and the chain checking the play itself (not feasible). Each lost too much. What the project needs: progress that means
something to a buyer and to another game, a holder who keeps their item whatever a game does, and one simple thing for a
developer to call.

## Decision

1. **Progress is a record kept by an issuer, one record per issuer per asset.** A record holds ADR-047's state: a level
   (`u32`), XP (`u64`) and stats (`(u16, i64)` pairs, at most `MaxStats`, 32). It is stored under the asset and the
   issuer together, so no issuer can touch another's record.
2. **An issuer is a typed value, and today it is a game.** A game is named by its Cartridge, the DRC-369 asset it was
   published as, and governed by whoever holds that Cartridge, as an ARQ Wallet is (ADR-070). Other kinds of issuer
   (the holder's own record, labelled as self-kept; a rules pallet) are added later as new variants with new indices,
   when a product needs one, without migrating a stored record or changing a call.
3. **A collection is static until its owner binds it to a home game.** The owner of a collection (for a singles
   collection, its creator) may name a game as its home, change it, or clear it. Only the home game keeps records on
   the collection's assets in this first phase. Rebinding changes which game writes next; records already kept stay,
   under their issuer.
4. **A game's holder names its writers.** Whoever holds the Cartridge authorises accounts, such as the game's servers,
   to write its records, and revokes them; at most `MaxWriters` (16) at once. When the Cartridge changes hands, so does
   this authority.
5. **Writers can only add.** A writer adds XP, may raise the level but never lower it, and sets stats, which may move
   either way (health, durability). A change that changes nothing is refused.
6. **Corrections are the holder's, separately.** Whoever holds the Cartridge, not a writer, may set a record to any
   values, lower ones included, to undo an exploit. It is a call of its own with an event of its own, so a correction is
   never mistaken for play.
7. **An asset listed for sale keeps its records still.** While it is listed, no record of it can be written or
   corrected, so a buyer receives what they looked at (ADR-066). Withdrawing or selling the listing frees it.
8. **A record never moves, locks or sells an asset, and never touches CGT.** A game that disappears leaves its records
   readable and the asset fully its holder's.
9. **Every change is an event naming who made it:** `StateChanged { collection, item, game, level, xp, stats, by }`
   (ADR-047 decision 12's event, with the issuer and the stats added) and `StateCorrected` with the same fields.
10. **No deposit of its own in this phase.** A record exists only for an asset whose mint paid its deposit, in a
    collection its owner chose to bind, one per asset per home game, and bounded in size; the deposits are placeholders
    in any case (U-14). Whether records hold a deposit is U-14's, before records by other issuers exist.
11. **Second phase, not this record's build:** visiting games, each keeping its own record on an asset once its holder
    has allowed that game in the launcher (one dialog per game, never per change). The storage and the calls are shaped
    for it: `record_progress` already names the game it writes for.

## Consequences

- `pallet-drc369` gains `set_home_game`, `authorise_writer`, `revoke_writer`, `record_progress` and
  `correct_progress`, the storage `HomeGame`, `Writers`, `WriterCount` and `Records`, and runtime API calls to read
  them. Existing calls keep their encoding, so `transaction_version` stays; `spec_version` moves to 10, deployed by the
  owner's sudo key.
- A game developer authorises their server's key once, binds their collection once, and from then on calls
  `record_progress` with no prompt for players. ARQADE's SDK will wrap it.
- A holder can always see who changed their asset and when, and no game can take back earned XP except by a
  correction everyone can see.
- Records cannot be written while an asset is listed: a game must tell a player whose item is on sale that its progress
  waits for the sale or its withdrawal.
- Weights are placeholders owed to M7.2, like every weight in the pallet.
