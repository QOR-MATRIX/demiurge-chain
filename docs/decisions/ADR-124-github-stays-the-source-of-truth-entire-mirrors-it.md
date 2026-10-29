# ADR-124: GitHub stays the source of truth; Entire mirrors it

**Status:** **Accepted**, 29 October 2026, by the owner. The owner suggested moving the projects to Entire, the Git forge
launched by GitHub's former chief executive, "if at all possible". Asked to choose between mirroring with GitHub as the
source of truth, migrating fully, or waiting, the owner chose to mirror. It decides **no economic value**, and loosens
no evidence: CI, its gates and the release process are unchanged.

## Context

**Entire** (`entire.com`) launched as a Git forge on 26 October 2026: code hosting, review, CI through partners
(Buildkite, and Depot, which runs existing workflows), and a memory of the agents that worked on the code. It can mirror
a GitHub repository, with GitHub staying the source of truth and pushes going through GitHub, or take a repository over
with its `entire-migrate` plug-in. Its command-line tool records AI agents' sessions beside the commits they made, in
Git refs that follow the remote they are pushed to, with secrets redacted. What it says on 29 October 2026 does not
cover private repositories, organisation permissions or pricing.

This repository relies on GitHub for more than its code:

- CI is GitHub Actions on the public repository (ADR-063), and every pull request and release gate runs there;
- the launcher's releases and its update channel are GitHub releases (`launcher-v*`, `launcher-channel`);
- the homepage is published by a Vercel project connected to this repository (ADR-100);
- pull requests are how work reaches `main`, and HANDOFF records them by number.

## Decision

1. **GitHub stays the source of truth** for the code, pull requests, CI, releases, the launcher's update channel and
   the homepage's deploys.
2. **Entire mirrors this repository from GitHub**, once the owner has made an Entire account for the QOR-MATRIX
   organisation and set the mirror up. Nothing is pushed to Entire directly, and nothing here depends on the mirror.
3. **A move to Entire is a later record**, made when Entire documents what this repository relies on (the list above),
   with a plan for each.

## Not decided here

- **Recording agents' sessions with Entire's command-line tool.** Sessions would be pushed with the code, and this
  repository is public. A session can carry the tokens, keys, email addresses and other values AGENTS.md section 9 keeps
  out of every log, so recording them is a new flow that needs a record of its own and the log check it requires.

## Consequences

- Nothing changes for anyone working here, or for CI.
- The mirror is the owner's to set up: it needs the owner's sign-in to Entire.
