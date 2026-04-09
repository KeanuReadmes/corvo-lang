#!/usr/bin/env bash
# coreutils/tests/run-parity.sh
# Compare all Corvo coreutils tools against GNU coreutils (required CI).
# Also reports parity against uutils where available (informational only).
#
# Usage:
#   coreutils/tests/run-parity.sh
#   coreutils/tests/run-parity.sh --require-docker
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

REQUIRE_DOCKER=0
for arg in "$@"; do
  [[ "$arg" == "--require-docker" ]] && REQUIRE_DOCKER=1
done

if ! command -v docker >/dev/null 2>&1; then
  if [[ "$REQUIRE_DOCKER" -eq 1 ]]; then
    echo "error: docker is required (pass without --require-docker to skip locally)" >&2
    exit 1
  fi
  echo "skip: docker not installed"
  exit 0
fi

docker build -f "$ROOT/coreutils/tests/Dockerfile" -t corvo-coreutils-parity "$ROOT"

FIX="$(mktemp -d)"
trap 'rm -rf "$FIX"' EXIT

# ── Fixtures (created on the host, mounted read-only into Docker) ──────────────
mkdir -p "$FIX/sub/inner" "$FIX/nested/deep"
printf "line1\nline2\nline3\n"       > "$FIX/a.txt"
printf "alpha\nbeta\ngamma\n"        > "$FIX/b.txt"
printf "line1\n\n\nline4\n\nline6\n" > "$FIX/blank.txt"
printf "col1\tcol2\tcol3\n"          > "$FIX/tabs.txt"
i=1; while [ "$i" -le 30 ]; do printf "line%d\n" "$i"; i=$((i+1)); done > "$FIX/long.txt"
printf "hidden\n"                    > "$FIX/.hidden"
printf "backup\n"                    > "$FIX/c.txt~"
printf "inner file\n"                > "$FIX/sub/inner/z.txt"
printf "nested\n"                    > "$FIX/nested/deep/n.txt"
ln -s a.txt   "$FIX/link-a"      2>/dev/null || true
ln -s missing "$FIX/broken-link" 2>/dev/null || true

docker run --rm -i \
  -e LC_ALL=C \
  -e TZ=UTC \
  -v "$FIX:/fixtures:ro" \
  corvo-coreutils-parity bash -s << 'INNER'
set -uo pipefail

PASS=0; FAIL=0

# ── Helpers ────────────────────────────────────────────────────────────────────
# Compare GNU vs Corvo: exit code and stdout must match.
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

# Measure wall-clock time for a command (informational).
show_time() {
  local label="$1"; shift
  local start end ms
  start=$(date +%s%N 2>/dev/null) || { echo "INFO: timing unavailable"; return 0; }
  "$@" >/dev/null 2>&1 || true
  end=$(date +%s%N)
  ms=$(( (end - start) / 1000000 ))
  printf "TIME  %-48s %dms\n" "$label" "$ms"
}

