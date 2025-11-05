#!/bin/bash
# Get JWT token from Keycloak for Agent Mesh MCP tools
#
# Usage: ./scripts/get-jwt-token.sh
# Output: JWT access token (stdout)
#
# Requirements:
# - Keycloak running at localhost:8080
# - 'dev' realm configured
# - 'goose-controller' client exists
# - OIDC_CLIENT_SECRET set in deploy/compose/.env.ce

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
error() {
    echo -e "${RED}ERROR: $1${NC}" >&2
}

success() {
    echo -e "${GREEN}$1${NC}" >&2
}

warn() {
    echo -e "${YELLOW}WARNING: $1${NC}" >&2
}

# Check if .env.ce exists
ENV_FILE="$(dirname "$0")/../deploy/compose/.env.ce"
if [ ! -f "$ENV_FILE" ]; then
    error ".env.ce file not found at: $ENV_FILE"
    error "Please ensure deploy/compose/.env.ce exists with OIDC_CLIENT_SECRET set"
    exit 1
fi

# Source the .env.ce file to get OIDC_CLIENT_SECRET
source "$ENV_FILE"

# Check if OIDC_CLIENT_SECRET is set
if [ -z "${OIDC_CLIENT_SECRET:-}" ]; then
    error "OIDC_CLIENT_SECRET not set in .env.ce"
    error "Please add: OIDC_CLIENT_SECRET=<your-secret>"
    exit 1
fi

# Keycloak configuration
KEYCLOAK_URL="http://localhost:8080"
REALM="dev"
CLIENT_ID="goose-controller"
USERNAME="dev-agent"
PASSWORD="dev-password"

# Check if Keycloak is running
if ! curl -sf "${KEYCLOAK_URL}/realms/${REALM}" >/dev/null 2>&1; then
    error "Keycloak is not running at ${KEYCLOAK_URL} or 'dev' realm not found"
    error "Please start Keycloak: cd deploy/compose && docker compose -f ce.dev.yml up keycloak"
    exit 1
fi

# Get JWT token using password grant (for testing)
TOKEN_RESPONSE=$(curl -sf -X POST \
    "${KEYCLOAK_URL}/realms/${REALM}/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "username=${USERNAME}" \
    -d "password=${PASSWORD}" \
    -d "grant_type=password" \
    -d "client_id=${CLIENT_ID}" \
    -d "client_secret=${OIDC_CLIENT_SECRET}" 2>&1)

# Check if curl succeeded
if [ $? -ne 0 ]; then
    error "Failed to get token from Keycloak"
    error "Response: $TOKEN_RESPONSE"
    exit 1
fi

# Extract access token
ACCESS_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')

# Check if token extraction succeeded
if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" = "null" ]; then
    error "Failed to extract access token from response"
    error "Response: $TOKEN_RESPONSE"
    exit 1
fi

# Decode token to get expiry (for info only)
EXPIRES_IN=$(echo "$TOKEN_RESPONSE" | jq -r '.expires_in')

# Output token to stdout (for piping)
echo "$ACCESS_TOKEN"

# Output info to stderr (won't interfere with piping)
success "✅ JWT token acquired successfully"
warn "⏰ Token expires in ${EXPIRES_IN} seconds (~$((EXPIRES_IN / 60)) minutes)"
warn "💡 Use: export MESH_JWT_TOKEN=\$(./scripts/get-jwt-token.sh)" >&2
