#!/usr/bin/env bash
set -euo pipefail
curl -fsS http://localhost:${KEYCLOAK_PORT:-8080} > /dev/null
