# Where Demiurge stands

For Andrew, 18 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this afternoon, from outside)

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, runtime 9: the **Mesh**
  — *launchers sharing files for pay* — is live there.
- **QOR ID** — *the one sign-in* — and **Social** — *the chat service* — on Railway.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`.
- **Launcher 0.1.12 for testers**, a **GitHub Release** — *a public download on the repository*.
  Download page: `claude.ai/artifact/Uw2NRswDw9quvSBxQFbBLv` (private until you share it).

## Newest: each QOR ID is now separate, in launcher 0.1.13 (built, not published)

Your ~59,000 came from your **old second account** — *another address made from your phrase* —
which the launcher re-made every time you asked for a "new" one. As you asked, each QOR ID is now
**a profile** — *its own wallet and data on this PC*:

- **Its own wallet**, one account. A new QOR ID gets a new phrase and starts at 0.
- **Signing out closes it completely**: balance, chats, Library and Mesh all leave the screen.
- **Who's playing?** then lists your QOR IDs by name only. Each opens with **its PIN** (6–12
  digits), chosen the first time you sign it out. The one you used last still opens by itself.
- **Your current wallet becomes the first profile** (Yaldobaoth's, 37,000). To get the 57,900
  account back as its own QOR ID: Settings → *Other accounts from this wallet's recovery phrase*.

Every test passes (302 host tests, 14 screen checks). A PIN keeps people at the keyboard out;
someone who can run programs as your Windows user could guess it slowly, so for real
separation use separate Windows logins. **Once you install 0.1.13, don't go back to 0.1.12**:
it would not find your wallet.

## What only you can do

1. **Say "commit and publish 0.1.13"**, or install it from your Desktop first and try it.
2. **Rule on one gate change:** ADR-098's check was replaced by ADR-099's (the old screens
   are gone). Is that a loosening? Last time, for similar removals, you said no.
3. **Share the download page** with testers, and **send each tester test CGT** from the faucet.
4. **Crediting godmode with 100,000,000 CGT:** a faucet transfer you sign; I need its address.
5. **Before Social opens to anyone else:** the rules reports are judged against (L8.7).
6. **Keep the `godmode` account safe**; move or delete `social-secrets.txt` on your Desktop.
7. **Still open:** `archive-bZLIQp/` (one zip); the old `QOR-MATRIX/arqade` repository; QQ
   Studio's update; the last two L1.4 checks; CI for QQ; the welcome-grant payer; an email;
   replace the key pasted on 5 October.

## Cost per month

GitHub downloads are free. The Mesh, QOR Media and translation: $0, all on each PC (the Mesh
uses n0's public relays for now). Railway is not yet measured. Vercel and Neon: free plans.
