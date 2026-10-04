# Rules for AI assistants

These rules apply to every AI assistant working in this repository: Claude, Cursor, Copilot, Codex
and any other.

## 1. Old documentation does not exist

The pre-realignment documentation was deleted on 2026-09-13 because it contradicted the current
direction. **Never restore it from git history, and never read, quote, summarise or rely on it, for
any reason.** That includes background, examples, numbers and "what the old plan said". If you need
a fact you cannot find in a current document, verify it in the code and write it into a current
document.

## 2. Read these, in this order

1. [`HANDOFF.md`](HANDOFF.md): where work stands
2. [`docs/DIRECTION.md`](docs/DIRECTION.md): the only roadmap and the scope
3. [`docs/DECISIONS.md`](docs/DECISIONS.md): the only decision log, and the index of the
   architecture decision records in [`docs/decisions/`](docs/decisions/README.md)
4. [`docs/economics/CGT.md`](docs/economics/CGT.md): the economic model, and
   [`docs/economics/OPEN_QUESTIONS.md`](docs/economics/OPEN_QUESTIONS.md): what it leaves undecided
5. [`docs/protocol/PROTOCOL.md`](docs/protocol/PROTOCOL.md): exactly what the chain does today, and
   [`chain/README.md`](chain/README.md): how to build and run it. **There is one chain, and it is
   `chain/`** (§4, §7)
6. [`.cursorrules`](.cursorrules): project law

The full list of current documents is [`docs/README.md`](docs/README.md). A document not in that list
is not current.

## 3. The code is the truth

- Verify claims against the code before repeating them.
- When you change behaviour, update the affected current document in the same change.
- Never add a second roadmap, status table, decision list or economic model anywhere. Update the
  existing one.
- A new decision is a new ADR in `docs/decisions/`, indexed in `docs/DECISIONS.md`. An ADR is never
  edited to change its decision; a later ADR supersedes it, and both say so.
- Pages inside frozen apps (for example `apps/hub/src/app/docs/**`) describe the old protocol and
  are not a reference.

## 4. Scope

Active: `chain/`, `services/qor-auth/`, `tools/qor-launcher/` and `products/arqade/` (ADR-069, since 4 October 2026). Everything under `apps/`, `cli/`,
`sdk/`, `packages/` and `client/` is frozen (D-011, amended by ADR-011): do not extend it or treat it as
working unless the owner brings it back into scope. `aeons/`, `archons/` and `syzygies/` are dead.

`chain/` is the Substrate L1 (ADR-013, located and named by ADR-032), started on 17 September 2026 once
the owner confirmed ADR-018 to ADR-032. **It is the only chain in this repository.**

**`framework/` held the custom Rust devnet `chain/` replaced. It was retired at M3.5 on 20 September 2026
and deleted.** Do not restore it, and do not write new code against it or its RPC vocabulary. What it established is not lost:
its behaviour is in `chain/`'s acceptance tests, its module-by-module mapping is in
[`docs/architecture/MIGRATION_INVENTORY.md`](docs/architecture/MIGRATION_INVENTORY.md), and what it did on
the wire is in [`docs/protocol/PROTOCOL.md`](docs/protocol/PROTOCOL.md)'s history. Unlike the
pre-realignment documentation (§1), it is code rather than a contradicting plan, so reading it in git
history is not forbidden — but it is an untrusted chain that accepted forged signatures, and nothing in it
should be copied forward.

## 5. Money

- CGT has 18 decimals. All amounts are integer Sparks in `u128`. Never use floating point for money.
- Reject excess precision; never truncate an amount.
- **Scaled arithmetic uses the SDK's helpers, never a hand-written `a * b / c`** (ADR-035). A fraction of an
  amount uses `Perbill`, `Permill` or another `PerThing`; a ratio uses `multiply_by_rational_with_rounding`,
  or `FixedU128`, which uses it. At eighteen decimals the full supply is `10^32` atomic units, so two
  amount-sized values multiplied together overflow `u128` long before the result would. Every pallet that
  moves CGT carries a test pinning the largest intermediate its money paths form.
- The base supply is 100,000,000,000,000 CGT, released over a decay curve, with a small perpetual
  issuance for infrastructure and fee burn as its counterweight (ADR-003, ADR-004). The fixed
  13,000,000,000 supply and D-001, D-002 and D-003 are superseded.
