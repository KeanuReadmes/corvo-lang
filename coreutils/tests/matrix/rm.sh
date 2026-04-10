#!/usr/bin/env bash
# coreutils/tests/matrix/rm.sh
# Extended flag-combination matrix for rm.
# Sourced by matrix/run-all.sh; requires run_case and PASS / FAIL in scope.

echo "=== rm ==="

_MTD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$_MTD'" EXIT

# ── Error cases ───────────────────────────────────────────────────────────────
run_case rm "no operand" \
  "gnu-rm" \
  "corvo /corvo/coreutils/rm.corvo"

run_case rm "no operand -f" \
  "gnu-rm -f" \
  "corvo /corvo/coreutils/rm.corvo -- -f"

run_case rm "nonexistent file" \
  "gnu-rm '$_MTD/no_such'" \
  "corvo /corvo/coreutils/rm.corvo -- '$_MTD/no_such'"

run_case rm "nonexistent file -f" \
  "gnu-rm -f '$_MTD/no_such'" \
  "corvo /corvo/coreutils/rm.corvo -- -f '$_MTD/no_such'"

mkdir -p "$_MTD/testdir_gnu" "$_MTD/testdir_corvo"
run_case rm "directory without -r" \
  "gnu-rm '$_MTD/testdir_gnu'" \
  "corvo /corvo/coreutils/rm.corvo -- '$_MTD/testdir_corvo'"

mkdir -p "$_MTD/nonempty_gnu" "$_MTD/nonempty_corvo"
echo "x" > "$_MTD/nonempty_gnu/f.txt"
echo "x" > "$_MTD/nonempty_corvo/f.txt"
run_case rm "non-empty dir with -d" \
  "gnu-rm -d '$_MTD/nonempty_gnu'" \
  "corvo /corvo/coreutils/rm.corvo -- -d '$_MTD/nonempty_corvo'"

# ── Successful removal (filesystem state checks) ──────────────────────────────
echo "x" > "$_MTD/gnu_f1.txt"
echo "x" > "$_MTD/corvo_f1.txt"
gnu-rm "$_MTD/gnu_f1.txt"   >/dev/null 2>&1
corvo /corvo/coreutils/rm.corvo -- "$_MTD/corvo_f1.txt" >/dev/null 2>&1
if [[ ! -e "$_MTD/gnu_f1.txt" ]] && [[ ! -e "$_MTD/corvo_f1.txt" ]]; then
  printf "PASS [rm] single file removed\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] single file not removed\n"; FAIL=$((FAIL+1))
fi

# Multiple files at once
echo "a" > "$_MTD/gnu_m1.txt"; echo "b" > "$_MTD/gnu_m2.txt"
echo "a" > "$_MTD/corvo_m1.txt"; echo "b" > "$_MTD/corvo_m2.txt"
gnu-rm "$_MTD/gnu_m1.txt" "$_MTD/gnu_m2.txt"   >/dev/null 2>&1
corvo /corvo/coreutils/rm.corvo -- "$_MTD/corvo_m1.txt" "$_MTD/corvo_m2.txt" >/dev/null 2>&1
if [[ ! -e "$_MTD/gnu_m1.txt" ]] && [[ ! -e "$_MTD/gnu_m2.txt" ]] && \
   [[ ! -e "$_MTD/corvo_m1.txt" ]] && [[ ! -e "$_MTD/corvo_m2.txt" ]]; then
  printf "PASS [rm] multiple files removed\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] multiple files not removed\n"; FAIL=$((FAIL+1))
fi

# -r recursive
mkdir -p "$_MTD/rec_gnu/sub" "$_MTD/rec_corvo/sub"
echo "a" > "$_MTD/rec_gnu/f.txt"
echo "a" > "$_MTD/rec_corvo/f.txt"
gnu-rm -r "$_MTD/rec_gnu"   >/dev/null 2>&1
corvo /corvo/coreutils/rm.corvo -- -r "$_MTD/rec_corvo" >/dev/null 2>&1
if [[ ! -d "$_MTD/rec_gnu" ]] && [[ ! -d "$_MTD/rec_corvo" ]]; then
  printf "PASS [rm] recursive remove (-r)\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] recursive remove (-r)\n"; FAIL=$((FAIL+1))
fi

# -d empty directory
mkdir -p "$_MTD/ed_gnu" "$_MTD/ed_corvo"
gnu-rm -d "$_MTD/ed_gnu"   >/dev/null 2>&1
corvo /corvo/coreutils/rm.corvo -- -d "$_MTD/ed_corvo" >/dev/null 2>&1
if [[ ! -d "$_MTD/ed_gnu" ]] && [[ ! -d "$_MTD/ed_corvo" ]]; then
  printf "PASS [rm] empty directory with -d\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] empty directory with -d\n"; FAIL=$((FAIL+1))
fi

# Verbose (-v): same path for both so output matches
echo "v" > "$_MTD/vf.txt"
gnu-rm -v "$_MTD/vf.txt" > /tmp/rm_m_gnu_v.out 2>/dev/null || true
echo "v" > "$_MTD/vf.txt"
corvo /corvo/coreutils/rm.corvo -- -v "$_MTD/vf.txt" > /tmp/rm_m_corvo_v.out 2>/dev/null || true
if diff -q /tmp/rm_m_gnu_v.out /tmp/rm_m_corvo_v.out >/dev/null 2>&1; then
  printf "PASS [rm] verbose (-v) output\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] verbose (-v) output\n"
  diff -u /tmp/rm_m_gnu_v.out /tmp/rm_m_corvo_v.out | head -10 || true
  FAIL=$((FAIL+1))
fi

# -rf combined: remove non-empty directory
mkdir -p "$_MTD/rf_gnu/x" "$_MTD/rf_corvo/x"
echo "a" > "$_MTD/rf_gnu/x/f.txt"
echo "a" > "$_MTD/rf_corvo/x/f.txt"
gnu-rm -rf "$_MTD/rf_gnu"   >/dev/null 2>&1
corvo /corvo/coreutils/rm.corvo -- -rf "$_MTD/rf_corvo" >/dev/null 2>&1
if [[ ! -d "$_MTD/rf_gnu" ]] && [[ ! -d "$_MTD/rf_corvo" ]]; then
  printf "PASS [rm] -rf combined\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] -rf combined\n"; FAIL=$((FAIL+1))
fi
