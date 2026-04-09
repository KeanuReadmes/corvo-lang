#!/usr/bin/env bash
# coreutils/tests/run-parity-matrix.sh
# Host orchestrator: build the Docker image, create fixtures, then run the
# extended flag-combination matrix inside the container.
#
# Usage:
#   coreutils/tests/run-parity-matrix.sh
#   coreutils/tests/run-parity-matrix.sh --require-docker
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

REQUIRE_DOCKER=0
for arg in "$@"; do
  [[ "$arg" == "--require-docker" ]] && REQUIRE_DOCKER=1
done

if ! command -v docker >/dev/null 2>&1; then
  if [[ "$REQUIRE_DOCKER" -eq 1 ]]; then
    echo "error: docker is required" >&2
    exit 1
  fi
  echo "skip: docker not installed"
  exit 0
fi

docker build -f "$ROOT/coreutils/tests/Dockerfile" -t corvo-coreutils-parity "$ROOT" >/dev/null

FIX="$(mktemp -d)"
trap 'rm -rf "$FIX"' EXIT

mkdir -p "$FIX/sub/inner" "$FIX/nested/deep"
printf "line1\nline2\nline3\n"       > "$FIX/a.txt"
printf "alpha\nbeta\ngamma\n"        > "$FIX/b.txt"
printf "line1\n\n\nline4\n\nline6\n" > "$FIX/blank.txt"
printf "col1\tcol2\tcol3\n"          > "$FIX/tabs.txt"
i=1; while [ "$i" -le 30 ]; do printf "line%d\n" "$i"; i=$((i+1)); done > "$FIX/long.txt"
printf "hidden\n"                    > "$FIX/.hidden"
printf "backup\n"                    > "$FIX/c.txt~"
printf "inner file\n"                > "$FIX/sub/inner/z.txt"
printf "nested\n"                    > "$FIX/nested/deep/n.txt"
ln -s a.txt   "$FIX/link-a"      2>/dev/null || true
ln -s missing "$FIX/broken-link" 2>/dev/null || true

docker run --rm \
  -e LC_ALL=C \
  -e TZ=UTC \
  -v "$FIX:/fixtures:ro" \
  corvo-coreutils-parity /corvo/tests/matrix/run-all.sh
