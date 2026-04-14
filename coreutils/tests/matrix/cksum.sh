#!/usr/bin/env bash
# coreutils/tests/matrix/cksum.sh
# Extended matrix for cksum.

echo "=== cksum matrix ==="

echo "hello world" > /tmp/cksm_a.txt
echo "test" > /tmp/cksm_b.txt
echo -n "no newline" > /tmp/cksm_c.txt

run_case cksum "matrix: single"       "gnu-cksum /tmp/cksm_a.txt"                          "corvo /corvo/coreutils/cksum.corvo -- /tmp/cksm_a.txt"
run_case cksum "matrix: two files"    "gnu-cksum /tmp/cksm_a.txt /tmp/cksm_b.txt"          "corvo /corvo/coreutils/cksum.corvo -- /tmp/cksm_a.txt /tmp/cksm_b.txt"
run_case cksum "matrix: no newline"   "gnu-cksum /tmp/cksm_c.txt"                          "corvo /corvo/coreutils/cksum.corvo -- /tmp/cksm_c.txt"
run_case cksum "matrix: missing"      "gnu-cksum /tmp/cksm_no_such"                        "corvo /corvo/coreutils/cksum.corvo -- /tmp/cksm_no_such"

rm -f /tmp/cksm_a.txt /tmp/cksm_b.txt /tmp/cksm_c.txt
