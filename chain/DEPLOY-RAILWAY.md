# Deploying the devnet on Railway

The record of how `Demiurge Devnet` runs on Railway (ADR-068): what exists, how it was made, and what to do
again. The plan and its reasons are [`../docs/architecture/DEVNET_PLAN.md`](../docs/architecture/DEVNET_PLAN.md).

**Status: live since 3 October 2026.** All seven steps are done. `Demiurge Devnet` answers at
`wss://rpc.qorsync.dev` (HTTP JSON-RPC on the same host): two validators and an RPC node on Railway, genesis
`0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a`, `spec_version` 8 since the owner's sudo runtime
upgrades of 4 October 2026 (the last entries of "What exists").

## The rule for secrets

As in [`../services/qor-auth/DEPLOY-RAILWAY.md`](../services/qor-auth/DEPLOY-RAILWAY.md): no secret is pasted into a
shell, a chat or a document. Each node makes its own keys on its own volume at first boot and prints only public
values (`DEMIURGE_PUBLIC` lines). The sudo and faucet keys are the owner's, in a browser wallet; only their
addresses are used here.

## The image

`ghcr.io/qor-matrix/demiurge-node:devnet`, built from `chain/Dockerfile` by the manual workflow **Devnet node
image** (`.github/workflows/devnet-image.yml`), with the `sudo` feature. If Railway cannot pull it, the package is
private: GitHub → QOR-MATRIX → Packages → `demiurge-node` → Package settings → Change visibility → Public.

## The services

Project `demiurge`, environment `production`, beside `qor-auth`, `Postgres` and `Redis`. Region `iad` for all three.

| Service | Source | Volume | Public domain | Variables |
| --- | --- | --- | --- | --- |
| `devnet-validator-a` | the image | `/data` | **none, ever** | `DEMIURGE_ROLE=validator`, `DEMIURGE_NODE_NAME=validator-a` |
| `devnet-validator-b` | the image | `/data` | **none, ever** | `DEMIURGE_ROLE=validator`, `DEMIURGE_NODE_NAME=validator-b`, `DEMIURGE_BOOTNODES` (step 3) |
| `devnet-rpc` | the image | `/data` | `rpc.qorsync.dev` → port 9944 (step 6 only) | `DEMIURGE_ROLE=rpc`, `DEMIURGE_NODE_NAME=rpc`, `DEMIURGE_RPC_CORS=all`, `DEMIURGE_BOOTNODES` (step 3) |

- **No validator gets a public domain or a TCP proxy.** They peer over Railway's private network
  (`<service>.railway.internal`).
- **`DEMIURGE_RPC_CORS=all` on the RPC node only.** The default answers 403 to any request for a public name
  (`DEVNET_PLAN.md` §2). `--rpc-methods safe` is always on for that role and refuses every method that changes the
  node.
- **Validators are never redeployed by a push.** They run a fixed image tag and redeploy only by hand; each redeploy
  pauses finality until it is back (two validators need both).
- Rate limits (`DEMIURGE_RPC_RATE_LIMIT` and the three others) stay unset until measured against the live node.

## The order

1. **Create the three services** from the image, each with its volume at `/data` and the variables above, no
   domain. Each boots, makes its keys and waits, because there is no chain specification yet.
2. **Read the logs** of each: its `DEMIURGE_PUBLIC` lines. Confirm the whole first log holds nothing else of note.
3. **Bootnodes:** set `DEMIURGE_BOOTNODES` on `devnet-validator-b` and `devnet-rpc` to
   `/dns/devnet-validator-a.railway.internal/tcp/30333/p2p/<validator-a's peer id>` (the RPC node may also list B).
4. **The specification:** `devnet_chain_spec` is given both validators' public keys and the owner's sudo and faucet
   addresses; the raw specification is built, committed under `chain/specs/`, and baked into the image, which is
   rebuilt by running the workflow again. `DEMIURGE_CHAIN_SPEC` then names it on all three services.
5. **Redeploy** the three on the new image. Check from the RPC node, over the private network: both validators
   author, hashes agree, finality advances.
6. **Public name:** on `devnet-rpc` only, add the custom domain `rpc.qorsync.dev`, port 9944, and the record Railway
   shows in Cloudflare, set as `id.qorsync.dev` is.
7. **Check publicly:** `https://rpc.qorsync.dev/health/readiness` 200, `system_chain` "Demiurge Devnet", an unsafe
   method refused. Then the URL goes into `GATES.toml` and `alpha.devnet-finality` is measured.

## What exists

- **3 October 2026, the image.** Workflow run on `e120241` succeeded (18 m 55 s); the package was made public by the
  owner (an organisation setting had to allow public packages first). The services are pinned to
  `ghcr.io/qor-matrix/demiurge-node:sha-e120241`, not `:devnet`, so a later build never changes a running validator.
- **3 October 2026, steps 1 and 2.** The three services were staged through Railway's tools (region `iad`, volume at
  `/data` each, the variables above, no domain) and applied by the owner in the dashboard. First boot, 15:25 UTC, read
  from each log: only the boot script's progress lines (Railway labels its standard-error lines "error"; they are not
  errors) and these public values. No secret appeared.

