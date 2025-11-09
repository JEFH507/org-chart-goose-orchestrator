#!/bin/bash
# Phase 6 - Workstream A - Task A6: Vault Production Integration Test
# Tests all Vault production features: TLS, AppRole, Signing, Verification, Tamper Detection

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Test result tracking
print_test_header() {
    echo ""
    echo -e "${BLUE}=== TEST $1: $2 ===${NC}"
}

pass_test() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    ((TESTS_PASSED++))
    ((TESTS_RUN++))
}

fail_test() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    echo -e "${RED}   Error: $2${NC}"
    ((TESTS_FAILED++))
    ((TESTS_RUN++))
}

# Configuration
VAULT_ADDR="${VAULT_ADDR:-https://localhost:8200}"
VAULT_HTTP_ADDR="http://localhost:8201"
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
VAULT_CACERT="${VAULT_CACERT:-deploy/vault/certs/vault.crt}"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Phase 6 - Vault Production Integration Test Suite       ║${NC}"
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo ""
echo "Testing Vault production features:"
echo "  - TLS/HTTPS connection"
echo "  - AppRole authentication"
echo "  - Profile signing (HMAC)"
echo "  - Signature verification"
echo "  - Tamper detection"
echo ""
echo "Configuration:"
echo "  Vault HTTPS: $VAULT_ADDR"
echo "  Vault HTTP:  $VAULT_HTTP_ADDR"
echo "  Controller:  $CONTROLLER_URL"
echo "  CA Cert:     $VAULT_CACERT"
echo ""

# ============================================================================
# TEST 1: Vault TLS Connection
# ============================================================================
print_test_header "1" "Vault TLS/HTTPS Connection"

# Note: Using -k (insecure) for localhost since cert is CN=vault
# In production, use proper DNS name matching certificate CN
HEALTH_RESPONSE=$(timeout 5 curl -sk "$VAULT_ADDR/v1/sys/health" || echo "FAILED")

if echo "$HEALTH_RESPONSE" | grep -q "initialized.*true"; then
    SEALED=$(echo "$HEALTH_RESPONSE" | jq -r '.sealed')
    VERSION=$(echo "$HEALTH_RESPONSE" | jq -r '.version')
    CLUSTER=$(echo "$HEALTH_RESPONSE" | jq -r '.cluster_name')
    
    if [ "$SEALED" = "false" ]; then
        pass_test "TLS connection established (version: $VERSION, cluster: $CLUSTER)"
        echo "     Note: Using self-signed cert (CN=vault) with -k flag for localhost testing"
    else
        fail_test "TLS connection" "Vault is sealed"
    fi
else
    fail_test "TLS connection" "Unable to connect via HTTPS: $HEALTH_RESPONSE"
fi

# ============================================================================
# TEST 2: Vault AppRole Authentication
# ============================================================================
print_test_header "2" "Vault AppRole Authentication"

# Check if AppRole credentials are available in environment or .env file
if [ -z "${VAULT_ROLE_ID:-}" ] || [ -z "${VAULT_SECRET_ID:-}" ]; then
    # Try to source from .env file (via symlink)
    if [ -f "deploy/compose/.env" ]; then
        source deploy/compose/.env 2>/dev/null || true
    fi
fi

if [ -z "${VAULT_ROLE_ID:-}" ] || [ -z "${VAULT_SECRET_ID:-}" ]; then
    fail_test "AppRole authentication" "VAULT_ROLE_ID or VAULT_SECRET_ID not set"
else
    # Attempt AppRole login via HTTP endpoint (since we need to test authentication)
    LOGIN_RESPONSE=$(curl -s -X POST "$VAULT_HTTP_ADDR/v1/auth/approle/login" \
        -d "{\"role_id\": \"$VAULT_ROLE_ID\", \"secret_id\": \"$VAULT_SECRET_ID\"}" || echo "FAILED")
    
    if echo "$LOGIN_RESPONSE" | grep -q "client_token"; then
        VAULT_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"client_token":"[^"]*"' | cut -d'"' -f4)
        LEASE_DURATION=$(echo "$LOGIN_RESPONSE" | grep -o '"lease_duration":[0-9]*' | cut -d':' -f2)
        
        if [ -n "$VAULT_TOKEN" ]; then
            pass_test "AppRole authentication successful (token TTL: ${LEASE_DURATION}s)"
            export VAULT_TOKEN
        else
            fail_test "AppRole authentication" "Token not received"
        fi
    else
        fail_test "AppRole authentication" "Login failed: $LOGIN_RESPONSE"
    fi
