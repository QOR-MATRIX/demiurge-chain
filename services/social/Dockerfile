# Social, for a container platform (ADR-091, roadmap L8.6). Devnet stack only.
#
# Build context is `services/social`, not the repository root:
#
#   docker build -t social services/social
#
# DATABASE_URL is not needed to build: every statement is checked at run time (no `query!` macros). The migrations
# are read at build time by `sqlx::migrate!` and embedded in the binary, so the directory must be in the context; the
# runtime image does not need it.

# --- build -------------------------------------------------------------------
FROM rust:1-bookworm AS build
WORKDIR /src

# TLS is rustls (reqwest without default features), so no OpenSSL.
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config \
 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release --locked

# --- run ---------------------------------------------------------------------
FROM debian:bookworm-slim AS run

# ca-certificates, so Social can reach QOR ID's introspection endpoint over TLS.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --create-home --uid 10001 social

COPY --from=build /src/target/release/social /usr/local/bin/social
USER social

# The platform assigns the port and passes PORT; Social reads it (8090 otherwise).
EXPOSE 8090

# /ready asks the database; /health answers 200 whatever its state, so a health check never points at it.
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD curl -fsS "http://127.0.0.1:${PORT:-8090}/ready" || exit 1

# Migrations run at startup, from inside the binary.
CMD ["social"]
