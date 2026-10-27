# ADR-122: The active crates take the repository's licence, Apache-2.0

**Status:** **Proposed**, 27 October 2026, for the owner to accept, amend or reject. The six manifests keep `MIT`
until the owner decides. The owner's delegation of 26 October 2026 ("Go with your best and most recommended decisions
for questions moving forward") does not decide this record: ADR-120, accepted the same day under those words, kept the
question for the owner, and on 27 October 2026 the owner answered "proceed with" to a list, written by the assistant,
that said it was still the owner's to decide. Changing the licence a crate declares is the copyright holder's act. It
decides **no economic value**, and would loosen no evidence: no gate, CI step or audit reads a crate's licence.

**Relates to:** [ADR-120](ADR-120-qor-os-carries-the-device-side-products.md), whose Consequences leave this question
to the owner and whose decision 5 has Apache-2.0 travel with the carried files; this record would answer the question
and does not edit ADR-120. [ADR-104](ADR-104-the-launcher-on-android-on-qt.md) makes the core the only code that holds
a key, shared by the Windows and Android launchers. [ADR-091](ADR-091-social.md) puts Social in active scope.

## Context

The repository's licence is the Apache License 2.0: `LICENSE` at the root, whose appendix reads "Copyright 2026
Demiurge"; README.md, which says the owner set it on 21 September 2026; and GitHub, which reports the repository as
Apache-2.0. The chain's workspace (`chain/Cargo.toml`) and `chain/primitives/mesh-proof` declare Apache-2.0.

Six crates in active scope (AGENTS.md section 4) declare `license = "MIT"` in their manifests:

- `tools/qor-launcher/core` (`qor-launcher-core`, not published): the code that holds keys, which the Windows and
  Android launchers build, and which QOR OS builds at a pinned commit (ADR-120 decision 3);
- `tools/qor-launcher/src-tauri` (`qor-launcher`): the Windows launcher;
- `tools/qor-launcher/host-proof` (`qor-host-proof`): the operating-system checks of the launcher's product host;
- `tools/qor-launcher/qontrol-git`: Qontrol's helper, a separate process that links libgit2, shipped in the installer;
- `services/qor-auth` (`qor-auth`): QOR ID;
- `services/social` (`social`, not published): Social.

README.md's Licence section said only frozen and dead packages declare MIT or ISC. For these six that was not so, and
the change that proposes this record corrects README.md to say so. ADR-120 named the first two and left which licence
governs them to the owner.

What reads the field, checked on 27 October 2026:

- **No lockfile.** `Cargo.lock` does not record licences, so changing the field touches no lockfile and no audit.
- **The launcher's bundles.** `tauri.conf.json` sets no `bundle.license`. `tauri-utils` 2.9.3, the version in the
  launcher's lockfile, documents that the licence identifier "to be included in the appropriate bundles" then defaults
  to the crate's field, so bundles that record a licence identifier take `qor-launcher`'s. Whether the Windows
  installer, the only bundle released (an NSIS `setup.exe`), records one was not checked.
- **Nothing in CI or the gates.** No CI step, gate criterion or audit reads a licence. There is no licence checker,
  no `NOTICE` file and no SPDX header in the tree, and `CONTRIBUTING.md` sets no terms for contributions.

What the crates already build with: the core depends on `sp-core` 43.0.0 in every build, and on this repository's
`mesh-proof` with its `chain` feature, which `desktop` turns on. Both declare Apache-2.0 alone, as does
`parity-scale-codec`, which the core uses with `chain`.

Who wrote them: every commit on `main` is by one GitHub identity, the owner's, most co-authored by Claude in the
owner's sessions; all 107 pull requests were opened by the owner, and the repository has no forks. History before the
import commit `e611c99` (29 September 2026) is not in this repository and was not examined.

## Options

**A. The six take Apache-2.0, matching `LICENSE` (recommended).** Set `license = "Apache-2.0"` in the six manifests,
and say so in README.md. Reasons:

- One licence for all active code, matching `LICENSE`, README.md, GitHub, the chain and `mesh-proof`, and the licence
  ADR-120 decision 5 has travel with the carried files.
- On the usual reading, anyone who redistributes a build of the core already carries Apache-2.0's conditions for
  `sp-core`; for the core's own files, Apache-2.0 adds notice of changed files (section 4(b)).
- Apache-2.0 carries an express patent licence (its section 3), which MIT's text does not, over code that holds keys.
  It grants no trademark rights (section 6).