- Until the issuance mechanism is designed and built, do not add any path that creates CGT outside
  `--dev`.
- **Never invent a value that `docs/economics/OPEN_QUESTIONS.md` lists as undecided**: no issuance
  rate, genesis split, decay curve, burn share or fee class in code, genesis files or documents. A test
  that needs such a number uses one that is clearly marked as a placeholder, never one presented as
  decided.
- Every mechanic that touches CGT passes the spend test (ADR-002) and names the demand sink it belongs
  to (ADR-006).

## 6. Language

Demiurge is infrastructure for creators to earn from use, not a vehicle for appreciation (ADR-008).
APY, yield, return in the financial sense, ROI, investment, passive income and any statement about
CGT's price or future value do not appear in copy, comments, identifiers or documentation. Validators
are paid for securing the chain, seeders for storing and serving, creators when their work is used,
licensed or remixed.

## 7. The base layer

Demiurge **is** a purpose-built Substrate L1 on the Polkadot SDK (ADR-013); the move completed at M3.5. Consensus, finality,
networking, storage, transaction validity and runtime upgrades use standard Substrate components.
Custom pallets are for the creative layer: DRC-369, royalties, agent rails, the economics and the Mesh.
A departure from a standard pallet needs a written reason.

- **There is one chain, and it is `chain/`.** The custom Rust devnet in `framework/` was retired at M3.5
  (§4), and CI fails if the directory returns: CI is GitHub Actions on the public repository since ADR-063. Nothing in this repository is described as a chain except
  `chain/`.
- **No migration code is written until the owner has reviewed
  [`docs/architecture/MIGRATION_INVENTORY.md`](docs/architecture/MIGRATION_INVENTORY.md).**
- Pallet names in the inventory are placeholders. Naming a new pallet needs the owner's approval (§8).

## 8. Working agreements

- Keep `cargo test --workspace` in `chain/` green, and run it **without** `SKIP_WASM_BUILD`. That variable
  is for iterating: it cannot build a chain specification, so a suite run under it is not evidence. Run it
  before calling chain work done.
- Report failures honestly, with the output.
- Gnostic naming (Aeons, Archons, Syzygies) is used for internal logic that no outside party reads. Anything in
  runtime metadata or published to other developers (crates, pallets, calls, events, storage, SDK and RPC
  surfaces) uses plain, greppable names under the Polkadot SDK convention (ADR-032). Ask the owner before naming
  a new module.
- Do not commit, push or deploy unless the owner asks.
- **[`OWNER.md`](OWNER.md) and [`docs/SYSTEMS.md`](docs/SYSTEMS.md) are rewritten at the end of every
  session. A session that leaves either stale is not finished.** They are the two documents the owner
  reads, and they are the first to rot, because everything else is written for whoever is doing the
  work. `OWNER.md` stays under one printed page and glosses every term of art in five words on first
  use; `SYSTEMS.md` says what every system is and whether it exists. Neither may contain a claim that
  was not checked against the tree in the same session.

## 9. Nothing secret reaches a log: a review requirement

Token material has reached QOR ID's logs three times, each from a new direction. These rules are
checked in review. A change that breaks one is incomplete, and is not approved, however it tests.

- **A new flow joins the log check in the same change.** A change that introduces a flow handling a
  token, key, secret, password, backup code, link or email address adds that flow to
  `services/qor-auth/src/log_hygiene.rs` (`nothing_secret_reaches_a_log_at_any_level`) in the same
  change. The values the flow handles go in the list the check looks for. A flow can be a route, a
  page, a background task, an email, a webhook or a log call.
- **Whatever the value is called.** CI's static scan matches variable names (`token`, `email`, …). A
  value under any other name (`code`, `sig`, `v`, a struct field) passes that scan unseen, so passing
  it is not evidence. Only the runtime check covers such a value, and it must.
- **SQL values are bound, never written into a statement.** sqlx logs full statement text. The build
  fails on a query built from values (`services/qor-auth/src/sql_hygiene.rs`).
- Both checks are release criteria in `docs/GATES.toml`: `alpha.no-secrets-in-logs` and
  `alpha.parameterised-sql`. Loosening either is the owner's decision, under that file's rule.
