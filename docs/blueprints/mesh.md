# The Mesh

**What it is:** a peer-to-peer network inside every QOR Launcher that stores and serves what Demiurge holds: published
work, fetched by anyone and checked against the fingerprint on chain, and each person's private backup, encrypted
before it leaves their PC. People who give disk and bandwidth (**seeders**) are paid in CGT for the work they prove, from
the **hosting escrow** of whoever put the work there. **Decided by**
[ADR-095](../decisions/ADR-095-the-mesh-on-iroh-paid-from-hosting-escrow.md) (accepted by the owner, 17 October 2026),
carrying out ADR-005 and ADR-006. **Proofs:** [`../architecture/MESH_PROOFS.md`](../architecture/MESH_PROOFS.md), a
proposal for the owner's review (U-6). **Roadmap:** L10 in [`DIRECTION.md`](../DIRECTION.md). What exists is in
[`SYSTEMS.md`](../SYSTEMS.md). **Built so far** (branch `session/mesh`, not merged): L10.1, the node in the launcher's host
(`tools/qor-launcher/core/src/mesh/`) and the Mesh page.

The owner's brief, 17 October 2026: an in-launcher torrenting network in which any person signed in with a QOR ID is
paid CGT for seeding files and data for others, "a sort of cloud file storage mesh", with being paid for seeding a main
mechanic.

## 1. Shape

```
 QOR Launcher (each PC)                                                 the chain
 ┌──────────────────────────────────────────────┐                     ┌───────────────────────────────┐
 │ Mesh page (React): on or off, disk and       │                     │ DRC-369: each work's root     │
 │ bandwidth limits, what is seeded, what was   │                     │ hosting escrow (pallet-mesh,  │
 │ served, what was paid; backup and restore    │                     │ ADR-096):                     │
 ├──────────────────────────────────────────────┤   QUIC (iroh)       │ deposits, prices, copies,     │
 │ Rust host `mesh`: iroh endpoint (node key),  │◄═══════════════════►│ assignments, challenges,      │
 │ iroh-blobs store (BLAKE3, 16 KiB groups),    │   other launchers   │ receipts, payouts             │
 │ limits, challenges answered, receipts signed │                     └───────────────────────────────┘
 │ through the vault; backup encryption         │──── signed calls ────────────▲
 └──────────────────────────────────────────────┘
```

- **One network, two kinds of content.** Published work is public and verified on arrival against its root on chain.
  Backup is ciphertext, in erasure-coded shards spread across seeders.
- **Two keys.** The launcher's **node key** (iroh, made on that PC) connects and serves. The person's **account**
  (the vault) receives payment and signs receipts. The account signs a **binding** naming its node once.
- **Two ways of being found.** iroh dials a node by its public key through home routers, with relays as fallback (n0's
  public relays until Demiurge runs its own). Who holds a root is read from the chain: the escrow's assignments, then its
  uploader, each dialed by the node it bound (ADR-097). A file no escrow pays for is shared by ticket.
- **One approval for the Mesh's own work.** Proofs, claims, assignments and receipts are signed under the Mesh grant,
  asked once when seeding is turned on; none can send the person's CGT. Opening an escrow keeps its dialog (ADR-097).

## 2. Seeding (the launcher's Mesh page)

| Setting | What it does |
| --- | --- |
| Seeding | Off at install. Turning it on shows what seeding means: disk and upload given to others, your network address seen by the peers you serve, payment only for proven work |
| Disk | The most the Mesh may use, in the launcher's data folder; nothing beyond it is ever written |
| Upload | The most bytes a second served to others; downloads for the person are never slowed for it |
| While | Release 1: while the launcher runs. Later: in the background, on a schedule, only on unmetered networks |

The page shows what is seeded (roots, sizes, whose), what was served this period, challenges answered and missed, and
CGT paid, each from the chain's own records. Copy follows ADR-008: "paid for seeding", never income or returns.

## 3. Published work

1. **Publishing.** Minting or publishing (Studio, Projects, QQ Studio's Publish) puts the bytes in the person's own
   Mesh store under their root, as the temporary content store does today, and seeds them from that PC.
2. **Fetching.** Anyone fetching a root asks the peers that hold it; iroh-blobs streams it verified against the root
   in 16 KiB groups, so a wrong byte is refused at once and fetched from another peer. Nothing unverified is kept.
3. **Hosting.** A publisher who wants their work held while their PC is off opens an escrow: roots, copies wanted, a
   price for storage per period and for bytes served, and a deposit. Seeders take assignments; the escrow pays them
   for proven work and returns what is left when the publisher stops.
4. **Takedown.** Seeders refuse roots on a takedown list, kept on chain. Who may add to it is not decided (ADR-095).

## 4. Private backup

1. **Choosing.** The person picks folders to back up (the Library's folders are offered).
2. **Encrypting on the PC.** Files are packed into **shards** (proposed: 64 MiB), each encrypted with
   XChaCha20-Poly1305 under a key derived from the vault's secret with HKDF-SHA256 and a fixed label, so the recovery
   phrase restores the backup as it restores the vault. A seeder sees only ciphertext and its BLAKE3 root.
3. **Erasure coding.** Shards are grouped and coded so that any *k* of *n* pieces rebuild a group (proposed: 4 of 6),
   using Reed-Solomon. Each piece goes to a different seeder; losing two of six loses nothing.
4. **The index.** Which files are in which shards, and which pieces are where, is itself encrypted and backed up the
   same way; its root is the one thing the person's launcher must know, and it is kept on chain in their escrow.
5. **Repair.** When a seeder misses challenges and its assignment ends, the person's launcher (when it next runs)
   rebuilds the missing piece from others and places it with another seeder.
6. **Restoring.** On a new PC: restore the vault from the recovery phrase, and the launcher finds the index through the
   escrow, fetches and decrypts it, and restores the files.
7. **Paying.** The person deposits into their backup escrow at the storage price they choose; seeders are paid for
   answered challenges. Nothing is paid for serving a backup except to its owner, who pays for their own restore.

## 5. Payment

Everything in ADR-095 decisions 4 and 5 and MESH_PROOFS.md: the uploader's escrow, the uploader's price, proven work
only, no CGT created, no burn until OPEN-4 decides one. A subsidy from the perpetual issuance (ADR-004) may join once
OPEN-1 to OPEN-3 are decided. **No price appears in code or documents as decided**: the launcher asks the person for
theirs, shows the offers others have made once there are any, and tests use values marked as placeholders.

## 6. Order of building (L10)

1. The network and one's own published work seeded, with no CGT moving (replacing the temporary content store).
2. Seeding others' published work: discovery, limits, fetch and verify.
3. The proofs (ADR-096), answered and signed by the launcher under the Mesh grant (ADR-097).
4. The escrow on the devnet, in `pallet-mesh` (ADR-096); the Mesh page shows what was paid.
5. Private backup: encryption, erasure coding, repair, restore.
6. Demiurge's own relays.
7. The takedown list.

## 7. Not in release 1

Seeding while the launcher is closed; mobile; proof of replication; any issuance subsidy; streaming media from the
Mesh into QOR Media before it is fully fetched; arbitrary files shared in the open.
