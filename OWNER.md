# Where the network and Launcher stand

For Andrew, 13 November 2026. Repairs are pushed at `042941c` on
`session/qorsync-domains` in draft PR #118. The service audits keep it unmerged.

The Launcher test helper now uses Cargo, the Rust build tool, to locate its
executable. All four previously failing spawn tests and the new regression pass.
Hosted checks pass 109 host tests with five ignored and 245 core tests with 10
ignored. Formatting and lints pass. The ignored checks remain separate acceptance.

Core and Launcher dependencies are patched for the four blocking vulnerability
findings. Local and hosted audits pass; 13 core and 23 host maintenance warnings
remain. QOR ID's network libraries are patched. QOR ID and Social each still fail
on an SQLite database dependency. Stable SQLx, the services' database library,
cannot accept the fixed version; disabling default features also leaves the
affected optional driver in its lock. No exception was added.

Compatible chain patches remove nine advisory findings, leaving 44 and 36
warnings. Wasmtime, which runs portable code, stays on its 36.0 line. The SDK pin
and consensus source are unchanged. All 213 hosted chain tests pass with
WebAssembly, portable runtime code, built. QOR ID, Social, ARQADE and coverage
jobs also pass. QOR OS's core pin has not moved.

QORSYNC.DEV remains the only public domain, with your approved service subdomains.
Launcher Developers links to QOR OS; six frozen web tiles are unavailable.
Retired deployment helpers refuse before host changes. Stable credential
identifiers remain unchanged. GitHub private vulnerability reporting is enabled.

Vercel confirms domain assignments, but the connected account cannot access the
assigned project. Social and ARQADE still need verified custom hosts. Released
installers were not replaced. No deployment, image, package or release was published.

Resolve service dependencies and maintenance warnings before a validated QOR OS
pin update and broader bridge commands, the shell-to-core interface. The only
roadmap is `docs/DIRECTION.md`; `docs/SYSTEMS.md` maps current implementations.