- A contribution to these crates would no longer meet a manifest that says one licence inside a repository that says
  another. Read by its terms, Apache-2.0's section 5 offers a contribution under it unless the contributor says
  otherwise.

*Costs:* six manifests and README.md change. On the usual reading, what was released under MIT stays available under
MIT to whoever has it, and Apache-2.0 asks a little more of a redistributor of these crates' own files than MIT,
chiefly notice of changed files. *If it changes later:* a later record, and the six fields again.

**B. MIT OR Apache-2.0, the Rust ecosystem's common pair.** Most of the core's dependencies offer both. It keeps MIT
open to anyone who relied on it. *Costs:* on the usual reading, the repository would need an MIT text naming a
holder, and a statement of the terms contributions come under. Two regimes in one repository whose rules ask for one
of everything. On the usual reading, a recipient could take MIT and leave the patent licence. The crates are not
published as libraries, so the convention has no audience here.

**C. Keep MIT, recorded as an exception.** *Costs:* the repository holds no MIT text and no MIT copyright line for
these crates, and MIT's text asks for its notice to be kept, so on that reading one would have to be added. The
divergence stays in every document that notes it, and the terms of a contribution stay unclear. No express patent
licence.

**D. Take the core's source out of this public repository.** *Costs:* it contradicts ADR-104 and ADR-120 decision 3,
which keep one public source for the code that holds keys; the open Windows launcher would build on closed code; and,
on the usual reading, what is already released stays released under the terms it came with.

## Recommendation

**A.** It makes the crates say what the owner already set for the repository. On the usual reading, anyone who
redistributes a build of the core already carries Apache-2.0's conditions for `sp-core`, and for the core's own files
Apache-2.0 adds notice of changed files. The reasons above are readings of the licences, not legal advice; counsel can
confirm them, as they can the open question below.

## Decision

None yet. On acceptance of option A:

1. From the commit that accepts it, the six active crates whose manifests say MIT declare `license = "Apache-2.0"`,
   matching `LICENSE`: `qor-launcher-core`, `qor-launcher`, `qor-host-proof`, `qontrol-git`, `qor-auth` and `social`.
2. Earlier commits and published releases, launchers 0.1.10 to 0.1.12 and 0.1.14 to 0.1.17 among them, keep what
   they declared. Nothing is re-released for this.
3. The frozen and dead packages README.md lists stay as published.
4. No `NOTICE` file is added. On the usual reading of Apache-2.0's section 4(d), with none there is no notice text a
   redistributor must carry.
5. README.md's Licence section says so; `docs/blueprints/qontrol.md`, which says libgit2 ships "in a tree whose crates
   are MIT", says Apache-2.0; and both indexes add to ADR-120's status that its licence question is answered here.

If the owner chooses B, C or D instead, this record is amended to it before it is accepted.

## Open, not decided here

**Who holds the copyright.** The tree names the holder differently: `LICENSE` says "Demiurge"; the `authors` field of
`tools/qor-launcher/src-tauri/Cargo.toml` says "Laustrup Group"; `bundle.copyright` in
`tools/qor-launcher/src-tauri/tauri.conf.json` says "Astra Matrix, Inc & Laustrup Group"; and QOR ID's manifest names
an individual as its author. Which entity holds the copyright, and how these lines should read, is the owner's
question, best settled with counsel. This record changes none of them.

**Whose act it is.** Changing the licence a crate declares is the copyright holder's act. Option A rests on the owner
being that holder (every commit on `main` is by the owner's identity), which counsel can confirm along with which
entity it is.

## Consequences

On acceptance of option A:

- Every active crate, the repository's `LICENSE`, README.md and GitHub say the same licence.
- Bundles that record a licence identifier take Apache-2.0 from the launcher's next build, since `tauri.conf.json`
  sets none (`tauri-utils` 2.9.3's documented default); whether the Windows installer records one was not checked. No
  installer is built in the accepting change. The installer carries no copy of `LICENSE`, as before; adding one is a
  later change.
- No lockfile, audit, CI step or gate changes.
- A build of the core from a commit before the accepting change keeps what that commit declared, and one from it on
  takes Apache-2.0. QOR OS takes it when it next moves its pin (ADR-120 decision 3).
- `qontrol-git` links libgit2, which keeps its own licence; the installer ships its text, as before.
- A licence check in CI, such as an allow-list of dependency licences, would be a tightening of its own.
