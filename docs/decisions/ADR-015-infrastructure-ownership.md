# ADR-015: Which service owns which infrastructure concern

**Status:** Accepted, 14 September 2026, by the project owner, as proposed. Nothing is deployed yet.
Clarified on 15 September 2026, at the owner's direction: QOR ID serves its own pages on its own subdomain,
and Vercel serves marketing and the web surfaces only (below). The clarification does not change the
decision.
**Superseded for the devnet's nodes** by [ADR-068](ADR-068-the-devnet-on-railway.md) (Railway, 3 October 2026).

## Context

Four hosted services are available: Supabase, Vercel, Fly.io and Railway. Five concerns need a home:
- Postgres and QOR ID's session storage;
- the web surfaces (the public viewer, the remote console, and the static portal);
- the QOR ID service;
- devnet nodes;
- the indexer that will back the public viewer's provenance (migration inventory Q-11).

The owner's constraint is to prefer fewer services over more.

What the workloads need, from the code:
- **QOR ID (`services/qor-auth`)** is a long-running Rust service. It needs Postgres and Redis today
  (`src/main.rs:72-80`).
- **Nodes** are long-running processes. They need persistent disks and a stable inbound TCP port for
  libp2p (`framework/network`). This holds for the current chain and for the Polkadot SDK chain.
- **An indexer** needs an archive node and a database.
- **The web surfaces** are static or server-rendered pages. `apps/portal` already has a `vercel.json`.
- **A Fly configuration already exists** for the node (`fly.toml`, region `iad`).

Vendor facts below come from each vendor's documentation as read on 14 September 2026. Prices change;
check them again before deploying.

## Decision

**Two services: Fly.io for everything that runs continuously or holds data, and Vercel for the web
surfaces. Supabase and Railway are not used.**

| Concern | Owner | Why | Lock-in risk |
| --- | --- | --- | --- |
| **Postgres** (QOR ID data; later the indexer's data) | Fly.io Managed Postgres | It sits on the same private network as QOR ID and the nodes, so no database endpoint faces the internet. Every plan includes a primary with a replica, automatic failover, backups and connection pooling, from $38 a month. Choosing it keeps the service count at two. | **Low.** Standard Postgres, so data leaves with `pg_dump` or logical replication. Backup and restore procedures are Fly-specific. |
| **Session storage** (Redis) | A small Redis on a Fly Machine with a volume, for the devnet stage | QOR ID needs Redis today. Sessions can be rebuilt (users sign in again), so durability matters less than for Postgres. | **Low** lock-in, but it is **self-managed**. Moving sessions into Postgres would remove Redis entirely; that is left open. |
| **QOR ID service** | A Fly.io app | A long-running service with private networking to its database and Redis. Secrets are set as Fly secrets, never in files (SECURITY.md). It uses the same region as the existing `fly.toml`. | **Low.** A container image and a `fly.toml`; any container host can run it. |
| **Devnet nodes** | Fly.io apps, each with a volume and a dedicated IPv4 address | Validators need persistent storage and a fixed inbound port. The existing Fly configuration already covers this. Dedicated IPv4 costs $2 a month per address. | **Medium.** Bootnode addresses and volumes are tied to Fly. Moving means new addresses and a full resync: acceptable for a devnet, less so for a testnet. |
| **Indexer** | A Fly.io app, with its archive node on a Fly volume and its data in the same Managed Postgres cluster | The same reasons as the nodes and QOR ID. Which indexer to run is still Q-11. | **Low to medium,** depending on the indexer chosen. |
| **Web surfaces** (public viewer, remote console, portal) | Vercel | Built for static and server-rendered sites, with preview deployments per branch and no servers to run. The portal is already configured for it. | **Low** if pages use only standard framework features. **Medium** if Vercel-specific features creep in (edge configuration, image optimisation, key-value storage). The rule: do not use them. |

**"Devnet nodes" means nodes of the Polkadot SDK chain.** The current custom Rust devnet is untrusted
(migration inventory R-1: its transaction validation accepts forged signatures for small-order keys). It
is **never deployed** to Fly or anywhere else public, whatever `fly.toml` describes.

### Clarification, 15 September 2026: QOR ID's pages and domain

This makes explicit a split the table above implies. It does not change the decision.

- **QOR ID serves its own pages, on its own subdomain.** The links in QOR ID's emails (address
  verification, confirmation of a new address, password reset) open pages QOR ID serves itself. They are
  reached at `https://<qor-id-subdomain>.demiurge.cloud`, a placeholder: the owner sets the name at
  deployment. That origin is QOR ID's `BASE_URL`.
