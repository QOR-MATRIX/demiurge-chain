# Running the stack locally

Everything the QOR Launcher needs, on one machine, plus a multi-validator devnet. Verified on
Windows 11 with Docker Desktop, Rust 1.98 and Node 24 (2026-09-13).

What the chain does is in [`docs/protocol/PROTOCOL.md`](../docs/protocol/PROTOCOL.md), and how to build and
test it is in [`chain/README.md`](../chain/README.md).

## 1. Postgres and Redis

Any Postgres 16 and Redis 7 will do. If you already have a stack running on the default ports (the
`vyb-social` compose project uses both), reuse it rather than fighting for the ports, and isolate
this service inside it:

```bash
# A separate role and database. Nothing existing is touched.
docker exec <postgres-container> psql -U <admin> -d postgres \
  -c "CREATE ROLE qor_auth LOGIN PASSWORD 'password';"
docker exec <postgres-container> psql -U <admin> -d postgres \
  -c "CREATE DATABASE qor_auth OWNER qor_auth;"
```

For Redis, point the auth service at an unused database index (`/3` below) so its keys cannot
collide with anything already in db 0.

Otherwise, start your own:

```bash
docker run -d --name qor-postgres -p 5432:5432 \
  -e POSTGRES_USER=qor_auth -e POSTGRES_PASSWORD=password -e POSTGRES_DB=qor_auth \
  postgres:16-alpine
docker run -d --name qor-redis -p 6379:6379 redis:7-alpine
```

## 2. The chain: a development node

> **There is one chain.** The custom Rust devnet in `framework/` was retired at M3.5 and is gone. If
> you have muscle memory for `demiurge-node-legacy`, `--faucet`, `--genesis` or `chain_getBlockNumber`,
> none of it applies any more: the chain is Substrate, and it speaks standard Substrate RPC
> ([`chain/README.md`](../chain/README.md)).

```bash
cd chain
cargo build --release --features sudo
./target/release/demiurge-node --dev --tmp
```

- `--dev` is a single-validator chain: Alice authors with Aura and finalises with GRANDPA, and the
  well-known development accounts are endowed in the chain specification.
- `--tmp` throws the database away on exit, which is what you want while iterating. Drop it, and pass
  `--base-path`, to keep a chain between runs.
- `--features sudo` builds `pallet-sudo` in. It is **development and test networks only** (ADR-037), and
  the mainnet shape is the same command without it.
- The RPC listens on `127.0.0.1:9944`, and the launcher points at `ws://127.0.0.1:9944`. The endpoint is
  a **WebSocket** address (ADR-040); `http://` is not a chain endpoint any more.

Check which chain answered, which is the one question worth asking of a node:

```bash
curl -s -X POST http://127.0.0.1:9944 -H 'Content-Type: application/json'   -d '{"jsonrpc":"2.0","id":1,"method":"system_chain","params":[]}'
# {"jsonrpc":"2.0","id":1,"result":"Demiurge Development"}
```

**There is no faucet and no mint.** The chain has no issuance mechanism: the issuance rate is OPEN-1 and
the genesis split is OPEN-2, and nothing may create CGT until they are decided (AGENTS.md §5). To fund an
account locally, send to it from one of the endowed development accounts — which is what the launcher's
live test does, from Alice.

## 3. The chain: two validators

Two nodes, agreeing and finalising, with one killed and restarted to prove catch-up. This is a script
rather than a cargo test, because it starts real nodes; it takes about two minutes.

```bash
cd chain
cargo build --release --features sudo
node scripts/check-two-validators.mjs
```

It reports 13 checks: both validators produce and agree, GRANDPA finalises, a killed validator's peer
carries on alone, and the restarted one catches up and agrees about the blocks it missed. Release gate
criterion `alpha.multi-validator` reads its result.

For a longer-lived local network, `--chain local` is the two-validator specification (Alice and Bob),
and each node runs with its own `--base-path`, `--port` and `--rpc-port`. The script is the worked
example.

## 4. The identity service

`sqlx` verifies its queries against a live database **at compile time**, so `DATABASE_URL` must be set
for the build, not just at runtime:

