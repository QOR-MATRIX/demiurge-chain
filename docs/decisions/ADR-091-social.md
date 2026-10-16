# ADR-091: Social is a service of its own, launcher first, and every QOR ID chooses its name

**Status:** **Accepted**, 15 October 2026, by the owner, who chose the recommended option in each of four questions:
"New service", "Server-held, E2E DMs next", "Launcher first" and "One-time claim". The owner directed the work the same
day: "an engaging and full featured social networking / high quality next gen chatting / socialization system with
complete personalized profiles per QOR ID (username) and customizable aspects never before seen in a chatting
environment … premium and … entirely seamless in onboarding". **Amends** [ADR-075](ADR-075-one-name-per-qor-id.md) for
one case (decision 5), and **brings L7.3 forward** of M8 (decision 6). It decides no economic value.

**Relates to:** [ADR-073](ADR-073-qor-id-sign-in-for-apps.md) (sign-in for apps), [ADR-074](ADR-074-arqade-on-vercel.md)
(ARQADE's chat), [ADR-078](ADR-078-levels-tasks-and-the-welcome-grant.md) and [ADR-079](ADR-079-avatars-in-qor-id.md) (level and avatar, which
Social shows), [ADR-080](ADR-080-effects-are-allowed.md) (effects), [ADR-056](ADR-056-no-lock-screen.md) (nothing may block
on QOR ID).

## Context

Social (named Agora until 28 September 2026, with VYB merged into it the same day) is the launcher's placeholder
surface L7.3: "Rooms, messages, presence, voice, profiles and feeds, under the same identity you play under". Nothing
of it exists. What does, checked on 15 October 2026:

- **QOR ID** (`services/qor-auth/`) holds one account per username (ADR-075), an avatar (ADR-079), XP and a level
  (ADR-078). It has no display name, no public profile of another person, no realtime transport, and no route that
  changes a username: `POST /api/v1/profile` answers 501. Its access tokens are HS256 JWTs whose secret only it holds;
  nothing outside it can check one except by asking it (`/oauth/userinfo`, for apps' own tokens).
- **Accounts made by key sign-in without a name** get `key_` and four bytes of the account in hex
  (`handlers/auth.rs`). The launcher has asked for a name since onboarding was built, so such names come from older
  clients and direct calls, and nothing lets their holders choose a real one.
- **ARQADE's chat** is one global stream in Postgres, read by polling every 2 seconds, with per-message delete, a
  report row and a mute list in the browser. No bans, triage or retention (`docs/blueprints/arqade.md`).
- **Nothing active pushes in realtime.** The launcher's webview may connect only to its own origins (its CSP); every
  call to QOR ID goes through the launcher's Rust host, which holds the tokens in the OS keychain.
- DIRECTION places all of L7 after M8 (the Mesh, entitlements, staking, escrow). Nothing in M8 is needed by rooms,
  messages, presence or profiles.

## Decision

1. **Social is a service of its own: `services/social/`**, in Rust on the same stack as QOR ID (axum, sqlx, Postgres),
   with its own database, deployed as its own Railway service. It holds rooms, memberships, messages, reactions, read
   positions, presence, blocks, reports and moderation, and each person's Social profile. It joins active scope
   (AGENTS.md §4). Names (approved with the option): the directory `services/social/` and the service "Social"; the
   crate is `social`.
2. **Identity stays in QOR ID.** Social never stores a password, key or QOR ID token, and never signs anything. A
   client presents its QOR ID access token once per connection; Social asks QOR ID whether it is live, with a
   credential of Social's own, through a new **token introspection** endpoint in QOR ID (`POST /oauth/introspect`,
   RFC 7662's shape, open only to registered services), and caches the answer briefly. A person is keyed by QOR ID's
   `sub`; their username, avatar and level are QOR ID's, shown as QOR ID returns them, never copied into a second
   identity.
3. **Realtime over one WebSocket per client**, typed JSON frames, versioned like the QOR host protocol (ADR-046): the
   client sends typed intents (send, edit, react, read, type); the service pushes events. In the launcher the socket is
   held by the Rust host, which already holds the token, and its events reach the interface as Tauri events; the
   webview's CSP is not widened.
4. **Privacy, release 1: server-held, encrypted in transit and at rest.** Message bodies are sealed with an AEAD key
   held by the service (keyed by version, so it can be rotated) before they reach the database, and travel only over
   TLS. The service can read them, so reports, moderation, search and history on every device work. **Release 2:
   end-to-end encrypted direct messages**, keyed from the vault, which the service cannot read; moderation of those
   rests on what a recipient reports. Release 2 is designed in its own record before it is built.
5. **Every QOR ID has a chosen name.** An account still on an automatic `key_…` name chooses its username **once**,
   under ADR-075's rules, the first time a name is shown to it (entering Social, or the launcher's account bubble);
   after that the name is fixed as ADR-075 has it. QOR ID records which names were chosen, and a claim of a taken name,
   or a second claim, is refused. This amends ADR-075, which built no way to choose a new name, for this one case.
   A person with no QOR ID at all is taken through the launcher's existing claim, which asks for the name first.
6. **Launcher first.** The launcher's Social surface becomes the first client, in its design system (themes, QFX,
   avatar ring and level). A web client on the same service follows. L7.3 is brought forward of M8 in DIRECTION:
   nothing it needs is in M8. ARQADE's chat keeps working as it is until ARQADE moves onto Social, which is a later
   item of its own.
7. **What release 1 holds** (the owner's direction, scoped to what can be built and tested): public rooms and private
   rooms by invitation, with owner and moderator roles; direct messages; replies, reactions, edits and deletes;
   typing and presence (online, away, a status line, and what someone is playing, when a product says so); unread
   counts; mentions; blocking; reports to the room's moderators and the service's operators; and **profiles per QOR
   ID** that a person shapes: a display line, a bio, an accent from their theme, a banner from the QFX backdrops, a
   showcase of DRC-369 assets they hold, and a card that others open from any message. Voice, feeds and search
   across rooms are later items.

## Consequences

- **Owner's decisions still open:** how long messages are kept (a retention rule, before release 1 goes live), and a
  content policy that reports are judged against. Until they are decided, release 1 runs only on the devnet stack,
  and nothing is deleted on a timer.
- QOR ID gains the introspection endpoint, the one-time claim and a public profile read; each joins
  `nothing_secret_reaches_a_log_at_any_level` in the same change (AGENTS.md §9), and every query is bound
  (`sql_hygiene.rs`). Social adopts both checks from its first commit.
- A second service doubles what is deployed and monitored. Monthly cost is estimated, not measured, at $5 to $15.
- The tokens Social receives are QOR ID's first-party access tokens, scoped as every QOR ID session is today (QOR ID
  checks no scope). A Social-specific scope is a later tightening.
- Nothing here touches CGT, the chain or DRC-369 state; the showcase only reads what an account holds.
