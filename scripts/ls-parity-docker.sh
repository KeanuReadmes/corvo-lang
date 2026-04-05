#!/usr/bin/env bash
# Compare GNU coreutils `ls` with `corvo coreutils/ls.corvo` inside Docker (Linux only).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REQUIRE_DOCKER=0
for arg in "$@"; do
  if [[ "$arg" == "--require-docker" ]]; then
    REQUIRE_DOCKER=1
  fi
done

if ! command -v docker >/dev/null 2>&1; then
  if [[ "$REQUIRE_DOCKER" -eq 1 ]]; then
    echo "error: docker is required for ls parity (pass without --require-docker to skip locally)" >&2
    exit 1
  fi
  echo "skip: docker not installed"
  exit 0
fi

docker build -f "$ROOT/coreutils/tests/ls-parity/Dockerfile" -t corvo-ls-parity "$ROOT"

FIX="$(mktemp -d)"
trap 'rm -rf "$FIX"' EXIT

mkdir -p "$FIX/sub/inner"
echo "hello" >"$FIX/a.txt"
echo "beta" >"$FIX/b.txt"
ln -s a.txt "$FIX/link-a" 2>/dev/null || true

docker run --rm \
  -e LC_ALL=C \
  -e TZ=UTC \
  -v "$FIX:/fixtures:ro" \
  corvo-ls-parity bash -ce '
    set -euo pipefail
    cd /fixtures
    run_case() {
      local name="$1"
      shift
      ls "$@" > /tmp/gnu.out
      corvo /corvo/coreutils/ls.corvo -- "$@" > /tmp/corvo.out
      if ! diff -u /tmp/gnu.out /tmp/corvo.out; then
        echo "ls-parity FAIL: $name" >&2
        exit 1
      fi
    }
    run_case "default" -1
    run_case "almost-all" -1 -A
    run_case "long-iso" -1 -l --time-style=long-iso
    run_case "inode" -1 -i
    run_case "reverse" -1 -r
    echo "ls-parity: all cases OK"
  '
