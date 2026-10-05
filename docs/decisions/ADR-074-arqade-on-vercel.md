# ADR-074: ARQADE is hosted on Vercel, with Postgres, and its players sign in with QOR ID

**Status:** Accepted, 5 October 2026. The host is the owner's decision ("create a new ARQADE site using v0 and host it
ourselves through Vercel"); the engineering below follows from it under the owner's delegation. It settles the hosting
question [ADR-069](ADR-069-arqade-the-gaming-platform.md) left open, and changes two parts of
[ADR-073](ADR-073-qor-id-sign-in-for-apps.md) (decisions 11 and 12, below). It decides no economic value.

**Identity form changed, 5 October 2026, by [ADR-075](ADR-075-one-name-per-qor-id.md):** a QOR ID is the username
alone, unique on its own; the `#0001` discriminator below is retired. The text is kept as it was written.

## Context

ARQADE ran on OpenAI Sites: vinext on Cloudflare Workers, Cloudflare D1 through drizzle, and players identified by the
Sites host's own ChatGPT sign-in. Publishing needed the owner's Sites tooling, which this repository cannot drive; on
5 October the owner republished, the site moved to a new address on its own
(`demiurge-arcade.andyithink.chatgpt.site`), every page sat behind the host's sign-in wall, and nothing could be checked
from outside. QOR ID sign-in (ADR-073) was built and live, but the site could not be seen to use it.

v0 is Vercel's tool for generating and editing Next.js screens. ARQADE was already Next.js code.

## Decision

1. **ARQADE is hosted on Vercel**, in the owner's Vercel team, deployed from `products/arqade/` in this repository.
   Its address is the `*.vercel.app` one Vercel assigns until the owner chooses a domain. **v0** is how the owner
   designs and edits screens; what v0 produces comes back into `products/arqade/` and passes the same CI before it is
   deployed. The site is not rebuilt from scratch: its games, devnet reads, SDK and sign-in were already tested.
2. **Standard Next.js** replaces vinext. Nothing in the site depends on Cloudflare or on Sites.
3. **Postgres replaces D1**, connected through `DATABASE_URL` (or `POSTGRES_URL`) as Vercel's Postgres integration
   sets it; Neon from Vercel's marketplace is the default. The arcade keeps the statement interface it was written
   against (`lib/db.ts`: prepare, bind, run, all, first, batch), so its rules did not change; a batch is one
   transaction on one connection. Statements keep `?` placeholders, numbered for Postgres; values are always bound.
   Times are milliseconds in `bigint`, read back as numbers (all far below 2^53; no CGT amount is stored here).
4. **Schema changes are SQL files** in `products/arqade/db/migrations/`, applied in order and recorded in
   `arqade_migrations` by `scripts/migrate.mjs` before every build, **only for a production build** (or by hand with
   `ARQADE_MIGRATE=1`). A preview of an unmerged branch never changes the live database. drizzle is removed.
5. **A player is a QOR ID account.** The live arcade (matches, rankings, chat) requires QOR ID sign-in; there is no
   ChatGPT sign-in and no anonymous "Explorer" alias. Other players see the QOR ID (`name#0001`) and an id that is the
   SHA-256 of the account's `sub`, never the `sub` itself. This **replaces ADR-073 decision 12** (binding a host
   identity to a QOR identity on proof of both): with one identity there is nothing to bind. Solo play still needs no
   sign-in.
6. **The live arcade accepts QOR ID's answer for 30 seconds.** It polls every two seconds; asking `/oauth/userinfo` on
   each poll would put every player's polling on QOR ID. A session row records when QOR ID last confirmed it
   (`checked`), and the arcade asks again once that is over 30 seconds old, so a revocation at QOR ID ends a player's
   arcade session within 30 seconds. The QOR Identity card still asks every time. This **narrows ADR-073 decision 11**
   ("each check asks `/oauth/userinfo`") for the arcade only.
7. **The site's errors name their real cause.** A sign-in that cannot start because of the site (its settings or its
   database) says so, instead of saying QOR ID could not be reached; logs record an error's kind and Postgres code,
   never its text, which can quote the values a statement carried.
8. **Origin checks and redirects use the request's Host header and relative paths**, not the URL the server rebuilds,
   which a proxy or `next start` can name differently.
9. **Data on Sites stays there.** Its practice players, matches and chat are not migrated: they were tied to ChatGPT
   identities this site no longer has.

## Consequences

- Evidence, 5 October 2026: 41 tests (the arenas' rules, races, rankings and chat now against Postgres in PGlite,
  with the site's own migrations; the sign-in module; the devnet reader; the SDK), type check, lint without errors, a
  production `next build`; migrations applied to a real Postgres 16 twice (the second a no-op); and the whole flow in
  headless Chrome against that Postgres and a local QOR ID: QOR ID's page "Sign in to ARQADE", back signed in as
  `arqtester#0001`, the live arcade accepting that player and creating a match hosted by `arqtester#0001`, the cookie
  unreadable by scripts, and after sign-out the arcade refusing with 401. Two planted faults (the 30-second allowance
  never expiring; an origin check passing any site) each failed a test.
- **Going live** needs the owner: a Postgres database added to the Vercel project (Neon, from the Vercel dashboard),
  ARQADE's Vercel callback added to QOR ID's `QOR_OAUTH_CLIENTS` on Railway, and the site's `QOR_CLIENT_ID`,
  `QOR_CLIENT_SECRET` and `QOR_REDIRECT_URI` set in Vercel.
- **Vercel's Hobby plan is for non-commercial use.** ARQADE today takes no payment and moves only test CGT. Before it
  takes a real payment or pays out real CGT (P7.5 onward, U-16), the project moves to a plan that allows commercial use;
  that is the owner's decision when it arises.
- The Sites site stops being ARQADE's. Taking it down is the owner's choice, in Sites.
- v0's edits arrive as changes to this repository; they are reviewed and tested like any other.
