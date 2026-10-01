# Hosting and operations: two domains

**Status:** A plan written 20 September 2026, **partly overtaken by decisions since**. Updated 30 September
2026. [ADR-042](../decisions/ADR-042-two-domains.md) (the two domains) was **accepted** on 28 September.
[ADR-043](../decisions/ADR-043-qor-id-as-an-identity-provider.md) and
[ADR-044](../decisions/ADR-044-validators-are-not-publicly-addressable.md) are still **Proposed**, and are not
decisions until the owner accepts them.

**What changed since it was written.** [ADR-063](../decisions/ADR-063-public-repository-actions-and-railway.md)
(29 September) moved **QOR ID, Postgres and Redis to Railway**, project `demiurge`, region `iad`, replacing
[ADR-015](../decisions/ADR-015-infrastructure-ownership.md)'s Fly.io for those three services. Its setup is
recorded in [`services/qor-auth/DEPLOY-RAILWAY.md`](../../services/qor-auth/DEPLOY-RAILWAY.md). It also made the
repository public as `ALaustrup/demiurge-chain`, so **CI is GitHub Actions and free**. On 30 September Postgres and
Redis were live on Railway, QOR ID was configured there but not yet built, and `id.qorsync.dev` still pointed at the
owner's computer (`HANDOFF.md` §4 item 32). **Everything else here that names Fly** (validators, the public RPC node,
the archive node, the indexer) **is still ADR-015's plan**, which ADR-063 did not reopen, and none of it is deployed.
Sections below that ADR-063 changed say so where they stand.

**Two domains, from the owner (20 September 2026):**

- **`qorsync.dev`** — all QOR operations: QOR ID and auth, the development backend, relays, agentic
  synchronisation, public chain RPC, and the launcher's backend needs such as update endpoints.
- **`demiurge.cloud`** — the user-facing frontends: dashboards and visual experiences.

**A correction to the brief.** The SQD indexer is
[ADR-028](../decisions/ADR-028-provenance-from-events-archive-node-and-indexer.md), not ADR-030; ADR-030
is the existential deposit. This plan uses ADR-028.

**QOR Engine.** The owner named it among the QOR operations. **It is not defined anywhere in this
repository** — no code, no document, no configuration mentions it, verified by searching the tree on
20 September 2026. It is recorded here as *named by the owner, scope undefined*, and a subdomain is
reserved for it below. **No scope is invented for it**, and nothing in this plan depends on it.

---

## 1. Inventory: what will need hosting, and when

"First needs to be live" is the earliest milestone at which the component cannot do its job from a
laptop. **Nothing needs to be live before it is deployable**, and most of this is not built yet.

