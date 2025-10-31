#!/usr/bin/env bash
set -euo pipefail

SPEC=${1:-docs/api/controller/openapi.yaml}

if command -v npx >/dev/null 2>&1; then
  npx --yes @stoplight/spectral-cli lint -v -f stylish -r .spectral.yaml "$SPEC" || true
  echo "(warn-only in Phase 0)"
else
  echo "spectral not available. Install Node.js and run: npx @stoplight/spectral-cli lint -r .spectral.yaml $SPEC"
fi
