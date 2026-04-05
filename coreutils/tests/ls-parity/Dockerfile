# Build Corvo in bookworm-compatible glibc, then run GNU ls parity checks.
# From repo root: `docker build -f coreutils/tests/ls-parity/Dockerfile -t corvo-ls-parity .`

FROM rust:1-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --all-features

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends coreutils bash ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /corvo/coreutils

COPY --from=builder /build/target/release/corvo /usr/local/bin/corvo
COPY coreutils/ls.corvo /corvo/coreutils/ls.corvo

WORKDIR /work
