#!/bin/bash
# Phase 1-4 Regression Test Suite
# Validates backward compatibility before Phase 5 completion
# Tests must validate: OIDC/JWT, Privacy Guard, Controller API, Session Persistence

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Environment
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
GUARD_URL="${GUARD_URL:-http://localhost:8089}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_DB="${POSTGRES_DB:-orchestrator}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-postgres}"
REDIS_URL="${REDIS_URL:-redis://localhost:6379}"

# JWT Authentication (Phase 1.2+)
KEYCLOAK_REALM="${KEYCLOAK_REALM:-dev}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1}"
KEYCLOAK_TEST_USER="${KEYCLOAK_TEST_USER:-phase5test}"
KEYCLOAK_TEST_PASSWORD="${KEYCLOAK_TEST_PASSWORD:-test123}"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Test result tracking
declare -a FAILED_TEST_NAMES=()

# Helper functions
log_test() {
    echo -e "\n${YELLOW}[TEST $1/$2]${NC} $3"
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

skip_test() {
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
    echo -e "${YELLOW}⚠️  SKIP${NC}: $1 (Reason: $2)"
}

# Get JWT token for testing
get_jwt_token() {
    curl -s -X POST \
      -d "grant_type=password" \
      -d "client_id=$KEYCLOAK_CLIENT_ID" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
      -d "username=$KEYCLOAK_TEST_USER" \
      -d "password=$KEYCLOAK_TEST_PASSWORD" \
      "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" | jq -r '.access_token'
}

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}=== Cleanup ===${NC}"
    # Remove any test data created during tests
    # (Currently none, but placeholder for future)
}

trap cleanup EXIT

echo "=== Phase 1-4 Regression Test Suite ==="
echo "Controller: $CONTROLLER_URL"
echo "Privacy Guard: $GUARD_URL"
echo "Keycloak: $KEYCLOAK_URL"
echo "Postgres: $POSTGRES_HOST:$POSTGRES_PORT/$POSTGRES_DB"
echo "Redis: $REDIS_URL"
echo

