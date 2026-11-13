# Every system this repository names

Rewritten 13 November 2026 against source and hosted checks at `97dcb13`
on `session/qorsync-domains`. This inventory identifies implementation and ownership; it does not
restate old CI counts or label historical deployments as currently live.
[DIRECTION.md](DIRECTION.md) remains the only roadmap and
[DECISIONS.md](DECISIONS.md) the decision index.

**Source present** means the named implementation exists, not that this session
built it or accepted a device/release. **Frozen** means it is not maintained here
or supported against the current protocol. **Planned** and **named only** do not
claim an implementation. The earlier live checks remain dated in HANDOFF.

## Network and shared core

| System | Verified source and boundary |
| --- | --- |
| Chain | `chain/`, its runtime and pallets are the active Substrate implementation. The compatible QUIC, zlib, Wasmtime and certificate/compression patches pass all 213 hosted workspace tests with WebAssembly built at `3d71009`. The audit retains 38 findings and 26 warnings. The SDK pin and consensus/runtime source are unchanged. A 22:34 UTC read-only public RPC probe identifies runtime 11 and advancing finalized headers; it is not deployment acceptance. |
| DRC-369 | `chain/pallets/drc369/`, `chain/pallets/drc369-royalties/` and the core's `chain/assets.rs` exist. Asset/schema/paid-mechanism acceptance stays in the roadmap and economic decisions. |
| QOR ID | `services/qor-auth/` and core `identity/mod.rs`; the core default is `https://id.qorsync.dev/api/v1`. The 22:33 UTC read-only health/readiness probes pass, including database/cache readiness; authentication was not exercised. |
| Social | `services/social/` and core `social.rs`; the existing provider-host default still needs a verified QORSYNC.DEV service-domain migration. |
| Creator-God Token | Core `cgt.rs` and chain runtime types implement integer amounts with 18 decimals. No economic value or payment capability was introduced by this work. QOR OS 1.0 remains free/test-value only. |
| QOR core | `tools/qor-launcher/core/` is the source used by QOR OS at a pinned commit. `commands.rs` exposes only `core_version`, `token_info`, `parse_amount`, `format_sparks`; richer native modules are not automatically bridge capabilities. |
| Agent rails | Core `agents.rs` exists. The frozen root SDK is not the integration route; caller identity, scope, trusted approval and key isolation remain requirements. |

## Launcher and device surfaces

The desktop host is in `tools/qor-launcher/src-tauri/`; React views are in
`tools/qor-launcher/src/`. The Qt/mobile shell is in `tools/qor-launcher/shell/`.
These are source locations, not claims of a new installed build.

