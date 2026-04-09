#!/usr/bin/env bash
# coreutils/tests/matrix/head.sh
# Extended flag-combination matrix for head.
# Sourced by matrix/run-all.sh; requires run_case and PASS / FAIL in scope.

echo "=== head ==="

run_case head "default (10 lines)"                 "gnu-head /fixtures/long.txt"                              "corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt"
run_case head "-n 1"                               "gnu-head -n 1 /fixtures/long.txt"                         "corvo /corvo/coreutils/head.corvo -- -n 1 /fixtures/long.txt"
run_case head "-n 5"                               "gnu-head -n 5 /fixtures/long.txt"                         "corvo /corvo/coreutils/head.corvo -- -n 5 /fixtures/long.txt"
run_case head "-n 10"                              "gnu-head -n 10 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n 10 /fixtures/long.txt"
run_case head "-n 0"                               "gnu-head -n 0 /fixtures/long.txt"                         "corvo /corvo/coreutils/head.corvo -- -n 0 /fixtures/long.txt"
run_case head "-n 30 (exact)"                      "gnu-head -n 30 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n 30 /fixtures/long.txt"
run_case head "-n 50 (exceeds)"                    "gnu-head -n 50 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n 50 /fixtures/long.txt"
run_case head "-n -1 (all-but-last-1)"             "gnu-head -n -1 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n -1 /fixtures/long.txt"
run_case head "-n -5 (all-but-last-5)"             "gnu-head -n -5 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n -5 /fixtures/long.txt"
run_case head "-n -30 (none)"                      "gnu-head -n -30 /fixtures/long.txt"                       "corvo /corvo/coreutils/head.corvo -- -n -30 /fixtures/long.txt"
run_case head "-c 1"                               "gnu-head -c 1 /fixtures/a.txt"                            "corvo /corvo/coreutils/head.corvo -- -c 1 /fixtures/a.txt"
run_case head "-c 10"                              "gnu-head -c 10 /fixtures/a.txt"                           "corvo /corvo/coreutils/head.corvo -- -c 10 /fixtures/a.txt"
run_case head "-c 100"                             "gnu-head -c 100 /fixtures/a.txt"                          "corvo /corvo/coreutils/head.corvo -- -c 100 /fixtures/a.txt"
run_case head "-c -5 (all-but-last-5-bytes)"       "gnu-head -c -5 /fixtures/a.txt"                           "corvo /corvo/coreutils/head.corvo -- -c -5 /fixtures/a.txt"
run_case head "two files (auto headers)"           "gnu-head /fixtures/a.txt /fixtures/b.txt"                 "corvo /corvo/coreutils/head.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case head "three files"                        "gnu-head /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt" "corvo /corvo/coreutils/head.corvo -- /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt"
run_case head "-q quiet multi"                     "gnu-head -q /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/head.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_case head "-v verbose single"                  "gnu-head -v /fixtures/a.txt"                              "corvo /corvo/coreutils/head.corvo -- -v /fixtures/a.txt"
run_case head "-v verbose multi"                   "gnu-head -v /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/head.corvo -- -v /fixtures/a.txt /fixtures/b.txt"
run_case head "-n 3 -v"                            "gnu-head -n 3 -v /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/head.corvo -- -n 3 -v /fixtures/a.txt /fixtures/b.txt"
run_case head "-n 3 -q"                            "gnu-head -n 3 -q /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/head.corvo -- -n 3 -q /fixtures/a.txt /fixtures/b.txt"
run_case head "missing file"                       "gnu-head /fixtures/no_such_file"                          "corvo /corvo/coreutils/head.corvo -- /fixtures/no_such_file"
