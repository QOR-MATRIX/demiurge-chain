# ADR-132: The homepage moves to QOR OS

**Status:** Accepted, 13 November 2026, under the owner's explicit direction to
align QOR-MATRIX/QOR-OS with Astra Matrix / QOR. Mirrors QOR OS QOR-0027.

## Context

The existing `qor` Vercel project still read this repository's `site/`, while
the owner named QOR OS as the canonical source. The authenticated Vercel CLI
can manage the project even though the connected app returns 404.

## Decision

1. Develop and deploy the homepage from `QOR-MATRIX/QOR-OS`, branch `main`,
   root `site`. Its source import from this repository at `d4c51d7` records
   Git blob hashes and preserves source, font and Analytics licences.
2. Keep the existing Vercel project `prj_pUNenZCSC3eh2tyJdHbHCt9rUlW0` in
   `astramatrix`, its QORSYNC.DEV binding and `www` redirect. The Git connection
   is verified as QOR OS repository `1435244138`. Use Node 24, locked install
   with scripts disabled, `npm run build`, and public output `dist`.
3. Freeze this repository's homepage copy. Disable its Git-triggered deployments
   and remove the duplicate homepage build/audit job and npm update proposals.
   QOR OS owns those checks in `.github/workflows/homepage.yml`; the existing
   upstream content, design and language checks remain intact.
4. Keep the chain, QOR ID, Social, ARQADE web application, Launcher/core and
   signed release assets here. QOR OS's core pin does not move. This changes
   only ADR-100's homepage ownership and ADR-131's implementation location.
5. Treat a working connection separately from a deployed revision or analytics
   collection. QOR OS's `docs/HOSTING.md` and session-08 receipts own the final
   live-site verification. Do not create replacement projects or copy credentials.

## Consequences

There is one maintained homepage and one deployment project. The frozen source
and its history remain available for licence/provenance review. This transfer
does not resolve the service audits or publish any network, device or package release.
