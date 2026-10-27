# ADR-121: The core's lockfile carries the launcher's ruzstd ignore

**Status:** **Proposed**, 26 October 2026, for the owner. Its option A loosens the launcher core's dependency audit,
and loosening evidence is the owner's separate call (`docs/GATES.toml`; AGENTS.md section 8). The change that moved
the chain client into the core (DIRECTION L13.2) carries option A in a commit of its own, so its CI can run. **That
change must not merge until the owner accepts this record or chooses another option.** It decides **no economic
value**.

**Relates to:** [ADR-089](ADR-089-an-advisory-in-a-crate-the-launcher-never-builds.md), which this extends, and whose
shape [ADR-107](ADR-107-a-wrapper-crate-that-holds-no-certificates.md) and
[ADR-115](ADR-115-an-advisory-in-the-sdks-secp256k1-stack-that-our-code-never-uses.md) copy.
[ADR-104](ADR-104-the-launcher-on-android-on-qt.md) decision 2 moves the Rust host into the core, and
[ADR-040](ADR-040-the-launchers-chain-client-is-subxt.md) makes `subxt` the chain client.
[ADR-120](ADR-120-qor-os-carries-the-device-side-products.md) has QOR OS build the core.

## Context

DIRECTION L13.2 moves the chain client from the launcher's Tauri crate into `tools/qor-launcher/core`, with QOR ID's
client, Social's socket, the Library's index, the Mesh and Qontrol. The core has a `Cargo.lock` of its own, which the
Android build resolves, QOR OS builds from, and CI audits. Before the move it ignored only RUSTSEC-2026-0387
(ADR-115).

With `subxt` in the core, its lockfile holds what the launcher's has held since 15 October 2026:

- **RUSTSEC-2026-0344**, `ruzstd` 0.8.3: a decompression bomb, where a tiny input can exhaust memory. It is fixed only
  on the 0.9 line, and no published `smoldot` uses that line.
- **It is in the lockfile and not in the build.** `ruzstd` comes only through `smoldot`, `smoldot-light` and
  `subxt-lightclient`, which is subxt's optional light client. The core does not enable it, because its chain client
  talks to a node over WebSocket (ADR-040). A lockfile lists every optional dependency, whatever is built.
- **Checked on 26 October 2026, with every feature on**, as the Windows launcher and QOR OS build it:
  `cargo tree --locked -i ruzstd --target all -e all --all-features` in the core prints nothing. With its default
  features, which the Android shell builds, `ruzstd` is not in the graph at all.

So the core's audit now fails on the advisory the launcher's ignores, for the same crate, reached the same way.

The launcher's other subxt-borne ignore, **RUSTSEC-2026-0383** (`webpki-root-certs`, ADR-107), needs nothing here.
The advisory database now lists 0.26.11, the version both lockfiles hold, as unaffected (checked 26 October 2026), so
`cargo audit` does not report it in either. Removing the launcher's ignore of it, with its CI step, would be a
tightening of its own, and is not part of this record.

## Options

**A. Extend ADR-089's guarded ignore to the core's lockfile (recommended).** Add RUSTSEC-2026-0344 to
`tools/qor-launcher/core/.cargo/audit.toml`, saying why, when it was verified and on what condition. CI's step
"Condition of the ruzstd ignore" then checks both lockfiles, the core's with every feature on. It fails when `ruzstd`
enters either compiled graph, and says to delete the ignore when it leaves either lockfile. Log it in
`docs/GATES.toml`'s change log and the core audit's entry in `[ci].quality_gates`. *Costs:* one more ignore, in the
core's file. *If it changes later:* when a `smoldot` on `ruzstd` 0.9 reaches subxt, the step says to delete both
ignores.

**B. Keep the chain client out of the core.** *Costs:* it undoes ADR-104 decision 2 for the module that signs
transactions. The phone (L13.4, L13.5) and QOR OS (ADR-120 decision 3) would need a second chain client, or the Tauri
crate. The Mesh, which signs under its grant through the chain client, could not stay in the core either.

**C. Leave the core's audit red.** *Costs:* a red security job on every branch and on `main`, which hides the next
real failure in the same job.

**D. Make the core's audit a report.** *Costs:* every future advisory in the lockfile the phone and QOR OS build from
would pass unnoticed.

## Recommendation

**A.** It ignores exactly the advisory ADR-089 already ignores, for the same crate reached the same way, only while CI
proves the crate is not built. B undoes a decision the owner made, and C and D hide more than this one advisory.

## Decision

None yet. On acceptance of option A:

1. `tools/qor-launcher/core/.cargo/audit.toml` ignores RUSTSEC-2026-0344 beside RUSTSEC-2026-0387, saying why, when it
   was verified and on what condition.
2. CI's step "Condition of the ruzstd ignore" checks the launcher's and the core's lockfiles, the core's with every
   feature on, and fails in each of the ways option A names.
3. `docs/GATES.toml` logs it, and the core audit's entry in `[ci].quality_gates` says so. Every other advisory still
   fails the core's audit.

The pull request that moves the chain client already holds these three in a commit of their own. If the owner chooses
another option, that commit comes out and the change is reworked to it.

## Consequences

- The core's audit passes, on the condition CI checks every run, as the launcher's has since ADR-089.
- Enabling subxt's light client anywhere builds `ruzstd`, and CI fails until this record and ADR-089 are revisited.
- When a lockfile update removes `ruzstd` 0.8, the step says to delete the ignore in both files.
