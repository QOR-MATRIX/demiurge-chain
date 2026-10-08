# Qontrol

**Status:** Blueprint, 21 September 2026. Intent, except where it names code that landed today.

## What it is

Qontrol is version control for people who make things, not only for people who write code: it
remembers every version of a project, shows what changed in words the creator already uses, and never
loses a take. On disk it is an ordinary git repository, so a collaborator who has never heard of
Qontrol can clone the folder and work in it with plain `git`. It lays out a new project correctly for
its kind — code, music or game — so the first commit is already the right shape, not a repository
full of render output nobody can clone a week later.

## Who it is for

- **The solo creator who has lost work.** Producers with `track_final_FINAL_v3_real.als`, artists with
  a folder of dated copies, game developers whose engine cache ate their project.
- **Small teams of two to five** who need to know who changed what and cannot run a Perforce server.
- **Developers**, who get a real git repository and lose nothing.
- **Agents**, later — under a delegation that is scoped, capped and revocable (M5.2, ADR-026).

The first two groups are the reason Qontrol exists. Git already serves the third.

## The foundation, and why

**The foundation is git's on-disk object model, unchanged.** Blobs, trees, commits, refs, the index,
`.gitignore`, `.gitattributes`. Qontrol writes the bytes git writes, honours the repository's own
config, and leaves a folder that `git status`, `git log` and any hosting service understand.

**The implementation is split, deliberately:**

- **gitoxide (`gix` 0.87) for every read path** — status, log, diff, branches, rev-walk. Pure Rust,
  fast on the thousands-of-files repositories that game and music projects actually are, and it links
  into the launcher host without bringing a C parser with it.
- **libgit2 (`git2` 0.21) for staging and write-tree only**, in a separate process. Nothing else.

The split is evidence, not taste. gitoxide's own `crate-status.md`, checked on 21 September 2026
against `gix` 0.87.1, marks `add files with .gitignore handling`, `tree from index` and `add and
remove entries` unimplemented, while marking `status`, `rev-walk`, diff and `create new commit from
tree` done. **The gap is staging, not push or rebase** — and staging sits in the middle of the only
chain that matters: stage → index → write-tree → commit.

**Alternatives rejected, each with its reason:**

| Rejected | Why it lost |
| --- | --- |
| **Invent an object model** | ADR-001. A version-control object model is the textbook silent-failure surface: a wrong tree or a dropped entry does not throw, it quietly loses a take and the creator finds out in three weeks. `DIRECTION.md` §2 puts storage layout in the "stay boring on purpose" column. The budget goes to the creator-economy layer above it. |
| **gitoxide alone, writing commits around the index** | It works — `write_blob` → tree `Editor` → `commit()` — and it leaves `.git/index` stale. The next plain `git status` in that folder compares HEAD against a stale index and reports files as modified that Qontrol committed correctly. The objects are right and the working state lies. That is exactly the failure ADR-001 says not to build. |
| **libgit2 everywhere, linked into the launcher host** | The host holds the vault. Qontrol opens folders the user picks, and later repositories from strangers, so it parses attacker-influenced input. libgit2 is a large C library with a parser history, and that is not what the process holding the signing keys should parse. |
| **Shell out to `git`** | `git` is absent from a clean Windows machine, which breaks L6.1's signed-installer story. And it inverts the launcher's security posture: the webview's capability file says in as many words that it may not spawn processes, and routine commits would mean the host spawning a `PATH`-resolved binary that runs user-controlled hooks. |
| **git-lfs** | Whole-file versioning with no delta or chunk reuse, a 2 GiB per-object ceiling on GitHub, a required batch-API server, and remote objects that cannot be deleted except by deleting the repository. One accidental 40 GB render is permanent. Its one good idea — enforced exclusive locks — is kept as a design goal. |
| **Perforce-style centralised server** | The correct model for large art teams, and it abandons git-on-disk, which is the one constraint Qontrol is built under. |
| **git-annex** | Genuinely distributed, and its symlink model is bad on Windows, the launcher's primary platform. The v7 unlocked-file workaround is fiddly, and that complexity is the standing complaint against it. |

