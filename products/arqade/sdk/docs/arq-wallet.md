# ARQ Wallet: paying players CGT from your game

**For game developers building on ARQADE.** This is how your game pays players in CGT: from a payout account that
belongs to your game alone, has no private key, and moves CGT only under rules you set and the chain enforces.

> **Status: decided; its chain half is built and live on Demiurge Devnet (4 October 2026).** This describes [ADR-070](../../../../docs/decisions/ADR-070-game-vaults.md),
> **accepted by the project owner on 4 October 2026**, who named the wallet **ARQ Wallet**. The chain module, `pallet-arq-wallet`,
> exists and is tested (`chain/pallets/arq-wallet/`): wallets, policy, delayed loosening and withdrawals, the payout
> authority, each outcome once, accruals and claims. Rounds with held prizes are live on the devnet too (`spec_version` 8). In the SDK, `arqWalletAddress`, `outcomeId`, `roundId`, `toChainPolicy`
> and `payoutArgs` exist; every API below marked *proposed* does not. What
> exists today is in [`../README.md`](../README.md). "ARQ Wallet" is the owner's name; the CLI and package names are
> placeholders until the project owner approves them.
>
> On Demiurge Devnet all CGT is **test CGT**. Nothing in this document says or implies that CGT has a cash value.

---

## The idea in one paragraph

When you publish a game, its build becomes a DRC-369 asset — your game's **Cartridge** — pinned to the exact commit it
was built from. Every Cartridge gets exactly one **ARQ Wallet**: an account derived from the Cartridge's id that
**nobody holds a key for**. You fund it from your own Vault. Your game server holds a narrow **payout authority** key
that can pay players only within the **policy** you wrote on chain: how much per payout, per player, per epoch. The
chain refuses to pay the same outcome twice, keeps what players are owed ahead of anything you withdraw, and makes
every loosening of your rules wait in public before it applies. If you sell your game, the Vault's controls go with
the Cartridge.

## Why it has no key

A prize account whose key sits on a game server is one leak away from empty, and nothing on chain can tell a payout
from theft. An ARQ Wallet has no key to leak. The worst a stolen **payout authority** key can do is pay out what your
policy already allows, until you revoke it from your own Vault. The worst a stolen **governor** key can do is
schedule a withdrawal or a looser policy, which waits in public for the delay, and which you cancel.

## Who can do what

| | Governor (you, from your own Vault) | Payout authority (your game server) | Player |
| --- | --- | --- | --- |
| Fund the Vault | yes | — | anyone can send CGT to it |
| Set or tighten the policy | yes, at once | — | — |
| Loosen the policy | yes, after the delay | — | — |
| Register or revoke the authority | yes, at once | — | — |
| Pause payouts | yes, at once | — | — |
| Withdraw | yes, after the delay, never what is owed or reserved | — | — |
| Pay an outcome | — | yes, within the policy | — |
| Open, settle or cancel a paid round | — | yes, within the policy | — |
| Claim an accrued payout | — | — | yes |

The governor is **whoever holds the Cartridge**, checked at every call. There is no separate admin list to keep in
step with ownership.

## Ignition: unlocking your ARQ Wallet

An ARQ Wallet goes live only after one real payout has finalised on Demiurge Devnet. The SDK walks it with one command
(*proposed*):

```text
$ arqade vault ignite --project ./my-game --network devnet

  1  Cartridge      mint build 3f2a…c901 as a DRC-369 asset          approve in your Vault
  2  ARQ Wallet     create the Vault for Cartridge 7:12              approve in your Vault
  3  Fund           send 1,000 CGT (test) to 5Gxx…Vault              approve in your Vault
  4  Policy         per payout 50 · per player/epoch 200 · …         approve in your Vault
  5  Authority      register 5Fyy…srv as payout authority            approve in your Vault
  6  Rehearsal      pay 1 test outcome to your own account, finalise
  7  Live           Vault 5Gxx… is live on Demiurge Devnet
```

The amounts shown are an example a developer chose; the platform sets none of them. Every step is a transaction you
read and approve in the launcher. The CLI never sees your Vault's key. The first deposit must be at least the
existential deposit, **100 CGT** (ADR-036), which keeps the account alive; it can never be withdrawn while the Vault
exists.

## The policy