# ── LS ─────────────────────────────────────────────────────────────────────────
cd /fixtures
echo "=== ls ==="
run_case ls "one column"             "gnu-ls -1"                                        "corvo /corvo/coreutils/ls.corvo -- -1"
run_case ls "almost-all (-A)"        "gnu-ls -1 -A"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -A"
run_case ls "all (-a)"               "gnu-ls -1 -a"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -a"
run_case ls "long-iso"               "gnu-ls -l --time-style=long-iso"                  "corvo /corvo/coreutils/ls.corvo -- -l --time-style=long-iso"
run_case ls "long-iso -A"            "gnu-ls -1 -A -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -A -l --time-style=long-iso"
run_case ls "long-iso -a"            "gnu-ls -1 -a -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -a -l --time-style=long-iso"
run_case ls "inode short"            "gnu-ls -1 -i"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -i"
run_case ls "inode long"             "gnu-ls -1 -l --time-style=long-iso -i"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -i"
run_case ls "reverse"                "gnu-ls -1 -r"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -r"
run_case ls "reverse long"           "gnu-ls -1 -l --time-style=long-iso -r"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -r"
run_case ls "classify (-F)"          "gnu-ls -1 -F"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -F"
run_case ls "blocks (-s)"            "gnu-ls -1 -s"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -s"
run_case ls "blocks long"            "gnu-ls -1 -s -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -s -l --time-style=long-iso"
run_case ls "human-readable (-h)"    "gnu-ls -1 -h -l --time-style=long-iso"            "corvo /corvo/coreutils/ls.corvo -- -1 -h -l --time-style=long-iso"
run_case ls "numeric-ids (-n)"       "gnu-ls -1 -l --time-style=long-iso -n"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -n"
run_case ls "no-group (-G)"          "gnu-ls -1 -l --time-style=long-iso -G"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -G"
run_case ls "comma (-m)"             "gnu-ls -m"                                        "corvo /corvo/coreutils/ls.corvo -- -m"
run_case ls "sort-size (-S)"         "gnu-ls -1 -l --time-style=long-iso -S"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -S"
run_case ls "sort-time (-t)"         "gnu-ls -1 -l --time-style=long-iso -t"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -t"
run_case ls "sort-none (-U)"         "gnu-ls -U -1"                                     "corvo /corvo/coreutils/ls.corvo -- -U -1"
run_case ls "sort-version (-v)"      "gnu-ls -1 -v"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -v"
run_case ls "sort-ext (-X)"          "gnu-ls -1 -X"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -X"
run_case ls "recursive (-R)"         "gnu-ls -R"                                        "corvo /corvo/coreutils/ls.corvo -- -R"
run_case ls "directory (-d)"         "gnu-ls -d /fixtures"                              "corvo /corvo/coreutils/ls.corvo -- -d /fixtures"
run_case ls "color=never long"       "gnu-ls -1 --color=never -l --time-style=long-iso" "corvo /corvo/coreutils/ls.corvo -- -1 --color=never -l --time-style=long-iso"
run_case ls "ignore (*.txt)"         "gnu-ls -1 -I '*.txt'"                             "corvo /corvo/coreutils/ls.corvo -- -1 -I '*.txt'"
run_case ls "hide backup (~)"        "gnu-ls -1 --hide='*~'"                            "corvo /corvo/coreutils/ls.corvo -- -1 --hide='*~'"
run_case ls "literal (-N)"           "gnu-ls -1 -N"                                     "corvo /corvo/coreutils/ls.corvo -- -1 -N"
run_case ls "time-access (-u)"       "gnu-ls -1 -l --time-style=long-iso -u"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -u"
run_case ls "time-status (-c)"       "gnu-ls -1 -l --time-style=long-iso -c"            "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -c"
run_case ls "hyperlink=no"           "gnu-ls -1 --hyperlink=no"                         "corvo /corvo/coreutils/ls.corvo -- -1 --hyperlink=no"
run_uutils_case ls "one column"      "uu-ls -1"                                         "corvo /corvo/coreutils/ls.corvo -- -1"
run_uutils_case ls "long-iso"        "uu-ls -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -l --time-style=long-iso"
show_time "gnu-ls -la"               gnu-ls -la /fixtures
show_time "corvo ls -la"             corvo /corvo/coreutils/ls.corvo -- -la /fixtures

# ── CAT ────────────────────────────────────────────────────────────────────────
echo "=== cat ==="
run_case cat "plain a.txt"            "gnu-cat /fixtures/a.txt"                          "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt"
run_case cat "plain b.txt"            "gnu-cat /fixtures/b.txt"                          "corvo /corvo/coreutils/cat.corvo -- /fixtures/b.txt"
run_case cat "two files"              "gnu-cat /fixtures/a.txt /fixtures/b.txt"          "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case cat "-n number all"          "gnu-cat -n /fixtures/a.txt"                       "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/a.txt"
run_case cat "-b number-nonblank"     "gnu-cat -b /fixtures/blank.txt"                   "corvo /corvo/coreutils/cat.corvo -- -b /fixtures/blank.txt"
run_case cat "-s squeeze-blank"       "gnu-cat -s /fixtures/blank.txt"                   "corvo /corvo/coreutils/cat.corvo -- -s /fixtures/blank.txt"
run_case cat "-E show-ends"           "gnu-cat -E /fixtures/a.txt"                       "corvo /corvo/coreutils/cat.corvo -- -E /fixtures/a.txt"
run_case cat "-T show-tabs"           "gnu-cat -T /fixtures/tabs.txt"                    "corvo /corvo/coreutils/cat.corvo -- -T /fixtures/tabs.txt"
run_case cat "-A show-all"            "gnu-cat -A /fixtures/tabs.txt"                    "corvo /corvo/coreutils/cat.corvo -- -A /fixtures/tabs.txt"
run_case cat "-n -E combined"         "gnu-cat -n -E /fixtures/a.txt"                    "corvo /corvo/coreutils/cat.corvo -- -n -E /fixtures/a.txt"
run_case cat "-b -E combined"         "gnu-cat -b -E /fixtures/blank.txt"                "corvo /corvo/coreutils/cat.corvo -- -b -E /fixtures/blank.txt"
run_case cat "-n -s combined"         "gnu-cat -n -s /fixtures/blank.txt"                "corvo /corvo/coreutils/cat.corvo -- -n -s /fixtures/blank.txt"
run_case cat "missing file"           "gnu-cat /fixtures/no_such_file"                   "corvo /corvo/coreutils/cat.corvo -- /fixtures/no_such_file"
run_uutils_case cat "plain a.txt"     "uu-cat /fixtures/a.txt"                           "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt"
run_uutils_case cat "-n number"       "uu-cat -n /fixtures/a.txt"                        "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/a.txt"
show_time "gnu-cat long.txt"          gnu-cat /fixtures/long.txt
show_time "corvo cat long.txt"        corvo /corvo/coreutils/cat.corvo -- /fixtures/long.txt

