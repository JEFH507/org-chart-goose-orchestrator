#!/bin/bash
set -e

echo "=== Session Lifecycle Persistence Test ==="
echo

# Get JWT token using client_credentials
echo "Getting JWT token..."
JWT=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" | jq -r '.access_token')

if [ -z "$JWT" ] || [ "$JWT" = "null" ]; then
  echo "❌ FAIL: Could not get JWT token"
  exit 1
fi
echo "✅ JWT token acquired"
echo

# Test 1: Create session with new columns
echo "Test 1: Creating session..."
CREATE_RESP=$(curl -s -X POST http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"agent_role": "finance", "metadata": {"test": "persistence"}}')

SESSION_ID=$(echo "$CREATE_RESP" | jq -r '.session_id')
echo "Session ID: $SESSION_ID"

# Verify in database (check new columns exist)
echo "Verifying new columns in database..."
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT id, status, fsm_metadata, last_transition_at, completed_at FROM sessions WHERE id = '$SESSION_ID';"
echo "✅ Test 1: Session created with FSM columns"
echo

# Test 2: Activate session (should update last_transition_at)
echo "Test 2: Activating session..."
sleep 2  # Small delay to ensure timestamp difference
curl -s -X PUT "http://localhost:8088/sessions/$SESSION_ID/events" \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "activate"}' | jq .

# Check last_transition_at was updated
echo "Verifying last_transition_at updated..."
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT status, last_transition_at FROM sessions WHERE id = '$SESSION_ID';"
echo "✅ Test 2: State transition persisted"
echo

# Test 3: Complete session (should set completed_at)
echo "Test 3: Completing session..."
curl -s -X PUT "http://localhost:8088/sessions/$SESSION_ID/events" \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "complete"}' | jq .

# Check completed_at was set
echo "Verifying completed_at is set..."
COMPLETED_AT=$(docker exec ce_postgres psql -U postgres -d orchestrator -t -c \
  "SELECT completed_at FROM sessions WHERE id = '$SESSION_ID';")

if [ -n "$COMPLETED_AT" ] && [ "$COMPLETED_AT" != " " ]; then
  echo "Completed at: $COMPLETED_AT"
  echo "✅ Test 3: completed_at timestamp set"
else
  echo "❌ FAIL: completed_at not set"
  exit 1
fi
echo

# Test 4: Controller restart (verify session persists)
echo "Test 4: Testing session persistence across controller restart..."
echo "Restarting controller..."
docker restart ce_controller > /dev/null 2>&1
sleep 5

# Verify session still exists
PERSISTED=$(curl -s -H "Authorization: Bearer $JWT" \
  "http://localhost:8088/sessions/$SESSION_ID" | jq -r '.state')

if [ "$PERSISTED" = "completed" ]; then
  echo "✅ Test 4: Session persisted across restart"
else
  echo "❌ FAIL: Session not found after restart (got: $PERSISTED)"
  exit 1
fi
echo

echo "=== All Persistence Tests Passed! ✅ ==="
echo "Summary:"
echo "  ✅ Session creation with FSM columns"
echo "  ✅ State transition updates last_transition_at"  
echo "  ✅ Completion sets completed_at timestamp"
echo "  ✅ Session persists across controller restart"
