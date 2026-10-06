# ADR-058: CI runs on Woodpecker, not GitHub Actions

**Status:** Accepted, 28 September 2026, by the project owner. **Superseded on 29 September 2026 by [ADR-063](ADR-063-public-repository-actions-and-railway.md).**
Before that, **decision 1 was superseded the same day by [ADR-059](ADR-059-operations-on-oracle-always-free.md):** the server runs on the free Oracle Cloud operations
machine, not Fly.io; agents connect to `ci-grpc.qorsync.dev:443`. The rest stands.
**Relates to:** roadmap item L1.7 (CI), [ADR-015](ADR-015-infrastructure-ownership.md) (Fly.io for services),
[ADR-042](ADR-042-two-domains.md) (`qorsync.dev` for operations, accepted the same day).

## Context

No GitHub Actions job in this repository ever executed: every run ended in a startup failure that
`probe.yml` traced to account billing on 21 September 2026. On 28 September the owner ruled that Actions trying
to charge them is unacceptable and chose **Woodpecker CI**, with Codeberg's Actions as a later fallback.

Vercel was considered as a host and cannot be one: it runs static sites and short-lived functions, and
Woodpecker needs an always-on server and build machines that run Docker. Three hosts were put to the owner:
a Fly.io server with an agent on the owner's computer, everything on the owner's computer, or Codeberg's hosted
Woodpecker. **The owner chose the first.**

## Decision

1. **The Woodpecker server runs on Fly.io at `https://ci.qorsync.dev`** (`infra/woodpecker/fly.toml`): a 256 MB
   machine that never sleeps, a 1 GB volume for its SQLite history, and a dedicated IPv4 for the agents' gRPC port.
   About $4–5 a month. Only the owner may sign in (`WOODPECKER_OPEN=false`).
2. **Builds run on an agent on the owner's computer**, in Docker (`infra/woodpecker/agent/`). It connects out to
   the server, so nothing on the computer is exposed. Pipelines queue while it is off.
3. **The code stays on GitHub.** Woodpecker reads it through a GitHub OAuth app and webhooks, which are free and
   unrelated to Actions billing.
4. **The pipelines are `.woodpecker/*.yaml`**, one per former Actions job, command for command: `chain`,
   `two-validators` (a nightly cron and by hand), `qor-auth`, `launcher`, `coverage` and `security`. Caches are
   named Docker volumes on the agent, so the repository is marked Trusted for volumes.
5. **GitHub Actions is switched off.** `ci.yml` starts only by hand and is deleted once Woodpecker has passed on
   `main`; `probe.yml` is deleted.
6. **Codeberg is the fallback.** Woodpecker speaks Codeberg natively and the pipelines would not change; the
   repository's Apache-2.0 licence qualifies for Codeberg's hosted instance.

## Consequences

- **The `ci` kind in `docs/GATES.toml` reads Woodpecker**, not `gh run list`; logged there as an evidence-rule
  change the owner approved. Until the gates dashboard's reader (`tools/qor-launcher/src-tauri/src/gates.rs`) is
  rewritten against Woodpecker's API, the ci units cannot be read and count as unmeasurable, as they never
  counted as met before either. **Owed.**
- **Coverage reports** are written to a volume on the agent instead of an Actions artifact; reading them into
  the dashboard is owed with the reader above.
- The launcher's view checks have only ever run on Windows. Their first Linux run is Woodpecker's.
- A CI server that is down misses webhooks; Woodpecker re-reads on the next push. The agent being off only delays.
