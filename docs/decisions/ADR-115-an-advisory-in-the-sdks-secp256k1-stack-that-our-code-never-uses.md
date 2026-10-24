# ADR-115: An advisory in the SDK's secp256k1 stack, which no code of ours uses

**Status:** **Accepted**, 24 October 2026, by the owner, who chose the recommendation ("Guarded ignore
(Recommended)"). Proposed and built the same day. Every option but B loosens the dependency audits of QOR ID and the
launcher, and loosening evidence is the owner's separate call (`docs/GATES.toml`; AGENTS.md §8). It decides **no
economic value**.

**Relates to:** [ADR-089](ADR-089-an-advisory-in-a-crate-the-launcher-never-builds.md) and
[ADR-107](ADR-107-a-wrapper-crate-that-holds-no-certificates.md), whose shape this copies: an audit ignores one
advisory only while a CI step proves it does not apply. [ADR-022](ADR-022-pin-polkadot-stable2606-1.md) pins the
Polkadot SDK, and [ADR-033](ADR-033-dependency-versions.md) rule 1 says its versions are not overridden.
[ADR-023](ADR-023-sr25519-with-ecosystem-derivation.md) and [ADR-024](ADR-024-ss58-addresses-prefix-42-until-mainnet.md)
make sr25519 the ecosystem's key scheme and SS58 its addresses.

## Context

On 24 October 2026 CI's nightly run on `main` failed its security job (run 40076288019), as every run would from then
on. One advisory had been published the day before:

- **RUSTSEC-2026-0387**, `crypto-bigint`: "Multiple timing variability issues in operations intended to be
  constant-time". Fixed in 0.7.6, published 23 October 2026. No release on the 0.5 line fixes it.

QOR ID's, the launcher host's and the launcher core's lockfiles hold `crypto-bigint` **0.5.5**, and each is compiled:

- **It arrives only through the pinned SDK.** `sp-core` 43.0.0, which the three pin to the chain's SDK release,
  depends on `k256` 0.13 unconditionally (not behind a feature), `k256` on `ecdsa` 0.16 and `elliptic-curve` 0.13, and
  `elliptic-curve` 0.13 requires `crypto-bigint` 0.5. In each lockfile,
  `cargo tree --locked -i <crate> --target all -e normal,build,dev --depth 1` shows `crypto-bigint` 0.5 depended on
  only by `elliptic-curve`, `elliptic-curve` only by `ecdsa` and `k256`, `ecdsa` only by `k256`, and `k256` only by
  `sp-core` (24 October 2026).
- **No update removes it.** `sp-core` 43.0.0 is the newest release; `k256` 0.14, the first on `crypto-bigint` 0.7, has
  no `sp-core` using it yet. Moving the three off the chain's SDK release would break ADR-022's pin, and a `[patch]`
  is forbidden (ADR-033 rule 1, criterion `alpha.no-dependency-patching`).
- **None of our code reaches what it warns of.** The timing issues matter in secret-key operations, and the only ones
  this chain serves are ECDSA's over secp256k1. QOR ID and the launcher use `sp-core` for sr25519 signatures, the
  derivation of sr25519 keys and SS58 addresses alone. No Rust file in the three crates names `ecdsa`, `k256`,
  `secp256k1` or `elliptic_curve`, nor `sp_core::ecdsa` (24 October 2026). The only mention of "ecdsa" is a file
  name, `id_ecdsa`, in the launcher's list of SSH keys a project must not commit.

Social's lockfile and Qontrol's helper's do not hold `crypto-bigint`. `chain/` holds 0.5.5 too, and its audit is a
report that does not fail the run (SECURITY.md, "Dependency advisories in `chain/`").

## Options

**A. Ignore RUSTSEC-2026-0387 in the three, on checked conditions (recommended).** Add it to
`services/qor-auth/.cargo/audit.toml` and `tools/qor-launcher/src-tauri/.cargo/audit.toml`, and give the launcher core
the file it has not needed until now, each saying why, when it was verified and on what conditions. One CI step
before the audits fails when code in any of the three uses ECDSA or secp256k1; when any crate but the next link depends
on `crypto-bigint` 0.5, `elliptic-curve`, `ecdsa` or `k256`; when a `crypto-bigint` below 0.7.6 joins a lockfile
beside 0.5, which the same advisory ID would otherwise hide; or once 0.5 leaves a lockfile, so the ignore is deleted
rather than left behind. Log it in `docs/GATES.toml`'s change log. *Costs:* one more ignore in two files, and a file
of its own in the core. *If it changes later:* when an SDK release moves `sp-core` to `k256` 0.14 and the pin follows
it, the step says to delete the ignores.

**B. Leave it red.** No change until upstream moves. *Costs:* a red security job on every branch and on `main`, which
every reader must learn to discount, and which hides the next real failure in the same job.

**C. Make the three audits reports.** As `chain/`'s is. *Costs:* every future advisory in QOR ID and the launcher
would pass unnoticed, not this one alone.

## Recommendation

A, for the reasons ADR-089 and ADR-107 chose theirs: it silences exactly one advisory, in one crate, only while CI
proves that what the advisory warns of is in nothing our code does, and only while nothing else comes to depend on it.

## Decision

**Option A**, chosen by the owner on 24 October 2026.

1. `services/qor-auth/.cargo/audit.toml` and `tools/qor-launcher/src-tauri/.cargo/audit.toml` ignore RUSTSEC-2026-0387
   beside their earlier ignores, and `tools/qor-launcher/core/.cargo/audit.toml` is created to ignore it alone, each
   saying why, when it was verified and on what conditions.
2. CI's security job has a step, "Condition of the crypto-bigint ignore", that fails in each of the ways option A
   names. It was seen to fail each way before it was committed: ECDSA used on one line and in a `use` list over
   lines, a changed set of dependents, `crypto-bigint` 0.5 gone from the lockfiles, and 0.7.2 beside it.
3. A comment beside `sp-core` in each crate's `Cargo.toml` records the condition, where someone reaching for
   `sp_core::ecdsa` would read it.
4. The ignore is logged in `docs/GATES.toml`'s change log and the three audits' entries in `[ci].quality_gates`.
   Every other advisory still fails the three audits.

## Consequences

- The security job passes again, on the condition the step checks every run.
- A change that brings ECDSA into QOR ID or the launcher, a signer or a recovery of an Ethereum-style address among
  them, fails CI until this record is revisited; the step names the files.
- `chain/` carries the advisory in its report (SECURITY.md lists it with the advisories not yet measured there).
- When the SDK's `sp-core` moves to `k256` 0.14 and the pin follows, the ignores go, and the step says so.
