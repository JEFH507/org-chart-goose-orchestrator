#!/bin/bash
# Integration test for department field (Option A migration)
# Tests CSV import, org tree API, and backward compatibility

set -e

CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:3000}"
TEST_CSV="tests/integration/test_data/org_chart_sample.csv"

echo "==========================================
Department Field Integration Test
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

# Test 2: Verify department index
echo "✓ Test 2: Verify department index exists"
INDEX=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT indexname FROM pg_indexes WHERE tablename = 'org_users' AND indexname = 'idx_org_users_department';")
if echo "$INDEX" | grep -q "idx_org_users_department"; then
  echo "  ✓ Department index exists"
else
  echo "  ❌ Department index NOT found"
  exit 1
fi

# Test 3: CSV import with department field
echo "✓ Test 3: CSV import with department field"
if [ ! -f "$TEST_CSV" ]; then
  echo "  ❌ Test CSV file not found: $TEST_CSV"
  exit 1
fi

# Import CSV via curl (multipart form-data)
IMPORT_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/org/import" \
  -F "file=@$TEST_CSV" \
  -H "Content-Type: multipart/form-data")

IMPORT_ID=$(echo "$IMPORT_RESPONSE" | jq -r '.import_id // empty')
USERS_CREATED=$(echo "$IMPORT_RESPONSE" | jq -r '.users_created // 0')
STATUS=$(echo "$IMPORT_RESPONSE" | jq -r '.status // "unknown"')

if [ -z "$IMPORT_ID" ] || [ "$IMPORT_ID" == "null" ]; then
  echo "  ❌ CSV import failed"
  echo "  Response: $IMPORT_RESPONSE"
  exit 1
fi

echo "  ✓ CSV imported successfully"
echo "    Import ID: $IMPORT_ID"
echo "    Users created: $USERS_CREATED"
echo "    Status: $STATUS"

# Test 4: Verify users in database with department
echo "✓ Test 4: Verify users have department field populated"
USER_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE department IS NOT NULL;")
USER_COUNT=$(echo "$USER_COUNT" | tr -d ' ')

if [ "$USER_COUNT" -ge 10 ]; then
  echo "  ✓ $USER_COUNT users have department field"
else
  echo "  ❌ Expected 10+ users with department, got: $USER_COUNT"
  exit 1
fi

# Test 5: Verify specific departments
echo "✓ Test 5: Verify specific department values"
FINANCE_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE department = 'Finance';")
FINANCE_COUNT=$(echo "$FINANCE_COUNT" | tr -d ' ')

ENGINEERING_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM org_users WHERE department = 'Engineering';")
ENGINEERING_COUNT=$(echo "$ENGINEERING_COUNT" | tr -d ' ')

echo "  ✓ Finance department: $FINANCE_COUNT users"
echo "  ✓ Engineering department: $ENGINEERING_COUNT users"

if [ "$FINANCE_COUNT" -lt 2 ]; then
  echo "  ❌ Expected at least 2 Finance users"
  exit 1
fi

# Test 6: Org tree API includes department
echo "✓ Test 6: GET /admin/org/tree includes department field"
TREE_RESPONSE=$(curl -s "$CONTROLLER_URL/admin/org/tree")
TOTAL_USERS=$(echo "$TREE_RESPONSE" | jq -r '.total_users // 0')

if [ "$TOTAL_USERS" -lt 10 ]; then
  echo "  ❌ Expected 10+ users in org tree, got: $TOTAL_USERS"
  exit 1
fi

# Check if first node has department field
FIRST_NODE_DEPT=$(echo "$TREE_RESPONSE" | jq -r '.tree[0].department // empty')
if [ -z "$FIRST_NODE_DEPT" ]; then
  echo "  ❌ Department field missing in org tree response"
  echo "  Response: $TREE_RESPONSE" | jq '.'
  exit 1
fi

echo "  ✓ Org tree includes department field: $FIRST_NODE_DEPT"
echo "  ✓ Total users: $TOTAL_USERS"

# Test 7: Nested nodes include department
echo "✓ Test 7: Nested org nodes include department"
NESTED_DEPT=$(echo "$TREE_RESPONSE" | jq -r '.tree[0].reports[0].department // empty')
if [ -z "$NESTED_DEPT" ]; then
  echo "  ❌ Department field missing in nested org node"
  exit 1
fi

echo "  ✓ Nested nodes include department: $NESTED_DEPT"

# Test 8: CSV update (upsert) preserves department
echo "✓ Test 8: CSV re-import updates department correctly"
# Create modified CSV in memory with updated department for user 5
UPDATED_CSV=$(cat "$TEST_CSV" | sed 's/Eve Finance Analyst,analyst,eve@company.com,Finance/Eve Finance Analyst,analyst,eve@company.com,Accounting/')

