# A public multi-validator devnet: the plan for `alpha.devnet-live`

**Status:** Written 1 October 2026; **decided 3 October 2026 by the owner, who chose every recommended answer in §9
([ADR-068](../decisions/ADR-068-the-devnet-on-railway.md)).** For §7's question the owner chose a sudo account with
the existential deposit and a separate faucet account holding marked test CGT. Nothing is deployed yet.

**What it is for.** The Alpha gate has a criterion `alpha.devnet-live` (`docs/GATES.toml`): kind `http`,
`url = ""`, `expect_status = 200`. It is unmeasurable until a URL is written there. Today the chain runs only
on the owner's computer with `--dev`. This plan says what has to exist for a devnet of two validators to be
reachable at a public address, what it costs, and who does what.

**What it must not do.** Put a `--dev` chain on the internet. ADR-063 decision 6 withholds public RPC because a
`--dev` chain's sudo key is the well-known Alice key. Anyone could replace the runtime.

**Records it touches.** ADR-015 puts devnet nodes on Fly.io. ADR-063 moved QOR ID, Postgres and Redis to
Railway and did not reopen the nodes. ADR-044 (validators not publicly addressable, public RPC a separate node)
is still **Proposed**. This plan recommends Railway for the nodes, which **needs a new ADR that supersedes
ADR-015 for the nodes**. It is not a decision until the owner makes it.

---

## 1. What must exist before any node is public

### 1.1 What the code has today (read on 1 October 2026)

| Fact | Where |
| --- | --- |
| Two built-in chain specifications, `dev` and `local`. Any other `--chain` value is read as a path to a JSON file | `chain/node/src/command.rs`, `load_spec` |
| `dev` uses Alice as the one validator. `local` uses Alice and Bob. Both use the SDK's well-known keyring keys for Aura and GRANDPA | `chain/node/src/chain_spec.rs` |
| The sudo key in **both** is Alice, written only when the `sudo` feature is on | `chain_spec.rs`, `genesis()` |
| Both endow every well-known keyring account with 1,000,000 CGT, a marked placeholder | `chain_spec.rs`, `DEVELOPMENT_ENDOWMENT` |
| The node has a `build-spec` subcommand | `chain/node/src/cli.rs` |
| **The node has no `key` subcommand.** `sc_cli::KeySubcommand` exists in the pinned `sc-cli` 0.61.0 and is not mounted | `cli.rs`, `command.rs` |
| The runtime offers no genesis presets (`get_preset` returns `None`) | `chain/runtime/src/lib.rs` |
| A validator will not create its own network key unless told to | `sc-cli` 0.61.0, `node_key_params.rs`; the two-validator script passes `--node-key` for this reason |
| The root `Dockerfile` and `fly.toml` describe the deleted `framework/` chain | Both say so in their own headers. Neither builds `chain/` |

### 1.2 What is missing, precisely

1. **A chain specification with its own keys.** No function and no JSON file describes a network whose
   validators and sudo key are not well-known. This is the item `HANDOFF.md` records as owed.
2. **A way to generate and insert keys with the node itself.** Without the `key` subcommand there is no
   `key generate`, `key insert` or `key generate-node-key`. Mounting `sc_cli::KeySubcommand` is the standard
   node-template arrangement: one enum variant in `cli.rs` and one match arm in `command.rs`.
3. **A container image for `chain/`.** There is none. CI shows what the Linux build needs
   (`.github/workflows/ci.yml`): stable Rust, the `wasm32v1-none` target, `clang`, `libclang-dev`, `llvm-dev`,
   `protobuf-compiler`, then `cargo build -p demiurge-node --release --features sudo --locked`.
4. **A first-boot script** in that image, which creates a node's keys on its own disk (§1.4).
5. **A decision about who holds CGT at genesis** (§7). A new specification cannot be finished without it.

Nothing else in `chain/node/` blocks this. `build-spec` and loading a specification from a file already work.

