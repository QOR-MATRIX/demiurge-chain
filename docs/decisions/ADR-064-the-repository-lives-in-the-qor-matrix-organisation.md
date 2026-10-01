# ADR-064: The public repository lives in the QOR-MATRIX organisation

**Status:** Accepted, 1 October 2026, by the project owner.
**Amends:** [ADR-063](ADR-063-public-repository-actions-and-railway.md), decision 1 (where the public repository
lives) and decision 2 (which repository CI runs on). Nothing else in ADR-063 changes.

## Context

ADR-063 published the code as `ALaustrup/demiurge-chain` so that GitHub Actions would be free. No job ever ran there.
Every run, from the first on 29 September to a re-run on 1 October, was refused before it started: "The job was not
started because your account is locked due to a billing issue". The lock is on the owner's personal GitHub account,
for a charge the owner disputes and will not pay.

GitHub bills Actions to whoever owns the repository. The owner created the organisation `QOR-MATRIX` (free plan) on
1 October, and pushed the same tree to `QOR-MATRIX/demiurge-chain`, public. **Its first run started and its jobs
executed**, seen by the owner in the Actions tab. That is the first time any job of this workflow has executed
anywhere.

## Decision

1. **The public repository is `QOR-MATRIX/demiurge-chain`.** It was created by pushing `7e154e1`, the head of
   `ALaustrup/demiurge-chain`, so the two hold the same commits. It was pushed, not transferred, so that the test
   cost nothing if it failed.
2. **CI is GitHub Actions on `QOR-MATRIX/demiurge-chain`**, the same workflow file unchanged. `GATES.toml`'s `[ci]`
   names it, and the launcher's gates dashboard reads it from there.
3. **`ALaustrup/demiurge-chain` is no longer pushed to.** The owner archives it, with a pointer to the new
   repository, once Railway deploys from the organisation's.
4. **`ALaustrup/demiurge-cloud` stays the private archive**, as ADR-063 left it.

## Consequences

- CI no longer depends on the personal account's billing state. The dispute is the owner's to pursue or drop and
  blocks nothing here.
- Railway's `qor-auth` service is connected to the old repository until Railway's GitHub app is given the
  organisation and the service's source is switched. Until then a push does not redeploy QOR ID.
- The organisation has one owner, whose personal account is the locked one. If GitHub ever extends that lock to
  organisations the account owns, CI stops again; adding a second owner would remove that single point.
- Whether the workflow passes is a separate question from whether it runs. Its Linux-only steps had never executed
  before this, so the first runs may find faults.
