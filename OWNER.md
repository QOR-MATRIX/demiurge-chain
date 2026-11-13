# Where the network and Launcher stand

For Andrew, 13 November 2026. Draft PR #118 on `session/qorsync-domains`
contains the tested Launcher repair and chain dependency patches at `3d71009`.
The service audits keep it unmerged; the QOR OS core pin has not moved.

The Launcher test helper uses Cargo, the Rust build tool, to locate its
executable. All four previously failing spawn tests and the new regression pass.
Hosted checks pass 109 host tests with five ignored and 245 core tests with 10
ignored. Formatting, lints and frontend checks pass. Ignored checks remain open.

The workflow follow-up pins action code and tool versions, disables saved
checkout credentials and adds weekly dependency proposals. Its hosted checks
are pending; publication stays manual.

Core and Launcher audits report zero vulnerability findings; 13 core and 23 host
maintenance warnings remain. QOR ID's network dependencies are patched. QOR ID
and Social each still fail on an SQLite database dependency. Stable SQLx, the
services' database library, cannot accept the fixed version; disabling default
features also retains the affected optional driver. No exception was added.

Compatible chain patches update certificate validation, trusted roots and
compression. All 213 hosted workspace tests pass with WebAssembly, portable
runtime code, built. Its audit retains 38 vulnerability findings and 26 warnings,
down from 44/36. The SDK, the chain's framework, pins the affected logging
dependency exactly; its upgrade was refused. The framework pin and runtime
source remain unchanged. This is not a deployed network upgrade.

QORSYNC.DEV remains the public domain with approved service subdomains. Launcher
Developers links to QOR OS; six frozen web tiles are unavailable. Retired
deployment helpers refuse before host changes. Stable credential identifiers
are unchanged. GitHub private vulnerability reporting is enabled.

Vercel returned domain assignments, but the connected account cannot open the
assigned project. Social and ARQADE still need verified custom hosts. Released
installers were not replaced. No deployment, image, package or release occurred.

Resolve service dependencies and maintenance warnings before a validated QOR OS
pin update and broader bridge commands, the shell-to-core interface. The only
roadmap is `docs/DIRECTION.md`; `docs/SYSTEMS.md` maps current implementations.
