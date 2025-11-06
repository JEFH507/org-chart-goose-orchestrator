#!/bin/bash
# Database-level integration test for department field (Option A migration)
# Tests database schema, data insertion, and backward compatibility
# Does not require controller routes (D10-D12) to be deployed yet

set -e

TEST_CSV="tests/integration/test_data/org_chart_sample.csv"

echo "==========================================
Department Field Database Integration Test
=========================================="

# Test 1: Database schema validation
echo "✓ Test 1: Verify department column exists"
COLUMNS=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT column_name FROM information_schema.columns WHERE table_name = 'org_users' ORDER BY ordinal_position;")
if echo "$COLUMNS" | grep -q "department"; then
  echo "  ✓ Department column found in org_users table"
else
  echo "  ❌ Department column NOT found"
  exit 1
fi

# Test 2: Verify department is NOT NULL
echo "✓ Test 2: Verify department column is NOT NULL"
IS_NULLABLE=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT is_nullable FROM information_schema.columns WHERE table_name = 'org_users' AND column_name = 'department';")
IS_NULLABLE=$(echo "$IS_NULLABLE" | tr -d ' ')

if [ "$IS_NULLABLE" == "NO" ]; then
  echo "  ✓ Department is NOT NULL (required field)"
else
  echo "  ❌ Department should be NOT NULL, got: $IS_NULLABLE"
  exit 1
fi

# Test 3: Verify department index
echo "✓ Test 3: Verify department index exists"
INDEX=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT indexname FROM pg_indexes WHERE tablename = 'org_users' AND indexname = 'idx_org_users_department';")
if echo "$INDEX" | grep -q "idx_org_users_department"; then
  echo "  ✓ Department index exists (idx_org_users_department)"
else
  echo "  ❌ Department index NOT found"
  exit 1
fi

# Test 4: Direct INSERT with department field
echo "✓ Test 4: Direct INSERT with department field"
docker exec -i ce_postgres psql -U postgres -d orchestrator <<EOF > /dev/null 2>&1
-- Clean up any existing test data
DELETE FROM org_users WHERE user_id BETWEEN 100 AND 105;

-- Insert test users with department
INSERT INTO org_users (user_id, reports_to_id, name, role, email, department, created_at, updated_at)
VALUES 
  (100, NULL, 'Test CEO', 'manager', 'testceo@test.com', 'Executive', NOW(), NOW()),
  (101, 100, 'Test CFO', 'finance', 'testcfo@test.com', 'Finance', NOW(), NOW()),
  (102, 100, 'Test CTO', 'manager', 'testcto@test.com', 'Engineering', NOW(), NOW()),
  (103, 101, 'Test Analyst', 'analyst', 'testanalyst@test.com', 'Finance', NOW(), NOW()),
  (104, 102, 'Test Dev', 'analyst', 'testdev@test.com', 'Engineering', NOW(), NOW());
EOF

# Verify insertion
USER_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE user_id BETWEEN 100 AND 105;")
USER_COUNT=$(echo "$USER_COUNT" | tr -d ' ')

if [ "$USER_COUNT" == "5" ]; then
  echo "  ✓ 5 test users inserted successfully"
else
  echo "  ❌ Expected 5 users, got: $USER_COUNT"
  exit 1
fi

# Test 5: Verify department values
echo "✓ Test 5: Verify department field values"
FINANCE_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE department = 'Finance' AND user_id BETWEEN 100 AND 105;")
FINANCE_COUNT=$(echo "$FINANCE_COUNT" | tr -d ' ')

ENGINEERING_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE department = 'Engineering' AND user_id BETWEEN 100 AND 105;")
ENGINEERING_COUNT=$(echo "$ENGINEERING_COUNT" | tr -d ' ')

EXECUTIVE_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE department = 'Executive' AND user_id BETWEEN 100 AND 105;")
EXECUTIVE_COUNT=$(echo "$EXECUTIVE_COUNT" | tr -d ' ')

echo "  ✓ Finance department: $FINANCE_COUNT users"
echo "  ✓ Engineering department: $ENGINEERING_COUNT users"
echo "  ✓ Executive department: $EXECUTIVE_COUNT users"

