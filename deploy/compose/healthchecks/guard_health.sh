#!/usr/bin/env sh
set -eu

GUARD_URL="${GUARD_URL:-http://localhost:8089}"

# Check status endpoint and verify response contains expected fields
curl -fsS --connect-timeout 2 --max-time 2 "${GUARD_URL}/status" 2>/dev/null | grep -q '"status"' || exit 1
