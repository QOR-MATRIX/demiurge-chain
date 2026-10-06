# GNOSIS

**Status:** Blueprint, 21 September 2026. **Proposed, and it describes intent rather than code that exists.** The
records it leans on — ADR-046 (the app host), ADR-047 (the object model), ADR-048 (interchange), ADR-050 (where
products live) — are all accepted: ADR-047 on 22 September 2026, the other three on 28 September. GNOSIS's track is P4
in `DIRECTION.md`, and no record defining GNOSIS itself exists yet. It fixes no price, rate, share or split: each is
OPEN-1, OPEN-2 or OPEN-4 and undecided.

**Since 22 September 2026:** ADR-047, the object model and DRC-369's wire format, is accepted and M2.3 is ticked, and M2.1 was ticked the same day against the owner's review of a ten-line summary of the inventory's DRC-369 section. Wherever this document says something waits on "M2.1 and M2.3", it now waits on neither, and where it calls the content fingerprint undecided, it is a BLAKE3-256 manifest root with its algorithm tagged.

## What it is

GNOSIS is a music production application where a song is a folder of readable text files instead of one sealed
document. Two people can work on it at once — one on the drums, one on the bass — and their work merges the way two
people editing different files in a codebase merge: cleanly, with nobody overwriting anybody. Everything large —
takes, bounces, samples — is stored once under a content fingerprint and referenced, never copied in.

## Who it is for

- **Collaborators who are not in the same room** — two producers, a producer and a vocalist, a band with four laptops.
  Today they swap whole project files by email and one of them loses work.
- **Producers who want a history**, not a backup folder called `final_v7_REAL`: a diff you can read and a version you
  can go back to.
- **Creators who will be paid for use** — a loop, a stem, a preset someone else builds on. **The rail for that payment
  is only partly there:** DRC-369 is started (M4.1 mint, and M4.2's royalties, a sale settled in CGT and nesting, are
  built), the Mesh is M8 and unstarted, and the licence fee, burn share and seeder rate it needs are OPEN-1, OPEN-2 and
  OPEN-4, named here and left empty.
- **Not, at first, mix engineers with a hundred VST3 plugins.** The first releases cannot load most of what they own.
  Said here so it is not discovered later.

## The foundation, and why

**Device I/O: CPAL** (`RustAudio/cpal` 0.17.3, 18 February 2026, Apache-2.0) — pure Rust, no C++ toolchain,
feature-gated ASIO path. Rejected: **RtAudio**, which works but drags a second toolchain in, kept as a named fallback;
and **miniaudio**, whose Rust bindings are fragmented across three repositories with no clear current one
(**UNVERIFIED**) — binding risk on the device layer is silent-failure risk. Linux is CPAL's JACK backend against
**PipeWire's JACK implementation**, not `jackd2`. **Decode is Symphonia** 0.5.5 (AAC, ALAC, FLAC, MP3, MP4,
OGG/Vorbis, WAV, AIFF), and the **real-time plumbing** is `rtrb`/`ringbuf` for lock-free SPSC with `basedrop` for
deferred deallocation: standard answers, nothing invented.

**Plugin hosting: CLAP first, through `clack-host`** (`prokopyl/clack`), the only serious host-side Rust option, with
`clack-extensions` for the standard extensions and a DAW-side fork at `MeadowlarkDAW/clack`; its licence is
**UNVERIFIED**, so read `LICENSE` first. Rejected for now: **JUCE 8**, which hosts everything at once and costs AGPLv3
or a commercial licence, and a **pure-Rust VST3 host**, which does not exist — `nih-plug`, `nice-plug` and
`clap-wrapper` are *export* frameworks and host nothing.

**ADR-001 applied, both ways.** Where failure is silent, use what is proven: device I/O, decode, git's object model
under Qontrol, the ring buffers, the real-time discipline. Where failure is loud and recoverable — a bad merge is
audible in seconds and the commit before it is still there — invent. That is the project format, the only part of
GNOSIS nobody else has. **The cautionary precedent:** **Meadowlark**, the most serious open Rust DAW attempt, slowed
to a stop, and its author's write-up is titled *"DAW Frontend Development Struggles"*. The engine did not kill it; the
UI and plugin-GUI embedding did, so this blueprint ships the differentiator before entering the graveyard.

### The VST3 licence reality, stated plainly

