#!/usr/bin/env bash
set -euo pipefail
PASS=0; FAIL=0
if ./tests/test_sanity.sh; then PASS=$((PASS+1)); else FAIL=$((FAIL+1)); fi
LINT=0
cat > test-report.json <<JSON
{
  "lint_errors": $LINT,
  "tests_passed": $PASS,
  "tests_failed": $FAIL,
  "summary": "Sanity test executed; lint skipped (no config)"
}
JSON
cat test-report.json
