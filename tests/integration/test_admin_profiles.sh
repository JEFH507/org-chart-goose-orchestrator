#!/bin/bash
# Phase 5 - Admin Profile Endpoints Test (D7-D9)
# Tests Vault integration for profile signing

set -e

CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
TEST_ROLE="vault-test-finance"

echo "========================================="
echo "Admin Profile Endpoints Test (D7-D9)"
echo "========================================="
echo ""

# Get JWT token (using phase5test - TODO: create proper admin user in dev realm)
echo "Step 0: Get JWT Token"
echo "Note: Using phase5test user (admin user doesn't exist in dev realm yet)"
ADMIN_TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/dev/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=phase5test" \
  -d "password=test123" \
  -d "grant_type=password" \
  -d "client_id=goose-controller" \
  | jq -r '.access_token')

if [[ -z "$ADMIN_TOKEN" || "$ADMIN_TOKEN" == "null" ]]; then
  echo "❌ Failed to get JWT token"
  echo "Tip: Make sure Keycloak is running and phase5test user exists"
  exit 1
fi

echo "✓ JWT token obtained (${ADMIN_TOKEN:0:20}...)"
echo ""

# Test D7: Create Profile
echo "========================================="
echo "Test D7: POST /admin/profiles (Create)"
echo "========================================="

PROFILE_JSON=$(cat <<EOF
{
  "role": "$TEST_ROLE",
  "display_name": "Vault Test - Finance Team",
  "description": "Test profile for Vault signing integration",
  "extensions": {
    "developer": {
      "enabled": true,
      "tools": {
        "developer__shell": true,
        "developer__text_editor": true
      }
    }
  },
  "privacy": {
    "mode": "REDACT",
    "enforcement_level": "STRICT",
    "redaction_rules": [
      {
        "entity_type": "SSN",
        "action": "REDACT",
        "applies_to": ["shell", "text_editor"]
      }
    ],
    "local_only_rules": []
  }
}
EOF
)

CREATE_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/profiles" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$PROFILE_JSON")

CREATE_STATUS=$(echo "$CREATE_RESPONSE" | jq -r '.role // .error')

if [[ "$CREATE_STATUS" == "$TEST_ROLE" ]]; then
  CREATED_AT=$(echo "$CREATE_RESPONSE" | jq -r '.created_at')
  echo "✓ D7 PASS: Profile created successfully"
  echo "  Role: $TEST_ROLE"
  echo "  Created: $CREATED_AT"
else
  echo "❌ D7 FAIL: Profile creation failed"
  echo "  Response: $CREATE_RESPONSE"
  exit 1
fi
echo ""

# Test D8: Update Profile
echo "========================================="
echo "Test D8: PUT /admin/profiles/{role} (Update)"
echo "========================================="

UPDATE_JSON=$(cat <<EOF
{
  "display_name": "Vault Test - Finance Team (Updated)",
  "description": "Updated description for Vault testing"
}
EOF
)

