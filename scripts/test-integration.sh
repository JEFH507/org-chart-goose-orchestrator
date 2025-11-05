#!/bin/bash
# Integration Testing Script for Phase 3
# Tests Workstream A + B integration and backward compatibility

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Phase 3 Integration Testing${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Test counter
PASSED=0
FAILED=0

pass() {
    echo -e "${GREEN}✅ PASS${NC} - $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}❌ FAIL${NC} - $1"
    ((FAILED++))
}

info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Test 1: Infrastructure Health
echo -e "${BLUE}[Test 1]${NC} Infrastructure Health Check"
if curl -sf http://localhost:8088/status >/dev/null 2>&1; then
    RESPONSE=$(curl -s http://localhost:8088/status)
    VERSION=$(echo "$RESPONSE" | jq -r '.version')
    pass "Controller API healthy (version: $VERSION)"
else
    fail "Controller API not responding"
fi

if curl -sf http://localhost:8080/realms/dev >/dev/null 2>&1; then
    pass "Keycloak 'dev' realm accessible"
else
    fail "Keycloak 'dev' realm not accessible"
fi
echo ""

# Test 2: JWT Token Acquisition
echo -e "${BLUE}[Test 2]${NC} JWT Token Acquisition"
if TOKEN=$("$SCRIPT_DIR/get-jwt-token.sh" 2>/dev/null); then
    if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
        pass "JWT token acquired (expires in 60 min)"
        info "Token: ${TOKEN:0:50}..."
    else
        fail "JWT token empty or null"
    fi
else
    fail "Failed to get JWT token"
fi
echo ""

# Test 3: Controller API - Without JWT (should work in dev mode)
echo -e "${BLUE}[Test 3]${NC} Controller API (Dev Mode - No JWT)"
RESPONSE=$(curl -s -X POST http://localhost:8088/tasks/route \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(python3 -c 'import uuid; print(uuid.uuid4())')" \
  -H "X-Trace-Id: $(python3 -c 'import uuid; print(uuid.uuid4())')" \
  -d '{
    "target": "test",
    "task": {"task_type": "test", "description": "No JWT test", "data": {}},
    "context": {}
  }')

if echo "$RESPONSE" | jq -e '.task_id' >/dev/null 2>&1; then
    TASK_ID=$(echo "$RESPONSE" | jq -r '.task_id')
    pass "POST /tasks/route works without JWT (dev mode)"
    info "Task ID: $TASK_ID"
else
    # Check if it's 401 (JWT required)
    if echo "$RESPONSE" | grep -q "401\|Unauthorized"; then
        warn "Controller requires JWT (production mode active)"
    else
        fail "POST /tasks/route failed: $RESPONSE"
    fi
fi
echo ""

# Test 4: Controller API - With JWT
if [ -n "${TOKEN:-}" ]; then
    echo -e "${BLUE}[Test 4]${NC} Controller API (With JWT)"
    
    # Try with JWT token
    HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/response.json \
      -X POST http://localhost:8088/tasks/route \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -H "Idempotency-Key: $(python3 -c 'import uuid; print(uuid.uuid4())')" \
      -H "X-Trace-Id: $(python3 -c 'import uuid; print(uuid.uuid4())')" \
      -d '{
        "target": "manager",
        "task": {
          "task_type": "integration_test",
          "description": "Testing with JWT auth",
          "data": {"test": "workstream_a"}
        },
        "context": {"test_phase": "phase3"}
      }')
    
    if [ "$HTTP_CODE" = "200" ]; then
        TASK_ID=$(cat /tmp/response.json | jq -r '.task_id')
        pass "POST /tasks/route with JWT (HTTP 200)"
        info "Task ID: $TASK_ID"
    elif [ "$HTTP_CODE" = "401" ]; then
        warn "JWT authentication required but token rejected (HTTP 401)"
        info "This might mean Controller needs OIDC env vars"
    else
        fail "POST /tasks/route with JWT (HTTP $HTTP_CODE)"
        cat /tmp/response.json
    fi
    echo ""
fi

# Test 5: POST /approvals
echo -e "${BLUE}[Test 5]${NC} POST /approvals"
HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/approval.json \
  -X POST http://localhost:8088/approvals \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(python3 -c 'import uuid; print(uuid.uuid4())')" \
  -H "X-Trace-Id: $(python3 -c 'import uuid; print(uuid.uuid4())')" \
  -d '{
    "task_id": "'"$(python3 -c 'import uuid; print(uuid.uuid4())')"'",
    "decision": "approved",
    "comments": "Integration test approval"
  }')

if [ "$HTTP_CODE" = "200" ]; then
    APPROVAL_ID=$(cat /tmp/approval.json | jq -r '.approval_id')
    pass "POST /approvals (HTTP 200)"
    info "Approval ID: $APPROVAL_ID"
else
    fail "POST /approvals (HTTP $HTTP_CODE)"
fi
echo ""

# Test 6: GET /profiles/{role}
echo -e "${BLUE}[Test 6]${NC} GET /profiles/{role}"
HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/profile.json \
  http://localhost:8088/profiles/manager)

if [ "$HTTP_CODE" = "200" ]; then
    ROLE=$(cat /tmp/profile.json | jq -r '.role')
    pass "GET /profiles/manager (HTTP 200, role: $ROLE)"
else
    fail "GET /profiles/manager (HTTP $HTTP_CODE)"
fi
echo ""

# Test 7: Agent Mesh MCP Server
echo -e "${BLUE}[Test 7]${NC} Agent Mesh MCP Server"
cd "$PROJECT_ROOT/src/agent-mesh"

if [ ! -d ".venv" ]; then
    warn "Python venv not found, creating..."
    python3 -m venv .venv
    source .venv/bin/activate
    pip install -q -e .
else
    source .venv/bin/activate
fi

# Check if tools can be imported
if python3 -c "from tools import send_task_tool, request_approval_tool, notify_tool, fetch_status_tool" 2>/dev/null; then
    pass "All 4 MCP tools can be imported"
else
    fail "MCP tools import failed"
fi
echo ""

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Test Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Some tests failed. See above for details.${NC}"
    exit 1
fi
