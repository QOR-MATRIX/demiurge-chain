# ADR-063: A public repository without its history, CI on GitHub Actions, and QOR ID on Railway

**Status:** Accepted, 29 September 2026, by the project owner.
**Supersedes:** [ADR-058](ADR-058-ci-on-woodpecker.md) (CI on Woodpecker) and
[ADR-060](ADR-060-operations-on-the-owners-computer.md) (operations on the owner's computer), and, for QOR ID,
Postgres and Redis, [ADR-015](ADR-015-infrastructure-ownership.md)'s Fly.io.
**Amended by:** [ADR-064](ADR-064-the-repository-lives-in-the-qor-matrix-organisation.md), 1 October 2026: the public
repository, and CI with it, moved to `QOR-MATRIX/demiurge-chain`.

## Context

Operations ran on the owner's computer behind a Cloudflare Tunnel (ADR-060): QOR ID, its Postgres and Redis, and
Woodpecker CI with its build agent. The owner travels, so everything was down whenever the computer was.

The owner chose Railway for what must always be online. Railway runs QOR ID, Postgres and Redis well, but its
containers are non-privileged, so Docker-in-Docker fails (Railway's own documentation), and every CI pipeline here
runs its steps in Docker. The owner chose GitHub Actions for CI. Actions on a private repository is billed per
minute, which the owner had ruled out on 28 September; on a public repository it is free.

The private repository could not simply be made public. Credentials removed from configuration remain in its
history, nine credential-shaped values in two tracked files were never rotated (`SECURITY.md`), and the deleted
pre-realignment documentation, which no assistant may read, is in its history too.

## Decision

1. **The code is published as `ALaustrup/demiurge-chain`, public, from today's tree, without history.** Its first
   commit, `e611c99`, is the private repository's tree at `d9bb19a` less the two files holding the unrotated values
   (`docker/n8n/docker-compose.yml`, `docker/docker-compose.testnet.yml`), which are ignored so they cannot be added
   back. Before it was pushed, gitleaks scanned the whole tree: six findings, all false positives (empty keys in
   `.env.example`, test fixtures, a placeholder, a deliberately wrong test secret). **`ALaustrup/demiurge-cloud` stays
   private, unchanged, as the archive.** Nothing was rewritten.
2. **CI is GitHub Actions on the public repository**: `.github/workflows/ci.yml`, on push and pull request to `main`,
   nightly for two validators, and by hand. `.woodpecker/` is deleted; the faults Woodpecker's four runs found are
   carried into the workflow. The launcher's browser checks keep Chromium's sandbox, which Actions' runners allow.
3. **QOR ID, Postgres and Redis run on Railway**, project `demiurge`, region `iad`, from `services/qor-auth` on
   `main`. Postgres and Redis are Railway's templates, whose passwords Railway generates. QOR ID's service settings
   are applied on the service, because Railway has deprecated `railway.json`, which is deleted.
   `services/qor-auth/DEPLOY-RAILWAY.md` records every setting.
4. **No secret passes through a chat or a shell history.** QOR ID's two JWT secrets were generated into a file on the
   owner's computer for the owner to paste into Railway; the data copy asks for the database password when it runs.
5. **The launcher's local QOR ID database was copied to Railway**, since it held the owner's account; the operations
   stack's held none.
6. **The public chain RPC is still withheld**, for the reason ADR-060 gave: a `--dev` chain's sudo key is public.

## Consequences

- CI no longer depends on any computer being on. Its cost is $0; Railway's is usage-based, expected at roughly
  $5–15 a month for these three services.
- Anything committed from now on is public. `SECURITY.md`'s rule for secrets in the tree matters more, not less.
- `infra/woodpecker/` is deleted. `infra/ops/` stays until `id.qorsync.dev` points at Railway, then goes too.
- `ci.qorsync.dev` has nothing to serve once the operations stack is stopped, and its DNS record is removed.
- The documents and review notes about QOR ID's known gaps are public along with a public QOR ID. None is a
  credential, and each is recorded as accepted or scheduled.