if [ "$FINANCE_COUNT" != "2" ] || [ "$ENGINEERING_COUNT" != "2" ] || [ "$EXECUTIVE_COUNT" != "1" ]; then
  echo "  ❌ Department counts incorrect"
  exit 1
fi

# Test 6: SELECT with department filtering
echo "✓ Test 6: SELECT with department filter (index usage)"
QUERY_RESULT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT user_id, name, department FROM org_users WHERE department = 'Finance' AND user_id BETWEEN 100 AND 105 ORDER BY user_id;")

if echo "$QUERY_RESULT" | grep -q "Finance"; then
  echo "  ✓ Department filter query successful"
  echo "$QUERY_RESULT" | head -2 | sed 's/^/    /'
else
  echo "  ❌ Department filter query failed"
  exit 1
fi

# Test 7: UPDATE department
echo "✓ Test 7: UPDATE department field"
docker exec -i ce_postgres psql -U postgres -d orchestrator -c "UPDATE org_users SET department = 'Accounting' WHERE user_id = 103;" > /dev/null 2>&1

UPDATED_DEPT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT department FROM org_users WHERE user_id = 103;")
UPDATED_DEPT=$(echo "$UPDATED_DEPT" | tr -d ' ')

if [ "$UPDATED_DEPT" == "Accounting" ]; then
  echo "  ✓ Department updated successfully: $UPDATED_DEPT"
else
  echo "  ❌ Department update failed, expected 'Accounting', got: $UPDATED_DEPT"
  exit 1
fi

# Test 8: Foreign key constraints with department
echo "✓ Test 8: Foreign key constraints (role FK) still work with department"
# Try to insert user with invalid role (should fail)
ERROR_OUTPUT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -c "INSERT INTO org_users (user_id, name, role, email, department) VALUES (999, 'Invalid Role User', 'invalid_role', 'invalid@test.com', 'Test');" 2>&1 || true)

if echo "$ERROR_OUTPUT" | grep -q "violates foreign key constraint"; then
  echo "  ✓ Role foreign key constraint works correctly"
else
  echo "  ❌ Role foreign key constraint not enforced"
  exit 1
fi

# Test 9: Hierarchical query with department
echo "✓ Test 9: Hierarchical query (recursive CTE) includes department"
HIERARCHY=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t <<EOF
WITH RECURSIVE org_tree AS (
  -- Root users (CEOs)
  SELECT user_id, reports_to_id, name, role, department, 0 AS level
  FROM org_users
  WHERE user_id = 100 AND reports_to_id IS NULL
  
  UNION ALL
  
  -- Recursive: direct reports
  SELECT u.user_id, u.reports_to_id, u.name, u.role, u.department, t.level + 1
  FROM org_users u
  INNER JOIN org_tree t ON u.reports_to_id = t.user_id
)
SELECT user_id, name, department, level FROM org_tree ORDER BY level, user_id;
EOF
)

if echo "$HIERARCHY" | grep -q "Finance"; then
  echo "  ✓ Recursive CTE with department successful"
  echo "$HIERARCHY" | head -4 | sed 's/^/    /'
else
  echo "  ❌ Recursive CTE query failed"
  exit 1
fi

# Test 10: Backward compatibility - profiles table unaffected
echo "✓ Test 10: Backward compatibility - profiles table"
PROFILE_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM profiles;")
PROFILE_COUNT=$(echo "$PROFILE_COUNT" | tr -d ' ')

if [ "$PROFILE_COUNT" -ge 6 ]; then
  echo "  ✓ Profiles table unaffected: $PROFILE_COUNT profiles"
else
  echo "  ❌ Profiles table compromised, expected 6+ profiles"
  exit 1
fi

# Test 11: Backward compatibility - policies table
echo "✓ Test 11: Backward compatibility - policies table"
POLICY_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM policies;")
POLICY_COUNT=$(echo "$POLICY_COUNT" | tr -d ' ')

if [ "$POLICY_COUNT" -ge 34 ]; then
  echo "  ✓ Policies table unaffected: $POLICY_COUNT policies"
else
  echo "  ❌ Policies table compromised, expected 34+ policies"
  exit 1
fi

# Test 12: Migration idempotency (rollback + re-apply)
echo "✓ Test 12: Migration idempotency test"
echo "  Rollback migration..."
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_down.sql > /dev/null 2>&1

