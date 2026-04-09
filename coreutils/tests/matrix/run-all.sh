#!/usr/bin/env bash
# coreutils/tests/matrix/run-all.sh
# Entry point executed inside the Docker container for the extended matrix.
# Sources helpers and each per-tool matrix script in turn.
set -uo pipefail

TESTS_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=../helpers.sh
. "$TESTS_DIR/helpers.sh"

PASS=0; FAIL=0

printf "%-8s %-50s %s\n" "SECTION" "CASE" "RESULT"
printf "%-8s %-50s %s\n" "--------" "--------------------------------------------------" "--------"

cd /fixtures

# shellcheck source=ls.sh
. "$TESTS_DIR/matrix/ls.sh"
# shellcheck source=cat.sh
. "$TESTS_DIR/matrix/cat.sh"
# shellcheck source=head.sh
. "$TESTS_DIR/matrix/head.sh"
# shellcheck source=tail.sh
. "$TESTS_DIR/matrix/tail.sh"
# shellcheck source=cp.sh
. "$TESTS_DIR/matrix/cp.sh"

echo ""
echo "======================================================"
echo "  coreutils parity matrix:  PASS=$PASS  FAIL=$FAIL"
echo "======================================================"
exit $(( FAIL > 0 ? 1 : 0 ))
