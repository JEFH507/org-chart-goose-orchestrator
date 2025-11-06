#!/bin/bash
# Phase 5 Integration Test: Profile Loading
# Tests that users can fetch their role profiles with JWT authentication

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
    local username=$1
    local password=$2
    curl -s -X POST \
      -d "grant_type=password" \
      -d "client_id=$KEYCLOAK_CLIENT_ID" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
      -d "username=$username" \
      -d "password=$password" \
      "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" | jq -r '.access_token'
}

echo "=== Phase 5 Profile Loading Integration Test ==="
echo "Controller: $CONTROLLER_URL"
echo "Keycloak: $KEYCLOAK_URL"
echo

# Test 1: Get JWT token for test user
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "Obtain JWT token for phase5test user"
JWT_TOKEN=$(get_jwt_token "phase5test" "test123")
if [ "$JWT_TOKEN" != "null" ] && [ -n "$JWT_TOKEN" ]; then
    pass_test "JWT token obtained"
else
    fail_test "JWT token retrieval" "Failed to get token from Keycloak"
    echo -e "${RED}Cannot proceed without JWT token${NC}"
    exit 1
fi

# Test 2: Fetch Finance profile
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/finance returns profile data"
FINANCE_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/finance" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$FINANCE_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    PROFILE_JSON=$(echo "$FINANCE_RESPONSE" | sed '$d')
    if echo "$PROFILE_JSON" | jq -e '.role == "finance"' > /dev/null 2>&1; then
        pass_test "Finance profile loaded successfully"
    else
        fail_test "Finance profile structure" "Expected role=finance in response"
    fi
else
    fail_test "Finance profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 3: Fetch Manager profile
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/manager returns profile data"
MANAGER_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/manager" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$MANAGER_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    PROFILE_JSON=$(echo "$MANAGER_RESPONSE" | sed '$d')
    if echo "$PROFILE_JSON" | jq -e '.role == "manager"' > /dev/null 2>&1; then
        pass_test "Manager profile loaded successfully"
    else
        fail_test "Manager profile structure" "Expected role=manager in response"
    fi
else
    fail_test "Manager profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 4: Fetch Analyst profile
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/analyst returns profile data"
ANALYST_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/analyst" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$ANALYST_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    PROFILE_JSON=$(echo "$ANALYST_RESPONSE" | sed '$d')
    if echo "$PROFILE_JSON" | jq -e '.role == "analyst"' > /dev/null 2>&1; then
        pass_test "Analyst profile loaded successfully"
    else
        fail_test "Analyst profile structure" "Expected role=analyst in response"
    fi
else
    fail_test "Analyst profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 5: Fetch Marketing profile
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/marketing returns profile data"
MARKETING_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/marketing" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$MARKETING_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    PROFILE_JSON=$(echo "$MARKETING_RESPONSE" | sed '$d')
    if echo "$PROFILE_JSON" | jq -e '.role == "marketing"' > /dev/null 2>&1; then
        pass_test "Marketing profile loaded successfully"
    else
        fail_test "Marketing profile structure" "Expected role=marketing in response"
    fi
else
    fail_test "Marketing profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 6: Fetch Support profile
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/support returns profile data"
SUPPORT_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/support" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$SUPPORT_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    PROFILE_JSON=$(echo "$SUPPORT_RESPONSE" | sed '$d')
    if echo "$PROFILE_JSON" | jq -e '.role == "support"' > /dev/null 2>&1; then
        pass_test "Support profile loaded successfully"
    else
        fail_test "Support profile structure" "Expected role=support in response"
    fi
else
    fail_test "Support profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 7: Fetch Legal profile
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/legal returns profile data"
LEGAL_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/legal" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$LEGAL_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "200" ]; then
    PROFILE_JSON=$(echo "$LEGAL_RESPONSE" | sed '$d')
    if echo "$PROFILE_JSON" | jq -e '.role == "legal"' > /dev/null 2>&1; then
        pass_test "Legal profile loaded successfully"
    else
        fail_test "Legal profile structure" "Expected role=legal in response"
    fi
else
    fail_test "Legal profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# Test 8: Profile contains required fields
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "Finance profile contains all required fields"
FINANCE_PROFILE=$(curl -s -X GET "$CONTROLLER_URL/profiles/finance" -H "Authorization: Bearer $JWT_TOKEN")
REQUIRED_FIELDS=("role" "display_name" "providers" "extensions" "goosehints" "gooseignore" "recipes" "privacy" "policies")
MISSING_FIELDS=()
for field in "${REQUIRED_FIELDS[@]}"; do
    if ! echo "$FINANCE_PROFILE" | jq -e ".$field" > /dev/null 2>&1; then
        MISSING_FIELDS+=("$field")
    fi
done

if [ ${#MISSING_FIELDS[@]} -eq 0 ]; then
    pass_test "All required fields present in profile"
else
    fail_test "Profile structure validation" "Missing fields: ${MISSING_FIELDS[*]}"
fi

# Test 9: Invalid role returns 404
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/invalid returns 404"
INVALID_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$CONTROLLER_URL/profiles/invalidrole" \
    -H "Authorization: Bearer $JWT_TOKEN")
if [ "$INVALID_RESPONSE" = "404" ]; then
    pass_test "Invalid role correctly returns 404"
else
    fail_test "Invalid role handling" "Expected 404, got $INVALID_RESPONSE"
fi

# Test 10: Profile without JWT returns 401
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS "GET /profiles/finance without JWT returns 401"
NO_AUTH_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$CONTROLLER_URL/profiles/finance")
if [ "$NO_AUTH_RESPONSE" = "401" ]; then
    pass_test "Profile endpoint requires authentication"
else
    fail_test "Authentication enforcement" "Expected 401, got $NO_AUTH_RESPONSE"
fi

# ============================================================================
# Summary
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Profile Loading Test Summary${NC}"
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
    echo -e "${RED}❌ PROFILE LOADING TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}✅ ALL PROFILE LOADING TESTS PASSED${NC}"
    exit 0
fi
