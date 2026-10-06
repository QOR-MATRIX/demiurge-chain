## Summary

<!-- What changed and why. Link the roadmap item in docs/DIRECTION.md, or the decision (D-xxx or ADR-xxx) indexed in docs/DECISIONS.md. -->

## Checklist

- [ ] On the roadmap (`docs/DIRECTION.md`), or a new ADR indexed in `docs/DECISIONS.md` was added first
- [ ] No existing ADR was edited to change its decision
- [ ] No deleted documentation was restored or relied on
- [ ] `cd chain && cargo test --workspace` passes, without `SKIP_WASM_BUILD`
- [ ] Launcher tests pass, if `tools/qor-launcher` changed
- [ ] QOR ID tests pass (`--include-ignored`, against Postgres and Redis), if `services/qor-auth` changed
- [ ] ARQADE tests pass (`cd products/arqade && npm test`), if `products/arqade` changed
- [ ] Affected current documents updated in this change
- [ ] CGT amounts are integer Sparks (18 decimals); no floating point
- [ ] No new path that creates CGT outside `--dev`
- [ ] No value listed in `docs/economics/OPEN_QUESTIONS.md` was invented
- [ ] Anything that touches CGT passes the spend test (ADR-002) and names its demand sink (ADR-006)
- [ ] No APY, yield, return, ROI, investment, passive-income or price language in copy, comments, names or docs (ADR-008)
- [ ] No keys, mnemonics or secrets committed

## Testing

<!-- What was run, and the result. -->