```bash
cd services/qor-auth
export DATABASE_URL="postgres://qor_auth:password@localhost:5432/qor_auth"
cargo build --release
```

The service applies its own migrations at startup, so do not apply them by hand first; it will fail
on the duplicate types. If you already did, drop and recreate the database and let the service own it.

```bash
export QOR_AUTH__SERVER__HOST=127.0.0.1
export QOR_AUTH__SERVER__PORT=8080
export QOR_AUTH__DATABASE__URL="postgres://qor_auth:password@localhost:5432/qor_auth"
export QOR_AUTH__DATABASE__MAX_CONNECTIONS=10
export QOR_AUTH__REDIS__URL="redis://localhost:6379/3"
export QOR_AUTH__JWT__ACCESS_SECRET="local-dev-access-secret-not-for-production-use"
export QOR_AUTH__JWT__REFRESH_SECRET="local-dev-refresh-secret-not-for-production-use"
export QOR_AUTH__JWT__ACCESS_EXPIRY_SECS=900
export QOR_AUTH__JWT__REFRESH_EXPIRY_SECS=2592000
export QOR_AUTH__JWT__ISSUER=qor-auth
export QOR_AUTH__SECURITY__MAX_LOGIN_ATTEMPTS=5
export QOR_AUTH__SECURITY__LOCKOUT_DURATION_SECS=900
export QOR_AUTH__SECURITY__PASSWORD_MIN_LENGTH=12
export RUN_ENV=development RUST_LOG=info

./target/release/qor-auth
```

If Redis needs a password, put it in the URL: `redis://:<password>@localhost:6379/3`.

**"migration 1 was previously applied but has been modified"** at startup, on a database that was
migrated on Windows before 1 October 2026, is a line-ending fault and not a modified migration. sqlx
records the SHA-384 of each migration file, and until `.gitattributes` pinned
`services/qor-auth/migrations/*.sql` to LF, a Windows clone held those files as CRLF, so such a
database holds CRLF checksums that no build produces any more. Correct it once, with the database
running and nothing else changed:

```powershell
$env:DATABASE_URL = 'postgres://<user>:<password>@127.0.0.1:<port>/qor_auth'
powershell -File services/qor-auth/scripts/correct-migration-checksums.ps1 -DryRun   # report only
powershell -File services/qor-auth/scripts/correct-migration-checksums.ps1
```

It rewrites a row only where it holds the CRLF checksum of the same file, leaves a row that already
holds the LF checksum alone (so a second run changes nothing), and leaves alone and reports a row that
holds anything else, because that migration really was modified. It runs no migration, never prints the
URL, uses `psql` if it is on PATH and Docker's otherwise, and refuses a host other than this machine
unless told `-AllowRemoteHost`. Then rebuild the service (`cargo build --release`): a binary built
before the change still carries the CRLF files and is refused by the corrected database.
`tools/qor-launcher/scripts/start-local.ps1` builds only when the binary is missing, so it will not do
that for you. The launcher's own database (`qor-local-pg`) was corrected and the binary rebuilt on
1 October 2026.

Check it:

```bash
curl -s http://127.0.0.1:8080/health
# {"status":"healthy","service":"qor-auth","version":"…","email_leaves_this_machine":false}
```

`email_leaves_this_machine` must be `false` before any end-to-end script is pointed at the service; see
**Email** under Notes. On Windows, `tools/qor-launcher/scripts/start-local.ps1` does all of this section
(Postgres, Redis, QOR ID on 8080 and a development chain) and starts QOR ID with email off:

```powershell
powershell -File tools/qor-launcher/scripts/start-local.ps1 -NoChain                 # email off
powershell -File tools/qor-launcher/scripts/start-local.ps1 -NoChain -EmailStandIn   # email to a stand-in on this machine
node services/qor-auth/scripts/e2e/sessions.mjs http://127.0.0.1:8080
```

Sessions are held in Redis, not Postgres, and `SessionService::record_use` needs **Redis 6 or later**
(`SET … XX KEEPTTL`).

## 5. The launcher

