# Where Demiurge stands

For Andrew, 17 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this afternoon, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime version 8.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.9**, unsigned, with Social and the Library: its installer is on your Desktop.

## New tonight: the Mesh, on branch `session/mesh` (not merged)

- **Decided as you chose (ADR-095):** a network inside every launcher, built on **iroh**
  — *a peer-to-peer library* — holding published work and, later, your **private backup**,
  encrypted on your PC. Seeding is **off until you turn it on**, with your own disk and
  upload limits. Seeders are paid in CGT from the **hosting escrow** — *CGT the uploader sets
  aside* — of whoever put the work there, only for work they prove. No CGT is created.
- **Built (step 1 of 7):** the Mesh page and the network itself. With seeding on, your
  published work is served; files are fetched by **ticket** — *a link naming file and PC* —
  and every byte is checked against its fingerprint. Proven between two launchers on one
  PC; not yet between two PCs. **No CGT moves yet.**

## What only you can do

1. **Review `docs/architecture/MESH_PROOFS.md`** — *how seeders prove their work* — before
   anyone is paid; and **approve a name** for the chain module that pays them (I propose
   `pallet-mesh`); and **say who may add to the takedown list**.
2. **Crediting godmode with 100,000,000 CGT** still needs your choice of the three options
   I gave, and godmode's chain address.
3. **Drive X: is nearly full** (it hit 0 bytes today while Steam was downloading). Free some
   room, or builds fail.
4. **Look at the Library** in 0.1.9; **test World, translation, friends and levels**.
5. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
6. **Keep the `godmode` account safe**; move or delete `social-secrets.txt` on your Desktop.
7. **Update QQ Studio** and publish again; the last two L1.4 checks; CI for QQ; the
   welcome-grant payer; an email; replace the key pasted on 5 October.
8. **Housekeeping:** `archive-bZLIQp/`; the old `QOR-MATRIX/arqade` repository.

## Cost per month

The Mesh, QOR Media and translation: $0; all run on each PC (the Mesh uses n0's public
relays for now, which n0 runs; ours come later). Railway is not yet measured. Vercel and Neon: free plans, per earlier notes.
