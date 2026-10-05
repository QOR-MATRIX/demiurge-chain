# The Demiurge ecosystem

**What this is.** The overview the six product blueprints hang from. Read it first; each blueprint
assumes it. Written 21 September 2026 against the tree.

**What this is not.** Not a roadmap and not a status table. The only roadmap is
[`docs/DIRECTION.md`](../DIRECTION.md); the status of every system this repository names is
[`docs/SYSTEMS.md`](../SYSTEMS.md). This document states one thing about status — **all six products are blueprints; two of them have a
first surface in the launcher and nothing more** — and otherwise points at the milestone that gates
each one. Those two are Qontrol, whose Projects view landed on 21 September 2026 (commit `5a44207`),
and QFX, whose backdrop landed the same day (`75b2ead`). A surface is not a product. All six now have
a product track in `DIRECTION.md` (P1 to P6), a gate in `GATES.toml` and an entry in `SYSTEMS.md`;
none has an accepted ADR.

**Naming.** QOR Engine, Qontrol, GNOSIS, QFX, Market and Stream are working names. `STREAM` in
particular is a placeholder used so the music-distribution product has something to be called. Anything
published to other developers — crates, pallets, calls, events, storage, SDK and RPC surfaces — takes
plain names under ADR-032 and needs the owner's approval (`AGENTS.md` §8).

**A seventh product, 4 October 2026.** The owner named **ARQADE**, the gaming platform, after this document was
written: [`arqade.md`](arqade.md), track P7, gate `arqade`, ADR-069 (accepted 4 October 2026). It uses the same five things in §1 and
fills none of them locally. It found four substrate gaps none of the six had hit (G-14 to G-17, below), and its store pages hit G-3.

---

## 1. One substrate, six apps

Every product in this ecosystem uses the same five things. This is the rule, not a preference:

| Every product… | uses | which is |
| --- | --- | --- |
| authenticates with | **QOR ID** | `services/qor-auth/`, real today. No product gets a login screen of its own |
| stores creations as | **DRC-369 assets** | M4, unstarted. One asset format for all six, or six formats on day one |
| versions them with | **Qontrol** | blueprint. Git-compatible on disk |
| distributes through | **the Mesh** | M8, unstarted. No product grows a CDN, a bucket or a package registry |
| settles in | **CGT** | the unit is real; fees, issuance and treasury are not |

**The consequence, stated so nobody has to infer it.** If a design would need its own identity, its own
asset format, its own storage or its own payment rail, that is a **substrate gap**. It gets recorded in
§4 and it does not get filled locally. A local fill is how a platform ends up with two identities, four
licence vocabularies and a second place keys live.

Creators are paid when their work is used, licensed or remixed. Seeders are paid for storing and
serving. Validators are paid for securing the chain. Nothing in this ecosystem is a reason to hold CGT
rather than spend it (ADR-002, ADR-008).

---

## 2. The six products

Every row's status is **blueprint**. Two rows have one landed surface each, noted in the row; nothing
else below exists.

| Product | What it is, in one sentence | Built on | First dependency |
| --- | --- | --- | --- |
| **QOR Engine** | A game and scene engine that is a custom Godot build tracking upstream, with QOR's additions as modules and editor plugins | **Godot 4.6+, MIT.** Not a fork, not from scratch. Rust additions via `gdext` | None for the build itself. **M4** for DRC-369 assets as live nodes; **M8** for delivery |
| **Qontrol** | Version control for creative work: git-compatible on disk, with large binaries, typed merges and publish-as-provenance | **gitoxide** for every read path; **libgit2 in a sidecar process** for stage and write-tree only; both behind Qontrol's own interface | None for local history — **the Projects surface and the sidecar are built** (`5a44207`): open, scaffold, status, log, branches, commit. **M4** for publish → mint; **M8** for any remote |
| **GNOSIS** | Music and audio production whose project format is a mergeable tree rather than one opaque binary file | CPAL, Symphonia, CLAP first via `clack-host`; the format is the new part | **Qontrol** (the format's payload is Qontrol's). **M4/M6** for settlement |
| **QFX** | The launcher's living visual layer, authored by creators and paid for when used: a backdrop, a binding to the chrome, and a scene editor | WebGL2 in the existing webview for layers 1–2; layer 3 is an **editor plugin in the QOR Engine custom build**, not a second tool | **Layer one is built** (`75b2ead`): one WebGL2 backdrop, the Ambience setting, and the chrome's contrast guarantee. ADR-051 was accepted on 22 September 2026, with the check narrowing approved; layers two and three are P2.4 to P2.6. **M4** for theme identity |
| **Market** (with **Library**) | Where a creator lists work and a buyer spends CGT on it, and where what you bought installs, patches and launches | `pallet-nfts` under DRC-369 (ADR-025); the launcher's existing host/webview split | **M4.1** fingerprint, **M6.4** fees, **M8.2** entitlements, **M8.3** the staking sink, **L6** installers |
| **Stream** *(placeholder name)* | Music distribution and licensing over the Mesh, with plays deciding who is paid | Opus (RFC 6716) and the Mesh's swarms | **U-6** — seeder work verification. Until that is answered, this is a research project, not a product |

