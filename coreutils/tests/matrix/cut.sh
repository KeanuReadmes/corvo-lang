#!/usr/bin/env bash
# coreutils/tests/matrix/cut.sh
# Extended flag-combination matrix for cut.

echo "=== cut matrix ==="
cd /fixtures

# Create test files
echo -e "a:b:c:d:e\n1:2:3:4:5" > /tmp/cut_matrix.txt

run_case cut "matrix: -f1"            "gnu-cut -d: -f1 /tmp/cut_matrix.txt"           "corvo /corvo/coreutils/cut.corvo -- -d: -f1 /tmp/cut_matrix.txt"
run_case cut "matrix: -f3"            "gnu-cut -d: -f3 /tmp/cut_matrix.txt"           "corvo /corvo/coreutils/cut.corvo -- -d: -f3 /tmp/cut_matrix.txt"
run_case cut "matrix: -f5"            "gnu-cut -d: -f5 /tmp/cut_matrix.txt"           "corvo /corvo/coreutils/cut.corvo -- -d: -f5 /tmp/cut_matrix.txt"
run_case cut "matrix: -f1,3,5"        "gnu-cut -d: -f1,3,5 /tmp/cut_matrix.txt"       "corvo /corvo/coreutils/cut.corvo -- -d: -f1,3,5 /tmp/cut_matrix.txt"
run_case cut "matrix: -f1-3"          "gnu-cut -d: -f1-3 /tmp/cut_matrix.txt"         "corvo /corvo/coreutils/cut.corvo -- -d: -f1-3 /tmp/cut_matrix.txt"
run_case cut "matrix: -f3-"           "gnu-cut -d: -f3- /tmp/cut_matrix.txt"          "corvo /corvo/coreutils/cut.corvo -- -d: -f3- /tmp/cut_matrix.txt"
run_case cut "matrix: -f-3"           "gnu-cut -d: -f-3 /tmp/cut_matrix.txt"          "corvo /corvo/coreutils/cut.corvo -- -d: -f-3 /tmp/cut_matrix.txt"

rm -f /tmp/cut_matrix.txt
