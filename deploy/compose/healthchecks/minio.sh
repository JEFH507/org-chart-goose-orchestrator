#!/usr/bin/env bash
set -euo pipefail
curl -fsS http://localhost:${MINIO_API_PORT:-9000}/minio/health/ready > /dev/null