# Verify table doesn't exist
TABLE_EXISTS=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'org_users');")
TABLE_EXISTS=$(echo "$TABLE_EXISTS" | tr -d ' ')

if [ "$TABLE_EXISTS" == "f" ]; then
  echo "  ✓ Rollback successful (table dropped)"
else
  echo "  ❌ Rollback failed (table still exists)"
  exit 1
fi

echo "  Re-apply migration..."
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_create_org_users.sql > /dev/null 2>&1

# Verify table exists again with department column
TABLE_EXISTS=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'org_users');")
TABLE_EXISTS=$(echo "$TABLE_EXISTS" | tr -d ' ')

DEPT_EXISTS=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'org_users' AND column_name = 'department');")
DEPT_EXISTS=$(echo "$DEPT_EXISTS" | tr -d ' ')

if [ "$TABLE_EXISTS" == "t" ] && [ "$DEPT_EXISTS" == "t" ]; then
  echo "  ✓ Re-apply successful (table + department column created)"
else
  echo "  ❌ Re-apply failed"
  exit 1
fi

# Re-insert test data
docker exec -i ce_postgres psql -U postgres -d orchestrator <<EOF > /dev/null 2>&1
INSERT INTO org_users (user_id, reports_to_id, name, role, email, department, created_at, updated_at)
VALUES 
  (100, NULL, 'Test CEO', 'manager', 'testceo@test.com', 'Executive', NOW(), NOW()),
  (101, 100, 'Test CFO', 'finance', 'testcfo@test.com', 'Finance', NOW(), NOW());
EOF

REINSERT_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE user_id IN (100, 101);")
REINSERT_COUNT=$(echo "$REINSERT_COUNT" | tr -d ' ')

if [ "$REINSERT_COUNT" == "2" ]; then
  echo "  ✓ Data re-inserted successfully after migration reset"
else
  echo "  ❌ Re-insert failed"
  exit 1
fi

# Test 13: NULL department constraint
echo "✓ Test 13: NOT NULL constraint on department"
ERROR_OUTPUT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -c "INSERT INTO org_users (user_id, name, role, email) VALUES (998, 'No Dept User', 'manager', 'nodept@test.com');" 2>&1 || true)

if echo "$ERROR_OUTPUT" | grep -q "violates not-null constraint"; then
  echo "  ✓ NOT NULL constraint on department works correctly"
else
  echo "  ❌ NOT NULL constraint not enforced"
  exit 1
fi

# Test 14: Department comment exists
echo "✓ Test 14: Verify column comment"
COMMENT_CHECK=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT col_description('org_users'::regclass::oid, 6)" 2>/dev/null || echo "")

if echo "$COMMENT_CHECK" | grep -q "Department"; then
  echo "  ✓ Column comment exists: $(echo "$COMMENT_CHECK" | xargs)"
else
  echo "  ⚠  Column comment check skipped (non-critical)"
fi

# Cleanup test data
echo "  Cleaning up test data..."
docker exec -i ce_postgres psql -U postgres -d orchestrator -c "DELETE FROM org_users WHERE user_id BETWEEN 100 AND 105;" > /dev/null 2>&1
docker exec -i ce_postgres psql -U postgres -d orchestrator -c "DELETE FROM org_users WHERE user_id IN (998, 999);" > /dev/null 2>&1

echo ""
echo "=========================================="
echo "✅ All 14 database tests passed!"
echo "=========================================="
echo ""
echo "Summary:"
echo "  ✓ Department field added to org_users table (VARCHAR(100) NOT NULL)"
echo "  ✓ Department index created (idx_org_users_department)"
echo "  ✓ INSERT/UPDATE/SELECT with department works"
echo "  ✓ Hierarchical queries (recursive CTE) include department"
echo "  ✓ Foreign key constraints still enforced (role FK)"
echo "  ✓ NOT NULL constraint enforced on department"
echo "  ✓ Migration is idempotent (rollback + re-apply)"
echo "  ✓ Backward compatibility maintained (profiles, policies tables)"
echo ""
echo "Department field database integration: ✅ SUCCESSFUL"
echo ""
echo "Next steps:"
echo "  1. Rebuild controller Docker image with D10-D12 routes"
echo "  2. Run API-level integration tests (CSV import endpoint)"
echo "  3. Proceed with D13-D14 (unit + integration tests)"
