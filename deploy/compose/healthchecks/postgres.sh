#!/usr/bin/env bash
set -euo pipefail
PGPORT=${POSTGRES_PORT:-5432}
pg_isready -h 127.0.0.1 -p "$PGPORT" -U postgres