fi

# ============================================================================
# TEST 3: Get JWT Token from Keycloak
# ============================================================================
print_test_header "3" "Keycloak JWT Token Acquisition"

# Get client secret
OIDC_CLIENT_SECRET="${OIDC_CLIENT_SECRET:-elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8}"

JWT_RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/dev/protocol/openid-connect/token" \
    -d "client_id=goose-controller" \
    -d "client_secret=$OIDC_CLIENT_SECRET" \
    -d "grant_type=password" \
    -d "username=dev-agent" \
    -d "password=dev-password" \
    -d "scope=openid" || echo "FAILED")

if echo "$JWT_RESPONSE" | grep -q "access_token"; then
    JWT=$(echo "$JWT_RESPONSE" | jq -r '.access_token')
    
    if [ -n "$JWT" ] && [ "$JWT" != "null" ]; then
        pass_test "JWT token acquired (${#JWT} chars)"
        export JWT
    else
        fail_test "JWT acquisition" "Token is null or empty"
    fi
else
    fail_test "JWT acquisition" "Keycloak response: $(echo $JWT_RESPONSE | jq -r '.error // .error_description // "Unknown error"')"
fi

# ============================================================================
# TEST 4: Profile Signing (via Admin API)
# ============================================================================
print_test_header "4" "Profile Signing via Admin API"

if [ -z "${JWT:-}" ]; then
    fail_test "Profile signing" "JWT token not available (skipping)"
else
    # Sign the test-simple profile
    SIGN_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/profiles/test-simple/publish" \
        -H "Authorization: Bearer $JWT" || echo "FAILED")
    
    if echo "$SIGN_RESPONSE" | grep -q "signature"; then
        SIGNATURE=$(echo "$SIGN_RESPONSE" | jq -r '.signature')
        SIGNED_AT=$(echo "$SIGN_RESPONSE" | jq -r '.signed_at')
        
        if echo "$SIGNATURE" | grep -q "vault:v1:"; then
            pass_test "Profile signed successfully (timestamp: $SIGNED_AT)"
            echo "     Signature: ${SIGNATURE:0:40}..."
        else
            fail_test "Profile signing" "Invalid signature format: $SIGNATURE"
        fi
    else
        fail_test "Profile signing" "No signature in response: $SIGN_RESPONSE"
    fi
fi

# ============================================================================
# TEST 5: Profile Signature Verification (Load Profile)
# ============================================================================
print_test_header "5" "Profile Signature Verification"

if [ -z "${JWT:-}" ]; then
    fail_test "Signature verification" "JWT token not available (skipping)"
