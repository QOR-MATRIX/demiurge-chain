# ADR-073: How QOR ID signs people in to other apps, and how ARQADE holds that sign-in

**Status:** Accepted, 5 October 2026, under the owner's delegation for engineering choices. It carries out
[ADR-043](ADR-043-qor-id-as-an-identity-provider.md) (accepted 4 October 2026) and
[ADR-069](ADR-069-arqade-the-gaming-platform.md) decision 4, and settles what ADR-043 left open: token lifetimes,
where the sign-in page is served, and how an app's server verifies a token. It decides no economic value.

## Context

ADR-043 decided the shape: an app sends the person to QOR ID, they sign in there, the app receives a single-use code
bound with PKCE, a browser never holds a 30-day refresh token, a reused refresh token ends the session, and sessions
record which app created them. ADR-069 added that ARQADE's server must learn who a person is without QOR ID's signing
secret (QOR ID's tokens are HS256 with a shared secret, `session_service.rs`). Nothing of it existed: no `/authorize`,
no client registry, no PKCE, no way to verify a token from outside. QOR ID's sessions live in Redis, not Postgres.

## Decision

1. **Five endpoints on QOR ID's own origin** (`services/qor-auth/src/handlers/oauth.rs`):
   `GET /oauth/authorize` (QOR ID's sign-in page), `POST /oauth/authorize` (the password, then a code back to the app),
   `POST /oauth/token` (`authorization_code` and `refresh_token` grants), `GET /oauth/userinfo`, `POST /oauth/revoke`.
   OAuth 2.1 authorization code with **S256 PKCE always required**, public clients included. No new protocol.
2. **The client registry is configuration**, one JSON array in `QOR_OAUTH_CLIENTS`: id, name, exact redirect URIs, and
   for an app with a server the **SHA-256 of its secret, never the secret**. Every client is first-party today; a table
   and an admin surface wait for a second one. Redirect URIs must be `https`, or `http` on loopback, with no fragment, and
   are compared exactly. The service refuses to start on a malformed registry.
3. **The password is checked by the same function as `/api/v1/auth/login`** (`authenticate_password`): identical
   refusals, lockouts, inactive accounts refused. A code is sent to the app only after that check.
4. **Nothing is sent to an unregistered app or address.** An unknown client or an unregistered redirect URI is answered
   on QOR ID's page; once both are known, malformed requests go back to the app as `error=invalid_request` with its state.
5. **Lifetimes** (ADR-043 named no number): an app's **session lives 8 hours**, refresh tokens included, after which the
   person signs in again; a **code lives 60 seconds**; the **sign-in page 10 minutes**. Access tokens keep QOR ID's 15
   minutes. Each is configuration under `oauth.*`.
6. **Refresh tokens rotate and are single-use.** A session stores the id (`jti`) of the one refresh token that may come
   next; each refresh replaces it. Presenting any other ends the session at once. `/api/v1/auth/refresh` refuses an app's
   refresh token, so rotation cannot be bypassed.
7. **Sessions record their app** (`client_id`) and tokens carry it (`cid`). Stored in Redis, with defaults, so sessions
   and tokens issued before this decode unchanged; no migration.
8. **An app's server verifies a person with `GET /oauth/userinfo`**, which QOR ID answers against a live session: the
   app learns `sub`, the QOR ID, the username and the chain account (SS58, or none until a key is proven, ADR-017), and
   a revoked session answers 401 at once. Asymmetric signing was not chosen: it would let a token outlive its revocation
   until expiry. No email address is returned.
9. **The sign-in page's `form-action` names the app's origin.** Browsers apply `form-action` to the redirect after a
   submission, so `'self'` alone would block the return. Only the one registered origin is added.
10. **Login CSRF is the app's to stop, by `state`.** ARQADE sets the state in an HttpOnly cookie scoped to `/api/auth`
    and refuses a callback whose state does not match it.
11. **ARQADE is a confidential client with a server-side session** (`products/arqade/lib/qor-session.ts`): it exchanges
    the code with PKCE and its secret, keeps both tokens in its own database, and gives the browser one cookie (HttpOnly,
    `SameSite=Lax`, `Secure` on https) naming the session. The row's key is the **SHA-256 of the cookie**, so the
    database alone cannot be replayed as a cookie. Each check asks `/oauth/userinfo`; an expired access token is
    refreshed once. Sign-out is same-origin only and revokes at QOR ID.
12. **An arcade player is bound to a QOR identity only on proof of both** in one request: the host's sign-in and QOR
    ID's. One QOR identity binds one player; a collision is refused and said.

## Consequences

- QOR ID gains 7 tests (143 in all, with Postgres 16 and Redis 7.4) and its log check drives the whole flow; four
  planted faults (PKCE skipped, reuse detection skipped, any redirect allowed, the JSON refresh accepting an app's token)
  and a planted log of the code each failed a test. ARQADE gains 9 tests; two planted faults were caught. The flow was
  driven end to end in Chrome against a local QOR ID and the local ARQADE worker.
- **Going live is three steps the owner takes**: merging (which deploys QOR ID on Railway), registering ARQADE in
  Railway's `QOR_OAUTH_CLIENTS` with its production callback and the hash of a new secret, and setting
  `QOR_CLIENT_ID`, `QOR_CLIENT_SECRET` and `QOR_REDIRECT_URI` in the site's host environment before redeploying it.
- Substrate gap G-8 (no per-client identity) closes.
- A second app, or a third-party one, needs its own decision before it is registered (ADR-043 excludes third parties).
