# `chain/`: the Demiurge Substrate L1

The purpose-built Substrate L1 on the Polkadot SDK (ADR-013), located and named by ADR-032. It replaces
the custom Rust devnet that lived in `framework/`, which was retired and deleted at M3.5 on 20 September
2026. This is the only chain in the repository.

## State

**M3, second increment: the chain produces and finalises blocks, and a client outside it now uses both.**
Run on 2026-09-17: a development node authored blocks with Aura and GRANDPA finalised them, which is the
first finality this project has had. The custom devnet it replaced had none and reported every transaction as
finalized regardless; it was deleted at M3.5.

**The QOR Launcher talks to this chain since 2026-09-20** (ADR-040). Its client is `subxt`, it builds
every call from the metadata this node serves, and its live test sends a transfer signed by the
launcher's vault and waits for GRANDPA to finalise it. That is the first thing outside `chain/` to
depend on this runtime, and the first end-to-end use of its finality.

| Piece | State |
| --- | --- |
| `runtime/` (`demiurge-runtime`) | System, timestamp, Aura (ADR-019), GRANDPA (ADR-018), balances, session and the validator set (ADR-020), `pallet-nfts` as the asset ledger (ADR-025), `pallet-drc369`, `pallet-utility` for the atomic batch a trade needs (ADR-053), `pallet-drc369-royalties` (ADR-061), and `pallet-sudo` behind a feature (ADR-037). `spec_version` 6, `transaction_version` 2 |
| `node/` (`demiurge-node`) | Runs. Aura authoring, GRANDPA voting, standard RPC, `dev` and `local` chain specifications |
| `pallets/validator-set` (`pallet-validator-set`) | **Mounted.** A governance-chosen set acting as `pallet-session`'s session manager (ADR-020). Its own tests cover every governance path and the three guards against a halted chain. The chain's validators come from it, verified by reading `Session::Validators` from a running node |
| `pallets/drc369` (`pallet-drc369`) | **Mounted, M4.1 only** (2026-09-22). The content reference (ADR-047's 41 bytes), the pinned commit, revision and the one-way switch to permanent, one singles collection per creator, owner enumeration and the `Drc369Api` runtime API. Every config value's source is in `runtime/src/assets.rs` and ADR-052; the deposits are placeholders (U-14) and the weights are placeholders owed to M7.2. Since 2026-09-29 it also records remix provenance at mint — `derived_from` and `remix_depth`, bounded at 16 (ADR-061). **Since 2026-10-01 it nests** (M4.2's nesting, M4.5's requirement R-2): `nest` and `unnest`, only by the owner of both assets, a cycle refused by a walk of at most eight steps, at most eight levels and sixty-four assets held (ADR-047 decision 13 row 7), and it is `pallet-nfts`'s `Locker`, so a nested asset and the asset holding it cannot be transferred, sold or burned until it is taken out. No deposit of its own. State and XP and physics are not started |
| `pallets/drc369-royalties` (`pallet-drc369-royalties`) | **Mounted as `Drc369Royalties`** (2026-09-29, M4.2's royalty half, ADR-061). Royalty terms set by an asset's creator while they hold it — up to eight recipients and a remix share that never rises once the work is remixed (ADR-062) — and a listing bought and settled in CGT that pays a remix's direct source, then the asset's recipients, then the seller, in one transaction. Every amount comes from one pure function, `split`, pinned at `u128::MAX`. No platform share, no fee, no deposit of its own; weights are placeholders owed to M7.2 |
| `pallet-sponsorship`, `pallet-agent-caps` | Not written yet. Names confirmed by the owner on 2026-09-17 and checked for collisions |

## What it deliberately does not contain

Values that [`docs/economics/OPEN_QUESTIONS.md`](../docs/economics/OPEN_QUESTIONS.md) leaves undecided are
not invented here, under AGENTS.md §5.

- **No transaction payment.** A `WeightToFee` is a fee decision, and fee classes and burn shares are
  OPEN-4. The custom chain charged no fee either, so a development network loses nothing by waiting.
- **No issuance and no genesis allocation.** The issuance rate is OPEN-1 and the genesis split is OPEN-2.
- **No mainnet address prefix.** Prefix 42 is for development and test networks (ADR-024); the mainnet
  prefix is a Public Release criterion.

## The decisions written into it

| Value | What | Record |
| --- | --- | --- |
| Precision | Eighteen decimals; `1 CGT = 10^18 Sparks` | ADR-035 |
| Existential deposit | 100 CGT, `10^20` Sparks | ADR-036, deriving from ADR-030 |
| Ticker | `CGT` | ADR-045 |
| Address prefix | 42, development and test networks only | ADR-024 |
| Block production | Aura | ADR-019 |
| Finality | GRANDPA, standalone chain | ADR-018 |
| Account keys | Sr25519, through `MultiSignature` | ADR-023 |
| Address type | `AccountIdLookup`; an address is a `MultiAddress` | ADR-041 |
| `pallet-sudo` | The `sudo` feature: development and test networks only, absent from mainnet | ADR-037 |
| Pinned SDK | `polkadot-sdk = "=2606.1.0"`, exact | ADR-022, ADR-033 |
| Assets | `pallet-nfts` bounds from ADR-047 and Asset Hub Westend at the pinned tag; placeholder deposits; only `pallet-drc369` creates an asset; nesting at most 8 deep and 64 wide, with `pallet-drc369` as the ledger's `Locker` | ADR-025, ADR-047, ADR-052 |

### The pin: noted and deferred, not stale

On 2026-09-17, resolving this workspace reported that **2606.2.0, 2606.3.0 and 2606.4.0 exist** while we
are pinned to 2606.1.0. The owner's decision that day: **stay on 2606.1.0.** Moving is its own task with
its own verification pass under ADR-033 rule 3, and mid-M3 is the wrong moment for it. ADR-022 had already
declined 2606-2 once, for the same reason.

**A move is proposed once the node and `pallet-validator-set` are in and the chain is producing blocks
under a governance-chosen validator set — not before.** This paragraph exists so the next person reads the
pin as deliberate rather than forgotten.

## Which chain is this

**The only one.** Until 20 September 2026 a second chain in this repository also produced blocks and also
defaulted to port 9944 — the custom Rust devnet in `framework/`, which had no finality, accepted forged
signatures for small-order keys (requirement R-1), and was never to be exposed. It was retired at M3.5,
and CI fails if the directory returns.

To ask a running node which chain it is, which is still worth doing:

```bash
curl -s -H 'Content-Type: application/json'   -d '{"jsonrpc":"2.0","id":1,"method":"system_chain","params":[]}' http://127.0.0.1:9944
```

This chain answers `"Demiurge Development"` or `"Demiurge Local Testnet"`. The launcher asks
automatically and shows the answer in its Chain surface (ADR-040).

## Building it

```bash
cd chain
cargo test                      # the runtime, node and pallet tests
SKIP_WASM_BUILD=1 cargo check    # fast, while iterating: Rust only, no wasm
cargo build --release            # the real thing, including the runtime's wasm
```

Two validators, agreement, finality and catch-up need real nodes, so they are a script rather than a
cargo test. It takes about two minutes:

```bash
cargo build -p demiurge-node --release --features sudo
node scripts/check-two-validators.mjs
```

`SKIP_WASM_BUILD=1` is for iterating. It is never evidence that the runtime builds: the wasm is what the
chain executes and what a runtime upgrade replaces.

A fresh account on a development node needs CGT before it can mint, because a mint holds deposits
(ADR-052). `scripts/dev-fund.mjs` sends it from Alice's development endowment. It is a transfer, so it
creates nothing, and it refuses any node that does not report a `Development` chain type:

```bash
cd scripts && npm install
node dev-fund.mjs <address> [amount in CGT, default 10000]
```

The launcher's live test is the other end of the same evidence: start a node, then run it from the
launcher's host crate.

```bash
./target/release/demiurge-node --dev --tmp
cd ../tools/qor-launcher/src-tauri && cargo test --lib chain::live -- --ignored --nocapture
```

## The two runtime configurations

`pallet-sudo` is behind the `sudo` feature (ADR-037), so removing it before mainnet is a build
configuration rather than a code edit under time pressure:

```bash
cargo build --release --features sudo   # development and test networks
cargo build --release                   # the mainnet shape: no sudo in the metadata
```

`public-release.no-sudo` in [`docs/GATES.toml`](../docs/GATES.toml) is met by the released runtime's
metadata showing no sudo call, not by a statement that sudo is unused.

**That path is demonstrated, not assumed.** Both shapes are built and their metadata read from a running
node. Re-measured on **2026-09-29**, after M4.2's royalty half added `pallet-drc369-royalties` (`spec_version` 4):
the development runtime's metadata is **98,230 bytes** and contains `Sudo`; the mainnet shape's is **96,322 bytes**
and does not (measured after ADR-062 added `Drc369::RemixCount`). Both contain `Nfts`, `Drc369`, `Utility` and `Drc369Royalties`. `TransactionPayment` is absent from
both, because fees are OPEN-4.

**Half re-measured since.** Nesting (`spec_version` 5, 2026-10-01) and `buy_exact` with the runtime API
`Drc369RoyaltiesApi` (`spec_version` 6, 2026-10-02) grew both. On **2026-10-02** a release node built without the
`sudo` feature, at `spec_version` 6, served **99,901 bytes** of metadata with no `Sudo` in it: that is the mainnet
shape. **The development shape (`--features sudo`) was not re-measured**; its last measurement is the 98,230 above.

On 2026-09-22, after M4.6 added `pallet-utility` (`spec_version` 3), they were 92,323 and 90,417. Earlier that day, after M4.1 added `pallet-nfts` and `pallet-drc369` (`spec_version` 2), they were
86,259 and 84,316 — and a node still carrying that runtime served exactly 86,259 bytes when it was restarted
to take the upgrade, which is the check that these numbers are measurements and not recollections. The
numbers after ADR-041 were 44,853 and 42,867, measured on 2026-09-20; before it, 38,517 and 36,532,
measured on 2026-09-17. They are recorded here because
a metadata size is evidence only for the runtime it was measured from: **re-measure both shapes whenever
the runtime changes**, rather than carrying a number forward.
