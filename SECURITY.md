# Security

## Status

**Demiurge has not been audited.** It is pre-release software. Do not hold real value on it.

Known open issues are tracked as roadmap items in [`docs/DIRECTION.md`](docs/DIRECTION.md). The most
serious today:

- **Ed25519 accepts a forged signature for a small-order public key, and this is deliberate.** The
  runtime verifies Ed25519 through `sp-core`, which uses `ed25519-zebra`'s ZIP-215 rule — a precisely
  specified rule chosen so that every node agrees on validity, which consensus needs more than it needs
  strictness. `chain/runtime/tests/acceptance.rs` pins it in both directions: the forgery is refused for
  Sr25519 on 64 of 64 messages and accepted for Ed25519 on 64 of 64. **What it costs:** an account whose
  address is a small-order Ed25519 point can be spent from by anyone. Nobody holds a secret for such an
  address, so nothing is taken from a person. Account keys are Sr25519 (ADR-023), so the scheme this
  chain actually uses is not exposed. Accepted as met by standard behaviour in **ADR-038**, rather than
  owning a consensus-critical divergence permanently. A test fails if the SDK ever becomes stricter, so
  the day that changes is the day someone revisits it.
- **An access token alone can register an agent** (ADR-014, known gap). Until the controller's vault
  signs each agent authorisation (L5.1), anyone holding a controller's access token can register an
  agent for that controller. It is not a design choice, and L5.1 is a Beta gate criterion, so the public
  testnet cannot open with it in place. Still true on 6 October 2026: `POST /api/v1/agents/register` takes the
  controller from the access token and checks only the agent key's own signature.
- **A recovery phrase was pasted into a chat on 5 October 2026.** Treat that key as exposed and replace it. If it
  is the devnet's sudo key, replace it with `sudo.setKey` from the current sudo account, signed in the owner's
  wallet, then move anything the old account holds. The devnet holds nothing of value (ADR-037), but whoever has
  its sudo key can replace its runtime.
- **The wallet extension** (`apps/wallet-extension`) derives keys non-standardly. Keys created there
  cannot be recovered by any other wallet (D-011).
- **Nine credential-shaped values from the private archive's history are covered by no rotation record.**
  They were in `docker/n8n/docker-compose.yml` (a Postgres password, its duplicate, a 64-hex n8n master
  encryption key and a basic-auth password) and `docker/docker-compose.testnet.yml` (four raw libp2p node
  secret keys and a Grafana admin password). Found on 21 September 2026 by a full-history scan. **Neither file
  is in today's tree, and neither is in the public repository's history**; they exist only in the history of
  the private archive. **If those services were ever run, rotate the values.** If the n8n stack was ever started
  with that encryption key, rotating the key alone makes its stored credentials undecryptable rather than safe —
  they have to be re-entered. The list is below, under "Nine more values".

**History note, same scan (21 September 2026), about the private archive.** Apart from those nine and the
three already recorded below, the private archive's history was clean: 130 commits from a single squashed
import, with no SSH or PEM private key, no AWS key, no GitHub token, no Slack, Stripe, Resend, Google or
Anthropic key, no signed JWT and no mnemonic. The public repository was published without that history
(ADR-063).

- **New live surfaces.** QOR ID is public at `https://id.qorsync.dev` (since 1 October 2026), and the devnet's
  RPC node at `wss://rpc.qorsync.dev` (since 3 October 2026). Each surface below is reachable from the internet, and none
  has been audited.
  - **Sign-in for other apps** (ADR-073): OAuth 2.1 code with PKCE at `/oauth/authorize`, `/oauth/token`,
    `/oauth/userinfo` and `/oauth/revoke`, with clients and their exact redirect URIs in `QOR_OAUTH_CLIENTS`,
    and refresh tokens rotated on every use. ARQADE is the one registered client.
  - **Avatar uploads** (ADR-079, migration 022): an uploaded image is re-encoded at 256 x 256 and served to
    anyone at `/avatars/{hash}`. Any signed-in account can report one, and administrators review reports and
    remove avatars.
  - **Signed `qor://pay` requests** (ADR-076, ADR-077): ARQADE's server signs a payment request, and the
    launcher checks the signature, the genesis hash and the 100,000 CGT cap, then asks in its host dialog.
  - **Three sign-ups per network address** (ADR-078, migration 020): the limit reads `X-Real-IP`, which it
    trusts because Railway's edge sets it. Behind any other proxy, that header is whatever the client sends,
    and without it (a direct local run) no limit applies.
- **No limit on concurrent QOR ID sessions.** A `max_sessions` setting existed but was never enforced. It
  was removed rather than left to imply a protection that does not exist.