**On linking libgit2 into the host: the sidecar was built, not the temporary link.** The budget rule
was that if the sidecar cost more than the rest of Phase 4, a dated temporary link was acceptable with
its expiry tied to the app-host ADR's first item. It did not: one crate and two commands. So there is
no dated exception to track, and the app-host record, ADR-046 (accepted by the owner on 28 September
2026), opens with one obligation already discharged in miniature.

## What is new

The object model is not the innovation. These four are.

**1. The interface.** Git's model is fine; its vocabulary assumes you already understand it.

- **No staging area in the default view.** Stage-on-commit, presented as *"these files will be
  saved"*. The index still exists on disk, exactly as git expects; the creator never meets it.
- **Plain words in place of porcelain:** `modified`, `added`, `removed`, `untracked`, `moved`,
  `conflicted`. A creator reads this surface, and `??` means nothing to them. **Branches are takes,
  versions or variants** in the copy, and ordinary git refs on disk, so a developer can check them out.
- **"What changed" for a session file.** An Ableton `.als` is gzip-compressed XML. Decompress it, diff
  it structurally, and say *"3 tracks added, tempo 124 → 128, Drum Rack device removed"*. No git
  interface does this, every producer wants it, and it is cheap because the format is documented.
- **A/B comparison for binaries.** Once both versions' chunks are in the local store, an audio A/B
  player or an image swipe falls out of the storage design rather than being an extra system.

**2. Scaffolds per project type.** The difference between "git works for music" and "git ate your
session" is one `.gitattributes` line and one ignore rule. Shipping the correct ones per discipline
*is* the product. Concretely in Architecture, below.

**3. Large binaries, handled without a vendor.** Content-defined chunking into a local
content-addressed store, with a small text pointer committed to git — so `git clone` still works and a
developer who has never heard of Qontrol gets a working repository with placeholder files. The pointer
carries the whole-file fingerprint the chain will store for a DRC-369 asset: one hash, two uses.

**4. DRC-369 and CGT, at the right grain.**

- **A commit is not a mint, and the product says so.** Every commit is free, local and offline.
  Minting is a separate, explicit act. Getting this wrong would make every save a transaction, which
  is a worse product and would need a fee class nobody has decided.
- **Publishing a version mints or revises a DRC-369 asset** whose content fingerprint is already in
  the pointer file. **A fork is the remix case:** its published asset names its parent, which is
  DRC-369 nesting and remix royalties (M4.2). That is the one place Qontrol earns a chain feature
  rather than consuming one — provenance for work in progress, not only for a finished file.
- **Qontrol sets no economic value.** Not the perpetual issuance rate (OPEN-1), not the genesis
  allocation split (OPEN-2), not a burn share or a fee class (OPEN-4, U-4). There is no fee on commit,
  push or lock, because a per-commit or per-lock fee would be a new fee class, and fee classes are
  undecided. What Mesh hosting of a side store costs, and what access-gating a private repository
  costs, are equally undecided; the surface quotes a cost only when the chain can quote one.
- **The spend test (ADR-002), stated.** Every CGT path here buys something used: storage served,
  access granted, a remix settled with the creator it came from. If CGT could never be traded for
  dollars, each still buys a service the creator wants. Both sinks — Mesh hosting and access gating —
  are already among the five named in `DIRECTION.md` §3 (ADR-006). Qontrol adds no sixth.

## Architecture

**One port, one set of types.** The launcher-side interface is `open_or_init`, `scaffold`, `read`,
`diff`, `commit`, and later `branch`, `checkout`, `publish`. The gitoxide/libgit2 seam is invisible
above it and reversible: the day gitoxide lands `tree from index` and `add`, the sidecar is deleted
and nothing above the port changes.

**The commit path, end to end:**

```text
Projects surface ──► Qontrol port (host process)
  read    status, rev-walk, diff, branches         [gitoxide, in-process]
  commit  1. stage_all  ──► qontrol-git sidecar    [libgit2, separate process]
          2. write_tree ──► qontrol-git sidecar    [libgit2, separate process]
          3. commit(tree, parents, message)        [gitoxide, in-process]
             → writes the commit, moves the ref, writes a reflog entry
```

