# ADR-107: An advisory in a wrapper crate that holds no certificates and is never built

**Status:** **Accepted**, 20 October 2026, by the owner, who chose the recommendation ("Guarded ignore (Recommended)").
Proposed and built the same day. Every option but C loosens the launcher's dependency audit, and loosening evidence is
the owner's separate call (`docs/GATES.toml`; AGENTS.md §8). It decides **no economic value**.

**Relates to:** [ADR-089](ADR-089-an-advisory-in-a-crate-the-launcher-never-builds.md), whose shape this copies: the
launcher's audit ignores one advisory only while a CI step proves it does not apply. The ignores of RUSTSEC-2023-0071
in `services/qor-auth/.cargo/audit.toml` and `services/social/.cargo/audit.toml` have the same shape.

## Context

On 20 October 2026 CI's security job failed on pull request #65, whose change touches none of the audited crates. Three
advisories had been published the day before and that day:

- **RUSTSEC-2026-0382**, `rustls-webpki` 0.103.15: "Certificate verification may accept distrusted intermediates".
  Fixed in 0.103.16.
- **RUSTSEC-2026-0381**, `webpki-roots` 1.0.9, and **RUSTSEC-2026-0383**, `webpki-root-certs` 1.0.9 and 0.26.11:
  "Bundled root certificates allow trust in distrusted emSign intermediates". Fixed in 1.0.10.

Updates fix all of them in QOR ID's, Social's and the launcher core's lockfiles, and all but one in the launcher host's:
`rustls-webpki` 0.103.15 to 0.103.16, `webpki-roots` 1.0.9 to 1.0.10, and `webpki-root-certs` 1.0.9 to 1.0.10. The one
left is **`webpki-root-certs` 0.26.11**:

- **It holds no certificates.** 0.26.11 is a "semver trick" release: its whole library is
  `pub use parent::TLS_SERVER_ROOT_CERTS;`, re-exporting the 1.x crate, which the lockfile now holds at 1.0.10, the
  fixed release. What it would hand a program is the fixed list. `cargo audit` reads versions, not contents, so it
  still reports 0.26.11.
- **It is never built for the launcher.** `rustls-platform-verifier` 0.5.3 depends on it only for `wasm32`.
  `cargo tree --locked -i webpki-root-certs@0.26 -e all --target <t>` prints nothing for Windows, Linux and macOS on
  x86-64 and ARM (20 October 2026), and prints its path for `wasm32-unknown-unknown`. The launcher ships as MSI, NSIS,
  deb, AppImage and dmg; its Android build resolves the core's own lockfile, which does not contain it.
- **No update removes it.** No 0.26 release after 0.26.11 exists. `rustls-platform-verifier` 0.5 requires 0.26;
  `jsonrpsee-client-transport` 0.24.11 requires that verifier; subxt 0.51.1, the newest, requires jsonrpsee 0.24. The
  launcher names jsonrpsee's TLS feature itself for Social's socket (ADR-091), and subxt names the same one.

Until something changes, the launcher's audit is red, and with it the security job, on every branch and on `main`.

## Options

**A. Ignore RUSTSEC-2026-0383 in the launcher, on checked conditions (recommended).** Add it to
`tools/qor-launcher/src-tauri/.cargo/audit.toml` beside ADR-089's ignore, saying why, when it was verified and on what
conditions. A CI step beside "Condition of the ruzstd ignore" fails when the 1.x crate that 0.26 re-exports is older
than 1.0.10, when 0.26 is in the graph of any Windows, Linux or macOS target, or when 0.26 has left the lockfile, so
the ignore is deleted rather than left behind. Log it in `docs/GATES.toml`'s change log. *Costs:* a second ignore in the
launcher. The advisory ID also covers 1.x, so a 1.x below 1.0.10 is not caught by `cargo audit` while the ignore
stands; the step's version check catches it instead. *If it changes later:* when subxt moves to a jsonrpsee on a newer
verifier, a lockfile update removes 0.26 and the step says to delete the ignore.

**B. Leave it red.** No change until upstream moves. *Costs:* a red check that every reader must learn to discount,
which hides the next real failure in the same job; #65 and every later pull request would merge with it failing.

**C. Fix only what updates fix.** Commit the updates, and decide this one later. *Costs:* B's, until then.

## Recommendation

A, for the reasons ADR-089 chose its option A: it silences exactly one advisory, in one crate, only while CI proves that
what the advisory warns of is not in anything the launcher builds. Here the proof is double, since the crate both holds
no certificates of its own and is never compiled.

## Decision

**Option A**, chosen by the owner on 20 October 2026.

1. `tools/qor-launcher/src-tauri/.cargo/audit.toml` ignores RUSTSEC-2026-0383 beside RUSTSEC-2026-0344, saying why,
   when it was verified and on what conditions.
2. CI's security job has a step, "Condition of the webpki-root-certs ignore", that fails when `webpki-root-certs` 0.26
   has left `Cargo.lock`, when the 1.x crate it re-exports is older than 1.0.10, or when
   `cargo tree --locked -i webpki-root-certs@0.26 -e all` prints anything for `x86_64-pc-windows-msvc`,
   `aarch64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin` or
   `aarch64-apple-darwin`. It was seen to fail in each of those ways before it was committed: with
   `wasm32-unknown-unknown` added to the list, with the bar raised to 1.0.11, with the crate's name changed, and on
   the lockfile before the update, which held 1.0.9.
3. A comment beside `jsonrpsee-client-transport` in `tools/qor-launcher/src-tauri/Cargo.toml` records the condition,
   where someone changing the launcher's TLS would read it.
4. The ignore is logged in `docs/GATES.toml`'s change log. Every other advisory still fails the launcher's audit, and
   the same day's other advisories were fixed by updates, not ignored.
