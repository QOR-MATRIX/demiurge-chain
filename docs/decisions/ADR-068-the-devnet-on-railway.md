# ADR-068: The devnet runs on Railway: two private validators, a public RPC node, and test CGT for a faucet

**Status:** Accepted, 3 October 2026, by the project owner, who chose the recommended answer to each of the nine
questions in [`../architecture/DEVNET_PLAN.md`](../architecture/DEVNET_PLAN.md) §9.
**Supersedes:** [ADR-015](ADR-015-infrastructure-ownership.md) for the devnet's nodes only (Fly.io). ADR-015 stays
in force for everything else it covers that ADR-063 did not already move.
**Accepts:** [ADR-044](ADR-044-validators-are-not-publicly-addressable.md), which was Proposed.
**Relies on:** [ADR-037](ADR-037-sudo-on-development-and-test-networks-only.md) (sudo on development and test
networks only) and [ADR-063](ADR-063-public-repository-actions-and-railway.md) (Railway).

## Context

The chain runs only on the owner's computer with `--dev`. It stops when the computer sleeps, restarts as a new chain,
and cannot be reached by anyone else, so the Market only ever shows one machine's listings. `alpha.devnet-live` has
had an empty URL since it was written. The plan was written on 1 October and waited on the owner's answers.

## Decision

1. **Host: Railway**, project `demiurge`, beside QOR ID. The image is built in GitHub Actions and published to the
   organisation's container registry; Railway runs the image.
2. **Topology A: two validators and one RPC node** (accepting ADR-044). The validators have no public hostname and
   peer over Railway's private network. Only the RPC node answers the public, with `--rpc-methods safe`, at
   `rpc.qorsync.dev` through Railway's TLS edge. With two validators, finality pauses while either restarts; that is
   accepted and said.
3. **Genesis holds two balances and no others.** A **sudo account** with exactly the existential deposit, so it
   exists and can sign upgrades; and a separate **faucet account** with a stock of **test CGT**, the same marked
   placeholder the development specification already endows its accounts with. Both are the owner's, made in a
   browser wallet on the owner's device; only their addresses ever leave it. This is test currency on a test
   network. It decides nothing about mainnet's genesis split (OPEN-2) or issuance (OPEN-1), which stay open, and a
   reset of the devnet discards it.
4. **Names:** chain name `Demiurge Devnet`, id `demiurge_devnet`, chain type `Live`.
5. **Alpha's gates:** `alpha.devnet-live` reads `https://rpc.qorsync.dev/health/readiness`, written into
   `GATES.toml` once it answers; and a new criterion, **`alpha.devnet-finality`**, requires the live network to keep
   finalising blocks, not merely to answer. Both are tightenings.
6. **Hands:** the assistant creates the Railway services through Railway's tools where it is permitted, and the
   owner clicks where it is not. The owner holds the sudo and faucet keys; no assistant or service ever sees them.

## Consequences

- AGENTS.md §5's "no path that creates CGT outside `--dev`" is met by reading: the devnet's two balances are written
  once at genesis by the owner's decision, on a test network, and nothing in the runtime creates CGT afterwards. A
  mint, a sale or a royalty still only moves it.
- Every validator's session keys sit on Railway's disks. ADR-037 already accepts that a devnet holds nothing of value.
- Railway's networking does not suit outside peers (ADR-015's objection). That bites at the public testnet, when
  outside validators join, and is decided then.
- Cost is usage-based and not yet measured; the plan's $11–22 a month is an illustration, replaced by the bill after
  a week.
