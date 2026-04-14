#!/usr/bin/env bash
# coreutils/tests/parity/b2sum.sh
# Required parity cases for b2sum.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== b2sum ==="

# Create test files
echo -n "hello" > /tmp/b2sum_a.txt
echo -n "world" > /tmp/b2sum_b.txt

run_case b2sum "single file"          "gnu-b2sum /tmp/b2sum_a.txt"                     "corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2sum_a.txt"
run_case b2sum "two files"            "gnu-b2sum /tmp/b2sum_a.txt /tmp/b2sum_b.txt"    "corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2sum_a.txt /tmp/b2sum_b.txt"
run_case b2sum "--tag mode"           "gnu-b2sum --tag /tmp/b2sum_a.txt"               "corvo /corvo/coreutils/b2sum.corvo -- --tag /tmp/b2sum_a.txt"
run_case b2sum "missing file"         "gnu-b2sum /tmp/b2sum_no_such"                   "corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2sum_no_such"

show_time "gnu-b2sum"   gnu-b2sum /tmp/b2sum_a.txt
show_time "corvo b2sum" corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2sum_a.txt

# Cleanup
rm -f /tmp/b2sum_a.txt /tmp/b2sum_b.txt
