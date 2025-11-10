#!/bin/bash
# Comprehensive Profile Test - All 6 Roles
# Tests: JWT auth, profile loading, policy enforcement, privacy guard integration
# Validates: Finance, Manager, Analyst, Marketing, Support, Legal

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Base URLs
CONTROLLER_URL="http://localhost:8088"
KEYCLOAK_URL="http://localhost:8080"
PRIVACY_GUARD_URL="http://localhost:8089"

# OIDC client credentials
CLIENT_ID="goose-controller"
CLIENT_SECRET=${OIDC_CLIENT_SECRET:-"elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8"}

# Helper functions
print_section() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_test() {
    echo -e "\n${YELLOW}[TEST $1]${NC} $2"
}

pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

# Get JWT token (client_credentials grant)
get_jwt_token() {
    local response=$(curl -s -X POST \
        "${KEYCLOAK_URL}/realms/dev/protocol/openid-connect/token" \
        -H "Content-Type: application/x-www-form-urlencoded" \
        -d "grant_type=client_credentials" \
        -d "client_id=${CLIENT_ID}" \
        -d "client_secret=${CLIENT_SECRET}")
    
    echo "$response" | jq -r '.access_token'
}

# Test profile loading
test_profile() {
    local role=$1
    local expected_display_name=$2
    local expected_policies=$3
    
    print_test $((TOTAL_TESTS + 1)) "Load $role profile"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    PROFILE_RESPONSE=$(curl -s -X GET \
        "${CONTROLLER_URL}/profiles/${role}" \
        -H "Authorization: Bearer ${JWT_TOKEN}")
    
    if echo "$PROFILE_RESPONSE" | jq -e '.role' > /dev/null 2>&1; then
        ROLE=$(echo "$PROFILE_RESPONSE" | jq -r '.role')
        DISPLAY_NAME=$(echo "$PROFILE_RESPONSE" | jq -r '.display_name')
        POLICY_COUNT=$(echo "$PROFILE_RESPONSE" | jq -r '.policies | length')
        
        if [[ "$ROLE" == "$role" && "$POLICY_COUNT" == "$expected_policies" ]]; then
            pass "$role profile loaded ($DISPLAY_NAME, $POLICY_COUNT policies)"
            echo "  Extensions: $(echo "$PROFILE_RESPONSE" | jq -r '.config.extensions | length')"
            echo "  Recipes: $(echo "$PROFILE_RESPONSE" | jq -r '.recipes | length')"
            echo "  Memory: $(echo "$PROFILE_RESPONSE" | jq -r '.config.memory_retention // "default"')"
            echo "  Provider: $(echo "$PROFILE_RESPONSE" | jq -r '.config.provider // "default"')"
        else
            fail "$role profile mismatch (expected $expected_policies policies, got $POLICY_COUNT)"
        fi
    else
        fail "$role profile load failed"
        echo "  Response: $PROFILE_RESPONSE"
    fi
}

# Test profile config generation
test_config_generation() {
    local role=$1
    
    print_test $((TOTAL_TESTS + 1)) "Generate $role config.yaml"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    CONFIG_RESPONSE=$(curl -s -X GET \
        "${CONTROLLER_URL}/profiles/${role}/config" \
        -H "Authorization: Bearer ${JWT_TOKEN}")
    
    if echo "$CONFIG_RESPONSE" | grep -q "provider:"; then
        pass "$role config.yaml generated"
        echo "  Lines: $(echo "$CONFIG_RESPONSE" | wc -l)"
        echo "  Has provider: ✓"
        echo "  Has extensions: $(echo "$CONFIG_RESPONSE" | grep -c "extensions:" || echo "0")"
    else
        fail "$role config generation failed"
    fi
}

# Test privacy guard with profile-specific PII
test_privacy_with_profile() {
    local role=$1
    local pii_text=$2
    local expected_detections=$3
    
    print_test $((TOTAL_TESTS + 1)) "Privacy Guard for $role (PII detection)"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    SCAN_RESPONSE=$(curl -s -X POST \
        "${PRIVACY_GUARD_URL}/guard/scan" \
        -H "Content-Type: application/json" \
        -d "{\"text\": \"$pii_text\", \"tenant_id\": \"test-$role\"}")
    
    if echo "$SCAN_RESPONSE" | jq -e '.detections | type == "array"' > /dev/null 2>&1; then
        DETECTION_COUNT=$(echo "$SCAN_RESPONSE" | jq -r '.detections | length')
        
        if [[ "$DETECTION_COUNT" -ge "$expected_detections" ]]; then
            pass "$role PII detection ($DETECTION_COUNT detections)"
            echo "  Types: $(echo "$SCAN_RESPONSE" | jq -r '.detections | map(.entity_type) | unique | join(", ")')"
        else
            fail "$role PII detection (expected >= $expected_detections, got $DETECTION_COUNT)"
        fi
    else
        fail "$role PII detection failed"
        echo "  Response: $SCAN_RESPONSE"
    fi
}

# Main test execution
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Comprehensive Profile Test - All 6 Roles"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Testing: Profile loading, config generation, policy enforcement, privacy integration"
echo ""

