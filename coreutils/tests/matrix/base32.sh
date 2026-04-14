#!/usr/bin/env bash
# coreutils/tests/matrix/base32.sh
# Extended matrix for base32.

echo "=== base32 matrix ==="

echo -n "Hello, World!" > /tmp/b32m_a.txt
printf "JBSWY3DPFQQFO33SNRSCC===" > /tmp/b32m_enc.txt
printf "IFBA====" > /tmp/b32m_ab.txt

run_case base32 "matrix: encode"      "gnu-base32 /tmp/b32m_a.txt"                         "corvo /corvo/coreutils/base32.corvo -- /tmp/b32m_a.txt"
run_case base32 "matrix: decode"      "gnu-base32 -d /tmp/b32m_enc.txt"                    "corvo /corvo/coreutils/base32.corvo -- -d /tmp/b32m_enc.txt"
run_case base32 "matrix: decode AB"   "gnu-base32 -d /tmp/b32m_ab.txt"                     "corvo /corvo/coreutils/base32.corvo -- -d /tmp/b32m_ab.txt"
run_case base32 "matrix: -w 0"        "gnu-base32 -w 0 /tmp/b32m_a.txt"                    "corvo /corvo/coreutils/base32.corvo -- -w 0 /tmp/b32m_a.txt"
run_case base32 "matrix: -w 8"        "gnu-base32 -w 8 /tmp/b32m_a.txt"                    "corvo /corvo/coreutils/base32.corvo -- -w 8 /tmp/b32m_a.txt"
run_case base32 "matrix: missing"     "gnu-base32 /tmp/b32m_no_such"                       "corvo /corvo/coreutils/base32.corvo -- /tmp/b32m_no_such"

rm -f /tmp/b32m_a.txt /tmp/b32m_enc.txt /tmp/b32m_ab.txt
