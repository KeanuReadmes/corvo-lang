#!/usr/bin/env bash
# coreutils/tests/parity/tail.sh
# Required parity cases for tail.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== tail ==="

run_case tail "default (10 lines)"    "gnu-tail /fixtures/long.txt"                      "corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt"
run_case tail "-n 5"                  "gnu-tail -n 5 /fixtures/long.txt"                 "corvo /corvo/coreutils/tail.corvo -- -n 5 /fixtures/long.txt"
run_case tail "-n 0"                  "gnu-tail -n 0 /fixtures/long.txt"                 "corvo /corvo/coreutils/tail.corvo -- -n 0 /fixtures/long.txt"
run_case tail "-n 50 (exceeds len)"   "gnu-tail -n 50 /fixtures/long.txt"                "corvo /corvo/coreutils/tail.corvo -- -n 50 /fixtures/long.txt"
run_case tail "-n +3 (from line 3)"   "gnu-tail -n +3 /fixtures/long.txt"                "corvo /corvo/coreutils/tail.corvo -- -n +3 /fixtures/long.txt"
run_case tail "-n +1 (all lines)"     "gnu-tail -n +1 /fixtures/long.txt"                "corvo /corvo/coreutils/tail.corvo -- -n +1 /fixtures/long.txt"
run_case tail "-c 20 (bytes)"         "gnu-tail -c 20 /fixtures/a.txt"                   "corvo /corvo/coreutils/tail.corvo -- -c 20 /fixtures/a.txt"
run_case tail "-c +1 (all bytes)"     "gnu-tail -c +1 /fixtures/a.txt"                   "corvo /corvo/coreutils/tail.corvo -- -c +1 /fixtures/a.txt"
run_case tail "two files (headers)"   "gnu-tail /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/tail.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case tail "-q quiet"              "gnu-tail -q /fixtures/a.txt /fixtures/b.txt"      "corvo /corvo/coreutils/tail.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_case tail "-v verbose single"     "gnu-tail -v /fixtures/a.txt"                      "corvo /corvo/coreutils/tail.corvo -- -v /fixtures/a.txt"
run_case tail "missing file"          "gnu-tail /fixtures/no_such_file"                  "corvo /corvo/coreutils/tail.corvo -- /fixtures/no_such_file"

run_uutils_case tail "default"        "uu-tail /fixtures/long.txt"                       "corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt"
run_uutils_case tail "-n 5"           "uu-tail -n 5 /fixtures/long.txt"                  "corvo /corvo/coreutils/tail.corvo -- -n 5 /fixtures/long.txt"

show_time "gnu-tail long.txt"   gnu-tail /fixtures/long.txt
show_time "corvo tail long.txt" corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt
