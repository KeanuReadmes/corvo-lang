#!/usr/bin/env bash
# coreutils/tests/matrix/cat.sh
# Extended flag-combination matrix for cat.
# Sourced by matrix/run-all.sh; requires run_case and PASS / FAIL in scope.

echo "=== cat ==="

run_case cat "plain single file"                   "gnu-cat /fixtures/a.txt"                                  "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt"
run_case cat "plain two files"                     "gnu-cat /fixtures/a.txt /fixtures/b.txt"                  "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case cat "three files"                         "gnu-cat /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt" "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt"
run_case cat "-n number all lines"                 "gnu-cat -n /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/a.txt"
run_case cat "-n with blank lines"                 "gnu-cat -n /fixtures/blank.txt"                           "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/blank.txt"
run_case cat "-b number non-blank"                 "gnu-cat -b /fixtures/blank.txt"                           "corvo /corvo/coreutils/cat.corvo -- -b /fixtures/blank.txt"
run_case cat "-b overrides -n"                     "gnu-cat -n -b /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -n -b /fixtures/blank.txt"
run_case cat "-s squeeze blank"                    "gnu-cat -s /fixtures/blank.txt"                           "corvo /corvo/coreutils/cat.corvo -- -s /fixtures/blank.txt"
run_case cat "-E show ends"                        "gnu-cat -E /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -E /fixtures/a.txt"
run_case cat "-T show tabs"                        "gnu-cat -T /fixtures/tabs.txt"                            "corvo /corvo/coreutils/cat.corvo -- -T /fixtures/tabs.txt"
run_case cat "-A show-all"                         "gnu-cat -A /fixtures/tabs.txt"                            "corvo /corvo/coreutils/cat.corvo -- -A /fixtures/tabs.txt"
run_case cat "-e equiv to -vE"                     "gnu-cat -e /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -e /fixtures/a.txt"
run_case cat "-t equiv to -vT"                     "gnu-cat -t /fixtures/tabs.txt"                            "corvo /corvo/coreutils/cat.corvo -- -t /fixtures/tabs.txt"
run_case cat "-n -E combined"                      "gnu-cat -n -E /fixtures/a.txt"                            "corvo /corvo/coreutils/cat.corvo -- -n -E /fixtures/a.txt"
run_case cat "-b -E combined"                      "gnu-cat -b -E /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -b -E /fixtures/blank.txt"
run_case cat "-n -s combined"                      "gnu-cat -n -s /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -n -s /fixtures/blank.txt"
run_case cat "-b -s combined"                      "gnu-cat -b -s /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -b -s /fixtures/blank.txt"
run_case cat "-s -E combined"                      "gnu-cat -s -E /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -s -E /fixtures/blank.txt"
run_case cat "-n -s -E all"                        "gnu-cat -n -s -E /fixtures/blank.txt"                     "corvo /corvo/coreutils/cat.corvo -- -n -s -E /fixtures/blank.txt"
run_case cat "-u (unbuffered, ignored)"            "gnu-cat -u /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -u /fixtures/a.txt"
run_case cat "missing file"                        "gnu-cat /fixtures/no_such_file"                           "corvo /corvo/coreutils/cat.corvo -- /fixtures/no_such_file"
run_case cat "missing among valid"                 "gnu-cat /fixtures/a.txt /fixtures/no_such_file /fixtures/b.txt" "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/no_such_file /fixtures/b.txt"
