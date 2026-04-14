#!/usr/bin/env bash
# coreutils/tests/parity/date.sh
# Required parity cases for date.
# Sourced by parity/run-all.sh; requires run_case, run_uutils_case, show_time
# and the PASS / FAIL counters to be in scope.
#
# Note: date output depends on current time and timezone, so we test format
# patterns and specific date parsing rather than exact output.

echo "=== date ==="
cd /fixtures

# Test format patterns with a fixed timestamp
run_case date "epoch seconds"         "gnu-date -d @0 +%Y-%m-%d"                      "corvo /corvo/coreutils/date.corvo -- -d @0 +%Y-%m-%d"
run_case date "epoch full"            "gnu-date -d @0 '+%Y-%m-%d %H:%M:%S' -u"        "corvo /corvo/coreutils/date.corvo -- -d @0 '+%Y-%m-%d %H:%M:%S' -u"
run_case date "year only"             "gnu-date -d @1000000000 +%Y -u"                "corvo /corvo/coreutils/date.corvo -- -d @1000000000 +%Y -u"
run_case date "month day"             "gnu-date -d @1000000000 +%m-%d -u"             "corvo /corvo/coreutils/date.corvo -- -d @1000000000 +%m-%d -u"
run_case date "time only"             "gnu-date -d @1000000000 +%H:%M:%S -u"          "corvo /corvo/coreutils/date.corvo -- -d @1000000000 +%H:%M:%S -u"
run_case date "weekday"               "gnu-date -d @1000000000 +%A -u"                "corvo /corvo/coreutils/date.corvo -- -d @1000000000 +%A -u"
run_case date "iso 8601 date"         "gnu-date -d @1000000000 +%F -u"                "corvo /corvo/coreutils/date.corvo -- -d @1000000000 +%F -u"

run_uutils_case date "epoch"          "uu-date -d @0 +%Y-%m-%d"                       "corvo /corvo/coreutils/date.corvo -- -d @0 +%Y-%m-%d"

show_time "gnu-date"   gnu-date +%Y-%m-%d
show_time "corvo date" corvo /corvo/coreutils/date.corvo -- +%Y-%m-%d
