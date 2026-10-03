# Deploying the devnet on Railway

The record of how `Demiurge Devnet` runs on Railway (ADR-068): what exists, how it was made, and what to do
again. The plan and its reasons are [`../docs/architecture/DEVNET_PLAN.md`](../docs/architecture/DEVNET_PLAN.md).

**Status, 3 October 2026:** not deployed. The image workflow is running for the first time.

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

Nothing yet. Each step above is recorded here, with its date, as it is done.
