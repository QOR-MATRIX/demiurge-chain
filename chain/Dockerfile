# syntax=docker/dockerfile:1
#
# The Demiurge node image, for the hosted devnet (docs/architecture/DEVNET_PLAN.md).
#
# The build context is chain/ itself, not the repository root:
#
#     docker build -f chain/Dockerfile -t demiurge-node chain
#
# chain/ is its own workspace and needs nothing outside it. chain/.dockerignore
# keeps target/ and node_modules/ out of the context. (The repository root's
# Dockerfile and .dockerignore describe the deleted framework/ chain and are not
# used here.)
#
# One image serves both roles, validator and RPC node, chosen at start by
# DEMIURGE_ROLE. chain/docker/entrypoint.sh documents every variable.
#
# Built with the `sudo` feature: development and test networks only (ADR-037).

# ---------- Stage 1: build ----------
# The recipe is CI's (.github/workflows/ci.yml, job two-validators): stable Rust,
# the wasm32v1-none target for the runtime, clang, libclang-dev, llvm-dev and
# protobuf-compiler, then a locked release build with the sudo feature.
FROM rust:1-bookworm AS builder

RUN apt-get -o Acquire::Retries=5 update \
    && apt-get -o Acquire::Retries=5 install -y --no-install-recommends \
        clang libclang-dev llvm-dev protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32v1-none && rustc --version

WORKDIR /build
COPY . .

# Optional: limit parallel jobs where the builder has little memory, for example
# a Docker Desktop VM. Unset, cargo uses every CPU.
ARG CARGO_BUILD_JOBS

# The cache mounts keep the registry and target/ between local builds. They are
# not part of any layer; the binary is copied out of target/ before the step ends.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build -p demiurge-node --release --features sudo --locked \
        ${CARGO_BUILD_JOBS:+--jobs "$CARGO_BUILD_JOBS"} \
    && install -D -m 0755 target/release/demiurge-node /out/demiurge-node

# ---------- Stage 2: run ----------
FROM debian:bookworm-slim

# jq reads the JSON the node's own `key` commands print, so the boot script can
# pick out public values without a secret ever reaching standard output.
# setpriv (util-linux) drops root before the node starts.
RUN apt-get -o Acquire::Retries=5 update \
    && apt-get -o Acquire::Retries=5 install -y --no-install-recommends \
        ca-certificates jq util-linux \
    && rm -rf /var/lib/apt/lists/*

# The node runs as this user, never as root. The container starts as root only
# so the boot script can take ownership of /data: a Railway volume is mounted
# owned by root, so this is what lets the node write it without
# RAILWAY_RUN_UID=0, which would run the node itself as root.
RUN groupadd --system --gid 10001 demiurge \
    && useradd --system --uid 10001 --gid demiurge --home-dir /data --shell /usr/sbin/nologin demiurge

COPY --from=builder /out/demiurge-node /usr/local/bin/demiurge-node
COPY docker/entrypoint.sh /usr/local/bin/demiurge-entrypoint
# The devnet's raw chain specification (ADR-068), built once from the validators'
# first-boot keys and the owner's two addresses and committed. Every node starts
# from this one file, so every node shares one genesis. Railway points
# DEMIURGE_CHAIN_SPEC here; without that variable a node still waits for
# /data/spec.json, as on first boot.
COPY specs/demiurge_devnet.raw.json /etc/demiurge/demiurge_devnet.raw.json
RUN chmod 0755 /usr/local/bin/demiurge-entrypoint \
    && /usr/local/bin/demiurge-node --version

# Every key, the keystore and the chain database live here: a Railway volume.
VOLUME ["/data"]

# 9944: JSON-RPC and the /health endpoints (public on the RPC node only).
# 30333: peer-to-peer, on the private network.
EXPOSE 9944 30333

ENTRYPOINT ["/usr/local/bin/demiurge-entrypoint"]
