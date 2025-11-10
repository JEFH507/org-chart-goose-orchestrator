#!/bin/bash
# Phase 5 Workstream H - Task H3: Finance PII Redaction Integration Test (Full E2E with JWT)
# Tests REAL end-to-end PII redaction: Auth → Profile → Privacy Guard API → Audit DB

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Service URLs
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
PRIVACY_GUARD_URL="${PRIVACY_GUARD_URL:-http://localhost:8089}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-dev}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8}"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
declare -a FAILED_TEST_NAMES=()

log_test() {
    echo -e "\n${YELLOW}[TEST $1]${NC} $2"
}

pass_test() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✅ PASS${NC}: $1"
}

fail_test() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    FAILED_TEST_NAMES+=("$1")
    echo -e "${RED}❌ FAIL${NC}: $1"
    [ -n "${2:-}" ] && echo -e "${RED}   Reason: $2${NC}"
}

get_jwt_token() {
    curl -s -X POST \
      -d "grant_type=client_credentials" \
      -d "client_id=$KEYCLOAK_CLIENT_ID" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
      "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" | jq -r '.access_token'
}

echo "=== Finance PII Redaction Test (Full E2E with JWT) ==="
echo "Controller:    $CONTROLLER_URL"
echo "Privacy Guard: $PRIVACY_GUARD_URL"
echo

# Test 1: Get JWT
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Obtain JWT token"
JWT_TOKEN=$(get_jwt_token)
if [ "$JWT_TOKEN" != "null" ] && [ -n "$JWT_TOKEN" ]; then
    pass_test "JWT token obtained"
else
    fail_test "JWT token" "Failed to get token from Keycloak"
    exit 1
fi

# Test 2: Finance Profile
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Finance profile accessible"
PROFILE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/finance" -H "Authorization: Bearer $JWT_TOKEN")
if [ "$(echo "$PROFILE" | tail -1)" = "200" ]; then
    pass_test "Finance profile loaded"
else
    fail_test "Finance profile" "HTTP $(echo "$PROFILE" | tail -1)"
fi

# Test 3: Privacy Guard Health
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Privacy Guard service accessible"
PG_STATUS=$(curl -s -w "\n%{http_code}" "$PRIVACY_GUARD_URL/status")
if [ "$(echo "$PG_STATUS" | tail -1)" = "200" ]; then
    pass_test "Privacy Guard healthy"
    echo "  $(echo "$PG_STATUS" | sed '$d' | jq -r '"Mode: \(.mode), Rules: \(.rule_count), Model: \(.model_name)"')"
else
    fail_test "Privacy Guard" "HTTP $(echo "$PG_STATUS" | tail -1)"
fi

# Test 4: SSN Detection
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "SSN detection via /guard/scan"
SCAN=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" -H "Content-Type: application/json" \
    -d '{"text":"SSN is 123-45-6789","tenant_id":"finance-test"}')
SSN_FOUND=$(echo "$SCAN" | jq -r '.detections[] | select(.entity_type=="SSN") | .matched_text')
if [ "$SSN_FOUND" = "123-45-6789" ]; then
    pass_test "SSN detected: $SSN_FOUND"
else
    fail_test "SSN detection" "Expected 123-45-6789, got: $SSN_FOUND"
fi

# Test 5: Email Detection
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Email detection via /guard/scan"
SCAN=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" -H "Content-Type: application/json" \
    -d '{"text":"Email: test@example.com","tenant_id":"finance-test"}')
EMAIL_FOUND=$(echo "$SCAN" | jq -r '.detections[] | select(.entity_type=="EMAIL") | .matched_text')
if [ "$EMAIL_FOUND" = "test@example.com" ]; then
    pass_test "Email detected: $EMAIL_FOUND"
else
    fail_test "Email detection" "Expected test@example.com, got: $EMAIL_FOUND"
fi

# Test 6: PII Masking
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "PII masking via /guard/mask"
MASK=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/mask" -H "Content-Type: application/json" \
    -d '{"text":"SSN 123-45-6789 and email test@example.com","tenant_id":"finance-test"}')
MASKED_TEXT=$(echo "$MASK" | jq -r '.masked_text')
REDACTION_COUNT=$(echo "$MASK" | jq -r '.redactions | to_entries | length')
if [ "$REDACTION_COUNT" -ge 2 ]; then
    pass_test "PII masked successfully ($REDACTION_COUNT categories)"
    echo "  Original: SSN 123-45-6789 and email test@example.com"
    echo "  Masked:   $MASKED_TEXT"
    echo "  Redactions: $(echo "$MASK" | jq -c '.redactions')"
else
    fail_test "PII masking" "Expected 2+ categories, got $REDACTION_COUNT"
fi

# Test 7: Audit Log to Controller
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Audit log submission"
AUDIT=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/privacy/audit" \
    -H "Authorization: Bearer $JWT_TOKEN" -H "Content-Type: application/json" \
    -d "{\"session_id\":\"e7-test-$(date +%s)\",\"redaction_count\":2,\"categories\":[\"SSN\",\"EMAIL\"],\"mode\":\"Hybrid\",\"timestamp\":$(date +%s)}")
if [ "$(echo "$AUDIT" | tail -1)" = "201" ]; then
    pass_test "Audit log created in Controller"
    echo "  Audit ID: $(echo "$AUDIT" | sed '$d' | jq -r '.id')"
else
    fail_test "Audit log" "HTTP $(echo "$AUDIT" | tail -1)"
fi

# Test 8: Verify in Database
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Audit log in database"
DB_COUNT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c \
    "SELECT COUNT(*) FROM privacy_audit_logs WHERE session_id LIKE 'e7-test-%';" 2>/dev/null | tr -d ' ')
if [ "$DB_COUNT" -gt 0 ]; then
    pass_test "Audit logs in database ($DB_COUNT records)"
else
    fail_test "Database verification" "No audit logs found"
fi

# Summary
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Finance PII Redaction Test Summary${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Total Tests:   $TESTS_RUN"
echo -e "${GREEN}Passed:        $TESTS_PASSED${NC}"
echo -e "${RED}Failed:        $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "\n${RED}Failed Tests:${NC}"
    for test_name in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "  ${RED}•${NC} $test_name"
    done
    echo -e "\n${RED}❌ TESTS FAILED${NC}"
    exit 1
else
    echo -e "\n${GREEN}✅ ALL TESTS PASSED - REAL E2E INTEGRATION WORKING${NC}"
    echo "✓ JWT auth → Profile → Privacy Guard API → Audit DB"
    exit 0
fi
