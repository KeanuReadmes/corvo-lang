#!/usr/bin/env bash
# coreutils/tests/run-parity-matrix.sh
# Extended flag-combination matrix comparing Corvo coreutils tools against
# GNU coreutils (and uutils when available).  Prints PASS/FAIL per case.
#
# Usage:
#   coreutils/tests/run-parity-matrix.sh
#   coreutils/tests/run-parity-matrix.sh --require-docker
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

REQUIRE_DOCKER=0
for arg in "$@"; do
  [[ "$arg" == "--require-docker" ]] && REQUIRE_DOCKER=1
done

if ! command -v docker >/dev/null 2>&1; then
  if [[ "$REQUIRE_DOCKER" -eq 1 ]]; then
    echo "error: docker is required" >&2
    exit 1
  fi
  echo "skip: docker not installed"
  exit 0
fi

docker build -f "$ROOT/coreutils/tests/Dockerfile" -t corvo-coreutils-parity "$ROOT" >/dev/null

FIX="$(mktemp -d)"
trap 'rm -rf "$FIX"' EXIT

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
run_row() {
  local section="$1" label="$2" gnu_cmd="$3" corvo_cmd="$4"
  local gnu_ec=0 corvo_ec=0
  eval "$gnu_cmd"   > /tmp/m_gnu.out   2>/tmp/m_gnu.err   || gnu_ec=$?
  eval "$corvo_cmd" > /tmp/m_corvo.out 2>/tmp/m_corvo.err || corvo_ec=$?
  if [[ "$gnu_ec" != "$corvo_ec" ]]; then
    printf "FAIL [%-4s] %-48s exit: gnu=%s corvo=%s\n" \
      "$section" "$label" "$gnu_ec" "$corvo_ec"
    { cat /tmp/m_gnu.err /tmp/m_corvo.err; } 2>/dev/null | head -4 || true
    FAIL=$((FAIL+1)); return
  fi
  if ! diff -q /tmp/m_gnu.out /tmp/m_corvo.out >/dev/null 2>&1; then
    printf "FAIL [%-4s] %-48s stdout differs\n" "$section" "$label"
    diff -u /tmp/m_gnu.out /tmp/m_corvo.out | head -20 || true
    FAIL=$((FAIL+1)); return
  fi
  printf "PASS [%-4s] %s\n" "$section" "$label"
  PASS=$((PASS+1))
}

printf "%-8s %-50s %s\n" "SECTION" "CASE" "RESULT"
printf "%-8s %-50s %s\n" "--------" "--------------------------------------------------" "--------"

# ══════════════════════════════════════════════════════════════════════════════
# LS
# ══════════════════════════════════════════════════════════════════════════════
cd /fixtures

# Output format flags
run_row ls "one column (-1)"                      "gnu-ls -1"                                               "corvo /corvo/coreutils/ls.corvo -- -1"
run_row ls "long (-l) long-iso"                   "gnu-ls -l --time-style=long-iso"                         "corvo /corvo/coreutils/ls.corvo -- -l --time-style=long-iso"
run_row ls "columns (-C)"                         "gnu-ls -C"                                               "corvo /corvo/coreutils/ls.corvo -- -C"
run_row ls "comma (-m)"                           "gnu-ls -m"                                               "corvo /corvo/coreutils/ls.corvo -- -m"

# Show-all flags
run_row ls "all (-a)"                             "gnu-ls -1 -a"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -a"
run_row ls "almost-all (-A)"                      "gnu-ls -1 -A"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -A"
run_row ls "all long"                             "gnu-ls -1 -a -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -1 -a -l --time-style=long-iso"
run_row ls "almost-all long"                      "gnu-ls -1 -A -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -1 -A -l --time-style=long-iso"

# Inode / blocks
run_row ls "inode short (-i)"                     "gnu-ls -1 -i"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -i"
run_row ls "inode long"                           "gnu-ls -1 -l --time-style=long-iso -i"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -i"
run_row ls "blocks (-s)"                          "gnu-ls -1 -s"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -s"
run_row ls "blocks long"                          "gnu-ls -1 -s -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -1 -s -l --time-style=long-iso"

