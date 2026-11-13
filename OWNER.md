# Where the network and Launcher stand

For Andrew, 13 November 2026. Repairs are on `session/qorsync-domains` in draft
PR #118. They are not merged, released or deployed.

The Launcher test failure is traced to an assumed Cargo build-folder layout.
Its stand-in product now uses the executable path Cargo reports. The four real
spawn checks remain required, alongside a new path-discovery regression.

The core and Launcher dependencies are patched for the four blocking advisory
findings. Local audits, dependency vulnerability checks, now report zero such
findings, while 13 core and 23 host maintenance warnings remain. QOR ID's network
libraries are patched. QOR ID and Social each still report one SQLite advisory;
the current stable SQLx library cannot accept the fixed SQLite version. No
exception was added. Hosted compilation and tests still need to accept the work.

Compatible chain patches remove nine advisory findings, leaving 44 and 36
warnings. Wasmtime, the WebAssembly runtime, stays on its supported 36.0 line.
The Polkadot SDK pin and consensus source are unchanged. Complete chain tests
with WebAssembly built remain required. QOR OS's core pin has not moved.

QORSYNC.DEV remains the only public domain, with your approved service subdomains.
Launcher Developers links to QOR OS; six frozen web tiles are unavailable.
Retired deployment helpers refuse before changing a host. Stable credential
identifiers are preserved, and GitHub's private vulnerability reporting is on.

Vercel confirms domain assignments, but the connected account cannot access the
assigned project. Social and ARQADE still need verified custom hosts. Source
updates do not migrate installed clients, mail or live services. No image,
package, release or production deployment was published.

The next work is service dependency remediation, hosted acceptance and remaining
warnings before expanding the bridge, the shell-to-core command interface.
`docs/DIRECTION.md` is the plan; `docs/SYSTEMS.md` maps current implementations.
