#!/usr/bin/env bash
# coreutils/tests/matrix/b2sum.sh
# Extended matrix for b2sum.

echo "=== b2sum matrix ==="

echo -n "test" > /tmp/b2m_a.txt
echo -n "data" > /tmp/b2m_b.txt

run_case b2sum "matrix: single"       "gnu-b2sum /tmp/b2m_a.txt"                           "corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2m_a.txt"
run_case b2sum "matrix: two files"    "gnu-b2sum /tmp/b2m_a.txt /tmp/b2m_b.txt"            "corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2m_a.txt /tmp/b2m_b.txt"
run_case b2sum "matrix: --tag"        "gnu-b2sum --tag /tmp/b2m_a.txt"                     "corvo /corvo/coreutils/b2sum.corvo -- --tag /tmp/b2m_a.txt"
run_case b2sum "matrix: missing"      "gnu-b2sum /tmp/b2m_no_such"                         "corvo /corvo/coreutils/b2sum.corvo -- /tmp/b2m_no_such"

rm -f /tmp/b2m_a.txt /tmp/b2m_b.txt