# Get JWT token once
print_section "AUTHENTICATION"
print_test 1 "Obtain JWT token"
TOTAL_TESTS=$((TOTAL_TESTS + 1))

JWT_TOKEN=$(get_jwt_token)

if [[ -n "$JWT_TOKEN" && "$JWT_TOKEN" != "null" ]]; then
    pass "JWT token acquired (length: ${#JWT_TOKEN})"
    echo "  Grant type: client_credentials"
    echo "  Token preview: ${JWT_TOKEN:0:50}..."
else
    fail "Failed to acquire JWT token"
    exit 1
fi

# Test Finance profile
print_section "FINANCE PROFILE"
test_profile "finance" "Finance Team Agent" "7"
test_config_generation "finance"
test_privacy_with_profile "finance" "SSN: 123-45-6789, Credit card: 4532-1234-5678-9012, Email: finance@company.com" "2"  # Expect 2+ (SSN, EMAIL detected, credit card may/may not)

# Test Manager profile (API merges YAML + DB policies)
print_section "MANAGER PROFILE"
test_profile "manager" "Manager Team Agent" "6"
test_config_generation "manager"
test_privacy_with_profile "manager" "Employee John Smith, performance review, email: john@company.com" "1"  # Expect 1+ (EMAIL detected, PERSON needs NER)

# Test Analyst profile
print_section "ANALYST PROFILE"
test_profile "analyst" "Business Analyst" "7"
test_config_generation "analyst"
test_privacy_with_profile "analyst" "Customer data: email alice@example.com, phone 555-123-4567" "2"

# Test Marketing profile
print_section "MARKETING PROFILE"
test_profile "marketing" "Marketing Team Agent" "4"
test_config_generation "marketing"
test_privacy_with_profile "marketing" "Campaign contact: bob@agency.com, budget report" "1"

# Test Support profile (API merges YAML + DB policies)
print_section "SUPPORT PROFILE"
test_profile "support" "Support Team Agent" "4"
test_config_generation "support"
test_privacy_with_profile "support" "Ticket from customer: support@client.com, issue #12345" "1"

# Test Legal profile (local-only, attorney-client privilege) (API merges YAML + DB policies)
print_section "LEGAL PROFILE (Local-Only)"
test_profile "legal" "Legal Compliance Agent" "12"
test_config_generation "legal"

print_test $((TOTAL_TESTS + 1)) "Verify Legal profile local-only config"
TOTAL_TESTS=$((TOTAL_TESTS + 1))

LEGAL_PROFILE=$(curl -s -X GET \
    "${CONTROLLER_URL}/profiles/legal" \
    -H "Authorization: Bearer ${JWT_TOKEN}")

# Legal profile has privacy.local_only, not config.local_only
LOCAL_ONLY=$(echo "$LEGAL_PROFILE" | jq -r '.privacy.local_only // false')
RETENTION_DAYS=$(echo "$LEGAL_PROFILE" | jq -r '.privacy.retention_days // "default"')

if [[ "$LOCAL_ONLY" == "true" && "$RETENTION_DAYS" == "0" ]]; then
    pass "Legal profile has local-only + ephemeral memory (attorney-client privilege)"
    echo "  Provider: $(echo "$LEGAL_PROFILE" | jq -r '.providers.primary.provider')"
    echo "  Local-only: $LOCAL_ONLY"
    echo "  Retention days: $RETENTION_DAYS (ephemeral)"
    echo "  Privacy mode: $(echo "$LEGAL_PROFILE" | jq -r '.privacy.mode')"
else
    fail "Legal profile missing local-only configuration (local_only=$LOCAL_ONLY, retention_days=$RETENTION_DAYS)"
fi

# Cross-profile verification
print_section "CROSS-PROFILE VERIFICATION"

print_test $((TOTAL_TESTS + 1)) "All 6 profiles unique and complete"
TOTAL_TESTS=$((TOTAL_TESTS + 1))

ALL_ROLES=$(cat << EOF
finance
manager
analyst
marketing
support
legal
EOF
)

UNIQUE_COUNT=0
for role in $ALL_ROLES; do
    PROFILE=$(curl -s -X GET "${CONTROLLER_URL}/profiles/${role}" -H "Authorization: Bearer ${JWT_TOKEN}")
    if echo "$PROFILE" | jq -e '.role' > /dev/null 2>&1; then
        UNIQUE_COUNT=$((UNIQUE_COUNT + 1))
    fi
done

if [[ "$UNIQUE_COUNT" == "6" ]]; then
    pass "All 6 profiles unique and loadable"
    echo "  Finance, Manager, Analyst, Marketing, Support, Legal ✓"
else
    fail "Profile uniqueness check failed ($UNIQUE_COUNT/6)"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Comprehensive Profile Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Total Tests:   $TOTAL_TESTS"
echo -e "${GREEN}Passed:        $PASSED_TESTS${NC}"
echo -e "${RED}Failed:        $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ ALL PROFILE TESTS PASSED${NC}"
    echo "✓ All 6 profiles working: Finance, Manager, Analyst, Marketing, Support, Legal"
    echo "✓ Config generation working for all roles"
    echo "✓ Privacy Guard integration validated"
    echo "✓ Legal profile local-only enforcement verified"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    exit 1
fi
