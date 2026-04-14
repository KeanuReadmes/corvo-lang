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
# shellcheck source=rm.sh
. "$TESTS_DIR/parity/rm.sh"
# shellcheck source=rmdir.sh
. "$TESTS_DIR/parity/rmdir.sh"
# shellcheck source=mkdir.sh
. "$TESTS_DIR/parity/mkdir.sh"
# shellcheck source=wc.sh
. "$TESTS_DIR/parity/wc.sh"
# shellcheck source=cut.sh
. "$TESTS_DIR/parity/cut.sh"
# shellcheck source=date.sh
. "$TESTS_DIR/parity/date.sh"
# shellcheck source=uptime.sh
. "$TESTS_DIR/parity/uptime.sh"
# shellcheck source=b2sum.sh
. "$TESTS_DIR/parity/b2sum.sh"
# shellcheck source=base32.sh
. "$TESTS_DIR/parity/base32.sh"
# shellcheck source=base64.sh
. "$TESTS_DIR/parity/base64.sh"
# shellcheck source=basenc.sh
. "$TESTS_DIR/parity/basenc.sh"
# shellcheck source=cksum.sh
. "$TESTS_DIR/parity/cksum.sh"

echo ""
echo "=================================================="
echo "  coreutils parity:  PASS=$PASS  FAIL=$FAIL"
echo "=================================================="
exit $(( FAIL > 0 ? 1 : 0 ))
