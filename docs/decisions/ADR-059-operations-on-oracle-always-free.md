# ADR-059: The operations server runs on Oracle Cloud's Always Free tier

> Editorial note, 13 November 2026: retired host/contact literals are redacted. Historical claims retain their original dates and are not current deployment evidence. ADR-130 establishes QORSYNC.DEV as the only public domain.

**Status:** Accepted, 28 September 2026, by the project owner. **Superseded the same day by
[ADR-060](ADR-060-operations-on-the-owners-computer.md)**: the owner rejected Oracle; the stack runs on the owner's
computer behind a Cloudflare Tunnel. Nothing was ever created on Oracle.
**Supersedes:** [ADR-058](ADR-058-ci-on-woodpecker.md)'s decision 1 (the Woodpecker server on Fly.io), the same day.
**Departs from:** [ADR-015](ADR-015-infrastructure-ownership.md), which placed QOR ID, Postgres, Redis and the
nodes on Fly.io, for the services named here. ADR-015 is otherwise unchanged; mainnet hosting is still outside
this record, as it is outside ADR-015.

## Context

Asked to deploy QOR ID and a public RPC node at ADR-015's estimated $50–70 a month, the owner asked for a free
option. Oracle Cloud's Always Free tier gives one account up to 4 Arm cores, 24 GB of memory and 200 GB of disk,
permanently, which fits QOR ID, Postgres, Redis, the Woodpecker server and later a development RPC node. The
alternatives put to the owner, the owner's own computer behind a tunnel or free web platforms that cannot run a
node, were weaker. **The owner chose Oracle.**

## Decision

1. **One `VM.Standard.A1.Flex` machine, 4 OCPUs and 24 GB, Ubuntu 24.04 on aarch64**, runs the stack in
   `infra/oracle/compose.yaml`: Caddy, QOR ID (built for ARM on the machine), Postgres, Redis and the Woodpecker
   server.
2. **Caddy is the only public listener** and serves one name per component with its own Let's Encrypt
   certificate (ADR-042, decisions 4 and 6): `id.qorsync.dev`, `ci.qorsync.dev`, and `ci-grpc.qorsync.dev` for
   Woodpecker's agents. A name gets a DNS record only when its component is running.
3. **Secrets are made on the machine** (`bootstrap.sh`) and never committed or printed. The Woodpecker agent secret
   is the one that leaves, to the owner's computer, over SSH. The GitHub OAuth secret is typed by the owner into
   `set-github-oauth.ps1` and goes straight to the machine over SSH.
4. **The account is upgraded to Pay As You Go with a $1 budget alert.** Oracle reclaims idle machines on
   free-only accounts; a Pay As You Go account keeps them, and Always Free resources are still not charged. The
   alert makes any charge visible at once.
5. **No public RPC node until the test chain has a private sudo key.** The `--dev` specification's sudo key is the
   well-known `//Alice`: an RPC endpoint on it lets anyone take the chain over. `rpc.qorsync.dev` is created with a
   test specification whose sudo key is generated privately; that specification is owed in `chain/`.
6. **QOR ID runs in production mode with no email service**, so it refuses email flows and says so; sign-up and
   sign-in with a vault key need none. Email waits on ADR-042's decision 3 (`retired-public-host`, whose DNS is
   still at GoDaddy).

## Consequences

- **$0 a month**, as long as usage stays inside the Always Free allowance. The budget alert is the guard.
- **One machine is one point of failure**, and Postgres on it has no off-machine backup yet (ADR-015's open
  question on backups still applies). Acceptable for development; not for mainnet.
- **Capacity is not guaranteed at signup**: Ampere machines are often out of capacity, and a retry or another
  availability domain may be needed.
- The launcher's QOR ID endpoint becomes `https://id.qorsync.dev/api/v1` once the stack is up; its default
  (`retired-public-host`) changes in the same change that proves the endpoint answers.
