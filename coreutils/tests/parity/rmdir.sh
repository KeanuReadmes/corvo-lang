#!/usr/bin/env bash
# coreutils/tests/parity/rmdir.sh
# Required parity cases for rmdir.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== rmdir ==="

_TD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$_TD'" EXIT

# ── No operands ───────────────────────────────────────────────────────────────
run_case rmdir "no operand" \
  "gnu-rmdir" \
  "corvo /corvo/coreutils/rmdir.corvo"

# ── Non-existent directory ────────────────────────────────────────────────────
run_case rmdir "nonexistent directory" \
  "gnu-rmdir '$_TD/no_such_dir'" \
  "corvo /corvo/coreutils/rmdir.corvo -- '$_TD/no_such_dir'"

# ── Remove an empty directory ─────────────────────────────────────────────────
mkdir -p "$_TD/empty_gnu" "$_TD/empty_corvo"
_gnu_ec=0; _corvo_ec=0
gnu-rmdir "$_TD/empty_gnu"   >/dev/null 2>&1 || _gnu_ec=$?
corvo /corvo/coreutils/rmdir.corvo -- "$_TD/empty_corvo" >/dev/null 2>&1 || _corvo_ec=$?
if [[ "$_gnu_ec" == "$_corvo_ec" ]] && \
   [[ ! -d "$_TD/empty_gnu" ]] && [[ ! -d "$_TD/empty_corvo" ]]; then
  printf "PASS [rmdir] remove empty directory\n"; PASS=$((PASS+1))
else
  printf "FAIL [rmdir] remove empty directory  exit: gnu=%s corvo=%s\n" \
    "$_gnu_ec" "$_corvo_ec"
  FAIL=$((FAIL+1))
fi

# ── Non-empty directory (error) ───────────────────────────────────────────────
mkdir -p "$_TD/nonempty_gnu" "$_TD/nonempty_corvo"
echo "x" > "$_TD/nonempty_gnu/f.txt"
echo "x" > "$_TD/nonempty_corvo/f.txt"
run_case rmdir "non-empty directory (error)" \
  "gnu-rmdir '$_TD/nonempty_gnu'" \
  "corvo /corvo/coreutils/rmdir.corvo -- '$_TD/nonempty_corvo'"

# ── Remove parents (-p) ───────────────────────────────────────────────────────
mkdir -p "$_TD/gnu_p/b/c" "$_TD/corvo_p/b/c"
_gnu_p_ec=0; _corvo_p_ec=0
gnu-rmdir -p "$_TD/gnu_p/b/c"   >/dev/null 2>&1 || _gnu_p_ec=$?
corvo /corvo/coreutils/rmdir.corvo -- -p "$_TD/corvo_p/b/c" >/dev/null 2>&1 || _corvo_p_ec=$?
if [[ "$_gnu_p_ec" == "$_corvo_p_ec" ]] && \
   [[ ! -d "$_TD/gnu_p" ]] && [[ ! -d "$_TD/corvo_p" ]]; then
  printf "PASS [rmdir] parents (-p)\n"; PASS=$((PASS+1))
else
  printf "FAIL [rmdir] parents (-p)  exit: gnu=%s corvo=%s\n" \
    "$_gnu_p_ec" "$_corvo_p_ec"
  FAIL=$((FAIL+1))
fi

# ── Verbose output (-v) ───────────────────────────────────────────────────────
# Use the same path so the printed directory name matches.
mkdir -p "$_TD/vtest_rmdir"
gnu-rmdir -v "$_TD/vtest_rmdir" > /tmp/rmdir_gnu_v.out 2>/dev/null || true
mkdir -p "$_TD/vtest_rmdir"
corvo /corvo/coreutils/rmdir.corvo -- -v "$_TD/vtest_rmdir" > /tmp/rmdir_corvo_v.out 2>/dev/null || true
if diff -q /tmp/rmdir_gnu_v.out /tmp/rmdir_corvo_v.out >/dev/null 2>&1; then
  printf "PASS [rmdir] verbose (-v)\n"; PASS=$((PASS+1))
else
  printf "FAIL [rmdir] verbose (-v)\n"
  diff -u /tmp/rmdir_gnu_v.out /tmp/rmdir_corvo_v.out | head -10 || true
  FAIL=$((FAIL+1))
fi

# ── uutils comparison (informational) ────────────────────────────────────────
mkdir -p "$_TD/uu_gnu_dir" "$_TD/uu_corvo_dir"
run_uutils_case rmdir "remove empty directory" \
  "uu-rmdir '$_TD/uu_gnu_dir'" \
  "corvo /corvo/coreutils/rmdir.corvo -- '$_TD/uu_corvo_dir'"

mkdir -p "$_TD/time_gnu_dir" "$_TD/time_corvo_dir"
show_time "gnu-rmdir dir"   gnu-rmdir "$_TD/time_gnu_dir"
mkdir -p "$_TD/time_gnu_dir"  # recreate for corvo timing
show_time "corvo rmdir dir" corvo /corvo/coreutils/rmdir.corvo -- "$_TD/time_gnu_dir"