- **Vercel serves marketing and the web surfaces only.** It does not proxy, rewrite or redirect any of
  QOR ID's paths.
- **Why.** Routing `demiurge.cloud` paths through Vercel to QOR ID on Fly would put a proxy and a rewrite
  layer between people and one-use tokens, and add a failure mode for no benefit. A dedicated subdomain
  keeps QOR ID's surface separate and the path direct.
- The subdomain's name and its TLS arrangement stay open until deployment (see "Open questions").

### Clarification, 15 September 2026: the archive node is a dependency of the public viewer

This makes explicit what the Indexer row implies, now that [ADR-028](ADR-028-provenance-from-events-archive-node-and-indexer.md)
has answered Q-11. It does not change the decision.

- **The public viewer depends on an archive node.** Its provenance comes from events that an indexer reads from an
  archive node (ADR-028). If that node is down, new history stops reaching the viewer. If it is lost, rebuilding the
  index needs a new archive node synced from genesis.
- **The archive node is its own Fly.io app, with a volume.** It runs with
  `--state-pruning archive-canonical --blocks-pruning archive-canonical`, and its RPC is private to the indexer.
- **Its disk grows with every block's state changes.** It joins the cost lines to watch, below.
- **The indexer is SQD's squid SDK** (ADR-028), a Fly.io app writing to the Managed Postgres cluster.
- **The table's "Which indexer to run is still Q-11"** is answered by ADR-028.

### Not used

**Supabase.** Its distinctive value is its own auth, storage, realtime and row-level-security APIs, and
QOR ID already is Demiurge's auth. Used only as Postgres, it would:
- add a vendor;
- put the database on the public internet;
- need IPv6 or a paid IPv4 add-on for direct connections;
- price point-in-time recovery separately, at $100 a month per 7-day retention window.

**Railway.** It overlaps Fly for services. Its TCP proxy assigns the public port and it allows no inbound
UDP, which is a poor fit for peer-to-peer nodes that advertise a fixed address.

## Consequences

**Concentration: accepted risk, revisit before mainnet.** A Fly outage takes QOR ID, the devnet and the
indexer down together. The owner accepted that risk on 14 September 2026 for the stages before mainnet.

It is revisited before mainnet genesis, together with:
- exporting database backups somewhere off Fly;
- running validators on more than one provider. A network whose validators all sit with one company is not
  decentralised.

**Cost lines to watch.**
- Dedicated IPv4 per node.
- Volume snapshots, billed since January 2026.
- Inter-region private networking, billed since February 2026.

Keeping everything in one region avoids the last.

**Scope.** This record covers project-run services up to the public testnet. Mainnet validators are
operated by many parties and are not a hosting decision.

**What is deployed first,** and when, is decided separately.

**Open questions.**
- Redis versus Postgres-backed sessions.
- Where off-Fly backups go.
- The domain and TLS arrangement for `demiurge.cloud`, whose DNS still points at a decommissioned server.

## Sources

- Fly.io: [Managed Postgres](https://fly.io/docs/mpg/), [pricing](https://fly.io/docs/about/pricing/), [cost management](https://fly.io/docs/about/cost-management/)
- Supabase: [connection pooling and limits](https://supabase.com/docs/guides/database/connecting-to-postgres/pooling-and-limits), [backups and PITR](https://axonbuild.com/blog/supabase-backup/)
- Railway: [TCP proxy](https://docs.railway.com/networking/tcp-proxy), [inbound UDP](https://station.railway.com/feedback/udp-inboud-connections-9f036ffc)
