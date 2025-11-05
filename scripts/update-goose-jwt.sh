#!/bin/bash
# Update Goose profiles.yaml with fresh JWT token
# Usage: ./scripts/update-goose-jwt.sh

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}Goose JWT Token Updater${NC}"
echo -e "${BLUE}============================================================${NC}"

# Source .env.ce to get KEYCLOAK_CLIENT_SECRET
if [ ! -f "deploy/compose/.env.ce" ]; then
    echo -e "${RED}❌ deploy/compose/.env.ce not found${NC}"
    exit 1
fi

source deploy/compose/.env.ce

if [ -z "$KEYCLOAK_CLIENT_SECRET" ]; then
    echo -e "${RED}❌ KEYCLOAK_CLIENT_SECRET not set in .env.ce${NC}"
    exit 1
fi

# Get JWT token from Keycloak
echo -e "${BLUE}ℹ️  Obtaining JWT token from Keycloak...${NC}"

TOKEN_RESPONSE=$(curl -s -X POST "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
    -d "client_id=goose-controller" \
    -d "grant_type=client_credentials" \
    -d "client_secret=$KEYCLOAK_CLIENT_SECRET" 2>&1)

MESH_JWT_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token' 2>/dev/null)

if [ -z "$MESH_JWT_TOKEN" ] || [ "$MESH_JWT_TOKEN" = "null" ]; then
    echo -e "${RED}❌ Failed to obtain JWT token from Keycloak${NC}"
    echo "   Response: $TOKEN_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✅ JWT token obtained${NC}"

# Update profiles.yaml
PROFILES_FILE="$HOME/.config/goose/profiles.yaml"

if [ ! -f "$PROFILES_FILE" ]; then
    echo -e "${YELLOW}⚠️  $PROFILES_FILE not found, creating...${NC}"
    mkdir -p "$(dirname "$PROFILES_FILE")"
    cat > "$PROFILES_FILE" <<EOF
# ~/.config/goose/profiles.yaml
extensions:
  agent_mesh:
    type: mcp
    command: ["python", "-m", "agent_mesh_server"]
    working_dir: "$PWD/src/agent-mesh"
    env:
      CONTROLLER_URL: "http://localhost:8088"
      MESH_JWT_TOKEN: "$MESH_JWT_TOKEN"
EOF
    echo -e "${GREEN}✅ Created $PROFILES_FILE${NC}"
else
    # Update existing file
    if grep -q "MESH_JWT_TOKEN:" "$PROFILES_FILE"; then
        # Replace existing token
        sed -i "s|MESH_JWT_TOKEN:.*|MESH_JWT_TOKEN: \"$MESH_JWT_TOKEN\"|" "$PROFILES_FILE"
        echo -e "${GREEN}✅ Updated JWT token in $PROFILES_FILE${NC}"
    else
        echo -e "${YELLOW}⚠️  MESH_JWT_TOKEN not found in profiles.yaml${NC}"
        echo "   Please manually add the token to your profiles.yaml"
        echo "   Token: $MESH_JWT_TOKEN"
        exit 1
    fi
fi

# Show token expiration
TOKEN_EXP=$(echo "$MESH_JWT_TOKEN" | cut -d. -f2 | base64 -d 2>/dev/null | jq -r '.exp' 2>/dev/null)
if [ -n "$TOKEN_EXP" ] && [ "$TOKEN_EXP" != "null" ]; then
    CURRENT_TIME=$(date +%s)
    TIME_LEFT=$((TOKEN_EXP - CURRENT_TIME))
    MINUTES_LEFT=$((TIME_LEFT / 60))
    echo -e "${BLUE}ℹ️  Token expires in $MINUTES_LEFT minutes${NC}"
fi

echo ""
echo -e "${GREEN}✅ Goose profiles.yaml updated successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Restart Goose Desktop (to reload profiles.yaml)"
echo "  2. Open Settings → Extensions"
echo "  3. Verify agent_mesh extension is loaded"
echo "  4. Test tools in Goose session"
echo ""
echo "Note: JWT tokens expire after 5 minutes by default."
echo "      Re-run this script if you get authentication errors."