| Surface | Verified entry point and scope |
| --- | --- |
| Gate, profiles, onboarding | `components/gate/`, `components/onboarding/`, `views/ProfilePanel.tsx`, core `profiles.rs` and `vault/`; stable application/keychain identifiers are unchanged. |
| Nexus and Developers | `views/Nexus.tsx`, `lib/systems.ts`; Developers links to QOR OS guidance with repository access required. Six frozen web integrations no longer offer dead public/local destinations. |
| Vault and Chain | `views/VaultView.tsx`, `views/ChainView.tsx`, core `vault/` and `chain/`; no new signing or network action was added. |
| Inventory, Studio and Market | `views/Inventory.tsx`, `views/Market.tsx`, core `chain/assets.rs` and `chain/market.rs`. The old DRC-369 Studio web tile remains frozen. |
| Projects / Qontrol | `views/Projects.tsx`, core `qontrol/`; current project machinery is distinct from frozen client examples. |
| Gates, Settings and Updates | `views/GatesView.tsx`, `views/SettingsView.tsx`, `views/UpdatesPanel.tsx`; configured gates are not successful runs or a release. |
| Library and Mesh | Core `library/` and `mesh/`; their remaining acceptance belongs to L9/L10. |
| QQ and ARQADE surfaces | Launcher integrations remain here; their native product implementations are developed in QOR OS under ADR-126. |
| QOR Mobile | Qt shell source exists. This session did not build, sign, install or test a mobile artifact. |
| QOR OS | [QOR-MATRIX/QOR-OS](https://github.com/QOR-MATRIX/QOR-OS), private. Its [developer guide](https://github.com/QOR-MATRIX/QOR-OS/blob/main/sdk/README.md) maps the supported device/product contribution routes; access is required. |

## Products and website

| Product | Ownership and source |
| --- | --- |
| QQ / QOR Engine | `products/qq/` is frozen here; active development is in QOR OS. Its agent tools are not new Launcher bridge commands. |
| QOR Media | `tools/qor-media/` is frozen here; active development is in QOR OS. |
| ARQADE | `products/arqade/app/` and the web application remain here; Qt app/games are frozen here and developed in QOR OS. Shared rules and recorded fixtures follow ADR-129. The provider-host website link awaits coordinated domain migration. |
| Qontrol | Core `qontrol/` and the Projects surface; the product roadmap remains P1. |
| QFX, GNOSIS, Stream | The product tracks in DIRECTION define these names and remaining work. This session makes no new implementation/readiness claim for them. |
| Market and Library | Core and Launcher sources listed above; completion of a product track or paid model is not implied. |
| Homepage | `site/`; canonical address QORSYNC.DEV. Source links now include QOR OS without changing the network's source ownership. The public homepage returns HTTP 200, but the HTML still lacks the exact QOR OS repository link. Hosting/project management access is unresolved; this source branch is not a deployed update. |

## Tooling, hosting and outstanding acceptance

The existing hosted workflow is `.github/workflows/ci.yml`. The new public-domain
check and its tests run there; the Launcher also checks its actual registry and
content security policy. Launcher spawn tests use Cargo's artifact messages, so
separate build directories no longer break executable discovery. No new framework
was introduced; existing dependencies were patched.
All external actions now use full upstream commit hashes. Checkouts do not persist
credentials; the auditor and coverage tools keep explicit versions and verified
downloads without fallback installation. Dependabot proposes weekly Actions
updates. Hosted run `43519214320` passes the build/test, design/accessibility and
coverage jobs; the service audits still fail. Publication remains manual; no
workflow was dispatched to build or publish an image. The separate CI dispatch
`43522526902` exercises the conditional two-validator recovery test, still
running. Its Rust 1.100.0 report fails on 66 generated DRC-369 `clone_on_copy`
diagnostics, retaining ADR-072's compiler-upgrade blocker. Required Rust 1.98.1
checks pass; no lint was suppressed.
The live Actions allowlist now permits only the nine action repositories used by
the two workflows. Default tokens stay read-only without pull-request approval.
`.github/actions-policy.json` records the verified setting. Global SHA enforcement
waits for the source pins to merge, so current main's jobs are not disabled.



Local 13 November audits of the repaired locks report zero vulnerability advisories
for the core and Launcher host, with 13 and 23 warnings respectively. Each service
still reports one SQLite advisory; stable SQLx cannot resolve the patched version.
The chain's compatible patches reduce its report from 53 to 38 vulnerability
findings, with 26 warnings. No exception was added. Hosted core/host compilation
and lints pass again at `97dcb13`, with 245 core tests passing (10 ignored) and 109 host
tests passing (five ignored). All four repaired spawn tests and the new regression
pass. Service audits, warning remediation and a separately validated QOR OS pin
move remain open; this is not live-service or release acceptance.
An inventory finding alone does not demonstrate exploitation.

[HOSTING.md](architecture/HOSTING.md) records the current verification boundary:
Vercel returned apex/`www` domain assignments, public DNS uses Cloudflare, but
the connected account could not open the assigned project. Public QOR ID health/readiness and the chain's read-only
WebSocket check pass at 22:33-22:34 UTC. Social/ARQADE service
hosts, installed settings, mail sending and replacement releases still need a
coordinated migration. No service or credential configuration was changed.

## Frozen, planned and named only

`apps/`, `cli/`, the root `sdk/`, `packages/` and `client/` stay frozen (D-011).
Retired network defaults/examples touched here use loopback, not a promise of
compatibility with today's public APIs. Legacy deployment helpers touched here
refuse before side effects; old production configuration is marked retired.
Explorer, Sophia, Scatter, Agent Foundry, DRC-369 Studio and Bounties web tiles
are explicitly unavailable. Existing Staking, Resonance and Worlds placeholders
remain placeholders. Developers instead opens the supported contribution guide.

The public SDK, broader agent bridge, MCP server, public viewer, remote console,
indexer, archive-node acceptance and installer work remain in DIRECTION or QOR
OS's masterplan; this source cleanup closes none of those gates. Relays and
agentic synchronisation remain names without a new scope. QOR Wallet names the
existing Vault; VYB Social names the frozen app. No new exchange product or
outside-money mechanism was specified.

`aeons/`, `archons/`, `syzygies/` and `packages/blockchain-wasm` remain outside the
active scope. The retired custom chain is not restored. Open economic values
stay in `economics/OPEN_QUESTIONS.md`; domain maintenance decides none of them.
