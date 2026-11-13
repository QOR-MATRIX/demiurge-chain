# Where the network and Launcher stand

For Andrew, 13 November 2026. Draft PR #118 on `session/qorsync-domains`
contains the Launcher repair and dependency/workflow patches tested at `97dcb13`.
Service audits keep it unmerged; QOR OS's core pin has not moved.

The Launcher helper uses Cargo, the Rust build tool, to locate its executable.
All four previously failing spawn tests and the new regression pass. Hosted
checks pass 109 host tests with five ignored and 245 core tests with 10 ignored.
Formatting, lints, frontend, design and accessibility checks pass.

Workflow action code and audit/coverage tool versions have reviewed pins.
Checkouts discard credentials. The live Actions allowlist permits nine used
repositories; default tokens are read-only and cannot approve pull requests.
Weekly dependency proposals need review. The two-validator recovery check is
running. Rust 1.100's optional report fails on 66 generated-code lint errors;
required checks pass on pinned Rust 1.98.1.

Core and Launcher audits report zero vulnerability findings; 13 core and 23
host maintenance warnings remain. QOR ID and Social still fail on an SQLite
database dependency. Stable SQLx, their database library, cannot accept the
fixed version; disabling default features retains the affected optional driver.
No exception was added.

Compatible chain patches update certificates and compression. All 213 hosted
tests pass with WebAssembly, portable runtime code, built. Its audit retains
38 vulnerability findings and 26 warnings, down from 44/36. The SDK, the chain's
framework, pins affected logging code exactly. Its pin and runtime source are
unchanged; this is not a deployed network upgrade.

Read-only probes confirm QORSYNC.DEV, QOR ID's database/cache readiness and
runtime 11 with advancing finalized blocks. Vercel reports domain assignments,
but the connected account cannot open the project. The live homepage lacks the
new QOR OS repository link. Social/ARQADE custom hosts and client migration
remain open. No deployment, image, package or release occurred.

Resolve service dependencies and maintenance warnings before a QOR OS pin update
and broader bridge commands, the shell-to-core interface. The only roadmap is
`docs/DIRECTION.md`; `docs/SYSTEMS.md` maps current implementations.
