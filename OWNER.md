# Where Demiurge stands

For Andrew, 29 October 2026 (evening). One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. **QOR OS** — *your operating system,
private repository* — is built on it. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

**qorsync.dev**, leading with QOR OS and offering **Launcher 0.1.17**; **Devnet** — *the
test chain on servers* — on code version 11 with two **peers** — *other nodes it talks
to*; **QOR ID** — *the one sign-in*; **Social** — *the chat service*; **ARQADE** — *the
gaming website*.

## This session

1. **You merged #112**, so ADR-125, Entire not used, is on `main`. An **ADR** is *a
   numbered decision record*. **CI** — *the automatic checks* — passed on the merge.
2. **QQ, QOR Media and ARQADE's app and games now move to QOR OS** (ADR-126), as you
   chose. Nothing in them had changed here since QOR OS took them on 26 October. Their
   copies here are **frozen** — *kept, but not worked on* — and the launcher keeps
   shipping them. Only a fix you release changes them.
3. **ARQADE's rules stay shared.** Each game's rules are one small file that ARQADE's
   server here checks runs by, so those five files stay the same in both places, and
   change in both at once.
4. **The gates are unchanged.** A **gate** is *a release's checklist*. QQ's gate stays
   unmet here, because three of its items now move to QOR OS's plan.

ADR-126 and these records are on the **branch** — *changes not yet in `main`* —
`session/after-112`, in its **pull request** — *a request to merge it*.

## What only you can do

1. **Merge the records' pull request.** It changes no code.
2. **Who owns the copyright?** The tree still names several holders. Settle it with a
   lawyer.
3. **The rest carried from before** is HANDOFF §4.0, "The owner's carried items": the
   economic numbers, the legal work, code signing, an outside security review, and the
   names still to choose.

**Next in engineering:** at your word, renting in the launcher, keys on Android (L13.3), or
moving `chain/`'s library past the advisory.

## Cost per month

As recorded: the homepage and ARQADE use Vercel's **Hobby plan** — *a non-commercial tier*
(HANDOFF §4.0); Railway's cost is not yet measured (SYSTEMS).
