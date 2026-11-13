# Contributing

1. **Read the current direction first**: [`HANDOFF.md`](HANDOFF.md), [`docs/DIRECTION.md`](docs/DIRECTION.md),
   [`docs/DECISIONS.md`](docs/DECISIONS.md) and [`docs/economics/CGT.md`](docs/economics/CGT.md). Work
   that is not on the roadmap, or that contradicts a decision, needs a new ADR in
   [`docs/decisions/`](docs/decisions/README.md), indexed in `DECISIONS.md`, first.
2. **Never restore or consult deleted documentation**, including from git history (D-012).
3. **Work on a branch.** Do not commit directly to `main`.
4. **Keep the tests green.**
   - Chain: `cd chain && cargo test --workspace`. **Not** under `SKIP_WASM_BUILD`: that leaves the
     runtime wasm unbuilt, so a chain specification cannot be built and a run under it is not evidence.
     `chain/` is the only chain; `framework/` was deleted at M3.5 and CI fails if it returns (`.github/workflows/ci.yml`)
   - Launcher host: `cd tools/qor-launcher/src-tauri && cargo test`
   - Identity service: see [`scripts/run-local-stack.md`](scripts/run-local-stack.md) (`DATABASE_URL` is needed at build time)
   - ARQADE: `cd products/arqade && npm test`
5. **Update the documentation in the same change.** If behaviour changes, the current document that
   describes it changes too. There is one roadmap, one decision log (with its ADRs) and one economic
   model; do not create others.
6. **Money is integers.** CGT amounts are integer embers in `u128`, with 18 decimals; a Spark is `10^15` embers. No floating point. Never
   invent a value that [`OPEN_QUESTIONS.md`](docs/economics/OPEN_QUESTIONS.md) lists as undecided.
7. **Language.** No APY, yield, return, ROI, investment, passive-income or price language in copy,
   comments, names or documentation (ADR-008).
8. **Commit messages** are short imperative sentences that say what changed and why.
9. **Naming**: Gnostic conventions (Aeons, Archons, Syzygies) for internal logic. Ask before naming a
   new module (`.cursorrules`).

## Workflow dependencies

Every external action in `.github/workflows/` uses a full commit hash verified in
its own upstream repository. Keep the version comment for Dependabot; its weekly
GitHub Actions proposals still need review and passing checks. For Rust toolchain
actions, the action revision and compiler channel are separate: keep the chain's
1.98.1 pin and the intentional newest-stable report as configured.

The auditor and coverage installer use a normal `taiki-e/install-action` release
commit, explicit tool versions, checksum verification and no fallback installer.
Do not pin its moving tool-name tags by hash: those commits become unreachable.
Checkouts do not persist the GitHub credential. Review action updates against
[GitHub's secure-use guidance](https://docs.github.com/en/actions/reference/security/secure-use)
and the action's own documentation. GitHub's workflow lockfile feature requires
Enterprise Cloud; this organisation currently uses GitHub Free, so the workflow
files carry the pins directly.

The devnet image workflow remains manual and publishing still needs the owner's
explicit instruction. A workflow-source check is not a published or validated image.