```bash
cd tools/qor-launcher
npm install
QOR_RPC_URL=http://127.0.0.1:9944 \
QOR_AUTH_URL=http://127.0.0.1:8080/api/v1 \
npm run app:dev
```

Those two environment variables exist because the Gate runs *before* the shell: until you have signed
in you cannot reach Settings, so without an override a fresh install has no way to be pointed at a
local stack in order to sign in.

**Projects (Qontrol) commits through a helper that is not bundled yet.** Build it once before starting
the launcher, or the Projects surface opens and reads repositories but says committing is off:

```bash
cargo build --manifest-path tools/qor-launcher/qontrol-git/Cargo.toml
```

## Notes

- **New accounts have no CGT, and there is no way to create any.** The identity service creates no CGT
  and calls no chain (migration inventory R-3); `register` no longer has a `starter_cgt_minted` field. The
  launcher's starter claim refuses, and says why: the chain has no issuance mechanism, because OPEN-1 and
  OPEN-2 are undecided. On a `--dev` node, fund an account by transferring to it from a development
  account endowed in the chain specification.
- **Email** goes through Resend. Set `RESEND_API_KEY`, `EMAIL_FROM` (a sender on a domain verified in
  Resend) and `BASE_URL` (where the links point). Without the first two, `forgot-password` answers 503 and
  registration sends no verification email. No email content is ever logged.
  **`cargo test` never sends mail, whatever the environment holds**: no test reads these variables, and a
  test build refuses any API base that is not on this machine. A *running* service does read them, from
  the environment it inherits and from a `.env` file beside it. Three things keep a local one from
  sending real mail by accident:
  - **`tools/qor-launcher/scripts/start-local.ps1` starts QOR ID with email off.** It takes
    `RESEND_API_KEY`, `EMAIL_FROM`, `BASE_URL` and `RESEND_WEBHOOK_SECRET` away from the process it starts
    and sets `RESEND_API_URL` to `http://127.0.0.1:59925`, so a key that arrives some other way still has
    nowhere to send but this machine. `-EmailStandIn` turns email on, to that address only, with the
    placeholder key and sender `scripts/e2e/email-via-resend.mjs` expects and links pointing at
    `http://127.0.0.1:8080`. **Real mail has to be asked for by name: `-SendRealEmail`**, which lets the
    process inherit the environment's settings. The script then asks the running service, and stops it if
    it would send real mail that was not asked for.
  - **`GET /health` says `email_leaves_this_machine`.** It is `true` only for a service that sends through
    Resend itself, and `false` when email is off or goes to a stand-in on the same machine. It says nothing
    else about configuration.
  - **Every end-to-end script (`services/qor-auth/scripts/e2e/`) reads that first and refuses to run**
    unless it is exactly `false` (`_guard.mjs`): it registers `@example.invalid` addresses, which sent for
    real bounce and count against the sending domain. A service that does not say, such as a binary built
    before 1 October 2026, is refused too. There is no switch to run anyway. Rebuild
    (`cargo build --release`) and start it as above.

  Starting the service by hand (§4) inherits whatever the shell holds. Unset `RESEND_API_KEY`, or set
  `RESEND_API_URL=http://127.0.0.1:59925`, first; the scripts refuse otherwise.
- **Admin routes** need an account whose role is `god`. Locally, set it in the database
  (`UPDATE users SET role = 'god' WHERE username = '...'`) and sign in again.
- **After moving the repository**, the launcher's build cache holds absolute paths to the old location
  and `cargo test` fails inside `tauri-build` with "failed to read plugin permissions". Run
  `cargo clean -p tauri` in `tools/qor-launcher/src-tauri` once.
- **Measuring the window from outside.** On a display scaled above 100%, a DPI-unaware process reading
  `GetWindowRect` gets *virtualized* coordinates. This launcher's 1800x1125 window is reported as
  1454x882, and a screenshot taken at those coordinates captures only the left 81% of the window. That
  looks exactly like a layout bug and is not one. Call
  `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)` in the measuring process before any geometry call.
- **The music module is disabled** in `services/qor-auth`. It does not compile and never has; see the
  note in `src/handlers/mod.rs`.
