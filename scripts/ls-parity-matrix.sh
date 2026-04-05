#!/usr/bin/env bash
# Compare GNU `ls` vs `corvo coreutils/ls.corvo` for many flag combinations (Docker).
# Prints PASS/FAIL per case; exits 1 if any case differs (stdout or exit code).
#
#   ./scripts/ls-parity-matrix.sh
#   ./scripts/ls-parity-matrix.sh --require-docker
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REQUIRE_DOCKER=0
for arg in "$@"; do
  if [[ "$arg" == "--require-docker" ]]; then
    REQUIRE_DOCKER=1
  fi
done

if ! command -v docker >/dev/null 2>&1; then
  if [[ "$REQUIRE_DOCKER" -eq 1 ]]; then
    echo "error: docker is required" >&2
    exit 1
  fi
  echo "skip: docker not installed"
  exit 0
fi

docker build -f "$ROOT/coreutils/tests/ls-parity/Dockerfile" -t corvo-ls-parity "$ROOT" >/dev/null

FIX="$(mktemp -d)"
trap 'rm -rf "$FIX"' EXIT

mkdir -p "$FIX/sub/inner" "$FIX/nested/deep"
echo "hello" >"$FIX/a.txt"
echo "longer-name" >"$FIX/b.txt"
echo "hidden" >"$FIX/.dotfile"
echo "backup" >"$FIX/c.txt~"
printf "x" >"$FIX/zero"
ln -s a.txt "$FIX/link-a" 2>/dev/null || true
ln -s missing "$FIX/broken-link" 2>/dev/null || true
echo "inner" >"$FIX/sub/inner/z.txt"

docker run --rm -i \
  -e LC_ALL=C \
  -e TZ=UTC \
  -v "$FIX:/fixtures:ro" \
  corvo-ls-parity bash -s << 'DOCKER_SCRIPT'
set -euo pipefail
cd /fixtures

PASS=0
FAIL=0

printf "%-32s %s\n" "CASE" "RESULT"
printf "%-32s %s\n" "--------------------------------" "--------"

run_row() {
  local label="$1"
  shift
  local gnu_ec=0 corvo_ec=0
  ls "$@" > /tmp/g.gnu.out 2>/tmp/g.gnu.err || gnu_ec=$?
  corvo /corvo/coreutils/ls.corvo -- "$@" > /tmp/g.corvo.out 2>/tmp/g.corvo.err || corvo_ec=$?
  if [[ "$gnu_ec" != "$corvo_ec" ]]; then
    printf "%-32s FAIL (exit %s vs %s)\n" "$label" "$gnu_ec" "$corvo_ec"
    FAIL=$((FAIL + 1))
    return
  fi
  if ! diff -q /tmp/g.gnu.out /tmp/g.corvo.out >/dev/null 2>&1; then
    printf "%-32s FAIL (stdout)\n" "$label"
    diff -u /tmp/g.gnu.out /tmp/g.corvo.out | head -25 || true
    FAIL=$((FAIL + 1))
    return
  fi
  printf "%-32s PASS\n" "$label"
  PASS=$((PASS + 1))
}

# --- cases: keep in sync with GNU ls flags you care about; LC_ALL=C TZ=UTC ---
run_row "one column" -1
run_row "almost all (-A)" -1 -A
run_row "all (-a)" -1 -a
run_row "inode short" -1 -i
run_row "inode long" -1 -l --time-style=long-iso -i
run_row "blocks" -1 -s
run_row "blocks long" -1 -s -l --time-style=long-iso
run_row "human long" -1 -h -l --time-style=long-iso
run_row "reverse" -1 -r
run_row "reverse long" -1 -l --time-style=long-iso -r
run_row "classify (-F)" -1 -F
run_row "long -A" -1 -A -l --time-style=long-iso
run_row "numeric ids (-n)" -1 -l --time-style=long-iso -n
run_row "no group (-G)" -1 -l --time-style=long-iso -G
run_row "comma (-m)" -m
run_row "columns (-C)" -C
run_row "width 40" -1 -w 40
run_row "directory only (-d)" -d
run_row "recursive (-R)" -R
run_row "sort size (-S)" -1 -l --time-style=long-iso -S
run_row "sort time (-t)" -1 -l --time-style=long-iso -t
run_row "sort none (-U)" -U -1
run_row "sort version (-v)" -1 -v
run_row "sort extension (-X)" -1 -X
run_row "hide backup tilde" --hide='*~' -1
run_row "ignore *.txt" -1 -I '*.txt'
run_row "color never long" -1 --color=never -l --time-style=long-iso
run_row "si human" -1 -l --time-style=long-iso --si -h
run_row "time access (-u)" -1 -l --time-style=long-iso -u
run_row "time status (-c)" -1 -l --time-style=long-iso -c
run_row "literal (-N)" -1 -N
run_row "hyperlink no" --hyperlink=no -1

echo ""
echo "Summary: PASS=$PASS FAIL=$FAIL"
exit $(( FAIL > 0 ? 1 : 0 ))
DOCKER_SCRIPT
