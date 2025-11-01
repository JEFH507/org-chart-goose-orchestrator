#!/usr/bin/env bash
set -euo pipefail

# Idempotent Vault dev bootstrap: ensure KV v2 at secret/ and a minimal read/list policy
VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-root}"
POLICY_NAME="${VAULT_POLICY_NAME:-goose-dev-read}"

hdr=( -H "X-Vault-Token: $VAULT_TOKEN" -H "Content-Type: application/json" )

# Health check
code=$(curl -s -o /dev/null -w "%{http_code}" "$VAULT_ADDR/v1/sys/health")
echo "[vault_bootstrap] sys/health HTTP $code"

# Mounts: ensure secret/ exists and is kv v2
mounts=$(curl -s "${hdr[@]}" "$VAULT_ADDR/v1/sys/mounts")
if echo "$mounts" | grep -q '"secret/"'; then
  echo "[vault_bootstrap] 'secret/' mount exists."
else
  echo "[vault_bootstrap] Enabling kv v2 at 'secret/'..."
  curl -s -X POST "${hdr[@]}" -d '{"type":"kv","options":{"version":"2"}}' "$VAULT_ADDR/v1/sys/mounts/secret" >/dev/null
fi

# Policy: read/list for secret v2 paths
read -r -d '' policy <<'POL'
path "secret/data/*" {
  capabilities = ["read", "list"]
}
path "secret/metadata/*" {
  capabilities = ["read", "list"]
}
POL

echo "[vault_bootstrap] Writing/ensuring policy '$POLICY_NAME'..."
curl -s -X PUT "${hdr[@]}" -d "{\"policy\": $(jq -Rs . <<< "$policy") }" "$VAULT_ADDR/v1/sys/policies/acl/$POLICY_NAME" >/dev/null || {
  # Fallback without jq for minimal environments
  pdata=$(printf '{"policy":"%s"}' "$(printf %s "$policy" | sed 's/"/\\"/g')")
  curl -s -X PUT "${hdr[@]}" -d "$pdata" "$VAULT_ADDR/v1/sys/policies/acl/$POLICY_NAME" >/dev/null
}

echo "[vault_bootstrap] Done. No secrets were written."