### 1.3 The specification

- **One new function in `chain_spec.rs`** beside `development_chain_spec` and `local_chain_spec`, taking the
  validators' public keys and the sudo address as plain constants. Public keys only.
- **Built with the `sudo` feature** (ADR-037: development and test networks only).
- **The raw specification is committed**, produced by `demiurge-node build-spec --chain <id> --raw`. Every node
  starts from that one file, so every node has the same genesis. A specification rebuilt from a different
  runtime build gives a different genesis hash, which is why the raw file is committed and not regenerated.
- **Name, id and chain type are proposals, not decisions.** Proposed: name `Demiurge Devnet`, id
  `demiurge_devnet`, chain type `Live`. They appear in `system_chain`, which clients read, so the owner
  approves them (AGENTS.md §8). `chain/scripts/dev-fund.mjs` refuses any chain that is not `Development`, so
  it will correctly refuse this one.

### 1.4 Keys: how they are made, and where each one lives

The rule is `services/qor-auth/DEPLOY-RAILWAY.md`'s: a secret is never pasted into a shell, a chat or a
document.

| Key | Made where | Secret lives | What leaves |
| --- | --- | --- | --- |
| **Sudo key** | By the owner, in a wallet on the owner's own device | With the owner only (ADR-037 decision 2). No service and no assistant ever sees it | The **address**. An address is public and may be sent in chat |
| **Session keys** (Aura Sr25519, GRANDPA Ed25519), per validator | On the validator's own disk, at first boot, by the first-boot script | On that validator's volume only | The two **public keys**, printed to the log |
| **Network key** (libp2p), per node | Same, with `key generate-node-key --file` | On that node's volume only | The **peer id**, printed to the log |

**The first-boot script must write secrets to files and print only public values.** `key generate` prints the
secret phrase to standard output. Sent to a log, that would be a secret in a log. The script redirects it to a
file on the volume and prints the public key by inspecting that file. This is checked in review before
anything is deployed, by reading a first boot's whole log.

**The order this forces.** The specification needs the public keys, and the keys are made on the nodes. So the
nodes boot once in a key-making mode, the public values are copied from the logs into `chain_spec.rs`, the raw
specification is built and committed, and only then do the nodes start the chain.

**In the repository:** the specification function, the raw specification, the Dockerfile, the first-boot
script. All public values.
**Never in the repository, a chat or a shell history:** the sudo recovery phrase, any session key secret, any
network key secret.

**A trade this accepts.** The validators' session keys sit on the hosting provider's disks. A validator has to
hold its session key to sign, so this is true of any hosted validator. ADR-037 already accepts that a devnet
holds nothing of value.

---

## 2. Topology for Alpha

**The minimum that honestly meets "multi-validator": two validators.** That is what `alpha.multi-validator`
proves in CI on the `local` specification.

**One honest limit of two.** GRANDPA needs more than two thirds of the validators. With two, that is both.
When either validator restarts, blocks continue only if the other was started with `--force-authoring`, and
**finality stops until both are back**. `check-two-validators.mjs` shows exactly this. A third validator does
not help: three still needs all three. Four tolerate one being away. This plan stays at two and says so.

### Should a validator serve public RPC?

| | A. Two validators and a separate RPC node (recommended) | B. Two validators, one of them serving RPC |
| --- | --- | --- |
| Machines | 3 | 2 |
| Follows ADR-044 (Proposed) and `HOSTING.md` §4 | Yes | No |
| A flood of RPC requests can stall block production | No | Yes |
| The machine holding a session key answers the internet | No | Yes, with safe methods only |
| Cost | One more node | — |

**Recommendation: A.** It is what `HOSTING.md` §4 already draws, and it means the first public endpoint is not
also a signing machine. B is cheaper by one node and is a fair choice for a network that holds nothing. The
owner picks (§9).

### Ports, flags and names

