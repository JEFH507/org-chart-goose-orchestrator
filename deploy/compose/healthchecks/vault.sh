#!/usr/bin/env bash
set -euo pipefail
curl -fsS http://localhost:${VAULT_PORT:-8200}/v1/sys/health > /dev/null