Market and Library already had roadmap items (L7.1, L7.2, M8.2, M8.3). The other four had no number
and no entry in `SYSTEMS.md` when this was written; **on 21 September 2026 all six got a product track
in `DIRECTION.md`, P1 to P6, and an entry in `SYSTEMS.md`.** What none has yet is an accepted record
defining it.

---

## 3. How they compose: two paths a creator walks

### A musician

1. She signs into the launcher with her **QOR ID** — the same account she plays under. GNOSIS asks for
   nothing else.
2. She starts a project from a GNOSIS scaffold. The project is a **tree of small text files** — one per
   track, per clip, per device chain — with audio held by fingerprint in a side store rather than
   inline. Text, diffable, mergeable: **Godot's `.tscn` is the precedent**, and GNOSIS takes that
   property as a requirement rather than a nicety.
3. Every save is a **Qontrol** commit: local, free, offline. **A commit is not a mint**, and the product
   says so, because the alternative is a fee class nobody has decided.
4. Her collaborator's changes merge, because two people editing two different tracks touch two
   different files and the merge driver is typed rather than line-based.
5. She publishes a version. That version becomes a **DRC-369 asset** whose content fingerprint is the
   hash Qontrol already computed — one hash, computed once. *(M4. Does not exist.)*
6. **Market** lists it. A studio spends **CGT** for a sync licence, a stem pack or a remix right: access
   gating, the second of ADR-006's five sinks. *(M6.4 and M8.3. Does not exist.)*
7. Someone samples a stem. The remix's asset names its parent, and the parent is paid because the
   derivation is recorded, not because anyone was honest. *(M4.2. The share is per-asset terms; the fee
   class that carries it is **OPEN-4** and no value for it appears anywhere.)*
8. The **Mesh** serves the bytes and seeders are paid for the bandwidth actually consumed. *(M8.1,
   blocked on U-6.)*

**Steps 1 to 4 are buildable now with no chain milestone at all. Steps 5 to 8 are chain work that has
not started.** That split is the honest shape of every path in this ecosystem.

### A theme author

1. Same QOR ID, same launcher.
2. She builds a scene in **QFX's editor**, which is an editor plugin in the QOR Engine custom build.
   There is no second scene tool to learn and no second scene format to export.
3. She versions it in **Qontrol**. The scene is text, so it diffs — and a **preset is literally a diff
   against a named parent**, which makes it the cleanest demonstration of remix royalties anywhere in
   the roadmap: the derivation is mechanical and verifiable.
4. She publishes it as a **DRC-369 asset** on **Market**, and is paid when someone uses it. *(M4 and M8.
   Until then, themes are compiled in and **nothing is shareable** — a local theme registry with its own
   download URL would be a second distribution rail and is refused.)*
5. One constraint holds throughout: **the default stays as chosen.** No scene is the default theme, and
   "still means still" is enforced by a test that compares two screenshots a second apart, not by
   convention.

---

## 4. The substrate gaps

The honest list of what the substrate does not provide, collected from all six blueprints. Each is
named here so that no product fills it locally.