# ── HEAD ───────────────────────────────────────────────────────────────────────
echo "=== head ==="
run_case head "default (10 lines)"    "gnu-head /fixtures/long.txt"                      "corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt"
run_case head "-n 5"                  "gnu-head -n 5 /fixtures/long.txt"                 "corvo /corvo/coreutils/head.corvo -- -n 5 /fixtures/long.txt"
run_case head "-n 0"                  "gnu-head -n 0 /fixtures/long.txt"                 "corvo /corvo/coreutils/head.corvo -- -n 0 /fixtures/long.txt"
run_case head "-n 50 (exceeds len)"   "gnu-head -n 50 /fixtures/long.txt"                "corvo /corvo/coreutils/head.corvo -- -n 50 /fixtures/long.txt"
run_case head "-n -5 (all-but-last)"  "gnu-head -n -5 /fixtures/long.txt"                "corvo /corvo/coreutils/head.corvo -- -n -5 /fixtures/long.txt"
run_case head "-c 20 (bytes)"         "gnu-head -c 20 /fixtures/a.txt"                   "corvo /corvo/coreutils/head.corvo -- -c 20 /fixtures/a.txt"
run_case head "-c -5 (all-but-last)"  "gnu-head -c -5 /fixtures/a.txt"                   "corvo /corvo/coreutils/head.corvo -- -c -5 /fixtures/a.txt"
run_case head "two files (headers)"   "gnu-head /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/head.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case head "-q quiet"              "gnu-head -q /fixtures/a.txt /fixtures/b.txt"      "corvo /corvo/coreutils/head.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_case head "-v verbose single"     "gnu-head -v /fixtures/a.txt"                      "corvo /corvo/coreutils/head.corvo -- -v /fixtures/a.txt"
run_case head "missing file"          "gnu-head /fixtures/no_such_file"                  "corvo /corvo/coreutils/head.corvo -- /fixtures/no_such_file"
run_uutils_case head "default"        "uu-head /fixtures/long.txt"                       "corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt"
run_uutils_case head "-n 5"           "uu-head -n 5 /fixtures/long.txt"                  "corvo /corvo/coreutils/head.corvo -- -n 5 /fixtures/long.txt"
show_time "gnu-head long.txt"         gnu-head /fixtures/long.txt
show_time "corvo head long.txt"       corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt

# ── TAIL ───────────────────────────────────────────────────────────────────────
echo "=== tail ==="
run_case tail "default (10 lines)"    "gnu-tail /fixtures/long.txt"                      "corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt"
run_case tail "-n 5"                  "gnu-tail -n 5 /fixtures/long.txt"                 "corvo /corvo/coreutils/tail.corvo -- -n 5 /fixtures/long.txt"
run_case tail "-n 0"                  "gnu-tail -n 0 /fixtures/long.txt"                 "corvo /corvo/coreutils/tail.corvo -- -n 0 /fixtures/long.txt"
run_case tail "-n 50 (exceeds len)"   "gnu-tail -n 50 /fixtures/long.txt"                "corvo /corvo/coreutils/tail.corvo -- -n 50 /fixtures/long.txt"
run_case tail "-n +3 (from line 3)"   "gnu-tail -n +3 /fixtures/long.txt"                "corvo /corvo/coreutils/tail.corvo -- -n +3 /fixtures/long.txt"
run_case tail "-n +1 (all lines)"     "gnu-tail -n +1 /fixtures/long.txt"                "corvo /corvo/coreutils/tail.corvo -- -n +1 /fixtures/long.txt"
run_case tail "-c 20 (bytes)"         "gnu-tail -c 20 /fixtures/a.txt"                   "corvo /corvo/coreutils/tail.corvo -- -c 20 /fixtures/a.txt"
run_case tail "-c +1 (all bytes)"     "gnu-tail -c +1 /fixtures/a.txt"                   "corvo /corvo/coreutils/tail.corvo -- -c +1 /fixtures/a.txt"
run_case tail "two files (headers)"   "gnu-tail /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/tail.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_case tail "-q quiet"              "gnu-tail -q /fixtures/a.txt /fixtures/b.txt"      "corvo /corvo/coreutils/tail.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_case tail "-v verbose single"     "gnu-tail -v /fixtures/a.txt"                      "corvo /corvo/coreutils/tail.corvo -- -v /fixtures/a.txt"
run_case tail "missing file"          "gnu-tail /fixtures/no_such_file"                  "corvo /corvo/coreutils/tail.corvo -- /fixtures/no_such_file"
run_uutils_case tail "default"        "uu-tail /fixtures/long.txt"                       "corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt"
run_uutils_case tail "-n 5"           "uu-tail -n 5 /fixtures/long.txt"                  "corvo /corvo/coreutils/tail.corvo -- -n 5 /fixtures/long.txt"
show_time "gnu-tail long.txt"         gnu-tail /fixtures/long.txt
show_time "corvo tail long.txt"       corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt

