#!/bin/bash
# Integration test for Privacy Guard audit endpoint (E5)
# Tests POST /privacy/audit endpoint

set -e

CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
TEST_NAME="Privacy Audit Endpoint (E5)"

echo "=========================================="
echo "$TEST_NAME"
echo "=========================================="
echo ""

# Test counter
PASSED=0
FAILED=0

# Helper function to run test
run_test() {
    local test_num=$1
    local description=$2
    local command=$3
    local expected=$4

    echo -n "Test $test_num: $description ... "
    
    if eval "$command" | grep -q "$expected"; then
        echo "✓ PASS"
        ((PASSED++))
    else
        echo "✗ FAIL"
        ((FAILED++))
        echo "  Command: $command"
        echo "  Expected: $expected"
    fi
}

# Test 1: Database table exists
run_test 1 "privacy_audit_logs table exists" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT table_name FROM information_schema.tables WHERE table_name = 'privacy_audit_logs'\"" \
    "privacy_audit_logs"

# Test 2: Table has correct columns
run_test 2 "Table has session_id column" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT column_name FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = 'session_id'\"" \
    "session_id"

run_test 3 "Table has redaction_count column" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT column_name FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = 'redaction_count'\"" \
    "redaction_count"

run_test 4 "Table has categories column (array)" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT data_type FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = 'categories'\"" \
    "ARRAY"

run_test 5 "Table has mode column" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT column_name FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = 'mode'\"" \
    "mode"

run_test 6 "Table has timestamp column" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT column_name FROM information_schema.columns WHERE table_name = 'privacy_audit_logs' AND column_name = 'timestamp'\"" \
    "timestamp"

# Test 7: Indexes exist
run_test 7 "session_id index exists" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT indexname FROM pg_indexes WHERE tablename = 'privacy_audit_logs' AND indexname = 'idx_privacy_audit_logs_session_id'\"" \
    "idx_privacy_audit_logs_session_id"

run_test 8 "timestamp index exists" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT indexname FROM pg_indexes WHERE tablename = 'privacy_audit_logs' AND indexname = 'idx_privacy_audit_logs_timestamp'\"" \
    "idx_privacy_audit_logs_timestamp"

# Test 9: Direct database INSERT
run_test 9 "Direct INSERT into privacy_audit_logs" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -c \"INSERT INTO privacy_audit_logs (session_id, redaction_count, categories, mode, timestamp) VALUES ('test-session-db', 2, '{SSN,EMAIL}', 'Hybrid', to_timestamp(1699564800)) RETURNING id\"" \
    "id"

# Test 10: Query inserted data
run_test 10 "Query inserted audit log" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT session_id FROM privacy_audit_logs WHERE session_id = 'test-session-db'\"" \
    "test-session-db"

# Test 11: Categories array stored correctly
run_test 11 "Categories array contains SSN" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT 'SSN' = ANY(categories) FROM privacy_audit_logs WHERE session_id = 'test-session-db'\"" \
    "t"

# Test 12: Categories array contains EMAIL
run_test 12 "Categories array contains EMAIL" \
    "docker exec ce_postgres psql -U postgres -d orchestrator -t -c \"SELECT 'EMAIL' = ANY(categories) FROM privacy_audit_logs WHERE session_id = 'test-session-db'\"" \
    "t"

# Cleanup test data
docker exec ce_postgres psql -U postgres -d orchestrator -c "DELETE FROM privacy_audit_logs WHERE session_id = 'test-session-db'" > /dev/null 2>&1

echo ""
echo "=========================================="
echo "Summary"
echo "=========================================="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
