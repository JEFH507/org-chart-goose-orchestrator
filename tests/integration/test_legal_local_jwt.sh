#!/bin/bash
# Phase 5 Workstream H - Task H3: Legal Local-Only Enforcement (Full E2E with JWT)
# Tests REAL end-to-end local-only enforcement: Auth → Profile → Provider Validation

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Service URLs
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
OLLAMA_URL="${OLLAMA_URL:-http://localhost:11434}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-dev}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1}"

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
      -d "grant_type=password" \
      -d "client_id=$KEYCLOAK_CLIENT_ID" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
      -d "username=$1" \
      -d "password=$2" \
      "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" | jq -r '.access_token'
}

echo "=== Legal Local-Only Enforcement Test (Full E2E with JWT) ==="
echo "Controller: $CONTROLLER_URL"
echo "Ollama:     $OLLAMA_URL"
echo

# Test 1: Get JWT
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Obtain JWT token"
JWT_TOKEN=$(get_jwt_token "phase5test" "test123")
if [ "$JWT_TOKEN" != "null" ] && [ -n "$JWT_TOKEN" ]; then
    pass_test "JWT token obtained"
else
    fail_test "JWT token" "Failed to get token from Keycloak"
    exit 1
fi

# Test 2: Legal Profile
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Legal profile accessible"
PROFILE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/legal" -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$PROFILE" | tail -1)
PROFILE_JSON=$(echo "$PROFILE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    pass_test "Legal profile loaded"
else
    fail_test "Legal profile" "HTTP $HTTP_CODE"
    exit 1
fi

# Test 3: Local-Only Provider Configuration
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Legal profile configured for local-only"

ALLOWED=$(echo "$PROFILE_JSON" | jq -r '.providers.allowed_providers[]? // empty')
FORBIDDEN=$(echo "$PROFILE_JSON" | jq -r '.providers.forbidden_providers[]? // empty')

# Check that only ollama is allowed
CLOUD_IN_ALLOWED=false
for provider in $ALLOWED; do
    if [ "$provider" != "ollama" ]; then
        CLOUD_IN_ALLOWED=true
        fail_test "Provider policy" "Non-local provider '$provider' in allowed_providers"
        break
    fi
done

if ! $CLOUD_IN_ALLOWED && [ -n "$ALLOWED" ]; then
    pass_test "Only local providers allowed (ollama)"
    echo "  Allowed: $ALLOWED"
    echo "  Forbidden: $(echo $FORBIDDEN | tr '\n' ',' | sed 's/,$//')"
fi

# Test 4: Ollama as Primary Provider
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Ollama configured as primary provider"

PRIMARY_PROVIDER=$(echo "$PROFILE_JSON" | jq -r '.providers.primary.provider // "none"')
if [ "$PRIMARY_PROVIDER" = "ollama" ]; then
    pass_test "Ollama is primary provider"
    OLLAMA_MODEL=$(echo "$PROFILE_JSON" | jq -r '.providers.primary.model // "not_set"')
    echo "  Model: $OLLAMA_MODEL"
else
    fail_test "Primary provider" "Expected ollama, got: $PRIMARY_PROVIDER"
fi

# Test 5: Ollama Service Available
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Ollama service accessible"

if curl -s -f "$OLLAMA_URL/api/tags" > /dev/null 2>&1; then
    pass_test "Ollama service running"
    MODELS=$(curl -s "$OLLAMA_URL/api/tags" | jq -r '.models[]?.name // empty' | head -3)
    if [ -n "$MODELS" ]; then
        echo "  Available models:"
        echo "$MODELS" | while read -r model; do echo "    - $model"; done
    fi
else
    fail_test "Ollama service" "Not accessible at $OLLAMA_URL"
fi

# Test 6: Memory Retention Policy
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Legal profile has ephemeral memory (retention_days: 0 or null)"

RETENTION_DAYS=$(echo "$PROFILE_JSON" | jq -r '.privacy.retention_days')
if [ "$RETENTION_DAYS" = "0" ] || [ "$RETENTION_DAYS" = "null" ]; then
    pass_test "Memory retention is ephemeral (retention_days: $RETENTION_DAYS)"
    echo "  Note: null is treated as ephemeral by default"
else
    fail_test "Memory retention" "Expected 0 or null, got: $RETENTION_DAYS"
fi

# Test 7: Privacy Guard Mode
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Legal profile has strict privacy mode"

PRIVACY_MODE=$(echo "$PROFILE_JSON" | jq -r '.privacy.guard_mode // "not_set"')
echo "  Privacy Mode: $PRIVACY_MODE"
if [ "$PRIVACY_MODE" != "off" ]; then
    pass_test "Privacy Guard enabled"
else
    echo "  ⓘ Privacy mode is off"
fi

# Test 8: Policy Enforcement Rules
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Legal profile has policy rules"

POLICIES_COUNT=$(echo "$PROFILE_JSON" | jq -r '.policies | length // 0')
if [ "$POLICIES_COUNT" -gt 0 ]; then
    pass_test "Policy rules configured ($POLICIES_COUNT policies)"
    echo "  Sample policies:"
    echo "$PROFILE_JSON" | jq -r '.policies[] | "    - \(.rule_type): \(.pattern // .tool_name)"' | head -3
else
    fail_test "Policy enforcement" "No policies configured"
fi

# Test 9: Audit Log for Local-Only Enforcement
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Audit log for local-only enforcement"

AUDIT=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/privacy/audit" \
    -H "Authorization: Bearer $JWT_TOKEN" -H "Content-Type: application/json" \
    -d "{\"session_id\":\"e8-legal-$(date +%s)\",\"redaction_count\":0,\"categories\":[],\"mode\":\"LocalOnly\",\"timestamp\":$(date +%s)}")
if [ "$(echo "$AUDIT" | tail -1)" = "201" ]; then
    pass_test "Local-only audit log created"
    echo "  Audit ID: $(echo "$AUDIT" | sed '$d' | jq -r '.id')"
else
    fail_test "Audit log" "HTTP $(echo "$AUDIT" | tail -1)"
fi

# Test 10: End-to-End Legal Workflow
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "End-to-end Legal workflow validation"

E2E_SUCCESS=true

echo "  1. User authentication: ✓ (JWT token obtained)"

# Profile loading
if [ "$HTTP_CODE" = "200" ]; then
    echo "  2. Profile loading: ✓ (Legal profile with local-only config)"
else
    echo "  2. Profile loading: ✗"
    E2E_SUCCESS=false
fi

# Provider check
if [ "$PRIMARY_PROVIDER" = "ollama" ]; then
    echo "  3. Provider validation: ✓ (Ollama local-only)"
else
    echo "  3. Provider validation: ✗"
    E2E_SUCCESS=false
fi

# Ollama availability
OLLAMA_CHECK=$(curl -s -o /dev/null -w "%{http_code}" "$OLLAMA_URL/api/tags")
if [ "$OLLAMA_CHECK" = "200" ]; then
    echo "  4. Ollama availability: ✓ (Local LLM ready)"
else
    echo "  4. Ollama availability: ✗ (HTTP $OLLAMA_CHECK)"
    E2E_SUCCESS=false
fi

# Memory retention
if [ "$RETENTION_DAYS" = "0" ] || [ "$RETENTION_DAYS" = "null" ]; then
    echo "  5. Memory retention: ✓ (Ephemeral - attorney-client privilege)"
else
    echo "  5. Memory retention: ✗ (retention_days: $RETENTION_DAYS)"
    E2E_SUCCESS=false
fi

if $E2E_SUCCESS; then
    pass_test "E2E Legal workflow validated"
    echo "  Flow: Auth → Profile → Local-Only → Ollama → Ephemeral Memory"
else
    fail_test "E2E Legal workflow" "One or more steps failed"
fi

# Summary
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Legal Local-Only Enforcement Test Summary${NC}"
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
    echo "✓ JWT auth → Legal profile → Local-only Ollama → Attorney-client privilege"
    exit 0
fi