| # | Gap | Who hits it | What closes it |
| --- | --- | --- | --- |
| **G-1** | **No asset format.** Its wire format is **decided** since 22 September 2026 (ADR-047, accepted; M2.3 ticked), and **its first pallet exists since the same day** (M4.1): mint, revise, make permanent, owner enumeration | All six. Every "publish" step in §3 | **Narrowing.** M4.1 is done; nesting, royalties and the rest are M4.2 to M4.5. Two open items stand before the freeze (Q-18, Q-19) |
| **G-2** | **No content fingerprint scheme.** The chain stores an opaque 32 bytes it never computes, and nobody has chosen what produces them | Qontrol's store key and DRC-369's identity must be **one hash**. Also GNOSIS's media refs, Market's download verification | **Closed on 22 September 2026 by ADR-047, accepted:** a BLAKE3-256 manifest root, its algorithm tagged. Qontrol's store, DRC-369's identity, GNOSIS's media refs and Market's verification share it |
| **G-3** | **No store for the bytes.** The Mesh is M8.1. **Since 22 September 2026 a temporary store exists, as the owner named it:** `temporary-content-store/` in the launcher's data directory, on one machine, labelled temporary | All six. A minted asset's bytes are on the creator's machine and nowhere else | **M8.1**, blocked on U-6 |
| **G-4** | **No licence vocabulary.** The public viewer is meant to show "its licence terms"; nothing defines what a term is | Stream needs per-play terms, Market resale, Library entitlement, QFX remix — **four vocabularies for one concept** | **One ADR, before any of the four is built** |
| **G-5** | **No entitlement format, and no offline proof.** The launcher cannot ask the chain "does this user own this" | Library and Stream need the same primitive. The obvious offline answer puts QOR ID's attestation behind access to value — exactly what ADR-017 and R-3 exist to prevent | **M8.2**, plus a recorded decision on offline launch |
| **G-6** | **No settlement above a balance transfer.** No transaction payment, no treasury, no issuance | Every product that charges for anything | **M6**, gated on **OPEN-1**, **OPEN-2**, **OPEN-4**. No rate, split or share may be invented to get moving |
| **G-7** | **U-6 is unanswered: what a seeder proves, to whom, how often** | The Mesh pays for unverifiable claims without it. **Stream has no play counting at all** unless U-6's answer produces one | **U-6**. If it is answered with signed client receipts a seeder redeems, Stream gets counting for free; if with a storage proof, Stream needs its own decision and probably should not exist yet |
| **G-8** | **No per-client identity in QOR ID.** A session records nothing about what created it, so "revoke this product's access" is inexpressible | The app host: six products sharing one session share one revocation switch | **Closed 5 October 2026** by ADR-043 §3 as built in ADR-073: a session records the app that created it, and an app's session can be ended on its own |
| **G-9** | **No exclusive-lock primitive.** Nothing holds a short-lived, identity-scoped claim over a path | Qontrol — and it is the feature creators on binary files actually want. QOR ID authenticates the claimant; nothing stores the claim | Undecided. Chain state is the wrong shape and latency, and there is no fee class to pay for it (OPEN-4). **Do not build a lock server with its own accounts** |
| **G-10** | **Exported files carry no Demiurge identity.** A `.glb` or `.flac` that leaves says nothing about who made it or where royalties settle | All six, at the moment a creator leaves | A **single C2PA assertion shape**, decided once for all six, not per product |
| **G-11** | **The provenance / royalty / entitlement graph has no format.** C2PA does origin, not economics | All six. If Demiurge publishes none, "a creator can leave" means abandoning the graph | **This is the one place where inventing a format is correct** (ADR-001: innovate where failure is loud). The specification can be drafted long before M4/M6/M8 can implement it |
| **G-12** | **No signed update channel and no installers** | Nothing can be installed, patched or verified — Library included | **L6** |
| **G-13** | **Four of the six products are not defined anywhere**, not even as names | Qontrol, GNOSIS, QFX, Stream | An entry each in `SYSTEMS.md` and a number in `DIRECTION.md`, **before any code**. **Closed on 21 September 2026** by P1 to P6 and a `SYSTEMS.md` entry each — after Qontrol's and QFX's first code, not before. A defining record for each is still missing |
| **G-14** | **No randomness source.** Aura has no VRF and no randomness pallet is mounted, so nothing on chain can make an allocation verifiable | ARQADE: card packs, any chance mechanic. Any product that ever draws | **An ADR of its own** (added 4 October 2026, ADR-069). Commit-and-reveal by a game authority still needs withholding and timeout rules. No pack is sold for CGT before it |
| **G-15** | **No creator-set collections and no enforced editions.** `Drc369::mint` mints to the signer, in the signer's one singles collection, and no supply cap can be set | ARQADE's card universes and sets; any limited edition in Market | **An ADR of its own** (ADR-069). Until then a set is declared in the manifest and no product claims a cap the chain does not enforce |
| **G-16** | **No account-bound asset.** The runtime's call filter lets `Nfts::transfer` through and nothing locks one item's transfer | ARQADE's trophies, if they are to stay with the player who earned them | **An ADR of its own** (ADR-069). Until then trophies are transferable and say so |
| **G-17** | **No primary sale.** `Drc369Royalties::buy` sells an asset that already exists and is listed; nothing mints a copy per buyer or enforces an edition at sale | ARQADE: game licences and in-game items sold to many players; any Market storefront | **An ADR of its own** (added 4 October 2026, ADR-071). Until then copies are minted ahead and listed one by one |