# Size format
run_row ls "human-readable (-h) long"             "gnu-ls -1 -h -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -1 -h -l --time-style=long-iso"
run_row ls "si long"                              "gnu-ls -1 -l --time-style=long-iso --si"                 "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso --si"

# Ownership
run_row ls "numeric-ids (-n) long"                "gnu-ls -1 -l --time-style=long-iso -n"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -n"
run_row ls "no-group (-G) long"                   "gnu-ls -1 -l --time-style=long-iso -G"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -G"

# Classify / indicator
run_row ls "classify (-F)"                        "gnu-ls -1 -F"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -F"
run_row ls "classify long"                        "gnu-ls -1 -F -l --time-style=long-iso"                   "corvo /corvo/coreutils/ls.corvo -- -1 -F -l --time-style=long-iso"

# Sort
run_row ls "sort-size (-S) long"                  "gnu-ls -1 -l --time-style=long-iso -S"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -S"
run_row ls "sort-time (-t) long"                  "gnu-ls -1 -l --time-style=long-iso -t"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -t"
run_row ls "sort-none (-U)"                       "gnu-ls -U -1"                                            "corvo /corvo/coreutils/ls.corvo -- -U -1"
run_row ls "sort-version (-v)"                    "gnu-ls -1 -v"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -v"
run_row ls "sort-ext (-X)"                        "gnu-ls -1 -X"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -X"
run_row ls "reverse (-r)"                         "gnu-ls -1 -r"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -r"
run_row ls "reverse long"                         "gnu-ls -1 -l --time-style=long-iso -r"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -r"
run_row ls "reverse sort-time"                    "gnu-ls -1 -l --time-style=long-iso -t -r"                "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -t -r"
run_row ls "reverse sort-size"                    "gnu-ls -1 -l --time-style=long-iso -S -r"                "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -S -r"

# Time fields
run_row ls "time-access (-u) long"                "gnu-ls -1 -l --time-style=long-iso -u"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -u"
run_row ls "time-status (-c) long"                "gnu-ls -1 -l --time-style=long-iso -c"                   "corvo /corvo/coreutils/ls.corvo -- -1 -l --time-style=long-iso -c"

# Filtering
run_row ls "ignore (*.txt)"                       "gnu-ls -1 -I '*.txt'"                                    "corvo /corvo/coreutils/ls.corvo -- -1 -I '*.txt'"
run_row ls "hide backup (~)"                      "gnu-ls -1 --hide='*~'"                                   "corvo /corvo/coreutils/ls.corvo -- -1 --hide='*~'"
run_row ls "hide-backup-files (-B)"               "gnu-ls -1 -B"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -B"

# Traversal
run_row ls "recursive (-R)"                       "gnu-ls -R"                                               "corvo /corvo/coreutils/ls.corvo -- -R"
run_row ls "recursive long"                       "gnu-ls -R -l --time-style=long-iso"                      "corvo /corvo/coreutils/ls.corvo -- -R -l --time-style=long-iso"
run_row ls "directory (-d)"                       "gnu-ls -d /fixtures"                                     "corvo /corvo/coreutils/ls.corvo -- -d /fixtures"

# Misc
run_row ls "literal (-N)"                         "gnu-ls -1 -N"                                            "corvo /corvo/coreutils/ls.corvo -- -1 -N"
run_row ls "color=never"                          "gnu-ls -1 --color=never"                                 "corvo /corvo/coreutils/ls.corvo -- -1 --color=never"
run_row ls "color=never long"                     "gnu-ls -1 --color=never -l --time-style=long-iso"        "corvo /corvo/coreutils/ls.corvo -- -1 --color=never -l --time-style=long-iso"
run_row ls "hyperlink=no"                         "gnu-ls -1 --hyperlink=no"                                "corvo /corvo/coreutils/ls.corvo -- -1 --hyperlink=no"
run_row ls "width 40 (-w)"                        "gnu-ls -1 -w 40"                                         "corvo /corvo/coreutils/ls.corvo -- -1 -w 40"

