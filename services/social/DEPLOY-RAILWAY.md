# Social on Railway (L8.6)

**Live since 16 October 2026** at `wss://social-production-e05c.up.railway.app/v1/socket`, beside QOR ID in Railway
project **`demiurge`**, environment `production`, region `iad` (ADR-063, ADR-091). This is the record of how it is set
up. Railway has deprecated `railway.json`, so every setting lives on the service and is written down here.

## Services

| Service | Source | Notes |
| --- | --- | --- |
| `Postgres-Yj3Q` | Railway's `postgres` template | Social's own database (ADR-091 decision 1), not QOR ID's. Railway named it; `social` refers to it by this name, so **renaming it breaks `DATABASE_URL`** |
| `social` | `QOR-MATRIX/demiurge-chain`, branch `main`, root `/services/social` | Built from `Dockerfile` |

## `social` settings

Root directory `/services/social`; Dockerfile `Dockerfile`; healthcheck **`/ready`** (it asks the database; `/health`
answers 200 whatever its state), timeout 120 s; restart on failure, 3 retries; redeploys only when
`/services/social/**` changes. `PORT` is pinned to **8090**, the port its domain `social-production-e05c.up.railway.app`
routes to. Migrations run at startup, from inside the binary. A custom domain (for example `social.qorsync.dev`, a CNAME
in Cloudflare like QOR ID's) is the owner's choice and not set.

## `social` variables

| Variable | Value |
| --- | --- |
| `DATABASE_URL` | `${{Postgres-Yj3Q.DATABASE_URL}}` |
| `SOCIAL_QOR_ID_URL` | `https://id.qorsync.dev` |
| `SOCIAL_CLIENT_ID` | `social` |
| `SOCIAL_CLIENT_SECRET` | **Secret.** Generated on the owner's machine; only its SHA-256 is registered with QOR ID |
| `SOCIAL_MESSAGE_KEYS` | **Secret.** `1:<base64 of 32 random bytes>`. Losing it loses every message sealed under it |
| `PORT` | `8090` |
| `RUST_LOG` | `social=info` |

## Generating the two secrets, on the owner's machine

```bash
node -e "const c=require('crypto');const s=c.randomBytes(32).toString('base64url');console.log('SOCIAL_CLIENT_SECRET='+s);console.log('secret_sha256='+c.createHash('sha256').update(s).digest('hex'));console.log('SOCIAL_MESSAGE_KEYS=1:'+c.randomBytes(32).toString('base64'))"
```

Paste `SOCIAL_CLIENT_SECRET` and `SOCIAL_MESSAGE_KEYS` into the `social` service's variables, and nowhere else (not a
chat). Keep a copy of the message key where the owner keeps the other credentials: without it, nothing sealed under it
can be read. **Rotating the client secret** is the same two places changed together: the new secret in `social`, its
SHA-256 in QOR ID's entry below; until both match, Social answers every hello `unavailable`. **Rotating the message key**
adds `2:<new key>` beside `1:` (never replacing it), so what version 1 sealed still opens.

## Registering Social with QOR ID

`qor-auth`'s `QOR_OAUTH_CLIENTS` (a JSON array; `services/qor-auth/DEPLOY-RAILWAY.md`) holds ARQADE's entry and
Social's, a service entry:

```json
{ "id": "social", "name": "Social", "redirect_uris": [], "secret_sha256": "<64 hex characters>", "introspect": true }
```

QOR ID refuses to start on a registry it cannot accept (a service with a redirect URI, or without a secret), so a
malformed entry shows at once. A changed variable takes effect when QOR ID next deploys.

## Checking it

`https://social-production-e05c.up.railway.app/ready` answers 200 while its database does. A hello with a token that is
not live answers `unauthorized` when Social could ask QOR ID, and `unavailable` when it could not: a client secret that
does not match QOR ID's entry shows as the second, and as a 401 from Railway's network on `/oauth/introspect` in
QOR ID's HTTP log.

## The launcher

The launcher's `social_url` defaults to this deployment (`src-tauri/src/social.rs`, `DEFAULT_SOCIAL`); a saved setting or
`QOR_SOCIAL_URL` still wins, and `ws://127.0.0.1:8090/v1/socket` reaches one running on this machine.

## Before anyone else uses it (L8.7)

How long messages are kept, and the content policy reports are judged against, are the owner's decisions; until both
are made, Social is for the owner's own use on the devnet stack.

## History

- **16 October 2026, first deployment.** The database, the service, its settings, domain and non-secret variables were
  made through Railway's connector; the owner pasted the two secrets. The first test refused Social at QOR ID: the
  registration line had not been added to `QOR_OAUTH_CLIENTS`, so it was added through the connector with the hash of
  the secret generated for it. Social then answered `unavailable`, and QOR ID's HTTP log showed its call refused (401):
  Social held a different secret, which the owner had generated separately and also pasted into a chat. It was replaced
  in `social` with the registered one, so the secret seen in the chat was never registered with QOR ID and authenticates
  nothing. The live check then answered `unauthorized`.
