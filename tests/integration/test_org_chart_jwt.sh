#!/bin/bash
# H4: Org Chart API Integration Test (REAL E2E with JWT)
# Tests CSV import, tree API, department filtering with actual HTTP calls

set -euo pipefail

# Test configuration
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
REALM="${KEYCLOAK_REALM:-dev}"
CLIENT_ID="${KEYCLOAK_CLIENT_ID:-goose-controller}"
CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1}"
USERNAME="${TEST_USERNAME:-phase5test}"
PASSWORD="${TEST_PASSWORD:-test123}"
CSV_FILE="${CSV_FILE:-tests/integration/test_data/org_chart_sample.csv}"

# Test result counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print test result
print_result() {
    local test_name="$1"
    local result="$2"
    local message="$3"
    
    ((TESTS_RUN++)) || true
    
    if [ "$result" = "PASS" ]; then
        ((TESTS_PASSED++)) || true
        echo -e "${GREEN}✓${NC} $test_name: $message"
    else
        ((TESTS_FAILED++)) || true
        echo -e "${RED}✗${NC} $test_name: $message"
    fi
}

# Print section header
print_header() {
    echo ""
    echo "=========================================="
    echo "$1"
    echo "=========================================="
    echo ""
}

print_header "H4: Org Chart API Integration Tests"

# Test 1: Get JWT token
print_header "Test 1: JWT Authentication"
JWT_RESPONSE=$(curl -s -X POST \
    -d "grant_type=password" \
    -d "client_id=$CLIENT_ID" \
    -d "client_secret=$CLIENT_SECRET" \
    -d "username=$USERNAME" \
    -d "password=$PASSWORD" \
    "$KEYCLOAK_URL/realms/$REALM/protocol/openid-connect/token")

JWT_TOKEN=$(echo "$JWT_RESPONSE" | jq -r '.access_token // empty')

if [ -n "$JWT_TOKEN" ] && [ "$JWT_TOKEN" != "null" ]; then
    print_result "JWT Authentication" "PASS" "Got valid JWT token (${#JWT_TOKEN} chars)"
else
    print_result "JWT Authentication" "FAIL" "Failed to get JWT token: $JWT_RESPONSE"
    exit 1
fi

# Test 2: Controller health check
print_header "Test 2: Controller API Accessibility"
HEALTH_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$CONTROLLER_URL/health" || echo "000")

if [ "$HEALTH_RESPONSE" = "200" ]; then
    print_result "Controller Health" "PASS" "Controller API accessible (HTTP 200)"
else
    print_result "Controller Health" "FAIL" "Controller returned HTTP $HEALTH_RESPONSE"
fi

# Test 3: Verify CSV file exists
print_header "Test 3: CSV Test Data Availability"
if [ -f "$CSV_FILE" ]; then
    LINE_COUNT=$(wc -l < "$CSV_FILE")
    print_result "CSV File Exists" "PASS" "Found CSV file with $LINE_COUNT lines"
else
    print_result "CSV File Exists" "FAIL" "CSV file not found: $CSV_FILE"
    exit 1
fi

# Test 4: CSV upload (POST /admin/org/import)
print_header "Test 4: CSV Upload to /admin/org/import"
IMPORT_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -F "file=@$CSV_FILE" \
    "$CONTROLLER_URL/admin/org/import" || echo "")

HTTP_CODE=$(echo "$IMPORT_RESPONSE" | tail -1)
IMPORT_BODY=$(echo "$IMPORT_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "201" ]; then
    IMPORT_ID=$(echo "$IMPORT_BODY" | jq -r '.import_id // empty')
    USERS_CREATED=$(echo "$IMPORT_BODY" | jq -r '.users_created // 0')
    print_result "CSV Upload" "PASS" "Import created (ID: $IMPORT_ID, $USERS_CREATED users)"
else
    print_result "CSV Upload" "FAIL" "HTTP $HTTP_CODE: $IMPORT_BODY"
fi

# Test 5: Verify users in database
print_header "Test 5: Database Verification (org_users table)"
if command -v docker &> /dev/null; then
    USER_COUNT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users;" 2>/dev/null | xargs || echo "0")
    
    if [ "$USER_COUNT" -gt 0 ]; then
        print_result "Database Users" "PASS" "Found $USER_COUNT users in org_users table"
    else
        print_result "Database Users" "FAIL" "No users found in database"
    fi
else
    print_result "Database Users" "SKIP" "Docker not available for DB query"
fi

# Test 6: Import history (GET /admin/org/imports)
print_header "Test 6: Import History API"
IMPORTS_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -H "Authorization: Bearer $JWT_TOKEN" \
    "$CONTROLLER_URL/admin/org/imports" || echo "")

