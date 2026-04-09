#!/usr/bin/env bash
# coreutils/tests/parity/ls.sh
# Required parity cases for ls.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== ls ==="
cd /fixtures

run_case ls "one column"             "gnu-ls -1"                                        "corvo /corvo/coreutils/ls.corvo -- -1"
run_case ls "almost-all (-A)"        "gnu-ls -1 -A"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -A"
run_case ls "all (-a)"               "gnu-ls -1 -a"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -a"
run_case ls "long-iso"               "gnu-ls -l --time-style=long-iso"                  "corvo /corvo/coreutils/ls.corvo -- -l --time-style=long-iso"
run_case ls "long-iso -A"            "gnu-ls -1 -A -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -A -l --time-style=long-iso"
run_case ls "long-iso -a"            "gnu-ls -1 -a -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -a -l --time-style=long-iso"
run_case ls "inode short"            "gnu-ls -1 -i"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -i"
run_case ls "inode long"             "gnu-ls -1 -l --time-style=long-iso -i"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -i"
run_case ls "reverse"                "gnu-ls -1 -r"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -r"
run_case ls "reverse long"           "gnu-ls -1 -l --time-style=long-iso -r"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -r"
run_case ls "classify (-F)"          "gnu-ls -1 -F"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -F"
run_case ls "blocks (-s)"            "gnu-ls -1 -s"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -s"
run_case ls "blocks long"            "gnu-ls -1 -s -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -s -l --time-style=long-iso"
run_case ls "human-readable (-h)"    "gnu-ls -1 -h -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -h -l --time-style=long-iso"
run_case ls "numeric-ids (-n)"       "gnu-ls -1 -l --time-style=long-iso -n"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -n"
run_case ls "no-group (-G)"          "gnu-ls -1 -l --time-style=long-iso -G"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -G"
run_case ls "comma (-m)"             "gnu-ls -m"                                        "corvo /corvo/coreutils/ls.corvo -- -m"
run_case ls "sort-size (-S)"         "gnu-ls -1 -l --time-style=long-iso -S"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -S"
run_case ls "sort-time (-t)"         "gnu-ls -1 -l --time-style=long-iso -t"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -t"
run_case ls "sort-none (-U)"         "gnu-ls -U -1"                                     "corvo /corvo/coreutils/ls.corvo -- -U -1"
run_case ls "sort-version (-v)"      "gnu-ls -1 -v"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -v"
run_case ls "sort-ext (-X)"          "gnu-ls -1 -X"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -X"
run_case ls "recursive (-R)"         "gnu-ls -R"                                        "corvo /corvo/coreutils/ls.corvo -- -R"
run_case ls "directory (-d)"         "gnu-ls -d /fixtures"                              "corvo /corvo/coreutils/ls.corvo -- -d /fixtures"
run_case ls "color=never long"       "gnu-ls -1 --color=never -l --time-style=long-iso" "corvo /corvo/coreutils/ls.corvo -- -1 --color=never -l --time-style=long-iso"
run_case ls "ignore (*.txt)"         "gnu-ls -1 -I '*.txt'"                             "corvo /corvo/coreutils/ls.corvo -- -1 -I '*.txt'"
run_case ls "hide backup (~)"        "gnu-ls -1 --hide='*~'"                            "corvo /corvo/coreutils/ls.corvo -- -1 --hide='*~'"
run_case ls "literal (-N)"           "gnu-ls -1 -N"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -N"
run_case ls "time-access (-u)"       "gnu-ls -1 -l --time-style=long-iso -u"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -u"
run_case ls "time-status (-c)"       "gnu-ls -1 -l --time-style=long-iso -c"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -c"
run_case ls "hyperlink=no"           "gnu-ls -1 --hyperlink=no"                         "corvo /corvo/coreutils/ls.corvo -- -1 --hyperlink=no"

run_uutils_case ls "one column"      "uu-ls -1"                                         "corvo /corvo/coreutils/ls.corvo -- -1"
run_uutils_case ls "long-iso"        "uu-ls -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -l --time-style=long-iso"

show_time "gnu-ls -la"  gnu-ls -la /fixtures
show_time "corvo ls -la" corvo /corvo/coreutils/ls.corvo -- -la /fixtures
