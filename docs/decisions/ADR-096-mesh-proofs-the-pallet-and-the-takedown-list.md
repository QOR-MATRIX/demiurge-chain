# ADR-096: The Mesh's proofs accepted, the pallet named `pallet-mesh`, the takedown list kept by root

**Status:** **Accepted**, 17 October 2026, by the owner, who was asked to review the proofs of seeders' work
([`../architecture/MESH_PROOFS.md`](../architecture/MESH_PROOFS.md)), approve the chain module's name, and say who keeps
the takedown list, and answered: "Go with best and most recommended path forward." Each decision below is the option
recommended to the owner that day. **Extends** [ADR-095](ADR-095-the-mesh-on-iroh-paid-from-hosting-escrow.md) (its
decisions 5 and 6, and the takedown list its decision 3 named). **Answers** U-6. It decides **no economic value**: prices
stay the uploader's, and no rate, pool, split or burn share is set (AGENTS.md §5).

## Context

ADR-095 left three things to the owner: the scheme that proves a seeder's work (U-6), proposed in MESH_PROOFS.md; the name
of the chain module that holds escrows and pays seeders, proposed as `pallet-mesh`; and who may add a root to the list
seeders refuse. Checked on 17 October 2026: the chain has no randomness source (Aura, no randomness pallet), its block
time is 6 seconds, and it keeps the last 256 block hashes (`BlockHashCount` from frame-system 48's
`SolochainDefaultConfig`, which `chain/runtime/src/lib.rs` derives), so an answer window must close within 256 blocks
of its seed.

## Decision

1. **MESH_PROOFS.md is accepted as the proof of seeders' work**, sections 1 to 5, with its proposed parameters: a period
   of 600 blocks (an hour), an answer window of the first 100 blocks after the period's seed block, and an assignment
   ending after three periods in a row unanswered. Storage is paid for answered challenges; serving for receipts the
   receiving account signs. Copies are counted by distinct accounts. The parameters are engineering values in the
   runtime's configuration, not economic ones, and may be changed by a later record.
2. **The chain cannot see QOR ID**, so of the receipt rules in MESH_PROOFS.md §3 the chain enforces: the receiver is
   neither the seeder nor the uploader; one receiver is paid for at most the file's length per period; the uploader's
   cap on bytes paid per period. That a receiving account is one a QOR ID signs in with is enforced only once QOR ID
   attests accounts on chain, which is its own later step; until then the uploader's cap bounds what collusion can draw,
   and the Mesh page says so where the price is set.
3. **The chain module is `pallet-mesh`** (`chain/pallets/mesh/`, `pallet_mesh`, runtime name `Mesh`), with plain names for
   its calls, events and storage (ADR-032).
4. **The takedown list is kept by the chain's root origin**: the sudo key on the devnet (the owner's), and governance when
   it exists. A listed root cannot be escrowed, its escrows pay nothing more, and seeders refuse it.
5. **Order of building.** Finding who holds a root (L10.2) reads the escrow's assignments on chain, so the chain's half
   comes first: `pallet-mesh` with the proofs and the escrow (L10.3, L10.4), then the launcher answering challenges,
   signing receipts and taking assignments (L10.2).

## Consequences

- U-6 is answered (OPEN_QUESTIONS.md says so); MESH_PROOFS.md is a decided design.
- **Before any validator other than the owner's runs,** challenges need unbiased randomness (MESH_PROOFS.md §2). Until
  then a validator could bias which piece is asked for; on the devnet the validators are the owner's.
- `pallet-mesh` moves CGT, so it follows AGENTS.md §5: integer Sparks, SDK helpers for every ratio, and a test pinning its
  largest intermediate; and it joins the coverage gate's pallets.
- Its arrival on the devnet is a runtime upgrade the owner signs with the sudo key.
