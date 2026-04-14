# Parity test image: Corvo coreutils alternatives vs GNU coreutils and uutils.
# Mirrors the Ubuntu 26.04+ layout where GNU tools are prefixed gnu-TOOL
# and the uutils Rust reimplementation owns the default plain names.
#
# Build from repo root:
#   docker build -f coreutils/tests/Dockerfile -t corvo-coreutils-parity .

FROM rust:1-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --all-features

# ── Runtime image ─────────────────────────────────────────────────────────────
# Ubuntu 24.04 LTS (Noble) ships both coreutils packages and closely mirrors
# the Ubuntu 26.04+ layout where GNU tools are prefixed gnu-TOOL.
FROM ubuntu:noble

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
       coreutils bash ca-certificates diffutils \
    && rm -rf /var/lib/apt/lists/*

# Save GNU coreutils as gnu-TOOL before uutils may replace the plain names.
# This matches the Ubuntu 26.04+ convention where GNU stays under the gnu- prefix.
RUN for tool in ls cat head tail cp rm rmdir mkdir cut wc date uptime b2sum base32 base64 basenc cksum; do \
      bin="$(command -v "$tool" 2>/dev/null)"; \
      [ -n "$bin" ] && cp "$bin" "/usr/local/bin/gnu-${tool}"; \
    done

# Install uutils-coreutils (Rust reimplementation).  Tolerate absence on images
# where the package is unavailable; tests fall back to GNU-only comparison.
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
       uutils-coreutils \
    && rm -rf /var/lib/apt/lists/* \
    || ( rm -rf /var/lib/apt/lists/* ; \
         echo "INFO: uutils-coreutils not available; uu-TOOL wrappers will be absent" )

# Create uu-TOOL wrappers.  If uutils installed a multicall binary named
# 'coreutils', delegate through it (same as the ubuntu 26.04 expected layout).
# Otherwise symlink whatever TOOL resolves to now.
RUN for tool in ls cat head tail cp rm rmdir mkdir cut wc date; do \
      if coreutils --list 2>/dev/null | grep -qw "$tool"; then \
        printf '#!/bin/sh\nexec coreutils %s "$@"\n' "$tool" \
          > "/usr/local/bin/uu-${tool}" \
        && chmod +x "/usr/local/bin/uu-${tool}"; \
      elif current="$(command -v "$tool" 2>/dev/null)"; [ -n "$current" ]; then \
        ln -sf "$current" "/usr/local/bin/uu-${tool}" 2>/dev/null || true; \
      fi; \
    done || true

COPY --from=builder /build/target/release/corvo /usr/local/bin/corvo
RUN mkdir -p /corvo/coreutils
COPY coreutils/*.corvo /corvo/coreutils/
COPY coreutils/tests/ /corvo/tests/
RUN find /corvo/tests -name "*.sh" -exec chmod +x {} \;

WORKDIR /work