Because libgit2 wrote the index and gitoxide committed the tree that index produced, `.git/index`
matches the new HEAD. That is the whole point of the split.

**The sidecar protocol** is one JSON request per line on stdin, one response per line on stdout,
line-delimited because a person needs to read a transcript when it misbehaves:

```text
{"op":"stage_all","repo":"/path"}   -> {"ok":true,"staged":3}
{"op":"write_tree","repo":"/path"}  -> {"ok":true,"tree":"<40 hex>"}
{"op":"ping"}                       -> {"ok":true}
```

Since 26 September 2026 (P1.2) it also answers `stage` (named files only), `plan`, `switch` and `discard`
(`qontrol-git/src/main.rs`). Every failure answers `{"ok":false,"error":"…"}` and the process stays up; it
exits when stdin closes.
The sidecar holds no keys, opens no network, and is built with `default-features = false` plus
`vendored-libgit2`, so OpenSSL and libssh2 are absent entirely and a distribution's system libgit2
cannot silently substitute itself — which matters for a reproducible signed installer (L6.1).

**The repository's own rules are honoured, as git would.** Staging goes through libgit2's `add_all`,
which consults `.gitignore`, `.gitattributes`, `core.autocrlf` and case sensitivity. On Windows that
is the difference between a clean tree and one that lies. Every scaffold's `.gitattributes` opens with
`* text=auto eol=lf`.

**Three tests, written to fail first** (`tools/qor-launcher/src-tauri/src/qontrol/tests.rs`):

1. `after_a_commit_the_working_tree_is_clean` — gitoxide's status reports nothing outstanding.
2. `git_itself_agrees_the_tree_is_clean` — where `git` is installed, `git status --porcelain` prints
   nothing either, so the lying index is pinned against the implementation that matters to a creator.
3. `the_two_ignore_implementations_agree` — against a fixture `.gitignore`, a path gitoxide does not
   surface as untracked is also absent from what libgit2 staged, and the reverse.

**The scaffolds, concretely.** Every kind gets the same spine, then differs in folders and rules:

```text
<project>/
  qontrol.toml      # committed: kind, fingerprint algorithm (provisional), side-store rules,
                    #            repo id (placeholder — see the gaps)
  .gitattributes    # committed: line endings, binary and unmergeable declarations
  .gitignore        # committed
  .qontrol/         # ignored: cas/, locks/, cache/
  README.md         # committed: what each folder is for, in plain words
```

| Kind | Folders | Ignored | Attributes that matter |
| --- | --- | --- | --- |
| **Code** | `src/ tests/ docs/` | `target/ node_modules/ dist/ build/ .env .env.* *.log .vs/ .idea/ .DS_Store Thumbs.db` | `* text=auto eol=lf`; `*.png *.ico binary`. Side store **off**, threshold high — a scaffold that silently diverts source files is a bug. |
| **Music** | `sessions/ stems/ samples/ renders/ reference/ notes/` | `renders/ reference/ *.asd *.reapeaks *.pkf *.sfk Backup/ "Freeze Files/" "Project Info/" *.als.bak *.rpp-bak` | `*.rpp text eol=lf` (Reaper projects are plain text: they diff and merge); `*.als *.flp binary`; `*.wav *.aiff binary -diff -merge`; `*.mid binary` |
| **Game** | `scenes/ assets/ scripts/ audio/ builds/` | `builds/ .godot/ .import/ *.translation export_presets.cfg *.tmp` | `*.tscn *.tres *.gd text eol=lf` — keeping scenes text is the whole argument; `*.psd *.blend binary -diff -merge`; `*.fbx *.png binary`; `*.wav binary -diff -merge` |

