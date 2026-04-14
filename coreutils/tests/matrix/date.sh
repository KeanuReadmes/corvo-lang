#!/usr/bin/env bash
# coreutils/tests/matrix/date.sh
# Extended flag-combination matrix for date.

echo "=== date matrix ==="
cd /fixtures

# Use a fixed timestamp for reproducibility
run_case date "matrix: %Y"            "gnu-date -d @0 +%Y -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%Y -u"
run_case date "matrix: %m"            "gnu-date -d @0 +%m -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%m -u"
run_case date "matrix: %d"            "gnu-date -d @0 +%d -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%d -u"
run_case date "matrix: %H"            "gnu-date -d @0 +%H -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%H -u"
run_case date "matrix: %M"            "gnu-date -d @0 +%M -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%M -u"
run_case date "matrix: %S"            "gnu-date -d @0 +%S -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%S -u"
run_case date "matrix: %F"            "gnu-date -d @0 +%F -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%F -u"
run_case date "matrix: %T"            "gnu-date -d @0 +%T -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%T -u"
run_case date "matrix: %s"            "gnu-date -d @0 +%s -u"                         "corvo /corvo/coreutils/date.corvo -- -d @0 +%s -u"
