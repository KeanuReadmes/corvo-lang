#!/usr/bin/env bash
# coreutils/tests/matrix/uptime.sh
# Extended flag-combination matrix for uptime.
#
# Note: Most uptime output varies with system state. We only test that
# options are accepted and produce consistent exit codes.

echo "=== uptime matrix ==="
cd /fixtures

# Test option handling - verify same exit behavior
run_case uptime "matrix: default"     "gnu-uptime >/dev/null && echo ok"              "corvo /corvo/coreutils/uptime.corvo -- >/dev/null && echo ok"
run_case uptime "matrix: -p"          "gnu-uptime -p >/dev/null && echo ok"           "corvo /corvo/coreutils/uptime.corvo -- -p >/dev/null && echo ok"
run_case uptime "matrix: --pretty"    "gnu-uptime --pretty >/dev/null && echo ok"     "corvo /corvo/coreutils/uptime.corvo -- --pretty >/dev/null && echo ok"
run_case uptime "matrix: -s"          "gnu-uptime -s >/dev/null && echo ok"           "corvo /corvo/coreutils/uptime.corvo -- -s >/dev/null && echo ok"
run_case uptime "matrix: --since"     "gnu-uptime --since >/dev/null && echo ok"      "corvo /corvo/coreutils/uptime.corvo -- --since >/dev/null && echo ok"
