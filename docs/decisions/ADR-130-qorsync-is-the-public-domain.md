# ADR-130: QORSYNC.DEV is the only public domain

**Status:** Accepted, 13 November 2026, on the owner's instruction to retire all
older public domains. The owner explicitly retained service subdomains beneath
QORSYNC.DEV. This supersedes ADR-042's two-domain split and ADR-100's retired-domain
redirect requirement; it does not change protocol, economic or licensing decisions.

## Decision

1. Public first-party copy and links use `https://qorsync.dev` and approved service
   subdomains. The QOR OS project link is
   [QOR-MATRIX/QOR-OS](https://github.com/QOR-MATRIX/QOR-OS). This repository remains
   the source of the network and Launcher; source, release and licence links must
   continue to name their actual repository. Third-party documentation is not a
   first-party service.
2. Remove retired public-host literals from tracked text, including examples and
   historical records. A historical redaction says so; it does not rewrite what
   was deployed at an earlier date. Do not rewrite git history, application or
   keychain identifiers, cryptographic separators or copyright attribution.
3. Frozen clients stay frozen. Their old protocol examples/defaults use loopback
   rather than sending legacy requests to today's public services. Obsolete
   deployment helpers refuse before host changes. This neither restores their
   compatibility nor authorises publishing a replacement stack.
4. The Launcher does not advertise frozen web surfaces as working destinations.
   Its Developers entry links to the QOR OS developer guide and explains that
   repository access is required. Existing appearance and interaction conventions
   are retained. Current core defaults for QOR ID and RPC already use QORSYNC.DEV.
5. Source cleanup is separate from live migration. The Social provider hostname,
   ARQADE provider hostname, existing installed settings and released binaries
   need a coordinated service-domain/release migration. Do not invent a working
   host, mail sender, redirect or replacement download. `HOSTING.md` records the
   observed provider state and the outstanding verification.
6. CI checks tracked text for retired hosts without printing matched content or
   contact details. The test suite covers encoded links, allowed identifiers and
   failure diagnostics. Keep secrets and restricted configuration out of scans.

## Evidence and limits

The source work starts from `74a1014`. QOR OS separately implements the same
policy in QOR-0026. On 13 November the connected Vercel account returned verified
domain assignments for the apex and `www`, and Cloudflare nameservers; its
project endpoint returned 404. This is not proof that the page or its service
APIs are operational. No DNS, mail, deployment, installed settings, credentials
or release artifacts were changed by this source maintenance.

The QOR OS pinned-core audit also found four vulnerability advisories and 13
warnings in versions still present at this upstream revision. Dependency
remediation remains ahead of new Launcher/DRC-369 capabilities; no advisory
exception or gate relaxation is part of this decision.