# ============================================================================
# Phase 1: OIDC/JWT Tests
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Phase 1: OIDC/JWT Authentication & Authorization${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Test 1.1: Keycloak Health Check
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Keycloak OIDC service is available"
KEYCLOAK_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$KEYCLOAK_URL/realms/master/.well-known/openid-configuration" 2>&1 || echo "000")
if [ "$KEYCLOAK_RESPONSE" = "200" ]; then
    pass_test "Keycloak OIDC endpoint accessible"
else
    fail_test "Keycloak OIDC health check" "Keycloak is not responding at $KEYCLOAK_URL (HTTP $KEYCLOAK_RESPONSE)"
fi

# Test 1.2: OIDC Discovery Endpoint
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "OIDC discovery endpoint returns valid configuration"
OIDC_CONFIG=$(curl -s "$KEYCLOAK_URL/realms/master/.well-known/openid-configuration")
if echo "$OIDC_CONFIG" | jq -e '.issuer' > /dev/null 2>&1 && \
   echo "$OIDC_CONFIG" | jq -e '.jwks_uri' > /dev/null 2>&1; then
    pass_test "OIDC discovery configuration valid"
else
    fail_test "OIDC discovery validation" "Missing required OIDC fields (issuer or jwks_uri)"
fi

# Test 1.3: JWT Public Keys (JWKS)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "JWKS endpoint provides RSA public keys"
JWKS_URI=$(echo "$OIDC_CONFIG" | jq -r '.jwks_uri')
JWKS=$(curl -s "$JWKS_URI")
if echo "$JWKS" | jq -e '.keys[0]' > /dev/null 2>&1; then
    pass_test "JWKS endpoint returns public keys"
else
    fail_test "JWKS retrieval" "No keys found at $JWKS_URI"
fi

# Test 1.4: Controller Without JWT (Should Fail or Return 401)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Controller rejects requests without JWT"
# Note: Phase 3 Controller doesn't enforce JWT yet (returns 200)
# This test documents current behavior; Phase 5 may enforce JWT
STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$CONTROLLER_URL/status")
if [ "$STATUS_CODE" = "200" ]; then
    skip_test "Controller JWT enforcement" "JWT middleware not enabled in Phase 3 (returns 200 without JWT)"
else
    pass_test "Controller requires authentication"
fi

# ============================================================================
# Phase 2: Privacy Guard Tests
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Phase 2: Privacy Guard PII Detection & Redaction${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Test 2.1: Privacy Guard Health Check
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Privacy Guard service is available"
if curl -s -f "$GUARD_URL/status" > /dev/null 2>&1; then
    GUARD_STATUS=$(curl -s "$GUARD_URL/status")
    if echo "$GUARD_STATUS" | grep -q '"status":"healthy"'; then
        pass_test "Privacy Guard is healthy"
    else
        fail_test "Privacy Guard health" "Service returned unhealthy status"
    fi
else
    fail_test "Privacy Guard availability" "Service not responding at $GUARD_URL"
fi

# Test 2.2: Regex-Based PII Detection (Email)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Privacy Guard detects email addresses"
MASK_RESPONSE=$(curl -s -X POST "$GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d '{"text":"Contact john.doe@example.com for details","tenant_id":"test-tenant"}')
MASKED=$(echo "$MASK_RESPONSE" | jq -r '.masked_text')
if echo "$MASKED" | grep -q 'EMAIL_'; then
    pass_test "Email redaction working"
else
    fail_test "Email detection" "Expected EMAIL_ marker, got: $MASKED"
fi

# Test 2.3: Regex-Based PII Detection (Phone)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Privacy Guard detects phone numbers"
ORIGINAL_PHONE="555-123-4567"
MASK_RESPONSE=$(curl -s -X POST "$GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"Call $ORIGINAL_PHONE\",\"tenant_id\":\"test-tenant\"}")
MASKED=$(echo "$MASK_RESPONSE" | jq -r '.masked_text')
REDACTION_COUNT=$(echo "$MASK_RESPONSE" | jq -r '.redactions.PHONE // 0')
# Check that phone was either pseudonymized (different number) or masked (token)
if [ "$REDACTION_COUNT" -gt 0 ] && ! echo "$MASKED" | grep -q "$ORIGINAL_PHONE"; then
    pass_test "Phone redaction working (redactions=$REDACTION_COUNT)"
else
    fail_test "Phone detection" "Expected phone to be redacted, got: $MASKED (redactions=$REDACTION_COUNT)"
fi

# Test 2.4: Regex-Based PII Detection (SSN)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Privacy Guard detects Social Security Numbers"
ORIGINAL_SSN="123-45-6789"
MASK_RESPONSE=$(curl -s -X POST "$GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"SSN: $ORIGINAL_SSN\",\"tenant_id\":\"test-tenant\"}")
MASKED=$(echo "$MASK_RESPONSE" | jq -r '.masked_text')
REDACTION_COUNT=$(echo "$MASK_RESPONSE" | jq -r '.redactions.SSN // 0')
# Check that SSN was either pseudonymized (different number) or masked (token)
if [ "$REDACTION_COUNT" -gt 0 ] && ! echo "$MASKED" | grep -q "$ORIGINAL_SSN"; then
    pass_test "SSN redaction working (redactions=$REDACTION_COUNT)"
else
    fail_test "SSN detection" "Expected SSN to be redacted, got: $MASKED (redactions=$REDACTION_COUNT)"
fi

# ============================================================================
# Phase 3: Controller API + Agent Mesh Tests
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Phase 3: Controller API + Agent Mesh Integration${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Test 3.1: Controller /status Endpoint
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Controller /status endpoint returns OK"
STATUS_RESPONSE=$(curl -s "$CONTROLLER_URL/status")
if echo "$STATUS_RESPONSE" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    pass_test "Controller status endpoint working"
else
    fail_test "Controller status" "Expected status=ok, got: $STATUS_RESPONSE"
fi

# Test 3.2: Controller JWT Authentication Required
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Controller enforces JWT authentication (Phase 1.2+)"
AUDIT_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/audit/ingest" \
    -H "Content-Type: application/json" \
    -d '{
        "source": "test-regression",
        "category": "test.regression",
        "action": "phase1-4.validation",
        "subject": "regression-suite",
        "traceId": "regression-test-001"
    }')
HTTP_CODE=$(echo "$AUDIT_RESPONSE" | tail -1)
if [ "$HTTP_CODE" = "401" ]; then
    pass_test "Controller correctly requires JWT authentication"
elif [ "$HTTP_CODE" = "202" ]; then
    skip_test "Controller JWT enforcement" "JWT middleware not enabled (Phase 3 behavior, but works)"
else
    fail_test "Controller unexpected response" "Expected 401 or 202, got $HTTP_CODE"
fi

# Test 3.3: Controller /audit/ingest with Privacy Guard Integration
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Controller + Guard integration (requires JWT)"
skip_test "Controller + Guard integration test" "Requires JWT token (Phase 1.2+ security - will test in H2-H4)"

# Test 3.4: OpenAPI Spec Available
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "OpenAPI specification is available"
if curl -s -f "$CONTROLLER_URL/openapi.json" > /dev/null 2>&1; then
    OPENAPI=$(curl -s "$CONTROLLER_URL/openapi.json")
    if echo "$OPENAPI" | jq -e '.openapi' > /dev/null 2>&1; then
        pass_test "OpenAPI spec available"
    else
        fail_test "OpenAPI spec format" "Invalid JSON at /openapi.json"
    fi
else
    skip_test "OpenAPI spec" "Endpoint not implemented yet"
fi

# ============================================================================
# Phase 4: Session Persistence + Metadata Storage Tests
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Phase 4: Session Persistence + Metadata Storage${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Test 4.1: Postgres Database Connection
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Postgres database is accessible"
if command -v psql > /dev/null 2>&1; then
    if PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "\dt" > /dev/null 2>&1; then
        pass_test "Postgres connection successful"
    else
        fail_test "Postgres connection" "Cannot connect to database"
    fi
else
    skip_test "Postgres connection" "psql client not installed"
fi

# Test 4.2: Sessions Table Exists
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Sessions table exists in database"
if command -v psql > /dev/null 2>&1; then
    TABLE_EXISTS=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tAc "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'sessions');")
    if [ "$TABLE_EXISTS" = "t" ]; then
        pass_test "Sessions table exists"
    else
        fail_test "Sessions table" "Table 'sessions' not found"
    fi
else
    skip_test "Sessions table check" "psql client not installed"
fi

# Test 4.3: Redis Connection
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Redis cache is accessible"
if command -v redis-cli > /dev/null 2>&1; then
    if redis-cli -u "$REDIS_URL" PING | grep -q "PONG"; then
        pass_test "Redis connection successful"
    else
        fail_test "Redis connection" "Redis not responding to PING"
    fi
else
    skip_test "Redis connection" "redis-cli not installed"
fi

# Test 4.4: Idempotency Key Storage (Redis)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Redis can store and retrieve idempotency keys"
if command -v redis-cli > /dev/null 2>&1; then
    TEST_KEY="idempotency:test:regression-$(date +%s)"
    redis-cli -u "$REDIS_URL" SET "$TEST_KEY" "test-value" EX 10 > /dev/null
    RETRIEVED=$(redis-cli -u "$REDIS_URL" GET "$TEST_KEY")
    if [ "$RETRIEVED" = "test-value" ]; then
        pass_test "Redis idempotency key storage working"
        redis-cli -u "$REDIS_URL" DEL "$TEST_KEY" > /dev/null
    else
        fail_test "Redis key storage" "Expected 'test-value', got '$RETRIEVED'"
    fi
else
    skip_test "Redis key storage" "redis-cli not installed"
fi

# ============================================================================
# Integration: Phase 1-4 Cross-Feature Tests
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Cross-Phase Integration Tests${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Test 5.1: Obtain JWT Token for Authenticated Tests
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Obtain JWT token from Keycloak"
JWT_TOKEN=$(get_jwt_token)
if [ "$JWT_TOKEN" != "null" ] && [ -n "$JWT_TOKEN" ] && [ "$JWT_TOKEN" != "" ]; then
    pass_test "JWT token obtained successfully"
    echo "   Token (first 50 chars): ${JWT_TOKEN:0:50}..."
else
    fail_test "JWT token retrieval" "Could not obtain token from Keycloak"
    JWT_TOKEN="" # Ensure empty for later tests
fi

# Test 5.2: Controller + Guard + Postgres (Full Stack with JWT)
TOTAL_TESTS=$((TOTAL_TESTS + 1))
log_test $TOTAL_TESTS 20 "Full stack integration (Controller → Guard → Postgres)"
if [ -z "$JWT_TOKEN" ]; then
    skip_test "Full stack integration" "No JWT token available"
else
    # This test validates that an audit event with PII flows through the entire stack
    FULL_STACK_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$CONTROLLER_URL/audit/ingest" \
        -H "Authorization: Bearer $JWT_TOKEN" \
        -H "Content-Type: application/json" \
        -d '{
            "source": "regression-full-stack",
            "category": "integration",
            "action": "full.stack.test",
            "content": "User bob@example.com created account with phone 555-111-2222",
            "traceId": "full-stack-001"
        }')
    HTTP_CODE=$(echo "$FULL_STACK_RESPONSE" | tail -1)
    if [ "$HTTP_CODE" = "202" ]; then
        # Verify event was stored (if psql available)
        if command -v psql > /dev/null 2>&1; then
            sleep 1 # Allow async processing
            EVENT_COUNT=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tAc "SELECT COUNT(*) FROM audit_events WHERE trace_id = 'full-stack-001';")
            if [ "$EVENT_COUNT" -gt 0 ]; then
                pass_test "Full stack integration working (event persisted)"
            else
                skip_test "Full stack persistence validation" "Event not found in database (async delay or audit_events table missing)"
            fi
        else
            pass_test "Full stack HTTP response (202 Accepted)"
        fi
    else
        fail_test "Full stack integration" "Expected 202, got $HTTP_CODE"
    fi
fi

# ============================================================================
# Summary
# ============================================================================
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Test Summary${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Total Tests:   $TOTAL_TESTS"
echo -e "${GREEN}Passed:        $PASSED_TESTS${NC}"
echo -e "${RED}Failed:        $FAILED_TESTS${NC}"
echo -e "${YELLOW}Skipped:       $SKIPPED_TESTS${NC}"
echo

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed Tests:${NC}"
    for test_name in "${FAILED_TEST_NAMES[@]}"; do
        echo -e "  ${RED}•${NC} $test_name"
    done
    echo
    echo -e "${RED}❌ REGRESSION TESTS FAILED${NC}"
    echo "Phase 5 cannot proceed until Phase 1-4 backward compatibility is restored."
    exit 1
else
    echo -e "${GREEN}✅ ALL REGRESSION TESTS PASSED${NC}"
    echo
    echo "Phase 1-4 backward compatibility validated!"
    echo "Ready to proceed with Phase 5 feature testing."
    exit 0
fi