| Service | Railway id | Peer id | Aura (Sr25519) | GRANDPA (Ed25519) |
| --- | --- | --- | --- | --- |
| `devnet-validator-a` | `162cfa79-0468-4c21-b6bf-383737b5600f` | `12D3KooWGeHvkUaGNGafZHuRZt9ewY3Nhd3Bn9tDRM41m3aHqfY5` | `5GucVTnpdUsNryEyH3YE6hbp1ZBTL3kopxFot639nxFRxrDa` (`0xd63de57626c37dea0c6328254dd25053581616c50a14f9f5cc54d8a3a9257841`) | `5CWsNNyHQXkose1Fw2fu1Ne6mSCq1Dtm4kxasV2QA8jkYfLm` (`0x13f466d0baf2cca085c1b4197fe9ee324c5ed715313740bab9a995510488e78c`) |
| `devnet-validator-b` | `b3be7f64-4be5-4fc6-a33d-be4f29a8032f` | `12D3KooWGg2UhBn1wr3nnf7mdBrG9MoBNB6J6qx9hKPohrKq47iK` | `5CPZLYtgagoUebiFHAoCu4Ush3CL9RDxq7smKtFBvixaTBgc` (`0x0e60fbf711e6a81b52a7b809a2f5ebbbae81ba9411670a2bd8b60a8581e0f074`) | `5Gxyx5psvaVw6JLbtZWo1jQsPfB4bSmgFFxjh7z5aJAq9B9D` (`0xd8cfdeb462210331996d2ae47a629f64bf6ebc0cb51116958e64d76f105f01e9`) |
| `devnet-rpc` | `85422843-06ae-408b-9241-feaaab989d03` | `12D3KooWDwW6HjmsHmWHhqFLqJFXS8LP8FK2pP6TMfat9ToexeYc` | — | — |

- **3 October 2026, step 3.** `DEMIURGE_BOOTNODES` set on `devnet-validator-b` (A) and `devnet-rpc` (A and B), with
  deploys skipped: they take effect at the redeploy in step 5.
- **3 October 2026, step 4.** The owner made two accounts in the polkadot{.js} extension: sudo
  `5HN6PZA4zkAMeump3zs9beg66kdPiAqaXbwBHKYxtEaXZadn`, faucet `5GuugU7trpXPfx1kfDza8zS146jYwvmHHkrgTFEnRgZh34T9`.
  `chain_spec::devnet` holds those and the four validator keys; `--chain demiurge_devnet` loads them; a test checks
  every key against the hex the boot logs printed (a swap of the two GRANDPA keys was planted and caught). The raw
  specification `chain/specs/demiurge_devnet.raw.json` (5.2 MB) was built with the `sudo` feature and **read back from
  a node started on it**: chain "Demiurge Devnet", `Live`, spec_version 6, **genesis
  `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a`**, sudo key the owner's, sudo 100 CGT, faucet
  1,000,000 CGT, total issuance 1,000,100 CGT, validators A and B. The image copies it to
  `/etc/demiurge/demiurge_devnet.raw.json`.
- **3 October 2026, the second image.** The first rerun built `e120241` again (the push had not reached GitHub);
  after the owner pushed, the run on `f836075` published `sha-f836075`, whose labels name that commit and whose
  `/etc/demiurge/demiurge_devnet.raw.json` has sha256 `2c7fbc90…d60d`, the committed file's.
- **3 October 2026, step 5.** The three services' image and `DEMIURGE_CHAIN_SPEC` were staged through Railway's tools
  and applied by the owner (20:23 UTC). All three initialised genesis `0x934e…254a`; validator-a authored #1 and
  validator-b #2, and #1 was finalised by #3.
- **3 October 2026, steps 6 and 7.** `rpc.qorsync.dev` attached to `devnet-rpc`, port 9944, through Railway's tools;
  the owner added the CNAME (`rpc` → `8634ia3g.up.railway.app`, DNS only, as `id.qorsync.dev`) and the TXT
  (`_railway-verify.rpc`) in Cloudflare; the certificate was issued. Measured publicly: `/health` 200
  `{"peers":2,"isSyncing":false}`, `/health/readiness` 200; `system_chain` "Demiurge Devnet", `Live`, spec_version 6,
  genesis `0x934e2caa…254a`; `author_rotateKeys` and `author_insertKey` refused (-32601, unsafe); `sudo.key` the
  owner's; faucet 1,000,000 CGT. **Ten minutes, 20:29–20:39 UTC:** finalised #55 → #155, authors validator-a 50 and
  validator-b 51, and both validators' logs reported finalized #155 `0xe42b…5407`, the RPC node's hash
  (`alpha.devnet-finality`). The launcher's read-only Market test passed against it (block 80, 0 listings).
- **3 October 2026, the first transfer.** The owner sent 100,000 CGT from the faucet to their launcher account
  `5DMPEXVc…qLxK` through polkadot.js Apps against `wss://rpc.qorsync.dev`. Read back after finalised #969: launcher
  100,000, faucet 900,000, sudo 100, total issuance unchanged at 1,000,100 CGT; five accounts exist (those three and
  the two validators' session accounts at 0).
- **4 October 2026, `spec_version` 7: the ARQ Wallet.** The upgrade was rehearsed first against a copy of the devnet's
  own runtime (`scripts/rehearse-upgrade.mjs`). The owner then signed `sudo.sudoUncheckedWeight(system.setCode(wasm))`
  with the sudo account in polkadot.js Apps. Read back from `wss://rpc.qorsync.dev`: spec_version 7,
  `transaction_version` 2, `:code` 516,222 bytes and the rehearsed file's SHA-256, `Sudo::Key` unchanged, `ArqWallet`
  at index 11 with 12 calls, finality advancing with both validators authoring, readiness 200.
- **4 October 2026, `spec_version` 8: ARQ Wallet rounds.** Rehearsed against the devnet's live version-7 runtime, then
  signed by the owner the same way. Read back: spec_version 8, `:code` 516,949 bytes and the rehearsed file's
  SHA-256, `Sudo::Key` unchanged, `ArqWallet` with 15 calls, `open_round` among them, finality advancing, readiness
  200. **Neither upgrade needed a new image or a redeploy:** the runtime runs as wasm from chain state, so the services
  stayed on `sha-f836075`, and the genesis is unchanged.
