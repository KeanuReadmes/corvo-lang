#!/usr/bin/env bash
# coreutils/tests/matrix/cp.sh
# Extended flag-combination matrix for cp.
# Sourced by matrix/run-all.sh; requires run_case and PASS / FAIL in scope.

echo "=== cp ==="

_MTD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$_MTD'" EXIT

# Verbose: identical destination so printed path matches
run_case cp "verbose (-v) single"                  \
  "gnu-cp -v /fixtures/a.txt '$_MTD/cp_v.txt'"     \
  "corvo /corvo/coreutils/cp.corvo -- -v /fixtures/a.txt '$_MTD/cp_v.txt'"

# Error cases
run_case cp "missing source"                       \
  "gnu-cp /fixtures/no_such_file /tmp/x_gnu"       \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/no_such_file /tmp/x_corvo"

run_case cp "missing operand"                      \
  "gnu-cp"                                          \
  "corvo /corvo/coreutils/cp.corvo"

run_case cp "dir without -r"                       \
  "gnu-cp /fixtures/sub /tmp/xd_gnu"                \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/sub /tmp/xd_corvo"

run_case cp "multi-source to file (error)"         \
  "gnu-cp /fixtures/a.txt /fixtures/b.txt '$_MTD/not_a_dir.txt'" \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt /fixtures/b.txt '$_MTD/not_a_dir.txt'"

# Content comparisons
rm -rf "$_MTD/gnu_single" "$_MTD/corvo_single"
mkdir -p "$_MTD/gnu_single" "$_MTD/corvo_single"
gnu-cp /fixtures/a.txt "$_MTD/gnu_single/"   >/dev/null 2>&1
corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt "$_MTD/corvo_single/" >/dev/null 2>&1
if diff -r "$_MTD/gnu_single" "$_MTD/corvo_single" >/dev/null 2>&1; then
  printf "PASS [cp] basic file copy content\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] basic file copy content\n"; FAIL=$((FAIL+1))
fi

rm -rf "$_MTD/gnu_multi" "$_MTD/corvo_multi"
mkdir -p "$_MTD/gnu_multi" "$_MTD/corvo_multi"
gnu-cp /fixtures/a.txt /fixtures/b.txt "$_MTD/gnu_multi/"   >/dev/null 2>&1
corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt /fixtures/b.txt "$_MTD/corvo_multi/" >/dev/null 2>&1
if diff -r "$_MTD/gnu_multi" "$_MTD/corvo_multi" >/dev/null 2>&1; then
  printf "PASS [cp] two-file copy content\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] two-file copy content\n"; FAIL=$((FAIL+1))
fi

rm -rf "$_MTD/gnu_rec" "$_MTD/corvo_rec"
gnu-cp -r /fixtures/sub "$_MTD/gnu_rec"   >/dev/null 2>&1
corvo /corvo/coreutils/cp.corvo -- -r /fixtures/sub "$_MTD/corvo_rec" >/dev/null 2>&1
if diff -r "$_MTD/gnu_rec" "$_MTD/corvo_rec" >/dev/null 2>&1; then
  printf "PASS [cp] recursive copy content\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] recursive copy content\n"; FAIL=$((FAIL+1))
fi

# -n (no-clobber): second run should NOT overwrite
echo "original" > "$_MTD/noclobber.txt"
gnu-cp -n /fixtures/a.txt "$_MTD/noclobber.txt" >/dev/null 2>&1 || true
if diff <(echo "original") "$_MTD/noclobber.txt" >/dev/null 2>&1; then
  printf "PASS [cp] no-clobber (-n) gnu behaviour\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] no-clobber (-n) gnu behaviour\n"; FAIL=$((FAIL+1))
fi
echo "original" > "$_MTD/noclobber2.txt"
corvo /corvo/coreutils/cp.corvo -- -n /fixtures/a.txt "$_MTD/noclobber2.txt" >/dev/null 2>&1 || true
if diff <(echo "original") "$_MTD/noclobber2.txt" >/dev/null 2>&1; then
  printf "PASS [cp] no-clobber (-n) corvo behaviour\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] no-clobber (-n) corvo behaviour\n"; FAIL=$((FAIL+1))
fi
