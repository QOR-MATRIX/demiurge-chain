# Where Demiurge stands

For Andrew, 17 October 2026 (night). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked tonight, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime version 8.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.10 for testers**, published as a **GitHub Release** — *a public download on
  the repository* — with QOR Media inside. Its download page with install steps and the
  test checklist: `claude.ai/artifact/Uw2NRswDw9quvSBxQFbBLv` (private until you share it).

## New tonight

- **The Mesh is merged** (pull request #40, every CI check green): seeders paid from an
  **escrow** — *CGT an uploader sets aside* — for proving they hold a file and for serving it.
- **The upgrade the devnet needs is rehearsed** against the devnet's own runtime: it keeps
  your sudo key, everything on the chain, and brings the Mesh with its one-hour period.

## What only you can do

1. **Upgrade the devnet to runtime 9**, as you did for 7 and 8: polkadot.js Apps, Developer →
   Sudo, `system.setCode` with the weight override, file `demiurge-runtime-v9.compact.compressed.wasm`
   from your Downloads (SHA-256 begins `318643587f6f`). Tell me when it is done; I read it back.
2. **Share the download page** with your testers (its Share menu), and **send each tester test
   CGT** from the faucet when they send you their address.
3. **Install 0.1.10 yourself first**: nobody has installed it yet. Windows warns because it
   is unsigned; choose More info, then Run anyway.
4. **Merge pull request #41** — *the notes for this release* — once its checks pass.
5. **Crediting godmode with 100,000,000 CGT:** a faucet transfer you sign; I need its address.
6. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
7. **Keep the `godmode` account safe**; move or delete `social-secrets.txt` on your Desktop.
8. **Housekeeping:** `archive-bZLIQp/` (one zip); the old `QOR-MATRIX/arqade` repository.
9. **Still open from before:** update QQ Studio and publish again; the last two L1.4 checks; CI for
   QQ; the welcome-grant payer; an email; replace the key pasted on 5 October.

## Cost per month

The release download is free on GitHub. The Mesh, QOR Media and translation: $0; all run on each PC (the Mesh uses n0's public
relays for now, which n0 runs; ours come later). Railway is not yet measured. Vercel and Neon: free plans, per earlier notes.
