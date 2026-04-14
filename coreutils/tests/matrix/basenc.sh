#!/usr/bin/env bash
# coreutils/tests/matrix/basenc.sh
# Extended matrix for basenc.

echo "=== basenc matrix ==="

echo -n "Hello!" > /tmp/bncm_a.txt
printf "SGVsbG8h" > /tmp/bncm_b64.txt
printf "JBSWY3BPEB3W64TMMQ======" > /tmp/bncm_b32.txt

run_case basenc "matrix: --base64 enc"      "gnu-basenc --base64 /tmp/bncm_a.txt"                "corvo /corvo/coreutils/basenc.corvo -- --base64 /tmp/bncm_a.txt"
run_case basenc "matrix: --base64 dec"      "gnu-basenc --base64 -d /tmp/bncm_b64.txt"            "corvo /corvo/coreutils/basenc.corvo -- --base64 -d /tmp/bncm_b64.txt"
run_case basenc "matrix: --base32 enc"      "gnu-basenc --base32 /tmp/bncm_a.txt"                "corvo /corvo/coreutils/basenc.corvo -- --base32 /tmp/bncm_a.txt"
run_case basenc "matrix: --base32 dec"      "gnu-basenc --base32 -d /tmp/bncm_b32.txt"            "corvo /corvo/coreutils/basenc.corvo -- --base32 -d /tmp/bncm_b32.txt"
run_case basenc "matrix: --base16 enc"      "gnu-basenc --base16 /tmp/bncm_a.txt"                "corvo /corvo/coreutils/basenc.corvo -- --base16 /tmp/bncm_a.txt"
run_case basenc "matrix: --base64 -w 0"     "gnu-basenc --base64 -w 0 /tmp/bncm_a.txt"           "corvo /corvo/coreutils/basenc.corvo -- --base64 -w 0 /tmp/bncm_a.txt"
run_case basenc "matrix: missing file"      "gnu-basenc --base64 /tmp/bncm_no_such"              "corvo /corvo/coreutils/basenc.corvo -- --base64 /tmp/bncm_no_such"

rm -f /tmp/bncm_a.txt /tmp/bncm_b64.txt /tmp/bncm_b32.txt
