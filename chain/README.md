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
| `runtime/` (`demiurge-runtime`) | System, timestamp, Aura (ADR-019), GRANDPA (ADR-018), balances, session and the validator set (ADR-020), `pallet-nfts` as the asset ledger (ADR-025), `pallet-drc369`, `pallet-utility` for the atomic batch a trade needs (ADR-053), `pallet-drc369-royalties` (ADR-061), `pallet-arq-wallet` (ADR-070), and `pallet-sudo` behind a feature (ADR-037). `spec_version` 8 in the tree (the devnet runs 7), `transaction_version` 2 |
| `node/` (`demiurge-node`) | Runs. Aura authoring, GRANDPA voting, standard RPC, `dev` and `local` chain specifications, the SDK's `key` subcommand, and `devnet_chain_spec` for the hosted devnet (not yet a built-in `--chain`: see "The devnet image") |
| `pallets/validator-set` (`pallet-validator-set`) | **Mounted.** A governance-chosen set acting as `pallet-session`'s session manager (ADR-020). Its own tests cover every governance path and the three guards against a halted chain. The chain's validators come from it, verified by reading `Session::Validators` from a running node |
| `pallets/drc369` (`pallet-drc369`) | **Mounted, M4.1 only** (2026-09-22). The content reference (ADR-047's 41 bytes), the pinned commit, revision and the one-way switch to permanent, one singles collection per creator, owner enumeration and the `Drc369Api` runtime API. Every config value's source is in `runtime/src/assets.rs` and ADR-052; the deposits are placeholders (U-14) and the weights are placeholders owed to M7.2. Since 2026-09-29 it also records remix provenance at mint — `derived_from` and `remix_depth`, bounded at 16 (ADR-061). **Since 2026-10-01 it nests** (M4.2's nesting, M4.5's requirement R-2): `nest` and `unnest`, only by the owner of both assets, a cycle refused by a walk of at most eight steps, at most eight levels and sixty-four assets held (ADR-047 decision 13 row 7), and it is `pallet-nfts`'s `Locker`, so a nested asset and the asset holding it cannot be transferred, sold or burned until it is taken out. No deposit of its own. State and XP and physics are not started |
| `pallets/drc369-royalties` (`pallet-drc369-royalties`) | **Mounted as `Drc369Royalties`** (2026-09-29, M4.2's royalty half, ADR-061). Royalty terms set by an asset's creator while they hold it — up to eight recipients and a remix share that never rises once the work is remixed (ADR-062) — and a listing bought and settled in CGT that pays a remix's direct source, then the asset's recipients, then the seller, in one transaction. Every amount comes from one pure function, `split`, pinned at `u128::MAX`. No platform share, no fee, no deposit of its own; weights are placeholders owed to M7.2 |
| `pallets/arq-wallet` (`pallet-arq-wallet`) | **Mounted as `ArqWallet` at index 11** (2026-10-04, ADR-070, ARQADE's P7.11). One keyless payout account per published game, derived from its DRC-369 Cartridge (`PalletId` `dmg/arqw`) and governed by whoever holds the Cartridge: a policy on chain (largest payout, epoch budget, cap per recipient, rule versions), tightened at once and loosened only after its delay; a payout authority that can only pay within it; each outcome paid once inside an outcome window; payouts that cannot reach a player owed and claimable, and never withdrawn; withdrawals delayed and never taking the deposit or what is owed. **Since `spec_version` 8, rounds**: a prize held before a paid round opens, settled to winners or released, never stranded. Its protocol bounds are **U-16 placeholders** marked in `runtime/src/assets.rs`; weights are placeholders owed to M7.2. **On Demiurge Devnet since 4 October 2026** (spec_version 7, set by the owner's sudo key after a rehearsal) |
| `pallet-sponsorship`, `pallet-agent-caps` | Not written yet. Names confirmed by the owner on 2026-09-17 and checked for collisions |


**Rust is pinned to 1.98.1** by `rust-toolchain.toml` (ADR-072): Rust 1.99's clippy lints code the pinned SDK's pallet
macros generate. rustup installs 1.98.1, `clippy`, `rustfmt` and `wasm32v1-none` on the first `cargo` run here. The pin
moves deliberately, as the SDK's does; CI's `chain-newest-clippy` job reports what the newest Rust would say.

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

## Keys

The node carries the SDK's standard `key` subcommand (`sc_cli::KeySubcommand`), mounted on 3 October 2026:

```bash
demiurge-node key generate --scheme sr25519            # prints a new secret phrase: never into a log
demiurge-node key inspect --public --scheme ed25519 0x<public key hex>
demiurge-node key insert --keystore-path <dir> --key-type aura --scheme sr25519 --suri <file holding the phrase>
demiurge-node key generate-node-key --file <file>      # the network key; the peer id goes to stderr
demiurge-node key inspect-node-key --file <file>       # prints the peer id
```

`--suri` and `inspect`'s URI accept a **file path**, and the file's content is used. Pass secrets that way, never
as an argument, which a process list or a shell history shows.

## The devnet image

The hosted devnet is `docs/architecture/DEVNET_PLAN.md`. Host Railway, two validators and a separate RPC node,
chain `Demiurge Devnet` (`demiurge_devnet`, type `Live`): the owner's decisions of 3 October 2026. **Nothing is
deployed yet**, and there is no built-in `--chain demiurge_devnet`: the validators' public keys come from their
first boot, so the specification is written after it (the plan's §1.4).

| Piece | What |
| --- | --- |
| `node/src/chain_spec.rs`, `devnet_chain_spec(validators, sudo, faucet)` | The specification, from public keys and addresses. Genesis holds exactly two balances: the faucet's test CGT, the same marked placeholder as `DEVELOPMENT_ENDOWMENT` (1,000,000 CGT), and the sudo account's existential deposit so it can sign. Validators hold nothing. Its tests pin the name, id and type, the two balances against the runtime's own genesis build, the validators and sudo key, its refusals, and that no SDK keyring key appears in it, readable or raw |
| `Dockerfile` | Multi-stage. The builder is CI's recipe; the runtime stage is `debian:bookworm-slim` with the binary, the boot script, `jq` and `setpriv`. Ports 9944 (RPC) and 30333 (peer-to-peer). Data at `/data` |
| `.dockerignore` | Keeps `target/` and `node_modules/` out of the build context |
| `docker/entrypoint.sh` | The first-boot script. Its header documents every variable |
| `.github/workflows/devnet-image.yml` | Started by hand only. Builds the image and publishes `ghcr.io/qor-matrix/demiurge-node:sha-<short>` and `:devnet` |

The build context is `chain/`, not the repository root (whose `Dockerfile` is the deleted `framework/` chain's):

```bash
docker build -f chain/Dockerfile -t demiurge-node chain
```

**One image, two roles**, set by `DEMIURGE_ROLE`:

- `validator`: `--validator --force-authoring`, keystore at `/data/keystore`, RPC left on localhost.
- `rpc`: `--rpc-external --rpc-methods safe --rpc-port 9944`, no session key, never a validator. Rate limits come
  from `DEMIURGE_RPC_RATE_LIMIT`, `DEMIURGE_RPC_MAX_CONNECTIONS`,
  `DEMIURGE_RPC_MAX_SUBSCRIPTIONS_PER_CONNECTION` and `DEMIURGE_RPC_MAX_BATCH_REQUEST_LEN`, each passed only when
  set. No number is chosen yet: the plan sets them from measurement.

**A public name needs `DEMIURGE_RPC_CORS=all` on the RPC node, which is not yet decided.** In the pinned
`sc-rpc-server` 31.0.0 (`utils.rs`, `host_filtering`) any `--rpc-cors` list, the default included, also turns on a
Host-header filter that admits only `localhost`, `127.0.0.1` and `[::1]` on the RPC port. Measured on 3 October
2026: with the default, `GET /health/readiness` sent as `Host: rpc.qorsync.dev` answered **403**; with
`--rpc-cors all` it answered 200. So the plan's "`--rpc-cors` left default" would refuse every request through
the host's edge. The script passes `DEMIURGE_RPC_CORS` as `--rpc-cors` only when it is set.

Both get `--base-path /data --node-key-file /data/node-key --allow-private-ip --no-telemetry`, and
`--bootnodes` from `DEMIURGE_BOOTNODES` (space-separated). `--allow-private-ip` is there because the nodes peer only
over the host's private network at Alpha, and a `Live` chain otherwise refuses private addresses.

**First boot.** The script makes the network key and, on a validator, an Aura (Sr25519) and a GRANDPA (Ed25519)
key, on `/data`. Every secret goes only to files on `/data` with mode 600. It prints only public values, one per
line, beginning `DEMIURGE_PUBLIC` (`peer_id`, `aura_ss58`, `aura_hex`, `grandpa_ss58`, `grandpa_hex`); later boots
reuse the keys and print the same values. With no specification at `DEMIURGE_CHAIN_SPEC` (default
`/data/spec.json`) it prints those values and **waits** instead of starting a chain.

**Users.** The container starts as root only to give `/data` to the user `demiurge` (uid 10001), because a Railway
volume is mounted owned by root, and then drops to that user before any key is made. The node never runs as root,
so a Railway service needs no `RAILWAY_RUN_UID`.

**Run locally on 3 October 2026, with Docker Desktop.** The image is 236 MB. A validator's first boot on an empty
volume printed the six `DEMIURGE_PUBLIC` lines and waited; its whole log was searched, inside a container, for
both recovery phrases, every three-word run of them, both secret seeds and the network key: no line matched,
and the same search found a phrase planted in a copy of the log. Files on the volume were 600 and directories
700, owned by `demiurge`. A second boot on the same volume printed the same public values. A throwaway
specification made by `devnet_chain_spec` from two such validators' keys then ran two validators and an RPC node
on a Docker network: both validators authored, they agreed on the best block, GRANDPA finalised, the RPC node
answered `system_chain` with `"Demiurge Devnet"` and `system_chainType` with `"Live"`, refused
`author_rotateKeys` and `author_insertKey` ("RPC call is unsafe to be called externally"), and answered
`/health/readiness` with 200. Nothing was deployed and no specification was committed.

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