| Component | What it is | Exists today? | First needs to be live | Why then |
| --- | --- | --- | --- | --- |
| **QOR ID** (`services/qor-auth`) | Rust/Axum identity service | **Yes**, runs locally | **Alpha**, and unavoidably at Beta | Accounts have to be the same accounts on more than one machine. Local-only works while one person tests; a team cannot share a local Postgres |
| **Postgres** | QOR ID's data; later the indexer's | **Yes, live on Railway** (ADR-063) | With QOR ID | Railway's Postgres template, private network. ADR-015's Fly Managed Postgres is superseded for QOR ID |
| **Redis** | QOR ID session storage | **Yes, live on Railway** (ADR-063) | With QOR ID | ADR-015 leaves Redis-versus-Postgres sessions open |
| **QOR ID's link pages** | The pages its emails open | **Yes**, served by QOR ID | With QOR ID | Verification and reset links are useless without a public HTTPS address. This is the open item in `HANDOFF.md` §4 item 7 |
| **Validator nodes** | `demiurge-node --validator` | **Yes**, run locally; the two-validator script is proven | **Alpha** (team devnet), certainly **M7** | A devnet more than one person reaches has to run somewhere. §4: no public hostnames |
| **Public RPC node** | A full node serving safe RPC | Node exists; **not deployed, not exposed** | **M5.1**, certainly **M7** | An SDK nobody can point at a node is untestable by anyone outside the team |
| **Archive node** | `--state-pruning archive-canonical` | **No.** Not built, not configured | **M5.4** (public viewer) | ADR-028: the indexer reads from it. Its disk grows forever |
| **SQD indexer** | `@subsquid/substrate-processor` into Postgres | **No.** ADR-028 chose it; the spike it requires has not been done | **M5.4** | The public viewer reads the indexer's database, not the chain |
| **Launcher update endpoint** | Update manifest and signed artifacts | **No.** L6.2 not started | **Before the public testnet** (L6, Beta) | An installed launcher with no update path cannot be fixed after release |
| **Public viewer** | The per-work shareable page | **No.** M5.4 | **M5.4** | It is a public page by definition |
| **Remote console** | The thin console (ADR-011) | **No.** M5.5 | **M5.5** | — |
| **Marketing / landing** | — | `apps/marketing-site` exists and is **frozen** | **Deferred by the owner** (2026-09-17) | No roadmap item, no gate. Not started |
| **Email sending** | Resend, from `demiurge.cloud` | **Yes, proven live** to Resend's test address | Already live; the **webhook** endpoint needs QOR ID public | §5 |
| **Resend webhooks** | Bounce and complaint reports | Endpoint **exists in code**, has never received a live delivery | With QOR ID | Resend cannot deliver to a laptop |
| **CI** | GitHub Actions on the public `QOR-MATRIX/demiurge-chain` (ADR-063, ADR-064) | **Jobs execute since 1 October**, in the owner's organisation; on the personal account every run was refused by a billing lock. No run has been seen to finish yet | **Now** | The first runs' results, and any fault the Linux-only steps show |
| **Relays** | Named by the owner | **Not defined in the repository** | Unknown | Scope undefined; a name is reserved, nothing more |
| **Agentic synchronisation** | Named by the owner | **Not defined in the repository** | Unknown | As above |
| **QOR Engine** | Named by the owner | **Not defined in the repository** | Unknown | As above |

**What this means for sequencing.** Only three things are both built and blocked on hosting today: QOR
ID with its database and Redis, a shared devnet, and CI. Everything else in the table is unbuilt, so
hosting it is not yet a question. **Deploy nothing that is not built.**

---

## 2. Domain map

One component per name. A name is created when the component it serves is ready to be deployed, not
before — an unused subdomain that resolves is an invitation to probe it.

### `qorsync.dev` — operations

| Subdomain | Component | Public? | When |
| --- | --- | --- | --- |
| `id.qorsync.dev` | QOR ID: the API **and** its own link pages | Yes | With QOR ID. See §3 and §5 — **this name is contested by the email question** |
| `rpc.qorsync.dev` | Public chain RPC, safe methods only | Yes | M5.1 / M7 |
| `indexer.qorsync.dev` | The SQD indexer's query API | Yes | M5.4 |
| `updates.qorsync.dev` | Launcher update manifests and artifacts | Yes | L6.2 |
| `status.qorsync.dev` | A status page, if one is wanted | Yes | Optional, any time |
| *(no name)* | **Validators** | **No** — §4 | — |
| *(no name)* | **Archive node** | **No.** Private to the indexer, on Fly's private network | M5.4 |
| *(no name)* | **Postgres, Redis** | **No.** Railway's private network only (ADR-063). A temporary TCP proxy on Postgres exists only for the data copy and is removed after it | With QOR ID |
| `relay.qorsync.dev` | Relays | Reserved | Scope undefined |
| `sync.qorsync.dev` | Agentic synchronisation | Reserved | Scope undefined |
| `engine.qorsync.dev` | QOR Engine | Reserved | Scope undefined |

### `demiurge.cloud` — what people look at

| Subdomain | Component | When |
| --- | --- | --- |
| `demiurge.cloud` | Marketing / landing | Deferred by the owner |
| `viewer.demiurge.cloud` | The public viewer (M5.4) | M5.4 |
| `console.demiurge.cloud` | The thin remote console (M5.5) | M5.5 |
| *(sending domain)* | Email envelope and DKIM — **already verified in Resend** | Live |

**The three reserved names are reserved, not planned.** They have no component, no cost line and no
milestone. If a scope is written for any of them, it gets a roadmap item first.

---

## 3. Cross-domain authentication

**The premise in the brief needs one correction, and it makes the problem smaller.**

**QOR ID does not use cookies.** It issues a JWT access token and a refresh token in the response body,
and every authenticated request carries `Authorization: Bearer …`
(`services/qor-auth/src/middleware/auth.rs:76`, `handlers/auth.rs:311`). Access tokens last **15
minutes** and refresh tokens **30 days** (`config.rs:92-93`). So there is no cookie to share and no
`SameSite` problem to solve: a frontend on `demiurge.cloud` can already call `id.qorsync.dev` with a
bearer token, cross-origin, today.

