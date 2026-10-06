# Instructions for AI assistants

The full rules are in [`AGENTS.md`](../AGENTS.md). The essentials:

1. **Old documentation was deleted. Never restore it from git history or rely on it, for any reason.**
2. Current documents are listed in [`docs/README.md`](../docs/README.md). Start with `HANDOFF.md`, then
   `docs/DIRECTION.md` (the only roadmap), `docs/DECISIONS.md` (the only decision log, indexing the ADRs
   in `docs/decisions/`) and `docs/economics/CGT.md` (the economic model).
3. The chain is the Substrate L1 in `chain/` (ADR-013, ADR-032), and it is the only one. The custom Rust
   devnet that preceded it, in `framework/`, was retired at M3.5 on 20 September 2026 and deleted; do not
   restore it or write against it, and CI fails if the directory returns. Write no migration code until
   the owner has reviewed `docs/architecture/MIGRATION_INVENTORY.md`. The platform is the QOR Launcher in
   `tools/qor-launcher/`. Identity is `services/qor-auth/`. ARQADE, the gaming platform, is
   `products/arqade/` (ADR-069).
4. CGT has 18 decimals. The base supply is 100,000,000,000,000 CGT with a perpetual issuance and fee
   burn (ADR-003, ADR-004). Amounts are integer Sparks in `u128`, never floating point. Never invent a
   value listed in `docs/economics/OPEN_QUESTIONS.md`.
5. No APY, yield, return, ROI, investment, passive-income or price language anywhere (ADR-008).
6. `apps/`, `cli/`, `sdk/`, `packages/` and `client/` are frozen and are not a reference.
7. The code is the truth. Update the current document in the same change as the behaviour.
