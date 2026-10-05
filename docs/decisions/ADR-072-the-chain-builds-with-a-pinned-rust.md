# ADR-072: The chain builds with a pinned Rust, 1.98.1, and reports what the newest Rust would say

**Status:** Accepted, 4 October 2026, by the project owner ("go with option A"), choosing option A of
[`../architecture/RUST_TOOLCHAIN.md`](../architecture/RUST_TOOLCHAIN.md).
**Amends:** [ADR-033](ADR-033-dependency-versions.md) rule 2 for `chain/` only: its Rust toolchain no longer follows
"CI actions and tooling stay current"; it moves as rule 3 moves the SDK pin. QOR ID, the launcher and ARQADE keep
following the newest stable Rust.

## Context

Rust 1.99.0 (28 September 2026) made `clippy` flag `clone_on_copy` in code the pinned SDK's `#[pallet::event]` and
`#[pallet::call]` macros generate. The chain's CI job installs the newest stable Rust and lints with `-D warnings`, so it
has failed at Lints on every run since, and its tests have not run in CI. The macros come with `polkadot-stable2606-1`
(ADR-022) and cannot be changed here. The options and their costs are in `RUST_TOOLCHAIN.md`.

## Decision

1. **`chain/rust-toolchain.toml` pins Rust 1.98.1** with `rustfmt`, `clippy` and the `wasm32v1-none` target. Every
   `cargo` run in `chain/` — a developer's and CI's — uses it. The chain's CI jobs install 1.98.1 explicitly as well.
2. **A report job lints the chain with the newest stable Rust on every scheduled and manual run, and never fails the
   run.** It is listed under `[ci].reports` in `GATES.toml`, not under `quality_gates`, and no criterion reads it, as with
   the `cargo audit` of `chain/`. It runs with `SKIP_WASM_BUILD`, because it reports lints and is never evidence.
3. **The pin moves deliberately**, as ADR-033 rule 3 moves the SDK: when the report is clean, when the SDK pin moves,
   or when a Rust security fix requires it — each as a change of its own with the chain's full suite run, without
   `SKIP_WASM_BUILD`.

## Consequences

- The chain's quality gate is exactly the check its evidence was produced under (fmt, clippy `-D warnings` and every
  test with the wasm built, on 1.98.1), and its tests run in CI again.
- New lints stop gating the chain until the pin moves. The report is how they stay visible; it is logged in
  `GATES.toml`'s change log as the evidence-rule change it is, with the owner's approval.
- Rust 1.98.1 receives no further patch releases. If a security fix ships only in a later Rust, rule 3 moves the pin.