# ── CP ─────────────────────────────────────────────────────────────────────────
echo "=== cp ==="
TD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$TD'" EXIT

# Helper: run both cp implementations and compare the resulting file tree.
test_cp_content() {
  local label="$1" gnu_args="$2" corvo_args="$3"
  local gnu_dst="$TD/gnu" corvo_dst="$TD/corvo"
  rm -rf "$gnu_dst" "$corvo_dst"
  mkdir -p "$gnu_dst" "$corvo_dst"
  local gnu_ec=0 corvo_ec=0
  eval "gnu-cp $gnu_args \"$gnu_dst/\""                         >/dev/null 2>&1 || gnu_ec=$?
  eval "corvo /corvo/coreutils/cp.corvo -- $corvo_args \"$corvo_dst/\"" >/dev/null 2>&1 || corvo_ec=$?
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

# Basic single-file copy — compare file content
test_cp_content "basic file copy"          "/fixtures/a.txt" "/fixtures/a.txt"

# Two-file copy into directory — compare resulting dir
test_cp_content "two files into dir"       "/fixtures/a.txt /fixtures/b.txt" \
                                           "/fixtures/a.txt /fixtures/b.txt"

# Verbose output: both use the same dest path so the printed path is identical
run_case cp "verbose (-v)"                 \
  "gnu-cp -v /fixtures/a.txt '$TD/cp_v.txt'" \
  "corvo /corvo/coreutils/cp.corvo -- -v /fixtures/a.txt '$TD/cp_v.txt'"

# Recursive copy — compare resulting directory trees
rm -rf "$TD/rsub_gnu" "$TD/rsub_corvo"
gnu_rec_ec=0; corvo_rec_ec=0
gnu-cp -r /fixtures/sub "$TD/rsub_gnu"   >/dev/null 2>&1 || gnu_rec_ec=$?
corvo /corvo/coreutils/cp.corvo -- -r /fixtures/sub "$TD/rsub_corvo" >/dev/null 2>&1 || corvo_rec_ec=$?
if [[ "$gnu_rec_ec" == "$corvo_rec_ec" ]] && \
   diff -r "$TD/rsub_gnu" "$TD/rsub_corvo" >/dev/null 2>&1; then
  printf "PASS [cp] recursive copy (-r)\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] recursive copy (-r)  exit: gnu=%s corvo=%s\n" \
    "$gnu_rec_ec" "$corvo_rec_ec"
  diff -r "$TD/rsub_gnu" "$TD/rsub_corvo" | head -10 || true
  FAIL=$((FAIL+1))
fi

# Error: missing source
run_case cp "missing source"               \
  "gnu-cp /fixtures/no_such_file /tmp/x_gnu" \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/no_such_file /tmp/x_corvo"

# Error: missing operand (no arguments)
run_case cp "missing operand"              \
  "gnu-cp"                                 \
  "corvo /corvo/coreutils/cp.corvo"

# Error: directory without -r
run_case cp "dir without -r"               \
  "gnu-cp /fixtures/sub /tmp/xd_gnu"       \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/sub /tmp/xd_corvo"

# uutils comparison (informational)
run_uutils_case cp "basic file copy"       \
  "uu-cp /fixtures/a.txt '$TD/uu_out.txt'" \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt '$TD/corvo_uu_out.txt'"

show_time "gnu-cp a.txt"   gnu-cp /fixtures/a.txt "$TD/gnu_time.txt"
show_time "corvo cp a.txt" corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt "$TD/corvo_time.txt"

# ── Summary ────────────────────────────────────────────────────────────────────
echo ""
echo "=================================================="
echo "  coreutils parity:  PASS=$PASS  FAIL=$FAIL"
echo "=================================================="
exit $(( FAIL > 0 ? 1 : 0 ))
INNER