**The launcher is not affected at all.** It keeps both tokens in the **OS keychain**, never on disk and
never in the webview (`tools/qor-launcher/src-tauri/src/identity/mod.rs:19`). It is a native client, not
a browser, so the same-origin policy does not apply to it. Nothing in this section changes the launcher.

### What is actually wrong today

**CORS is open to every origin.** `main.rs:195-199` builds
`CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`. Any website in the world can
make a cross-origin call to QOR ID from a visitor's browser. Bearer tokens make that less dangerous than
cookies would — an attacker's page cannot silently attach a token it does not have — but it also means
**nothing today constrains which frontends may talk to QOR ID**, and there is no list of legitimate
origins to check a redirect against. This has to change before any browser frontend ships, and it is a
tightening, not a feature.

### The gap between "works" and "right"

A browser frontend can hold a bearer token, but the honest options are poor:

| Where a browser frontend keeps tokens | Problem |
| --- | --- |
| `localStorage` | Any XSS on `demiurge.cloud` reads a **30-day refresh token** and takes the account. The launcher avoids this with the keychain; a web page has no keychain |
| In memory only | Survives no reload. Every refresh means signing in again |
| A cookie on `.qorsync.dev` | Cross-site to `demiurge.cloud`, so it needs `SameSite=None; Secure`, and it is not sent on a plain fetch from the other domain anyway |

This is why the pattern is an **identity-provider redirect flow**, which is
[ADR-043](../decisions/ADR-043-qor-id-as-an-identity-provider.md).

### What an identity-provider flow requires, and what QOR ID lacks

The shape: the frontend redirects the browser to QOR ID, the person signs in **on QOR ID's own origin**,
QOR ID redirects back with a short-lived authorization code, and the frontend exchanges that code for
tokens. OAuth 2.1 authorization code with PKCE is the boring choice, and boring is what ADR-001 asks for
in a security-critical path.

**QOR ID has none of the pieces.** Searched on 20 September 2026:

| Piece | Present? |
| --- | --- |
| An `/authorize` endpoint and a sign-in page for it | **No** |
| A client registry (a client id per frontend) | **No.** Sessions belong to an account; there is no notion of *which application* |
| A redirect-URI allowlist | **No** |
| Authorization codes (single-use, short-lived, PKCE-bound) | **No** |
| Refresh tokens bound to a client, and rotated on use | **No.** One refresh token per session, not rotated |
| An origin allowlist for CORS | **No** — `Any`, above |

### What that implies for the current session model

Assessment only. **No code was changed.**

1. **Sessions are per-account, not per-client.** Revoking "the web console's access" is not expressible:
   revocation is per session, and a session does not record what created it. Every frontend added makes
   that coarser.
2. **Sessions record no IP address and no last-activity time** — already a known gap in `SECURITY.md`.
   With one native client that is tolerable; with browser frontends on a second domain, "where is this
   session signed in from?" becomes a question a person will reasonably ask.
3. **A 30-day refresh token is sized for the launcher's keychain, not for a browser.** If browser
   frontends get refresh tokens at all, they should be shorter-lived, rotated on every use, and
   invalidated on reuse. That is a change to the token model, not a configuration value.
4. **There is no client identity to bind a token to**, so a token stolen from one frontend is valid
   everywhere, including against the launcher's endpoints.
5. **`BASE_URL` is one origin.** QOR ID builds its email links from it. If QOR ID serves an authorization
   page as well, that page and the link pages share an origin — which is fine, and is an argument for
   keeping all of QOR ID on one name rather than splitting it across two domains.

**Cost, honestly:** this is a milestone's worth of work in the identity service, not a configuration
change, and it is security-critical. It should not be started until a frontend actually needs it, which
is **M5.4 at the earliest**. What should happen *before* then, and is cheap: replace the open CORS policy
with an allowlist.

---

## 4. Validators and public RPC: the topology

**Validators get no public hostname, no public RPC and no DNS record.** A validator's signing key is the
one thing an attacker wants, and every reachable surface on the machine holding it is a route to it. The
public RPC endpoint is a **different node**, with a different job.