| Field | Meaning |
| --- | --- |
| `maxPayout` | The largest single payout, in Sparks |
| `epochBlocks` | How long an epoch lasts, in blocks |
| `epochBudget` | The most the Vault pays in one epoch, all players together |
| `perRecipientPerEpoch` | The most one account receives in one epoch |
| `ruleVersions` | The game rule versions a payout may name. A build update cannot pay under rules you did not list |
| `accrualExpiryBlocks` | How long an accrued, unclaimed payout waits before returning to the Vault |
| `loosenDelayBlocks` | How long a loosening or a withdrawal waits before it applies. The protocol sets its minimum |

Amounts are **integer Sparks** (`1 CGT = 10^18 Sparks`), as decimal strings in JSON and `bigint` in TypeScript. The
SDK's `parseCgt` and `formatCgt` convert, and refuse excess precision rather than rounding. The SDK's
`validatePolicy` checks a policy before you sign it (both exist today, [`../README.md`](../README.md)).

## Paying a player

What exists today: the arguments, built and checked. Signing and sending them is your server's own chain client
(the SDK's will be **proposed** until M5.1).

```ts
// On your game server, with the payout authority's key. Never in a browser.
import { arqWalletAddress, outcomeId, parseCgt, payoutArgs } from 'arqade-sdk'; // placeholder name

const wallet = arqWalletAddress(cartridge.collection, cartridge.item); // where your game's CGT is
const args = payoutArgs({
  collection: cartridge.collection,
  item: cartridge.item,
  outcome: outcomeId('flux-four', match.id, winner.seat), // the same inputs give the same id: retries pay once
  issuedAt: currentBlock,          // a payout older than the outcome window is refused
  ruleVersion: 'flux-four@3',      // must be in the policy's ruleVersions
  to: winner.account,              // the player's SS58 address, proven through QOR ID
  amount: parseCgt('25'),          // bigint Sparks, sent as a decimal string
});
await api.tx.arqWallet.payout(...args).signAndSend(authority); // then wait for finality and the Paid or Accrued event
```

- **Paid twice? Impossible by construction.** Resend the same `outcomeId` after a timeout and the chain answers with
  the first payout, not a second.
- **Below the existential deposit**, to an account that does not exist yet, the payout **accrues** in the Vault and
  the player claims it later. Your game shows it as owed, not as received.
- **Inclusion is not success.** The SDK resolves only at finality with the pallet's `Paid` or `Accrued` event.

## Paid rounds and proof of prize

Before a paid round accepts anyone's entry, its prize is **held** inside the wallet: `open_round(round, prize,
ruleVersion, closesAt)`, with `roundId('your-game', …)` as the id. A player's client reads the hold from chain state
(`Rounds`) and shows "prize reserved on chain" before they pay. From `closesAt`, `settle_round` pays up to 64 winners
from the hold, owes what cannot reach them yet, and releases the rest; `cancel_round` releases it all. Neither you nor
anyone else can withdraw or pay out a held prize, and a round nobody settles in time can be released by anyone.

> **Paid entry that can win a prize stays switched off in production** until the project owner decides U-16 and has a
> legal review (ADR-069). On the devnet the whole flow can be built and tested with test CGT.

## What the chain checks, and what it cannot

It checks: the caller's role, the policy, the outcome id, the hold, the accrual, the delays, and that the Vault serves
only its Cartridge. **It cannot check that your game told the truth about who won.** The authority attests; the chain
limits the damage. Server-side sessions, replay checks and anti-cheat are yours (ARQADE blueprint §10).

## Recovering from mistakes

| What happened | What you do |
| --- | --- |
| The game server's key leaked | Revoke the authority (instant), register a new one. Loss is capped by the policy |
| Your Vault's key leaked | Cancel any pending withdrawal or loosening during the delay; move the Cartridge to a fresh account |
| A payout went to the wrong player | It is final. Pay the right player under a new outcome id; the chain keeps both on record |
| You are retiring the game | Pause, let accruals be claimed or expire, then withdraw after the delay |

## What is not decided

- The protocol's bounds: the shortest `loosenDelayBlocks`, the outcome window, the longest accrual expiry (U-16).
- Whether entry fees may flow into a Vault directly (U-16).
- Deposits for the Vault's storage (U-14) and any platform share (U-15).
- Every name on this page.