# Combined
run_row ls "inode+reverse long"                   "gnu-ls -1 -i -r -l --time-style=long-iso"               "corvo /corvo/coreutils/ls.corvo -- -1 -i -r -l --time-style=long-iso"
run_row ls "almost-all+sort-size"                 "gnu-ls -1 -A -S"                                         "corvo /corvo/coreutils/ls.corvo -- -1 -A -S"
run_row ls "all+sort-time long"                   "gnu-ls -1 -a -t -l --time-style=long-iso"               "corvo /corvo/coreutils/ls.corvo -- -1 -a -t -l --time-style=long-iso"

# ══════════════════════════════════════════════════════════════════════════════
# CAT
# ══════════════════════════════════════════════════════════════════════════════
run_row cat "plain single file"                   "gnu-cat /fixtures/a.txt"                                  "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt"
run_row cat "plain two files"                     "gnu-cat /fixtures/a.txt /fixtures/b.txt"                  "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_row cat "three files"                         "gnu-cat /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt" "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt"
run_row cat "-n number all lines"                 "gnu-cat -n /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/a.txt"
run_row cat "-n with blank lines"                 "gnu-cat -n /fixtures/blank.txt"                           "corvo /corvo/coreutils/cat.corvo -- -n /fixtures/blank.txt"
run_row cat "-b number non-blank"                 "gnu-cat -b /fixtures/blank.txt"                           "corvo /corvo/coreutils/cat.corvo -- -b /fixtures/blank.txt"
run_row cat "-b overrides -n"                     "gnu-cat -n -b /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -n -b /fixtures/blank.txt"
run_row cat "-s squeeze blank"                    "gnu-cat -s /fixtures/blank.txt"                           "corvo /corvo/coreutils/cat.corvo -- -s /fixtures/blank.txt"
run_row cat "-E show ends"                        "gnu-cat -E /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -E /fixtures/a.txt"
run_row cat "-T show tabs"                        "gnu-cat -T /fixtures/tabs.txt"                            "corvo /corvo/coreutils/cat.corvo -- -T /fixtures/tabs.txt"
run_row cat "-A show-all"                         "gnu-cat -A /fixtures/tabs.txt"                            "corvo /corvo/coreutils/cat.corvo -- -A /fixtures/tabs.txt"
run_row cat "-e equiv to -vE"                     "gnu-cat -e /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -e /fixtures/a.txt"
run_row cat "-t equiv to -vT"                     "gnu-cat -t /fixtures/tabs.txt"                            "corvo /corvo/coreutils/cat.corvo -- -t /fixtures/tabs.txt"
run_row cat "-n -E combined"                      "gnu-cat -n -E /fixtures/a.txt"                            "corvo /corvo/coreutils/cat.corvo -- -n -E /fixtures/a.txt"
run_row cat "-b -E combined"                      "gnu-cat -b -E /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -b -E /fixtures/blank.txt"
run_row cat "-n -s combined"                      "gnu-cat -n -s /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -n -s /fixtures/blank.txt"
run_row cat "-b -s combined"                      "gnu-cat -b -s /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -b -s /fixtures/blank.txt"
run_row cat "-s -E combined"                      "gnu-cat -s -E /fixtures/blank.txt"                        "corvo /corvo/coreutils/cat.corvo -- -s -E /fixtures/blank.txt"
run_row cat "-n -s -E all"                        "gnu-cat -n -s -E /fixtures/blank.txt"                     "corvo /corvo/coreutils/cat.corvo -- -n -s -E /fixtures/blank.txt"
run_row cat "-u (unbuffered, ignored)"            "gnu-cat -u /fixtures/a.txt"                               "corvo /corvo/coreutils/cat.corvo -- -u /fixtures/a.txt"
run_row cat "missing file"                        "gnu-cat /fixtures/no_such_file"                           "corvo /corvo/coreutils/cat.corvo -- /fixtures/no_such_file"
run_row cat "missing among valid"                 "gnu-cat /fixtures/a.txt /fixtures/no_such_file /fixtures/b.txt" "corvo /corvo/coreutils/cat.corvo -- /fixtures/a.txt /fixtures/no_such_file /fixtures/b.txt"

