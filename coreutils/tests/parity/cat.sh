#!/usr/bin/env bash
# coreutils/tests/parity/cat.sh
# Required parity cases for cat.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== cat ==="

run_case cat "plain a.txt"            "gnu-cat /fixtures/a.txt"                          "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt"
run_case cat "plain b.txt"            "gnu-cat /fixtures/b.txt"                          "corvo /corvo/coreutils/cat.corvo -- /fixtures/b.txt"
run_case cat "two files"              "gnu-cat /fixtures/a.txt /fixtures/b.txt"          "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case cat "-n number all"          "gnu-cat -n /fixtures/a.txt"                       "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/a.txt"
run_case cat "-b number-nonblank"     "gnu-cat -b /fixtures/blank.txt"                   "corvo /corvo/coreutils/cat.corvo -- -b /fixtures/blank.txt"
run_case cat "-s squeeze-blank"       "gnu-cat -s /fixtures/blank.txt"                   "corvo /corvo/coreutils/cat.corvo -- -s /fixtures/blank.txt"
run_case cat "-E show-ends"           "gnu-cat -E /fixtures/a.txt"                       "corvo /corvo/coreutils/cat.corvo -- -E /fixtures/a.txt"
run_case cat "-T show-tabs"           "gnu-cat -T /fixtures/tabs.txt"                    "corvo /corvo/coreutils/cat.corvo -- -T /fixtures/tabs.txt"
run_case cat "-A show-all"            "gnu-cat -A /fixtures/tabs.txt"                    "corvo /corvo/coreutils/cat.corvo -- -A /fixtures/tabs.txt"
run_case cat "-n -E combined"         "gnu-cat -n -E /fixtures/a.txt"                    "corvo /corvo/coreutils/cat.corvo -- -n -E /fixtures/a.txt"
run_case cat "-b -E combined"         "gnu-cat -b -E /fixtures/blank.txt"                "corvo /corvo/coreutils/cat.corvo -- -b -E /fixtures/blank.txt"
run_case cat "-n -s combined"         "gnu-cat -n -s /fixtures/blank.txt"                "corvo /corvo/coreutils/cat.corvo -- -n -s /fixtures/blank.txt"
run_case cat "missing file"           "gnu-cat /fixtures/no_such_file"                   "corvo /corvo/coreutils/cat.corvo -- /fixtures/no_such_file"

run_uutils_case cat "plain a.txt"     "uu-cat /fixtures/a.txt"                           "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt"
run_uutils_case cat "-n number"       "uu-cat -n /fixtures/a.txt"                        "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/a.txt"

show_time "gnu-cat long.txt"   gnu-cat /fixtures/long.txt
show_time "corvo cat long.txt" corvo /corvo/coreutils/cat.corvo -- /fixtures/long.txt