Each `README.md` says why: *"`renders/` is ignored on purpose — a render is made from the session and
the stems, so it can be made again. A stem cannot."* **The most dangerous rule in a music scaffold is
an ignore rule that drops recorded audio.** Ableton writes recordings into the project folder; Logic
writes into `Audio Files/`. Those are never ignored: ignoring an audio folder destroys a take.

**Game is Godot-first** — written when QOR Engine was planned on Godot, a plan ADR-081 withdrew; QQ, the QOR Engine, keeps its scenes in the same `scenes/` folder (ADR-083), and `.tscn` is still the precedent: a text
scene format that is diffable and mergeable, which is what makes a game project reviewable at all.
**Unity and Unreal are named variants**, each needing a rule the ignore file cannot express — Unity's
`.meta` files are always committed and Asset Serialization Mode must be Force Text, or nothing in the
project can be reviewed; Unreal's `*_BuiltData.uasset` is regenerable and enormous. The `-merge` flag
on `.uasset`, `.umap` and `.psd` is the honest half: those files cannot be merged, so git must refuse
rather than corrupt them. The other half is an exclusive lock, which `qontrol.toml` declares so the
surface can refuse an edit before it happens.

**The pointer file, for later.** With a side store, a large file is replaced in git by a text pointer:

```text
version     https://qontrol.demiurge/spec/v1
fingerprint <alg>:<whole-file digest>    # the value a DRC-369 asset carries
manifest    <alg>:<digest of the chunk list>
size        <bytes>
```

Chunking is content-defined (FastCDC), in a layer that never touches identity: the fingerprint is a
function of the bytes and nothing else, so re-tuning the chunker never changes what an asset is. **The
chunker's parameters are not Qontrol's to fix, and are not yet measured.** The object-model blueprint
proposes min 256 KiB / target 1 MiB / max 4 MiB, reasoned from restic's and borg's settings rather
than measured on Demiurge content — measure it before writing it down as decided. Which hash the
fingerprint uses was decided on 22 September 2026: ADR-047's BLAKE3-256, with its algorithm tagged, so
`qontrol.toml` names it and it is no longer provisional.

## Substrate consumed

| Substrate | How Qontrol uses it |
| --- | --- |
| **QOR ID** | The commit author. One identity across every product; no per-host git account, no separate credential. Signing a published version uses the launcher vault's Sr25519 key (ADR-023, ADR-039), behind the host dialog (L1.4). |
| **DRC-369** | Publishing mints or revises an asset whose content fingerprint is the pointer file's fingerprint; a fork's asset names its parent. **Publishing a version is built** (M4.1, 22 September 2026; see Phase 5): the Projects surface mints the commit HEAD points at. Revising from Qontrol and fork → remix provenance are not built in the launcher, though the chain records a remix's source since 29 September 2026. |
| **Qontrol** | It is this product. Everything else in the ecosystem versions through it rather than carrying its own history. |
| **Mesh** | Qontrol's remote. Push, clone and the shared side store are Mesh operations. **M8, does not exist.** |
| **CGT** | Settlement for Mesh hosting of the side store and for access-gated private repositories. Both are existing sinks; neither price is set here (OPEN-4, U-4). |

**SUBSTRATE GAP — remote storage.** Qontrol's remote is the Mesh, and the Mesh is M8.1. Do not build a
Qontrol-only remote, do not stand up a bucket, do not run an LFS server. `.qontrol/cas/` is a
one-machine store, not a second storage rail, and the slices before M8 have **no remote at all**.

**SUBSTRATE GAP — exclusive locks.** Nothing in the substrate holds a short-lived, identity-scoped
claim over a path. QOR ID authenticates the claimant; where the claim lives is unsolved. Chain state
is the wrong shape and latency for a lock taken for twenty minutes, and there is no fee class to pay
for one anyway (OPEN-4, U-4). A Qontrol lock server with its own accounts is a gap papered over.

**SUBSTRATE GAP — repository identity.** A repository needs a stable id across clones. The natural
answer is a DRC-369 collection id. Token identity is decided — `(CollectionId, ItemId)`, ADR-047 — and
DRC-369 exists since M4.1 (22 September 2026), with one singles collection per creator; a collection per
repository is not something the chain offers yet (G-15 in `ECOSYSTEM.md`). Until then `qontrol.toml` carries a
locally generated UUID **marked in the file as a placeholder**; neither is built, since the scaffolds write no
`qontrol.toml` yet. Do not invent a token identity in a scaffold.

