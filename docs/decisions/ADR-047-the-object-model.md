# ADR-047: The object model — a BLAKE3 manifest root on chain, chunking beneath it, and a mint that pins a commit

> Editorial note, 13 November 2026: retired host/contact literals are redacted. Historical claims retain their original dates and are not current deployment evidence. ADR-130 establishes QORSYNC.DEV as the only public domain.

**Status:** **Accepted, 22 September 2026, by the project owner**, with the three questions reserved to the owner answered as recommended (Decision 13, rows 11 to 13). Proposed on 21 September 2026.
**The owner's summary, in plain words:** an asset is identified by a fingerprint of its contents, so it cannot be quietly swapped; how the bytes are stored can change without changing what the asset is; minting pins one exact commit, never a moving branch; the fingerprint method is tagged so it can be upgraded later; and an asset can be revised until a one-way switch makes it permanent.
**Carries out:** roadmap item **M2.3** — the wire format decisions FRAME forces: field bounds, token identity, the physics fixed-point encoding and the rental time unit (`docs/DIRECTION.md:235-236`). Ticking it needs an accepted ADR, because `beta.wire-format-frozen` reads *"The DRC-369 wire format and token identity are frozen, recorded in an accepted ADR, before any SDK is published"* (`docs/GATES.toml:674-677`, `done = false`, no evidence).
**Blocked ahead of it:** **M2.1** — the owner has not read the migration inventory, and AGENTS.md §7 forbids migration code until that is done. This record cites the inventory's findings; it does not stand in for reading it. **Cleared 22 September 2026:** M2.1 was ticked against the owner's review of a ten-line summary of the inventory's DRC-369 section.
**Open before the freeze:** two items — the first carried forward by the owner on 22 September 2026, the second found the same day while building M4.1 — see "Open items before the freeze" below. Accepting this record decided the format; it did not freeze it.
**Depends on:** [ADR-001](ADR-001-innovation-budget.md) (be boring where failure is silent), [ADR-009](ADR-009-universal-minting.md) (any file, from anyone), [ADR-025](ADR-025-drc369-on-pallet-nfts.md) (`pallet-nfts` is the ownership ledger), [ADR-028](ADR-028-provenance-from-events-archive-node-and-indexer.md) (history is events, read by an indexer), [ADR-035](ADR-035-eighteen-decimals.md) (scaled arithmetic uses the SDK's helpers).
**Reserved to the owner:** three of the thirteen questions in Decision 13, **answered on 22 September 2026**. The other ten are engineering.

## Context

**Nothing of this exists yet.** `construct_runtime!` mounts System, Timestamp, Aura, Grandpa, Balances, Session, ValidatorSet and (feature-gated) Sudo — no `pallet-nfts`, no `pallet-assets`, no DRC-369 (`chain/runtime/src/lib.rs:324-340`). DRC-369 is M4 and unstarted; the old module was deleted with `framework/` and nothing is carried (`docs/DIRECTION.md:174`). Studio is a tile pointing outside the launcher, at `retired-public-host/create` (`tools/qor-launcher/src/lib/systems.ts:115-124`); it moves inside at L7.5. M4 is blocked by nothing technical and nothing economic — only by M2.1 and M2.3 (`docs/SYSTEMS.md:131-141`).

**The chain does not commit us to a content hash.** The runtime hashes with `BlakeTwo256` and `H256` (`chain/runtime/src/lib.rs:94,196`), but that governs the state trie, storage keys and headers. File bytes are never in a call (`docs/architecture/MIGRATION_INVENTORY.md:216-226`, F-D1), so the runtime will never *compute* a content hash: it stores 32 opaque bytes and compares them. The choice is free in exactly one direction — for as long as the runtime never has to **verify** a proof over that hash.

**Two substrate gaps, named and not filled here.**

- **The Mesh does not exist.** F-D1 puts file bytes off chain, on the Mesh. The Mesh protocol is **M8.1** (`docs/DIRECTION.md:396`); the launcher's Mesh page is a placeholder at L7.4 (`docs/SYSTEMS.md:53`). M4 can therefore mint a fingerprint of bytes the protocol has nowhere to put. What a seeder proves is **U-6**, undefined (`docs/economics/OPEN_QUESTIONS.md:118-122`).
- **Qontrol is defined nowhere in this repository** — zero occurrences in `docs/`, checked 21 September 2026, before its Projects surface and helper landed the same day (`5a44207`) and before `docs/blueprints/qontrol.md`; neither changes the fact this record needs. The owner has settled its engine: gitoxide for every read path, libgit2 for stage and write-tree only, in a sidecar process, both behind Qontrol's own interface. That settles the one fact this record needs: **Qontrol is git-compatible on disk, so its commit ids are git object ids — SHA-1 today, SHA-256 under Git 3.0 — and never BLAKE3.**

### What FRAME and `pallet-nfts` force — not negotiable

| Forced | Shape | Where |
| --- | --- | --- |
| Encoding | SCALE, everywhere | `codec` in `chain/runtime/Cargo.toml:16` |
| Account, signature, address | `AccountId32`/Sr25519 (ADR-023), `MultiSignature`, `MultiAddress` (ADR-041) | `chain/runtime/src/lib.rs:56-57,68,71` |
| Asset identity | `(CollectionId, ItemId)`, permanent from the day an SDK ships; `CollectionId` must be `Incrementable` | `pallet-nfts` `Config` |
| Every field bounded | `BoundedVec` under `StringLimit`, `KeyLimit`, `ValueLimit`, plus `ApprovalsLimit`, `ItemAttributesApprovalsLimit`, `MaxTips`, `MaxDeadlineDuration`, `MaxAttributesPerCall`, `Features`. The bounds become wire format | F-D1, `pallet-nfts` `Config` |
| Deposits exist, and who pays | `mint` charges the caller even when minting to another account; `force_mint` the collection owner; `mint_pre_signed` the recipient; for `set_metadata` the code charges the collection owner while the documentation says the signer | ADR-025:53-63 |
| Deposits and fees precede a creator's first CGT | "Mint without holding CGT first" (M4.4) means someone else covers both, and who is undecided | F-D7, F-Q1 |
| Nesting is not provided | No item-owns-item relation; cycles must be refused (R-2, M4.5) and no tree may be walked inside a transfer's weight | F-D3 |
| Royalties bind only to chain-settled sales | A plain transfer names no price, so nothing can be taken from it. True of every chain | F-D5 |
| Physics cannot stay floating point | Fixed-point, with a published quantisation rule | F-D4 |
| Events are the only history | Cleared each block; state pruned past 256 blocks; what an event omits is gone | F-D2, ADR-028 |
| Every call carries a benchmarked weight | Sixteen call types in the old surface | F-D8 |

Everything else below is free, and is chosen here.

## Decision

1. **The content fingerprint is BLAKE3-256**, carried as a 32-byte root. It is a Merkle tree over the bytes by construction, so any byte range can be verified against the root the chain already holds with an O(log n) proof — roughly twenty hashes for a gibibyte. Choosing it now is choosing **not** to invent a Merkle layer for U-6 later, and a Merkle layer invented later would enter the wire format and be frozen. ADR-001, applied where failure is silent.

2. **The algorithm is tagged, not assumed.** A SCALE enum `HashAlgo { Blake3_256 = 0, Sha2_256 = 1, Blake2_256 = 2 }`. One byte per asset, and the escape hatch for the fact above: a Qontrol commit id is a git object id, never a BLAKE3 root. Without the tag, a second algorithm after `beta.wire-format-frozen` is a breaking change.

3. **The chain stores one fixed-size reference per asset, and nothing that varies with the file.**

   ```rust
   struct ContentRef { algo: HashAlgo, root: H256, size: u64 }   // 41 bytes, constant, forever
   ```

4. **`root` is the hash of a manifest, never of a file, and never a location.** A song is one file; a game asset is a mesh, three textures, a material and a physics body; a stem pack is twelve WAVs. A manifest root is 32 bytes for all of them, so no bounded list of file hashes enters the format and is wrong the first time someone uploads thirteen stems. The manifest is itself content-addressed, so the root is also the fetch address and verification is transitive: root → manifest → each file → any range of any file, one primitive from chain to byte.

5. **The manifest is off chain, SCALE, canonically ordered and version-prefixed.** Two encoders of the same content must produce identical bytes, or one asset has two identities. JSON cannot promise that.

   ```rust
   struct Manifest { version: u16, entries: Vec<Entry>, created: u64,
                     source: Option<SourceRef> }              // the Qontrol commit, see 8
   struct Entry    { path: String,        // relative, normalised, no ".."
                     media_type: String,  // IANA
                     content: ContentRef, // BLAKE3 root of that file
                     role: Role }         // Primary | Preview | Source | Component | Licence
   ```

6. **Chunking lives below identity and never touches it.** Identity and range verification are BLAKE3 over the whole byte stream at 16 KiB chunk-group granularity. Transfer, storage and delta patching use **FastCDC**, normalisation level 2, **min 256 KiB, target 1 MiB, max 4 MiB**. Boundaries, counts and parameters appear nowhere on chain and in no event, so the chunker can be retuned or replaced per client with no coordinated release. 1 MiB gives roughly 50,000 chunks for a 50 GB build, an index a player's machine can hold; 4 MiB keeps one chunk in one transfer window.

7. **The per-asset record in `pallet-drc369`** — `pallet-nfts` keeps ownership; this is the delta.

   ```rust
   origin: ContentRef          // immutable after mint
   current: ContentRef         // == origin at mint
   revisable: bool             // one-way: true -> false
   derived_from: Option<(u32,u32)>, remix_depth: u8      // remix, immutable
   parent: Option<(u32,u32)>, depth: u8                  // nesting, R-2 enforced
   state: AssetState           // level: u32, xp: u64, stats: BoundedVec<(u16,i64), MaxStats>
   physics: Option<Physics>    // FixedU128 / FixedI128, half-to-even at 10^-6
   ```

   The rounding rule is as much a part of the format as the type: two SDKs that round differently mint two different assets from one upload.

8. **Minting pins a version, and the pin is by hash, always.** `origin` is set at mint to the manifest root and is never writable again. The manifest's `source` names the Qontrol commit id and, for a human, the branch it came from. Qontrol checks at mint that the named commit's tree hashes to the entries' roots; the chain cannot check it and does not try. **On the next commit after a mint, nothing happens on chain** — the repository advances, the asset still names what it pinned. That is not a limitation; it is the entire meaning of a mint.

9. **An asset may never name a branch.** It defeats the spend test (ADR-002): someone who spent CGT on `main` bought a name whose referent the seller still controls. It destroys provenance permanently — under ADR-028 an event reading *"asset 7 now tracks main"* records no fact about any bytes, and there is no later repair. And branches move backwards: force-push exists, and an asset whose content can silently regress is not an asset. The launcher may *display* "minted from `main` at `a1b2c3…`"; the chain stores the hash.

10. **Carrying a new version is a separate, signed, visible act.** `revise(collection, item, new: ContentRef)` requires the owner and `revisable`, writes `current`, and emits `Revised { from, to, by }`. `revisable` is set at mint, can be turned **off** once by the owner and never back on, so a creator can promise permanence and nobody can revoke it afterwards. `origin` stays readable regardless.

11. **Three provenance chains, kept separate.** Content: commit → parent, off chain, verified by hashes. Asset: mint → transfers → revisions, on chain as events, reconstructed by the indexer (ADR-028). Remix: `derived_from`, one immutable field, read directly. Remix depth is checked **at mint** and refused past the bound, so no settlement path walks the graph — F-D3's rule that no tree is walked inside a transfer's weight.

12. **Events carry everything the indexer needs, because it can recover nothing they omit (F-D2).** `Minted { collection, item, owner, by, origin, derived_from }` · `Revised { collection, item, from, to, by }` · `Locked { collection, item }` · `Nested { parent, child, by, depth }` · `Unnested { parent, child, by }` · `Sold { collection, item, from, to, price, royalties }` · `StateChanged { collection, item, level, xp, by }`.

13. **The free values, and what the owner must answer yes or no to.** Ten are engineering; **three are not**.

    | # | Question | Recommendation | Owner's? |
    | --- | --- | --- | --- |
    | 1 | Content hash: BLAKE3-256, SHA-256 or BLAKE2-256? | **BLAKE3-256** | no |
    | 2 | Tag the algorithm or hard-code one? | **Tag it** | no |
    | 3 | Manifest hash or file hash on chain? | **Manifest, always** | no |
    | 4 | Chunker and parameters | **FastCDC, 256 KiB / 1 MiB / 4 MiB, off chain** | no |
    | 5 | `CollectionId` / `ItemId` | `u32` / `u32` (Asset Hub's values) | no |
    | 6 | `StringLimit` / `KeyLimit` / `ValueLimit` | `256` / `64` / `256` | no |
    | 7 | `MaxNestingDepth` / `MaxChildren` / `MaxRemixDepth` / `MaxStats` / `MaxRoyaltyRecipients` | `8` / `64` / `16` / `32` / `8` | no |
    | 8 | Physics type and rounding | `FixedU128`/`FixedI128`, half-to-even at 10⁻⁶ | no |
    | 9 | Rental time unit (F-D6) | **Block number** — no clock dependency, no drift, no timestamp surface | no |
    | 10 | May an asset name a branch? | **No** (Decision 9) | no — but say yes explicitly if you disagree |
    | 11 | `revisable` at all, or pure immutability? | **Yes, with the one-way lock.** Game assets are the flagship (ADR-009) and must be patchable; a work sold as final gets `origin` plus the lock | **yours — product.** **Answered 2026-09-22: yes** — revisable until a one-way switch makes it permanent |
    | 12 | One collection per single-file mint, or one "singles" collection per creator? | **One per creator**, created on first mint, so a creator owes one collection deposit rather than one per upload | **yours — it changes what a new creator owes (F-D7).** **Answered 2026-09-22: one singles collection per creator** |
    | 13 | Where content lives before the Mesh exists (M8.1) | **Name it, label it temporary, keep it out of the wire format** — the format says "a BLAKE3 root", never where to fetch it | **yours — scope.** **Answered 2026-09-22: a store labelled temporary until the Mesh.** Which store is named when M4.1 first needs one |

    None of these is an economic value. Royalty shares are a `Permill`; deposit and fee *amounts* are configuration this record does not set — which fee classes exist at all is **U-4**, the burn share per class is **OPEN-4**, the issuance rate is **OPEN-1**. Scaled arithmetic uses the SDK's helpers, never a hand-written `a * b / c` (AGENTS.md §5, ADR-035).

## Consequences

**What it costs.** The pinned `sp-io` has no BLAKE3 host function — `keccak_256`, `keccak_512`, `sha2_256`, `blake2_128`, `blake2_256` and the `twox_*` family only, with zero occurrences of `blake3` (verified in `sp-io` 48.0.0, the version `chain/Cargo.lock` pins under ADR-022). If the runtime ever has to *verify* a BLAKE3 proof — the obvious U-6 design — it must compute BLAKE3 in wasm at a benchmarked weight, or gain a host function, which couples node and runtime and needs a written reason under AGENTS.md §7. This record does not decide that, because at M4 the chain stores a root and verifies nothing. The deferral is recorded so it is not decided by accident.

**What it makes possible.** A seeder can prove it served a specific range of a specific asset against the 32 bytes the chain already holds, without a second Merkle design. Qontrol and the Library can delta-patch versions without the chain knowing. A DRC-369 identity is a function of bytes and nothing else, which is what "a standard others adopt" (ADR-009) requires.

**What it forecloses.** Assets that track a moving reference. Chunk-level deduplication between Qontrol's git object store and the Mesh blob store, since those objects are SHA-1 or SHA-256 while content is BLAKE3 — the tag makes that cost visible instead of accidental. And storing a file's own hash on chain, which no later version can add without a breaking change.

### Alternatives rejected

- **SHA-256 as the fingerprint.** It has the `sp-io` host function, hardware acceleration on every current CPU, and Git 3.0 behind it, so a Qontrol tree hash and an asset fingerprint would be one primitive. It lost on one property: a whole-file digest proves nothing until the whole file is re-read, so U-6 would have to invent a Merkle layer, and that layer's arity, leaf size and domain separation would enter the frozen format. Reversible via `HashAlgo = 1`.
- **BLAKE2-256, the chain's own hash.** Has the host function too, and would make `ContentRef::root` type-identical to `Hash`. Same missing tree as SHA-256, with none of SHA-256's Git compatibility or hardware support. Reusing the state-trie hash for content is a coincidence of names, not a design.
- **An IPFS CID over UnixFS.** Rejected on a documented failure, not on taste: a UnixFS CID is a function of the content *and* the chunk size, the DAG layout and the CID version, so two people importing the same bytes with different chunker settings get two identities for one work. A standard whose identity moves with the uploader's tool settings cannot be adopted by anyone else, which is ADR-009's whole requirement. Its addressing ideas survive in Decision 4.
- **Multihash's self-describing varint** instead of the `HashAlgo` enum. Identical benefit — one byte, algorithm agility — at the cost of depending on a numeric registry Demiurge neither controls nor can freeze, inside a format we are freezing. A three-variant SCALE enum is smaller and ours.
- **A bounded list of file hashes on chain.** Wrong the first time someone uploads thirteen stems against a bound of twelve, and it puts F-D1's per-field deposit and weight cost on every asset for a structure that varies per upload.
- **A URL, gateway address or Mesh peer id on chain.** A location is not content: it rots, it can be repointed by whoever runs it, and it is not what was bought. Decision 4 makes the root the fetch address, so locations stay a client concern.
- **restic's or borg's model, where chunk boundaries define the repository format.** Proven code, wrong property: in both, the chunker's parameters are part of the on-disk format, so retuning them is a format migration. Keeping the chunker invisible to the chain (Decision 6) is exactly what those designs give up.
- **A new item per version, instead of `revise`.** It breaks entitlements (the Library would re-grant on every patch), breaks nesting (children point at the superseded parent), multiplies item deposits per patch, and splits one work's provenance across an unbounded set of `(collection, item)` pairs the indexer must re-stitch.
- **Pure immutability, no `revise` at all.** Correct for a work sold as final, fatal for the flagship case: a game asset that can never be patched is dead on arrival (ADR-009). The one-way lock gives both, and makes which one applies visible before anyone spends.
- **`pallet-transaction-storage` for small files.** F-D1 lists it as a candidate — up to `MaxTransactionSize`, 8 MiB by default, with storage proofs — and says whether it fits the Mesh design is open. It stays open; nothing here depends on it either way.

### Feasibility, honestly

**Reused:** `pallet-nfts` and `pallet-nft-fractionalization` (ADR-025; `pallet-assets` is present for fractionalization only, which ADR-031 states is not a reversal of it), `sp-arithmetic`'s `FixedU128`, the `blake3` crate, a FastCDC crate, `subxt` in the launcher (ADR-040), the vault's Sr25519 signing (ADR-039). **New:** `pallet-drc369` (fingerprint, nesting with R-2, state and XP, physics, owner index and runtime API), `pallet-drc369-royalties`, the manifest codec, and Studio's upload-hash-mint path.

**Three months, one founder and an agent:** this ADR accepted, M2.1 read, then M4.1, M4.2 and M4.5, plus Studio moved inside the launcher — realistic because ADR-025 already made the hard architectural call and none of it needs an economic value. **Twelve months:** M4.3 (rental, fractional ownership, fixed-point physics); M4.4 sponsored deposits, which cannot be built until U-4 says what a fee class is and who a sponsor is; the SDK, the viewer, the indexer, an archive node. **Beyond:** the Mesh (M8.1), U-6, and any on-chain verification of a storage proof. **"Any file" does not mean the chain holds it; it means the chain holds 41 bytes about it — and until the owner answers question 13, it does not mean anyone but the creator can fetch it.**

## Open items before the freeze

Added on 22 September 2026, after acceptance. These change no decision above: each names a value this record chose that
must be examined again before `beta.wire-format-frozen`, because the freeze makes it permanent. Each is answered by a
later ADR, never by editing this one.

1. **Answered 28 September 2026 by [ADR-057](ADR-057-eight-royalty-recipients.md): eight stays.** **`MaxRoyaltyRecipients` = 8 (decision 13, row 7).** Carried forward by the owner in the acknowledgement that ticked
   M2.1. **The reason is GNOSIS's song-credits case** (`docs/blueprints/gnosis.md`, "What GNOSIS needs from the asset
   format"): a track's writers, performers, publisher and the sources of its samples can pass eight, and after the
   freeze raising the bound is a breaking change. Recorded as the migration inventory's **Q-18**, which
   `beta.royalty-recipients` in `docs/GATES.toml` counts, so Beta cannot be met, and the format cannot be frozen, while
   the bound stands unexamined. The question is only whether eight is enough; no royalty share, rate or price is part of
   it.
2. **What the root is a function of** (decisions 5 and 8, against "What it makes possible"). Found on 22 September
   2026 while building the manifest for M4.1. Decision 5 puts `created` and `source` inside the manifest, and
   decision 8 has `source` name the commit and, for a person, the branch. So the root depends on the files **and** on
   the commit's id, its time and the branch's name: two commits with identical files have two roots, and one commit
   minted from two branches has two roots. "What it makes possible" says an identity is "a function of bytes and
   nothing else". Both cannot hold. The launcher follows decisions 5 and 8 as written — `created` is the pinned
   commit's time, so the same commit always makes the same manifest, and `source` carries the branch — and the chain
   stores the commit on its own as well (ADR-052). A later record says which holds before the freeze: the manifest
   as decided, or a manifest of the files alone with the commit and branch kept beside it. Recorded as the migration
   inventory's **Q-19**, which `beta.manifest-identity` counts.
3. **Whether the media-type table is part of an asset's identity** (decision 5's `media_type`, against "What it
   makes possible"). Found on 22 September 2026, when the launcher's table was widened at the owner's request so
   that video, archives, documents and models are named rather than recorded as raw bytes. Every file's media type
   sits **inside the hashed manifest**, and the table that produces it lives in the client: two clients with
   different tables compute two roots for the same bytes, and a client that widens its table — as this one did that
   day — gives the same project a new root. This record says only "IANA" and names no table, so today nothing
   contradicts it and nothing is specified either. A later record says which holds before the freeze: the mapping is
   part of the format and is published with it; or the manifest carries no media type at all and a reader infers
   one; or the field stays for readers and is excluded from the hash. Recorded as the migration inventory's
   **Q-20**, which `beta.media-types` counts.

## What this record does not decide

- **Any economic value.** No fee class, burn share, deposit amount, issuance rate or genesis split appears here (U-4, OPEN-4, OPEN-1, OPEN-2). Nothing here creates CGT, and no path that creates CGT outside `--dev` is proposed.
- **U-6.** What a seeder proves, to whom and how often, and therefore whether the runtime ever verifies a BLAKE3 proof.
- **Whether gated content on the Mesh is encrypted.** Content-defined chunking leaks information about plaintext to anyone who observes boundaries, and per-buyer encryption and deduplication are mutually exclusive. That belongs with U-6 and ADR-006's access-gating sink, not inside a chunker.
- **Where bytes live before the Mesh (question 13), and Qontrol's own object storage.** If Qontrol would need its own object store, its own author identity or its own way to pay for storage, that is a substrate gap, recorded and not filled locally.
- **Rental, fractionalization and royalty mechanics** beyond the fields and bounds above; question 9 fixes only the time unit.
- **Pallet names** beyond ADR-025's and ADR-032's (AGENTS.md §8), and **the SDK's shape** — this record freezes what the SDK must encode, not what it looks like.
