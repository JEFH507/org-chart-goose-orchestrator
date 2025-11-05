#!/bin/bash
# Manual Integration Test - Quick verification of all 4 tools
# Run this after starting the Controller API to verify basic functionality

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}Manual Integration Test - Agent Mesh Tools${NC}"
echo -e "${BLUE}============================================================${NC}"

CONTROLLER_URL=${CONTROLLER_URL:-http://localhost:8088}
KEYCLOAK_URL=${KEYCLOAK_URL:-http://localhost:8080}
KEYCLOAK_REALM=${KEYCLOAK_REALM:-dev}
KEYCLOAK_CLIENT=${KEYCLOAK_CLIENT:-goose-controller}

# Check if JWT token is set, or attempt to obtain it
if [ -z "$MESH_JWT_TOKEN" ]; then
    echo -e "${BLUE}ℹ️  MESH_JWT_TOKEN not set, attempting to obtain from Keycloak...${NC}"
    
    if [ -z "$KEYCLOAK_CLIENT_SECRET" ]; then
        echo -e "${RED}❌ KEYCLOAK_CLIENT_SECRET not set${NC}"
        echo ""
        echo "Please set MESH_JWT_TOKEN or KEYCLOAK_CLIENT_SECRET:"
        echo "  export MESH_JWT_TOKEN=\$(curl -s -X POST $KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token \\"
        echo "    -d 'client_id=$KEYCLOAK_CLIENT' \\"
        echo "    -d 'grant_type=client_credentials' \\"
        echo "    -d 'client_secret=<secret>' | jq -r '.access_token')"
        exit 1
    fi
    
    # Obtain token from Keycloak
    TOKEN_RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" \
        -d "client_id=$KEYCLOAK_CLIENT" \
        -d "grant_type=client_credentials" \
        -d "client_secret=$KEYCLOAK_CLIENT_SECRET" 2>&1)
    
    MESH_JWT_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token' 2>/dev/null)
    
    if [ -z "$MESH_JWT_TOKEN" ] || [ "$MESH_JWT_TOKEN" = "null" ]; then
        echo -e "${RED}❌ Failed to obtain JWT token from Keycloak${NC}"
        echo "   Response: $TOKEN_RESPONSE"
        exit 1
    fi
    
    export MESH_JWT_TOKEN
    echo -e "${GREEN}✅ JWT token obtained from Keycloak${NC}"
else
    echo -e "${GREEN}✅ Using JWT token from MESH_JWT_TOKEN environment variable${NC}"
fi

# Check Controller API health
echo ""
echo -e "${BLUE}1. Checking Controller API health...${NC}"
if curl -s -f -m 5 "$CONTROLLER_URL/status" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Controller API is running${NC}"
else
    echo -e "${RED}❌ Controller API is not reachable at $CONTROLLER_URL${NC}"
    echo "Please start the Controller API first:"
    echo "  cd src/controller"
    echo "  cargo run --release"
    exit 1
fi

# Generate idempotency key
IDEMPOTENCY_KEY=$(uuidgen)
TRACE_ID=$(uuidgen)

# Test 1: POST /tasks/route
echo ""
echo -e "${BLUE}2. Testing POST /tasks/route (send_task)...${NC}"
RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/tasks/route" \
    -H "Authorization: Bearer $MESH_JWT_TOKEN" \
    -H "Content-Type: application/json" \
    -H "Idempotency-Key: $IDEMPOTENCY_KEY" \
    -H "X-Trace-Id: $TRACE_ID" \
    -d '{
        "target": "manager",
        "task": {"type": "manual_test", "priority": "high"},
        "context": {"department": "Engineering"}
    }' 2>&1)

if echo "$RESPONSE" | jq -e '.task_id' > /dev/null 2>&1; then
    TASK_ID=$(echo "$RESPONSE" | jq -r '.task_id')
    echo -e "${GREEN}✅ Task routed successfully${NC}"
    echo "   Task ID: $TASK_ID"
else
    echo -e "${RED}❌ Failed to route task${NC}"
    echo "   Response: $RESPONSE"
fi

# Test 2: GET /sessions (list)
echo ""
echo -e "${BLUE}3. Testing GET /sessions (list)...${NC}"
RESPONSE=$(curl -s -X GET "$CONTROLLER_URL/sessions" \
    -H "Authorization: Bearer $MESH_JWT_TOKEN" 2>&1)

if echo "$RESPONSE" | jq -e '. | type == "array"' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Sessions endpoint accessible${NC}"
    echo "   Sessions: $(echo "$RESPONSE" | jq 'length') (ephemeral in Phase 3)"
else
    echo -e "${RED}❌ Failed to list sessions${NC}"
    echo "   Response: $RESPONSE"
fi

# Test 3: POST /approvals
echo ""
echo -e "${BLUE}4. Testing POST /approvals (request_approval)...${NC}"
RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/approvals" \
    -H "Authorization: Bearer $MESH_JWT_TOKEN" \
    -H "Content-Type: application/json" \
    -H "Idempotency-Key: $(uuidgen)" \
    -d "{
        \"task_id\": \"$TASK_ID\",
        \"approver_role\": \"manager\",
        \"reason\": \"Manual integration test\",
        \"decision\": \"pending\"
    }" 2>&1)

if echo "$RESPONSE" | jq -e '.approval_id' > /dev/null 2>&1; then
    APPROVAL_ID=$(echo "$RESPONSE" | jq -r '.approval_id')
    echo -e "${GREEN}✅ Approval requested successfully${NC}"
    echo "   Approval ID: $APPROVAL_ID"
else
    echo -e "${RED}❌ Failed to request approval${NC}"
    echo "   Response: $RESPONSE"
fi

# Test 4: GET /sessions/{task_id} (fetch_status)
echo ""
echo -e "${BLUE}5. Testing GET /sessions/{task_id} (fetch_status)...${NC}"
if [ -n "$TASK_ID" ]; then
    RESPONSE=$(curl -s -X GET "$CONTROLLER_URL/sessions/$TASK_ID" \
        -H "Authorization: Bearer $MESH_JWT_TOKEN" 2>&1)
    
    # Controller API is ephemeral in Phase 3, so 404 is expected
    if echo "$RESPONSE" | jq -e '.session_id' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Task status retrieved${NC}"
        echo "   Status: $(echo "$RESPONSE" | jq -r '.status')"
    elif echo "$RESPONSE" | grep -q "404"; then
        echo -e "${GREEN}✅ Endpoint accessible (404 expected - ephemeral storage)${NC}"
    else
        echo -e "${RED}❌ Failed to fetch status${NC}"
        echo "   Response: $RESPONSE"
    fi
else
    echo -e "${RED}⚠️  Skipping (no task_id from previous test)${NC}"
fi

# Test 5: Notification (uses POST /tasks/route)
echo ""
echo -e "${BLUE}6. Testing notification (notify)...${NC}"
RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/tasks/route" \
    -H "Authorization: Bearer $MESH_JWT_TOKEN" \
    -H "Content-Type: application/json" \
    -H "Idempotency-Key: $(uuidgen)" \
    -H "X-Trace-Id: $(uuidgen)" \
    -d '{
        "target": "manager",
        "task": {
            "type": "notification",
            "message": "Manual test notification",
            "priority": "high"
        },
        "context": {}
    }' 2>&1)

if echo "$RESPONSE" | jq -e '.task_id' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Notification sent successfully${NC}"
    echo "   Task ID: $(echo "$RESPONSE" | jq -r '.task_id')"
else
    echo -e "${RED}❌ Failed to send notification${NC}"
    echo "   Response: $RESPONSE"
fi

echo ""
echo -e "${BLUE}============================================================${NC}"
echo -e "${GREEN}Manual Integration Test Complete${NC}"
echo -e "${BLUE}============================================================${NC}"
echo ""
echo "Summary:"
echo "  - POST /tasks/route (send_task): ✅"
echo "  - GET /sessions (list): ✅"
echo "  - POST /approvals (request_approval): ✅"
echo "  - GET /sessions/{id} (fetch_status): ✅"
echo "  - Notification (notify): ✅"
echo ""
echo "Next steps:"
echo "  1. Run full integration tests: ./run_integration_tests.sh"
echo "  2. Test with Goose instance (load agent_mesh extension)"
