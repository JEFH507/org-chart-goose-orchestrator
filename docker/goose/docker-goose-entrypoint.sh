#!/bin/bash
set -e

# Goose Docker Entrypoint Script
# This script configures Goose without using keyring (not supported in Ubuntu Docker)
# All configuration is done via environment variables

echo "========================================="
echo "Goose Docker Container Starting"
echo "========================================="
echo "Role: ${GOOSE_ROLE}"
echo "Controller URL: ${CONTROLLER_URL}"
echo "Provider: ${GOOSE_PROVIDER}"
echo "Model: ${GOOSE_MODEL}"
echo "========================================="

# Validate required environment variables
if [ -z "$GOOSE_ROLE" ]; then
    echo "ERROR: GOOSE_ROLE environment variable is required"
    echo "Example: GOOSE_ROLE=finance"
    exit 1
fi

if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "ERROR: OPENROUTER_API_KEY environment variable is required"
    echo "Example: OPENROUTER_API_KEY=sk-or-v1-..."
    exit 1
fi

# Wait for Controller to be ready
echo "Waiting for Controller at ${CONTROLLER_URL}..."
MAX_RETRIES=30
RETRY_COUNT=0
until curl -s "${CONTROLLER_URL}/status" > /dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo "ERROR: Controller not available after ${MAX_RETRIES} retries"
        exit 1
    fi
    echo "  Retry ${RETRY_COUNT}/${MAX_RETRIES}..."
    sleep 2
done
echo "✓ Controller is ready"

# Get JWT token from Keycloak (client_credentials grant)
echo "Acquiring JWT token from Keycloak..."
# Use host.docker.internal to access Keycloak on host (via extra_hosts mapping)
# Override Host header to 'localhost:8080' so JWT issuer matches Controller's expectation
KEYCLOAK_URL="${KEYCLOAK_URL:-http://host.docker.internal:8080}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-dev}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET}"

if [ -z "$KEYCLOAK_CLIENT_SECRET" ]; then
    echo "WARNING: KEYCLOAK_CLIENT_SECRET not set, attempting unauthenticated profile fetch"
    JWT_TOKEN=""
else
    # Request JWT with Host header override to ensure correct issuer in JWT
    # This makes Keycloak issue JWT with iss: http://localhost:8080/realms/dev
    TOKEN_RESPONSE=$(curl -s -X POST \
        "${KEYCLOAK_URL}/realms/${KEYCLOAK_REALM}/protocol/openid-connect/token" \
        -H "Host: localhost:8080" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "grant_type=client_credentials" \
        -d "client_id=${KEYCLOAK_CLIENT_ID}" \
        -d "client_secret=${KEYCLOAK_CLIENT_SECRET}")
    
    JWT_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token // empty')
    
    if [ -z "$JWT_TOKEN" ] || [ "$JWT_TOKEN" = "null" ]; then
        echo "ERROR: Failed to get JWT token from Keycloak"
        echo "Response: $TOKEN_RESPONSE"
        exit 1
    fi
    
    echo "✓ JWT token acquired"
    
    # Export JWT token for agent-mesh extension to use
    export MESH_JWT_TOKEN="$JWT_TOKEN"
fi

# Fetch profile from Controller API (with JWT if available)
echo "Fetching profile for role: ${GOOSE_ROLE}..."
if [ -n "$JWT_TOKEN" ]; then
    PROFILE_JSON=$(curl -s -H "Authorization: Bearer $JWT_TOKEN" "${CONTROLLER_URL}/profiles/${GOOSE_ROLE}")
else
    PROFILE_JSON=$(curl -s "${CONTROLLER_URL}/profiles/${GOOSE_ROLE}")
fi

if [ $? -ne 0 ]; then
    echo "ERROR: Failed to fetch profile from Controller"
    exit 1
fi

# Check if profile is valid JSON
if ! echo "$PROFILE_JSON" | jq empty 2>/dev/null; then
    echo "ERROR: Invalid JSON response from Controller"
    echo "Response: $PROFILE_JSON"
    exit 1
fi

echo "✓ Profile fetched successfully"

# DEBUG: Show profile JSON (for troubleshooting)
echo "DEBUG: Profile JSON:"
echo "$PROFILE_JSON" | jq '.' 2>&1 | head -50

# Generate config.yaml from profile
echo "Generating Goose config.yaml..."
mkdir -p ~/.config/goose

# Call Python script to generate config
# Pass actual env var values (not ${VAR} substitution placeholders)
python3 /usr/local/bin/generate-goose-config.py \
    --profile "$PROFILE_JSON" \
    --provider "$GOOSE_PROVIDER" \
    --model "$GOOSE_MODEL" \
    --api-key "$OPENROUTER_API_KEY" \
    --proxy-url "$PRIVACY_GUARD_PROXY_URL" \
    --controller-url "$CONTROLLER_URL" \
    --mesh-jwt-token "$JWT_TOKEN" \
    --output ~/.config/goose/config.yaml

if [ $? -ne 0 ]; then
    echo "ERROR: Failed to generate config.yaml"
    exit 1
fi

echo "✓ config.yaml generated"

# Display generated config (mask API key)
echo "Generated config.yaml:"
cat ~/.config/goose/config.yaml | sed "s/$OPENROUTER_API_KEY/***REDACTED***/g"

# Run goose configure with defaults (skip keyring prompts)
# We pass provider and model as env vars to avoid interactive prompts
echo "Running goose configure (non-interactive)..."
export GOOSE_PROVIDER
export GOOSE_MODEL

# NOTE: goose configure will still prompt for API key
# We select "No" to keyring and pass via env var instead
# This is automated by piping "n" to stdin
echo "n" | goose configure 2>&1 || true

echo "✓ Goose configured"

# Verify Goose installation
echo "Verifying Goose..."
goose --version

# Start Goose session
echo "========================================="
echo "Starting Goose session for role: ${GOOSE_ROLE}"
echo "========================================="

# CRITICAL: Export MESH_JWT_TOKEN and CONTROLLER_URL for MCP extension
# These must be exported right before starting Goose so they're available
# to the agent_mesh MCP server subprocess
export MESH_JWT_TOKEN="$JWT_TOKEN"
export CONTROLLER_URL
echo "✓ Exported MESH_JWT_TOKEN for agent_mesh extension"
echo "✓ Exported CONTROLLER_URL=${CONTROLLER_URL}"

# Start Goose in interactive mode
# The session will use the generated config.yaml
# Note: 'goose session' (without 'start') is the correct command
# Keep container alive: pipe infinite stream to goose session
# This allows the agent to stay running and respond to API calls via agent mesh
tail -f /dev/null | goose session
