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
# shellcheck source=rm.sh
. "$TESTS_DIR/matrix/rm.sh"
# shellcheck source=rmdir.sh
. "$TESTS_DIR/matrix/rmdir.sh"
# shellcheck source=mkdir.sh
. "$TESTS_DIR/matrix/mkdir.sh"
# shellcheck source=wc.sh
. "$TESTS_DIR/matrix/wc.sh"
# shellcheck source=cut.sh
. "$TESTS_DIR/matrix/cut.sh"
# shellcheck source=date.sh
. "$TESTS_DIR/matrix/date.sh"
# shellcheck source=uptime.sh
. "$TESTS_DIR/matrix/uptime.sh"
# shellcheck source=b2sum.sh
. "$TESTS_DIR/matrix/b2sum.sh"
# shellcheck source=base32.sh
. "$TESTS_DIR/matrix/base32.sh"
# shellcheck source=base64.sh
. "$TESTS_DIR/matrix/base64.sh"
# shellcheck source=basenc.sh
. "$TESTS_DIR/matrix/basenc.sh"
# shellcheck source=cksum.sh
. "$TESTS_DIR/matrix/cksum.sh"

echo ""
echo "======================================================"
echo "  coreutils parity matrix:  PASS=$PASS  FAIL=$FAIL"
echo "======================================================"
exit $(( FAIL > 0 ? 1 : 0 ))
