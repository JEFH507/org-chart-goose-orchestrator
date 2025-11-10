#!/bin/bash
# Sign all 8 profiles with Vault HMAC signatures
# This is a permanent fix - signed profiles persist in database
#
# Usage: ./scripts/sign-all-profiles.sh
#
# Prerequisites:
# - All services running (Controller, Vault, Keycloak)
# - Vault unsealed
# - Database migrations applied (0002, 0006 - profiles table exists)
# - Profiles seeded in database (migration 0006)

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "Signing All 8 Profiles with Vault"
echo "========================================="

# Get JWT token using client_credentials grant
echo "Acquiring JWT token from Keycloak (client_credentials grant)..."

# Use client_credentials grant (service-to-service auth)
# This works immediately without user setup
JWT=$(curl -s -X POST \
    "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=client_credentials" \
    -d "client_id=goose-controller" \
    -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" | jq -r '.access_token')

if [ -z "$JWT" ] || [ "$JWT" = "null" ]; then
    echo -e "${RED}ERROR: Failed to get JWT token${NC}"
    exit 1
fi

echo -e "${GREEN}✓ JWT token acquired (client_credentials)${NC}"

# List of all 8 profiles
PROFILES=("analyst" "developer" "finance" "hr" "legal" "manager" "marketing" "support")

# Controller URL
CONTROLLER_URL="http://localhost:8088"

# Counters
SUCCESS_COUNT=0
FAIL_COUNT=0
ALREADY_SIGNED_COUNT=0

# Sign each profile
for ROLE in "${PROFILES[@]}"; do
    echo ""
    echo "----------------------------------------"
    echo "Signing profile: $ROLE"
    echo "----------------------------------------"
    
    # Check if profile exists
    PROFILE_CHECK=$(curl -s -H "Authorization: Bearer $JWT" \
        "${CONTROLLER_URL}/profiles/${ROLE}" 2>&1)
    
    if echo "$PROFILE_CHECK" | jq -e '.error' > /dev/null 2>&1; then
        # Profile load failed - check if it's a signature error or not found
        ERROR_MSG=$(echo "$PROFILE_CHECK" | jq -r '.error')
        
        if [[ "$ERROR_MSG" == *"not found"* ]]; then
            echo -e "${RED}✗ Profile not found in database${NC}"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            continue
        elif [[ "$ERROR_MSG" == *"signature invalid"* ]] || [[ "$ERROR_MSG" == *"signature missing"* ]]; then
            echo -e "${YELLOW}⚠ Profile exists but needs signing${NC}"
            # Continue to signing step
        else
            echo -e "${RED}✗ Unknown error: $ERROR_MSG${NC}"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            continue
        fi
    else
        # Profile loaded successfully - check if already signed
        HAS_SIGNATURE=$(echo "$PROFILE_CHECK" | jq -r '.signature.signature // empty')
        
        if [ -n "$HAS_SIGNATURE" ]; then
            echo -e "${GREEN}✓ Already signed (signature: ${HAS_SIGNATURE:0:30}...)${NC}"
            ALREADY_SIGNED_COUNT=$((ALREADY_SIGNED_COUNT + 1))
            continue
        fi
    fi
    
    # Sign profile via publish endpoint
    echo "Calling POST /admin/profiles/${ROLE}/publish..."
    
    RESPONSE=$(curl -s -X POST \
        -H "Authorization: Bearer $JWT" \
        "${CONTROLLER_URL}/admin/profiles/${ROLE}/publish" 2>&1)
    
    # Check if successful
    if echo "$RESPONSE" | jq -e '.signature' > /dev/null 2>&1; then
        SIGNATURE=$(echo "$RESPONSE" | jq -r '.signature')
        SIGNED_AT=$(echo "$RESPONSE" | jq -r '.signed_at')
        echo -e "${GREEN}✓ Signed successfully${NC}"
        echo "  Signature: ${SIGNATURE:0:50}..."
        echo "  Signed at: $SIGNED_AT"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        ERROR=$(echo "$RESPONSE" | jq -r '.error // "Unknown error"')
        echo -e "${RED}✗ Signing failed: $ERROR${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done

echo ""
echo "========================================="
echo "Signing Summary"
echo "========================================="
echo -e "${GREEN}Successfully signed:  $SUCCESS_COUNT${NC}"
echo -e "${YELLOW}Already signed:       $ALREADY_SIGNED_COUNT${NC}"
echo -e "${RED}Failed:               $FAIL_COUNT${NC}"
echo "Total profiles:       ${#PROFILES[@]}"
echo ""

# Verify all profiles now have signatures
echo "Verifying all profiles in database..."
UNSIGNED_COUNT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c \
    "SELECT COUNT(*) FROM profiles WHERE signature IS NULL OR signature = 'null'::jsonb;" | tr -d ' ')

echo "Profiles without signatures: $UNSIGNED_COUNT"

if [ "$UNSIGNED_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓ All profiles signed successfully!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ $UNSIGNED_COUNT profiles still unsigned${NC}"
    echo "Run this script again or check Controller logs for errors"
    exit 1
fi
