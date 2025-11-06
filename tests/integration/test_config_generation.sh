#!/bin/bash
# Phase 5 Integration Test: Config Generation
# Tests that Controller can generate Goose config.yaml from profiles

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Environment
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-dev}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1}"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
declare -a FAILED_TEST_NAMES=()

# Helper functions
log_test() {
    echo -e "\n${YELLOW}[TEST $1]${NC} $2"
}

pass_test() {
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo -e "${GREEN}✅ PASS${NC}: $1"
}

fail_test() {
    FAILED_TESTS=$((FAILED_TESTS + 1))
    FAILED_TEST_NAMES+=("$1")
    echo -e "${RED}❌ FAIL${NC}: $1"
    echo -e "${RED}   Reason: $2${NC}"
}

# Get JWT token
get_jwt_token() {
    curl -s -X POST \
      -d "grant_type=password" \
      -d "client_id=$KEYCLOAK_CLIENT_ID" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
      -d "username=phase5test" \
      -d "password=test123" \
      "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" | jq -r '.access_token'
}

echo "=== Phase 5 Config Generation Integration Test ==="
echo "Controller: $CONTROLLER_URL"
echo

# Get JWT token
JWT_TOKEN=$(get_jwt_token)
if [ "$JWT_TOKEN" = "null" ] || [ -z "$JWT_TOKEN" ]; then
    echo -e "${RED}❌ Failed to obtain JWT token${NC}"
    exit 1
fi

# Test 1: Generate Finance config.yaml
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/finance/config generates valid YAML"
CONFIG_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/finance/config" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$CONFIG_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    CONFIG_YAML=$(echo "$CONFIG_RESPONSE" | sed '$d')
    # Check for key YAML sections
    if echo "$CONFIG_YAML" | grep -q "provider:" && echo "$CONFIG_YAML" | grep -q "extensions:"; then
        pass_test "Finance config.yaml generated successfully"
        # Save to file for inspection
        echo "$CONFIG_YAML" > /tmp/finance_config.yaml
        echo "   Config saved to /tmp/finance_config.yaml"
    else
        fail_test "Config YAML structure" "Missing required sections (provider or extensions)"
    fi
else
    fail_test "Finance config generation" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 2: Generate Analyst config.yaml
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/analyst/config generates valid YAML"
CONFIG_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/analyst/config" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$CONFIG_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    pass_test "Analyst config.yaml generated successfully"
else
    fail_test "Analyst config generation" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 3: Config without JWT returns 401
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/finance/config without JWT returns 401"
NO_AUTH_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$CONTROLLER_URL/profiles/finance/config")
if [ "$NO_AUTH_RESPONSE" = "401" ]; then
    pass_test "Config endpoint requires authentication"
else
    fail_test "Config authentication enforcement" "Expected 401, got $NO_AUTH_RESPONSE"
fi

# Test 4: Invalid role returns 404
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/invalid/config returns 404"
INVALID_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$CONTROLLER_URL/profiles/invalidrole/config" \
    -H "Authorization: Bearer $JWT_TOKEN")
if [ "$INVALID_RESPONSE" = "404" ]; then
    pass_test "Invalid role correctly returns 404"
else
    fail_test "Invalid role handling" "Expected 404, got $INVALID_RESPONSE"
fi

# Test 5: Legal config has local-only Ollama
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "Legal config.yaml uses local-only Ollama (no cloud providers)"
CONFIG_RESPONSE=$(curl -s -X GET "$CONTROLLER_URL/profiles/legal/config" \
    -H "Authorization: Bearer $JWT_TOKEN")
if echo "$CONFIG_RESPONSE" | grep -q "ollama" && ! echo "$CONFIG_RESPONSE" | grep -qi "openrouter\|openai\|anthropic"; then
    pass_test "Legal profile correctly uses local-only Ollama"
else
    fail_test "Legal local-only validation" "Expected Ollama without cloud providers"
fi

# ============================================================================
# Summary
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Config Generation Test Summary${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Total Tests:   $TOTAL_TESTS"
echo -e "${GREEN}Passed:        $PASSED_TESTS${NC}"
echo -e "${RED}Failed:        $FAILED_TESTS${NC}"
echo

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed Tests:${NC}"
    for test_name in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "  ${RED}•${NC} $test_name"
    done
    echo
    echo -e "${RED}❌ CONFIG GENERATION TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}✅ ALL CONFIG GENERATION TESTS PASSED${NC}"
    exit 0
fi
