# Social

Rooms, direct messages, presence and profiles per QOR ID, in a service of its own (ADR-091); World, friends and
circles, and the language of every message (ADR-092). The design is
[`docs/blueprints/social.md`](../../docs/blueprints/social.md); what exists is in
[`docs/SYSTEMS.md`](../../docs/SYSTEMS.md). **Live since 16 October 2026** beside QOR ID on Railway
(`DEPLOY-RAILWAY.md`); its first client is the launcher's Social surface.

## What it is

- **One WebSocket per client** at `/v1/socket` (`src/socket.rs`). The first frame is `hello` with a version and the
  person's QOR ID access token; Social asks QOR ID's `POST /oauth/introspect` whether it is live, under its own
  credential (`src/identity.rs`), and answers `welcome` or closes. Then typed requests (`src/api.rs`) and pushed events
  (`src/hub.rs`). A socket closes with `token_expired` when its token does.
- **Postgres of its own** (`migrations/`, run at start). Message bodies are sealed at rest with XChaCha20-Poly1305
  under versioned keys from the environment, bound to their room and sequence (`src/seal.rs`). The service can read
  them (release 1); end-to-end encrypted direct messages are release 2.
- **Every rule is checked in the store** (`src/store.rs`): membership, invitations, roles, direct rooms of two, blocks,
  ownership of edits and deletes, limits per person across sockets, NFKC-cleaned text (`src/text.rs`).
- **World** (migration 002): one room everyone joins at hello, starting at its newest message; it makes nobody a
  contact; its moderators come from `SOCIAL_WORLD_MODERATORS` and delete and mute (`world.mute`); its own sending pace.
- **Friends and circles**: requests answered by the one asked, friends seeing each other's presence, a block ending
  both; circles private to their owner, opening a private room with `circles.open`.
- **Languages**: a message keeps the `lang` its sender's launcher found, and a profile the `language` its person
  reads; Social checks only their shape. Translation itself happens on each reader's PC, never here.
- **Nothing secret reaches a log**: `may_log` in `src/main.rs` holds the WebSocket library to info and above, since it
  prints frames with their payloads at debug and trace; `src/log_hygiene.rs` drives every flow at trace through that
  filter and fails on a token, the client secret, a key, words, a bio, a status or a note. Every statement is a literal
  with bound parameters (`src/sql_hygiene.rs`); migrations are LF on every platform (`src/migration_hygiene.rs`).

## Configuration

| Variable | What |
| --- | --- |
| `DATABASE_URL` | Social's own Postgres |
| `SOCIAL_QOR_ID_URL` | QOR ID's origin; `https://id.qorsync.dev` unless set |
| `SOCIAL_CLIENT_ID`, `SOCIAL_CLIENT_SECRET` | Social's credential with QOR ID, registered there as a service (`"introspect": true`, a `secret_sha256`, no redirect URI; `services/qor-auth/DEPLOY-RAILWAY.md`) |
| `SOCIAL_MESSAGE_KEYS` | `1:<base64 of 32 random bytes>`; add `2:…` to rotate, keeping 1 to read what it sealed |
| `SOCIAL_WORLD_MODERATORS` | World's moderators, QOR ID usernames separated by commas; none unless set. Read at each hello, so a change applies when the person next connects |
| `PORT` | 8090 unless set |
| `RUST_LOG` | `social=info` unless set; the WebSocket library never logs below info whatever it says |

## Deploying

`Dockerfile` builds the service (context `services/social`); `GET /ready` asks the database and is what a health check
points at (`/health` answers 200 whatever the database's state). `DEPLOY-RAILWAY.md` is the plan for Railway (L8.6).

## Build and test

```bash
cd services/social
DATABASE_URL=postgres://ci:ci@127.0.0.1:55432/postgres cargo test
cargo fmt --all -- --check && cargo clippy --all-targets --locked -- -D warnings
```

The tests need a Postgres whose user can create databases (`#[sqlx::test]` makes one per test). On this PC:
`docker run -d --rm --name social-dev-pg -e POSTGRES_USER=ci -e POSTGRES_PASSWORD=ci -p 127.0.0.1:55432:5432
public.ecr.aws/docker/library/postgres:16-alpine`. They drive a real WebSocket on a loopback port; QOR ID is stood in
for, except in the log check, which runs the real QOR ID client against a stand-in QOR ID over HTTP. They run four at a
time (`.cargo/config.toml`): sqlx gives all of a run's tests one parent pool of 20 connections, and more at once wait on
one another until some time out.

**Dependency audit:** CI audits Social, ignoring only RUSTSEC-2023-0071 (`rsa`), which is in `Cargo.lock` through
sqlx's optional MySQL driver and not compiled, exactly as in QOR ID (`.cargo/audit.toml`, approved by the owner on
15 October 2026). A CI step fails if `cargo tree --locked -i rsa --target all -e all` ever prints anything here.
