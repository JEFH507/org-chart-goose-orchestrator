#!/bin/bash
# Setup Keycloak 'dev' Realm for goose-org-twin
# This script creates the dev realm, client, and test user for OIDC authentication

set -e

KEYCLOAK_URL=${KEYCLOAK_URL:-http://localhost:8080}
ADMIN_USER=${KEYCLOAK_ADMIN:-admin}
ADMIN_PASS=${KEYCLOAK_ADMIN_PASSWORD:-admin}

echo "=========================================="
echo "Keycloak Dev Realm Setup"
echo "=========================================="
echo ""
echo "Target: $KEYCLOAK_URL"
echo "Admin user: $ADMIN_USER"
echo ""

# Step 1: Get admin access token
echo "Step 1: Authenticating as admin..."
ADMIN_TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=$ADMIN_USER" \
  -d "password=$ADMIN_PASS" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" | jq -r '.access_token')

if [ -z "$ADMIN_TOKEN" ] || [ "$ADMIN_TOKEN" = "null" ]; then
  echo "❌ Failed to authenticate as admin"
  echo "   Check KEYCLOAK_ADMIN and KEYCLOAK_ADMIN_PASSWORD environment variables"
  exit 1
fi

echo "✅ Admin authenticated"
echo ""

# Step 2: Check if dev realm already exists
echo "Step 2: Checking if 'dev' realm exists..."
REALM_EXISTS=$(curl -s -o /dev/null -w "%{http_code}" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  "$KEYCLOAK_URL/admin/realms/dev")

if [ "$REALM_EXISTS" = "200" ]; then
  echo "⚠️  Dev realm already exists"
  echo "   To recreate, delete it first in Keycloak admin console"
  echo "   Or run: curl -X DELETE -H \"Authorization: Bearer $ADMIN_TOKEN\" $KEYCLOAK_URL/admin/realms/dev"
  echo ""
  echo "Proceeding with client and user setup..."
else
  echo "Creating 'dev' realm..."
  
  # Create dev realm
  curl -s -X POST "$KEYCLOAK_URL/admin/realms" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "realm": "dev",
      "enabled": true,
      "displayName": "Development Realm",
      "accessTokenLifespan": 3600,
      "ssoSessionIdleTimeout": 1800,
      "ssoSessionMaxLifespan": 36000,
      "loginTheme": "keycloak"
    }'
  
  echo "✅ Dev realm created"
fi
echo ""

# Step 3: Create goose-controller client
echo "Step 3: Creating 'goose-controller' client..."

# Check if client exists
CLIENT_ID=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
  "$KEYCLOAK_URL/admin/realms/dev/clients?clientId=goose-controller" | jq -r '.[0].id // empty')

if [ -n "$CLIENT_ID" ]; then
  echo "⚠️  Client 'goose-controller' already exists (ID: $CLIENT_ID)"
  echo "   Updating client configuration..."
  
  curl -s -X PUT "$KEYCLOAK_URL/admin/realms/dev/clients/$CLIENT_ID" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "clientId": "goose-controller",
      "enabled": true,
      "protocol": "openid-connect",
      "publicClient": false,
      "serviceAccountsEnabled": true,
      "directAccessGrantsEnabled": true,
      "standardFlowEnabled": true,
      "implicitFlowEnabled": false,
      "attributes": {
        "access.token.lifespan": "3600"
      }
    }'
  
  echo "✅ Client updated"
else
  curl -s -X POST "$KEYCLOAK_URL/admin/realms/dev/clients" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "clientId": "goose-controller",
      "enabled": true,
      "protocol": "openid-connect",
      "publicClient": false,
      "serviceAccountsEnabled": true,
      "directAccessGrantsEnabled": true,
      "standardFlowEnabled": true,
      "implicitFlowEnabled": false,
      "attributes": {
        "access.token.lifespan": "3600"
      }
    }'
  
  echo "✅ Client created"
  
  # Get the newly created client ID
  CLIENT_ID=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
    "$KEYCLOAK_URL/admin/realms/dev/clients?clientId=goose-controller" | jq -r '.[0].id')
fi
echo ""

# Step 4: Get client secret
echo "Step 4: Retrieving client secret..."
CLIENT_SECRET=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
  "$KEYCLOAK_URL/admin/realms/dev/clients/$CLIENT_ID/client-secret" | jq -r '.value')

if [ -z "$CLIENT_SECRET" ] || [ "$CLIENT_SECRET" = "null" ]; then
  echo "⚠️  Client secret not found, regenerating..."
  curl -s -X POST "$KEYCLOAK_URL/admin/realms/dev/clients/$CLIENT_ID/client-secret" \
    -H "Authorization: Bearer $ADMIN_TOKEN"
  
  CLIENT_SECRET=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
    "$KEYCLOAK_URL/admin/realms/dev/clients/$CLIENT_ID/client-secret" | jq -r '.value')