| Node | Public | Flags that matter |
| --- | --- | --- |
| Validator A | **Nothing.** No hostname, no DNS record | `--validator --chain <raw spec> --base-path /data --node-key-file /data/node-key --force-authoring`. RPC left on its default, localhost |
| Validator B | **Nothing** | As A, plus `--bootnodes` naming A on the private network |
| RPC node | **RPC only**, through the host's TLS edge | `--chain <raw spec> --base-path /data --rpc-external --rpc-methods safe --rpc-port 9944`, a bootnode, and the limits below. **No `--validator`, no session key** |

- **`--rpc-methods safe`** refuses the methods that change a node, such as `author_insertKey` and
  `author_rotateKeys`. `--rpc-external` without it must never be deployed.
- **Rate limiting.** The pinned node has `--rpc-rate-limit <calls per minute, per connection>`,
  `--rpc-max-connections`, `--rpc-max-subscriptions-per-connection` and `--rpc-max-batch-request-len`
  (`sc-cli` 0.61.0, `rpc_params.rs`). **The numbers are not chosen here.** They are set from what the launcher
  actually uses, measured against the live node, and written into the deploy record.
- **CORS. Corrected 3 October 2026 by measurement:** in the pinned `sc-rpc-server` 31.0.0 any `--rpc-cors` value,
  the default included, also turns on a Host-header filter admitting only `localhost`, `127.0.0.1` and `[::1]`, so the
  default answered **403** to a request for `rpc.qorsync.dev`. The RPC node runs with `--rpc-cors all`
  (`DEMIURGE_RPC_CORS=all`), which a browser wallet and polkadot.js Apps also need for the owner's upgrades;
  `--rpc-methods safe` still refuses every method that changes the node. Validators keep the default.
- **TLS.** `.dev` is HSTS-preloaded (`HOSTING.md` §7), so the endpoint is `wss://` and `https://` only. The
  host's edge terminates TLS. No certificate is managed by hand.
- **The name is `rpc.qorsync.dev`**, from `HOSTING.md` §2. Its DNS record is created in Cloudflare when the
  node is ready, not before, set the same way as `id.qorsync.dev`.
- **Peer-to-peer at Alpha is private.** All three nodes are the project's own, so they peer over the host's
  private network. No peer-to-peer port faces the internet. A public bootnode is needed only when outside
  nodes join, which is the public testnet (Beta), not Alpha.
- **The launcher's default is `wss://rpc.demiurge.cloud`** (`tools/qor-launcher/src-tauri/src/chain/mod.rs`).
  That name does not serve the chain. Changing the default to `wss://rpc.qorsync.dev` is a launcher change for
  a later session, once the endpoint answers.

---

## 3. Hosting options

Prices were read on **1 October 2026** from the pages cited. Usage-based figures need the node's memory and
CPU, which **have not been measured on Linux**. The one measurement available: the owner's `--dev` node on
Windows held 135 MB of private memory on 1 October 2026, with one validator and almost no traffic. It is an
indication, not a figure to budget from.

### Does it work for a Substrate node?

| | Railway | Fly.io | Hetzner Cloud (one VPS) |
| --- | --- | --- | --- |
| Persistent disk | Yes. One volume per service; 5 GB on Hobby, 50 GB on Pro, growable to 1 TB on Pro | Yes. Volumes | Yes. 40 GB local disk on the smallest plan |
| Peer-to-peer between our own nodes | Yes. Private network carries TCP and UDP, with `<service>.railway.internal` names | Yes. Private network | Yes. One machine, or a private network |
| Peer-to-peer from outside (Beta, not Alpha) | **Weak.** One TCP proxy per service, Railway picks the public port, no static inbound address. Workable for TCP with an advertised address; untested here | Yes, with a dedicated IPv4 per node | Yes. A real public address |
| Public `wss://` with a certificate | Yes. The edge supports WebSockets, which may stay open indefinitely; certificates are automatic | Yes | **Only if we run a reverse proxy** (for example Caddy) and keep it running |
| HTTP `GET` health check through the same name | Yes | Yes | Yes, through the proxy |
| Stays up with the owner's computer off | Yes | Yes, if auto-stop is off | Yes |
| Already in use by the project | **Yes** (QOR ID, ADR-063) | No. The old `fly.toml` is for a deleted chain | No |