```
                      the internet
                            |
      +---------------------+---------------------+
      |                     |                     |
rpc.qorsync.dev    indexer.qorsync.dev    updates.qorsync.dev
(full node,          (SQD query API)      (static artifacts)
 safe RPC, TLS)              |
      |                      |
      |               +------+------+
      |               |   indexer   |
      |               +------+------+
      |                      |   private network only
      |               +------+------+
      |               | archive node|  --state-pruning archive-canonical
      |               +------+------+  --blocks-pruning archive-canonical
      |                      |
 =====+======================+======================  libp2p p2p mesh
      |                      |                  |
 +----+----+           +-----+---+        +-----+----+
 |validator|           |validator|        | validator|   NO public hostname
 |    A    |           |    B    |        |    C     |   NO RPC exposed
 +---------+           +---------+        +----------+   p2p port only
```

**The rules this topology encodes:**

- **Validators expose exactly one port: the libp2p p2p port.** They need inbound p2p to peer, which
  ADR-015 already accounted for with a dedicated IPv4 per node. They expose no RPC to the internet:
  `--rpc-methods=safe` is not sufficient, because the safest RPC is the one that is not listening. Bind
  RPC to localhost or to Fly's private network only.
- **Validators have no DNS record**, so they are not discoverable by name. Their peer addresses are
  configured, not published.
- **The public RPC node is a full node — not a validator, and not the archive node.** It runs
  `--rpc-external --rpc-methods safe`, behind TLS, rate-limited. It holds no session key. If it is
  overwhelmed, block production is unaffected, which is the entire point of separating it.
- **The archive node is private to the indexer**, which ADR-015's clarification already says. It gets no
  public name, even though it is the most expensive disk in the system.
- **Bootnodes.** Whatever is published as a bootnode address is public by definition. Publish the
  **public RPC node** as the bootnode, never a validator.
- **What this does not fix.** ADR-015 already records that validators all sitting with one provider is
  not decentralisation, and that it is revisited before mainnet. This topology does not change that, and
  it should not be read as making a single-provider validator set acceptable.

---

## 5. Email: where it should send from

**What is true today.** QOR ID sends through Resend from `demiurge.cloud`. The domain is verified and
authorised; DKIM, SPF and DMARC (at `p=none`) all resolve and match, checked in public DNS on
2026-09-15. A live send to Resend's test address worked end to end, and the links verified an address and
reset a password. Delivery to a real inbox is still unconfirmed. Demiurge mail is deliberately never sent
from `vybz.cloud`, because separate projects do not share a sending reputation.

**The question the second domain creates** is not really "where should email send from". It is:

> **A person gets an email from `demiurge.cloud` and clicks a link that opens `id.qorsync.dev`.**

That mismatch is the exact shape of a phishing email. Teaching people that Demiurge's emails link to a
different domain trains them to click through domain changes, which is the habit that makes phishing
work. It also costs deliverability: alignment between the sending domain and the links in the body is
something spam filters weigh.

**So the sender and the link have to match.** Three ways, and only the first two are honest:

| Option | What it means | Cost |
| --- | --- | --- |
| **A. Keep QOR ID's user-facing pages on `demiurge.cloud`** (say `account.demiurge.cloud`), with the API on `id.qorsync.dev` | Sender, link and page all match | QOR ID serves two origins, and `BASE_URL` is the page origin. Conflicts with "all QOR operations on `qorsync.dev`" |
| **B. Move transactional sending to `qorsync.dev`** | Sender, link and page all match, on the operations domain | A second sending domain verified in Resend, with its own DKIM, SPF and DMARC, and its own reputation from zero. People see mail from a domain they may not recognise |
| **C. Accept the mismatch** | Send from `demiurge.cloud`, link to `qorsync.dev` | **Not recommended.** It is a phishing pattern and a deliverability cost, for no benefit |

**Recommendation: A.** The people reading these emails are being asked to click a one-use link that
verifies an address or resets a password, which is exactly the moment to show them nothing surprising.
`demiurge.cloud` is what they will know Demiurge by, it is already verified and proven live, and option A
changes nothing about sending — no new reputation to build, no re-verification, and no risk to a path
that currently works. The operations domain is the right home for QOR ID's **API**; it is the wrong home
for the three pages a non-technical person will ever see.

