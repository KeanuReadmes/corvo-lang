#!/usr/bin/env bash
# coreutils/tests/matrix/tail.sh
# Extended flag-combination matrix for tail.
# Sourced by matrix/run-all.sh; requires run_case and PASS / FAIL in scope.

echo "=== tail ==="

run_case tail "default (10 lines)"                 "gnu-tail /fixtures/long.txt"                              "corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt"
run_case tail "-n 1"                               "gnu-tail -n 1 /fixtures/long.txt"                         "corvo /corvo/coreutils/tail.corvo -- -n 1 /fixtures/long.txt"
run_case tail "-n 5"                               "gnu-tail -n 5 /fixtures/long.txt"                         "corvo /corvo/coreutils/tail.corvo -- -n 5 /fixtures/long.txt"
run_case tail "-n 0"                               "gnu-tail -n 0 /fixtures/long.txt"                         "corvo /corvo/coreutils/tail.corvo -- -n 0 /fixtures/long.txt"
run_case tail "-n 30 (exact)"                      "gnu-tail -n 30 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n 30 /fixtures/long.txt"
run_case tail "-n 50 (exceeds)"                    "gnu-tail -n 50 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n 50 /fixtures/long.txt"
run_case tail "-n +1 (all)"                        "gnu-tail -n +1 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n +1 /fixtures/long.txt"
run_case tail "-n +3 (from line 3)"                "gnu-tail -n +3 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n +3 /fixtures/long.txt"
run_case tail "-n +30 (last only)"                 "gnu-tail -n +30 /fixtures/long.txt"                       "corvo /corvo/coreutils/tail.corvo -- -n +30 /fixtures/long.txt"
run_case tail "-n +100 (past end)"                 "gnu-tail -n +100 /fixtures/long.txt"                      "corvo /corvo/coreutils/tail.corvo -- -n +100 /fixtures/long.txt"
run_case tail "-c 10 (bytes)"                      "gnu-tail -c 10 /fixtures/a.txt"                           "corvo /corvo/coreutils/tail.corvo -- -c 10 /fixtures/a.txt"
run_case tail "-c 100 (exceeds)"                   "gnu-tail -c 100 /fixtures/a.txt"                          "corvo /corvo/coreutils/tail.corvo -- -c 100 /fixtures/a.txt"
run_case tail "-c +1 (all bytes)"                  "gnu-tail -c +1 /fixtures/a.txt"                           "corvo /corvo/coreutils/tail.corvo -- -c +1 /fixtures/a.txt"
run_case tail "-c +5 (from byte 5)"                "gnu-tail -c +5 /fixtures/a.txt"                           "corvo /corvo/coreutils/tail.corvo -- -c +5 /fixtures/a.txt"
run_case tail "two files (auto headers)"           "gnu-tail /fixtures/a.txt /fixtures/b.txt"                 "corvo /corvo/coreutils/tail.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case tail "three files"                        "gnu-tail /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt" "corvo /corvo/coreutils/tail.corvo -- /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt"
run_case tail "-q quiet multi"                     "gnu-tail -q /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/tail.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_case tail "-v verbose single"                  "gnu-tail -v /fixtures/a.txt"                              "corvo /corvo/coreutils/tail.corvo -- -v /fixtures/a.txt"
run_case tail "-v verbose multi"                   "gnu-tail -v /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/tail.corvo -- -v /fixtures/a.txt /fixtures/b.txt"
run_case tail "-n 3 -v"                            "gnu-tail -n 3 -v /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/tail.corvo -- -n 3 -v /fixtures/a.txt /fixtures/b.txt"
run_case tail "-n 3 -q"                            "gnu-tail -n 3 -q /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/tail.corvo -- -n 3 -q /fixtures/a.txt /fixtures/b.txt"
run_case tail "missing file"                       "gnu-tail /fixtures/no_such_file"                          "corvo /corvo/coreutils/tail.corvo -- /fixtures/no_such_file"