- **QOR ID weaknesses that remain** (2026-09-15, not fixed). None reports work it did not do.
  - Some accounts cannot be recovered at all. They are named below under "Accepted lockouts in QOR ID".
  - Sessions record no real IP address: the address stored is a placeholder (`0.0.0.0`), so it is not
    shown. Since 2 October 2026 each session records when it was last used (sign-in or refresh), shown as
    `last_used_at`.
  - ~~A password-only account holds an `on_chain_address` derived from a hash, which no key can sign for.~~
    **Closed on 2026-09-19** (ADR-017, ADR-039). Registration creates no address, `hash_to_address` is
    removed so nothing can derive one again, the profile reports no chain account until a key is proven, and
    migration 018 cleared the derived addresses already stored.
  - Email goes through Resend. Delivery to a real inbox is not yet confirmed.
    - The `demiurge.cloud` sending domain is verified in Resend. Checked in public DNS on 2026-09-15: DKIM,
      SPF and the bounce MX record resolve and match.
    - DMARC is published at `p=none`, which only monitors: receivers are not asked to reject mail that
      fails authentication for `demiurge.cloud`.
    - The owner set a sending-only key in the environment on 2026-09-15. Resend refuses it anything but
      sending.
    - A live run the same day sent both messages from `noreply@demiurge.cloud` to Resend's test address
      `delivered@resend.dev`, and the whole flow completed through the links they carried. Resend marked
      both delivered, the reset message some minutes after the verification message.
    - `demiurge.cloud` itself has no MX record, so no address at it can receive mail.
    - QOR ID serves the pages the links open (`/verify-email` and `/reset-password`) on its own subdomain
      (ADR-015, clarified 2026-09-15). The name is set at deployment: `BASE_URL` is `https://id.qorsync.dev`
      (`services/qor-auth/DEPLOY-RAILWAY.md`). `BASE_URL` has no default, and email stays unconfigured unless it
      is an origin: HTTPS, or HTTP to this machine, with no path.
    - Resend's bounce and complaint reports are handled and verified, but none has been received live. QOR ID
      has had a public address since 1 October 2026; that the endpoint is set up in the Resend dashboard and
      `RESEND_WEBHOOK_SECRET` is set in Railway is **not confirmed** (an owner step). Without the secret every
      delivery is refused with 503.
- **There is deliberately no Dependabot, and no `.github/dependabot.yml`.** Automated dependency pull
  requests would fight [ADR-033](docs/decisions/ADR-033-dependency-versions.md) rule 1: the pinned SDK
  release governs every version it dictates, and its transitive pins are not overridden. A bot proposing
  those upgrades would open pull requests that must all be closed, and the one time it was right would be
  lost among them. Advisories are surfaced instead by `cargo audit` in CI — failing the run for
  `services/qor-auth` and the launcher, reporting without failing for `chain/` — and moving the pin is a
  deliberate task under ADR-033 rule 3. **If Dependabot is ever enabled, it needs a decision record
  first.**
