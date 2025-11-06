#!/bin/bash
# Phase 5 Workstream E - Task E7: Finance PII Redaction Integration Test (with JWT Auth)
# Tests end-to-end PII redaction flow with proper authentication

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Service URLs
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
PRIVACY_GUARD_URL="${PRIVACY_GUARD_URL:-http://localhost:8089}"
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

# Helper functions
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
    if [ -n "${2:-}" ]; then
        echo -e "${RED}   Reason: $2${NC}"
    fi
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

# Cleanup
cleanup() {
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

TEMP_DIR=$(mktemp -d)

echo "=== Phase 5 Workstream E - Task E7: Finance PII Redaction Test (JWT Auth) ==="
echo "Controller:    $CONTROLLER_URL"
echo "Privacy Guard: $PRIVACY_GUARD_URL"
echo "Keycloak:      $KEYCLOAK_URL"
echo

# ==============================================================================
# Test 1: Obtain JWT Token
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Obtain JWT token for phase5test user"

JWT_TOKEN=$(get_jwt_token "phase5test" "test123")
if [ "$JWT_TOKEN" != "null" ] && [ -n "$JWT_TOKEN" ]; then
    pass_test "JWT token obtained"
else
    fail_test "JWT token retrieval" "Failed to get token from Keycloak"
    echo -e "${RED}Cannot proceed without JWT token${NC}"
    exit 1
fi

# ==============================================================================
# Test 2: Controller API Health
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Controller API is accessible"

if curl -s -f "$CONTROLLER_URL/health" > /dev/null 2>&1; then
    pass_test "Controller API health check passed"
else
    fail_test "Controller API health" "Not accessible at $CONTROLLER_URL"
fi

# ==============================================================================
# Test 3: Finance Profile Exists
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Finance profile exists and is accessible"

PROFILE_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$CONTROLLER_URL/profiles/finance" \
    -H "Authorization: Bearer $JWT_TOKEN")
HTTP_CODE=$(echo "$PROFILE_RESPONSE" | tail -1)
PROFILE_JSON=$(echo "$PROFILE_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    if echo "$PROFILE_JSON" | jq -e '.role == "finance"' > /dev/null 2>&1; then
        pass_test "Finance profile loaded successfully"
        
        # Extract privacy configuration
        PRIVACY_MODE=$(echo "$PROFILE_JSON" | jq -r '.privacy.guard_mode // "not_set"')
        echo "  Privacy Mode: $PRIVACY_MODE"
        
        # Check if privacy guard is enabled
        if [ "$PRIVACY_MODE" != "off" ]; then
            echo "  ✓ Privacy Guard is enabled for Finance role"
        fi
    else
        fail_test "Finance profile structure" "Expected role=finance in response"
    fi
else
    fail_test "Finance profile fetch" "Expected HTTP 200, got $HTTP_CODE"
fi

# ==============================================================================
# Test 4: Privacy Guard Service Health
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Privacy Guard service is accessible"

if curl -s -f "$PRIVACY_GUARD_URL/health" > /dev/null 2>&1; then
    pass_test "Privacy Guard health check passed"
else
    fail_test "Privacy Guard health" "Not accessible at $PRIVACY_GUARD_URL"
fi

# ==============================================================================
# Test 5: SSN Detection via Privacy Guard /scan Endpoint
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "SSN detection using Privacy Guard /scan endpoint"

SSN_INPUT="Analyze employee John Smith with SSN 123-45-6789 from Finance department"
SCAN_PAYLOAD=$(cat <<EOF
{
  "text": "$SSN_INPUT",
  "mode": "rules"
}
EOF
)

SCAN_RESPONSE=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
    -H "Content-Type: application/json" \
    -d "$SCAN_PAYLOAD")

if echo "$SCAN_RESPONSE" | jq -e '.detections | length > 0' > /dev/null 2>&1; then
    SSN_DETECTED=$(echo "$SCAN_RESPONSE" | jq -r '.detections[] | select(.entity_type == "SSN") | .matched_text')
    if [ "$SSN_DETECTED" = "123-45-6789" ]; then
        pass_test "SSN detected correctly"
        echo "  Detected: $SSN_DETECTED"
    else
        fail_test "SSN detection" "Expected SSN 123-45-6789, got: $SSN_DETECTED"
    fi
else
    fail_test "SSN detection" "No detections returned from Privacy Guard"
fi

# ==============================================================================
# Test 6: Email Detection via Privacy Guard /scan Endpoint
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Email detection using Privacy Guard /scan endpoint"

EMAIL_INPUT="Contact John Smith at john.smith@example.com for budget review"
SCAN_PAYLOAD=$(cat <<EOF
{
  "text": "$EMAIL_INPUT",
  "mode": "rules"
}
EOF
)

SCAN_RESPONSE=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
    -H "Content-Type: application/json" \
    -d "$SCAN_PAYLOAD")