fi

echo "✅ Client secret: $CLIENT_SECRET"
echo ""

# Step 5: Create test user
echo "Step 5: Creating test user 'dev-agent'..."

# Check if user exists
USER_ID=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
  "$KEYCLOAK_URL/admin/realms/dev/users?username=dev-agent" | jq -r '.[0].id // empty')

if [ -n "$USER_ID" ]; then
  echo "⚠️  User 'dev-agent' already exists (ID: $USER_ID)"
else
  curl -s -X POST "$KEYCLOAK_URL/admin/realms/dev/users" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "username": "dev-agent",
      "enabled": true,
      "email": "dev-agent@goose-org-twin.local",
      "emailVerified": true,
      "firstName": "Dev",
      "lastName": "Agent",
      "credentials": [{
        "type": "password",
        "value": "dev-password",
        "temporary": false
      }]
    }'
  
  echo "✅ User created (username: dev-agent, password: dev-password)"
  
  # Get the newly created user ID
  USER_ID=$(curl -s -H "Authorization: Bearer $ADMIN_TOKEN" \
    "$KEYCLOAK_URL/admin/realms/dev/users?username=dev-agent" | jq -r '.[0].id')
fi
echo ""

# Step 6: Verify OIDC endpoints
echo "Step 6: Verifying OIDC endpoints..."
ISSUER=$(curl -s "$KEYCLOAK_URL/realms/dev/.well-known/openid-configuration" | jq -r '.issuer')
JWKS_URI=$(curl -s "$KEYCLOAK_URL/realms/dev/.well-known/openid-configuration" | jq -r '.jwks_uri')

if [ "$ISSUER" = "http://localhost:8080/realms/dev" ]; then
  echo "✅ OIDC Issuer: $ISSUER"
  echo "✅ JWKS URI: $JWKS_URI"
else
  echo "❌ OIDC endpoints not accessible"
  exit 1
fi
echo ""

# Step 7: Test token acquisition
echo "Step 7: Testing JWT token acquisition..."
TEST_TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/dev/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=dev-agent" \
  -d "password=dev-password" \
  -d "grant_type=password" \
  -d "client_id=goose-controller" \
  -d "client_secret=$CLIENT_SECRET" | jq -r '.access_token')

if [ -z "$TEST_TOKEN" ] || [ "$TEST_TOKEN" = "null" ]; then
  echo "❌ Failed to acquire test token"
  echo "   Check client configuration and user credentials"
else
  echo "✅ Test token acquired successfully"
  echo ""
  echo "   Token preview (first 50 chars):"
  echo "   ${TEST_TOKEN:0:50}..."
fi
echo ""

# Step 8: Summary
echo "=========================================="
echo "✅ Setup Complete!"
echo "=========================================="
echo ""
echo "OIDC Configuration for .env.ce:"
echo "----------------------------------------"
echo "OIDC_ISSUER_URL=http://keycloak:8080/realms/dev"
echo "OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs"
echo "OIDC_AUDIENCE=goose-controller"
echo ""
echo "Client Credentials:"
echo "----------------------------------------"
echo "Client ID: goose-controller"
echo "Client Secret: $CLIENT_SECRET"
echo ""
echo "Test User Credentials:"
echo "----------------------------------------"
echo "Username: dev-agent"
echo "Password: dev-password"
echo "Email: dev-agent@goose-org-twin.local"
echo ""
echo "Next Steps:"
echo "----------------------------------------"
echo "1. Add client secret to .env.ce:"
echo "   OIDC_CLIENT_SECRET=$CLIENT_SECRET"
echo ""
echo "2. Restart the controller:"
echo "   docker compose -f deploy/compose/ce.dev.yml restart controller"
echo ""
echo "3. Verify JWT enforcement:"
echo "   docker logs ce_controller 2>&1 | grep JWT"
echo "   # Should see: \"JWT verification enabled\""
echo ""
echo "4. Test authentication:"
echo "   # Get token:"
echo "   TOKEN=\$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \\"
echo "     -d 'username=dev-agent' \\"
echo "     -d 'password=dev-password' \\"
echo "     -d 'grant_type=password' \\"
echo "     -d 'client_id=goose-controller' \\"
echo "     -d 'client_secret=$CLIENT_SECRET' | jq -r '.access_token')"
echo ""
echo "   # Test Controller API:"
echo "   curl -X POST http://localhost:8088/tasks/route \\"
echo "     -H \"Authorization: Bearer \$TOKEN\" \\"
echo "     -H \"Content-Type: application/json\" \\"
echo "     -H \"Idempotency-Key: \$(uuidgen)\" \\"
echo "     -d '{\"target\":\"manager\",\"task\":{\"task_type\":\"test\",\"description\":\"test\"},\"context\":{}}'"
echo ""
echo "=========================================="
