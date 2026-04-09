#!/usr/bin/env bash
# coreutils/tests/parity/cp.sh
# Required parity cases for cp.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.

echo "=== cp ==="

_TD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$_TD'" EXIT

# Compare resulting file trees after running both cp implementations.
# Args: label gnu_src_args corvo_src_args  (destination is always appended)
_cp_content() {
  local label="$1" gnu_args="$2" corvo_args="$3"
  local gnu_dst="$_TD/gnu" corvo_dst="$_TD/corvo"
  rm -rf "$gnu_dst" "$corvo_dst"
  mkdir -p "$gnu_dst" "$corvo_dst"
  local gnu_ec=0 corvo_ec=0
  eval "gnu-cp $gnu_args \"$gnu_dst/\""                                   >/dev/null 2>&1 || gnu_ec=$?
  eval "corvo /corvo/coreutils/cp.corvo -- $corvo_args \"$corvo_dst/\""  >/dev/null 2>&1 || corvo_ec=$?
  if [[ "$gnu_ec" != "$corvo_ec" ]]; then
    printf "FAIL [cp] %-46s exit: gnu=%s corvo=%s\n" "$label" "$gnu_ec" "$corvo_ec"
    FAIL=$((FAIL+1)); return
  fi
  if ! diff -r "$gnu_dst" "$corvo_dst" >/dev/null 2>&1; then
    printf "FAIL [cp] %-46s result files differ\n" "$label"
    diff -r "$gnu_dst" "$corvo_dst" | head -15 || true
    FAIL=$((FAIL+1)); return
  fi
  printf "PASS [cp] %s\n" "$label"
  PASS=$((PASS+1))
}

# Content comparisons
_cp_content "basic file copy"    "/fixtures/a.txt" "/fixtures/a.txt"
_cp_content "two files into dir" "/fixtures/a.txt /fixtures/b.txt" "/fixtures/a.txt /fixtures/b.txt"

# Verbose output: identical destination so printed path matches
run_case cp "verbose (-v)"                 \
  "gnu-cp -v /fixtures/a.txt '$_TD/cp_v.txt'" \
  "corvo /corvo/coreutils/cp.corvo -- -v /fixtures/a.txt '$_TD/cp_v.txt'"

# Recursive copy — compare resulting directory trees
rm -rf "$_TD/rsub_gnu" "$_TD/rsub_corvo"
_gnu_rec_ec=0; _corvo_rec_ec=0
gnu-cp -r /fixtures/sub "$_TD/rsub_gnu"   >/dev/null 2>&1 || _gnu_rec_ec=$?
corvo /corvo/coreutils/cp.corvo -- -r /fixtures/sub "$_TD/rsub_corvo" >/dev/null 2>&1 || _corvo_rec_ec=$?
if [[ "$_gnu_rec_ec" == "$_corvo_rec_ec" ]] && \
   diff -r "$_TD/rsub_gnu" "$_TD/rsub_corvo" >/dev/null 2>&1; then
  printf "PASS [cp] recursive copy (-r)\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] recursive copy (-r)  exit: gnu=%s corvo=%s\n" \
    "$_gnu_rec_ec" "$_corvo_rec_ec"
  diff -r "$_TD/rsub_gnu" "$_TD/rsub_corvo" | head -10 || true
  FAIL=$((FAIL+1))
fi

# Error cases
run_case cp "missing source"    \
  "gnu-cp /fixtures/no_such_file /tmp/x_gnu" \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/no_such_file /tmp/x_corvo"

run_case cp "missing operand"   \
  "gnu-cp"                      \
  "corvo /corvo/coreutils/cp.corvo"

run_case cp "dir without -r"    \
  "gnu-cp /fixtures/sub /tmp/xd_gnu" \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/sub /tmp/xd_corvo"

# uutils comparison (informational)
run_uutils_case cp "basic file copy"         \
  "uu-cp /fixtures/a.txt '$_TD/uu_out.txt'"  \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt '$_TD/corvo_uu_out.txt'"

show_time "gnu-cp a.txt"   gnu-cp /fixtures/a.txt "$_TD/gnu_time.txt"
show_time "corvo cp a.txt" corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt "$_TD/corvo_time.txt"