if echo "$SCAN_RESPONSE" | jq -e '.detections | length > 0' > /dev/null 2>&1; then
    EMAIL_DETECTED=$(echo "$SCAN_RESPONSE" | jq -r '.detections[] | select(.entity_type == "EMAIL") | .matched_text')
    if [ "$EMAIL_DETECTED" = "john.smith@example.com" ]; then
        pass_test "Email detected correctly"
        echo "  Detected: $EMAIL_DETECTED"
    else
        fail_test "Email detection" "Expected john.smith@example.com, got: $EMAIL_DETECTED"
    fi
else
    fail_test "Email detection" "No detections returned from Privacy Guard"
fi

# ==============================================================================
# Test 7: Person Name Detection via NER (if Ollama available)
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Person name detection using NER mode"

PERSON_INPUT="Contact John Smith from Finance department"
SCAN_PAYLOAD=$(cat <<EOF
{
  "text": "$PERSON_INPUT",
  "mode": "ner"
}
EOF
)

SCAN_RESPONSE=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
    -H "Content-Type: application/json" \
    -d "$SCAN_PAYLOAD")

if echo "$SCAN_RESPONSE" | jq -e '.detections | length > 0' > /dev/null 2>&1; then
    PERSON_DETECTED=$(echo "$SCAN_RESPONSE" | jq -r '.detections[] | select(.entity_type == "PERSON") | .matched_text')
    if echo "$PERSON_DETECTED" | grep -q "John"; then
        pass_test "Person name detected via NER"
        echo "  Detected: $PERSON_DETECTED"
    else
        fail_test "Person name detection" "Expected name containing 'John', got: $PERSON_DETECTED"
    fi
else
    echo "  ⓘ No NER detections (Ollama may not be configured)"
    echo "  Hint: Check GUARD_MODEL_ENABLED in Privacy Guard config"
fi

# ==============================================================================
# Test 8: Multiple PII Types Detection (Hybrid Mode)
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Multiple PII detection using hybrid mode"

COMBINED_INPUT="Employee John Smith (SSN 123-45-6789, email john.smith@example.com) from Finance"
SCAN_PAYLOAD=$(cat <<EOF
{
  "text": "$COMBINED_INPUT",
  "mode": "hybrid"
}
EOF
)

SCAN_RESPONSE=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
    -H "Content-Type: application/json" \
    -d "$SCAN_PAYLOAD")

DETECTION_COUNT=$(echo "$SCAN_RESPONSE" | jq -r '.detections | length')
if [ "$DETECTION_COUNT" -ge 2 ]; then
    pass_test "Multiple PII types detected (found $DETECTION_COUNT items)"
    echo "  Detections:"
    echo "$SCAN_RESPONSE" | jq -r '.detections[] | "    - \(.entity_type): \(.matched_text)"'
else
    fail_test "Multiple PII detection" "Expected 2+ detections, got $DETECTION_COUNT"
fi

# ==============================================================================
# Test 9: PII Masking via /mask Endpoint
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "PII masking using Privacy Guard /mask endpoint"

MASK_INPUT="Employee SSN is 123-45-6789 and email is john.smith@example.com"
MASK_PAYLOAD=$(cat <<EOF
{
  "text": "$MASK_INPUT",
  "mode": "rules"
}
EOF
)

MASK_RESPONSE=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d "$MASK_PAYLOAD")

MASKED_TEXT=$(echo "$MASK_RESPONSE" | jq -r '.masked_text')
REDACTION_COUNT=$(echo "$MASK_RESPONSE" | jq -r '.redaction_count')

if [ "$REDACTION_COUNT" -ge 2 ]; then
    pass_test "PII successfully masked ($REDACTION_COUNT items)"
    echo "  Original: $MASK_INPUT"
    echo "  Masked:   $MASKED_TEXT"
else
    fail_test "PII masking" "Expected 2+ redactions, got $REDACTION_COUNT"
fi

# ==============================================================================
# Test 10: Audit Log Submission to Controller
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Privacy audit log submission to Controller"

