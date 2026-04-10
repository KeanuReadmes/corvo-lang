#!/usr/bin/env bash
# coreutils/tests/parity/rm.sh
# Required parity cases for rm.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== rm ==="

_TD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$_TD'" EXIT

# ── No operands ───────────────────────────────────────────────────────────────
run_case rm "no operand" \
  "gnu-rm" \
  "corvo /corvo/coreutils/rm.corvo"

run_case rm "no operand with -f" \
  "gnu-rm -f" \
  "corvo /corvo/coreutils/rm.corvo -- -f"

# ── Non-existent paths ────────────────────────────────────────────────────────
run_case rm "nonexistent file" \
  "gnu-rm '$_TD/no_such_file'" \
  "corvo /corvo/coreutils/rm.corvo -- '$_TD/no_such_file'"

run_case rm "nonexistent file with -f" \
  "gnu-rm -f '$_TD/no_such_file'" \
  "corvo /corvo/coreutils/rm.corvo -- -f '$_TD/no_such_file'"

# ── Remove a regular file ─────────────────────────────────────────────────────
echo "hello" > "$_TD/gnu_f.txt"
echo "hello" > "$_TD/corvo_f.txt"
_gnu_rm_ec=0; _corvo_rm_ec=0
gnu-rm "$_TD/gnu_f.txt"   >/dev/null 2>&1 || _gnu_rm_ec=$?
corvo /corvo/coreutils/rm.corvo -- "$_TD/corvo_f.txt" >/dev/null 2>&1 || _corvo_rm_ec=$?
if [[ "$_gnu_rm_ec" == "$_corvo_rm_ec" ]] && \
   [[ ! -e "$_TD/gnu_f.txt" ]] && [[ ! -e "$_TD/corvo_f.txt" ]]; then
  printf "PASS [rm] remove regular file\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] remove regular file  exit: gnu=%s corvo=%s\n" \
    "$_gnu_rm_ec" "$_corvo_rm_ec"
  FAIL=$((FAIL+1))
fi

# ── Directory without -r ──────────────────────────────────────────────────────
mkdir -p "$_TD/testdir_gnu" "$_TD/testdir_corvo"
run_case rm "directory without -r" \
  "gnu-rm '$_TD/testdir_gnu'" \
  "corvo /corvo/coreutils/rm.corvo -- '$_TD/testdir_corvo'"

# ── Recursive removal (-r) ────────────────────────────────────────────────────
mkdir -p "$_TD/rmr_gnu/sub" "$_TD/rmr_corvo/sub"
echo "a" > "$_TD/rmr_gnu/f.txt"
echo "a" > "$_TD/rmr_corvo/f.txt"
_gnu_rec_ec=0; _corvo_rec_ec=0
gnu-rm -r "$_TD/rmr_gnu"   >/dev/null 2>&1 || _gnu_rec_ec=$?
corvo /corvo/coreutils/rm.corvo -- -r "$_TD/rmr_corvo" >/dev/null 2>&1 || _corvo_rec_ec=$?
if [[ "$_gnu_rec_ec" == "$_corvo_rec_ec" ]] && \
   [[ ! -d "$_TD/rmr_gnu" ]] && [[ ! -d "$_TD/rmr_corvo" ]]; then
  printf "PASS [rm] recursive remove (-r)\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] recursive remove (-r)  exit: gnu=%s corvo=%s\n" \
    "$_gnu_rec_ec" "$_corvo_rec_ec"
  FAIL=$((FAIL+1))
fi

# ── Verbose output (-v) ───────────────────────────────────────────────────────
# Use the same path for both so the printed filename matches.
echo "x" > "$_TD/vtest.txt"
gnu-rm -v "$_TD/vtest.txt" > /tmp/rm_gnu_v.out 2>/dev/null || true
echo "x" > "$_TD/vtest.txt"
corvo /corvo/coreutils/rm.corvo -- -v "$_TD/vtest.txt" > /tmp/rm_corvo_v.out 2>/dev/null || true
if diff -q /tmp/rm_gnu_v.out /tmp/rm_corvo_v.out >/dev/null 2>&1; then
  printf "PASS [rm] verbose (-v)\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] verbose (-v)\n"
  diff -u /tmp/rm_gnu_v.out /tmp/rm_corvo_v.out | head -10 || true
  FAIL=$((FAIL+1))
fi

# ── Remove empty directory with -d ────────────────────────────────────────────
mkdir -p "$_TD/emptydir_gnu" "$_TD/emptydir_corvo"
_gnu_d_ec=0; _corvo_d_ec=0
gnu-rm -d "$_TD/emptydir_gnu"   >/dev/null 2>&1 || _gnu_d_ec=$?
corvo /corvo/coreutils/rm.corvo -- -d "$_TD/emptydir_corvo" >/dev/null 2>&1 || _corvo_d_ec=$?
if [[ "$_gnu_d_ec" == "$_corvo_d_ec" ]] && \
   [[ ! -d "$_TD/emptydir_gnu" ]] && [[ ! -d "$_TD/emptydir_corvo" ]]; then
  printf "PASS [rm] remove empty directory (-d)\n"; PASS=$((PASS+1))
else
  printf "FAIL [rm] remove empty directory (-d)  exit: gnu=%s corvo=%s\n" \
    "$_gnu_d_ec" "$_corvo_d_ec"
  FAIL=$((FAIL+1))
fi

# -d on non-empty directory: should fail
mkdir -p "$_TD/nonempty_gnu" "$_TD/nonempty_corvo"
echo "x" > "$_TD/nonempty_gnu/f.txt"
echo "x" > "$_TD/nonempty_corvo/f.txt"
run_case rm "non-empty dir with -d (error)" \
  "gnu-rm -d '$_TD/nonempty_gnu'" \
  "corvo /corvo/coreutils/rm.corvo -- -d '$_TD/nonempty_corvo'"

# ── uutils comparison (informational) ────────────────────────────────────────
echo "u" > "$_TD/uu_gnu.txt"
echo "u" > "$_TD/uu_corvo.txt"
run_uutils_case rm "remove regular file" \
  "uu-rm '$_TD/uu_gnu.txt'" \
  "corvo /corvo/coreutils/rm.corvo -- '$_TD/uu_corvo.txt'"

cp /fixtures/a.txt "$_TD/time_gnu.txt" 2>/dev/null || echo "x" > "$_TD/time_gnu.txt"
show_time "gnu-rm file"   gnu-rm   "$_TD/time_gnu.txt"
cp /fixtures/a.txt "$_TD/time_corvo.txt" 2>/dev/null || echo "x" > "$_TD/time_corvo.txt"
show_time "corvo rm file" corvo /corvo/coreutils/rm.corvo -- "$_TD/time_corvo.txt" || true
