#!/usr/bin/env bash
set -euo pipefail
curl -fsS http://localhost:${OLLAMA_PORT:-11434}/api/tags > /dev/null