**SUBSTRATE GAP — the git author line.** A git commit carries a name and an email address; QOR ID is
neither, and inventing a domain to put after the `@` would be exactly that, an invention. Until the
rendering is decided the author line comes from the repository's own git config, and Qontrol names the
committer in its own surface.

**SUBSTRATE GAP — delegated commits, and published history.** An agent committing for a creator needs
scoped, revocable, capped delegation: M5.2 and ADR-026, recorded and not enforced today. A public page
listing an asset's published versions needs the provenance path of ADR-028 — events, an archive node
and an indexer — and the indexer is M5.4.

## The first usable slice

**A creator opens a folder and stops losing work.** They pick Projects, choose a kind, name it, and
get a laid-out folder that is already a repository with the right ignore rules. They work. The surface
shows what changed in plain words, they type a sentence, and they commit. The folder is an ordinary
git repository, so it can be zipped, copied to another machine, or opened with any git tool — and it
is clean under `git status`, which is the promise the whole split exists to keep.

That slice needs no node, no economic value and no pallet; it is not migration code, so the M2.1 hold
does not apply. The launcher's Projects surface and the `qontrol-git` sidecar landed on 21 September
2026 in commit `5a44207`, with the three tests above; everything below this line is still intent. What
the slice deliberately cannot do: branch, check out, merge, push, handle a large file specially, or diff
a session file structurally. It could not mint anything either, until 22 September 2026: minting is
Phase 5's first half, built with M4.1 and described there. **The line-by-line diff arrived on 23 September
2026, and P1.1 is ticked.** Choosing a change shows what changed inside it: the index's blob against the
working-tree file converted to what git would store under the repository's own `core.autocrlf` and
`.gitattributes` (`src-tauri/src/qontrol/diff.rs`), so a line ending the repository converts is never shown
as a change, and `-diff` makes a file binary. No diff driver's program is ever run. Structural diffs of
session files are still Phase 4 (P1.4).

**Phase 1's git layer arrived on 26 September 2026, and P1.2 is ticked.** Branch (gitoxide, at HEAD, not
switched to); switch and discard (libgit2's safe checkout in the helper, which refuses rather than
overwrites, and refuses to "discard" a new file because that would delete it); per-file commit, matched by
exact name because libgit2's pathspec treats `take[1].wav` as a glob; a guard before staging
(`src-tauri/src/qontrol/guard.rs`) that holds back a file over 50 MiB or one named or shaped like a
credential until the person accepts it by name, and runs again at commit; and the helper bundled as a
sidecar with libgit2's licence (`scripts/build-helper.mjs`, `src-tauri/tauri.bundle.conf.json`), which
settles risk 5's notice. **Not built from Phase 1:** commit signing with the vault key, which the roadmap's
P1.2 did not include. Merge and push are still not built.

## Phases to a full product

Estimates are for one founder and an agent, on a tree where the chain milestones compete for the same
weeks — honest guesses, not commitments. What is built runs in CI, which passes on `main`
(GitHub Actions on the public repository, ADR-063), with the Qontrol tests under `qontrol-no-skips`.

**Phase 1 — the git layer (reused library calls; days to a week).** Branch, switch, discard a change,
per-file diff, per-file commit. Commit signing with the vault key. A guard before staging: warn on a
file over a threshold and on anything that looks like a credential, because stage-on-commit means the
creator is not inspecting a list. Bundling the helper (`externalBin`) belongs here. *Done 2026-09-26 (P1.2),
except commit signing, which is not built.*

**Phase 2 — the scaffolds, completed (new, days).** The three above in full, plus Unity and Unreal
variants, `qontrol.toml` and the lock-required set, tested by a fixture repository that ends clean
under both status implementations.

