# Where Demiurge stands

For Andrew, 27 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

**qorsync.dev**, offering **Launcher 0.1.17**; **Devnet** — *the test chain on servers* —
on code version 11, two **peers** — *other nodes it talks to* — and **finalising** —
*making blocks permanent*; **QOR ID** — *the one sign-in*; **Social** — *the chat
service*; **ARQADE** — *the gaming website*.

## This session

1. **You merged #107**; **CI** — *GitHub's automatic builds and tests* — passed on `main`.
   The launcher's **core** — *the code that holds keys* — now does all signing and service
   calls, so **QOR OS** — *your own operating system* — can build it from `main`.
2. **ADR-121 is accepted.** This **ADR** — *a numbered decision record* — ignores one
   **advisory** — *a published library flaw* — in code nothing builds, as CI proves every
   run. Your "proceed with" came after the merge, so it is logged as a retroactive approval.
3. **ADR-122 is proposed, not decided:** item 2 below.
4. **Found, not fixed:** the **gates dashboard** — *the launcher's release checks page* —
   reads a bold "**Accepted**" as not accepted, so ten product gates under-count their
   decisions (not Alpha or Beta). Changing the dashboard is a **loosening** — *weakening a
   release check* — so yours to decide; un-bolding those 21 records changes no rule.

Items 2 to 4 are on the **branch** — *changes not yet in `main`* — `session/after-107`.

## The release gates (as HANDOFF measured them, 22 October)

**Alpha** — *the first test release* — lacks M3.2, which needs the **genesis split** — *who
holds CGT at launch* — (OPEN-2), and two launcher dialogs you have not tried (L1.4).
**Beta** — *the worldwide test* — has 8 of 45 checks met; most others wait on the list
below.

## What only you can do

1. **Merge `session/after-107`** once its **pull request** — *a request to merge it* — is
   open. Merging it changes no licence.
2. **Decide ADR-122.** Six active **crates** — *packages of Rust code* — say **MIT** — *a
   short permissive licence* — while the repository says **Apache-2.0** — *permissive, with
   a patent grant*. I recommend Apache-2.0 for all six, old releases keeping MIT. Accept,
   keep MIT, or offer both.
3. **Who owns the copyright?** The tree names "Demiurge", "Laustrup Group", "Astra Matrix,
   Inc & Laustrup Group" and, in QOR ID, a person. Settle which entity, with a lawyer.
4. **Play an arena** — *a two-player ARQADE game* — with a friend or a second QOR ID
   (HANDOFF §4 item 157).
5. **Back up the update key** — *what signs launcher updates* — now: from any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   and keep the key file and its password offline, together.
6. **For rewards in ARQADE:** decide **ADR-116** — *confirming runs before paying* — and
   the reward pool's numbers (largest payout, budget per period, most per player).
7. **Let a test agent** — *a program acting for you* — **ask you:** in
   `X:\REPOZ\qq-interface\tools\qor-launcher\src-tauri` run `cargo run --example agent_link`
   and press *Allow agent* (ticks L5.1).
8. **With a lawyer:** Terms of Service, a privacy policy, and the opinion on rewards, prizes
   and **KYC** — *identity checks* — before anything carries value.
9. **The economic numbers** — *issuance, genesis split, decay, fees* — (OPEN-1 to OPEN-4,
   U-4, U-11). Most of the Beta gate waits on them.
10. **Code-signing certificates**, an **external security review**, and **names** for the
    **indexer** — *the chain's search service* —, the **SDK** — *the developers' toolkit* —
    and the **MCP server** — *how AI tools reach Demiurge*.
11. **Smaller items still open:** HANDOFF §4.0, "The owner's carried items".

**Next in engineering:** at your word, renting in the launcher, or keys on Android (L13.3).

## Cost per month

As recorded: the homepage and ARQADE use Vercel's **Hobby plan** — *a non-commercial tier*
(HANDOFF §4.0); Railway's cost is not yet measured (SYSTEMS); no other cost is recorded.