HTTP_CODE=$(echo "$IMPORTS_RESPONSE" | tail -1)
IMPORTS_BODY=$(echo "$IMPORTS_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "200" ]; then
    TOTAL_IMPORTS=$(echo "$IMPORTS_BODY" | jq -r '.total // 0')
    print_result "Import History" "PASS" "Found $TOTAL_IMPORTS import records"
else
    print_result "Import History" "FAIL" "HTTP $HTTP_CODE: $IMPORTS_BODY"
fi

# Test 7: Org tree API (GET /admin/org/tree)
print_header "Test 7: Org Chart Tree API"
TREE_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -H "Authorization: Bearer $JWT_TOKEN" \
    "$CONTROLLER_URL/admin/org/tree" || echo "")

HTTP_CODE=$(echo "$TREE_RESPONSE" | tail -1)
TREE_BODY=$(echo "$TREE_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "200" ]; then
    TOTAL_USERS=$(echo "$TREE_BODY" | jq -r '.total_users // 0')
    print_result "Org Tree API" "PASS" "Org tree returned $TOTAL_USERS users"
else
    print_result "Org Tree API" "FAIL" "HTTP $HTTP_CODE: $TREE_BODY"
fi

# Test 8: Department field in tree response
print_header "Test 8: Department Field Validation"
if [ "$HTTP_CODE" = "200" ]; then
    # Check if any user has department field
    HAS_DEPARTMENT=$(echo "$TREE_BODY" | jq -r '.tree[0].department // empty')
    
    if [ -n "$HAS_DEPARTMENT" ]; then
        print_result "Department Field" "PASS" "Department field present: $HAS_DEPARTMENT"
    else
        print_result "Department Field" "FAIL" "Department field missing in tree response"
    fi
else
    print_result "Department Field" "SKIP" "Tree API failed, cannot check department"
fi

# Test 9: Hierarchical structure validation
print_header "Test 9: Hierarchical Structure"
if [ "$HTTP_CODE" = "200" ]; then
    # Check if root user has reports (nested structure)
    ROOT_REPORTS=$(echo "$TREE_BODY" | jq -r '.tree[0].reports | length' 2>/dev/null || echo "0")
    
    if [ "$ROOT_REPORTS" -gt 0 ]; then
        print_result "Tree Hierarchy" "PASS" "Root user has $ROOT_REPORTS direct reports"
    else
        print_result "Tree Hierarchy" "FAIL" "No hierarchical structure found"
    fi
else
    print_result "Tree Hierarchy" "SKIP" "Tree API failed"
fi

# Test 10: Duplicate CSV upload (should update existing users)
print_header "Test 10: CSV Re-import (Upsert Logic)"
REIMPORT_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -F "file=@$CSV_FILE" \
    "$CONTROLLER_URL/admin/org/import" || echo "")

HTTP_CODE=$(echo "$REIMPORT_RESPONSE" | tail -1)
REIMPORT_BODY=$(echo "$REIMPORT_RESPONSE" | head -n -1)

if [ "$HTTP_CODE" = "201" ]; then
    USERS_CREATED=$(echo "$REIMPORT_BODY" | jq -r '.users_created // 0')
    USERS_UPDATED=$(echo "$REIMPORT_BODY" | jq -r '.users_updated // 0')
    
    if [ "$USERS_UPDATED" -gt 0 ] || [ "$USERS_CREATED" -eq 0 ]; then
        print_result "CSV Upsert" "PASS" "Upsert logic working (created: $USERS_CREATED, updated: $USERS_UPDATED)"
    else
        print_result "CSV Upsert" "WARN" "Expected updates, got creates: $USERS_CREATED"
    fi
else
    print_result "CSV Upsert" "FAIL" "HTTP $HTTP_CODE: $REIMPORT_BODY"
fi

# Test 11: Database audit trail (org_imports table)
print_header "Test 11: Import Audit Trail"
if command -v docker &> /dev/null; then
    IMPORT_STATUS=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT status FROM org_imports ORDER BY uploaded_at DESC LIMIT 1;" 2>/dev/null | xargs || echo "unknown")
    
    if [ "$IMPORT_STATUS" = "complete" ]; then
        print_result "Import Audit" "PASS" "Latest import status: complete"
    else
        print_result "Import Audit" "WARN" "Import status: $IMPORT_STATUS (expected: complete)"
    fi
else
    print_result "Import Audit" "SKIP" "Docker not available"
fi

# Test 12: Department filtering in database
print_header "Test 12: Department Field in Database"
if command -v docker &> /dev/null; then
    DEPT_COUNT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(DISTINCT department) FROM org_users;" 2>/dev/null | xargs || echo "0")
    
    if [ "$DEPT_COUNT" -gt 0 ]; then
        print_result "Department Data" "PASS" "Found $DEPT_COUNT unique departments in database"
    else
        print_result "Department Data" "FAIL" "No department data in database"
    fi
else
    print_result "Department Data" "SKIP" "Docker not available"
fi

# Final summary
print_header "H4 Test Results Summary"
echo "Tests Run:    $TESTS_RUN"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed.${NC}"
    exit 1
fi
