# Where the network and Launcher stand

For Andrew, 13 November 2026. Draft PR #118 on `session/qorsync-domains`
contains repairs tested at `97dcb13` and the requested homepage analytics integration.
Service audits keep it unmerged; QOR OS's core pin has not moved.

The Launcher helper locates executable output from Cargo, the Rust build tool.
Its four previously failing spawn tests and new regression pass. Hosted checks pass
109 host tests with five ignored and 245 core tests with 10 ignored, plus
formatting, frontend, design and accessibility checks.

Workflow actions and tools have reviewed pins; checkouts discard credentials.
Default tokens are read-only, and the allowlist permits nine repositories.
All 13 two-validator recovery checks pass on a hosted local test network.
Rust 1.100's optional report fails on 66 generated-code lint errors;
required checks pass on pinned Rust 1.98.1.

Core and Launcher audits report zero vulnerability findings; 13 core and 23
host maintenance warnings remain. QOR ID and Social still fail on an SQLite
database dependency. Stable SQLx, their database library, cannot accept the
fixed version; disabling default features retains the affected optional driver.
No exception was added.

All 213 chain tests pass with WebAssembly, portable runtime code, built.
Compatible dependency patches leave 38 vulnerability findings and 26 warnings.
The chain framework and runtime source are unchanged; no network upgrade occurred.

The homepage now installs pinned Vercel Analytics 2.0.1. Its static build,
33 content checks, six privacy/build tests and dependency audit pass locally.
Only canonical homepage pageviews qualify; URL details are removed, browser
privacy opt-outs are honoured and preview visits are excluded. This is source
integration; live collection is unverified.

Read-only probes confirm QORSYNC.DEV, QOR ID readiness and advancing finalized
blocks. Vercel retains domain assignments but project access still fails after
renewed team approval. The live homepage lacks the new QOR OS repository link.
Social/ARQADE custom hosts and client migration remain open. No production
deployment, image, package publication or release occurred.

Resolve service dependencies and maintenance warnings before a QOR OS pin update
and broader bridge commands, the shell-to-core interface. The only roadmap is
`docs/DIRECTION.md`; `docs/SYSTEMS.md` maps current implementations.
