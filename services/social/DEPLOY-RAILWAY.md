# Social on Railway (L8.6)

**Not deployed** (15 October 2026). This is the plan for deploying Social beside QOR ID in Railway project
**`demiurge`** (ADR-063, ADR-091), and becomes the record of how it is set up once it is. Railway has deprecated
`railway.json`, so every setting lives on the service and is written down here. Creating services and setting variables
are the owner's steps in the Railway dashboard.

## Services

| Service | Source | Notes |
| --- | --- | --- |
| `Social-Postgres` | Railway's `postgres` template | Social's own database (ADR-091 decision 1), not QOR ID's. Volume at `/var/lib/postgresql/data` |
| `social` | `QOR-MATRIX/demiurge-chain`, branch `main`, root `/services/social` | Built from `Dockerfile` |

## `social` settings

Root directory `/services/social`; Dockerfile `Dockerfile`; healthcheck **`/ready`** (it asks the database; `/health`
answers 200 whatever its state), timeout 120 s; restart on failure, 3 retries; redeploys only when
`/services/social/**` changes. It reads Railway's `PORT`. Migrations run at startup, from inside the binary.
A public domain with WebSockets: Railway serves `wss://` on its generated domain; a custom one (for example
`social.qorsync.dev`, a CNAME in Cloudflare like QOR ID's) is the owner's choice.

## `social` variables

| Variable | Value |
| --- | --- |
| `DATABASE_URL` | `${{Social-Postgres.DATABASE_URL}}` |
| `SOCIAL_QOR_ID_URL` | `https://id.qorsync.dev` |
| `SOCIAL_CLIENT_ID` | `social` |
| `SOCIAL_CLIENT_SECRET` | **Secret.** Generated below; only its SHA-256 goes to QOR ID |
| `SOCIAL_MESSAGE_KEYS` | **Secret.** `1:<base64 of 32 random bytes>`, generated below. Losing it loses every message sealed under it |
| `RUST_LOG` | `social=info` |

## Generating the two secrets, on the owner's machine

```bash
node -e "const c=require('crypto');const s=c.randomBytes(32).toString('base64url');console.log('SOCIAL_CLIENT_SECRET='+s);console.log('secret_sha256='+c.createHash('sha256').update(s).digest('hex'));console.log('SOCIAL_MESSAGE_KEYS=1:'+c.randomBytes(32).toString('base64'))"
```

Paste `SOCIAL_CLIENT_SECRET` and `SOCIAL_MESSAGE_KEYS` into the `social` service's variables, and nowhere else. Keep a
copy of the message key where the owner keeps the other credentials: without it, nothing sealed under it can be read.

## Registering Social with QOR ID

In the `qor-auth` service's `QOR_OAUTH_CLIENTS` (a JSON array; `services/qor-auth/DEPLOY-RAILWAY.md`), add a service
entry beside ARQADE's, with the `secret_sha256` printed above:

```json
{ "id": "social", "name": "Social", "redirect_uris": [], "secret_sha256": "<64 hex characters>", "introspect": true }
```

QOR ID refuses to start on a registry it cannot accept (a service with a redirect URI, or without a secret), so a
mistake shows at once rather than later.

## Pointing the launcher at it

The launcher's `social_url` defaults to `ws://127.0.0.1:8090/v1/socket` until Social is deployed. Once it is, the
default becomes its `wss://` address in a launcher change of its own; until a build carries that, a launcher reaches it
with `QOR_SOCIAL_URL=wss://<its domain>/v1/socket`.

## Before anyone else uses it (L8.7)

How long messages are kept, and the content policy reports are judged against, are the owner's decisions; until both
are made, Social is for the owner's own use on the devnet stack.
