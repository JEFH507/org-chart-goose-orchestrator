#!/bin/sh
# Vault initialization script for dev mode
# Ensures Transit engine is mounted on startup

set -e

VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-root}"

echo "Waiting for Vault to be ready..."
until curl -sf "$VAULT_ADDR/v1/sys/health" > /dev/null 2>&1; do
    sleep 1
done

echo "Vault is ready. Initializing..."

# Enable Transit engine (idempotent)
echo "Enabling Transit engine..."
curl -sf -X POST \
    -H "X-Vault-Token: $VAULT_TOKEN" \
    "$VAULT_ADDR/v1/sys/mounts/transit" \
    -d '{"type":"transit"}' \
    2>/dev/null || echo "Transit engine already enabled"

echo "Vault initialization complete."