# ══════════════════════════════════════════════════════════════════════════════
# HEAD
# ══════════════════════════════════════════════════════════════════════════════
run_row head "default (10 lines)"                 "gnu-head /fixtures/long.txt"                              "corvo /corvo/coreutils/head.corvo -- /fixtures/long.txt"
run_row head "-n 1"                               "gnu-head -n 1 /fixtures/long.txt"                         "corvo /corvo/coreutils/head.corvo -- -n 1 /fixtures/long.txt"
run_row head "-n 5"                               "gnu-head -n 5 /fixtures/long.txt"                         "corvo /corvo/coreutils/head.corvo -- -n 5 /fixtures/long.txt"
run_row head "-n 10"                              "gnu-head -n 10 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n 10 /fixtures/long.txt"
run_row head "-n 0"                               "gnu-head -n 0 /fixtures/long.txt"                         "corvo /corvo/coreutils/head.corvo -- -n 0 /fixtures/long.txt"
run_row head "-n 30 (exact)"                      "gnu-head -n 30 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n 30 /fixtures/long.txt"
run_row head "-n 50 (exceeds)"                    "gnu-head -n 50 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n 50 /fixtures/long.txt"
run_row head "-n -1 (all-but-last-1)"             "gnu-head -n -1 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n -1 /fixtures/long.txt"
run_row head "-n -5 (all-but-last-5)"             "gnu-head -n -5 /fixtures/long.txt"                        "corvo /corvo/coreutils/head.corvo -- -n -5 /fixtures/long.txt"
run_row head "-n -30 (none)"                      "gnu-head -n -30 /fixtures/long.txt"                       "corvo /corvo/coreutils/head.corvo -- -n -30 /fixtures/long.txt"
run_row head "-c 1"                               "gnu-head -c 1 /fixtures/a.txt"                            "corvo /corvo/coreutils/head.corvo -- -c 1 /fixtures/a.txt"
run_row head "-c 10"                              "gnu-head -c 10 /fixtures/a.txt"                           "corvo /corvo/coreutils/head.corvo -- -c 10 /fixtures/a.txt"
run_row head "-c 100"                             "gnu-head -c 100 /fixtures/a.txt"                          "corvo /corvo/coreutils/head.corvo -- -c 100 /fixtures/a.txt"
run_row head "-c -5 (all-but-last-5-bytes)"       "gnu-head -c -5 /fixtures/a.txt"                           "corvo /corvo/coreutils/head.corvo -- -c -5 /fixtures/a.txt"
run_row head "two files (auto headers)"           "gnu-head /fixtures/a.txt /fixtures/b.txt"                 "corvo /corvo/coreutils/head.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_row head "three files"                        "gnu-head /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt" "corvo /corvo/coreutils/head.corvo -- /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt"
run_row head "-q quiet multi"                     "gnu-head -q /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/head.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_row head "-v verbose single"                  "gnu-head -v /fixtures/a.txt"                              "corvo /corvo/coreutils/head.corvo -- -v /fixtures/a.txt"
run_row head "-v verbose multi"                   "gnu-head -v /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/head.corvo -- -v /fixtures/a.txt /fixtures/b.txt"
run_row head "-n 3 -v"                            "gnu-head -n 3 -v /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/head.corvo -- -n 3 -v /fixtures/a.txt /fixtures/b.txt"
run_row head "-n 3 -q"                            "gnu-head -n 3 -q /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/head.corvo -- -n 3 -q /fixtures/a.txt /fixtures/b.txt"
run_row head "missing file"                       "gnu-head /fixtures/no_such_file"                          "corvo /corvo/coreutils/head.corvo -- /fixtures/no_such_file"