### Monthly cost for topology A (two validators and an RPC node)

| | Railway | Fly.io | Hetzner Cloud |
| --- | --- | --- | --- |
| Unit prices read | RAM $10 per GB; CPU $20 per vCPU; volume $0.15 per GB; egress $0.05 per GB. Hobby plan $5 a month including $5 of usage; Pro $20 including $20 | Volume $0.15 per GB; dedicated IPv4 $2.00; egress $0.02 per GB (North America, Europe); `shared-cpu-1x` with 256 MB $1.94 | CX23 (2 vCPU, 4 GB, 40 GB) €5.49 in Germany and Finland; IPv4 €0.50; CPX11 in the USA €17.49 or $20.49. All excluding VAT |
| What three nodes cost | **Usage-based, so not known until measured.** Illustration only: if each node averages 0.25 to 0.5 GB and 0.05 to 0.1 vCPU, that is $3.50 to $7 a node, **about $11 to $22 a month** plus under $1 of volume. The existing plan's included usage is shared with QOR ID | **Not verified.** The price of a 1 GB machine could not be read: Fly's calculator is interactive. 256 MB is too small to trust for a node. Three 256 MB machines and three 5 GB volumes would be $8.07, a floor and not a plan | **€5.99 a month** for one CX23 in Europe running all three nodes, or **$20.49 plus IPv4** for a CPX11 in the USA |
| Sources | <https://docs.railway.com/reference/pricing/plans>, <https://docs.railway.com/volumes/reference> | <https://fly.io/pricing/>, <https://docs.fly.io/about/pricing/> | <https://docs.hetzner.com/general/infrastructure-and-availability/price-adjustment/>, <https://docs.hetzner.com/cloud/servers/overview/> |

### Operational burden, and what breaks

| | Railway | Fly.io | Hetzner Cloud |
| --- | --- | --- | --- |
| For an owner who travels and is not an operator | **Lowest.** Same dashboard, same secrets rule and same domain steps as QOR ID. No operating system to patch | Medium. A second vendor, a second bill, a command-line tool, and machines that stop by default | **Highest.** A Linux server to patch, an SSH key to guard, a firewall, a reverse proxy and certificates. Nobody restarts it but the owner or an assistant with SSH |
| What breaks | A redeploy of a service with a volume has a short gap, so **finality pauses on every validator redeploy**. A push must not redeploy validators (§6 step 9). An image running as a non-root user cannot write its volume unless `RAILWAY_RUN_UID=0` is set. Hobby's 5 GB volume fills one day | A validator that auto-stops misses its slots. Snapshots and inter-region traffic are billed (ADR-015) | One machine holds both validators, so one fault stops everything. Third-party reports say the cheap CX plans are sometimes unavailable to order; **not verified** on a vendor page |

### Recommendation: Railway

**Reason.** For Alpha the nodes only have to talk to each other and answer one public name. Railway's private
network does the first and its edge does the second, with a certificate and no proxy to run. It is the host the
owner already uses, with the rule for secrets already written down. Hetzner is the cheapest and asks the most
of an owner who is not an operator. Fly is ADR-015's choice and still the better home once outside nodes must
peer, which is a Beta question.

**What ADR-015 said against Railway, and why it does not bite yet.** "Its TCP proxy assigns the public port and
it allows no inbound UDP, which is a poor fit for peer-to-peer nodes that advertise a fixed address." That is
about nodes reachable from outside. At Alpha no outside node joins. **It will bite at the public testnet**, and
moving then means new addresses and a resync, which ADR-015 already calls acceptable for a devnet.

