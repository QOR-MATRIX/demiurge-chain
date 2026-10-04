# ADR-070: Game Vaults: one keyless payout account per published game, governed by whoever holds the game

**Status:** **Proposed**, 4 October 2026. **Decision 1 is the owner's direction**, given that day and recorded rather
than proposed. **Decisions 2 to 10 are the assistant's recommendations** and are not accepted; nothing is built on
them until this record says Accepted. The developer-facing description is
[`products/arqade/sdk/docs/game-vaults.md`](../../products/arqade/sdk/docs/game-vaults.md); the step is P7.11 in
[`../DIRECTION.md`](../DIRECTION.md).

**Relates to:** [ADR-069](ADR-069-arqade-the-gaming-platform.md) (ARQADE; this answers half of the U-16 it opened);
[ADR-026](ADR-026-agent-delegation-with-pallet-proxy.md) (delegation and spend caps, the alternative weighed in
decision 3); [ADR-029](ADR-029-sponsor-pays-the-existential-deposit.md) and
[ADR-036](ADR-036-existential-deposit-100-dmrg.md) (the existential deposit); [ADR-047](ADR-047-the-object-model.md)
(a mint pins a commit); [ADR-001](ADR-001-innovation-budget.md), [ADR-002](ADR-002-value-from-spending.md),
[ADR-006](ADR-006-demand-sinks.md), [ADR-008](ADR-008-language-discipline.md).

## Context

On 4 October 2026 the owner said: games may pay rewards in CGT from a specialised wallet, available to developers
through the ARQADE SDK, attributed to one game project each, configured for that project's needs and with special
permissions for working with it — one dedicated treasury per published game — and asked for an innovative way for a
developer to unlock it.

What the tree offers today (read 4 October 2026):

- **A game build can already become an asset.** `Drc369::mint` takes a content reference and a pinned commit
  (`chain/pallets/drc369/src/lib.rs:449-456`); the launcher mints a Qontrol project's commit (M4.1). So a published
  game can have an on-chain identity that names exactly which build it is, and an owner.
- **No keyless account pattern is mounted.** No pallet derives an account from a `PalletId`, and `pallet-proxy` is not
  in the runtime. ADR-026 decided proxies and a spend-cap pallet for agents; neither is built (M5.2).
- **The chain charges no fees and has no treasury**, and nothing may create CGT (OPEN-1, OPEN-2, AGENTS.md §5).
- **The existential deposit is 100 CGT** (ADR-036). A transfer that would open an account with less fails.
- **QOR ID can register an agent key under a controller** (ADR-014), and keeps identity off chain (ADR-027).

The obvious design — a normal account whose secret key the developer's game server holds — fails the way hot
wallets fail: one leaked server key empties every prize at once, and nothing on chain can tell a payout from theft.

## Decision

