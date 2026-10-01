# Demiurge-Cloud

[![Pleroma CI](https://github.com/QOR-MATRIX/demiurge-chain/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/QOR-MATRIX/demiurge-chain/actions/workflows/ci.yml)

> CI runs on GitHub Actions for this public repository (ADR-063): every push and pull request to `main`, and
> two validators nightly.

A layer-1 blockchain whose currency is **CGT, the Creator God Token**, and the **QOR Launcher**, a
desktop platform for holding CGT, signing in with one **QOR ID**, and using games and creative work
built on the chain.

> **Status: pre-release, unaudited, not for real value.** The chain runs as a local development or
> two-validator network on the Polkadot SDK: Aura authors and GRANDPA finalises, and the validator set
> comes from governance. Nothing above the base layer exists yet — no DRC-369, no royalties, no agent
> rails. The economic model was replaced on 2026-09-13 and is not implemented: no issuance, no treasury
> and no transaction fee, because each needs a value that is undecided. Production endpoints are offline
> and nothing is deployed.

## Start here

1. **[`HANDOFF.md`](HANDOFF.md)**: where work stands and what is next
2. **[`docs/DIRECTION.md`](docs/DIRECTION.md)**: what is being built, the roadmap, and scope
3. **[`docs/DECISIONS.md`](docs/DECISIONS.md)**: why, and the index of architecture decision records
4. **[`docs/economics/CGT.md`](docs/economics/CGT.md)**: the economic model
5. **[`docs/protocol/PROTOCOL.md`](docs/protocol/PROTOCOL.md)**: exactly what the chain does today

Every current document is listed in [`docs/README.md`](docs/README.md). Older documentation was
deleted and must never be restored or used.

## CGT at a glance

| | |
| --- | --- |
| What gives it value | Demand to spend it, not demand to hold it (ADR-002) |
| Base supply | 100,000,000,000,000 CGT, mostly allocated at genesis and released over a multi-decade decay curve (ADR-003) |
| Issuance and burn | A small perpetual issuance, targeted under 1% a year, pays for Mesh storage and bandwidth and for validator security; a share of every fee is burned as its counterweight (ADR-004) |
| Precision | 18 decimals. 1 CGT = 10^18 Sparks |

The issuance rate, the genesis split, the decay curve and the burn shares are undecided
([`OPEN_QUESTIONS.md`](docs/economics/OPEN_QUESTIONS.md)), so **none of the supply model is implemented**.
The chain declares no total-supply constant at all and charges no CGT fee
([`CGT.md`](docs/economics/CGT.md) §2 and §8). The earlier 13 billion figure left the code on
20 September 2026.

## Repository map

| Path | What it is | Status |
| --- | --- | --- |
| `chain/` | The chain: a purpose-built Substrate L1 on the Polkadot SDK (ADR-013), named and located by ADR-032. Runtime, node and `pallet-validator-set` | Active |
| `services/qor-auth/` | QOR ID identity service (Rust, Axum, Postgres, Redis) | Active |
| `tools/qor-launcher/` | The QOR Launcher (Tauri 2, Rust host, React) | Active |
| `apps/`, `cli/`, `sdk/`, `packages/`, `client/` | Clients built for the pre-realignment protocol | Frozen (D-011) |

## Running it

[`scripts/run-local-stack.md`](scripts/run-local-stack.md) covers a development node, two validators,
the identity service and the launcher. [`chain/README.md`](chain/README.md) covers the chain on its own.

```bash
cd chain && cargo test --workspace          # chain test suite (not SKIP_WASM_BUILD: it is not evidence)
cd tools/qor-launcher && npm run app:dev    # launcher
```

## Security

The protocol has not been audited. **Do not open a public issue for a vulnerability**; see
[`SECURITY.md`](SECURITY.md), which says how to report one and records the security track, the accepted
lockouts and the dependency advisories being carried.

## Licence

**Apache-2.0** ([`LICENSE`](LICENSE)), matching what `chain/Cargo.toml` already declared. Set by the
owner on 21 September 2026; before that there was no licence file at all, so default copyright applied.
The copyright line reads "Demiurge"; if a legal entity should hold it instead, that line is the one to
change.

**Two inconsistencies are carried deliberately.** Several **frozen** packages declare `MIT` in their own
manifests (`cli/`, `packages/*`, `client/`), and `apps/guru/sophia-demo` declares `ISC`; the **dead**
pallets under `aeons/` declare `MIT`. Those declarations predate the realignment and were not reviewed
when this licence was set. They are not edited here: `apps/`, `cli/`, `sdk/`, `packages/` and `client/`
are frozen scope (D-011, amended by ADR-011), and changing a licence declaration on code already
published under it is a legal act rather than a tidy-up. The root `LICENSE` governs the repository; a
per-package declaration is that package's own claim about itself. Reconciling them is open.