- **Dependency advisories accepted with conditions** (owner, 2026-09-14; logged in `docs/GATES.toml`).
  - `services/qor-auth`: `rsa` (RUSTSEC-2023-0071) has no fixed release. It is in the lockfile but not in
    the compiled dependency graph, so it is ignored. CI fails if it ever enters the graph.
  - `framework/`: six advisories were exempt until the custom chain was retired. **The exemption expired
    with the directory on 2026-09-20**, when M3.5 deleted it, and CI now fails if `framework/` returns at
    all.
  - `chain/`: **no advisory is exempted, and there is no `.cargo/audit.toml`.** `cargo audit` reports 10
    vulnerabilities, every one transitive through the pinned Polkadot SDK. They were measured one by one
    on 2026-09-20: **four of the ten are compiled and six are not**, and every one of the four is held by
    the pin. Each is named, with what blocks it and what would close it, in
    [Dependency advisories in `chain/`](#dependency-advisories-in-chain) below. **This corrects an earlier
    claim in this file that none of the ten was reachable.** The audit runs in CI as a report that does not
    fail the run; it is not a gate, because making it pass today would need an exemption file.
  - `tools/qor-launcher/src-tauri`: no exemption file, deliberately. Two of its warnings arrive with
    `subxt`, and why `lru` is not in the compiled graph is recorded beside the dependency in `Cargo.toml`,
    where someone would turn that feature on.
  - RUSTSEC-2026-0285 (`rustls`, published 2026-09-14) is fixed in both lockfiles by upgrading to 0.23.45.
    **Correction, 2026-09-19:** only QOR ID's lockfile carried the upgrade. The launcher's still held 0.23.44,
    so `cargo audit` failed there, and the claim above had been true of one lockfile rather than two. It was
    upgraded on 2026-09-19 and both now audit clean. It is the kind of claim worth re-running rather than
    re-reading: the check takes a minute.
- **Accepted lockouts in QOR ID** (recorded as accepted at the owner's direction, 2026-09-15). These
  accounts cannot be recovered. Every route that would recover them would also let someone other than the
  owner take the account.
  - **No verified email, a forgotten password and no unspent backup code.** While it knew its password,
    the account could have added an address or regenerated its codes. Once all three are gone, nothing
    recovers it.
  - **A lost inbox and a forgotten password.** This applies to an account whose verified address it can no
    longer read, and to one whose address bounced permanently or reported a message as spam, since nothing
    more is sent to it.
    - An account that registered with an email holds no backup codes. It cannot generate them either:
      regeneration is only for an account without an email, or one that already holds codes.
    - While it still knows its password, it can move to a new address.
  - **A forgotten password while signed in.** Being signed in adds nothing. Adding or changing an address
    and regenerating codes all need the password, so that a stolen access token cannot attach an attacker's
    address or mint codes. Such a person recovers by the routes an account has when signed out, if it has
    one.
  - **A lost key on a keypair account.**
    - An account created by key sign-in has a random password nobody knows, no email and no backup codes,
      so it cannot add an address or codes either.
    - Its key is its only credential, and QOR ID never holds a key on anyone's behalf (ADR-014, ADR-017).

  **One more, once, from the key-scheme change of 2026-09-19** (ADR-023, ADR-039). It is not an ongoing
  class: it applies only to accounts created before that day.
  - Every key stored before it was an Ed25519 key, and accounts are Sr25519 now, so those 32 bytes name an
    account nobody holds a secret for. Migration 018 cleared them, which lets a password account link its
    new key.
  - **An account created by key sign-in before that day cannot sign in afterwards.** Its password is a
    random value nobody knows and its key no longer verifies, so it has to be created again. No network
    holds value, and QOR ID was first deployed on 1 October 2026, after the change, so the accounts this
    touches are development accounts.

  Temporary refusals are not lockouts:
  - the sign-in lockout after failed attempts expires;
  - the limits on verification links clear within 5 minutes and 24 hours;
  - a 503 because email is not configured ends when it is.

Fixed on 2026-09-13 (milestone M1):

- **Transaction replay.** Nonces are now enforced on chain (D-004).
- **Partial writes from failed transactions** (D-005).
- **Divergent state from direct-write RPC methods**, now restricted to `--dev` nodes (D-008).

Remediated on 2026-09-14 (security track, `docs/DIRECTION.md` §7.1). Each was found during the M1 audit
and carries forward as a requirement for the Polkadot SDK chain and QOR ID (ADR-013):

- **Committed secrets removed.** The `qor-auth` database password and both JWT signing secrets were
  removed from `services/qor-auth/config/production.toml`, `config/production/qor-auth.toml` and
  `config/production/.env.hub`. **They remain in git history and are treated as compromised.** The
  database password and both JWT secrets must be rotated before any deployment. **The owner confirmed
  the rotation on 14 September 2026** (`HANDOFF.md` §3). Production values come from the environment
  (`QOR_AUTH__DATABASE__URL`, `QOR_AUTH__JWT__ACCESS_SECRET`, `QOR_AUTH__JWT__REFRESH_SECRET`).
- **Nine more values, in the private archive's history, are NOT recorded as rotated.** Found on 21 September
  2026 by a full-history scan of the private archive, when they were still in its tree. **This list is the
  record.** Each is named by file and variable, with the line it was on then; no value is written here or
  anywhere else in this repository's documentation.

  | File | Line | Variable |
  | --- | --- | --- |
  | `docker/n8n/docker-compose.yml` | 8 | `POSTGRES_PASSWORD` |
  | `docker/n8n/docker-compose.yml` | 31 | `DB_POSTGRESDB_PASSWORD` |
  | `docker/n8n/docker-compose.yml` | 36 | `N8N_ENCRYPTION_KEY` |
  | `docker/n8n/docker-compose.yml` | 41 | `N8N_BASIC_AUTH_PASSWORD` |
  | `docker/docker-compose.testnet.yml` | 25, 59, 89, 119 | `NODE_KEY` (four libp2p node secret keys) |
  | `docker/docker-compose.testnet.yml` | 184 | `GF_SECURITY_ADMIN_PASSWORD` |

  **Neither file is in today's tree or in the public repository's history.** The public repository, now
  `QOR-MATRIX/demiurge-chain` (ADR-063, moved by ADR-064), was published on 29 September 2026 from the tree
  without history and without these two files, which its `.gitignore` names. They exist only in the history of
  the private archive, `ALaustrup/demiurge-cloud`. **If those services were ever run, rotate the values.**
  `N8N_ENCRYPTION_KEY` is the one to treat most carefully: it is the master key n8n uses to decrypt every
  credential it stores, so if that stack was ever started, rotating the key makes the stored credentials
  **undecryptable rather than safe** — each one has to be re-entered afterwards.

  **CI's credential scan would not have seen them**, which is why nobody noticed: its pathspec is still
  `*.toml`, `*.env`, `*.env.*` (`.github/workflows/ci.yml`), and both files were `.yml`; it matches two
  lowercase TOML key names; and it reads the checked-out tree, never history. Widening the pathspec is a
  tightening and is still worth doing, so that a credential in a `.yml` file is caught next time. Checked on
  6 October 2026: the same pattern over `*.yml` today finds one line, the throwaway database URL CI's own build
  uses on `localhost` (`ci.yml`), which a widened scan would have to tell apart first.
- **Seeded administrator removed.** Migration 008 no longer seeds the `godmode` account or its recovery
  code. On a database that ran the original 008:
  - migration 010 disables the account;
  - the stored sqlx checksum for version 8 must be updated once before the service starts, as described
    in the migration's header.

  The old password and recovery code remain in git history. If that password is used anywhere else,
  change it.
- **CGT created by a transfer to oneself.** A signed `Balances` module call naming the sender as the
  recipient credited the amount, because both balances were read before either was written. The
  dev-only `balances_transfer` RPC had the same flaw. Self-transfers are now refused, with tests at
  module and runtime level. Fixed in the custom chain being replaced; the requirement carries to the
  Polkadot SDK chain.
- **Unauthorised DRC-369 mint and nesting.** A signed mint could place an asset in any account. Nesting
  checked the child's owner but not the parent's, which let a stranger block an owner's burn. A
  transaction now mints only into the caller's own account, and nesting requires owning both assets.
  Tests cover both. Nesting cycles among one owner's own assets are still possible; refusing them is
  requirement R-2 for the DRC-369 pallet.
- **QOR ID no longer creates or holds agent private keys** (ADR-014).
  - Agent registration requires the agent's own public key and a signature proving possession.
  - The controller is taken from the access token.
  - Migration 011 removes every key the old flow stored.
  - Only the controller can list, read, change or deactivate its agents, and an agent cannot register
    agents.
- **Key links are authenticated and proven.**
  - `POST /api/v1/auth/link-keypair` now requires an access token.
  - `POST /api/v1/profile/link-wallet` no longer writes to a placeholder account.
  - Both require a signature over a single-use challenge, and bind the key to the account in the token.
  - Replacing an already linked key is refused.
  - QOR ID verified Ed25519 signatures strictly, so a forged signature for a small-order key was refused.
    **Since 2026-09-19 it verifies Sr25519** (ADR-023), so that forgery is refused as a signature the
    service does not accept at all. The test that measures it is kept and now measures both halves, as
    ADR-023 point 5 requires.

  Both QOR ID changes were verified against a running service and Postgres 16 on 2026-09-14: 27
  end-to-end checks passed.
- **QOR ID challenge signatures are domain-separated** (security track item 7).
  - A key signs `demiurge:qor-id:challenge:v1:` followed by the challenge, never the bare challenge. The
    same vault key signs chain transactions, so an identity server can no longer obtain a signature that
    could mean something else.
  - `qor-auth` refuses a signature over the bare challenge at sign-in, registration, agent registration
    and key links.
  - The launcher refuses to sign anything that is not in the service's challenge format,
    `demiurge:<unix seconds>:<64 lowercase hex>`.
  - Verified with unit tests on both sides, and against a running service on 2026-09-14 (28 end-to-end
    checks passed).
- **The launcher asks before it signs or repoints** (L1.4, implemented 2026-09-14).
  - The vault has no way to sign without an approval. Every transfer, starter-grant claim and QOR ID
    challenge is shown first in a native dialog that the host draws. The dialog is built from the values
    being signed: for a transfer, the exact amount, both addresses and the node; for a challenge, the
    account and the identity service.
  - One exception (ADR-016). Unlocking or creating the vault opens a grant for its first account to answer
    QOR ID sign-in and name-claim challenges without a dialog. Since ADR-056 opening it asks nothing: its key is
    in the operating system's keychain, so the grant rests on the person being signed in to their computer.
    The grant closes when a QOR ID session is established, when the vault locks, or after five minutes.
    Nothing else uses it.
  - Nobody is asked to approve a signature that cannot be made, such as one from a locked vault or an
    account it does not hold.
  - The chain node and QOR ID endpoints change only after the same kind of approval, which names the
    current and the requested address.
  - The webview no longer holds any dialog permission, so it cannot draw a look-alike prompt.
  - Unit-tested with a scripted confirmer. **One native dialog has been used in a running launcher:** on 5 and
    6 October 2026 the owner approved `qor://pay` tips in the host dialog of an installed launcher 0.1.7. The
    rest of the list in `HANDOFF.md` — a transfer and an endpoint change, each approved once and declined once,
    and a declined `qor://pay` request — is not recorded as tried, so roadmap item L1.4 stays unchecked until it
    is.
- **QOR ID no longer reports work it did not do.** Found while bringing `qor-auth` under the new format and
  lint gates (L1.7):
  - `POST /api/v1/zk/verify` answered `valid: true` for any non-empty hex string, and the attestation
    request returned an invented id. Both now refuse with 501; no proof is verified (D-010).
  - The god-only CGT transfer and refund endpoints reported success with an all-zero transaction hash.
    They now refuse with 501.
  - Passwords were checked against a hard-coded 6 characters while `security.password_min_length` was
    never read. The configured minimum is now enforced at registration and in both password resets.
  - `server.host` was never read, so the service listened on every interface. It now binds the
    configured address.
  - Logout did nothing. It now deletes the session, so its refresh token stops working. Every access token
    is also checked against its session, so logout ends the access token at once, not when it expires. A
    session lookup that fails refuses the request with 500. Verified against a running service, Postgres 16
    and Redis on 2026-09-14 (15 end-to-end checks, including a second session of the same account left
    working and a Redis outage refused).
  - `validator` was upgraded from 0.18 to 0.20, which removes the vulnerable `idna` 0.5.

  Verified against a running service and Postgres 16 on 2026-09-14: 26 end-to-end checks passed.
- **Registration no longer calls the chain** (migration inventory R-3).
  - `POST /api/v1/auth/register` called `balances_claimStarter` on whatever node `BLOCKCHAIN_RPC_URL`
    named, with no check that it was a development chain, and told the user "100 CGT has been added".
  - It created nothing only because the custom node refused the claim outside `--dev --faucet`, required
    the claimant's own signature, and was sent one parameter where three were required. That node was deleted at M3.5; the defence that mattered is that QOR ID no longer makes the call at all.
  - The call, the `starter_cgt_minted` field and the claim are removed, and `BLOCKCHAIN_RPC_URL` is no
    longer read. QOR ID has no HTTP client for any chain.
  - Verified on 2026-09-14 with `services/qor-auth/scripts/e2e/registration-mints-nothing.mjs`: a fake
    chain node received no request from two registrations (8 checks). The previous build sent it two
    claims and failed 5 of the 8.
- **QOR ID sessions list and revoke for real.**
  - `GET /api/v1/profile/sessions` always returned an empty list, and `DELETE
    /api/v1/profile/sessions/{id}` answered 204 without deleting anything.
  - The list now shows the caller's live sessions and marks the current one. It omits the IP address and
    last activity, because neither is recorded. *(Since 2 October 2026 it shows `last_used_at`; the IP
    address is still a placeholder and still omitted.)*
  - Revoking deletes one of the caller's sessions, so its access and refresh tokens stop working at once.
    Another user's session and an unknown id are both 404.
  - Verified on 2026-09-14 with `services/qor-auth/scripts/e2e/sessions.mjs` (20 checks; the previous
    build failed 11) and a Redis-backed unit test run with `--ignored`.
- **QOR ID no longer reports work it did not do** (second sweep, 2026-09-14). Each fix has tests against a
  real Postgres, and `services/qor-auth/scripts/e2e/account-and-admin.mjs` checks them through routing (35
  checks, all passing).
  - **Email verification.** Registration stored `email_verified` inverted, so every supplied address was
    marked verified. An address is now stored unverified until verify-email succeeds. Migration 012
    corrects existing rows: an address marked verified while its verification token is still outstanding
    was never verified. It also adds a constraint that refuses that state. Three of the four new tests
    failed on the previous code; the fourth tests the migration itself.
  - **Links in the service log.** With email not configured, the service wrote whole emails, reset and
    verification links included, to its log. It now never logs an email. `forgot-password` refuses with
    503 before any lookup, and registration says no verification email was sent. Service logs from the
    previous build held 8 links; the new build's log held none.
  - **Account disclosure in `forgot-password`.** The answer is identical for every identifier, and the
    lookup and sending happen after it. A reset link goes only to an active account's verified address. Both
    new tests failed on the previous code.
  - **Admin routes.** Every route does what it reports or refuses; CGT transfer and refund stay 501.
    - Ban, unban and role change return 404 for a user that does not exist. Each writes its change and
      audit row in one transaction, under the administrator's id from the access token.
    - Migration 013 refuses the nil id for any account or audit row, so a placeholder actor cannot be stored.
    - Ban and role change revoke the account's sessions. An administrator cannot ban, or change the role of,
      their own account, and unbanning an account that is not banned is refused.
    - Statistics count the sessions held in Redis and the `login` rows sign-in now writes, instead of a
      hard-coded 0 and rows nothing wrote.
    - Two defects that would have shown once the routes were reachable are fixed: role change bound text to
      an enum column, and the user list could not decode an account without an email.
    - Only after that was the role check fixed. Sign-in wrote the role as `God`, and `require_god` compared
      it with `god`; both now use `UserRole::as_str`. Tokens issued before carry `God` and need a new
      sign-in.
  - **Email password reset** sent two SQL statements as one prepared statement (42601), so it always failed.
    It is now one transaction of two statements, and a token can be spent once. The new test failed on the
    previous code with the same error.
  - **Profile stubs.** Profile update returns 501, and the profile no longer returns a balance it did not
    read. Avatar upload also returned 501 until 6 October 2026, when it was built (ADR-079, migration 022):
    `POST /api/v1/profile/avatar` re-encodes the image and stores it.
- **Account recovery and sign-in hardened** (2026-09-15). Each fix has tests against a real Postgres; those
  that need Redis run in CI with `--include-ignored`.
  - **Email through Resend.**
    - Verification and reset messages go through Resend's HTTP API, configured only by `RESEND_API_KEY`
      and `EMAIL_FROM` in the environment. Without both, every send is refused with 503.
    - Neither the API key nor a message is ever logged. `RESEND_API_URL` is used only if it is HTTPS, or
      HTTP to the local machine. The SMTP client (`lettre`) is removed.
    - Checked end to end against a local stand-in for Resend (`scripts/e2e/email-via-resend.mjs`, 13
      checks): an address is verified through the message it receives, an unverified address gets no reset
      link, and a verified one does.
    - Live through `demiurge.cloud` on 2026-09-15, to Resend's test address:
      - registration's message was delivered, and its link verified the address;
      - a reset link was sent, and it reset the password;
      - each link worked once;
      - neither the links nor the key reached the service log.

      Delivery to a real inbox is not yet confirmed.
  - **A password reset ends every session.** Resetting by email token or by backup code revokes all of the
    account's sessions before the change commits. If they cannot be revoked, nothing is reset. Both new
    tests failed on the previous code.
  - **Backup codes are single-use and stored hashed.**
    - An account without an email receives ten codes, and only their SHA-256 hashes are stored. Migration
      014 moves each existing plain-text code into that form and drops the column.
    - A code is spent in the same transaction that sets the password, a spent code is refused, and the
      response says how many remain.
    - A wrong code, a spent code and an unknown username get the same answer.
  - **Sign-in discloses nothing.**
    - An unknown account, a wrong password, a locked account and an account that is not active all get the
      same 401. The identical-answer test failed on the previous code.
    - Every attempt does the same work: one lookup, one Argon2 verification (against a stand-in hash when no
      account matches) and one attempt-counter update. In the end-to-end check, an unknown username took
      313 ms to refuse and a wrong password 314 ms.
    - A failed attempt is still counted only against an open account.
- **Recovery routes for locked-out accounts** (2026-09-15). The committed code had neither route (404).
  - **A new verification link.**
    - `POST /api/v1/auth/resend-verification` issues a new token for an active account with an unverified
      email address, and sends it. The new token replaces the old one, so an earlier link stops working.
    - It refuses with 503 when email is not configured. Otherwise it answers every identifier identically,
      and looks up and sends after the response.
    - Each account is sent at most one message every 5 minutes, and 5 in any 24 hours. The limit is checked
      and recorded in the statement that replaces the token, and registration's message counts as the first
      (migration 015).
    - An account whose token lapsed recovers through it: shown by a database test, and end to end against a
      stand-in for Resend (24 checks, including a reset link once the account is verified).
  - **New backup codes.**
    - `POST /api/v1/profile/backup-codes` issues ten new codes for an account without an email address, or
      for one that added an address while it held codes.
    - It needs both the access token and the account's password, so a stolen token alone cannot mint
      recovery codes and take the account over. A wrong password counts as a failed sign-in attempt, and a
      locked or inactive account is refused.
    - One transaction locks the account row, deletes every old code, spent or not, and stores the new hashes.
      Two requests at once leave exactly one set of ten (tested).
  - **Adding or changing an email address** (the owner's decision on 2026-09-15). The committed code had no
    such route (404).
    - `POST /api/v1/profile/email` takes `{"email", "password"}` with the access token. A stolen token alone
      cannot attach an address. A wrong password is refused and counts as a failed sign-in attempt, and a
      locked or inactive account is refused. It refuses with 503 when email is not configured.
    - The new address is held as pending (migration 016) and sent a confirmation link that lasts 24 hours.
      Until it is confirmed the account is unchanged. An account without an email still has none, gets no
      reset link, and keeps its backup codes. An account changing its address keeps the old one.
    - When the account already has an address, that address is told a change was requested. The notice
      carries no link.
    - Requests share the verification-link limits: one message every 5 minutes, and 5 in 24 hours.
    - An address another account holds is refused, both on request and on confirmation. If either message
      cannot be sent, the pending change is withdrawn.
    - Confirming does not touch backup codes, so an account that held codes ends up with both routes. It can
      still regenerate its codes with its password.
    - A password reset cancels any pending change, so an address an intruder left pending does not complete
      once the owner resets.
    - Tested against a real Postgres (nine tests; the reset one needs Redis). Verified end to end against a
      stand-in for Resend (40 checks, including a reset link once the added address is confirmed and a
      backup code still working after it).
  - **What remains unrecoverable** is named above, under "Accepted lockouts in QOR ID".
- **A check that would have stopped testing silently** (2026-09-17). `registration-mints-nothing.mjs`
  carries the end-to-end evidence for requirement R-3: that registration creates no currency and calls no
  chain. It asserted that the response never mentions "cgt". The ticker became `CGT` the same day
  (ADR-034), after which that assertion would have passed while testing nothing.
  - It now asserts against every name the currency answers to, and records in a comment that the list is
    extended in the same change as any future rename.
  - A passing check that tests nothing is worse than no check, because it is counted as evidence.
- **A citation of deleted documentation** (2026-09-17). The launcher's `cgt.rs` justified the currency's
  name by citing two pre-realignment documents deleted on 2026-09-13. AGENTS.md §1 forbids relying on
  them. The citation is removed and points at `.cursorrules`.
- **Links that survive mail scanners** (2026-09-15). The committed code served no page at the links' addresses
  (404).
  - Email security products open links on their own, which would burn a one-use token before the person
    clicked. QOR ID now serves `/verify-email` and `/reset-password`.
  - Opening either only reads: it shows who the link is for and a button, or says the link no longer works.
  - The token is spent only by the form the button submits. This covers all three link types: address
    verification, confirmation of a new address, and password reset.
  - A reset form with two different passwords, or one too short, changes nothing and keeps the link usable.
  - A used or expired link says what happened and how long such links last. It offers a form that asks for a
    new one, and that form answers exactly as the API does for every identifier.
  - Pages run no script and send no referrer. They cannot be framed, cached or indexed.
  - Request log spans now record only the path. Before, the full URI was logged at debug level. In the
    fail-first run the committed build wrote both tokens it was sent into its log; the new build wrote
    none.
  - Tested: opening each kind of link three times changes nothing, and only the button does. Verified end to
    end in `scripts/e2e/email-via-resend.mjs` (67 checks in all): each link opened three times first, a
    refused password form, a used link, and a new link asked for on the page.
- **Bounce and complaint reports from Resend** (2026-09-15). The committed code had no such route (404), so a
  reset sent to a dead address failed silently for good.
  - `POST /api/v1/webhooks/resend` accepts a delivery only with a valid Svix signature, keyed with
    `RESEND_WEBHOOK_SECRET`. A missing or wrong signature is refused, and so is a timestamp more than five
    minutes from now. The HMAC comparison is constant-time.
  - Without the secret, every delivery is refused with 503.
  - Each delivery id is acted on once. It is recorded and acted on in one transaction, so a failure lets
    Resend retry.
  - A permanent bounce, a Resend suppression or a complaint marks each recipient address undeliverable
    (migration 017). Only a SHA-256 of the address is stored.
  - Nothing more is sent to a marked address: every send checks the list, which `AppState::new` attaches.
    A pending change to it is withdrawn, the profile shows `email_deliverable: false`, and the account gets
    an audit row.
  - Moving away from a marked address skips the notice to it. Adding a marked address is refused.
  - A temporary bounce marks nothing.
  - Logs carry the event type, Resend's email id and counts, never an address, subject or link.
  - Tested, including Svix's documented signature example and a replayed, stale or forged delivery. Verified
    end to end:
    - unsigned, wrongly signed and ten-minute-old deliveries are refused, and mark nothing;
    - a temporary bounce marks nothing;
    - a permanent bounce stops reset links, is acted on once when repeated, and shows in the profile;
    - a complaint stops sending;
    - no address, link, key or secret reached the service log.
  - **Not yet received live.** Resend can deliver only to a public address; QOR ID has one,
    `https://id.qorsync.dev`, since 1 October 2026. That the endpoint is set up in the Resend dashboard and
    `RESEND_WEBHOOK_SECRET` is set on the Railway service is **not confirmed**; both are owner steps
    (`HANDOFF.md` §4.0, `services/qor-auth/DEPLOY-RAILWAY.md`).
  - **Unmarking is admin-only** (`POST /api/v1/admin/email-suppressions/unmark`).
    - A god account cannot clear an address that is its own or pending on its own account. If the
      account holder could clear its own mark, the mark would mean nothing.
    - A reason is required.
    - The removal and its audit row, under the acting administrator and holding the address's hash, never
      the address, are one transaction.
    - The address travels in the body, since paths are logged. An address that is not marked is 404.
    - Tested:
      - an administrator unmarks an address, and the action is audited with no address in the row;
      - an address that is not marked is not found, and nothing is written;
      - unmarking needs an address and a reason;
      - an administrator cannot unmark their own address, current or pending;
      - an account holder's token is refused (403), in the log check.
- **No token, key, secret, password, link or address in any log, at any level** (2026-09-15; GATES
  `alpha.no-secrets-in-logs`). Token material reached the logs three times from different directions:
  reset links written to the service log, message bodies, and full request URIs at debug level. So the
  property is checked as a whole.
  - **At runtime:** `log_hygiene::nothing_secret_reaches_a_log_at_any_level` drives through the real
    router every flow that handles such a value. It covers registration, both link pages, reset, sign-in,
    a change of address, signed, forged and stale webhooks, and the unmark route, with every log record
    captured at trace level, including records from crates that log through `log`. It fails if any
    token, key, secret, password, backup code, link or address appears. It first checks that the capture
    worked, so an empty log cannot pass.
  - **Statically, in CI's security job:** the build fails if a log call in `services/qor-auth/src`
    interpolates such a value by name, or a request span records the URI. On a planted file it flagged an
    interpolated token and address, a span recording the URI, and a bare `TraceLayer`, and passed the
    harmless calls beside them.
  - **SQL is always parameterised, and the build enforces it** (GATES `alpha.parameterised-sql`). sqlx logs
    each statement's full text at debug level, so a value written into SQL text, rather than bound as a
    parameter, would reach the log. The log check found this through another test's literal SQL.
    - `sql_hygiene::every_query_in_the_source_is_a_literal_with_bound_parameters` reads the crate's
      source.
    - It fails on a sqlx query function given SQL that is not fixed at compile time. Fixed means a string
      literal, `include_str!`, an upper-case constant, or a loop variable over an upper-case constant.
    - It also fails on importing those functions unprefixed, on a `QueryBuilder` fed by `format!`, and on
      `format!` or `push_str` text that reads as SQL.
    - Its first run found four `raw_sql` calls in migration tests that an earlier search had missed. They
      run migration files read in with `include_str!`, which are fixed text, and the check now says so.
    - It must have seen the service's queries before a clean result counts.
    - A second test pins that each of those forms is found.
  - **Limits, and the rule that covers them.** The runtime check covers only the flows it drives. The
    static scan matches variable names, so a value under another name passes it unseen. AGENTS.md §9
    therefore makes it a review requirement, not advice: a change that introduces a flow handling such a
    value adds it to the runtime check in the same change, whatever the value is called.

## Dependency advisories in `chain/`

**Measured on 20 September 2026**, as its own task: establish what is actually compiled, then upgrade or
report each advisory under [ADR-033](docs/decisions/ADR-033-dependency-versions.md) rules 2 and 4.
`cargo audit` reports **10 vulnerabilities and 13 warnings** in `chain/`. Nothing is exempted, and the
directory deliberately has no `.cargo/audit.toml`.

**Correction.** This file, `HANDOFF.md`, `docs/GATES.toml` and CI all said that none of the ten was
reachable through a normal dependency edge. **Four of them are.** The claim came from a `cargo tree` run
that was not made per crate *version*: two versions of `hickory-proto` and two of `tracing-subscriber` are
in the lockfile at once, so a query answering for the unreachable copy read as though it covered both.
The measurement below is per advisory, by exact version.

**Method**, so it can be repeated. For each crate and version:
`cargo tree --workspace -i <crate>@<version> --target all`. `cargo tree` follows normal, build and
dev edges by default, so a dev-dependency counts as compiled; `--target all` covers platform-gated
dependencies. A crate the query cannot reach is in `Cargo.lock` only because a lockfile resolves the
union of every optional dependency in the graph, and no feature this workspace can enable selects it —
which was confirmed by repeating each query with `--all-features`.

### The four that are compiled

| Advisory | Crate | Reached through | What blocks the fix |
| --- | --- | --- | --- |
| RUSTSEC-2025-0055 (ANSI escapes in logged input) | `tracing-subscriber` 0.3.19 | `sc-tracing` 47.0.0 and `sp-tracing` 19.0.0, so the node and every runtime crate | `sc-tracing` 47.0.0 requires `tracing-subscriber = "=0.3.19"`, **exactly**. Fixed in 0.3.20. |
| RUSTSEC-2026-0119 (O(n²) name compression, CPU exhaustion) | `hickory-proto` 0.24.4 | `hickory-resolver` 0.24.4 → `libp2p-dns` 0.42.0, and `libp2p-mdns` 0.46.0 → `libp2p` 0.54.1 → `sc-network` 0.58.0 | `libp2p-mdns` 0.46.0 requires `hickory-proto ^0.24.1`. Fixed in 0.26.1, which `libp2p-mdns` first admits at 0.49.0. |
| RUSTSEC-2026-0119, as above | `hickory-proto` 0.25.2 | `hickory-resolver` 0.25.2 → `litep2p` 0.14.3 → `sc-network` 0.58.0 and `sc-network-types` 0.22.0 | `hickory-resolver` 0.25.2 requires `hickory-proto ^0.25`. |
| RUSTSEC-2026-0118 (NSEC3 proof validation, unbounded loop) | `hickory-proto` 0.25.2 | as above | **No fixed release exists**, at any version. |

**What they reach.** The `hickory` advisories are in DNS message handling, which a node uses to resolve
bootnode addresses (`libp2p-dns`, `litep2p`) and to discover local peers (`libp2p-mdns`). Both need a DNS
response the node processes. Whether the NSEC3 path in RUSTSEC-2026-0118 runs at all depends on the
resolver's validation settings, which were **not** measured — it is listed as compiled, not as exercised.
RUSTSEC-2025-0055 needs input the node logs and a person reading those logs in a terminal.

**Why none of them moves.** Each is held by a version the pinned SDK release dictates (ADR-022), and
ADR-033 rule 1 says the pinned release governs and its transitive pins are not overridden. Forcing any of
them forward would need a `[patch]` section, which rule 1 forbids and which criterion
`alpha.no-dependency-patching` fails. A `cargo update --precise --dry-run` for each names the blocking
requirement and the path back to `demiurge-node`; that is the evidence for the table above.

**The way through** (ADR-033 rule 4: report, do not spin).

- **`tracing-subscriber` is not fixable by moving the pin within what is published.** `sc-tracing` 47.0.0
  is the newest release, and every release from 45.0.0 carries the same exact pin. It closes when upstream
  Substrate relaxes it, and not before.
- **The two `hickory` advisories may be fixable by a later SDK release**, since libp2p admits the fixed
  `hickory-proto` at `libp2p-mdns` 0.49.0. Whether `polkadot-stable2606-2`, `-3` or `-4` carries a libp2p
  or litep2p new enough was **not** measured: `polkadot-sdk = "=2606.1.0"` is an exact pin, so even a dry
  run needs the manifest edited, and editing it *is* the move.
- **This is a reason to propose the pin move.** ADR-033 rule 3 names "a later patch release of the pinned
  line fixes a security issue" as a trigger, and the first step of that task is the measurement the exact
  pin prevents here. It is not done incidentally, and not during M3.
- **RUSTSEC-2026-0118 is carried regardless**, because no release anywhere fixes it.

### The six that are not compiled

In `Cargo.lock`, absent from every graph this workspace can resolve.

| Advisory | Crate | In the lockfile through |
| --- | --- | --- |
| RUSTSEC-2026-0258 | `h2` 0.3.27 | `hyper` 0.14.32 |
| RUSTSEC-2025-0009 | `ring` 0.16.20 | `rcgen` 0.11.3 |
| RUSTSEC-2026-0098, RUSTSEC-2026-0099, RUSTSEC-2026-0104 | `rustls-webpki` 0.101.7 | `libp2p-tls` 0.5.0 |
| RUSTSEC-2025-0055 | `tracing-subscriber` 0.2.25 | `ark-relations` 0.5.1 |

Each carrier is itself unreachable, so none of the six is one enabled feature away. A current, fixed copy
of `ring`, `rustls-webpki`, `h2` and `tracing-subscriber` is in the graph alongside the old one, which is
why the old copies are easy to misread as compiled.

### Why there is still no exemption file

An `audit.toml` would make the run pass and make all ten invisible, including the four that are compiled
and the next one that appears. The advisories are recorded here instead, each with the condition that
closes it. CI runs `cargo audit` in `chain/` as a **report that does not fail the run**
(`docs/GATES.toml`, `[ci].reports`), so a new advisory is visible without anything being silenced, and no
release criterion reads it. Making it a gate is a decision for when the four can actually be closed.

**Re-run it whenever `chain/Cargo.lock` changes, and as the first step of any SDK pin move.**

## Cross-origin requests

**Until 21 September 2026 QOR ID answered `Access-Control-Allow-Origin: *`** on every route, including
`/api/v1/admin/*`. Any page in any tab could script this API from a visitor's browser and read the
response. Bearer tokens made it less dangerous than cookies would have — an attacker's page cannot
silently attach a token it does not hold — but nothing constrained which frontends could talk to QOR ID,
and there was no list of legitimate origins to check a redirect against.

**It is now an allowlist**, `server.allowed_origins`, read from configuration because the origins are a
deployment fact rather than an engineering choice. An origin that is not on the list receives no
`Access-Control-Allow-Origin` header at all, so the browser refuses the response.

- **The default is loopback only** — `localhost` and `127.0.0.1`. A real hostname in the defaults would
  mean a deployment origin had been compiled in, and a test fails if one appears.
- **An empty list allows nothing**, which is the correct configuration for a deployment serving no
  browser frontend. It does not mean "allow everything".
- **The launcher and agents are unaffected.** They are not browsers, they send no `Origin`, and CORS
  does not apply to them.
- **Production allows only the loopback defaults** (6 October 2026). The list can be set only in
  `config/production.toml`, which does not set it. Setting it from the environment as
  `QOR_AUTH__SERVER__ALLOWED_ORIGINS` stops QOR ID from starting. The `config` crate (0.14.1) is given no list
  separator in `config.rs`, so it refuses a string where it expects a list. This was tested with one value, a
  comma-separated list, an empty value and the `__0` form. Nothing needs a public origin today: QOR ID's own pages
  are same-origin, and ARQADE calls QOR ID from its server, not from the browser.
- **Four tests hold it**, and they were proven to fail first: with `allow_origin(Any)` restored, three
  of the four fail, including one whose only job is that a wildcard never comes back.

**If a browser frontend is ever given a session cookie**, this allowlist stops being a tightening and
becomes a prerequisite: the CORS specification forbids `Access-Control-Allow-Origin: *` alongside
credentials, so a cookie face cannot use a wildcard even if someone wanted to.

## Reporting a vulnerability

Do not open a public issue. Contact the maintainer, [@ALaustrup](https://github.com/ALaustrup), directly,
with reproduction steps. Allow reasonable time for a fix before disclosure.

## Rules for contributors

- **Never commit private keys, mnemonics, validator key files or secrets.** `.gitignore` excludes the
  known file patterns; check before adding any key-shaped file. Configuration files hold no passwords,
  connection strings with credentials, or signing secrets: those come from the environment.
- **Any method that changes chain state must be a signed transaction executed in a block.** Direct
  storage writes from RPC are development-only (D-008).
- **No code path may create CGT outside `--dev`** until the issuance mechanism of ADR-004 is designed,
  built and reviewed.
- **Nothing secret reaches a log** (AGENTS.md §9; checked in review, and a change that fails it is not
  approved).
  - A change that introduces a flow handling a token, key, secret, password, backup code, link or email
    address in `services/qor-auth` adds that flow to `src/log_hygiene.rs` in the same change, whatever
    its values are named.
  - Passing CI's name-based scan is not evidence on its own.
  - Every SQL value is bound as a parameter; a query built from values fails the build.
