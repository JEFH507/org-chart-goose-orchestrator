#!/usr/bin/env sh
set -euo pipefail
curl -fsS --connect-timeout 2 --max-time 2 http://localhost:8088/status >/dev/null
