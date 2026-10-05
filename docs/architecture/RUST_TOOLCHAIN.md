# The chain's CI is red because Rust 1.99 lints code the SDK's macros write

**Decided 4 October 2026: option A** ("go with option A"), recorded as
[ADR-072](../decisions/ADR-072-the-chain-builds-with-a-pinned-rust.md) and logged in `GATES.toml`. Kept as the write-up the
owner decided from. Written the same day, because every way to turn CI green weakens a check a release gate reads.

## What happened

- CI installs whatever Rust is newest (`dtolnay/rust-toolchain@stable`). On 28 September Rust **1.99.0** came out.
- Its `clippy` flags `clone_on_copy` — "you cloned something you could have copied" — **inside code the Polkadot SDK's
  macros generate** for every pallet's events and calls (`#[pallet::event]`, `#[pallet::call]`). Measured on
  this PC with 1.99.0 on 4 October: 36 hits in `pallet-drc369`, 3 in `pallet-validator-set`, and the build stopped
  before reaching the others. None is in code anyone here wrote, and the same code is clean under 1.98.1.
- So the chain job's **Lints** step has failed on `main` since the first run after 28 September (runs `37192481323`,
  `37216392160`, `37259761355`), and its **Tests** step has not run in CI since. QOR ID's and the launcher's jobs
  are unaffected. Locally, on 1.98.1, fmt, clippy and all 143 chain tests pass with the runtime built.

The SDK version is pinned (`polkadot-stable2606-1`, ADR-022) and the macros come with it, so the code cannot be
changed here.

## The options

| | What it is | Cost | What breaks later |
| --- | --- | --- | --- |
| **A. Pin Rust to 1.98.1 for the chain, and report the newest** | `chain/rust-toolchain.toml` and the chain jobs use 1.98.1. A scheduled CI job runs `clippy` on the newest stable and **reports without failing**, as `cargo audit` of `chain/` already does | Small. One file, two lines of CI, one report job | New lints stop gating the chain until the pin is raised. The report keeps them visible, so raising it is a choice, not a surprise |
| **B. Allow `clone_on_copy` in each pallet module** | `#[allow(clippy::clone_on_copy)]` on every `pub mod pallet` | Five lines | The lint also stops catching that mistake in hand-written pallet code, silently and for good |
| **C. Wait** | Leave CI red until a clippy release stops linting macro output | Nothing now | The chain's tests never run in CI meanwhile. A red CI is how real failures go unseen |
| **D. Move to a newer SDK release** | ADR-033 rule 3: a deliberate, tested upgrade | Days. Every pallet, the runtime, the launcher's `sp-core` pin, re-measured metadata | Only fixes it if the newer macros are clean, unknown until tried |

## Recommendation

**A.** It keeps every check exactly as it was when the chain's evidence was produced, and it says so in the open:
the pin is one visible line, and a weekly report shows what the newest Rust would say. Option B hides the lint in code
we write; C leaves the chain untested in CI; D is the right move one day, for its own reasons, not as a fix for this.

If you say yes, it is logged in `GATES.toml` as the evidence-rule change it is, with your approval and the date.
