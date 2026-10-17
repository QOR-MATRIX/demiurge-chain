# ADR-095: The Mesh on iroh, seeders paid from the uploader's hosting escrow, published work and private backup

**Status:** **Accepted**, 17 October 2026, by the owner, who directed: "begin working on the Mesh and ensure that earning
CGT is a main mechanic and results in rewarded CGT for sharing / hosting / seeeding files and media to the Demiurge
ecosystem. Think of this like a in launcher torrenting network which allows any QOR ID logged in user to earn CGT for
seeding files and data for others to create a sort of cloud file storage mesh." Asked four questions, the owner chose:
**"Uploaders' escrow"** (recommended), **"Plus private backup"** (beyond the recommended published content alone),
**"iroh"** (recommended) and **"Opt-in with limits"** (recommended). **Carries out** [ADR-005](ADR-005-useful-work-mining.md)
(the Mesh is Demiurge's mining) and the Mesh-hosting sink of [ADR-006](ADR-006-demand-sinks.md). It decides **no
economic value**: no price, rate, pool, split or burn share (AGENTS.md §5; OPEN-1 to OPEN-4).

**Relates to:** [ADR-047](ADR-047-the-object-model.md) (BLAKE3 roots, 16 KiB verification groups, the temporary content
store this replaces), [ADR-002](ADR-002-value-from-spending.md) (the spend test), [ADR-004](ADR-004-issuance-and-burn.md)
(issuance designated in part for the Mesh, rate open), [ADR-008](ADR-008-language-discipline.md) (language).

## Context

Checked on 17 October 2026: the launcher's Mesh page is a placeholder. A minted asset's bytes live in
`temporary-content-store/` on the creator's PC only (`src-tauri/src/content/mod.rs`, ADR-047 question 13), so nobody
else can fetch them. The chain holds each asset's 41-byte fingerprint (`pallet_drc369::ContentRef`): an algorithm tag, a
BLAKE3-256 root and a length. Nothing proves that anyone stored or served bytes (U-6), and the perpetual issuance that
ADR-004 designates in part for the Mesh has no rate (OPEN-1), no pool (OPEN-2) and no curve (OPEN-3).

**iroh** (n0, MIT or Apache-2.0; `iroh` 1.3.0 of 28 September 2026, `iroh-blobs` 0.103.1 of 6 October 2026) is a Rust
peer-to-peer library: QUIC connections between devices dialled by public key, hole punching through home routers with
relays as fallback, and `iroh-blobs`, which streams a blob verified against its plain BLAKE3 hash in 16 KiB groups
(`IROH_BLOCK_SIZE = BlockSize::from_chunk_log(4)`, read in its source the same day). That is ADR-047's fingerprint and
granularity exactly: a seeder's bytes can be checked against the root the chain already holds, range by range, with no
second hash. BitTorrent fingerprints pieces with SHA-1 or SHA-256 and would need a second identity per file; libp2p would
need the verified transfer built on top.

## Decision

1. **The Mesh is a peer-to-peer network inside every QOR Launcher, built on iroh and iroh-blobs.** It lives in the
   launcher's Rust host (module `mesh`, commands `mesh_*`), where the vault and the chain client already are; it has no
   window of its own, so the owner's preference for Qt (16 October 2026) does not apply to it. A launcher's Mesh identity
   is an iroh node key made on that PC, never the vault's key; it is tied to the person's chain account by a statement
   the vault signs, so payment reaches the account and a stolen node key cannot move CGT. Content is addressed by the
   BLAKE3 roots of ADR-047, and every byte fetched is verified against them before it is kept or shown. The Mesh
   replaces the temporary content store: the same roots, served to others.

2. **Seeding is opt-in, with limits the person sets.** Off when the launcher is installed. Turning it on asks how much
   disk and upload bandwidth to give, and says plainly that seeding shows the person's network address to the peers
   they serve. In release 1 a launcher seeds only while it runs. Any person signed in with a QOR ID may seed.

3. **What the Mesh holds, release 1: published work and private backup.**
   - **Published work:** content whose root is on chain (DRC-369 assets, Cartridges, and what Market and ARQADE list),
     public, verified on arrival. Seeders refuse any root on a takedown list.
   - **Private backup:** a person's own files, encrypted on their PC before any byte leaves it, with keys derived from
     their vault (so the recovery phrase restores the backup too), split with erasure coding (*any k of n pieces rebuild
     the whole*) across several seeders, and repaired when seeders go away. A seeder holds ciphertext it cannot read.
   - **Not held:** arbitrary files shared in the open, as a torrent network carries them.

4. **Seeders are paid from the uploader's hosting escrow; the Mesh creates no CGT.** Whoever puts work on the Mesh
   (a publisher for their work, a person for their backup) deposits CGT into a hosting escrow for its roots, at a price
   **they** set for storage over time and for bytes served. Seeders are paid from that escrow, each period, only for work
   proven under decision 5. Unspent escrow goes back to whoever deposited it when they stop. The protocol fixes no price:
   offers are made by uploaders and taken by seeders. This is the Mesh-hosting sink (ADR-006): CGT spent for a service
   used, which passes the spend test (ADR-002). A subsidy from the perpetual issuance (ADR-004) may join it once OPEN-1
   to OPEN-3 are decided; until then nothing in the Mesh creates CGT, on the devnet or anywhere outside `--dev`. Whether
   a share of hosting payments is burned is OPEN-4, and none is until it is decided.

5. **Payment only for proven work.** A seeder is paid for storage it answers challenges for, and for bytes the receiving
   person signs for. The scheme (what is challenged, how often, which receipts count, what stops a seeder paying itself)
   answers U-6, and is proposed separately for the owner's review in
   [`../architecture/MESH_PROOFS.md`](../architecture/MESH_PROOFS.md); no payment is made before it is accepted.

6. **Names.** "The Mesh" for the network; "seeding" and "seeders"; "hosting escrow". The chain module that holds escrows
   and pays seeders is proposed as `pallet-mesh`, and is **not named until the owner approves** (AGENTS.md §7, §8).
   Copy follows ADR-008: seeders are paid for storing and serving; no return, yield or income is promised or described.

## Consequences

- **Built in order** (DIRECTION.md L10): the network and seeding one's own published work with no CGT moving; seeding
  others' work; the proofs; the escrow on the devnet; private backup; Demiurge's own relays; the takedown list. M8.1 (the
  Mesh) and L7.4 (Mesh seeding) are carried by L10, as L7.1's local half is by L9.
- **Discovery** of who holds a root uses iroh's peer gossip until the escrow's on-chain assignments exist; peers are
  found through n0's public relays until Demiurge runs its own, which see who connects to whom but never the bytes.
- **Privacy:** escrows on chain show who pays to host how much; a backup's existence and size are visible there, never
  its contents or names.
- **Risk carried:** a seeder claiming several copies on one disk is not fully preventable without proof of replication,
  which MESH_PROOFS.md weighs; release 1 counts distinct copies only by distinct QOR ID accounts and says so.
- **Not decided here:** any price, any rate or pool, the burn share, the pallet's name, and who maintains the takedown
  list.