- **VST3 is MIT** since **VST 3.8, 29 October 2025**: "Neither fees nor memberships are required", "No need to sign
  any documents". A closed-source product may host it, and the whole obligation is the MIT notice; **pre-3.8 copies
  carry the old dual licence**, so vendor 3.8 or later. **Trademark is separate:** the *VST* word mark and *VST
  Compatible* logo carry Steinberg's guidelines, and a breach does not revoke MIT rights. GNOSIS displays neither
  unless the owner decides to.
- **VST2 is not available** — `aeffect.h`/`aeffectx.h` may not be redistributed and Steinberg stopped issuing licences
  in 2018. No workaround, and we will not look for one. **AAX is out of scope**: Avid registration, iLok, PACE
  signing, renewed annually.
- **ASIO went the other way:** relicensed in November 2025 to **GPLv3+ OR proprietary**, so a closed-source Windows
  host that wants ASIO takes Steinberg's proprietary agreement — the only Steinberg paperwork GNOSIS needs, on the
  driver layer. **UNVERIFIED against Steinberg's primary ASIO page**, read from reporting (open question 5).

**CLAP is MIT**, community-owned, stable at **1.2.6 (11 March 2025)**, with currency into 2026 **UNVERIFIED** because
the project tags rather than releases. **So the licensing argument for CLAP-first is gone**, and what remains is
engineering: a flat C ABI rather than a COM object model, threading *specified* with a `thread-check` extension the
host hands the plugin, and extensions anyone can add.

## What is new

**A text-based, mergeable project format designed for Qontrol** — the centrepiece, and the only part of GNOSIS that is
not assembly. **The precedent is Godot's `.tscn`:** a scene is a text file, a person can read it, `git diff` shows
what changed, and two people editing different parts of it merge without a custom tool. Godot proved the property for
scenes; nobody has done it for a song. GNOSIS takes it as a **requirement**: **text, diffable, mergeable, or it is not
the format.** We take the property, not the dialect — an engine's own format is lock-in, answered here by DAWproject
export and automatic stems (R9, ADR-048).

### Why no DAW project merges, and what the format does about each

| Cause | Real example | What GNOSIS does |
| --- | --- | --- |
| **Binary serialised object graphs** | FL Studio `.flp` is a serialised object graph with no public spec; Pro Tools `.ptx` needs decryption before the block tree can be walked | Be text |
| **Text that is one giant document** | Ableton `.als` is gzipped XML — every track, clip, device, knob and automation point in one document, rewritten wholesale on every save | Be many files |
| **Opaque plugin state** | Even DAWproject, the best open format, stores plugin state as a referenced blob inside the archive | Two honest tiers (R7) |
| **Positional identity** | Tracks and clips identified by index or document order: insert a track on one branch, delete an earlier one on the other, and a *textually clean* merge produces a valid file describing the wrong song | Stable ids, explicit order (R2) |

