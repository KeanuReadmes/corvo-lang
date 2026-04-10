#!/usr/bin/env bash
# coreutils/tests/parity/mkdir.sh
# Required parity cases for mkdir.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== mkdir ==="

_TD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$_TD'" EXIT

# ── No operands ───────────────────────────────────────────────────────────────
run_case mkdir "no operand" \
  "gnu-mkdir" \
  "corvo /corvo/coreutils/mkdir.corvo"

# ── Create a new directory ────────────────────────────────────────────────────
_gnu_ec=0; _corvo_ec=0
gnu-mkdir "$_TD/gnu_new"   >/dev/null 2>&1 || _gnu_ec=$?
corvo /corvo/coreutils/mkdir.corvo -- "$_TD/corvo_new" >/dev/null 2>&1 || _corvo_ec=$?
if [[ "$_gnu_ec" == "$_corvo_ec" ]] && \
   [[ -d "$_TD/gnu_new" ]] && [[ -d "$_TD/corvo_new" ]]; then
  printf "PASS [mkdir] create new directory\n"; PASS=$((PASS+1))
else
  printf "FAIL [mkdir] create new directory  exit: gnu=%s corvo=%s\n" \
    "$_gnu_ec" "$_corvo_ec"
  FAIL=$((FAIL+1))
fi

# ── Directory already exists (error without -p) ───────────────────────────────
mkdir -p "$_TD/existing_gnu" "$_TD/existing_corvo"
run_case mkdir "existing directory (error)" \
  "gnu-mkdir '$_TD/existing_gnu'" \
  "corvo /corvo/coreutils/mkdir.corvo -- '$_TD/existing_corvo'"

# ── -p: no error when directory already exists ────────────────────────────────
mkdir -p "$_TD/exist_p_gnu" "$_TD/exist_p_corvo"
run_case mkdir "existing directory with -p (no error)" \
  "gnu-mkdir -p '$_TD/exist_p_gnu'" \
  "corvo /corvo/coreutils/mkdir.corvo -- -p '$_TD/exist_p_corvo'"

# ── -p: create nested directories ────────────────────────────────────────────
_gnu_p_ec=0; _corvo_p_ec=0
gnu-mkdir -p "$_TD/gnu_nested/b/c"   >/dev/null 2>&1 || _gnu_p_ec=$?
corvo /corvo/coreutils/mkdir.corvo -- -p "$_TD/corvo_nested/b/c" >/dev/null 2>&1 || _corvo_p_ec=$?
if [[ "$_gnu_p_ec" == "$_corvo_p_ec" ]] && \
   [[ -d "$_TD/gnu_nested/b/c" ]] && [[ -d "$_TD/corvo_nested/b/c" ]]; then
  printf "PASS [mkdir] create nested directories (-p)\n"; PASS=$((PASS+1))
else
  printf "FAIL [mkdir] create nested directories (-p)  exit: gnu=%s corvo=%s\n" \
    "$_gnu_p_ec" "$_corvo_p_ec"
  FAIL=$((FAIL+1))
fi

# ── Verbose output (-v) ───────────────────────────────────────────────────────
# Use the same (not-yet-existing) path so printed names match.
gnu-mkdir -v "$_TD/vtest_mkdir" > /tmp/mkdir_gnu_v.out 2>/dev/null || true
rm -rf "$_TD/vtest_mkdir"
corvo /corvo/coreutils/mkdir.corvo -- -v "$_TD/vtest_mkdir" > /tmp/mkdir_corvo_v.out 2>/dev/null || true
if diff -q /tmp/mkdir_gnu_v.out /tmp/mkdir_corvo_v.out >/dev/null 2>&1; then
  printf "PASS [mkdir] verbose (-v)\n"; PASS=$((PASS+1))
else
  printf "FAIL [mkdir] verbose (-v)\n"
  diff -u /tmp/mkdir_gnu_v.out /tmp/mkdir_corvo_v.out | head -10 || true
  FAIL=$((FAIL+1))
fi

# ── uutils comparison (informational) ────────────────────────────────────────
run_uutils_case mkdir "no operand" \
  "uu-mkdir" \
  "corvo /corvo/coreutils/mkdir.corvo"

show_time "gnu-mkdir dir"   gnu-mkdir "$_TD/time_gnu_dir"
show_time "corvo mkdir dir" corvo /corvo/coreutils/mkdir.corvo -- "$_TD/time_corvo_dir"
