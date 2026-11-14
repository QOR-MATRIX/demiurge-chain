# ADR-131: Vercel Web Analytics on the public homepage

Homepage ownership and implementation location are superseded by
[ADR-132](ADR-132-the-homepage-moves-to-qor-os.md), 13 November 2026. The decision below retains its original scope.

**Status:** Accepted, 13 November 2026, under the owner's explicit request
`npm i @vercel/analytics` and existing development authority. This authorises
source work; it does not claim the owner enabled collection or requested a deployment.
Amends ADR-100 decision 2's no-build rule and its exclusion of analytics.
The homepage's ownership, content, visual design and canonical domain stay as
decided by ADR-100, ADR-108, ADR-123, ADR-127, ADR-128 and ADR-130.

## Decision

1. Install `@vercel/analytics` in `site/`, independent of the frozen root npm
   workspace. Pin version 2.0.1 and its integrity in `package-lock.json`. Its MIT
   licence accompanies its browser module. No additional framework is needed.
2. Use a small Node 24 build to copy only public assets and the package's
   unmodified browser entry point into `dist/`. Neither development files nor
   `node_modules/` are served. Install from the lock with lifecycle scripts disabled.
3. Initialise only on the canonical HTTPS homepage. Respect browser Do Not Track
   and Global Privacy Control opt-outs. Accept homepage pageviews only; discard
   URL queries, fragments and extra fields, and refuse custom events and other
   paths or origins. Do not add identity, wallet, session or account values.
4. Keep scripts and collection same-origin under the existing content security
   policy. Use `no-referrer` and disable development event logging. Local and
   preview visits do not initialise analytics.
5. Check the static output and the installed SDK's initialisation/redaction
   behaviour in the homepage CI job; audit the locked dependency and review
   weekly update proposals. Dashboard activation and production verification
   require working project access and the owner's publication instruction.

## Evidence and limits

The initial locked install, 41-file build, 33 static checks, six analytics/build
tests and npm audit pass locally with Node 24.19.0/npm 12.0.1. The tests use a
simulated document and the real installed SDK; they do not assert browser or
live dashboard acceptance. Vercel project access still returns 404 after renewed
team approval. No production deployment or analytics activation was performed.

Primary references: Vercel's [quickstart](https://vercel.com/docs/analytics/quickstart),
[package configuration](https://vercel.com/docs/analytics/package), and
[data redaction](https://vercel.com/docs/analytics/redacting-sensitive-data).