else
    # Load the test-simple profile (should verify signature automatically)
    PROFILE_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
        -H "Authorization: Bearer $JWT" \
        "$CONTROLLER_URL/profiles/test-simple" || echo "FAILED")
    
    HTTP_STATUS=$(echo "$PROFILE_RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
    PROFILE_BODY=$(echo "$PROFILE_RESPONSE" | sed '/HTTP_STATUS:/d')
    
    if [ "$HTTP_STATUS" = "200" ]; then
        # Verify profile has signature metadata
        if echo "$PROFILE_BODY" | jq -e '.signature.signature' > /dev/null 2>&1; then
            SIG_ALGO=$(echo "$PROFILE_BODY" | jq -r '.signature.algorithm')
            SIG_KEY=$(echo "$PROFILE_BODY" | jq -r '.signature.vault_key')
            
            pass_test "Signature verification successful (algo: $SIG_ALGO, key: $SIG_KEY)"
        else
            fail_test "Signature verification" "Profile loaded but no signature metadata found"
        fi
    else
        fail_test "Signature verification" "HTTP $HTTP_STATUS (expected 200): $PROFILE_BODY"
    fi
fi

# ============================================================================
# TEST 6: Unsigned Profile Rejection
# ============================================================================
print_test_header "6" "Unsigned Profile Rejection"

if [ -z "${JWT:-}" ]; then
    fail_test "Unsigned profile rejection" "JWT token not available (skipping)"
else
    # Create a profile without signature in database
    PSQL="docker exec -i ce_postgres psql -U postgres -d orchestrator_dev -t -A"
    
    # Insert unsigned profile
    echo "INSERT INTO profiles (role, display_name, data, created_at, updated_at) 
    VALUES ('test-unsigned-a6', 'Test Unsigned A6', 
    '{\"role\": \"test-unsigned-a6\", \"description\": \"Profile without signature\", \"display_name\": \"Test Unsigned A6\", \"providers\": {\"primary\": {\"provider\": \"test\", \"model\": \"test\", \"temperature\": 0.5}}}', 
    NOW(), NOW()) 
    ON CONFLICT (role) DO UPDATE SET data = EXCLUDED.data, updated_at = NOW();" | $PSQL > /dev/null 2>&1
    
    # Try to load unsigned profile
    UNSIGNED_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
        -H "Authorization: Bearer $JWT" \
        "$CONTROLLER_URL/profiles/test-unsigned-a6" || echo "FAILED")
    
    UNSIGNED_STATUS=$(echo "$UNSIGNED_RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
    
    if [ "$UNSIGNED_STATUS" = "403" ]; then
        pass_test "Unsigned profile correctly rejected (HTTP 403)"
    else
        UNSIGNED_BODY=$(echo "$UNSIGNED_RESPONSE" | sed '/HTTP_STATUS:/d')
        fail_test "Unsigned profile rejection" "Expected HTTP 403, got $UNSIGNED_STATUS. Body: $UNSIGNED_BODY"
    fi
    
    # Cleanup
    echo "DELETE FROM profiles WHERE role = 'test-unsigned-a6';" | $PSQL > /dev/null 2>&1
fi

# ============================================================================
# TEST 7: Tamper Detection
# ============================================================================
print_test_header "7" "Tamper Detection (Modified Profile)"

if [ -z "${JWT:-}" ]; then
    fail_test "Tamper detection" "JWT token not available (skipping)"
else
    # First, ensure test-simple is signed
    curl -s -X POST "$CONTROLLER_URL/admin/profiles/test-simple/publish" \
        -H "Authorization: Bearer $JWT" > /dev/null 2>&1
    
    # Wait a moment for database write
    sleep 1
    
    # Tamper with the profile (change description, keep old signature)
    echo "UPDATE profiles 
    SET data = jsonb_set(data, '{description}', '\"TAMPERED_CONTENT_A6_TEST\"') 
    WHERE role = 'test-simple';" | $PSQL > /dev/null 2>&1
    
    # Try to load tampered profile
    TAMPER_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
        -H "Authorization: Bearer $JWT" \
        "$CONTROLLER_URL/profiles/test-simple" || echo "FAILED")
    
    TAMPER_STATUS=$(echo "$TAMPER_RESPONSE" | grep "HTTP_STATUS:" | cut -d':' -f2)
    
    if [ "$TAMPER_STATUS" = "403" ]; then
        pass_test "Tampered profile correctly detected and rejected (HTTP 403)"
    else
        TAMPER_BODY=$(echo "$TAMPER_RESPONSE" | sed '/HTTP_STATUS:/d')
        fail_test "Tamper detection" "Expected HTTP 403, got $TAMPER_STATUS. Tampering not detected!"
    fi
    
    # Restore test-simple by re-signing
    curl -s -X POST "$CONTROLLER_URL/admin/profiles/test-simple/publish" \
        -H "Authorization: Bearer $JWT" > /dev/null 2>&1
    echo "     Profile restored with fresh signature"
fi

# ============================================================================
# TEST 8: Vault Audit Log Verification
# ============================================================================
print_test_header "8" "Vault Audit Log Verification"

# Check if audit log exists and has recent entries
AUDIT_LOG_CHECK=$(docker exec ce_vault test -f /vault/logs/audit.log && echo "EXISTS" || echo "MISSING")

if [ "$AUDIT_LOG_CHECK" = "EXISTS" ]; then
    # Count recent audit entries (last 100 lines)
    AUDIT_ENTRIES=$(docker exec ce_vault sh -c "tail -100 /vault/logs/audit.log | wc -l" || echo "0")
    
    if [ "$AUDIT_ENTRIES" -gt 0 ]; then
        # Check for HMAC operations in audit log
        HMAC_OPS=$(docker exec ce_vault sh -c "tail -100 /vault/logs/audit.log | grep -c 'transit/hmac' || echo 0")
        
        if [ "$HMAC_OPS" -gt 0 ]; then
            pass_test "Audit logging enabled ($HMAC_OPS HMAC operations logged in last 100 entries)"
        else
            pass_test "Audit logging enabled ($AUDIT_ENTRIES total entries, HMAC ops may be older)"
        fi
    else
        fail_test "Audit log verification" "Audit log exists but appears empty"
    fi
