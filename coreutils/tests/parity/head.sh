#!/usr/bin/env bash
# coreutils/tests/parity/head.sh
# Required parity cases for head.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== head ==="

run_case head "default (10 lines)"    "gnu-head /fixtures/long.txt"                      "corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt"
run_case head "-n 5"                  "gnu-head -n 5 /fixtures/long.txt"                 "corvo /corvo/coreutils/head.corvo -- -n 5 /fixtures/long.txt"
run_case head "-n 0"                  "gnu-head -n 0 /fixtures/long.txt"                 "corvo /corvo/coreutils/head.corvo -- -n 0 /fixtures/long.txt"
run_case head "-n 50 (exceeds len)"   "gnu-head -n 50 /fixtures/long.txt"                "corvo /corvo/coreutils/head.corvo -- -n 50 /fixtures/long.txt"
run_case head "-n -5 (all-but-last)"  "gnu-head -n -5 /fixtures/long.txt"                "corvo /corvo/coreutils/head.corvo -- -n -5 /fixtures/long.txt"
run_case head "-c 20 (bytes)"         "gnu-head -c 20 /fixtures/a.txt"                   "corvo /corvo/coreutils/head.corvo -- -c 20 /fixtures/a.txt"
run_case head "-c -5 (all-but-last)"  "gnu-head -c -5 /fixtures/a.txt"                   "corvo /corvo/coreutils/head.corvo -- -c -5 /fixtures/a.txt"
run_case head "two files (headers)"   "gnu-head /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/head.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case head "-q quiet"              "gnu-head -q /fixtures/a.txt /fixtures/b.txt"      "corvo /corvo/coreutils/head.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_case head "-v verbose single"     "gnu-head -v /fixtures/a.txt"                      "corvo /corvo/coreutils/head.corvo -- -v /fixtures/a.txt"
run_case head "missing file"          "gnu-head /fixtures/no_such_file"                  "corvo /corvo/coreutils/head.corvo -- /fixtures/no_such_file"

run_uutils_case head "default"        "uu-head /fixtures/long.txt"                       "corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt"
run_uutils_case head "-n 5"           "uu-head -n 5 /fixtures/long.txt"                  "corvo /corvo/coreutils/head.corvo -- -n 5 /fixtures/long.txt"

show_time "gnu-head long.txt"   gnu-head /fixtures/long.txt
show_time "corvo head long.txt" corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt
