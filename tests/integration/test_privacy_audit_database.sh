#!/bin/bash
# Integration test for Privacy Guard audit database schema (E5)
# Tests privacy_audit_logs table structure

set -e

TEST_NAME="Privacy Audit Database Schema (E5)"

echo "=========================================="
echo "$TEST_NAME"
echo "=========================================="
echo ""

# Test counter
PASSED=0
FAILED=0

# Test 1: Table exists
echo -n "Test 1: privacy_audit_logs table exists ... "
RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'privacy_audit_logs'" | tr -d ' ')
if [ "$RESULT" = "1" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL (count: $RESULT)"
    ((FAILED++))
fi

# Test 2-7: Columns exist
for col in id session_id redaction_count categories mode timestamp created_at; do
    ((TEST_NUM=PASSED+FAILED+1))
    echo -n "Test $TEST_NUM: Column '$col' exists ... "
    RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = '$col'" | tr -d ' ')
    if [ "$RESULT" = "1" ]; then
        echo "✓ PASS"
        ((PASSED++))
    else
        echo "✗ FAIL"
        ((FAILED++))
    fi
done

# Test 8: categories column is ARRAY type
echo -n "Test 8: categories is ARRAY type ... "
RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT data_type FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = 'categories'" | tr -d ' ')
if [ "$RESULT" = "ARRAY" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL (type: $RESULT)"
    ((FAILED++))
fi

# Test 9-12: Indexes exist
for idx in idx_privacy_audit_logs_session_id idx_privacy_audit_logs_timestamp idx_privacy_audit_logs_mode idx_privacy_audit_logs_created_at; do
    ((TEST_NUM=PASSED+FAILED+1))
    echo -n "Test $TEST_NUM: Index '$idx' exists ... "
    RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM pg_indexes WHERE tablename = 'privacy_audit_logs' AND indexname = '$idx'" | tr -d ' ')
    if [ "$RESULT" = "1" ]; then
        echo "✓ PASS"
        ((PASSED++))
    else
        echo "✗ FAIL"
        ((FAILED++))
    fi
done

# Test 13: INSERT test data
echo -n "Test 13: INSERT audit log ... "
INSERT_RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "INSERT INTO privacy_audit_logs (session_id, redaction_count, categories, mode, timestamp) VALUES ('test-e5-integration', 3, '{SSN,EMAIL,PHONE}', 'Hybrid', to_timestamp(1699564800)) RETURNING id" | tr -d ' ')
if [ -n "$INSERT_RESULT" ]; then
    echo "✓ PASS (id: $INSERT_RESULT)"
    ((PASSED++))
else
    echo "✗ FAIL"
    ((FAILED++))
fi

# Test 14: Query audit log
echo -n "Test 14: Query inserted audit log ... "
QUERY_RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT session_id FROM privacy_audit_logs WHERE session_id = 'test-e5-integration'" | tr -d ' ')
if [ "$QUERY_RESULT" = "test-e5-integration" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL (result: $QUERY_RESULT)"
    ((FAILED++))
fi

# Test 15: Redaction count stored correctly
echo -n "Test 15: Redaction count = 3 ... "
COUNT_RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT redaction_count FROM privacy_audit_logs WHERE session_id = 'test-e5-integration'" | tr -d ' ')
if [ "$COUNT_RESULT" = "3" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL (count: $COUNT_RESULT)"
    ((FAILED++))
fi

# Test 16: Categories array validation
echo -n "Test 16: Categories contains SSN ... "
SSN_CHECK=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT 'SSN' = ANY(categories) FROM privacy_audit_logs WHERE session_id = 'test-e5-integration'" | tr -d ' ')
if [ "$SSN_CHECK" = "t" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL"
    ((FAILED++))
fi

# Test 17: Mode stored correctly
echo -n "Test 17: Mode = 'Hybrid' ... "
MODE_RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT mode FROM privacy_audit_logs WHERE session_id = 'test-e5-integration'" | tr -d ' ')
if [ "$MODE_RESULT" = "Hybrid" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL (mode: $MODE_RESULT)"
    ((FAILED++))
fi

# Test 18: Timestamp conversion
echo -n "Test 18: Timestamp converted from Unix epoch ... "
TS_RESULT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c "SELECT extract(epoch from timestamp)::bigint FROM privacy_audit_logs WHERE session_id = 'test-e5-integration'" | tr -d ' ')
if [ "$TS_RESULT" = "1699564800" ]; then
    echo "✓ PASS"
    ((PASSED++))
else
    echo "✗ FAIL (timestamp: $TS_RESULT)"
    ((FAILED++))
fi

# Cleanup
docker exec ce_postgres psql -U postgres -d orchestrator -c "DELETE FROM privacy_audit_logs WHERE session_id LIKE 'test-e5%'" > /dev/null 2>&1

echo ""
echo "=========================================="
echo "Summary"
echo "=========================================="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All tests passed!"
    echo ""
    echo "Next: Test HTTP endpoint with curl (requires controller rebuild)"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