# ══════════════════════════════════════════════════════════════════════════════
# TAIL
# ══════════════════════════════════════════════════════════════════════════════
run_row tail "default (10 lines)"                 "gnu-tail /fixtures/long.txt"                              "corvo /corvo/coreutils/tail.corvo -- /fixtures/long.txt"
run_row tail "-n 1"                               "gnu-tail -n 1 /fixtures/long.txt"                         "corvo /corvo/coreutils/tail.corvo -- -n 1 /fixtures/long.txt"
run_row tail "-n 5"                               "gnu-tail -n 5 /fixtures/long.txt"                         "corvo /corvo/coreutils/tail.corvo -- -n 5 /fixtures/long.txt"
run_row tail "-n 0"                               "gnu-tail -n 0 /fixtures/long.txt"                         "corvo /corvo/coreutils/tail.corvo -- -n 0 /fixtures/long.txt"
run_row tail "-n 30 (exact)"                      "gnu-tail -n 30 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n 30 /fixtures/long.txt"
run_row tail "-n 50 (exceeds)"                    "gnu-tail -n 50 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n 50 /fixtures/long.txt"
run_row tail "-n +1 (all)"                        "gnu-tail -n +1 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n +1 /fixtures/long.txt"
run_row tail "-n +3 (from line 3)"                "gnu-tail -n +3 /fixtures/long.txt"                        "corvo /corvo/coreutils/tail.corvo -- -n +3 /fixtures/long.txt"
run_row tail "-n +30 (last only)"                 "gnu-tail -n +30 /fixtures/long.txt"                       "corvo /corvo/coreutils/tail.corvo -- -n +30 /fixtures/long.txt"
run_row tail "-n +100 (past end)"                 "gnu-tail -n +100 /fixtures/long.txt"                      "corvo /corvo/coreutils/tail.corvo -- -n +100 /fixtures/long.txt"
run_row tail "-c 10 (bytes)"                      "gnu-tail -c 10 /fixtures/a.txt"                           "corvo /corvo/coreutils/tail.corvo -- -c 10 /fixtures/a.txt"
run_row tail "-c 100 (exceeds)"                   "gnu-tail -c 100 /fixtures/a.txt"                          "corvo /corvo/coreutils/tail.corvo -- -c 100 /fixtures/a.txt"
run_row tail "-c +1 (all bytes)"                  "gnu-tail -c +1 /fixtures/a.txt"                           "corvo /corvo/coreutils/tail.corvo -- -c +1 /fixtures/a.txt"
run_row tail "-c +5 (from byte 5)"                "gnu-tail -c +5 /fixtures/a.txt"                           "corvo /corvo/coreutils/tail.corvo -- -c +5 /fixtures/a.txt"
run_row tail "two files (auto headers)"           "gnu-tail /fixtures/a.txt /fixtures/b.txt"                 "corvo /corvo/coreutils/tail.corvo -- /fixtures/a.txt /fixtures/b.txt"
run_row tail "three files"                        "gnu-tail /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt" "corvo /corvo/coreutils/tail.corvo -- /fixtures/a.txt /fixtures/b.txt /fixtures/blank.txt"
run_row tail "-q quiet multi"                     "gnu-tail -q /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/tail.corvo -- -q /fixtures/a.txt /fixtures/b.txt"
run_row tail "-v verbose single"                  "gnu-tail -v /fixtures/a.txt"                              "corvo /corvo/coreutils/tail.corvo -- -v /fixtures/a.txt"
run_row tail "-v verbose multi"                   "gnu-tail -v /fixtures/a.txt /fixtures/b.txt"              "corvo /corvo/coreutils/tail.corvo -- -v /fixtures/a.txt /fixtures/b.txt"
run_row tail "-n 3 -v"                            "gnu-tail -n 3 -v /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/tail.corvo -- -n 3 -v /fixtures/a.txt /fixtures/b.txt"
run_row tail "-n 3 -q"                            "gnu-tail -n 3 -q /fixtures/a.txt /fixtures/b.txt"         "corvo /corvo/coreutils/tail.corvo -- -n 3 -q /fixtures/a.txt /fixtures/b.txt"
run_row tail "missing file"                       "gnu-tail /fixtures/no_such_file"                          "corvo /corvo/coreutils/tail.corvo -- /fixtures/no_such_file"

# ══════════════════════════════════════════════════════════════════════════════
# CP — stdout/stderr and exit-code comparison; writable /tmp scratch
# ══════════════════════════════════════════════════════════════════════════════
TD="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$TD'" EXIT