### The three to decide first

1. ~~**The content fingerprint scheme (G-2), inside M2.3.**~~ **Decided on 22 September 2026 (ADR-047).**
   It is the one value that Qontrol's store, the chain's asset identity, Market's download verification and
   GNOSIS's media refs all share, and it becomes expensive the moment an SDK is published.
2. **The licence-and-entitlement vocabulary (G-4, G-5) as one ADR**, before Stream, Market, Library or
   QFX has a scope. Four products will otherwise each invent one.
3. **U-6 (G-7).** It decides whether the Mesh can pay anybody honestly, and whether Stream is a product.

One further owner decision sat outside this table because it is not a substrate gap. **M2.1** — the
owner reading the migration inventory — blocked every line of chain work in every blueprint. It cleared on
22 September 2026: the owner acknowledged a ten-line summary of the inventory's DRC-369 section, and M2.1
was ticked against that summary.

**A note on the design checks, because this document was drafted a few hours before they changed.**
QFX layer one was forbidden by the launcher's own CI: five design checks failed on any literal
implementation, and those checks are read by a release-gate criterion. The narrowing landed on
21 September 2026 — `check-design.mjs` now exempts `src/qfx/` and nothing else, the exemption was
proven narrow by planting a `requestAnimationFrame` in `ChainView.tsx` and watching the check fail,
and the scope change is recorded in `GATES.toml`'s change log (`42d00c3`). Layers two and three make
the same demand again and do not inherit that answer.

---

## 5. What exists today