**Phase 3 — reading a project as a creator sees it (new, about two weeks per format).** `.als` gunzip
and structural XML diff first: the format is documented and the work is in the summarising rather than
the parsing. `.rpp` and `.tscn` are already text. Image swipe and audio A/B come with Phase 4's store.

**Phase 4 — large binaries (new; a three-month slice, the least certain estimate here).** Chunker,
local content-addressed store under `.qontrol/cas/`, pointer files, local advisory locks. It must be
correct under interruption — a partial write that loses a take is unacceptable — which is most of why
this is months and not weeks.

**Phase 5 — the chain.** Publish a version → mint or revise a DRC-369 asset; fork → remix provenance.

**Publishing a version is built** (M4.1, 22 September 2026; roadmap P1.6, half). The Projects surface has a
Mint panel that shows the commit HEAD points at, by its full hash, and mints it. What that does, in order:

- **It reads the commit, not the folder.** The tree of the commit HEAD points at is walked in the
  repository's object store (`qontrol/publish.rs`). A project with uncommitted changes is refused — the
  asset would not be what is on the creator's screen — and so is one with no commit, a submodule, or a
  file name that is not UTF-8. It is pinned by hash, never by branch (ADR-047 decisions 8 and 9).
- **It builds the manifest ADR-047 decides** (`src-tauri/src/content/`). One entry per file, each with its
  own BLAKE3-256 fingerprint and length, sorted by path byte-for-byte whatever order the files were read
  in; a media type from a fixed table of IANA-registered types, and `application/octet-stream` for anything
  else; the role `Licence` for a licence file at the top of the project and `Component` for every other
  file; `created` set to the pinned commit's time, so the same commit always makes the same manifest; and
  `source` naming the commit and, for a person, its branch. The whole is SCALE-encoded, as the chain
  encodes, and its BLAKE3-256 is the asset's root. Whether the commit and branch should be inside the
  manifest at all is ADR-047's open item 2, Q-19.
- **It asks, then writes, then sends.** The host dialog names the project, the commit, the number of
  files, the fingerprint and the deposits the mint will hold, read from the node. Only after the person
  approves and the vault signs are the files and the manifest written to the **temporary content store**
  — `temporary-content-store/` in the launcher's own data directory, each blob under its BLAKE3 root,
  labelled temporary until the Mesh, as the owner answered ADR-047's question 13 — and the mint sent.
- **It appears in the Inventory,** read back from chain storage: the name, the reference, the pinned
  commit, and whether it is permanent, with **Make permanent** through the same dialog and vault.

**Not built yet:** revising an asset with a later commit — `Drc369::revise` exists on chain, and the
Projects surface has no button for it — and fork → remix provenance, whose chain half exists since
29 September 2026 (a mint records `derived_from`, ADR-061) and whose launcher half does not. The bytes
are on one machine until the Mesh (M8.1).

**Beyond — the Mesh (M8).** Push and clone against the Mesh, the shared side store settled in CGT,
cross-machine exclusive locks once a substrate primitive exists, and optionally a git-lfs
batch-and-locking shim in front of it. Each is a consequence of the Mesh existing, not a prerequisite.

**In three months**, one founder and an agent ship Phases 1 to 3 complete and Phase 4 started: a
creator runs a real project in it, on one machine, and shares it by any means git already supports.
**In twelve months**, Phase 4 done — large files deduplicated locally, advisory locks, A/B comparison
— with Phase 5 written against a chain that may not have reached M4. **Beyond twelve months**,
everything with the Mesh in it, and nothing with the Mesh in it should be promised sooner.

## Dependencies on chain and launcher milestones

**Qontrol's roadmap is P1, in `DIRECTION.md` §7**, added on 21 September 2026 at the owner's
instruction. `AGENTS.md` §3 forbids a second roadmap, so this blueprint keeps no list of its own; its
phases above map onto P1.2 to P1.7, and its gate is `qontrol` in `GATES.toml`.