A fifth cause is **embedded and absolute media** (Reaper's `.RPP` is plain text and still carries paths inline), and
two more are self-inflicted: **non-deterministic save** and **UI state in the project**. The graveyard matters too:
Splice Studio shut down in stages ending **31 May 2023** because it could *version* and never *merge*, sitting outside
the DAW snapshotting opaque files. Owning the format is what makes merge possible.

### The project tree

```
song/
  .gitattributes        # merge drivers, eol=lf, binary paths
  project.toml          # id, name, sample rate, track_order = [ids]
  tempo.toml            # tempo map + meter map, sorted records
  media.lock            # per media object: fingerprint, name, channels, rate, frames
  tracks/t_9f2c.toml    # identity: name, colour, i/o routing, device_chain = [ids]
  tracks/t_9f2c/clips/c_4a71.toml              # media ref, source range, position, fades, gain
  tracks/t_9f2c/clips/c_4a71.notes             # note events, sorted by musical position
  tracks/t_9f2c/automation/d_1e88.cutoff.auto  # breakpoints, sorted
  mix/t_9f2c.toml       # gain, pan, sends — separate file from track identity
  devices/d_1e88.toml   # plugin identity + every exposed param + optional state ref
  local/                # UNTRACKED, always
```

**That is the entire committed tree**, and if a byte in it is not readable in a text editor, something is wrong.
**Content-addressed and referenced, never in the tree:** takes, bounces, stems, samples, impulse responses, masters,
video, and every plugin's opaque state chunk. **Never committed:** `local/` — zoom, scroll, selection, layout, undo
history, scan and peak caches. A clip's source is `media = "blake3-256:7a1f…"` plus a source range in musical time —
the bytes live in the content store, the algorithm is tagged rather than assumed, and `media.lock` is the only place a
fingerprint meets a readable name.

### Nine rules

**R1 — One file per independently-editable unit.** Two people editing different tracks touch disjoint files, so plain
three-way merge resolves them with no conflict and no driver: the common case goes from impossible to free. It is why
`mix/` is split from `tracks/` — renaming a track and changing its gain must not collide.

**R2 — Stable ids, explicit order.** Every object carries a UUID, and order lives only in explicit lists
(`track_order`, `device_chain`), never in file order — killing positional identity, the cause of silently wrong
merges.

**R3 — Nothing binary in the tree.** Media by content fingerprint only, and **the fingerprint is the substrate's, not
GNOSIS's**: ADR-047, accepted on 22 September 2026, makes it BLAKE3-256 as a 32-byte root with the algorithm tagged in
a SCALE enum. The tag is why `media.lock` writes `blake3-256:` instead of assuming: a Qontrol commit id is a git object
id, never BLAKE3. **The fingerprint is decided and built**: the launcher computes ADR-047's manifest
(`tools/qor-launcher/src-tauri/src/content/`) when it mints a commit. The store those references point at is the Mesh
(M8.1), which is **not built**; a temporary content store on the minting machine stands in for it (G-2, G-5).

**R4 — Canonical serialisation, enforced by the writer.** Fixed key order, fixed decimal formatting with no float
round-trip, LF, UTF-8, no timestamps, no machine names, no UI state. **A save that changes nothing produces a
byte-identical tree** — a test, not a claim.

**R5 — Automation and notes as sorted line records, in musical time.** A note is one line —
`5:1:000  0:0:240  60  100  1  n_7f3a`, being start, length, key, velocity, channel and id under a commented header —
and a breakpoint is the same shape. Positions are **bar:beat:tick, not seconds**, so editing the tempo map does not
rewrite every note line; records are sorted by position, so a diff is exactly the events that changed and two people
working in different bars produce non-overlapping hunks. Every event carries an id, which is what makes R6 possible.

**R6 — A typed merge driver for the cases text merge gets wrong.** Line merge is right most of the time and
confidently wrong the rest.

| File class | Merge semantics | What plain text merge does instead |
| --- | --- | --- |
| `.notes`, `.auto` | Keyed set union by event id. Same id both sides = real conflict; different ids = union regardless of adjacency | Two people adding notes in the same bar produce adjacent lines and a spurious conflict |
| `track_order`, `device_chain` | Sequence merge with insert/delete and a deterministic tie-break, both outcomes reported | Silently interleaves — a compressor before the EQ instead of after |
| `mix/*.toml` scalars | Genuine conflict, surfaced musically: "you both set the gain on Drums — keep mine / keep theirs / A-B them" | A `<<<<<<<` hunk in a file the musician will never open |
| `media.lock` | Fingerprint equality; different fingerprints = both kept, both named | A path collision that looks resolved and points at the wrong take |

The drivers are the format's; **the registry binding path patterns to drivers is Qontrol's**, and is not built here
(G-1).

**R7 — Plugin state in two honest tiers.** *Tier 1, parameters:* every automatable parameter as `id = value` in
`devices/*.toml` — CLAP gives stable `clap_id`s, VST3 exposes parameter ids, both diffable. *Tier 2, opaque state:*
the plugin's own chunk, content-addressed like media, and **it does not merge.** Hence the rule that buys the most:
**GNOSIS's own devices are Tier 1 only, permanently**, so **a native-only project merges completely** and third-party
plugins are the degraded case.

**R8 — Non-destructive by construction.** A clip is `(media ref, source range, position, fades, warp, gain)`; a bounce
creates a *new* content-addressed media object plus a clip referencing it, and the sources stay. Without this, "we
both edited the same take" has no representation to merge.

**R9 — Interchange is DAWproject; the working format stays the tree.** DAWproject is MIT, v1.0, a ZIP of `project.xml` and
`metadata.xml` against published schemas, supported by Bitwig 5.0.9, Studio One 6.5, Cubase 14, Cubasis 3.7.1, VST
Live 2.2 and Nuendo 14: the right interchange format and the wrong working format, since one XML document in a ZIP
will not merge. Export it anyway, with stems and an SMF automatically rather than on request (ADR-048, 7 and 8).

### Two people, one song, and the Windows traps

Ana on drums and Ben on bass touch disjoint files, so three-way merge resolves with zero conflicts and no driver runs;
the only shared file is `track_order`, and only if one added or removed a track. Both editing the drum track still
merges, because records are sorted by musical position and the keyed-union driver merges by event id where hunks
touch. Both setting the gain on Drums is **a real conflict**, surfaced as a musical choice. Both changing a synth's
opaque state **does not merge, and the format says so**: one side wins, the other is still in the history.

- The tree ships its own `.gitattributes`: `* text eol=lf`, event files bound to the typed drivers, the content store
  marked `-text -diff`. Qontrol honours the repository's config and `.gitattributes` **as git would**, `core.autocrlf`
  and case included — otherwise a Windows checkout differs from what was committed, and the status is clean while the
  tree is wrong.
- **Every id is lowercase hex**, because paths differing only by case collide silently on Windows and macOS; and
  **musical positions never appear in a file name**, because `1:0:000` contains a colon, illegal in a Windows path.
- The ignore rules for `local/` must be obeyed identically by the read path and the stage path, or an untracked cache
  file gets committed by one and hidden by the other.

## Architecture

Four processes, and the split is a security decision before it is an engineering one.

1. **The launcher host** (Rust, Tauri) — the sealed vault (Argon2id + XChaCha20-Poly1305, Sr25519), QOR ID sign-in,
   the confirmation dialog before any signature. `unsafe_code = "forbid"` stays here, and **no plugin binary, no git
   parser and no stranger's bytes are parsed here.**
2. **The Qontrol sidecar** — gitoxide serves every read path (status, log, diff, branches) in-process in safe Rust;
   libgit2 serves stage and write-tree only, in a helper binary the host spawns over a minimal stdio protocol, both
   behind Qontrol's own interface, so the split is invisible above it and reversible the day gitoxide's
   `tree from index` lands. It exists today as ADR-046's first obligation discharged rather than owed:
   `tools/qor-launcher/qontrol-git/`, `git2 0.21` with `vendored-libgit2`, `unsafe_code = "forbid"`, six operations
   (stage all, stage named files, plan, write-tree, switch, discard) plus a ping, bundled with the launcher through
   Tauri `externalBin` since 26 September 2026 (P1.2), so an installed launcher can commit.
3. **The audio engine process** — CPAL out, the graph, transport, native Tier-1 devices. It owns the real-time thread;
   the webview never enters the audio path, and IPC never blocks it.
4. **Plugin host processes, one per plugin instance or per vendor** — `clack-host` for CLAP, later a C++ shim for
   VST3, audio crossing by shared-memory ring. **The reasoning that put libgit2 in a sidecar puts a third-party DSP
   binary there a fortiori:** a plugin is an arbitrary native binary the user chose, and it must never share an
   address space with the vault. Crash isolation comes free with the boundary.

**Real-time discipline, on the audio thread:** no allocation, no locks, no syscalls, no logging, no `Drop` that frees
— no `Vec::push` that may grow, no `Box::new`, no `format!`, no `Mutex::lock`, no `Arc` drop reaching zero. Buffers
are pre-allocated and fixed size, cross-thread traffic is a lock-free SPSC ring, deallocation is deferred through
`basedrop`. The two failure modes that ship in real products are an oversized callback buffer reallocating on the
real-time thread, and the callback blocking on a mutex held by a normal-priority thread — priority inversion, heard as
an intermittent click.

**Two departures to write down rather than assume.** A panic unwinding through a C FFI callback is undefined
behaviour, so every callback boundary takes `catch_unwind` or the crate takes `panic = "abort"` with a stated reason —
an ADR, not a comment. And `unsafe_code = "forbid"`, which the host sets today, **cannot hold in the audio crate**:
the lint is relaxed there alone with the reason recorded, the departure AGENTS.md §7 asks for.

**Latency, arithmetically.** 64 frames at 48 kHz is 1.33 ms per buffer and 2.67 ms across two, realistically **3.6–5.6
ms** round trip once driver and converter overhead is counted; 128 frames is 5.33 ms across two and realistically 6–8
ms. That is native ASIO or CoreAudio with decent hardware, not shared-mode WASAPI, which is 10 ms and up.

## Substrate consumed

| Substrate | How GNOSIS uses it | State |
| --- | --- | --- |
| **QOR ID** | Sign-in and the commit author identity. Commits are signed by the Sr25519 key the launcher vault derives, through the host confirmation dialog. **Reuse the one signer; never build a second** | Real, and live at `https://id.qorsync.dev`. 166 tests (6 October 2026); the vault derives Sr25519 as `sp-core` and Polkadot.js do (L3.2, ADR-039). L1.4 — host confirmation before any signature — is implemented and still unticked |
| **Qontrol** | The versioning substrate: gitoxide for every read path, libgit2 in the sidecar for stage and write-tree, behind Qontrol's interface. GNOSIS also needs its merge policy registry (which path patterns get which driver, versioned with the format) and a claim/lock for what cannot merge | The split path and the sidecar exist; **the content store, the merge policy registry and claims do not** — G-1 |
| **DRC-369** | A song version is the nestable asset that carries royalties; stems are its nested components; a sampled loop carries remix royalties to whoever made it | **M4, started.** M4.1 (mint, 22 September 2026) is done and M4.2's royalties and nesting are built; nothing in GNOSIS uses them yet — G-3 |
| **Mesh** | The content-addressed store the media references point at, with dedup and partial fetch, so cloning a song fetches what the current version references, not every take ever recorded | **M8.1** — G-2 |
| **CGT** | Settlement when a loop is licensed, a stem is remixed, or a work is used | Balance transfers, and since 29 September 2026 a sale settled in CGT with its royalties (ADR-061). No fees, no treasury, no issuance; every rate and share is OPEN-1/2/4 — G-4 |

### Substrate gaps, named and not filled locally

- **G-1 — Qontrol's content store, merge policy registry and claims.** GNOSIS must not ship its own history store: the
  exact local workaround to refuse.
- **G-2 — The Mesh media store (M8.1).** Until it exists, media lives in a local content directory that is explicitly
  not a substrate, is documented as temporary, and never enters a wire format (ADR-047, 13).
- **G-3 — DRC-369's wire format.** Decided by ADR-047 (M2.3, 22 September 2026) and frozen on SDK publication.
  GNOSIS's asset requirements were checked against it that day (the table at the end); two of the format's open items,
  Q-19 and Q-20, are still open before the freeze.
- **G-4 — Settlement above a sale.** A sale settled in CGT with royalties exists (ADR-061); there is no transaction
  payment (M6.4), no treasury (M6.5) and no sponsored fees (M4.4). **No per-use price, burn share, seeder rate or licence fee is chosen here — those are OPEN-1, OPEN-2 and
  OPEN-4**, and a prototype needing a number marks it a placeholder.
- **G-5 — The content fingerprint.** `media.lock`'s and DRC-369's must be one scheme or the product forks the
  substrate. **ADR-047, accepted on 22 September 2026, makes it BLAKE3-256 with a tagged algorithm, and it is built:**
  the launcher computes the manifest root when it mints, and M4.1's pallet stores the tagged reference. GNOSIS writes
  the same tag; it picks nothing.
- **G-6 — The licence vocabulary.** Nothing defines what a licence term is, and Stream, Market, Library and QFX need
  the same one. One ADR, before any of the four.
- **G-7 — GNOSIS and Qontrol have no product home.** Their numbers arrived on 21 September 2026 — P4 and P1 in
  `docs/DIRECTION.md`, with an entry each in `docs/SYSTEMS.md` and a gate each. ADR-050 is accepted (28 September
  2026), but under its decision 10 a product gets `products/<name>` only with its own defining record accepted, and
  neither has one, so neither has a directory of its own, and GNOSIS's tests unit stays unmeasurable until it does.

## The first usable slice

**The format, before the DAW.** The first thing worth opening lets a creator:

1. Open a folder of WAVs, lay them on a timeline against a tempo map, set gain and pan, and **commit** — through
   Qontrol, signed by their QOR ID key, the host dialog asking first.
2. Hand the folder to a collaborator who has GNOSIS, edit a different track each, and **merge with no conflict**, both
   machines producing the identical tree and the identical sound.
3. Read the diff and understand it: `+5:1:000 0:0:240 60 100 1 n_7f3a` is a note, and a person can see that.
4. Save twice with no edits in between and get a **byte-identical tree**.

No plugins, no recording, no DAWproject yet: a player, a mixer and a history — and it already does the thing no DAW
does.

**Two tests, written to fail first, before either path is trusted.** After a commit through the split read/stage path,
gitoxide's status **and**, where git is installed, `git status --porcelain` both report clean, pinning the lying-index
failure rather than trusting it. And ignore agreement, both directions: a file untracked in gitoxide's status is
skipped by the sidecar's add, and a file the sidecar stages is not reported untracked — both honouring the
repository's config and `.gitattributes` as git would. **Both exist for Qontrol's own scaffolds**
(`tools/qor-launcher/src-tauri/src/qontrol/tests.rs`, proven to fail first in `5a44207`). GNOSIS needs the same two
run against its own project tree, because its `local/` rules are the ones that must agree, and that fixture is not
written.

## Phases to a full product

| Phase | Work | Class |
| --- | --- | --- |
| **Reused, not written** | Tauri 2 shell, sealed vault, QOR ID sign-in, host/webview split, signing dialog, the Qontrol sidecar and read path, CPAL, Symphonia, `rtrb`/`basedrop`, `clack-host`, DAWproject's schema | **Reused.** 204 launcher Rust tests and 166 QOR ID tests exist (6 October 2026); the rest are maintained third-party crates |
| **0 — the format alone** | Spec, deterministic writer, round-trip reader, the typed merge drivers, `.gitattributes`, a merge test suite over fixtures. A library and a CLI, **zero audio code** | **New, weeks to a few months.** The highest-leverage work available, testable without a DAW around it |
| **1 — playback** | CPAL out, Symphonia decode, transport and clock, clip playback, gain/pan mixer, tempo map, waveform drawing, commit and merge through Qontrol | **New, ~3 months**, on one platform. Assembly of maintained crates, not research |
| **2 — recording and native devices** | Input and takes, Tier-1-only EQ, compressor, delay, sampler. DAWproject import/export against the published MIT schema. Stems + tempo map + SMF export | **New, 3–6 months** |
| **3 — CLAP hosting** | `clack-host` in the sandboxed plugin process: load, process, params, state, correct threading with `thread-check`, and plugin delay compensation across a graph that re-plans when a plugin changes its latency at runtime | **New, 12 months.** "Load a plugin and hear it" is weeks; *correct* is a year |
| **4 — the graveyard** | Plugin GUI embedding across Win32/AppKit/X11/Wayland inside a webview shell. VST3 via a C++ shim over `public.sdk/source/vst/hosting`. AU on macOS, non-optional for this audience | **New, 12 months each**, and this is what stopped Meadowlark |
| **Beyond** | Mesh-backed media store. Song versions and stems as DRC-369 assets with remix royalties. Per-use settlement in CGT | **Blocked on M4, M6, M8 and on OPEN-1/2/4.** No rate, share or price may be chosen |

**What one founder and an agent ship in three months:** Phase 0 complete, and Phase 1 playing audio on one platform.
**Not** Phase 2, not a plugin, not a recording path, not three platforms. And the clock has not started: the
launcher's current phase comes first (the substrate ADRs it also waited on were all accepted by 28 September 2026),
and GNOSIS's track, P4, has no item started (G-7). **In twelve months:** Phases 0–2 and an honest start on Phase 3 — CLAP plugins loading and processing,
demonstrable but not yet correct when one changes its latency mid-session. **Beyond twelve months:** Phase 3 finished,
Phase 4 at all, and every substrate-dependent line above. A DAW someone finishes a record in is further out still —
Bitwig took years with a team that had shipped one before — and Qontrol's substrate half is not GNOSIS's three months
to spend.

## Dependencies on chain and launcher milestones

**Nothing in Phases 0–2 depends on a chain milestone.** The format, the merge drivers, DAWproject and the whole audio
engine can start today. That is deliberate.

- **L1.4** — host confirmation before any signature. Every GNOSIS commit signs, so this is the prerequisite for
  committing at all. **Open.**
- **L1.6 / L1.7** — the launcher's tests in CI, and CI on `main` starting and passing. **CI runs**, on GitHub
  Actions on the public repository (ADR-063), and passes on `main`. A merge suite and a
  byte-identical-save test are evidence only once they join it; until then they are decoration.
- **L3.2** — Sr25519 keys and SS58 display, **done** (ADR-039). **M2.3** — the wire format decisions FRAME forces:
  field bounds, token identity, fixed-point encoding. **GNOSIS's asset requirements must enter that input set before
  it freezes.**
- **M4.1 / M4.2** — mint, transfer and the fingerprint `media.lock` must share (G-5); then nesting with bounded depth
  and royalties, remix included, settled in CGT. **M4.4 / M4.5** — sponsored fees, so a creator can publish without
  holding CGT, and the acceptance tests including the nesting-cycle refusal a stem-of-a-stem exercises.
- **M5.2 / M5.3** — agent rails and the MCP server, for an agent that masters or tags a track under a scoped, capped
  delegation. **M6.4 / M6.5** — transaction payment and treasury; without them there is no settlement above a
  transfer, and their parameters are OPEN-4 and OPEN-2.
- **M8.1 / M8.2** — the Mesh, and entitlements from DRC-369. **L4** (Studio and assets, depending on M4) and **L7.5**
  (Studio inside the launcher) are where GNOSIS's publish path lands — beside them, not before them.

## What "full support" means, and what it does not

- **"Full VST3 support"** means GNOSIS hosts VST3 under the MIT SDK 3.8 or later, with no Steinberg agreement and no
  source disclosure. It does **not** mean every plugin loads — one that crashes is isolated and stays isolated. It
  does **not** include **VST2**, which is not legally available to a new entrant, at all, ever, nor **AAX**. It is
  **Phase 4**, so for the first year it means nothing at all.
- **"Full CLAP support"** means a named extension set: `audio-ports`, `note-ports`, `params`, `state`, `latency`,
  `thread-check`, `timer-support`, `gui`, every other extension declined until asked for by name. It does **not** mean
  the first release loads what a musician owns: the CLAP library is **394 plugins across 93 vendors**, strong on
  synths, thin on the mix-bus and mastering tools people are loyal to, and **Ableton Live, Logic Pro and Pro Tools do
  not host CLAP at all.** A hard adoption wall.
- **"Mergeable project"** means different tracks always merge, different bars of the same track always merge, and
  ordered lists merge with both outcomes reported. It does **not** mean every conflict resolves automatically: opaque
  plugin state never merges, and two people rewriting the same eight bars should **not** auto-merge, because
  convergence is not musical correctness. That is also why the durable format is not a CRDT, whatever a live session
  on top of it uses. **"Text format"** does not mean humans author it: machines write it canonically, humans read
  diffs.
- **"Versioned by Qontrol"** means git-compatible objects, refs and working tree, read by gitoxide and staged by
  libgit2 in the sidecar. It does **not** mean a content store, a merge policy registry or claims exist (G-1), nor
  that media lives anywhere but a local temporary directory until the Mesh lands (G-2). **"Commit"** means a signed
  git commit on the creator's own disk: it does **not** touch the chain, mint anything or settle anything, and **a
  song is not an on-chain asset until it is published as one.** DRC-369 minting exists (M4.1); GNOSIS has no
  publish path to it yet.
- **"Open your session in Cubase"** means DAWproject carries your arrangement, your automation and your plugin
  *settings*. **It does not carry your plugins. On a machine without them, those tracks will be silent.** That
  sentence, or one as blunt, appears in the UI at the moment of export. The universal exit is stems + a tempo map + an
  SMF + a plain-text session sheet, exported automatically rather than on request.
- **"Creators get paid for use"** describes intent, and intent only. **No price, rate, share or split exists**,
  because OPEN-1, OPEN-2 and OPEN-4 are undecided and inventing one is forbidden; any number in a prototype is marked
  a placeholder. It is a statement about payment for work someone chose to use, and about nothing else.
- **"Low latency"** means single-digit milliseconds round trip at 64–128 frames on native ASIO or CoreAudio, not a
  claim about shared-mode WASAPI or any particular interface. **"Cross-platform"** means Windows, macOS and Linux
  eventually: the first releases target one platform, and open question 5 decides whether that one is Windows.

## Risks

1. **The graveyard is real and it is not the DSP.** Plugin GUI embedding and the editor UI stopped the most serious
   prior Rust DAW; phasing the format first does not remove that risk, only ensures something shipped before we reach
   it.
2. **A format alone is not a product people adopt.** Splice had distribution, a user base and funding, and shut Studio
   down in 2023. GNOSIS has what Splice lacked — the format — and lacks what Splice had.
3. **CLAP-first is an adoption wall.** The first release cannot load a large share of what the target user paid for;
   open question 4 asks the owner to accept that knowingly or change the phasing.
4. **ASIO paperwork is unconfirmed**, read from reporting rather than Steinberg's primary licence text; if it is
   wrong, the Windows low-latency story changes. **`clack-host`'s licence is also unread.**
5. **The format freezes the moment someone's song is in it.** Every file carries a `v1` marker from the first commit,
   and a migration path is designed before the second version, not after.
6. **A non-deterministic writer destroys the premise silently.** One reordered key or float round-trip and every diff
   becomes noise. The byte-identical-save test is the guard, and it is worthless until it runs in CI, which does run
   now and is green on `main` since 5 October 2026 (ADR-063).
7. **Two ignore implementations can disagree**, so `local/` gets committed by one path and hidden by the other. Pinned
   by test for Qontrol's scaffolds; GNOSIS's own tree needs its own fixture, which is not written yet.
8. **Waiting on substrate that is not finished.** Assets have started (M4.1 done, M4.2's royalties and nesting
   built); media and settlement still depend on M6, M8 and OPEN-1/2/4. Shipping GNOSIS's own versions while waiting
   breaks the substrate rule on day one and forks the platform.
9. **Scope.** Games are the flagship (ADR-009) and music is the strongest second, with the cleanest per-use story —
   and competing with the flagship for attention is a real risk to it, now that GNOSIS has a track (P4) beside it.

## Decided by the owner, 22 September 2026

Each answer is the owner's and is reversible.

1. **GNOSIS's home is under `products/`**, `products/gnosis/`, as ADR-050 (accepted 28 September 2026) lays out.
   Its track is P4.
2. **Its asset requirements go into M2.3 before the format freezes.** ADR-047 decided the format on 22 September;
   the freeze comes before any SDK is published (`beta.wire-format-frozen`). They are checked below.
3. **BLAKE3 is accepted** (ADR-047). `media.lock`'s fingerprints and DRC-369's are one scheme.
4. **CLAP first, VST3 later.**
5. **On Windows, WASAPI first; ASIO later, through an SDK the user supplies**, so GNOSIS itself signs no Steinberg
   agreement.
6. **GNOSIS is Apache-2.0, like the repository.** Any dependency that would force otherwise is flagged:
   - `clack-host`'s licence is still **unverified** — read before it is added;
   - Symphonia is MPL-2.0, file-level copyleft: it does not change GNOSIS's licence, and changes to its own files
     stay MPL-2.0 (unverified against the crate's current `Cargo.toml`, to be checked when it is added);
   - ASIO is GPLv3-or-proprietary, which the user-supplied route in 5 keeps out of GNOSIS's distribution;
   - JUCE (AGPLv3 or commercial) stays rejected.
7. **GNOSIS's own devices keep no opaque state, ever.**
8. **One licence-vocabulary ADR, shared across the products**, before any of them invents one. Not yet written.
9. **The project format is named later, by the owner.**

### GNOSIS's asset requirements, checked against ADR-047

Checked on 22 September 2026 against the accepted record, so that any conflict is raised before the format
freezes rather than after.

| Requirement | ADR-047 | Result |
| --- | --- | --- |
| A song version is a nestable asset | `parent`, `depth` (decision 7); `MaxNestingDepth` 8 | **Met** |
| Stems are nested inside it | `MaxChildren` 64 per parent | **Met, with a limit to know.** A session with more than 64 stems nests them in groups — song, group, stem — which depth 8 allows |
| One fingerprint for `media.lock` and DRC-369 | BLAKE3-256 manifest root, algorithm tagged (decisions 1–2) | **Met**; `media.lock` already writes `blake3-256:` |
| A sampled stem carries remix royalties to its source | `derived_from`, `MaxRemixDepth` 16 (decisions 7, 11) | **Met** |
| A song's splits reach everyone owed | `MaxRoyaltyRecipients` 8 | **Decided: eight stays (ADR-057, the owner, 28 September 2026).** More than eight parties are grouped before mint, a publisher or collective taking one share and dividing it off chain. What this blueprint recommended: A track's writers, performers, publisher and sample sources can pass eight. The bound is an engineering value in ADR-047, not one of the owner's, and it becomes permanent when the format freezes. **Carried forward by the owner on 22 September 2026** as ADR-047's open item 1 and the migration inventory's Q-18; `beta.royalty-recipients` counts Q-18, so Beta cannot be met while the bound stands unexamined |
| A song version can be patched, or made permanent | `revise`, and `revisable`'s one-way switch (decision 10) | **Met** |
| Media lives somewhere before the Mesh | a store labelled temporary (question 13) | **Met**, and it never enters the wire format |
