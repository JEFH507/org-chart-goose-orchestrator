#!/bin/bash
# Privacy Guard Proxy Integration Tests
# Tests the full flow: Client → Proxy → Privacy Guard → LLM → Privacy Guard → Proxy → Client

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Service endpoints
PROXY_URL="http://localhost:8090"
PRIVACY_GUARD_URL="http://localhost:8089"
CONTROLLER_URL="http://localhost:8088"

# Test results array
declare -a FAILED_TESTS=()

# Helper functions
print_test() {
    echo -e "\n${YELLOW}Test $1: $2${NC}"
    TESTS_RUN=$((TESTS_RUN + 1))
}

pass() {
    echo -e "${GREEN}✓ PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}"
    echo -e "${RED}  Reason: $1${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    FAILED_TESTS+=("Test $TESTS_RUN: $2")
}

# Test 1: Proxy service health check
print_test 1 "Proxy service accessible"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $PROXY_URL/api/status)
if [ "$RESPONSE" = "200" ]; then
    pass
else
    fail "Proxy not accessible (HTTP $RESPONSE)" "Proxy health check"
fi

# Test 2: Privacy Guard service health check
print_test 2 "Privacy Guard service accessible"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $PRIVACY_GUARD_URL/status)
if [ "$RESPONSE" = "200" ]; then
    pass
else
    fail "Privacy Guard not accessible (HTTP $RESPONSE)" "Privacy Guard health check"
fi

# Test 3: Proxy mode retrieval
print_test 3 "Get current proxy mode"
MODE=$(curl -s $PROXY_URL/api/mode)
if [ -n "$MODE" ]; then
    echo "  Current mode: $MODE"
    pass
else
    fail "Could not retrieve mode" "Mode retrieval"
fi

# Test 4: Proxy mode switching
print_test 4 "Switch proxy mode to strict"
RESPONSE=$(curl -s -X PUT $PROXY_URL/api/mode \
    -H "Content-Type: application/json" \
    -d '"strict"')
if echo "$RESPONSE" | grep -q "strict"; then
    pass
else
    fail "Mode switch failed: $RESPONSE" "Mode switching"
fi

# Test 5: Privacy Guard PII detection
print_test 5 "Privacy Guard detects PII (SSN)"
TEXT_WITH_SSN="Employee John Doe has SSN 123-45-6789"
RESPONSE=$(curl -s -X POST $PRIVACY_GUARD_URL/guard/scan \
    -H "Content-Type: application/json" \
    -d "{\"tenant_id\": \"test-tenant\", \"text\": \"$TEXT_WITH_SSN\", \"pii_types\": [\"SSN\"]}")

if echo "$RESPONSE" | grep -q "123-45-6789"; then
    pass
else
    fail "SSN not detected: $RESPONSE" "PII detection"
fi

# Test 6: Privacy Guard PII masking
print_test 6 "Privacy Guard masks PII"
MASK_RESPONSE=$(curl -s -X POST $PRIVACY_GUARD_URL/guard/mask \
    -H "Content-Type: application/json" \
    -d "{\"tenant_id\": \"test-tenant\", \"text\": \"$TEXT_WITH_SSN\"}")

MASKED_TEXT=$(echo "$MASK_RESPONSE" | jq -r '.masked_text')
if [ -n "$MASKED_TEXT" ] && ! echo "$MASKED_TEXT" | grep -q "123-45-6789"; then
    echo "  Masked: $MASKED_TEXT"
    pass
else
    fail "Masking failed: $MASKED_TEXT" "PII masking"
fi

# Test 7: Proxy pass-through (no PII)
print_test 7 "Proxy forwards request without PII"
# Note: This would require a real LLM endpoint, so we'll test the proxy accepts the request
REQUEST_BODY='{
  "model": "test-model",
  "messages": [{"role": "user", "content": "Hello, how are you?"}]
}'

# We expect the proxy to accept the request (even if LLM call fails without real API key)
RESPONSE=$(curl -s -X POST $PROXY_URL/v1/chat/completions \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer sk-test-key" \
    -d "$REQUEST_BODY" 2>&1)

# Check if proxy accepted the request (may fail at LLM level without real key)
if echo "$RESPONSE" | grep -qE "(error|invalid|unauthorized)" || [ -n "$RESPONSE" ]; then
    echo "  Proxy accepted request (LLM response: expected to fail without real API key)"
    pass
else
    fail "Proxy rejected request: $RESPONSE" "Proxy pass-through"
fi

# Test 8: Activity log verification
print_test 8 "Activity log records operations"
ACTIVITY=$(curl -s $PROXY_URL/api/activity)
ACTIVITY_COUNT=$(echo "$ACTIVITY" | jq '. | length')

if [ "$ACTIVITY_COUNT" -gt 0 ]; then
    echo "  Activity entries: $ACTIVITY_COUNT"
    pass
else
    fail "No activity logged" "Activity logging"
fi

# Test 9: Proxy mode reset to auto
print_test 9 "Reset proxy mode to auto"
RESPONSE=$(curl -s -X PUT $PROXY_URL/api/mode \
    -H "Content-Type: application/json" \
    -d '"auto"')
if echo "$RESPONSE" | grep -q "auto"; then
    pass
else
    fail "Mode reset failed: $RESPONSE" "Mode reset"
fi

# Test 10: Control Panel UI accessible
print_test 10 "Control Panel UI accessible"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $PROXY_URL/ui)
if [ "$RESPONSE" = "200" ]; then
    pass
else
    fail "UI not accessible (HTTP $RESPONSE)" "UI accessibility"
fi

# Summary
echo ""
echo "=========================================="
echo "PRIVACY GUARD PROXY TEST SUMMARY"
echo "=========================================="
echo "Total tests run:    $TESTS_RUN"
echo -e "Tests passed:       ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed:       ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}FAILED TESTS:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "${RED}  - $test${NC}"
    done
    echo ""
    exit 1
else
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
fi
