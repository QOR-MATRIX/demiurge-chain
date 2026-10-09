# Current documentation

**These are the only current documents.** Anything not listed here is not current. The
pre-realignment documentation was deleted on 2026-09-13 and must never be restored or consulted,
including from git history (D-012).

| Document | Purpose |
| --- | --- |
| [`OWNER.md`](../OWNER.md) | **For the owner.** One page: what exists, what is next, and what only he can do. Rewritten every session |
| [`README.md`](../README.md) | The repository's front door: what Demiurge is, the repository map, how to run it |
| [`HANDOFF.md`](../HANDOFF.md) | **Start here.** State at the end of the last working session, and what is next |
| [`docs/DIRECTION.md`](DIRECTION.md) | What Demiurge is, current status, **the only roadmap**, scope |
| [`docs/GATES.toml`](GATES.toml) | Release gates (Alpha, Beta, Public Release): the only definition of their criteria, exactly how progress is computed, and the rule for changing evidence. Accepted |
| [`docs/DECISIONS.md`](DECISIONS.md) | **The only decision log**: decisions D-000 to D-012, and the index of the architecture decision records |
| [`docs/decisions/`](decisions/README.md) | Architecture decision records ADR-001 onwards, one file per decision |
| [`docs/economics/CGT.md`](economics/CGT.md) | **The economic model**: supply, issuance and burn, the Mesh, demand sinks, display layer, language |
| [`docs/economics/OPEN_QUESTIONS.md`](economics/OPEN_QUESTIONS.md) | Every economic value not yet decided, and what would decide it |
| [`docs/protocol/PROTOCOL.md`](protocol/PROTOCOL.md) | Exactly what the chain does today: transactions, blocks, state root, slots, import rules, genesis, networking |
| [`docs/architecture/MIGRATION_INVENTORY.md`](architecture/MIGRATION_INVENTORY.md) | Every module of the custom chain mapped to its Polkadot SDK equivalent (ADR-013), with the frictions and the questions. Q-1 to Q-18 are decided (ADR-017 to ADR-032, confirmed by the owner on 2026-09-17; ADR-041; ADR-057), Q-19 and Q-20 are open, and every claim was re-checked against the pinned release |
| [`docs/audit/RECONCILIATION.md`](audit/RECONCILIATION.md) | The code measured against the direction: supply hard-codes, reversibility, contradictions, primitive status, critical path, language, visual identity |
| [`docs/SYSTEMS.md`](SYSTEMS.md) | **Every system this repository names**, what it actually is, and whether it exists. Plain language, one table. Rewritten at the end of every session |
| [`docs/blueprints/ECOSYSTEM.md`](blueprints/ECOSYSTEM.md) | **The products and the one substrate they share**, written for the first six (ARQADE has its own blueprint, below): how they compose, the substrate gaps none of them may fill locally, and honest feasibility. Read before any product blueprint. Intent, not a roadmap: the products' items are in `DIRECTION.md` |
| [`docs/blueprints/qontrol.md`](blueprints/qontrol.md) | Qontrol, version control for creative work: git-compatible on disk, gitoxide reads, libgit2 stages in a separate process. Its first surface, Projects, is built, line-by-line diffs included |
| [`docs/blueprints/qfx.md`](blueprints/qfx.md) | QFX, the launcher's living visual layer, in three layers, with the contrast and frame-time contracts as testable requirements. Layer one's first slice is built |
| [`docs/blueprints/qq.md`](blueprints/qq.md) | QQ, the QOR Engine (ADR-081, ADR-083): a Qt 6 engine for virtual worlds, with a native Studio, a Player for the launcher and ARQADE's browser, and an agent that designs and builds; the TypeScript preview that runs meanwhile. P3.1 started on 9 October 2026 |
| [`products/qq/README.md`](../products/qq/README.md) | QQ's code: what is built, how to build and test it with the owner's Qt, how its tests were proven, and why it is not in CI yet |
| [`docs/blueprints/qor-engine.md`](blueprints/qor-engine.md) | QOR Engine as the withdrawn Godot plan. **Superseded by ADR-081**, and replaced by [`qq.md`](blueprints/qq.md) (ADR-082); kept as a record, not a basis for building |
| [`docs/blueprints/gnosis.md`](blueprints/gnosis.md) | GNOSIS, music production whose project format is a text, diffable, mergeable tree designed for Qontrol. Not started |
| [`docs/blueprints/market.md`](blueprints/market.md) | Market and Library expanded: one listing model for games, tools, editor plugins, QFX scenes, presets and themes. Market's first slice is in the launcher (every listing on chain, buying, no search); Library is not started. L7.1 and L7.2 |
| [`docs/blueprints/stream.md`](blueprints/stream.md) | Stream, a placeholder name for music distribution and licensing over the Mesh. Does not exist before M8, and depends on U-6 |
| [`docs/blueprints/arqade.md`](blueprints/arqade.md) | ARQADE, the gaming platform: the owner's 4 October 2026 handoff corrected against the tree, with what changed and why. QOR ID, DRC-369 only, CGT only, a developer SDK. P7, ADR-069 (accepted 4 October 2026) |
| [`products/arqade/sdk/README.md`](../products/arqade/sdk/README.md) | The ARQADE SDK: what exists (CGT amounts as integer Sparks, ARQ Wallet policy checks), what is proposed and what each waits on. Unpublished; names are placeholders |
| [`products/arqade/sdk/docs/arq-wallet.md`](../products/arqade/sdk/docs/arq-wallet.md) | For game developers: ARQ Wallets, one keyless payout account per published game, and "ignition", how to unlock one. ADR-070, accepted; built on the chain (`pallet-arq-wallet`) and live on Demiurge Devnet |
| [`products/arqade/sdk/docs/publishing.md`](../products/arqade/sdk/docs/publishing.md) | For game developers: self-publishing on ARQADE, the four stages and their checks, prices up to 10,000 CGT, in-game items. ADR-071, accepted; the checks are built |
| [`products/arqade/sdk/docs/backing.md`](../products/arqade/sdk/docs/backing.md) | For game developers: backing in CGT (campaigns, memberships, tips), rewards and never proceeds. ADR-071, accepted; tips are live (ARQADE's own games, devnet test CGT, paid through the launcher); campaigns and memberships are not built |
| [`products/arqade/sdk/docs/building-with-agents.md`](../products/arqade/sdk/docs/building-with-agents.md) | For game developers: building a game with an external LLM through the Demiurge MCP server. ADR-071 and ADR-010; not built |
| [`products/arqade/README.md`](../products/arqade/README.md) | ARQADE's front door: what it is, how to run and test it (`npm test`), and how it is deployed on Vercel (ADR-074) |
| [`products/arqade/IMPLEMENTATION.md`](../products/arqade/IMPLEMENTATION.md) | ARQADE's own record of what is built in `products/arqade/` and what is not, kept with the code |
| [`products/arqade/INTEGRATION_INVENTORY.md`](../products/arqade/INTEGRATION_INVENTORY.md) | Every ARQADE control, statistic, status and terminal command: its source of truth, permission, persistence, failure behaviour and verification (P7.1) |
| [`docs/REALIGNMENT-2026-09-21.md`](REALIGNMENT-2026-09-21.md) | A dated record of the 21 September 2026 realignment: what was measured, what was found false, the fix list in three buckets, and whether the asset work can start |
| [`docs/architecture/HOSTING.md`](architecture/HOSTING.md) | Where every component is hosted, the subdomain map for `qorsync.dev` and `demiurge.cloud`, cross-domain authentication, the validator and RPC topology, email, cost and certificates. A plan written 20 September 2026, largely carried out: QOR ID, Postgres, Redis and the devnet on Railway (ADR-063, ADR-068), ARQADE on Vercel (ADR-074) |
| [`docs/architecture/DEVNET_PLAN.md`](architecture/DEVNET_PLAN.md) | Decided 3 October 2026 (ADR-068): how a two-validator test network reaches a public address, what `chain/node/` lacks to produce a specification without development keys, three hosting options with cited prices, and the nine decisions the owner made. Carried out: the devnet is live at `wss://rpc.qorsync.dev` |
| [`docs/architecture/RUST_TOOLCHAIN.md`](architecture/RUST_TOOLCHAIN.md) | Why the chain pins Rust 1.98.1: why CI failed on Rust 1.99 (its clippy linted code the SDK's macros generate) and the pin that fixed it. Decided 4 October 2026, option A, ADR-072; kept as the record |
| [`docs/architecture/SPONSORSHIP.md`](architecture/SPONSORSHIP.md) | How sponsors are designated, what they pay, caps, draining and fee classes (U-4). **A proposal awaiting the owner's decision** |
| [`docs/architecture/AGENT_KEY_CUSTODY.md`](architecture/AGENT_KEY_CUSTODY.md) | How QOR ID stops holding agent private keys. **Decided and implemented** (ADR-014, migration 011); §1 and §2 describe the exposure as it was before the fix, not as it is |
| [`docs/architecture/ADDRESS_TYPE.md`](architecture/ADDRESS_TYPE.md) | The runtime's address type, `IdentityLookup` or `AccountIdLookup`: what each costs, what breaks if it changes after M5.1, and the recommendation (Q-17, F-Q10). **Decided 2026-09-20 (ADR-041); kept as the record of the proposal, with one cost estimate corrected** |
| [`docs/architecture/PLATFORM_REALIGNMENT.md`](architecture/PLATFORM_REALIGNMENT.md) | The September 2026 audit: what was wrong and why. A record, not a plan |
| [`chain/README.md`](../chain/README.md) | The Substrate L1: what is mounted, how to build and run it, and the pin. **The only chain** |
| [`services/qor-auth/DEPLOY-RAILWAY.md`](../services/qor-auth/DEPLOY-RAILWAY.md) | QOR ID, its Postgres and its Redis on Railway (ADR-063): how every service is set up, since Railway keeps its settings on the service |
| [`chain/DEPLOY-RAILWAY.md`](../chain/DEPLOY-RAILWAY.md) | The devnet on Railway (ADR-068): the image, the three services and their variables, the order of first boot, and what exists |
| [`scripts/run-local-stack.md`](../scripts/run-local-stack.md) | How to run a development node, a multi-validator devnet, the identity service and the launcher |
| [`tools/qor-launcher/README.md`](../tools/qor-launcher/README.md) | Launcher architecture and security design |
| [`docs/design/DESIGN_SYSTEM.md`](design/DESIGN_SYSTEM.md) | How every surface looks and behaves: tokens, themes, type and spacing scales, components, motion, accessibility settings, what is ruled out |
| [`.cursorrules`](../.cursorrules) | Project law: naming, CGT, supply, economics, language, documentation |
| [`AGENTS.md`](../AGENTS.md) | Rules for AI assistants working in this repository |
| [`MISSING_ASSETS.md`](../MISSING_ASSETS.md) | Binary assets absent from the repository |
| [`CONTRIBUTING.md`](../CONTRIBUTING.md) | How to make a change |
| [`SECURITY.md`](../SECURITY.md) | Reporting vulnerabilities; security status |

## Rules

1. The code is the source of truth for what runs. If a current document disagrees with the code
   about what runs, fix the document in the same change. Documents that state direction (the ADRs,
   `CGT.md`) say plainly where the code does not follow them yet.
2. There is one roadmap (`DIRECTION.md`) and one decision log (`DECISIONS.md`, which indexes the ADRs).
   There is one economic model (`economics/CGT.md`). Do not start a second one of any of them.
3. Adding a new current document means adding it to this table. A document that is not in this
   table is not current.
4. When a document stops being true, update it, or delete it and remove it from this table. Outdated
   documents are not kept. An ADR is the exception: it is never edited to change its decision; a later
   ADR supersedes it, and both say so.
