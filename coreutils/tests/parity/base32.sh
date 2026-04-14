#!/usr/bin/env bash
# coreutils/tests/parity/base32.sh
# Required parity cases for base32.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== base32 ==="

echo -n "Hello, World!" > /tmp/base32_a.txt
printf "JBSWY3DPFQQFO33SNRSCC===" > /tmp/base32_encoded.txt

run_case base32 "encode file"         "gnu-base32 /tmp/base32_a.txt"                       "corvo /corvo/coreutils/base32.corvo -- /tmp/base32_a.txt"
run_case base32 "decode file"         "gnu-base32 -d /tmp/base32_encoded.txt"              "corvo /corvo/coreutils/base32.corvo -- -d /tmp/base32_encoded.txt"
run_case base32 "wrap=0 (no wrap)"    "gnu-base32 -w 0 /tmp/base32_a.txt"                  "corvo /corvo/coreutils/base32.corvo -- -w 0 /tmp/base32_a.txt"
run_case base32 "wrap=8"              "gnu-base32 -w 8 /tmp/base32_a.txt"                  "corvo /corvo/coreutils/base32.corvo -- -w 8 /tmp/base32_a.txt"
run_case base32 "missing file"        "gnu-base32 /tmp/base32_no_such"                     "corvo /corvo/coreutils/base32.corvo -- /tmp/base32_no_such"

show_time "gnu-base32"   gnu-base32 /tmp/base32_a.txt
show_time "corvo base32" corvo /corvo/coreutils/base32.corvo -- /tmp/base32_a.txt

# Cleanup
rm -f /tmp/base32_a.txt /tmp/base32_encoded.txt
