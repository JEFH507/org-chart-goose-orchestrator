#!/usr/bin/env bash
set -euo pipefail

# Simple port preflight; suggest alternatives if busy.
PORTS=(
  "Keycloak:8080"
  "Vault:8200"
  "Postgres:5432"
  "Ollama:11434"
  "MinIO S3 (opt):9000"
  "MinIO Console (opt):9001"
  "Seaweed S3 (opt):8333"
  "Seaweed Master UI (opt):9333"
  "Seaweed Filer API (opt):8081"
)

busy=0
for entry in "${PORTS[@]}"; do
  name=${entry%%:*}
  port=${entry##*:}
  if ss -lnt 2>/dev/null | awk '{print $4}' | grep -q ":${port}$"; then
    echo "[!] Port ${port} (${name}) appears to be in use"
    busy=1
  fi
done

if [[ $busy -eq 1 ]]; then
  echo "\nOne or more ports are busy. Consider overriding in deploy/compose/.env.ce (e.g., KEYCLOAK_PORT=18080)."
  exit 1
else
  echo "All default ports appear available."
fi