| Depends on | For what |
| --- | --- |
| **Nothing** | Phases 1–4. No chain dependency, no economic value, not migration code. |
| **M2.1** — the owner reviews the migration inventory | Nothing chain-side may be written before this. **Ticked on 22 September 2026**, against the owner's review of a ten-line summary of the inventory's DRC-369 section. |
| **M2.3** — the DRC-369 wire format, including the content-hash algorithm and its tag | Fixes what the pointer file's `fingerprint` line means. **Ticked on 22 September 2026 (ADR-047).** |
| **M4.1 / M4.2** — DRC-369 mint, transfer and content fingerprint; nesting and remix royalties settled in CGT | Publish a version: **built with M4.1, 22 September 2026**. Fork as remix provenance: the chain records it since M4.2's royalty half (29 September 2026); the launcher does not offer it yet. |
| **M5.2 / ADR-026** — delegated agent keys with spend caps | An agent committing for a creator. |
| **M5.4** — the public viewer, on the provenance source chosen in M2 (ADR-028) | Published-version history outside the launcher. |
| **M8.1** — the Mesh, and seeder work verification (U-6) | A remote at all. |
| **M8.2** — entitlements from DRC-369 | Access-gated private repositories. |
| **L1.4** — host-side confirmation before any signature | Publishing. Implemented and unit-tested; unticked until the native dialogs are exercised in a running launcher. |
| **L1.6 / L1.7** — the launcher's tests run and pass in CI | The three evidence tests are only evidence once CI runs them. **CI runs them**, under `qontrol-no-skips`, and passes on `main` since 5 October 2026. |
| **L4.1** — minting from the launcher | Shares the publish path. |
| **L6.1 / L6.2** — signed installers and a verifying update channel | Bundling the sidecar binary and shipping it with the launcher. |
| **L7.1 / L7.4** — Library delta patches, Mesh seeding | The chunk store Qontrol builds is the same store a delta patch reads. |

## What "full support" means, and what it does not

**"Git-compatible on disk" means exactly this:** the folder is a real git repository; the objects,
refs, index and config are git's; `git status`, `git log`, `git diff` and `git clone` work; a
collaborator with plain git and no Qontrol can work in it and their commits are visible in Qontrol;
nothing about the repository is lost if Qontrol is uninstalled.

**It does not mean Qontrol is a full git client, and it does not mean the work is safe.** Bluntly:

- **No merge, rebase, cherry-pick, stash, submodules, worktrees or history rewriting**, in any phase
  described here, and no conflict resolution interface. Qontrol refuses to merge unmergeable binaries
  by design and does not resolve text conflicts. If a creator needs those verbs, they use git.
- **No hooks.** Qontrol neither runs nor installs them. A repository with hooks keeps them for `git`.
- **No network git.** No push, pull, fetch, clone-from-URL or credential handling in any slice before
  M8. The sidecar is built without OpenSSL and libssh2, so this is structural rather than a to-do.
- **No git-lfs.** Not read, not written, not migrated. A repository already using LFS opens and shows
  history; its LFS files appear as their pointer text.
- **No cross-machine locks** until a substrate primitive exists to hold the claim, and **no signed
  commits in the first slices** — gitoxide lists signed commits and tags as unimplemented, so signing
  rides with the publish path and the vault, not with every commit.
- **A commit is not a backup.** History and the side store live in the same folder on the same disk.
  Until the Mesh exists there is no second copy anywhere, and Qontrol cannot recover a file it never
  saw — including one an ignore rule kept out. Losing the drive loses everything.
- **"Full support" for a DAW or engine means the correct ignore and attribute rules plus a readable
  diff for the named formats.** It does not mean parsing every proprietary format, it does not mean
  understanding a project well enough to merge two edits of one session, and it never means a format
  the vendor changes silently keeps diffing. When a format moves, support breaks until it is fixed.
- **Not for kernel-scale repositories.** The target is a creator's project, not a monorepo with a
  million commits.

## Risks

1. **The lying index, and the tests that skip.** The whole foundation section exists because of it.
   On a machine without git only one implementation is checking, and the tests return early when the
   helper is not built. They say so, and under `--features qontrol-no-skips` — which CI and the Qontrol
   gate's suite use — they fail instead, because a skip reads as a pass.