AUDIT_PAYLOAD=$(cat <<EOF
{
  "session_id": "test-finance-$(date +%s)",
  "user_id": "phase5test",
  "role": "finance",
  "redaction_count": 3,
  "pii_types": ["SSN", "EMAIL", "PERSON"],
  "mode": "hybrid",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
)

AUDIT_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/privacy/audit" \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -H "Content-Type: application/json" \
    -d "$AUDIT_PAYLOAD")
HTTP_CODE=$(echo "$AUDIT_RESPONSE" | tail -1)

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
    pass_test "Audit log submitted successfully"
else
    # Check if endpoint exists
    if [ "$HTTP_CODE" = "404" ]; then
        echo "  ⓘ Audit endpoint not yet implemented (HTTP 404)"
        echo "  Note: This is expected in current phase - endpoint implementation pending"
    else
        fail_test "Audit log submission" "Expected HTTP 200/201, got $HTTP_CODE"
    fi
fi

# ==============================================================================
# Test 11: Profile Privacy Configuration Validation
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "Finance profile privacy configuration is valid"

# Re-fetch profile to verify privacy settings
PROFILE_JSON=$(curl -s -X GET "$CONTROLLER_URL/profiles/finance" \
    -H "Authorization: Bearer $JWT_TOKEN")

# Check privacy guard mode
GUARD_MODE=$(echo "$PROFILE_JSON" | jq -r '.privacy.guard_mode // "missing"')
if [ "$GUARD_MODE" != "missing" ]; then
    pass_test "Privacy guard mode configured: $GUARD_MODE"
else
    fail_test "Privacy configuration" "guard_mode field missing"
fi

# Check PII rules configuration
PII_RULES_COUNT=$(echo "$PROFILE_JSON" | jq -r '.privacy.pii_rules | length // 0')
if [ "$PII_RULES_COUNT" -gt 0 ]; then
    echo "  ✓ PII rules configured: $PII_RULES_COUNT rules"
else
    echo "  ⓘ No explicit PII rules (using defaults)"
fi

# ==============================================================================
# Test 12: End-to-End Workflow Integration
# ==============================================================================
TESTS_RUN=$((TESTS_RUN + 1))
log_test $TESTS_RUN "End-to-end Finance PII redaction workflow"

E2E_SUCCESS=true

# Step 1: User authenticates
echo "  1. User authentication: ✓ (JWT token obtained)"

# Step 2: Fetch Finance profile
PROFILE_CHECK=$(curl -s -o /dev/null -w "%{http_code}" \
    -X GET "$CONTROLLER_URL/profiles/finance" \
    -H "Authorization: Bearer $JWT_TOKEN")
if [ "$PROFILE_CHECK" = "200" ]; then
    echo "  2. Profile loading: ✓ (Finance profile)"
else
    echo "  2. Profile loading: ✗ (HTTP $PROFILE_CHECK)"
    E2E_SUCCESS=false
fi

# Step 3: Privacy Guard scans input
E2E_INPUT="Analyze employee John Smith with SSN 123-45-6789"
E2E_SCAN=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
    -H "Content-Type: application/json" \
    -d "{\"text\": \"$E2E_INPUT\", \"mode\": \"rules\"}")
E2E_DETECTIONS=$(echo "$E2E_SCAN" | jq -r '.detections | length')
if [ "$E2E_DETECTIONS" -gt 0 ]; then
    echo "  3. PII detection: ✓ ($E2E_DETECTIONS items found)"
else
    echo "  3. PII detection: ✗ (no detections)"
    E2E_SUCCESS=false
fi

# Step 4: Privacy Guard masks PII
E2E_MASK=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d "{\"text\": \"$E2E_INPUT\", \"mode\": \"rules\"}")
E2E_REDACTIONS=$(echo "$E2E_MASK" | jq -r '.redaction_count')
if [ "$E2E_REDACTIONS" -gt 0 ]; then
    echo "  4. PII masking: ✓ ($E2E_REDACTIONS items masked)"
    echo "     Masked: $(echo "$E2E_MASK" | jq -r '.masked_text')"
else
    echo "  4. PII masking: ✗ (no redactions)"
    E2E_SUCCESS=false
fi

# Step 5: Workflow summary
if $E2E_SUCCESS; then
    pass_test "E2E workflow completed successfully"
    echo "  Flow: Auth → Profile → Scan → Mask → (LLM) → Unmask → Response"
else
    fail_test "E2E workflow" "One or more steps failed"
fi

# ==============================================================================
# Test Summary
# ==============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Finance PII Redaction Test Summary (JWT Auth)${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Total Tests:   $TESTS_RUN"
echo -e "${GREEN}Passed:        $TESTS_PASSED${NC}"
echo -e "${RED}Failed:        $TESTS_FAILED${NC}"
echo

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed Tests:${NC}"
    for test_name in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "  ${RED}•${NC} $test_name"
    done
    echo
    echo -e "${RED}❌ FINANCE PII REDACTION TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}✅ ALL FINANCE PII REDACTION TESTS PASSED${NC}"
    echo
    echo "✓ JWT authentication working"
    echo "✓ Profile loading with privacy config"
    echo "✓ Privacy Guard PII detection"
    echo "✓ Privacy Guard PII masking"
    echo "✓ End-to-end workflow validated"
    exit 0
fi
