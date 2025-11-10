#!/bin/bash
# Vault AppRole Setup Script
# Phase 6 Task A2 - Configure AppRole authentication for Controller
#
# Prerequisites:
# - Vault is running and unsealed
# - You have the root token in your password manager
#
# Usage: ./scripts/vault-setup-approle.sh

set -e

VAULT_ADDR="${VAULT_ADDR:-https://localhost:8200}"
POLICY_FILE="deploy/vault/policies/controller-policy.hcl"

echo "═══════════════════════════════════════════════════════════"
echo "   Vault AppRole Setup for Controller"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Check if policy file exists
if [ ! -f "$POLICY_FILE" ]; then
    echo "❌ Policy file not found: $POLICY_FILE"
    echo "Run this script from the project root directory."
    exit 1
fi

# Prompt for root token (user retrieves from password manager)
echo "🔐 Authentication Required"
echo ""
echo "Please retrieve your Vault root token from your password manager."
read -sp "Vault Root Token: " VAULT_TOKEN_RAW
echo ""
echo ""

# Trim whitespace and newlines
VAULT_TOKEN=$(echo "$VAULT_TOKEN_RAW" | tr -d '[:space:]')

if [ -z "$VAULT_TOKEN" ]; then
    echo "❌ Root token is required"
    exit 1
fi

export VAULT_TOKEN
export VAULT_SKIP_VERIFY=true  # For self-signed cert

echo "✅ Token received (length: ${#VAULT_TOKEN} characters)"
echo ""

echo "Step 1: Creating controller policy..."
docker exec -e VAULT_TOKEN -e VAULT_SKIP_VERIFY ce_vault \
    vault policy write controller-policy /vault/policies/controller-policy.hcl || {
    echo "❌ Failed to create policy"
    echo "Make sure Vault is running and the policy file is mounted"
    exit 1
}
echo "✅ Policy created"
echo ""

echo "Step 2: Enabling AppRole authentication..."
docker exec -e VAULT_TOKEN -e VAULT_SKIP_VERIFY ce_vault \
    vault auth enable approle 2>/dev/null || echo "ℹ️  AppRole already enabled"
echo "✅ AppRole enabled"
echo ""

echo "Step 3: Creating controller-role..."
docker exec -e VAULT_TOKEN -e VAULT_SKIP_VERIFY ce_vault \
    vault write auth/approle/role/controller-role \
    token_policies="controller-policy" \
    token_ttl=1h \
    token_max_ttl=4h \
    bind_secret_id=true
echo "✅ Role created"
echo ""

echo "Step 4: Generating credentials..."
echo ""

ROLE_ID=$(docker exec -e VAULT_TOKEN -e VAULT_SKIP_VERIFY ce_vault \
    vault read -field=role_id auth/approle/role/controller-role/role-id)

SECRET_ID=$(docker exec -e VAULT_TOKEN -e VAULT_SKIP_VERIFY ce_vault \
    vault write -field=secret_id -f auth/approle/role/controller-role/secret-id)

echo "═══════════════════════════════════════════════════════════"
echo "   AppRole Credentials Generated"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "🔑 ROLE_ID (static):"
echo "$ROLE_ID"
echo ""
echo "🔑 SECRET_ID (rotatable):"
echo "$SECRET_ID"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "⚠️  IMPORTANT: Save these credentials securely!"
echo ""
echo "1. Add to your password manager as separate entries:"
echo "   - Name: 'Vault AppRole - Role ID (controller)'"
echo "   - Name: 'Vault AppRole - Secret ID (controller)'"
echo ""
echo "2. Add to deploy/compose/.env (DO NOT COMMIT!):"
echo ""
echo "   VAULT_ROLE_ID=$ROLE_ID"
echo "   VAULT_SECRET_ID=$SECRET_ID"
echo ""
echo "3. Remove VAULT_TOKEN from .env (no longer needed)"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "✅ AppRole setup complete!"
echo ""
echo "Next steps:"
echo "1. Save credentials to password manager"
echo "2. Update deploy/compose/.env with credentials above"
echo "3. Rebuild controller with AppRole support"