| Thing | State |
| --- | --- |
| **The chain** (`chain/`) | Real. Aura produces, GRANDPA finalises, the validator set comes from governance. 70 tests. **It moves CGT between accounts and, since 22 September 2026, mints DRC-369 assets** (M4.1) — no fees, no issuance, no treasury |
| **QOR ID** (`services/qor-auth/`) | Real. Sign-in, sessions, email verification, backup codes, agent registration. 125 tests against real Postgres and Redis when last run, on 21 September 2026; nothing in it has changed since |
| **The launcher** (`tools/qor-launcher/`) | Real. Sealed vault, QOR ID sign-in, transfers and mints proven end to end against a node, chain view, gates dashboard, themes, the Projects surface with Mint, an Inventory read from the chain, and the QFX backdrop. 116 host tests and 6 sidecar tests |
| **CGT** | The unit is real: 18 decimals, 1 CGT = 10^18 Sparks. **Supply, issuance, fees and treasury all need numbers nobody has decided** |
| **DRC-369** | **M4.1 done, 22 September 2026.** `pallet-drc369` over `pallet-nfts`: mint with a content reference and a pinned commit, revise, make permanent, owner enumeration. Nesting, royalties, state and physics are M4.2 onwards |
| **The Mesh** | **M8. Unstarted.** Blocked on U-6 |
| **The six products** | **None of them exists as a product.** Two have one landed launcher surface each (Qontrol's Projects, QFX layer one). All six have a product track (P1 to P6), a gate and a `SYSTEMS.md` entry; none has an accepted ADR |

---

## 6. Feasibility, honestly

One founder with an agent. "Reused" means somebody else already built it and it is MIT or equivalent.

**Within about three months, with no chain milestone at all:**
Qontrol's first slice — gitoxide reads, a libgit2 sidecar for stage and write-tree, scaffolds, local
history, local advisory locks, no remote. A rebranded Godot 4.6 build on Windows, Linux and macOS,
launched beside the launcher rather than inside it, with a Rust GDExtension exposing read-only QOR ID
identity and glTF/FBX proven in and out. GNOSIS's project-tree format — the specification, a
deterministic writer, a round-trip reader, a typed merge driver and a merge test suite — plus device
I/O, decode, transport and a basic mixer. QFX's contrast guarantee, tier ladder and
reduced-motion-by-ownership against a placeholder shader, *after* the ADR. The local SDK and the
app-host's socket, spawn tokens and peer credentials.

**Within about twelve months:**
CLAP hosting done *correctly* (loading a plugin and hearing it is weeks; correct is a year), plugin GUI
embedding, VST3 and AU. The QFX theme package format with host-side shader validation. A QOR Engine
editor build carrying Demiurge panels, and a `.tscn` semantic merge driver in CI. Publish → DRC-369
mint — **built with M4.1 on 22 September 2026** — and fork → remix provenance, gated on M4.2. Delta patching, and C2PA
manifests in exports.

**Beyond, or blocked:**
Anything Mesh-backed — Qontrol's remote, asset delivery, Library installs, Stream at all. Per-play
settlement and every licence fee (M6, OPEN-4). Cross-machine locks. A native GPU surface composited
under the webview. Consoles.

### What "full support" does not mean

- **Unity and Unreal: assets only.** glTF and USD interchange plus existing community migration tools.
  **Project round-trip is not achievable at any budget** — closed formats, closed source, and licence
  terms that forbid the product shape. Say this in public copy, not in a footnote.
- **Godot's rendering is not Unreal's,** and that is fine, because fidelity parity is not the target.
  QOR Engine competes on what Godot does not have: assets that are live, royalties that follow a
  remix, delivery over the Mesh, agent rails in the editor, settlement at the moment of use.
- **QOR Engine is a custom build tracking upstream**, with additions as modules and editor plugins —
  never a hard fork. Godot is MIT, so a branded distribution is shippable. The trademark line is
  **"QOR Engine, built on Godot"**, and nothing that implies endorsement.
- **GNOSIS in three months is not a DAW anyone finishes a record in.** It is the format plus a host.
  The format is the differentiator; the DAW is years.
- **Qontrol is git-compatible on disk, not a hosting service.** After a commit through the split path,
  gitoxide's status and `git status --porcelain` both report clean — that is pinned by test, because an
  index that lies is exactly the silent failure ADR-001 says to stay boring about.
- **"Publish" does not mean "mint" until M4.** A commit is free, local and offline. Minting is a
  separate, explicit act.
- **Per-play is a payout mechanism, not a demand sink.** If Stream ever exists, the demand comes from
  what a listener pays; plays decide who receives it. A pot funded by issuance instead would fail the
  spend test outright, and that condition is the first line of Stream's ADR.

---

## 7. Reserved names

`SYSTEMS.md` lists names the owner has used that the repository does not define. These blueprints
define **QOR Engine** and nothing else on that list. The rest stay reserved, **no scope is invented for
them, and none of these blueprints asks about them**:

- **Relays** — nothing beyond the name.
- **Agentic synchronisation** — nothing beyond the name, and *not* the agent rails of M5.2, which are a
  different, defined thing.
- **QOR Wallet** — `SYSTEMS.md` already records the question: whether this is a second name for the
  launcher's Vault or a second place keys live.
- **VYB Social** — merged into Social by the owner on 28 September 2026; the VYB name is dropped.
- **Demiurge Exchange** — `SYSTEMS.md` records why this one needs deciding before anything is built,
  and that record stands unchanged here.

Each needs a decision and a roadmap item before it needs a line of code or a subdomain. Defining one is
the owner's call, not a blueprint's.

---

## 8. Housekeeping this document creates

- `docs/blueprints/` is new. Every file in it must be added to [`docs/README.md`](../README.md)'s table,
  or it is not current.
- A blueprint is **not** an ADR. Where a blueprint recommends a decision — the fingerprint scheme, the
  licence vocabulary, the design-system amendment QFX needs, a pallet name — that decision becomes its
  own ADR in `docs/decisions/`, indexed in `DECISIONS.md`.
- Qontrol, GNOSIS, QFX and Stream needed entries in `SYSTEMS.md` and numbers in `DIRECTION.md` before any
  of them got code. They have both since 21 September 2026 — P1 to P6 — though Qontrol and QFX had code
  first.
