# Where Demiurge stands

For Andrew, 30 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. **QOR OS** — *your operating system,
private repository* — is built on it. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

**qorsync.dev**, still the 29 October page until this branch merges; **Devnet** — *the
test chain on servers* — on code version 11 with two **peers** — *other nodes it talks
to*; **QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the
gaming website*.

## This session

1. **You merged #113**, so ADR-126 is on `main`. An **ADR** is *a numbered decision
   record*. **CI** — *the automatic checks* — passed on it.
2. **The homepage, redone** (ADR-127), as you asked: everything current about QOR OS, a
   Download section, a menu and large tap targets on phones, and a new background: QOR
   OS's mark as a lens that bends a nebula of light and answers your finger.
3. **QOR Mobile 0.1.0**, your name for the launcher on Android, now on the phone too. Its
   **APK** — *the phone's install file* — is built, signed with a new **test key** —
   *the project's signature for testers* — and ran on the Android emulator. It shows the
   first screen only: no wallet yet. It has not run on a real phone.

The site's changes are on the **branch** — *changes not yet in `main`* —
`session/site-qor-os-current`, in its **pull request** — *a request to merge it*.

## What only you can do

1. **Publish QOR Mobile's release first**, then merge the pull request: the page links
   the APK, and my attempt to publish it was refused by my permission system. The pull
   request gives the one command.
2. **Back up the test key**, `%USERPROFILE%\.qor-launcher\android\`, off this PC. Lose
   it, and testers must uninstall before the next release.
3. **Try QOR Mobile on your phone**, and tell me whether it starts.
4. **The rest carried from before** is HANDOFF §4.0: the economic numbers, the legal
   work, code signing, an outside security review, and the names still to choose.

**Next in engineering:** keys on Android (L13.3), so QOR Mobile can hold a wallet.

## Cost per month

As recorded: the homepage and ARQADE use Vercel's **Hobby plan** — *a non-commercial tier*
(HANDOFF §4.0); Railway's cost is not yet measured (SYSTEMS).
