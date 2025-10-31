#!/usr/bin/env bash
set -euo pipefail
curl -fsS http://localhost:8088/status | jq -e '.status=="ok"' >/dev/null
