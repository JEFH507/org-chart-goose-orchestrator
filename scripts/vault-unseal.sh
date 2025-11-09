#!/bin/bash
# Vault Unseal Script
# Usage: ./scripts/vault-unseal.sh

set -e

echo "=== Vault Unseal Helper ==="
echo ""
echo "This script will unseal Vault using your unseal key."
echo "Your unseal key should be stored securely in your password manager."
echo ""

# Check if Vault is running
if ! docker ps | grep -q ce_vault; then
    echo "❌ Vault container is not running"
    exit 1
fi

# Check Vault status
VAULT_STATUS=$(docker exec ce_vault vault status 2>&1 || true)

if echo "$VAULT_STATUS" | grep -q "Sealed.*false"; then
    echo "✅ Vault is already unsealed"
    docker exec ce_vault vault status
    exit 0
fi

# Vault is sealed, proceed with unseal (need 3 of 5 keys)
echo "🔒 Vault is sealed. Unsealing..."
echo ""
echo "This Vault requires 3 of 5 unseal keys (threshold=3)"
echo ""

# Unseal with 3 keys
for i in 1 2 3; do
    read -sp "Enter unseal key $i of 3: " UNSEAL_KEY
    echo ""
    
    # Trim whitespace
    UNSEAL_KEY=$(echo "$UNSEAL_KEY" | tr -d '[:space:]')
    
    # Unseal Vault
    RESULT=$(docker exec -e VAULT_SKIP_VERIFY=true ce_vault vault operator unseal "$UNSEAL_KEY")
    
    # Check if still sealed
    if echo "$RESULT" | grep -q "Sealed.*false"; then
        echo "✅ Vault unsealed successfully after $i keys!"
        echo ""
        docker exec ce_vault vault status
        exit 0
    else
        PROGRESS=$(echo "$RESULT" | grep "Unseal Progress" | awk '{print $3}')
        echo "   Progress: $PROGRESS"
    fi
done

echo ""
echo "✅ Vault unsealed successfully!"
echo ""
docker exec ce_vault vault status
