# Homepage ownership has moved

The canonical QORSYNC.DEV homepage is maintained in
[QOR-MATRIX/QOR-OS/site](https://github.com/QOR-MATRIX/QOR-OS/tree/main/site).
This copy is frozen under [ADR-132](../docs/decisions/ADR-132-the-homepage-moves-to-qor-os.md).
Do not develop or deploy it. QOR OS imported its source at `d4c51d7` with hashes
and original licences; this historical source remains for provenance.

The existing [Astra Matrix / QOR project](https://vercel.com/astramatrix/qor)
now reads QOR OS, branch `main`, root `site`. Its
[hosting guide](https://github.com/QOR-MATRIX/QOR-OS/blob/main/docs/HOSTING.md)
owns installation, build, privacy tests, deployment and live verification.
Repository access is required. The Git deployment switch here is disabled;
homepage CI and dependency updates belong to QOR OS.

Network services, the shared core and existing signed Launcher/Mobile release
assets remain in demiurge-chain. Their ownership is not changed by this transfer.