**Not verified for Railway:** whether a release build of the node fits Railway's build limits. This plan avoids
the question by building the image in GitHub Actions and having Railway run the image (§6 step 4).

---

## 4. What the gate needs

**What `kinds.http` does** (`docs/GATES.toml`): an HTTP `GET` to `url`, met when the status equals
`expect_status`. A node's RPC is JSON-RPC over `POST` and WebSocket, so a plain `GET` to the root is not a
useful check.

**The node has health endpoints on the RPC port.** In the pinned `sc-rpc-server` 31.0.0
(`src/middleware/node_health.rs`, the version in `chain/Cargo.lock`):

| Path | Answers 200 when | Otherwise |
| --- | --- | --- |
| `GET /health` | The node answers `system_health` at all. Body: `{"peers":…,"isSyncing":…,"shouldHavePeers":…}` | 500 |
| `GET /health/readiness` | The node is **not syncing and has at least one peer**, or is a chain that should have no peers | 500 |

Both were also checked against the owner's running `--dev` node on 1 October 2026: each answered 200.

**Proposed for the gate:**

```toml
url = "https://rpc.qorsync.dev/health/readiness"
expect_status = 200
```

**Why readiness and not `/health`.** `/health` answers 200 from a node with no peers, which would report a
lone RPC node with both validators dead as live. It is the same trap `alpha.qor-id-live` avoids by naming
`/ready`. Readiness needs a peer, and the RPC node's only peers are the validators.

**What it still does not prove.** That blocks are being finalised, or that both validators are up: one peer is
enough. `alpha.multi-validator` proves agreement and finality in CI, on the `local` specification, not on the
deployed network. If the owner wants the live network's finality measured, that is a new, tighter criterion
and a separate decision. The URL is written into `GATES.toml` only once the endpoint answers, as that file
requires.

---

## 5. Upgrading and resetting

**A runtime upgrade** reaches the devnet the way `HANDOFF.md` records for the development chain:
`sudo.sudoUncheckedWeight(system.setCode(wasm))`, signed by the sudo key. Accounts, assets and blocks are
kept. The node binaries need not be replaced, because the runtime runs as wasm from state.

- **Only the owner can do it**, because only the owner holds the sudo key. It is signed in the owner's wallet,
  through a page such as polkadot.js Apps pointed at `wss://rpc.qorsync.dev`. The assistant builds the wasm and
  says which file to choose; it never signs. **Not verified:** that polkadot.js Apps works against this runtime
  through the public endpoint. `dev-fund.mjs` uses the same library against a local node.
- **The launcher has no sudo path.** Searched on 1 October 2026: nothing in `tools/qor-launcher` mentions sudo.
- **The sudo account needs a balance to sign anything.** `frame-system`'s nonce check refuses a transaction
  from an account that does not exist on chain (`check_nonce.rs` in `frame-system` 48.0.0, the version in
  `chain/Cargo.lock`). So the sudo address needs at least the
  existential deposit, 100 CGT (ADR-036), at genesis. That is part of §7's question.
- **Check `spec_version` afterwards**, by reading it back, as the handoff lesson says.

**A reset** means a new genesis: a new raw specification, every volume wiped, every node restarted.

| What a reset costs | |
| --- | --- |
| Every account's balance, every minted asset, every listing and every royalty term | Gone |
| QOR ID accounts | Unaffected. They are not on the chain |
| Keys | Unaffected. An address is the same on the new chain, and holds nothing |
| Clients | Must reconnect. A transaction signed for the old chain is refused, because the genesis hash is signed |

**The rule proposed:** upgrade in place whenever possible. Reset only when an upgrade cannot work, such as a
storage change with no migration, and say so to testers beforehand. A reset is also the only repair if the sudo
key is lost.

---

