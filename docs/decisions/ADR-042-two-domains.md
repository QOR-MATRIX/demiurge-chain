# ADR-042: Two domains — `qorsync.dev` for operations, `retired-public-host` for what people see

> Editorial note, 13 November 2026: retired host/contact literals are redacted. Historical claims retain their original dates and are not current deployment evidence. ADR-130 establishes QORSYNC.DEV as the only public domain.

**Current scope:** The two-domain policy is superseded by [ADR-130](ADR-130-qorsync-is-the-public-domain.md), 13 November 2026. The historical decision below is retained.

**Status:** **Accepted**, 28 September 2026, by the project owner (proposed 20 September 2026). On acceptance the
`qorsync.dev` zone held only Vercel's own managed defaults (a catch-all `*` and the apex, both routing to Vercel,
and CAA records allowing Let's Encrypt, Sectigo and Google); no record had been created. The first name made under
it is `ci.qorsync.dev` (ADR-058). The
plan behind it is [`../architecture/HOSTING.md`](../architecture/HOSTING.md).

**Relates to:** [ADR-015](ADR-015-infrastructure-ownership.md), whose 15 September 2026 clarification put
QOR ID's pages on a subdomain of `retired-public-host`. If this record is accepted, that clarification is
amended — and decision 3 below says exactly how, because the amendment is not the obvious one.

## Context

The owner acquired a second domain and named the split on 20 September 2026:

- **`qorsync.dev`** — all QOR operations: QOR ID and auth, the development backend, relays, agentic
  synchronisation, public chain RPC, and the launcher's backend needs such as update endpoints.
- **`retired-public-host`** — the user-facing frontends: dashboards and visual experiences.

ADR-015 chose the providers and is not reopened here. What it did not anticipate is a second domain, and
a second domain changes three things that a provider choice does not touch: where a person's browser
sends them, what a person sees in an email, and which names exist for an attacker to find.

Two facts from the tree shape this, both verified on 20 September 2026:

- **QOR ID authenticates with bearer tokens, not cookies** (`middleware/auth.rs:76`). Splitting the
  surface across two registrable domains therefore breaks no session, because there is no cookie whose
  scope a domain boundary would cut.
- **QOR ID sends email from `retired-public-host`**, verified in Resend and proven live on 2026-09-15.

## Decision

1. **`qorsync.dev` is the operations domain.** QOR ID's API, the public chain RPC, the indexer's query
   API and the launcher's update endpoint live there, one component per subdomain, as mapped in
   `HOSTING.md` §2.

2. **`retired-public-host` is the domain for what a person looks at.** The public viewer (M5.4), the remote
   console (M5.5) and any marketing surface live there.

3. **QOR ID's three user-facing pages stay on `retired-public-host`, while its API moves to
   `id.qorsync.dev`.** The pages its emails open — address verification, confirmation of a new address,
   password reset — are served on `retired-public-host`, and `BASE_URL` is that origin.

   **This is a deliberate exception to decision 1, and it is the part of this record most likely to be
   rejected.** The reason is in `HOSTING.md` §5: email is sent from `retired-public-host`, and an email whose
   sender and link are on different domains is the exact shape of a phishing message. Whichever way the
   owner resolves it, **the sender domain and the link domain must be the same domain**. Option B in
   that section — moving transactional sending to `qorsync.dev` and putting all of QOR ID there — meets
   that requirement equally well and is a coherent alternative; it costs a second verified sending domain
   and a sending reputation built from zero. **Option C, accepting the mismatch, is rejected.**

4. **A subdomain is created when the component it serves is ready to deploy**, not in advance. A name
   that resolves to something unfinished is a surface to probe, and a name reserved in a document costs
   nothing.

5. **`relay.qorsync.dev`, `sync.qorsync.dev` and `engine.qorsync.dev` are reserved names with no
   scope.** Relays, agentic synchronisation and **QOR Engine** were named by the owner and are defined
   nowhere in this repository. **No scope is invented for them here.** Each needs a roadmap item and its
   own decision before it needs a subdomain.

6. **Every `qorsync.dev` name gets a publicly trusted certificate, including development ones.** The
   `.dev` top-level domain is HSTS-preloaded in the browsers, so plain HTTP is refused and a certificate
   error has no click-through. Per-hostname certificates rather than a wildcard, so that no DNS API token
   has to exist (`HOSTING.md` §7).

## Consequences

- **ADR-015's clarification is amended, not reversed.** Its rule that QOR ID serves its own pages
  directly, with no proxy or rewrite between a person and a one-use token, is kept exactly. What changes
  is which domain those pages sit on, and that QOR ID's API is addressed separately.
- **QOR ID serves two origins** under decision 3. That is a real cost: two certificates, two names in
  configuration, and a CORS allowlist that has to name both. It buys an email in which the sender, the
  link and the page all agree.
- **The launcher is unaffected.** It holds tokens in the OS keychain and talks to whatever endpoints it
  is configured with; a domain change is a configuration change to it, not a redesign.
- **Nothing is deployed by this record.** It decides where things will go, not that anything goes there.
  What is deployed first, and when, remains a separate decision, as ADR-015 already says.
- **The number of names is a security surface.** Decision 4 keeps it minimal. The cost is a small piece
  of work at each deployment instead of one large DNS setup.
- **If the owner takes option B instead**, decisions 1, 2, 4, 5 and 6 stand unchanged; only decision 3 is
  replaced, and a new sending domain has to be verified in Resend with its own DKIM, SPF and DMARC before
  any email is sent from it.

## What this record does not decide

- Which subdomain names are final (`HOSTING.md` §8, item 5).
- Redis versus Postgres-backed sessions, and where off-Fly backups go. Both are ADR-015's open questions
  and both are still open.
- Anything about mainnet validator hosting, which ADR-015 places outside its own scope.