# Save to temp file
TEMP_CSV="/tmp/org_chart_updated.csv"
echo "$UPDATED_CSV" > "$TEMP_CSV"

# Re-import
UPDATE_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/org/import" \
  -F "file=@$TEMP_CSV" \
  -H "Content-Type: multipart/form-data")

USERS_UPDATED=$(echo "$UPDATE_RESPONSE" | jq -r '.users_updated // 0')

if [ "$USERS_UPDATED" -lt 1 ]; then
  echo "  ❌ Expected at least 1 user updated, got: $USERS_UPDATED"
  exit 1
fi

echo "  ✓ Users updated: $USERS_UPDATED"

# Verify department change
EVE_DEPT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT department FROM org_users WHERE user_id = 5;")
EVE_DEPT=$(echo "$EVE_DEPT" | tr -d ' ')

if [ "$EVE_DEPT" == "Accounting" ]; then
  echo "  ✓ Department updated correctly: $EVE_DEPT"
else
  echo "  ❌ Expected department 'Accounting', got: $EVE_DEPT"
  exit 1
fi

# Clean up
rm -f "$TEMP_CSV"

# Test 9: Import history includes all imports
echo "✓ Test 9: GET /admin/org/imports shows import history"
HISTORY_RESPONSE=$(curl -s "$CONTROLLER_URL/admin/org/imports")
TOTAL_IMPORTS=$(echo "$HISTORY_RESPONSE" | jq -r '.total // 0')

if [ "$TOTAL_IMPORTS" -lt 2 ]; then
  echo "  ❌ Expected at least 2 imports, got: $TOTAL_IMPORTS"
  exit 1
fi

echo "  ✓ Import history: $TOTAL_IMPORTS total imports"

# Test 10: Backward compatibility - profiles table unaffected
echo "✓ Test 10: Backward compatibility check"
PROFILE_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM profiles;")
PROFILE_COUNT=$(echo "$PROFILE_COUNT" | tr -d ' ')

if [ "$PROFILE_COUNT" -ge 6 ]; then
  echo "  ✓ Profiles table unaffected: $PROFILE_COUNT profiles"
else
  echo "  ❌ Profiles table compromised, expected 6+ profiles"
  exit 1
fi

# Test 11: Workstream C policies still work
echo "✓ Test 11: Workstream C policies unaffected"
POLICY_COUNT=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT COUNT(*) FROM policies;")
POLICY_COUNT=$(echo "$POLICY_COUNT" | tr -d ' ')

if [ "$POLICY_COUNT" -ge 34 ]; then
  echo "  ✓ Policies table unaffected: $POLICY_COUNT policies"
else
  echo "  ❌ Policies table compromised, expected 34+ policies"
  exit 1
fi

# Test 12: Rollback and re-apply migration (idempotency)
echo "✓ Test 12: Migration idempotency test"
echo "  Rollback migration..."
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_down.sql > /dev/null 2>&1

echo "  Re-apply migration..."
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_create_org_users.sql > /dev/null 2>&1

# Verify table exists again
TABLE_EXISTS=$(docker exec -i ce_postgres psql -U postgres -d orchestrator -t -c "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'org_users');")
TABLE_EXISTS=$(echo "$TABLE_EXISTS" | tr -d ' ')

if [ "$TABLE_EXISTS" == "t" ]; then
  echo "  ✓ Migration rollback and re-apply successful"
else
  echo "  ❌ Migration rollback/re-apply failed"
  exit 1
fi

# Re-import CSV after rollback
echo "  Re-importing CSV after migration reset..."
REIMPORT_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/org/import" \
  -F "file=@$TEST_CSV" \
  -H "Content-Type: multipart/form-data")

REIMPORT_CREATED=$(echo "$REIMPORT_RESPONSE" | jq -r '.users_created // 0')
if [ "$REIMPORT_CREATED" -ge 10 ]; then
  echo "  ✓ Data re-imported successfully: $REIMPORT_CREATED users"
else
  echo "  ❌ Re-import failed, expected 10+ users"
  exit 1
fi

echo ""
echo "=========================================="
echo "✅ All 12 tests passed!"
echo "=========================================="
echo ""
echo "Summary:"
echo "  - Department field added to org_users table"
echo "  - Department index created"
echo "  - CSV import handles department field"
echo "  - API responses include department"
echo "  - Upsert updates department correctly"
echo "  - Migration is idempotent (rollback + re-apply)"
echo "  - Backward compatibility maintained"
echo ""
echo "Department field integration: ✅ SUCCESSFUL"