2. **Two ignore implementations.** gitoxide decides what the creator sees; libgit2 decides what is
   committed. A divergence commits a file someone believed was ignored, or hides one they expected to
   save. Pinned in both directions, by a fixture that must grow with every pattern the scaffolds use.
3. **Stage-on-commit stages everything.** It is the right interface, and it is also how a secret or a
   40 GB render enters history. The Phase 1 guard is not a nicety.
4. **libgit2's parser surface.** The process boundary contains a crash and memory corruption; it does
   not make the parser safe. The sidecar holds no keys and opens no network, and its advisories still
   need tracking — there is deliberately no `.cargo/audit.toml` in this tree.
5. **What the C dependency costs.** A C toolchain becomes a build requirement wherever the sidecar is
   built, CI included, where it builds and passes. And libgit2 is GPLv2
   with a linking exception, in a tree whose crates are MIT: the exception permits shipping it, but
   the installer's third-party notices must carry libgit2's terms explicitly. Owner-visible.
6. **The sidecar goes missing.** A partial install, an antivirus quarantine, an unsigned binary on
   macOS, and commits stop. It resolves from `QONTROL_GIT_BIN`, then beside the launcher executable,
   then a development build tree. Since 26 September 2026 the installer puts it beside the executable
   (`externalBin` in `src-tauri/tauri.bundle.conf.json`), on Windows; the macOS and Linux bundles are not
   built yet. The port reports whether the helper is present.
7. **Two hash algorithms, permanently.** Git object ids are SHA-1 today and SHA-256 under Git 3.0;
   content fingerprints are BLAKE3-256 (ADR-047, M2.3), so deduplication between the git object store and
   the content store is impossible across that boundary. The algorithm tag keeps this decidable.
8. **Chunk boundaries leak information about content.** Published research shows per-user randomised
   chunker parameters can be extracted, after which the leak is protocol-agnostic. Irrelevant for
   public content, real for access-gated content on a Mesh where seeders are strangers — it belongs
   with U-6 and the access-gating sink, not inside a chunker.
9. **Scope creep toward being git.** Every missing verb is a request. The answer is the stopping rule:
   Qontrol serves creators who do not want git's model, and the moment it grows a rebase interface it
   is a worse git for the people who already have one.

## Decided by the owner, 22 September 2026

Every question this blueprint asked was answered that day. Each answer is the owner's and is reversible.

1. **A roadmap number:** P1 (answered on 21 September).
2. **Repositories are SHA-1 by default**, for compatibility with every tool that exists; **SHA-256 is an option**
   at `init`. Asset identity rests on ADR-047's BLAKE3 fingerprint, not on the commit hash, so the choice does
   not reach the chain.
3. **The game template is Godot-first, with Unity and Unreal variants.**
4. **A user's Unity project is never switched to text serialisation automatically.** Qontrol offers it and
   explains why; the user decides.
5. **Two placeholders, as proposed:** a locally generated repository id and the repository's own git author line,
   both marked provisional. ADR-047 is accepted, so the repository id becomes a DRC-369 collection id once M4.1
   can give a repository one. *M4.1 exists since 22 September 2026, with one collection per creator rather than
   per repository, and the scaffolds write no repository id yet.*
6. **Local file locks only**, until the platform can hold a lock. No Qontrol lock service.
7. **The helper ships inside the launcher as a bundled sidecar.** An installed launcher must be able to commit
   (P1.2). *Done 2026-09-26: a Windows installer carries it; the macOS and Linux bundles are not yet built.*
8. **Publishing revises one asset** across versions, per ADR-047 — revisable until its one-way switch makes it
   permanent.
9. **Qontrol becomes its own process under ADR-046**, dated when the app-host protocol exists.

**Sequencing, also the owner's:** P1.1 stays unticked until in-file diffs exist, honouring `core.autocrlf` and
`.gitattributes` as git does. **That diff is the next Qontrol item after M4.1.** *Done 2026-09-23; P1.1 is
ticked.*
