#!/usr/bin/env bash
# coreutils/tests/parity/run-all.sh
# Entry point executed inside the Docker container for required parity tests.
# Sources helpers and each per-tool parity script in turn.
set -uo pipefail

TESTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=../helpers.sh
. "$TESTS_DIR/helpers.sh"

PASS=0; FAIL=0

cd /fixtures

# shellcheck source=ls.sh
. "$TESTS_DIR/parity/ls.sh"
# shellcheck source=cat.sh
. "$TESTS_DIR/parity/cat.sh"
# shellcheck source=head.sh
. "$TESTS_DIR/parity/head.sh"
# shellcheck source=tail.sh
. "$TESTS_DIR/parity/tail.sh"
# shellcheck source=cp.sh
. "$TESTS_DIR/parity/cp.sh"

echo ""
echo "=================================================="
echo "  coreutils parity:  PASS=$PASS  FAIL=$FAIL"
echo "=================================================="
exit $(( FAIL > 0 ? 1 : 0 ))
