# Where Demiurge stands

For Andrew, 17 October 2026 (late evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime version 8.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.9**, unsigned, with Social and the Library: its installer is on your Desktop.

## New tonight: the Mesh pays for its work, on branch `session/mesh` (not merged)

- **Built (steps 1 to 4 of 7):** an uploader opens an **escrow** — *CGT set aside for
  seeders* — at their own prices. Seeders take the file on, prove each hour that they still
  hold it, and are paid for that and for what they serve, from **receipts** — *a downloader's
  signed "I got this"*. Files can now be found by fingerprint alone.
- **One question, once:** turning seeding on asks whether the Mesh may sign its own work
  for your account. It can never send your CGT (ADR-097, my engineering choice).
- **Seen working** on a test chain on this PC, paid to the exact Spark. **Not on the devnet
  yet**, and not yet between two PCs.

## What only you can do

1. **Say when to merge `session/mesh`**, then **sign the devnet's upgrade to runtime 9** with
   your sudo key — *the devnet's admin key* — so the Mesh can pay there. I rehearse it on a
   local chain first.
2. **Crediting godmode with 100,000,000 CGT:** the recommended way is a transfer from the
   faucet, which you sign. I still need godmode's chain address.
3. **Drive X: has 99 GB free** (90% full), better than this morning. Builds fail near zero.
4. **Look at the Library** in 0.1.9; **test World, translation, friends and levels**.
5. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
6. **Keep the `godmode` account safe**; move or delete `social-secrets.txt` on your Desktop.
7. **Update QQ Studio** and publish again; the last two L1.4 checks; CI for QQ; the
   welcome-grant payer; an email; replace the key pasted on 5 October.
8. **Housekeeping:** `archive-bZLIQp/` (one zip); the old `QOR-MATRIX/arqade` repository.

## Cost per month

The Mesh, QOR Media and translation: $0; all run on each PC (the Mesh uses n0's public
relays for now, which n0 runs; ours come later). Railway is not yet measured. Vercel and Neon: free plans, per earlier notes.
