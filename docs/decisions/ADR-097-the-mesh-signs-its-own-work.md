# ADR-097: The Mesh signs its own work under one standing approval, and trusts the node for none of it

**Status:** **Accepted**, 17 October 2026, under the owner's standing delegation for engineering choices, in the session
that followed their "Go with best and most recommended path forward" (ADR-096). **Extends**
[ADR-095](ADR-095-the-mesh-on-iroh-paid-from-hosting-escrow.md) and [ADR-096](ADR-096-mesh-proofs-the-pallet-and-the-takedown-list.md),
and adds a second standing approval beside the arrival grant of [ADR-016](ADR-016-sign-in-with-unlock.md). It decides **no
economic value**: no price, rate, pool, split or burn share.

## Context

The vault signs nothing without an approval: every signature is a host-drawn dialog the webview cannot reach, except the
arrival grant for QOR ID's challenge (ADR-016, ADR-056). The Mesh's work cannot live with that. A seeder answers a storage
challenge every period for every file it holds (ADR-096: hourly), within a window of minutes, while nobody may be at the
PC; a person downloading signs a receipt for every seeder they received from. A dialog for each would make seeding
impossible and downloading tiresome.

None of that work can send the person's CGT anywhere. `bind_node`, `take_assignment`, `leave_assignment`,
`answer_challenge` and `submit_receipts` move CGT only from an escrow to the caller, and a receipt lets an uploader's
escrow pay the seeder who served the person. The chain charges no fees today (OPEN-4), so a transaction costs its sender
nothing.

What is dangerous is the node. The launcher builds every transaction from the metadata the node it is connected to serves
(ADR-040), and a node that lied about its metadata could have "the Mesh's answer" encoded as a transfer. With a dialog, the
person approves the intent but never sees the bytes; without one, nobody looks at all. The same holds for a receipt's
bytes if the launcher asked the node for them.

## Decision

1. **One standing approval: the Mesh grant.** When the person turns seeding on, the launcher asks once, in a host-drawn
   dialog, whether the Mesh may sign its own work for the active account: proofs of what this PC holds, claims for payment
   for what it served, taking and leaving the files it holds, and receipts for what this PC downloads. Approved, the grant
   is recorded in the Mesh's settings for that account and lasts until seeding is turned off or the person withdraws it.
   Declined, seeding still runs as L10.1 built it: serving and fetching, with nothing signed and nothing paid.
2. **What the grant signs, and nothing else.** Mesh calls `bind_node` (0), `take_assignment` (4), `leave_assignment` (5),
   `answer_challenge` (7) and `submit_receipts` (8), and receipts for bytes this launcher received and verified. Opening,
   topping up and closing an escrow move the person's CGT, so each keeps its dialog. No command the interface can call
   signs with the grant; only the host's Mesh module does, from values it built.
3. **Nothing the grant signs comes from the node.** The launcher encodes each granted call itself: the Mesh's pallet index
   in the runtime (12) and the call's index, compiled in, then its arguments in SCALE. The node's metadata must encode the
   call it names back to exactly those bytes, or nothing is signed. A receipt's bytes are built in the launcher from the
   chain's genesis hash, as the pallet builds them (`receipt_message`), never asked of the node. A runtime upgrade that
   moves the Mesh's pallet or a call makes the grant refuse until a launcher that knows the new indices ships: it fails
   closed.
4. **When fees exist, the grant needs a ceiling first.** Once the chain charges fees (OPEN-4), every granted transaction
   would spend the person's CGT without asking. Before that runtime ships, the grant gains a most-per-period the person
   sets, and this record is superseded to say so.
5. **Receipts travel over the Mesh.** After a fetch verifies, the receiving launcher signs one receipt per seeder for the
   bytes received from it in the period so far (cumulative, as the pallet pays) and sends it to that seeder over the Mesh's
   own protocol (`demiurge/mesh-receipt/1`). The seeder checks the signature and that it names itself, keeps it, and
   submits what it holds in batches. A receipt for a period the chain no longer accepts is dropped.
6. **Finding who holds a root (L10.2)** reads the chain: the escrows for that root, their assigned seeders and the
   uploader, and each one's bound node. A node is dialed by its key, through iroh's address lookup.
7. **Taking others' work.** While seeding with the grant, the launcher takes copies an escrow still asks for, of files that
   fit the person's disk limit, never its own and never a root on the takedown list; the file is fetched and verified
   before the assignment is taken, so a seeder never holds an assignment for bytes it does not have.
8. **A development chain's period is a minute.** The pallet reads its period and answer window from storage that defaults
   to ADR-096's values; only the `--dev` chain's genesis sets 10 blocks and 5, so a launcher's test reaches a challenge in
   minutes. The local testnet and the devnet keep 600 and 100, and the devnet gains the pallet by an upgrade with no
   genesis for it. Nothing changes either value after genesis. This does not change ADR-096.

## Consequences

- The vault gains one standing approval and its rule changes from "nothing signs without an approval" to "nothing signs
  without an approval given for it": a dialog each time, the arrival grant, or the Mesh grant.
- The grant protects against the webview and against a lying node, not against code already running on the PC as the
  person, which can read the key from the OS keychain anyway (ADR-056's trade).
- A launcher must ship when a runtime upgrade moves the Mesh's pallet index or its granted calls.
- The grant's flows handle no secret but the key, which never leaves the vault: receipts and signatures are published on
  chain when claimed. The live test runs the whole flow with every level of the log on and checks no recovery phrase
  reaches it.