# Verbose: both use the same destination so printed path is identical
run_row cp "verbose (-v) single"                  \
  "gnu-cp -v /fixtures/a.txt '$TD/cp_v.txt'"      \
  "corvo /corvo/coreutils/cp.corvo -- -v /fixtures/a.txt '$TD/cp_v.txt'"

# Error cases
run_row cp "missing source"                       \
  "gnu-cp /fixtures/no_such_file /tmp/x_gnu"      \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/no_such_file /tmp/x_corvo"

run_row cp "missing operand"                      \
  "gnu-cp"                                         \
  "corvo /corvo/coreutils/cp.corvo"

run_row cp "dir without -r"                       \
  "gnu-cp /fixtures/sub /tmp/xd_gnu"               \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/sub /tmp/xd_corvo"

run_row cp "multi-source to file (error)"         \
  "gnu-cp /fixtures/a.txt /fixtures/b.txt '$TD/not_a_dir.txt'" \
  "corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt /fixtures/b.txt '$TD/not_a_dir.txt'"

# Content comparison helpers (inline — no sub-function to keep heredoc simple)
rm -rf "$TD/gnu_single" "$TD/corvo_single"
mkdir -p "$TD/gnu_single" "$TD/corvo_single"
gnu-cp /fixtures/a.txt "$TD/gnu_single/"   >/dev/null 2>&1
corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt "$TD/corvo_single/" >/dev/null 2>&1
if diff -r "$TD/gnu_single" "$TD/corvo_single" >/dev/null 2>&1; then
  printf "PASS [cp] basic file copy content\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] basic file copy content\n"; FAIL=$((FAIL+1))
fi

rm -rf "$TD/gnu_multi" "$TD/corvo_multi"
mkdir -p "$TD/gnu_multi" "$TD/corvo_multi"
gnu-cp /fixtures/a.txt /fixtures/b.txt "$TD/gnu_multi/"   >/dev/null 2>&1
corvo /corvo/coreutils/cp.corvo -- /fixtures/a.txt /fixtures/b.txt "$TD/corvo_multi/" >/dev/null 2>&1
if diff -r "$TD/gnu_multi" "$TD/corvo_multi" >/dev/null 2>&1; then
  printf "PASS [cp] two-file copy content\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] two-file copy content\n"; FAIL=$((FAIL+1))
fi

rm -rf "$TD/gnu_rec" "$TD/corvo_rec"
gnu-cp -r /fixtures/sub "$TD/gnu_rec"   >/dev/null 2>&1
corvo /corvo/coreutils/cp.corvo -- -r /fixtures/sub "$TD/corvo_rec" >/dev/null 2>&1
if diff -r "$TD/gnu_rec" "$TD/corvo_rec" >/dev/null 2>&1; then
  printf "PASS [cp] recursive copy content\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] recursive copy content\n"; FAIL=$((FAIL+1))
fi

# -n (no-clobber): second run should NOT overwrite
echo "original" > "$TD/noclobber.txt"
gnu-cp    -n /fixtures/a.txt "$TD/noclobber.txt" >/dev/null 2>&1 || true
if diff <(echo "original") "$TD/noclobber.txt" >/dev/null 2>&1; then
  printf "PASS [cp] no-clobber (-n) gnu behaviour\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] no-clobber (-n) gnu behaviour\n"; FAIL=$((FAIL+1))
fi
echo "original" > "$TD/noclobber2.txt"
corvo /corvo/coreutils/cp.corvo -- -n /fixtures/a.txt "$TD/noclobber2.txt" >/dev/null 2>&1 || true
if diff <(echo "original") "$TD/noclobber2.txt" >/dev/null 2>&1; then
  printf "PASS [cp] no-clobber (-n) corvo behaviour\n"; PASS=$((PASS+1))
else
  printf "FAIL [cp] no-clobber (-n) corvo behaviour\n"; FAIL=$((FAIL+1))
fi

# ══════════════════════════════════════════════════════════════════════════════
# Summary
# ══════════════════════════════════════════════════════════════════════════════
echo ""
echo "======================================================"
echo "  coreutils parity matrix:  PASS=$PASS  FAIL=$FAIL"
echo "======================================================"
exit $(( FAIL > 0 ? 1 : 0 ))
INNER
