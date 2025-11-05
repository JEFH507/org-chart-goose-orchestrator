#!/bin/bash
# Phase 4 Workstream D: Idempotency Testing Script
#
# Tests duplicate request handling with idempotency keys
#
# Prerequisites:
# - Controller running with IDEMPOTENCY_ENABLED=true
# - Redis running and connected
# - JWT token available (or JWT disabled)

set -e

CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
JWT_TOKEN="${JWT_TOKEN:-}"

echo "=== Phase 4 Idempotency Testing ==="
echo "Controller URL: $CONTROLLER_URL"
echo ""

# Helper function to make authenticated requests
make_request() {
    local method="$1"
    local path="$2"
    local data="$3"
    local idempotency_key="$4"
    
    local headers=(-H "Content-Type: application/json")
    
    if [ -n "$JWT_TOKEN" ]; then
        headers+=(-H "Authorization: Bearer $JWT_TOKEN")
    fi
    
    if [ -n "$idempotency_key" ]; then
        headers+=(-H "Idempotency-Key: $idempotency_key")
    fi
    
    if [ "$method" = "POST" ]; then
        curl -s -w "\n%{http_code}" "${headers[@]}" -X POST "$CONTROLLER_URL$path" -d "$data"
    else
        curl -s -w "\n%{http_code}" "${headers[@]}" -X GET "$CONTROLLER_URL$path"
    fi
}

# Test 1: Duplicate POST /sessions with same Idempotency-Key
echo "### Test 1: Duplicate POST /sessions (same Idempotency-Key)"
echo "Expected: First request creates session (201), second returns cached response (200)"
echo ""

IDEMPOTENCY_KEY_1="test-duplicate-session-$(date +%s)"
SESSION_PAYLOAD='{"agent_role":"finance","metadata":{"test":"idempotency"}}'

echo "Request 1: Creating session with Idempotency-Key: $IDEMPOTENCY_KEY_1"
RESPONSE_1=$(make_request POST "/sessions" "$SESSION_PAYLOAD" "$IDEMPOTENCY_KEY_1")
STATUS_1=$(echo "$RESPONSE_1" | tail -1)
BODY_1=$(echo "$RESPONSE_1" | head -n -1)
echo "Response: $BODY_1"
echo "Status: $STATUS_1"
echo ""

sleep 1

echo "Request 2: Duplicate request with same Idempotency-Key: $IDEMPOTENCY_KEY_1"
RESPONSE_2=$(make_request POST "/sessions" "$SESSION_PAYLOAD" "$IDEMPOTENCY_KEY_1")
STATUS_2=$(echo "$RESPONSE_2" | tail -1)
BODY_2=$(echo "$RESPONSE_2" | head -n -1)
echo "Response: $BODY_2"
echo "Status: $STATUS_2"
echo ""

if [ "$BODY_1" = "$BODY_2" ]; then
    echo "✅ PASS: Responses match (cached response returned)"
else
    echo "❌ FAIL: Responses differ (cache miss or disabled)"
fi
echo ""

# Test 2: Different Idempotency-Keys produce different responses
echo "### Test 2: Different Idempotency-Keys (different responses)"
echo "Expected: Each request creates a new session with different session_id"
echo ""

IDEMPOTENCY_KEY_2="test-unique-session-1-$(date +%s)"
IDEMPOTENCY_KEY_3="test-unique-session-2-$(date +%s)"

echo "Request 1: Idempotency-Key: $IDEMPOTENCY_KEY_2"
RESPONSE_3=$(make_request POST "/sessions" "$SESSION_PAYLOAD" "$IDEMPOTENCY_KEY_2")
BODY_3=$(echo "$RESPONSE_3" | head -n -1)
SESSION_ID_1=$(echo "$BODY_3" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
echo "Response: $BODY_3"
echo "Session ID: $SESSION_ID_1"
echo ""

sleep 1

echo "Request 2: Idempotency-Key: $IDEMPOTENCY_KEY_3"
RESPONSE_4=$(make_request POST "/sessions" "$SESSION_PAYLOAD" "$IDEMPOTENCY_KEY_3")
BODY_4=$(echo "$RESPONSE_4" | head -n -1)
SESSION_ID_2=$(echo "$BODY_4" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
echo "Response: $BODY_4"
echo "Session ID: $SESSION_ID_2"
echo ""

if [ "$SESSION_ID_1" != "$SESSION_ID_2" ]; then
    echo "✅ PASS: Different sessions created (unique idempotency keys)"
else
    echo "❌ FAIL: Same session returned (should be different)"
fi
echo ""

# Test 3: Missing Idempotency-Key header (pass through, no caching)
echo "### Test 3: Missing Idempotency-Key header"
echo "Expected: Each request creates a new session (no caching)"
echo ""

echo "Request 1: No Idempotency-Key"
RESPONSE_5=$(make_request POST "/sessions" "$SESSION_PAYLOAD" "")
BODY_5=$(echo "$RESPONSE_5" | head -n -1)
SESSION_ID_3=$(echo "$BODY_5" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
echo "Response: $BODY_5"
echo "Session ID: $SESSION_ID_3"
echo ""

sleep 1

echo "Request 2: No Idempotency-Key"
RESPONSE_6=$(make_request POST "/sessions" "$SESSION_PAYLOAD" "")
BODY_6=$(echo "$RESPONSE_6" | head -n -1)
SESSION_ID_4=$(echo "$BODY_6" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
echo "Response: $BODY_6"
echo "Session ID: $SESSION_ID_4"
echo ""

if [ "$SESSION_ID_3" != "$SESSION_ID_4" ]; then
    echo "✅ PASS: Different sessions created (no caching without key)"
else
    echo "❌ FAIL: Same session returned (should be different)"
fi
echo ""

# Test 4: Check Redis cache directly
echo "### Test 4: Verify Redis cache content"
echo "Expected: Cached response exists for Test 1 Idempotency-Key"
echo ""

if command -v docker &> /dev/null; then
    CACHE_KEY="idempotency:$IDEMPOTENCY_KEY_1"
    echo "Checking Redis key: $CACHE_KEY"
    CACHED_VALUE=$(docker exec ce_redis redis-cli GET "$CACHE_KEY" 2>/dev/null || echo "")
    
    if [ -n "$CACHED_VALUE" ]; then
        echo "✅ PASS: Cache entry exists"
        echo "Cached value (truncated): ${CACHED_VALUE:0:200}..."
        
        # Check TTL
        TTL=$(docker exec ce_redis redis-cli TTL "$CACHE_KEY" 2>/dev/null || echo "-1")
        echo "TTL: $TTL seconds (~$(($TTL / 3600)) hours remaining)"
    else
        echo "⚠️ WARNING: Cache entry not found (idempotency may be disabled)"
    fi
else
    echo "⚠️ SKIP: Docker not available (cannot check Redis directly)"
fi
echo ""

echo "=== Idempotency Testing Complete ==="
echo ""
echo "Summary:"
echo "- Test 1: Duplicate request handling"
echo "- Test 2: Unique idempotency keys"
echo "- Test 3: Missing idempotency key (no caching)"
echo "- Test 4: Redis cache verification"
echo ""
echo "To enable idempotency, set IDEMPOTENCY_ENABLED=true in .env.ce"
echo "and rebuild the controller container."
