# ADR-044: Validators have no public hostname; public RPC is a separate node

**Status:** Proposed 20 September 2026; **accepted 3 October 2026 by the project owner, through
[ADR-068](ADR-068-the-devnet-on-railway.md).** The decision below is unchanged. The
topology behind it is [`../architecture/HOSTING.md`](../architecture/HOSTING.md) §4.

**Relates to:** [ADR-015](ADR-015-infrastructure-ownership.md), which gives each devnet node a Fly app, a
volume and a dedicated IPv4; [ADR-028](ADR-028-provenance-from-events-archive-node-and-indexer.md), which
introduces the archive node; [ADR-020](ADR-020-permissioned-validators-now-npos-later.md), whose
governance-chosen validator set is what these machines hold keys for.

## Context

Nothing is deployed today. Validators run on laptops, and the two-validator script starts them with RPC
on localhost because that is where a script reaches them. The first time a node runs somewhere other
people can reach, that arrangement becomes a decision rather than a default — and defaults chosen by
accident are how a validator ends up answering RPC on the public internet.

A validator holds a session key. It is the one asset on the machine an attacker wants, and every
listening service is a path toward it. Meanwhile there are three legitimate reasons for the outside world
to talk to a node: an SDK submitting transactions (M5.1), the indexer reading history (ADR-028), and the
launcher pointing at a chain. None of them requires talking to a validator.

## Decision

1. **A validator has no public hostname and no DNS record.** It is not discoverable by name. Its peer
   addresses are configured for its peers, not published.

2. **A validator exposes exactly one port to the internet: the libp2p p2p port.** It needs inbound p2p to
   peer at all, which ADR-015 already accounted for with a dedicated IPv4 per node.

3. **A validator's RPC is not reachable from the internet.** Bound to localhost or to the private
   network, never `--rpc-external`. **`--rpc-methods=safe` is not a substitute:** the safest RPC is the
   one that is not listening.

4. **Public RPC is a different node.** A full node — not a validator, not the archive node — running
   `--rpc-external --rpc-methods safe` behind TLS, rate-limited, holding no session key. It is the one
   that gets a name, `rpc.qorsync.dev` under ADR-042.

5. **The archive node stays private to the indexer**, which restates ADR-015's clarification rather than
   changing it. It gets no public name even though it is the most expensive disk in the system.

6. **Whatever is published as a bootnode is the public RPC node, never a validator.** A bootnode address
   is public by definition; that is what a bootnode is for.

## Consequences

- **An RPC node is one more machine to pay for** — `HOSTING.md` §6 puts it at roughly $7–12 a month,
  estimated. That is the cost of not answering the internet from a machine holding a signing key.
- **Public RPC load cannot stop block production.** If the RPC node is overwhelmed or taken down, the
  validators neither know nor care. This is the main thing the separation buys.
- **Reaching a validator to operate it needs a private path** — Fly's private network, or an SSH path
  that is itself a decision. This record does not choose one, and whoever deploys the first validator has
  to, because the alternative is discovering that the convenient path was the public one.
- **The two-validator script is unaffected.** It runs two local nodes and talks to them on localhost,
  which is consistent with decision 3 rather than an exception to it.
- **This says nothing about decentralisation.** ADR-015 already records that validators all sitting with
  one provider is not a decentralised network, and that it is revisited before mainnet. A better topology
  at one provider is still one provider.
- **Monitoring needs the private path too.** If validator metrics are wanted, they are collected over the
  private network. `--no-prometheus` is what the two-validator script uses today, and exposing metrics
  publicly would reintroduce exactly the surface decision 3 removes.

## What this record does not decide

- Whether validators sit behind sentry nodes. Sentries are the stronger version of decision 1 and would
  be worth their own record before mainnet; at devnet and testnet scale they are more machines to pay for
  and more to get wrong.
- How operators reach a validator, as above.
- Anything about mainnet validators, who are operated by other people and are not a hosting decision at
  all.
