#!/usr/bin/env bash
# coreutils/tests/matrix/base64.sh
# Extended matrix for base64.

echo "=== base64 matrix ==="

echo -n "Hello, World!" > /tmp/b64m_a.txt
printf "SGVsbG8sIFdvcmxkIQ==" > /tmp/b64m_enc.txt
printf "QUJD" > /tmp/b64m_abc.txt

run_case base64 "matrix: encode"      "gnu-base64 /tmp/b64m_a.txt"                         "corvo /corvo/coreutils/base64.corvo -- /tmp/b64m_a.txt"
run_case base64 "matrix: decode"      "gnu-base64 -d /tmp/b64m_enc.txt"                    "corvo /corvo/coreutils/base64.corvo -- -d /tmp/b64m_enc.txt"
run_case base64 "matrix: decode ABC"  "gnu-base64 -d /tmp/b64m_abc.txt"                    "corvo /corvo/coreutils/base64.corvo -- -d /tmp/b64m_abc.txt"
run_case base64 "matrix: -w 0"        "gnu-base64 -w 0 /tmp/b64m_a.txt"                    "corvo /corvo/coreutils/base64.corvo -- -w 0 /tmp/b64m_a.txt"
run_case base64 "matrix: -w 4"        "gnu-base64 -w 4 /tmp/b64m_a.txt"                    "corvo /corvo/coreutils/base64.corvo -- -w 4 /tmp/b64m_a.txt"
run_case base64 "matrix: -w 10"       "gnu-base64 -w 10 /tmp/b64m_a.txt"                   "corvo /corvo/coreutils/base64.corvo -- -w 10 /tmp/b64m_a.txt"
run_case base64 "matrix: missing"     "gnu-base64 /tmp/b64m_no_such"                       "corvo /corvo/coreutils/base64.corvo -- /tmp/b64m_no_such"

rm -f /tmp/b64m_a.txt /tmp/b64m_enc.txt /tmp/b64m_abc.txt