1. **The owner's direction (recorded).** A game pays CGT rewards from a payout account dedicated to that game,
   reached by developers through the ARQADE SDK, holding permissions scoped to that game. Working name: **Game Vault**
   (distinct from the player's Vault, which holds keys; a Game Vault holds none). The owner's word "treasury" is kept
   out of names because the chain's treasury is a different thing (M6.5).

2. **A Game Vault is keyless.** Its account is derived from a pallet identifier and the game's id, the SDK's standard
   pattern for accounts nobody can sign for (`PalletId::into_sub_account_truncating`). No private key exists to leak,
   phish or lose. CGT leaves it only through the pallet's calls, under the rules below.

3. **One dedicated pallet, not proxies.** Options weighed:
   - **A. A key the developer holds.** Rejected above.
   - **B. A pure proxy account with a `GamePayout` proxy type and ADR-026's spend-cap pallet.** Reuses a decided
     design, but a proxy filter sees the call and not the amount (ADR-026 §2), so every rule below would live in the
     cap pallet anyway, behind a second indirection, and a pure proxy's spawner holds an `Any` proxy over it — a key
     that can do anything again.
   - **C. A pallet that owns Game Vault accounts and exposes only the calls below.** What `pallet-treasury` does for
     its own account, scoped per game. **Recommended**: it is the boring construction for keyless funds (ADR-001), and
     every rule is a check in one place. Its name is the owner's (AGENTS.md §8); the working placeholder is
     `pallet-game-vaults`.

4. **The game is a DRC-369 asset, and whoever holds it governs its Vault.** Publishing a game mints its build as a
   DRC-369 asset — the **Cartridge** — pinning the commit it was built from. A Game Vault is created for exactly one
   Cartridge and derived from its id, so a game cannot have two Vaults and a Vault cannot serve two games. The
   **governor** is the Cartridge's current holder, read from `pallet-nfts` at each call: selling or transferring the
   game moves control of its Vault with it, and nobody else can govern it. A revised build keeps the same Cartridge
   and the same Vault.

5. **Three powers, three parties.**
   - **The governor** (the developer's own Vault, signing in the launcher): fund, set the payout policy, register or
     revoke the game's payout authority, pause, and withdraw.
   - **The payout authority**: one Sr25519 key the developer's game server holds, registered on chain for that Vault
     only (and recorded in QOR ID as an agent of the developer, ADR-014). It can **only** call `payout` and open or
     settle a round, and only inside the policy. A stolen authority key can move at most what the policy allows before
     the governor revokes it.
   - **The player**: receives payouts, and claims what accrued when it could not be paid at once.

6. **The policy is chain state, and loosening waits.** Each Vault carries a policy its governor sets: the largest
   single payout, a budget per epoch, a cap per recipient per epoch, the rule versions payouts may name, and an
   expiry for unclaimed accruals. Every value is the **developer's**, set per game; the protocol sets bounds only, and
   those bounds are U-16. **Tightening takes effect at once. Loosening a cap and withdrawing funds take effect only
   after a delay**, during which the change is visible on chain and can be cancelled, so a stolen governor key cannot
   empty a Vault before anyone notices. (The same rule `GATES.toml` applies to its own evidence.)

7. **Exactly once, enforced by the chain.** Every payout names an **outcome id**, the game's rule version and the
   recipient. The pallet refuses an outcome id it has already paid (kept for a bounded window), so a retried or
   replayed server request cannot pay twice whatever the game's database believes. The `Paid` event carries all
   three, so anyone can audit a game's payouts from chain history.

8. **A prize is reserved before anyone pays to compete for it ("proof of prize").** A paid round opens with a hold on
   its prize inside the Vault. Players' clients read the hold from chain state before entering; settling the round pays
   from the hold, cancelling releases it. **Pending claims rank ahead of withdrawals**: a governor cannot withdraw CGT
   already owed or reserved.

9. **Below the existential deposit, payouts accrue.** A payout to an account that does not exist, and is smaller than
   100 CGT, is recorded as claimable in the Vault instead of failing. The player claims once the sum can open the
   account, or once their account exists (by ADR-029's sponsorship, U-4). Expired accruals return to the Vault.

10. **What the developer must do first, and the SDK walks it — "ignition".** Mint the Cartridge, create its Vault,
    fund it (at least the existential deposit, which keeps the account alive), set a policy, register the payout
    authority, and run a payout end to end on Demiurge Devnet with test CGT. The SDK's `arqade vault ignite` command
    performs these in order, shows each transaction for approval in the developer's own Vault, and refuses to mark a
    Vault live until the devnet payout has finalised. Nothing in it holds the governor's key.

## Consequences

- **This answers what funds a prize** (U-16's second part): the game's own Vault, funded by its developer, and
  optionally by entry fees routed into it. U-16 keeps the rest: whether paid entry may lead to a prize at all, and the
  protocol's bounds on a policy (the longest and shortest delay, the outcome window, the accrual expiry).
- **The spend test holds** (ADR-002): a developer spends CGT to pay players for play they want to reward, and nothing
  pays anyone for holding CGT in a Vault. A Vault earns nothing on its balance, and copy never says otherwise
  (ADR-008). A payout is not a demand sink; the demand is the developer's need for CGT to fund one. Entry fees into a
  Vault are access gating (ADR-006).
- **Chain work, on the chain's roadmap.** A new pallet (named by the owner), a runtime upgrade, its tests with the
  runtime built, and the weights owed to M7.2. Its money paths follow ADR-035 (no hand-written `a * b / c`) and carry
  a test of their largest intermediate. It needs nothing from M5.2, OPEN-1, OPEN-2 or OPEN-4.
- **What a developer cannot do with it**: pay a player more than the policy says, pay twice for one outcome, withdraw
  what is owed or reserved, or make the Vault serve another game. **What it cannot stop**: a game that lies about who
  won. The authority attests; the chain enforces limits, not truth. Anti-cheat stays the game's (blueprint §8).
- **Paid chance stays off** (ADR-069 decision 7) until U-16 and a legal review, Game Vault or not.

## What this record does not decide

- The pallet's name, the SDK's package names, and the CLI's name (AGENTS.md §8).
- Any number: the protocol's bounds on delays, windows and expiries (U-16), deposits for Vault storage (U-14), or a
  platform share of entries (U-15).
- Whether a Vault may hold DRC-369 assets to award (trophies minted ahead of time). Recommended later, after G-16.
- Whether entry fees may be paid into a Vault directly, which is U-16's first part.
