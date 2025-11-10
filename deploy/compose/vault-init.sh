#!/bin/sh
# Vault initialization script - Phase 6 (TLS enabled)
# Ensures Transit engine is mounted on startup

set -e

VAULT_ADDR="${VAULT_ADDR:-https://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-root}"
CURL_OPTS=""

# Skip TLS verification for self-signed certs in dev
if [ "${VAULT_SKIP_VERIFY}" = "true" ]; then
    CURL_OPTS="-k"
fi

echo "Waiting for Vault to be ready (HTTPS enabled)..."
until curl -sf $CURL_OPTS "$VAULT_ADDR/v1/sys/health" > /dev/null 2>&1; do
    sleep 1
done

echo "Vault is ready. Initializing..."

# Enable Transit engine (idempotent)
echo "Enabling Transit engine..."
curl -sf $CURL_OPTS -X POST \
    -H "X-Vault-Token: $VAULT_TOKEN" \
    "$VAULT_ADDR/v1/sys/mounts/transit" \
    -d '{"type":"transit"}' \
    2>/dev/null || echo "Transit engine already enabled"

echo "Vault initialization complete."
