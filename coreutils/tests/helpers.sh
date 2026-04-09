#!/usr/bin/env bash
# coreutils/tests/helpers.sh
# Shared helper functions for all parity and matrix test scripts.
# Source this file; do NOT execute it directly.
#
# Callers must declare PASS and FAIL as integer variables before sourcing.

# Compare GNU vs Corvo: exit code and stdout must match.
# Args: section label gnu_cmd corvo_cmd
run_case() {
  local section="$1" label="$2" gnu_cmd="$3" corvo_cmd="$4"
  local gnu_ec=0 corvo_ec=0
  eval "$gnu_cmd"   > /tmp/t_gnu.out   2>/tmp/t_gnu.err   || gnu_ec=$?
  eval "$corvo_cmd" > /tmp/t_corvo.out 2>/tmp/t_corvo.err || corvo_ec=$?
  if [[ "$gnu_ec" != "$corvo_ec" ]]; then
    printf "FAIL [%-4s] %-46s exit: gnu=%s corvo=%s\n" \
      "$section" "$label" "$gnu_ec" "$corvo_ec"
    { cat /tmp/t_gnu.err /tmp/t_corvo.err; } 2>/dev/null | head -5 || true
    FAIL=$((FAIL+1)); return
  fi
  if ! diff -q /tmp/t_gnu.out /tmp/t_corvo.out >/dev/null 2>&1; then
    printf "FAIL [%-4s] %-46s stdout differs\n" "$section" "$label"
    diff -u /tmp/t_gnu.out /tmp/t_corvo.out | head -25 || true
    FAIL=$((FAIL+1)); return
  fi
  printf "PASS [%-4s] %s\n" "$section" "$label"
  PASS=$((PASS+1))
}

# Compare uutils vs Corvo: informational only (never fails the suite).
# Args: section label uu_cmd corvo_cmd
run_uutils_case() {
  local section="$1" label="$2" uu_cmd="$3" corvo_cmd="$4"
  local uu_bin; uu_bin="$(echo "$uu_cmd" | awk '{print $1}')"
  command -v "$uu_bin" >/dev/null 2>&1 || return 0
  local uu_ec=0 corvo_ec=0
  eval "$uu_cmd"    > /tmp/u_uu.out    2>/tmp/u_uu.err    || uu_ec=$?
  eval "$corvo_cmd" > /tmp/u_corvo.out 2>/tmp/u_corvo.err || corvo_ec=$?
  if [[ "$uu_ec" != "$corvo_ec" ]] || \
     ! diff -q /tmp/u_uu.out /tmp/u_corvo.out >/dev/null 2>&1; then
    printf "INFO [%-4s] %-46s uutils differs (not required)\n" "$section" "$label"
  else
    printf "INFO [%-4s] %-46s uutils matches\n" "$section" "$label"
  fi
}

# Print wall-clock execution time for a command (informational).
# Args: label cmd [args...]
show_time() {
  local label="$1"; shift
  local start end ms
  start=$(date +%s%N 2>/dev/null) || { echo "INFO: timing unavailable"; return 0; }
  "$@" >/dev/null 2>&1 || true
  end=$(date +%s%N)
  ms=$(( (end - start) / 1000000 ))
  printf "TIME  %-48s %dms\n" "$label" "$ms"
}
