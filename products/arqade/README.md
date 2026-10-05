# ARQADE

The gaming platform powered by Demiurge (ADR-069): solo games, two live multiplayer arenas, read-only views of
Demiurge Devnet, QOR ID sign-in, and the developer SDK in [`sdk/`](sdk/README.md). Design:
[`docs/blueprints/arqade.md`](../../docs/blueprints/arqade.md); steps: P7 in [`docs/DIRECTION.md`](../../docs/DIRECTION.md).

It is a standard Next.js site hosted on **Vercel**, with **Postgres** for the live arcade, and players sign in with
**QOR ID** (ADR-074). Screens may be designed in v0; what v0 produces comes back here and passes CI like any change.

## Run it

Node 22.13 or later.

```sh
npm ci
npm test          # 41 tests: arenas and chat against Postgres (PGlite), sign-in, the devnet reader, the SDK
npm run typecheck
npm run lint
npm run build     # migrations (production only, see below), then next build
```

`npm run dev` serves the site on `http://localhost:3000`. Solo games, the devnet views and the SDK need nothing else.
The live arcade and sign-in need the settings below.

## Settings

Set in Vercel's project settings (or in `.env.local` for a local run; `.env*` is ignored by git).

| Name | What it is |
|---|---|
| `DATABASE_URL` (or `POSTGRES_URL`) | The Postgres connection string. Vercel's Postgres integration (Neon) sets it |
| `QOR_CLIENT_ID` | ARQADE's id in QOR ID's registry: `arqade` |
| `QOR_CLIENT_SECRET` | ARQADE's secret for QOR ID; at least 32 characters. Only its SHA-256 is given to QOR ID (`QOR_OAUTH_CLIENTS` on Railway). **Never committed** |
| `QOR_REDIRECT_URI` | This site's callback, `https://<site>/api/auth/callback`, registered at QOR ID exactly |
| `QOR_ID_URL` | QOR ID's address; defaults to `https://id.qorsync.dev` |

Without a database the arcade says it is being connected; without the three `QOR_*` settings the sign-in button is
not shown.

## The database

Schema changes are SQL files in [`db/migrations/`](db/migrations/), applied in order and recorded in
`arqade_migrations` by [`scripts/migrate.mjs`](scripts/migrate.mjs). It runs before every build but only migrates a
**production** build (`VERCEL_ENV=production`), or any database when run by hand with `ARQADE_MIGRATE=1`:

```sh
ARQADE_MIGRATE=1 DATABASE_URL=postgres://… npm run migrate
```

A preview of an unmerged branch never changes the live database. Statements are written with `?` placeholders and
values are always bound ([`lib/db.ts`](lib/db.ts)).

## What it is not, yet

No payment, no real CGT, no signing from a player's Vault (P7.4 onward). Vercel's Hobby plan is for non-commercial use;
the project moves to a plan that allows commercial use before ARQADE takes a payment (ADR-074).

Every control, what it reads and how it was checked: [`INTEGRATION_INVENTORY.md`](INTEGRATION_INVENTORY.md).