## 6. Steps, in order

Nothing below depends on the owner's computer staying on. Steps marked **Owner** involve a secret or an
account only the owner has.

| # | Who | What |
| --- | --- | --- |
| 1 | **Owner** | Answer §9. Nothing starts before questions 1 to 4 |
| 2 | **Owner** | Make the sudo account. Install a wallet extension in the browser, choose "create account", write the recovery phrase on paper, and keep it offline. **Send only the address.** Never the phrase |
| 3 | Assistant | Write the ADR for the host and the topology. Mount the `key` subcommand. Add `chain/Dockerfile` and the first-boot script. Tests for both. `cargo test --workspace` in `chain/`, without `SKIP_WASM_BUILD` |
| 4 | Assistant | Add a workflow, started by hand, that builds the image with `--features sudo` and publishes it to the repository's container registry |
| 5 | **Owner** | In GitHub, open the repository's Actions tab, choose that workflow, press "Run workflow". If the package is private, make it public under the package's settings |
| 6 | **Owner** | In Railway, project `demiurge`: create three services from the image, named for the two validators and the RPC node. Add a volume to each, mounted at `/data`. Set no public domain on any of them yet. The assistant supplies the exact names and variables in a deploy record, as `DEPLOY-RAILWAY.md` does. **Or** the owner asks the assistant to create them through Railway's tools, which this plan did not do |
| 7 | Both | First boot makes each node's keys on its own volume. The assistant reads the logs, confirms that no secret is in them, and copies the public keys and peer ids |
| 8 | Assistant | Write the specification function with those public values and the sudo address. Build the raw specification, commit it, rebuild the image (step 5 again) |
| 9 | **Owner** | Redeploy the three services on the new image. Turn automatic redeploys off for the validators, so a push never restarts them |
| 10 | Assistant | Check over the private network, from the RPC node: both validators author, block hashes agree, finality advances. This is `check-two-validators.mjs`'s list, read from the live network |
| 11 | **Owner** | In Railway, on the RPC service: Settings, Networking, add the custom domain `rpc.qorsync.dev`, target port 9944. In Cloudflare, add the record Railway shows, set as `id.qorsync.dev` is |
| 12 | Assistant | Confirm `https://rpc.qorsync.dev/health/readiness` answers 200, that `system_chain` gives the new name, and that an unsafe method is refused. Set the rate limits from measurement |
| 13 | Assistant | Write the URL into `GATES.toml`, write `chain/DEPLOY-RAILWAY.md`, update `HOSTING.md`, `chain/README.md`, `OWNER.md` and `docs/SYSTEMS.md`. Measure memory, CPU and disk after a week and replace §3's illustration with the bill |
| 14 | **Owner** | Set a usage limit in Railway (already owed, `HANDOFF.md` §4 item 32) |

---

## 7. Risks, and what is deliberately left out

### Open question for the owner: how does anyone hold CGT on this network?

**This plan does not answer it, and nothing may be built that assumes an answer.**

- AGENTS.md §5: "do not add any path that creates CGT outside `--dev`" until issuance is designed. The genesis
  split is OPEN-2 and the issuance rate is OPEN-1.
- On `--dev`, testers are funded from Alice's endowment by `dev-fund.mjs`. That works because Alice's key is
  public, which is exactly what a public network must not have.
- **A network with no CGT in it is live but unusable.** An account needs 100 CGT to exist (ADR-036). A mint
  holds deposits (ADR-052). Even the sudo account cannot sign without a balance (§5).
- So a non-`--dev` specification needs some balance written at genesis, or it serves the gate and nothing else.
  Whether a genesis balance on a test network counts as "creating CGT outside `--dev`", who would hold it, how
  much, and how it would reach testers are **the owner's decisions**. The `local` specification already endows
  well-known accounts with a marked placeholder; whether that precedent extends to a public network is part of
  the same question.
