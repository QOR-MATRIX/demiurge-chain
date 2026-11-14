# Where the network and Launcher stand

For Andrew, 13 November 2026. Draft PR #118 on `session/qorsync-domains`
contains the tested network/Launcher repairs and homepage ownership handoff.
Service audits keep it unmerged; QOR OS's core pin has not moved.

The Launcher reads executable locations from Cargo, the Rust build tool.
Its four formerly failing spawn tests and new regression pass. Hosted checks
pass 109 host tests with five ignored and 245 core tests with 10 ignored,
plus formatting, frontend, design and accessibility checks.

Workflow actions and tools have reviewed pins; checkouts discard credentials.
Default tokens are read-only; the allowlist permits nine action repositories.
All 13 two-validator recovery checks pass on a hosted local test network.
Rust 1.100's optional report fails on 66 generated-code lint errors;
required checks pass on pinned Rust 1.98.1.

Core and Launcher audits report zero vulnerability findings; 13 core and
23 host maintenance warnings remain. QOR ID and Social still fail on an SQLite
database dependency. Stable SQLx, their database library, cannot resolve the
fixed version. No exception was added.

All 213 chain tests pass with WebAssembly, portable runtime code, built.
Compatible dependency patches leave 38 vulnerability findings and 26 warnings.
The chain framework and runtime source are unchanged; no network upgrade occurred.

The homepage is now maintained in QOR-MATRIX/QOR-OS, with its Analytics package,
tests and licences. This copy is frozen. The existing Astra Matrix / qor project
now builds QOR OS's main branch and QORSYNC.DEV serves it. Links, redirect,
security headers and Analytics script delivery pass live checks. QOR OS PR #9 is
deployed at `f4c9863`; all 47 public files match reviewed bytes with the proxy
header correction in place. Dashboard collection remains unproven. This session
validates and commits the pending upstream ownership handoff; no service code changes.

QOR ID readiness and advancing finalized blocks pass dated read-only probes.
Social/ARQADE custom hosts and client migration remain open. The homepage was
deployed; no network upgrade, device image or package release was published.

Resolve service dependencies before a core pin update and broader bridge commands,
the shell-to-core interface. DIRECTION is the roadmap; SYSTEMS maps ownership.
