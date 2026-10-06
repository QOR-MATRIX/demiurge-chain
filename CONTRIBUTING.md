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
6. **Money is integers.** CGT amounts are Sparks in `u128`, 18 decimals. No floating point. Never
   invent a value that [`OPEN_QUESTIONS.md`](docs/economics/OPEN_QUESTIONS.md) lists as undecided.
7. **Language.** No APY, yield, return, ROI, investment, passive-income or price language in copy,
   comments, names or documentation (ADR-008).
8. **Commit messages** are short imperative sentences that say what changed and why.
9. **Naming**: Gnostic conventions (Aeons, Archons, Syzygies) for internal logic. Ask before naming a
   new module (`.cursorrules`).
