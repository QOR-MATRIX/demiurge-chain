# ADR-089: An advisory in a crate the launcher never builds

**Status:** **Accepted**, 15 October 2026, by the owner, who chose the recommendation ("Go with best recommendation"):
**option A**. Proposed the same day; every option but D loosens the launcher's dependency audit, and loosening evidence
is the owner's separate call (`docs/GATES.toml`; AGENTS.md §8). **Not yet built:** writing the ignore file was refused
by the assistant's own permission system as a weakening of security, so the change below waits for the owner to allow
it or make it (HANDOFF §4 item 74).

**Relates to:** [ADR-063](ADR-063-public-repository-actions-and-railway.md) (CI), the RUSTSEC-2023-0071 ignore in
`services/qor-auth/.cargo/audit.toml` (approved 14 September 2026 and logged in `docs/GATES.toml`), whose shape option A
copies.

## Context

On 15 October 2026 CI's security job failed on pull request #29 at the step
"Audit launcher host dependencies" (`cargo audit` in `tools/qor-launcher/src-tauri`). It has failed on `main` too, at the same step, in every scheduled run
from 13 October 2026 (and at QOR ID's audit as well on 15 October); the run of 12 October passed. Three new advisories
appeared that week. Two had fixed releases and are fixed: `rustls` 0.23.45 to 0.23.46 (RUSTSEC-2026-0346 to 0349) and `quinn-proto`
0.11.17 to 0.11.18 (RUSTSEC-2026-0338, 0340, 0343): `rustls` in the launcher's, QOR ID's and the chain's lockfiles,
`quinn-proto` in the launcher's. The third
cannot be fixed by an update:

- **RUSTSEC-2026-0344**, `ruzstd` 0.8.3: "Decompression bomb in `ruzstd`: a tiny input can exhaust memory (OOM)".
  Fixed in `ruzstd` 0.9.1.
- `ruzstd` is in the launcher's `Cargo.lock` only as a dependency of `smoldot` 2.2.0, through `smoldot-light` 1.3.2 and
  `subxt-lightclient` 0.51.0, subxt's optional light client. **The launcher never builds it:**
  `cargo tree -i ruzstd --locked --target all --all-features` prints nothing (15 October 2026), and the launcher talks to
  a node over WebSocket (ADR-040), never as a light client. The lockfile records it because Cargo writes optional
  dependencies that a dependency's features could enable, whether or not anything enables them.
- **No update removes it.** Every published `smoldot` (3.0.0, 4.0.0 and 5.0.0 included) requires `ruzstd` `^0.8.0`, and
  subxt 0.51.1, the newest, still depends on `subxt-lightclient`. Only a `smoldot` release on `ruzstd` 0.9 closes it.

`cargo audit` reads the lockfile, not the compiled graph, so it reports a crate that is never built. Until something
changes, the launcher's audit is red, and with it the security job, on every branch.

## Options

**A. Ignore RUSTSEC-2026-0344 in the launcher, on a checked condition (recommended).** Add
`tools/qor-launcher/src-tauri/.cargo/audit.toml` ignoring that one advisory, written like QOR ID's: the advisory, why it
does not apply, the date it was verified, and the condition. Add a CI step beside "Condition of the rsa ignore" that
fails when `cargo tree -i ruzstd --locked --target all -e all` prints anything, so the ignore cannot cover a `ruzstd`
that is built. Log it in `docs/GATES.toml`'s change log. *Costs:* one more ignore, in a crate that until now had none by
the owner's instruction (20 September 2026). *If it changes later:* when a `smoldot` on `ruzstd` 0.9 is published, a
lockfile update removes `ruzstd` 0.8, the condition step reports `ruzstd` gone, and the ignore is deleted; if the
launcher ever enables the light client, the condition step fails first. The audit stays a gate for every other
advisory.

**B. Make the launcher's audit report-only, like the chain's.** `continue-on-error: true` on that step. *Costs:* every
future advisory in the launcher, including ones in code it does build, stops failing CI; this is the larger loosening.
Not recommended.

**C. Audit the compiled graph instead of the lockfile.** Replace `cargo audit` with `cargo deny check advisories`, which
resolves features and skips crates that are never built. *Costs:* a new tool in CI and its configuration (`deny.toml`),
and a different tool from QOR ID's and the helper's audits; it reaches the same answer as A for this advisory without an
ignore. *If it changes later:* moving back is a one-line change. A reasonable alternative to A if the owner prefers no
ignore files at all.

**D. Leave it red.** No change; the security job fails on every branch and on `main` until a `smoldot` release on
`ruzstd` 0.9. *Costs:* a red check that every reader must learn to discount, which hides the next real failure in the
same job. Not recommended.

## Recommendation

A, because it is the narrowest: it silences exactly one advisory, only while a check proves the crate is not built, in
the shape the owner already approved for QOR ID. C is the alternative if the owner would rather keep the launcher free
of ignore files. A comment beside `subxt` in `tools/qor-launcher/src-tauri/Cargo.toml` records the condition either way,
where someone enabling the light client would read it.

## Decision

**Option A**, chosen by the owner on 15 October 2026. `tools/qor-launcher/src-tauri/.cargo/audit.toml` ignores
RUSTSEC-2026-0344 alone, saying why, when it was verified and on what condition. A step in CI's security job,
"Condition of the ruzstd ignore", fails when `cargo tree --locked -i ruzstd --target all -e all` prints anything, and also
when `ruzstd` has left the launcher's `Cargo.lock`, so the ignore is deleted rather than left behind. The ignore is
logged in `docs/GATES.toml`'s change log. Every other advisory still fails the launcher's audit.

This is the launcher's first ignore. The owner's instruction of 20 September 2026 kept it free of them; this decision is
the owner's exception, for this advisory, on this condition.
