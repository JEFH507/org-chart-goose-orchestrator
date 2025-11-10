#!/usr/bin/env bash
# Complete Keycloak seeding with client_credentials grant support
# Idempotent: safe to run multiple times

set -euo pipefail

# Configuration
KC_CONTAINER="${KEYCLOAK_CONTAINER:-ce_keycloak}"
KC_ADMIN_USER="${KEYCLOAK_ADMIN:-admin}"
KC_ADMIN_PASS="${KEYCLOAK_ADMIN_PASSWORD:-admin}"
REALM="${KEYCLOAK_REALM:-dev}"
CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8}"
ROLES=(orchestrator auditor)

# Check container running
if ! docker ps --format '{{.Names}}' | grep -qx "$KC_CONTAINER"; then
  echo "[keycloak_seed] ERROR: Container $KC_CONTAINER not running." >&2
  exit 1
fi

KCADM="/opt/keycloak/bin/kcadm.sh"

exec_kc() {
  docker exec "$KC_CONTAINER" bash -c "$*"
}

echo "[keycloak_seed] Starting complete Keycloak setup..."

# Login to master realm
exec_kc "$KCADM config credentials --server http://localhost:8080 --realm master --user $KC_ADMIN_USER --password $KC_ADMIN_PASS" > /dev/null 2>&1

# Create realm
if exec_kc "$KCADM get realms/$REALM" > /dev/null 2>&1; then
  echo "[keycloak_seed] ✓ Realm '$REALM' exists"
else
  echo "[keycloak_seed] Creating realm '$REALM'..."
  exec_kc "$KCADM create realms -s realm=$REALM -s enabled=true"
fi

# Get or create client
if CLIENT_DATA=$(exec_kc "$KCADM get clients -r $REALM -q clientId=$CLIENT_ID" 2>/dev/null) && echo "$CLIENT_DATA" | grep -q '"clientId"'; then
  echo "[keycloak_seed] ✓ Client '$CLIENT_ID' exists"
  CLIENT_UUID=$(echo "$CLIENT_DATA" | grep '"id"' | head -1 | awk -F'"' '{print $4}')
else
  echo "[keycloak_seed] Creating client '$CLIENT_ID'..."
  CLIENT_UUID=$(exec_kc "$KCADM create clients -r $REALM \
    -s clientId=$CLIENT_ID \
    -s protocol=openid-connect \
    -s enabled=true \
    -s publicClient=false \
    -s serviceAccountsEnabled=true \
    -s directAccessGrantsEnabled=true \
    -s secret=$CLIENT_SECRET \
    -s 'redirectUris=[\"http://localhost/*\"]' -i")
fi

# Update client to ensure correct configuration
echo "[keycloak_seed] Configuring client for service account auth..."
exec_kc "$KCADM update clients/$CLIENT_UUID -r $REALM \
  -s publicClient=false \
  -s serviceAccountsEnabled=true \
  -s directAccessGrantsEnabled=true \
  -s secret=$CLIENT_SECRET" > /dev/null 2>&1

# Add audience mapper (critical for JWT validation)
echo "[keycloak_seed] Adding audience mapper..."
if exec_kc "$KCADM get clients/$CLIENT_UUID/protocol-mappers/models -r $REALM" | grep -q "audience-mapper"; then
  echo "[keycloak_seed] ✓ Audience mapper exists"
else
  exec_kc "$KCADM create clients/$CLIENT_UUID/protocol-mappers/models -r $REALM \
    -s name=audience-mapper \
    -s protocol=openid-connect \
    -s protocolMapper=oidc-audience-mapper \
    -s 'config.\"included.client.audience\"=$CLIENT_ID' \
    -s 'config.\"access.token.claim\"=true'" > /dev/null 2>&1
  echo "[keycloak_seed] ✓ Audience mapper created"
fi

# Create roles
for role in "${ROLES[@]}"; do
  if exec_kc "$KCADM get roles -r $REALM" | grep -q "\"name\".*:.*\"$role\""; then
    echo "[keycloak_seed] ✓ Role '$role' exists"
  else
    echo "[keycloak_seed] Creating role '$role'..."
    exec_kc "$KCADM create roles -r $REALM -s name=$role -s 'description=$role role for dev'"
  fi
done

# Create test user for password grant (optional)
TEST_USER="dev-agent"
TEST_PASS="dev-password"

if exec_kc "$KCADM get users -r $REALM -q username=$TEST_USER" | grep -q '"username"'; then
  echo "[keycloak_seed] ✓ User '$TEST_USER' exists"
else
  echo "[keycloak_seed] Creating user '$TEST_USER'..."
  USER_ID=$(exec_kc "$KCADM create users -r $REALM \
    -s username=$TEST_USER \
    -s enabled=true \
    -s emailVerified=true \
    -s 'requiredActions=[]' \
    -s 'credentials=[{\"type\":\"password\",\"value\":\"$TEST_PASS\",\"temporary\":false}]' -i")
  
  # Assign roles to user
  for role in "${ROLES[@]}"; do
    exec_kc "$KCADM add-roles -r $REALM --uusername $TEST_USER --rolename $role" > /dev/null 2>&1 || true
  done
fi

echo ""
echo "[keycloak_seed] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "[keycloak_seed] ✅ Keycloak Setup Complete"
echo "[keycloak_seed] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Realm:              $REALM"
echo "Client ID:          $CLIENT_ID"
echo "Client Secret:      $CLIENT_SECRET"
echo "Service Account:    Enabled ✓"
echo "Password Grant:     Enabled ✓"
echo "Audience Mapper:    goose-controller ✓"
echo ""
echo "Test User:          $TEST_USER (password: $TEST_PASS)"
echo "Roles:              ${ROLES[*]}"
echo ""
echo "Endpoints:"
echo "  Token:  http://localhost:8080/realms/$REALM/protocol/openid-connect/token"
echo "  JWKS:   http://localhost:8080/realms/$REALM/protocol/openid-connect/certs"
echo ""
echo "Test client_credentials grant:"
echo "  curl -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \\"
echo "    -d 'grant_type=client_credentials' \\"
echo "    -d 'client_id=goose-controller' \\"
echo "    -d 'client_secret=$CLIENT_SECRET' | jq -r '.access_token'"
echo ""
