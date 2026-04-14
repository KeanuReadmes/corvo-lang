#!/usr/bin/env bash
# coreutils/tests/matrix/wc.sh
# Extended flag-combination matrix for wc.

echo "=== wc matrix ==="
cd /fixtures

# Create test file
echo -e "hello world\ntest line\nthird" > /tmp/wc_matrix.txt

run_case wc "matrix: -l"              "gnu-wc -l /tmp/wc_matrix.txt"                  "corvo /corvo/coreutils/wc.corvo -- -l /tmp/wc_matrix.txt"
run_case wc "matrix: -w"              "gnu-wc -w /tmp/wc_matrix.txt"                  "corvo /corvo/coreutils/wc.corvo -- -w /tmp/wc_matrix.txt"
run_case wc "matrix: -c"              "gnu-wc -c /tmp/wc_matrix.txt"                  "corvo /corvo/coreutils/wc.corvo -- -c /tmp/wc_matrix.txt"
run_case wc "matrix: -lw"             "gnu-wc -lw /tmp/wc_matrix.txt"                 "corvo /corvo/coreutils/wc.corvo -- -l -w /tmp/wc_matrix.txt"
run_case wc "matrix: -wc"             "gnu-wc -wc /tmp/wc_matrix.txt"                 "corvo /corvo/coreutils/wc.corvo -- -w -c /tmp/wc_matrix.txt"
run_case wc "matrix: -lc"             "gnu-wc -lc /tmp/wc_matrix.txt"                 "corvo /corvo/coreutils/wc.corvo -- -l -c /tmp/wc_matrix.txt"
run_case wc "matrix: -lwc"            "gnu-wc -lwc /tmp/wc_matrix.txt"                "corvo /corvo/coreutils/wc.corvo -- -l -w -c /tmp/wc_matrix.txt"

rm -f /tmp/wc_matrix.txt
