# Where Demiurge stands

For Andrew, 18 October 2026 (afternoon). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this afternoon, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime 9: the **Mesh**
  — *launchers sharing files for pay* — is live there, so paid seeding can be tested.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.11 for testers**, a **GitHub Release** — *a public download on the repository*
  — with QOR Media inside. Download page and checklist: `claude.ai/artifact/Uw2NRswDw9quvSBxQFbBLv`
  (private until you share it).

## Newest: one wallet under two QOR IDs, fixed in launcher 0.1.12

You saw one balance shown under two QOR IDs on one PC. The cause: the launcher always opened on
the wallet's **first account** — *one address inside your wallet* — whichever QOR ID was signed in,
and paid ARQADE tips from it. In 0.1.12 (**its installer is on your Desktop**; the download page already offers it):

- **Each account keeps its own QOR ID**, and the launcher opens on the one you used last.
- **The Vault labels every account** with its QOR ID, and warns, with a one-click fix, when the
  account on screen isn't the signed-in QOR ID's. The menu's account card warns too.
- **Settings, signed out, lists each account** to sign in as, and *A new QOR ID* makes one on a new,
  empty account, so two QOR IDs never share CGT.
- **Tips pay from the signed-in QOR ID's own account.**

Every launcher test passes (296, plus 14 screen checks). Not a lock between people: whoever uses
the launcher can reach every account. Two people who must keep CGT apart need separate Windows
logins.

## Earlier today

- **Launcher 0.1.11 published for testers:** right-click a Library file → *Publish to the Mesh*,
  and the new look. Testers need **2,500 test CGT** each.
- **The devnet** runs runtime 9 with the Mesh, an upgrade you signed last night.

## What only you can do

1. **Publish 0.1.12 with the one command I gave you** (I'm not allowed to create releases). Until
   then the download page's button answers "not found". Then merge **pull request #43**.
2. **Share the download page** with your testers (its Share menu), and **send each tester test
   CGT** from the faucet when they send you their address.
3. **Crediting godmode with 100,000,000 CGT:** a faucet transfer you sign; I need its address.
4. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
5. **Keep the `godmode` account safe**; move or delete `social-secrets.txt` on your Desktop.
6. **Housekeeping:** `archive-bZLIQp/` (one zip); the old `QOR-MATRIX/arqade` repository.
7. **Still open from before:** update QQ Studio and publish again; the last two L1.4 checks; CI for
   QQ; the welcome-grant payer; an email; replace the key pasted on 5 October.

## Cost per month

The release download is free on GitHub. The Mesh, QOR Media and translation: $0; all run on each PC (the Mesh uses n0's public
relays for now, which n0 runs; ours come later). Railway is not yet measured. Vercel and Neon: free plans, per earlier notes.
