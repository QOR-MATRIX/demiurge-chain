# Where Demiurge stands

For Andrew, 8 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today

- **Devnet** — *the test chain on servers* — at `wss://rpc.qorsync.dev`, version 8, on
  Railway — *a server host*. You hold its admin key. 150 tests pass.
- **QOR ID** — *the one sign-in* — at `id.qorsync.dev`: unique names, `/account`, sign-in
  for websites, levels and avatars. 169 tests pass.
- **ARQADE** — *the gaming website* — at `qor-arqade-tau.vercel.app`: games, rankings, chat,
  levels, tips paid through the launcher. 49 tests pass.
- **Launcher 0.1.8**, unsigned — *Windows warns on install*. **CI** — *automatic build and
  test* — is green on main.

## Done this session (8 October, not yet committed)

IBM Bob had left unfinished, untested work. It was checked, and the faults fixed:

- **QQ**, *the QOR Engine*, has its first slice: a QQ entry on the launcher's rail where you
  place glowing shapes and particle emitters, drag them, press Play and Stop, and save the
  scene into a project, where one change is one line in the history. 41 screen checks.
  Not ticked yet: a picture (sprite) still to add, and you have not tried it in the launcher.
- **ARQADE from the launcher** opens in its own guarded window; a tip there goes straight to
  the launcher's approval. Bob's version could never have shown anything.
- **Six backdrops** — *the moving launcher background* — to choose in Settings.
- **Your avatar on ARQADE's sidebar.**
- **Owed welcome grants** can be listed and marked paid by you; nothing pays them yet.

**Not built:** fees, new CGT, a production network, paying grants, the launcher's sign-in
inside the ARQADE window, agent rails — *spending limits for AI agents* — and the Mesh.

## Next work

1. QQ: the sprite and a run in the launcher, then tick P3.1; then 3D.
2. Pay owed welcome grants from a Welcome account; then ARQADE signed in by the launcher.

## What only you can do

1. **Review and merge** this branch's pull request when it is opened; re-run the audit if not done.
2. **Try QQ and ARQADE's window** in a launcher built from it; with the sprite, that is
   what P3.1 still needs.
3. **Install the next launcher build** for your avatar on its glowing ring.
4. **Add an email address** at `id.qorsync.dev/account`, so a lost password can be reset.
5. **Replace the exposed key** pasted into a chat on 5 October; if devnet admin, move
   admin (`sudo.setKey`).
6. **Delete `QOR-MATRIX/arqade`**, the stray copy Vercel made, if not done.
7. **Bounce reports:** a Resend — *QOR ID's email sender* — webhook and its secret in
   Railway as `RESEND_WEBHOOK_SECRET`.
8. **In Cloudflare**, delete the `ci` record and the tunnel. Rotate the nine old credentials
   if those services ran; revoke SSH key `admin@pleroma`.
9. **Decide the nine frozen tiles** on the Nexus — *the launcher's home map*.
10. **Before commercial use:** a paid Vercel plan, a legal review and trademark clearance.
11. **Open questions:** OPEN-1 to OPEN-4, U-4, U-14 to U-18, Q-19 and Q-20.

## Cost per month

Railway not yet measured (estimated $16–37). Vercel and Neon — *ARQADE's database host*:
$0 on free plans. No spending cap (your decision, 1 October).
