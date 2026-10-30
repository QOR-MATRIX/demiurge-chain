# Where Demiurge stands

For Andrew, 30 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. **QOR OS** — *your operating system,
private repository* — is built on it. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

**qorsync.dev**, the new page since you merged #114; **QOR Mobile 0.1.0** and the **QOR
Launcher 0.1.17**, both downloadable from it; **Devnet** — *the test chain on servers* —
on code version 11 with two **peers** — *other nodes it talks to*; **QOR ID** — *the one
sign-in*; **Social** — *the chat service*, at its Railway address; **ARQADE** — *the
gaming website*.

## This session

1. **You merged #114**, so the homepage that leads with QOR OS and offers both downloads
   is live. An **ADR** is *a numbered decision record*; this one is ADR-127.
2. **QOR Mobile 0.1.0 is published**, as you asked: the **pre-release** — *a test
   version, marked so on GitHub* — `android-v0.1.0`. Its **APK** — *the phone's install
   file* — matches the size and fingerprint the page shows. For about two minutes after
   your merge, before it was published, the page's link to it did not work.
3. **Search engines told** of the new page through **IndexNow** — *a service Bing and
   others read to re-visit a page now*.
4. **Records:** L13.8, the Android release, is ticked. They are on the **branch** —
   *changes not yet in `main`* — `session/after-114`, in its **pull request** — *a
   request to merge it*.

## What only you can do

1. **Merge the records' pull request.**
2. **Back up the test key**, `%USERPROFILE%\.qor-launcher\android\`, off this PC. Lose
   it, and testers must uninstall before the next release.
3. **Try QOR Mobile on your phone**: it has run only on the emulator.
4. **The rest carried from before** is HANDOFF §4.0: the economic numbers, the legal
   work, code signing, an outside security review, and the names still to choose.

**Next in engineering:** keys on Android (L13.3), so QOR Mobile can hold a wallet. The
homepage still lists controller support in ARQADE and QOR Media as work left in QOR OS;
it is now done there, and saying so takes a record that restates ADR-127 decision 3.

## Cost per month

As recorded: the homepage and ARQADE use Vercel's **Hobby plan** — *a non-commercial tier*
(HANDOFF §4.0); Railway's cost is not yet measured (SYSTEMS).