**This contradicts the brief's placement of QOR ID**, so it is the owner's call. It is listed in §8.

**Either way, unchanged:** DMARC stays at `p=none` until the reports are read; the webhook endpoint still
needs a public HTTPS address before a live bounce can be received; and `RESEND_API_URL` stays unset in
any live environment.

---

## 6. Cost

**These figures are carried from [ADR-015](../decisions/ADR-015-infrastructure-ownership.md), which read
them from vendor documentation on 14 September 2026. They were not re-checked for this plan, and
ADR-015's own instruction stands: check them again before deploying.** Anything marked *estimate* was not
read from a vendor page at all and has to be confirmed.

| Component | Cheapest honest option | Rough monthly | What forces an upgrade |
| --- | --- | --- | --- |
| **QOR ID, Postgres, Redis** | **Railway, usage-based (ADR-063)**, replacing the Fly lines ADR-015 costed (a Machine, **$38** Managed Postgres, a Redis Machine) | **~$5–15 for all three** *(ADR-063's expectation, not yet measured)* | Usage. A Railway usage limit is still to be agreed with the owner (`HANDOFF.md` §4 item 32). Moving sessions into Postgres would delete the Redis service, and is ADR-015's open question |
| **Validator node** (each) | Fly Machine, a volume, and a **dedicated IPv4 at $2** | ~$7–12 each *(estimate)*, plus the $2 | Disk growth; validator count |
| **Public RPC node** | As a validator, but no dedicated IPv4 if it is fronted by TLS | ~$7–12 *(estimate)* | Request volume. Rate-limit before upgrading |
| **Archive node** | Fly Machine and a volume that **only ever grows** | ~$10 plus disk *(estimate)* | **Time.** This is the line that grows without anyone doing anything |
| **SQD indexer** | Fly Machine, writing to the same Postgres | ~$2–5 *(estimate)* | Reindex speed |
| **Frontends** | **Vercel Hobby, free** | **$0** | See trap 1 below |
| **Launcher updates** | Static artifacts: object storage, or the GitHub release CDN | **$0–5** *(estimate)* | Bandwidth, once there are real users |
| **Email** | **Resend free tier** | **$0** | Volume beyond the free allowance |
| **CI** | GitHub Actions minutes | **$0.** The repository is public since 29 September (ADR-063), and Actions is free for public repositories | Nothing, while the repository is public. Runs are blocked until the owner clears the account's billing lock |
| **Domains** | Two registrations | ~$25–40 a year *(estimate)*; `.dev` is not the cheap TLD | — |

**Rough total before anything else is built: about $5–15 a month** on Railway (ADR-063), down from the $50–70 this
plan first estimated, which the $38 Fly Postgres dominated. The node, indexer and archive lines are still ADR-015's
Fly figures and are spent only when those components are built.

### Free-tier traps that break things quietly

In roughly the order they are likely to bite:

1. **Vercel Hobby forbids commercial use.** A free Hobby project is for non-commercial work. A public
   viewer for a platform that moves a currency is very likely commercial in Vercel's terms. This does not
   fail at a threshold — it fails when somebody notices. **Confirm before relying on it.**
2. **Fly Machines stop when idle, by design.** *(QOR ID is on Railway since ADR-063, where a service sleeps only
   if its serverless setting is turned on. `DEPLOY-RAILWAY.md` does not record it either way, so check it stays
   off when the service is first built. The Fly nodes below are unchanged.)* A stopped QOR ID means the **first request after a quiet
   period is slow or times out** — and the person hitting it is someone clicking a verification link from
   an email. Validators and the archive node must **never** be set to auto-stop: a validator that sleeps
   misses its slots, and the two-validator script already shows the chain stops finalising while one of
   two is away. Set `min_machines_running = 1` for anything that must not sleep, and accept the cost.
3. **Postgres connection limits.** *(Written for Fly; on Railway QOR ID's pool is capped at 10,
   `DEPLOY-RAILWAY.md`.)* The smallest plans cap connections, and QOR ID plus an indexer
   plus a migration run can exceed them. The failure is "too many connections" at the worst moment.
   ADR-015's plan includes pooling; use it.
4. **The archive node's disk fills.** It grows with every block, forever, and the failure mode is the
   indexer quietly stopping. That needs an alert on free disk, not a monthly glance.
5. **Resend's free tier has a daily cap as well as a monthly one.** A burst of registrations can hit the
   daily cap while the monthly figure still looks fine, and the symptom is verification emails that never
   arrive.
6. **GitHub Actions minutes, if the repository ever goes private again.** Resolved by ADR-063: the public
   repository's minutes are free. The `two-validators` job's runner time is still **unmeasured**, because no job
   has executed yet; it matters again only if CI moves to a private repository.

---

## 7. `.dev` is HSTS-preloaded: every name needs real HTTPS

**The whole `.dev` top-level domain is on the HSTS preload list**, shipped inside the browsers
themselves. A browser will not make a plain-HTTP request to any `*.qorsync.dev` name, and it will **not
offer a click-through** for a certificate error. There is no "just for development" exception.

What follows:

- **Every `qorsync.dev` name needs a publicly trusted certificate**, including anything called `dev`,
  `staging` or `test`. A self-signed certificate on `dev.qorsync.dev`, opened in a browser, is a hard
  failure with no bypass.
- **Certificates are free and automatic on every provider here.** Railway (QOR ID, ADR-063) and Fly issue and
  renew certificates per hostname once the DNS record points at the service, and Vercel does the same for its
  domains. **Cost: $0.** The work is ordering DNS and issuance correctly, not paying for it.
- **A wildcard (`*.qorsync.dev`) needs DNS-01 validation**, which means an API token for the DNS
  provider. Per-hostname certificates need no such token, so **prefer per-hostname** — which also means a
  name that does not exist has no certificate, consistent with §2's rule that names are created when
  their component is.
- **Non-browser clients are not bound by HSTS**, but they get TLS anyway. The launcher's chain endpoint
  is a WebSocket address, so a deployed public RPC is `wss://rpc.qorsync.dev`, never `ws://`. Nothing in
  the launcher should be pointed at a plain `ws://` address outside a local development node.
- **Certificate expiry is an outage.** Automatic renewal fails quietly when a DNS record moves. Whatever
  monitoring exists should check certificate expiry, not only that a page loads.
- **`demiurge.cloud` is not preloaded**, so it carries no such constraint — one more small argument for
  §5's option A, where the pages a stranger opens from an email live on the domain with the more
  forgiving failure mode.

---

## 8. What the owner has to decide

Separated from what this plan can recommend on its own.

**Blocking — nothing can be deployed without these:**

1. **Where QOR ID's user-facing pages live** (§5). The brief puts all of QOR ID on `qorsync.dev`; this
   plan recommends the **API on `id.qorsync.dev` and the link pages on `account.demiurge.cloud`**, so
   that the email a person receives and the link they click are on the same domain. Accepting the brief
   as written means accepting a sender/link mismatch, which is option C and is not recommended.
2. **Whether email moves to `qorsync.dev`** (option B), which is the other way to make sender and link
   agree. It costs a second verified sending domain and a reputation built from zero.
3. **The Vercel Hobby licence question** (§6, trap 1) — confirm it, or budget for a Pro seat.
4. **Decided 29 September (ADR-063):** whether to pay for Actions minutes or make the repository public. It
   was published as the public `ALaustrup/demiurge-chain`, without history. What remains is the account's
   billing lock, which the owner clears in Settings, Billing and plans.

**Needed before the relevant component ships, not before:**

5. **The subdomain names in §2.** They are proposals. `id`, `rpc`, `indexer` and `updates` are
   conventional; none of them is load-bearing.
6. **Redis or Postgres-backed sessions** — ADR-015's open question, which this plan does not close. It is
   one whole line of cost and one whole service.
7. **Where off-Fly database backups go** — also ADR-015's open question, also still open.
8. **When to start the identity-provider work** (§3, ADR-043). It is a milestone's worth of
   security-critical work, and it is not needed until a browser frontend exists — M5.4 at the earliest.

**Named but undefined — the owner may want to define these, or drop them:**

9. **QOR Engine, relays, agentic synchronisation.** Named in the brief, defined nowhere in the
   repository. Each has a reserved name in §2 and nothing else. **No scope was invented for any of
   them.** If any of them is real, it needs a roadmap item and its own decision before it needs a
   subdomain.

**Not a decision, but do it first:** replace QOR ID's `allow_origin(Any)` with an origin allowlist (§3).
That is a tightening, and it needs no domain to be chosen.
