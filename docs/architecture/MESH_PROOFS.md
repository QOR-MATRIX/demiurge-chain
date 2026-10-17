# Proving seeders' work: a design proposal (U-6)

**Status:** A proposal for the owner's review, written 17 October 2026. **Nothing here is decided, and no seeder is paid
until the owner accepts it** ([ADR-095](../decisions/ADR-095-the-mesh-on-iroh-paid-from-hosting-escrow.md) decision 5).
It answers U-6 in [`../economics/OPEN_QUESTIONS.md`](../economics/OPEN_QUESTIONS.md): what a seeder proves, to whom, and
how often. It sets **no number that is an economic value**: prices are the uploader's (ADR-095 decision 4); the
intervals and sizes below are engineering parameters, each marked **proposed**, and the owner may change any of them.

**Decided already:**
- **ADR-005:** seeders are paid for storage and bandwidth the network actually consumes, verified against fingerprints
  on chain; no artificial difficulty.
- **ADR-047:** every file is named by its BLAKE3-256 root; any 16 KiB group of it can be checked against that root with a
  short proof (the BLAKE3 tree's sibling hashes, about 1 KiB for a 1 GiB file).
- **ADR-095:** payment comes from the uploader's hosting escrow at the uploader's price; nothing creates CGT.

## 1. Two kinds of work, two kinds of proof

| Work | Paid for | Proof | Who checks |
| --- | --- | --- | --- |
| **Storage** | Holding a root's bytes over a period | Answers to random challenges: a 16 KiB group chosen by the chain, returned with its BLAKE3 proof | The chain, on the answer itself |
| **Serving** | Bytes delivered to a person | A receipt the receiving person's launcher signs, naming the root, the byte range, the seeder and the period | The chain, on the signature and the rules in §3 |

Storage is what a backup pays for; serving is what published work mostly pays for. An uploader's escrow says what it
pays for each (either may be zero).

## 2. Storage challenges

1. **Assignment.** A seeder takes on a root from an escrow (an on-chain call naming the root and the seeder's account).
   The escrow records which seeders hold it, up to the number of copies the uploader asked for.
2. **The challenge.** Each **period** (proposed: 600 blocks, one hour at 6 seconds a block), the chain derives, for each
   assignment, which 16 KiB group must be shown. The seed is the hash of a block **after** the period opens, so a
   seeder cannot know it in advance and keep only the groups it will be asked for.
3. **The answer.** Within a **window** (proposed: 100 blocks, ten minutes), the seeder submits the group's 16 KiB and
   its BLAKE3 proof. The chain checks the proof against the root it already holds. Checking costs a few dozen hashes.
4. **Payment.** A seeder that answered a period is owed that period's storage price for the root, from the escrow. One
   that missed it is owed nothing for that period; after **three** missed periods in a row (proposed) the assignment
   ends and another seeder may take it.
5. **Cost.** 16 KiB plus about 1 KiB of proof per root per hour, on chain. That bounds how many roots one account can
   hold economically, and is why a backup is stored as a few large shards, not many small files.

**Why not more.** Filecoin-style proofs of replication (sealing each copy so it costs real disk) prevent one disk from
posing as several copies. They are slow, complex and need heavy cryptography. Release 1 does without them and says so:
distinct copies are counted only by distinct QOR ID accounts (§4), and the uploader's choice of copies is honoured on
that basis.

**Randomness.** The chain uses Aura and has no randomness source (checked 17 October 2026: no randomness pallet in
`chain/runtime/src/lib.rs`). A block hash can be influenced by the validator who produces it, by choosing which
transactions to include. On the devnet the validators are the owner's, so this is acceptable there. **Before anyone
else runs a validator**, challenges need unbiased randomness: BABE's VRF, or a public beacon such as drand. That is a
question for mainnet's consensus, not for the Mesh, and is recorded here so it is not forgotten.

## 3. Serving receipts

1. **The receipt.** When a person's launcher has received and verified a range of a root from a seeder, it signs a
   receipt with the account's key (through the vault, never a raw key): the root, the range, the seeder's account, the
   period. One receipt covers a session's bytes, so a download is one signature, not one per piece.
2. **Submitting.** The seeder collects receipts and submits them in batches; the chain checks each signature and pays
   the escrow's serving price for the bytes, once per receipt.
3. **What counts.** A receipt counts only if:
   - the receiving account is **not** the seeder's, nor the uploader's (no paying oneself);
   - the receiving account is one a QOR ID signs in with (QOR ID's key sign-in), so making accounts costs a sign-up,
     which QOR ID already limits to three per network address (ADR-078 decision 9);
   - the bytes are within the root's length, and the same receiving account's receipts for the same root and period
     do not exceed the root's length (proposed) — downloading a file once pays for one download;
   - for published work an uploader may restrict receipts to accounts entitled to it (buyers of a DRC-369 asset, for
     example), so a pair of colluding accounts cannot drain an escrow with downloads nobody wanted.
4. **What it does not stop.** Two real people with two QOR IDs downloading from each other to draw from a third party's
   escrow. The uploader's caps (a most-per-period for serving, and the entitlement filter) bound the loss; it is not
   zero, and the uploader is told so where the price is set.

## 4. Identities

- **Account:** the person's chain account (the vault's key), which receives payment and signs receipts.
- **Node:** the launcher's iroh key, made on that PC, which connects and serves. A **binding** (the account signs "this
  node is mine") is registered on chain once; the chain accepts answers and receipts only from bound nodes.
- **QOR ID:** one account per QOR ID for counting distinct copies and receipts (§2, §3). A person with two QOR IDs is
  two seeders; QOR ID's sign-up limits are what make that costly.

## 5. What the chain stores

Per escrow: the depositor, the roots, the prices, the copies asked for, the balance, the assignments. Per period:
which assignments answered. Per receipt: a hash, to refuse it twice. Nothing about a backup's contents or names.

## 6. What the owner is asked to decide

1. Accept storage challenges and serving receipts as the proof of work (§1).
2. Accept the proposed period, window and missed-period limit, or set others (§2).
3. Accept that release 1 counts copies by distinct QOR ID accounts, without proof of replication (§2, §4).
4. Accept the receipt rules, including the uploader's entitlement filter and caps (§3).
5. Note that unbiased randomness is required before validators other than the owner's run (§2).
