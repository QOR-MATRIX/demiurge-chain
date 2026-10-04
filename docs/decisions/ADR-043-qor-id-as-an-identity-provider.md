# ADR-043: A browser frontend signs in to QOR ID by redirect, not by holding a refresh token

**Status:** **Accepted**, 4 October 2026, by the project owner ("I accept ADR-069 and ADR-043 as written"),
with [ADR-069](ADR-069-arqade-the-gaming-platform.md): ARQADE is the browser frontend decision 6 waited for, so
items 1 to 3 may now start. Proposed 20 September 2026. **Item 4 was done before acceptance**: QOR ID answers
cross-origin requests only from `allowed_origins` (`services/qor-auth/src/main.rs:201-230`). Nothing else is built.
The assessment behind it is [`../architecture/HOSTING.md`](../architecture/HOSTING.md) §3.

**Relates to:** [ADR-042](ADR-042-two-domains.md), which puts frontends and QOR ID on different domains;
[ADR-016](ADR-016-sign-in-with-unlock.md), which is about the launcher and is not changed by this;
[ADR-011](ADR-011-web-surface.md), which is what creates the need.

## Context

ADR-011 adds two browser surfaces: a public viewer and a thin remote console. Under ADR-042 they are on
`demiurge.cloud` and QOR ID is on `qorsync.dev`. The question is how a page on one domain authenticates
against a service on the other.

**What the code actually does**, read on 20 September 2026:

- QOR ID issues a JWT **access token (15 minutes)** and a **refresh token (30 days)** in the response
  body, and authenticates every request from an `Authorization: Bearer …` header
  (`middleware/auth.rs:76`, `handlers/auth.rs:311`, `config.rs:92-93`).
- **There is no cookie anywhere in the flow.** So the usual cross-domain problem — a cookie that will not
  cross a registrable domain boundary — does not arise. A page on `demiurge.cloud` can call QOR ID
  cross-origin with a bearer token today.
- **CORS allows every origin**: `CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`
  (`main.rs:195-199`).
- **The launcher keeps both tokens in the OS keychain**, never on disk and never in the webview
  (`identity/mod.rs:19`).

So the problem is not that a browser frontend *cannot* authenticate. It is that the only ways for it to
**keep** a token are bad ones. `localStorage` exposes a 30-day refresh token to any XSS on the frontend's
origin — the launcher escapes this by having a keychain, and a web page has none. Memory-only storage
means signing in again on every reload.

There is a second problem the domain split makes visible rather than creates: **QOR ID has no concept of
a client application.** A session belongs to an account and records nothing about what created it, so
"revoke the console's access" cannot be expressed, and a token stolen from one frontend is valid against
every endpoint, including the launcher's.

## Decision

1. **A browser frontend authenticates by redirect, not by holding long-lived credentials.** The frontend
   sends the browser to QOR ID; the person signs in **on QOR ID's own origin**; QOR ID redirects back
   with a single-use authorization code; the frontend exchanges that code for tokens. **OAuth 2.1
   authorization code with PKCE**, which is the boring, specified choice ADR-001 asks for in a
   security-critical path. No new protocol is invented.

2. **A browser frontend never receives a 30-day refresh token.** If it receives a refresh token at all,
   that token is short-lived, **rotated on every use**, and **invalidated for the whole session if an
   already-used one is presented again**, which is what makes theft detectable.

3. **Sessions gain a client identity.** Each frontend is a registered client with an id and an allowlist
   of redirect URIs, and a session records which client created it — so access can be revoked per client
   instead of per account.

4. **CORS stops allowing every origin.** QOR ID answers cross-origin requests only from an explicit
   allowlist of frontend origins. **This part is separable and should be done first:** it is a
   tightening, it needs no domain decision and no redirect flow, and it is a small change.

5. **The launcher is out of scope.** It is a native client with a keychain, not a browser. It keeps the
   flow it has, and ADR-016 is untouched.

6. **None of items 1 to 3 is started until a browser frontend actually needs it**, which is **M5.4 at the
   earliest.** Building an authorization server before there is a client to use it means shipping
   security-critical code that nothing exercises.

## Consequences

- **This is a milestone's worth of work in QOR ID, not a configuration change.** An `/authorize`
  endpoint, a sign-in page served by QOR ID, a client registry, a redirect-URI allowlist, single-use
  PKCE-bound authorization codes, and a rotating refresh-token model — none of which exists today.
  Pretending otherwise is how this gets rushed later.
- **Item 4 can land long before the rest**, and should. It closes a real gap: today any website can make
  cross-origin calls to QOR ID from a visitor's browser.
- **Item 3 changes the sessions table and the session model.** It is compatible with the two gaps already
  named in `SECURITY.md` — sessions record no IP address and no last-activity time — and whoever does
  this work should close those at the same time, since it is the same table and the same surfaces.
- **Revocation becomes finer**, which is a user-visible improvement: "sign out everywhere" can become
  "sign out of the console", and a compromised frontend can be cut off without ending every session.
- **A specification, not a library, is the commitment.** OAuth 2.1 with PKCE is implementable against the
  RFCs; whether to adopt a crate or write the endpoints is not decided here, and ADR-033 governs the
  choice if a dependency is added.
- **If this is rejected**, the fallback is a browser frontend holding tokens in `localStorage`, and the
  consequence should be stated plainly rather than discovered: an XSS on `demiurge.cloud` becomes account
  takeover on QOR ID.

## What this record does not decide

- The token lifetimes for browser clients. It says short and rotated; it names no number.
- Whether QOR ID becomes a general OIDC provider for third parties. This covers first-party frontends
  only. Third-party sign-in is a much larger commitment and needs its own record.
- Where the sign-in page is served from, which is ADR-042 decision 3 and is contested there.