UPDATE_RESPONSE=$(curl -s -X PUT "$CONTROLLER_URL/admin/profiles/$TEST_ROLE" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$UPDATE_JSON")

UPDATE_STATUS=$(echo "$UPDATE_RESPONSE" | jq -r '.role // .error')

if [[ "$UPDATE_STATUS" == "$TEST_ROLE" ]]; then
  UPDATED_AT=$(echo "$UPDATE_RESPONSE" | jq -r '.updated_at')
  echo "✓ D8 PASS: Profile updated successfully"
  echo "  Role: $TEST_ROLE"
  echo "  Updated: $UPDATED_AT"
else
  echo "❌ D8 FAIL: Profile update failed"
  echo "  Response: $UPDATE_RESPONSE"
  exit 1
fi
echo ""

# Verify update was applied
VERIFY_RESPONSE=$(curl -s "$CONTROLLER_URL/profiles/$TEST_ROLE")
VERIFY_NAME=$(echo "$VERIFY_RESPONSE" | jq -r '.display_name')
if [[ "$VERIFY_NAME" == "Vault Test - Finance Team (Updated)" ]]; then
  echo "✓ Update verification: Display name changed correctly"
else
  echo "⚠️  Update verification: Display name mismatch (got: $VERIFY_NAME)"
fi
echo ""

# Test D9: Publish Profile (Vault Signing)
echo "========================================="
echo "Test D9: POST /admin/profiles/{role}/publish (Vault Signing)"
echo "========================================="

# Verify Vault is accessible
VAULT_HEALTH=$(curl -s http://localhost:8200/v1/sys/health | jq -r '.sealed // "error"')
if [[ "$VAULT_HEALTH" == "false" ]]; then
  echo "✓ Vault health check: Unsealed and ready"
else
  echo "❌ Vault health check failed (sealed=$VAULT_HEALTH)"
  exit 1
fi
echo ""

PUBLISH_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/profiles/$TEST_ROLE/publish" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

SIGNATURE=$(echo "$PUBLISH_RESPONSE" | jq -r '.signature // .error')

if [[ "$SIGNATURE" =~ ^vault:v1: ]]; then
  SIGNED_AT=$(echo "$PUBLISH_RESPONSE" | jq -r '.signed_at')
  echo "✓ D9 PASS: Profile signed with Vault successfully"
  echo "  Role: $TEST_ROLE"
  echo "  Signature: ${SIGNATURE:0:40}..."
  echo "  Signed At: $SIGNED_AT"
  echo ""
  
  # Verify signature is stored in profile
  PROFILE_CHECK=$(curl -s "$CONTROLLER_URL/profiles/$TEST_ROLE")
  STORED_SIGNATURE=$(echo "$PROFILE_CHECK" | jq -r '.signature.signature // "none"')
  
  if [[ "$STORED_SIGNATURE" == "$SIGNATURE" ]]; then
    echo "✓ Signature verification: Stored in profile correctly"
    SIGNATURE_ALGO=$(echo "$PROFILE_CHECK" | jq -r '.signature.algorithm')
    SIGNATURE_KEY=$(echo "$PROFILE_CHECK" | jq -r '.signature.vault_key')
    echo "  Algorithm: $SIGNATURE_ALGO"
    echo "  Vault Key: $SIGNATURE_KEY"
  else
    echo "❌ Signature verification: Mismatch between publish response and stored profile"
    exit 1
  fi
else
  echo "❌ D9 FAIL: Profile signing failed"
  echo "  Response: $PUBLISH_RESPONSE"
  exit 1
fi
echo ""

# Test re-publish (signature should change)
echo "========================================="
echo "Test: Re-publish after update (Signature Uniqueness)"
echo "========================================="

# Make another update
curl -s -X PUT "$CONTROLLER_URL/admin/profiles/$TEST_ROLE" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"description": "Second update for re-sign test"}' > /dev/null

# Re-publish
REPUBLISH_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/profiles/$TEST_ROLE/publish" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

NEW_SIGNATURE=$(echo "$REPUBLISH_RESPONSE" | jq -r '.signature')

if [[ "$NEW_SIGNATURE" != "$SIGNATURE" ]]; then
  echo "✓ Re-publish test: Signature changed after update"
  echo "  Original: ${SIGNATURE:0:40}..."
  echo "  New:      ${NEW_SIGNATURE:0:40}..."
else
  echo "❌ Re-publish test: Signature did not change (expected different HMAC)"
  exit 1
fi
echo ""

# Cleanup: Delete test profile
echo "========================================="
echo "Cleanup: Delete test profile"
echo "========================================="

# Note: DELETE endpoint doesn't exist yet, so we'll leave the profile in DB
echo "ℹ️  Test profile '$TEST_ROLE' left in database (no DELETE endpoint yet)"
echo ""

# Summary
echo "========================================="
echo "Test Summary"
echo "========================================="
echo "✓ D7: Create Profile - PASS"
echo "✓ D8: Update Profile - PASS"
echo "✓ D9: Publish (Vault Signing) - PASS"
echo "✓ Signature Verification - PASS"
echo "✓ Re-publish Uniqueness - PASS"
echo ""
echo "All admin profile tests passed! 🎉"
echo "Vault integration is working correctly."
