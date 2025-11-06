#!/bin/bash
# Phase 5 Workstream H - Profile Smoke Tests (All 6 Profiles)
# Quick validation: Load profile → Check schema structure → Detect potential issues

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Service URLs
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-dev}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1}"

# Test counters
TOTAL_PROFILES=0
PASSED_PROFILES=0
FAILED_PROFILES=0
declare -A PROFILE_ISSUES=()

echo "=== Profile Smoke Test: All 6 Roles ==="
echo "Controller: $CONTROLLER_URL"
echo

# JWT helper
get_jwt_token() {
    curl -s -X POST \
      -d "grant_type=password" \
      -d "client_id=$KEYCLOAK_CLIENT_ID" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
      -d "username=$1" \
      -d "password=$2" \
      "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" | jq -r '.access_token'
}

# Get JWT
echo -e "${BLUE}🔑 Obtaining JWT token...${NC}"
JWT_TOKEN=$(get_jwt_token "phase5test" "test123")
if [ "$JWT_TOKEN" = "null" ] || [ -z "$JWT_TOKEN" ]; then
    echo -e "${RED}❌ Failed to get JWT token from Keycloak${NC}"
    exit 1
fi
echo -e "${GREEN}✅ JWT token obtained${NC}"
echo

# Smoke test function
smoke_test_profile() {
    local ROLE=$1
    local EXPECTED_PROVIDER_FORMAT=$2  # "array" or "object"
    
    TOTAL_PROFILES=$((TOTAL_PROFILES + 1))
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}Testing Profile: $ROLE${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    local ISSUES=()
    
    # Fetch profile
    RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/$ROLE" \
        -H "Authorization: Bearer $JWT_TOKEN")
    HTTP_CODE=$(echo "$RESPONSE" | tail -1)
    PROFILE_JSON=$(echo "$RESPONSE" | sed '$d')
    
    # Check HTTP status
    if [ "$HTTP_CODE" != "200" ]; then
        ISSUES+=("HTTP status $HTTP_CODE (expected 200)")
    fi
    
    # Check required fields
    for FIELD in role display_name description providers extensions goosehints gooseignore privacy policies recipes; do
        if ! echo "$PROFILE_JSON" | jq -e ".$FIELD" >/dev/null 2>&1; then
            ISSUES+=("Missing field: $FIELD")
        fi
    done
    
    # Check provider structure
    if [ "$EXPECTED_PROVIDER_FORMAT" = "array" ]; then
        if ! echo "$PROFILE_JSON" | jq -e '.providers | type == "array"' >/dev/null 2>&1; then
            ISSUES+=("Providers not an array (expected array format)")
        else
            PROVIDER_COUNT=$(echo "$PROFILE_JSON" | jq '.providers | length')
            if [ "$PROVIDER_COUNT" -lt 1 ]; then
                ISSUES+=("No providers configured")
            fi
        fi
    elif [ "$EXPECTED_PROVIDER_FORMAT" = "object" ]; then
        if ! echo "$PROFILE_JSON" | jq -e '.providers.primary' >/dev/null 2>&1; then
            ISSUES+=("Missing providers.primary (expected object format)")
        fi
        if ! echo "$PROFILE_JSON" | jq -e '.providers.allowed_providers' >/dev/null 2>&1; then
            ISSUES+=("Missing providers.allowed_providers")
        fi
    fi
    
    # Check privacy config
    PRIVACY_MODE=$(echo "$PROFILE_JSON" | jq -r '.privacy.mode')
    if [ "$PRIVACY_MODE" = "null" ]; then
        ISSUES+=("Missing privacy.mode")
    fi
    
    PRIVACY_STRICTNESS=$(echo "$PROFILE_JSON" | jq -r '.privacy.strictness')
    if [ "$PRIVACY_STRICTNESS" = "null" ]; then
        ISSUES+=("Missing privacy.strictness")
    fi
    
    # Check policies
    POLICY_COUNT=$(echo "$PROFILE_JSON" | jq '.policies | length')
    if [ "$POLICY_COUNT" -lt 1 ]; then
        ISSUES+=("No policies configured")
    fi
    
    # Check recipes
    RECIPE_COUNT=$(echo "$PROFILE_JSON" | jq '.recipes | length')
    if [ "$RECIPE_COUNT" -lt 3 ]; then
        ISSUES+=("Expected 3 recipes, found $RECIPE_COUNT")
    fi
    
    # Check goosehints/gooseignore
    GLOBAL_HINTS=$(echo "$PROFILE_JSON" | jq -r '.goosehints.global')
    if [ "$GLOBAL_HINTS" = "null" ] || [ -z "$GLOBAL_HINTS" ]; then
        ISSUES+=("Missing goosehints.global")
    fi
    
    GLOBAL_IGNORE=$(echo "$PROFILE_JSON" | jq -r '.gooseignore.global')
    if [ "$GLOBAL_IGNORE" = "null" ] || [ -z "$GLOBAL_IGNORE" ]; then
        ISSUES+=("Missing gooseignore.global")
    fi
    
    # Report results
    if [ ${#ISSUES[@]} -eq 0 ]; then
        PASSED_PROFILES=$((PASSED_PROFILES + 1))
        echo -e "${GREEN}✅ PASS${NC}: $ROLE profile OK"
        echo "  Provider format: $EXPECTED_PROVIDER_FORMAT"
        echo "  Privacy: $PRIVACY_MODE / $PRIVACY_STRICTNESS"
        echo "  Policies: $POLICY_COUNT"
        echo "  Recipes: $RECIPE_COUNT"
    else
        FAILED_PROFILES=$((FAILED_PROFILES + 1))
        echo -e "${RED}❌ FAIL${NC}: $ROLE profile has issues"
        for ISSUE in "${ISSUES[@]}"; do
            echo -e "${RED}   • $ISSUE${NC}"
        done
        PROFILE_ISSUES[$ROLE]="${ISSUES[*]}"
    fi
    echo
}

# Test all 6 profiles (all use object format with primary/planner/worker + allowed/forbidden)
smoke_test_profile "finance" "object"
smoke_test_profile "manager" "object"
smoke_test_profile "analyst" "object"
smoke_test_profile "marketing" "object"
smoke_test_profile "support" "object"
smoke_test_profile "legal" "object"

# Summary
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Profile Smoke Test Summary${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Total Profiles: $TOTAL_PROFILES"
echo -e "${GREEN}Passed:         $PASSED_PROFILES${NC}"
echo -e "${RED}Failed:         $FAILED_PROFILES${NC}"

if [ $FAILED_PROFILES -gt 0 ]; then
    echo
    echo -e "${RED}Failed Profiles:${NC}"
    for ROLE in "${!PROFILE_ISSUES[@]}"; do
        echo -e "  ${RED}❌ $ROLE${NC}"
        IFS=' ' read -ra ISSUES_ARRAY <<< "${PROFILE_ISSUES[$ROLE]}"
        for ISSUE in "${ISSUES_ARRAY[@]}"; do
            echo -e "     • $ISSUE"
        done
    done
    echo
    echo -e "${RED}❌ SMOKE TEST FAILED${NC}"
    exit 1
else
    echo
    echo -e "${GREEN}✅ ALL PROFILES PASSED SMOKE TEST${NC}"
    echo "All 6 profiles have valid schema and required fields"
    exit 0
fi