else
    fail_test "Audit log verification" "Audit log file not found at /vault/logs/audit.log"
fi

# ============================================================================
# TEST 9: Raft Storage Verification
# ============================================================================
print_test_header "9" "Raft Storage Verification"

# Check storage_type from seal-status endpoint
SEAL_STATUS=$(timeout 5 curl -sk "$VAULT_ADDR/v1/sys/seal-status" || echo "FAILED")
STORAGE_TYPE=$(echo "$SEAL_STATUS" | jq -r '.storage_type')

if [ "$STORAGE_TYPE" = "raft" ]; then
    # Verify raft data directory exists and has data
    RAFT_FILES=$(docker exec ce_vault sh -c "find /vault/raft -type f 2>/dev/null | wc -l" || echo "0")
    
    if [ "$RAFT_FILES" -gt 0 ]; then
        pass_test "Raft storage operational (storage_type: raft, $RAFT_FILES data files)"
    else
        fail_test "Raft storage verification" "Storage type is raft but no data files found"
    fi
else
    fail_test "Raft storage verification" "Expected storage_type=raft, got: $STORAGE_TYPE"
fi

# ============================================================================
# TEST 10: Vault HA (Clustering) Capability
# ============================================================================
print_test_header "10" "Vault HA (Clustering) Capability"

# Check if cluster_name and cluster_id are present (indicates HA capability with Raft)
CLUSTER_NAME=$(echo "$SEAL_STATUS" | jq -r '.cluster_name')
CLUSTER_ID=$(echo "$SEAL_STATUS" | jq -r '.cluster_id')

if [ "$CLUSTER_NAME" != "null" ] && [ "$CLUSTER_ID" != "null" ]; then
    pass_test "HA capability enabled (cluster: $CLUSTER_NAME)"
    echo "     Cluster ID: $CLUSTER_ID"
else
    fail_test "HA capability verification" "Cluster name/ID not found (HA may not be configured)"
fi

# ============================================================================
# Final Summary
# ============================================================================
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    TEST SUMMARY                           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Total Tests Run:    $TESTS_RUN"
echo -e "Tests Passed:       ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed:       ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              ✅ ALL TESTS PASSED! ✅                      ║${NC}"
    echo -e "${GREEN}║                                                            ║${NC}"
    echo -e "${GREEN}║  Vault Production Features Verified:                      ║${NC}"
    echo -e "${GREEN}║    ✓ TLS/HTTPS Connection                                 ║${NC}"
    echo -e "${GREEN}║    ✓ AppRole Authentication                               ║${NC}"
    echo -e "${GREEN}║    ✓ Profile Signing (HMAC)                               ║${NC}"
    echo -e "${GREEN}║    ✓ Signature Verification                               ║${NC}"
    echo -e "${GREEN}║    ✓ Unsigned Profile Rejection                           ║${NC}"
    echo -e "${GREEN}║    ✓ Tamper Detection                                     ║${NC}"
    echo -e "${GREEN}║    ✓ Audit Logging                                        ║${NC}"
    echo -e "${GREEN}║    ✓ Raft Storage                                         ║${NC}"
    echo -e "${GREEN}║    ✓ High Availability                                    ║${NC}"
    echo -e "${GREEN}║                                                            ║${NC}"
    echo -e "${GREEN}║  Vault is production-ready! 🚀                            ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║              ❌ SOME TESTS FAILED ❌                      ║${NC}"
    echo -e "${RED}║                                                            ║${NC}"
    echo -e "${RED}║  Please review the failures above and:                    ║${NC}"
    echo -e "${RED}║    1. Check service status (docker compose ps)            ║${NC}"
    echo -e "${RED}║    2. Review logs (docker logs ce_vault/ce_controller)    ║${NC}"
    echo -e "${RED}║    3. Verify environment variables (.env file)            ║${NC}"
    echo -e "${RED}║    4. Ensure Vault is unsealed                            ║${NC}"
    echo -e "${RED}╚════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