- **What can be said without deciding it:** the gate itself can be met by a network where only the validators
  and the sudo account exist. Usefulness to testers waits on the answer.

### Risks

| Risk | What it means |
| --- | --- |
| Finality pauses whenever one of two validators restarts | Expected with two (§2). It resumes when both are up |
| The sudo key is one key held by one person | ADR-037 accepts this for a network that holds nothing. If it is lost, the only repair is a reset |
| A secret reaches a log at first boot | The script is written so it cannot, and the first log is read before anything else is done (§1.4). If one does, those keys are discarded and remade |
| The disk fills | Blocks are kept for ever by default and growth is unmeasured. Hobby volumes are 5 GB. Needs a usage alert, and a measurement after a week |
| Cost is usage-based and unmeasured | §3's range is an illustration. A usage limit caps it (step 14) |
| All nodes with one provider | Not decentralisation. ADR-015 already says so, and revisits it before mainnet |
| A public endpoint is probed and flooded | Safe methods only, rate limits, and a separate RPC node so validators are unaffected |
| Railway is a poor fit once outside nodes must peer | Known (§3). The move is a Beta decision, costing new addresses and a resync |
| ADR-044 is still Proposed | Topology A follows it. Accepting this plan with A is a reason to accept ADR-044 too |

### Deliberately left out

- **A faucet, issuance, or any genesis allocation.** Above.
- **Transaction fees.** OPEN-4. The chain charges none.
- **An archive node and the indexer.** M5.4 (`HOSTING.md` §1).
- **A public bootnode and outside validators.** The public testnet.
- **Sentry nodes, monitoring dashboards and telemetry.** Nodes run with `--no-telemetry`. A status check beyond
  the gate is not planned here.
- **Backups of chain state.** A devnet's state is rebuilt by resyncing from the other nodes. Losing every
  volume at once is a reset.
- **Changing the launcher's default endpoint.** A later launcher change (§2).

---

## 8. What could not be verified

- Fly.io's price for a machine with 1 GB or more. Only the 256 MB figure was read from a vendor page.
- Hetzner's CPX11 specification, and whether CX23 can be ordered today. The prices are from Hetzner's own
  price-adjustment page; the plan pages render prices with scripts and could not be read.
- The node's memory, CPU and disk use on Linux, and so Railway's real monthly cost.
- Which Railway plan the owner's account is on. No hosting API was called for this plan.
- Whether Railway can build the node itself. The plan builds the image elsewhere.
- That libp2p peers cleanly over Railway's private network. Its documentation says TCP and UDP are carried;
  no node has been run there.
- That polkadot.js Apps can submit the sudo call through the public endpoint.

---

## 9. What the owner decides

1. **Host for the Alpha nodes:** Railway (recommended), Fly.io (ADR-015 as it stands), or a Hetzner server?
   Railway needs a new ADR superseding ADR-015 for the nodes.
2. **Topology:** A, two validators and a separate RPC node (recommended), or B, two validators with one
   serving RPC?
3. **CGT at genesis (§7):** may a public test network's genesis hold any balance at all? If yes, who holds it
   and how does it reach testers? If no, the devnet meets the gate and is not usable for minting until
   issuance is designed. Yes or no, and the rest follows.
4. **Names:** chain name `Demiurge Devnet`, id `demiurge_devnet`, chain type `Live`. Accept, or give others.
5. **The gate URL:** `https://rpc.qorsync.dev/health/readiness`, the stricter of the two. Yes or no.
6. **A live-finality criterion** in the Alpha gate, beyond the HTTP check. Yes or no. It would be a tightening.
7. **ADR-044:** accept it along with topology A? Yes or no.
8. **Who clicks in Railway:** the owner, from a written record, or the assistant through Railway's tools?
9. **The sudo wallet:** a browser wallet extension on the owner's device (proposed), or another arrangement
   the owner prefers. Either way the phrase never leaves the owner.
