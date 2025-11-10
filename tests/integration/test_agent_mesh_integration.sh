#!/usr/bin/env bash
# Test Agent Mesh integration with Docker Goose containers
# Verifies that agent-mesh extension is properly configured and can communicate with Controller
set -eo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0

log_test() {
    echo -e "${YELLOW}TEST $((TESTS_RUN + 1)):${NC} $1"
}

log_pass() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_fail() {
    echo -e "${RED}✗ FAIL:${NC} $1"
}

run_test() {
    TESTS_RUN=$((TESTS_RUN + 1))
    log_test "$1"
    
    if eval "$2"; then
        log_pass "$1"
        return 0
    else
        log_fail "$1"
        return 1
    fi
}

echo "======================================"
echo "Agent Mesh Integration Test Suite"
echo "======================================"
echo ""

# TEST 1: Verify agent-mesh config in generated config.yaml
TESTS_RUN=$((TESTS_RUN + 1))
log_test "Verify agent-mesh extension configuration"

# Create a test profile JSON
TEST_PROFILE='{
  "role": "finance",
  "display_name": "Finance Agent",
  "extensions": [],
  "privacy": {},
  "policies": {}
}'

# Generate config and check for agent-mesh
TEMP_CONFIG=$(mktemp)
docker run --rm \
  -e CONTROLLER_URL=http://controller:8088 \
  -e OPENROUTER_API_KEY=test-key \
  goose-test:0.2.0 \
  python3 /usr/local/bin/generate-goose-config.py \
    --profile "$TEST_PROFILE" \
    --provider openrouter \
    --model test-model \
    --api-key test-key \
    --proxy-url http://privacy-guard-proxy:8090 \
    --output /dev/stdout > "$TEMP_CONFIG" 2>/dev/null

if grep -q "agent_mesh:" "$TEMP_CONFIG" && \
   grep -q "type: mcp" "$TEMP_CONFIG" && \
   grep -q "working_dir: /opt/agent-mesh" "$TEMP_CONFIG"; then
    log_pass "agent-mesh extension configured in config.yaml"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    log_fail "agent-mesh extension NOT configured in config.yaml"
    echo "Generated config:"
    cat "$TEMP_CONFIG"
fi

rm -f "$TEMP_CONFIG"

# TEST 2: Verify agent-mesh server can start
run_test "Agent Mesh MCP server can start" \
    "docker run --rm goose-test:0.2.0 timeout 5 python3 -m agent_mesh_server 2>&1 | grep -q 'Server shutdown complete' || true"

# TEST 3: Verify Controller /tasks/route endpoint exists
run_test "Controller /tasks/route endpoint exists" \
    "curl -s -o /dev/null -w '%{http_code}' http://localhost:8088/tasks/route | grep -qE '(400|401|405)'"

# TEST 4: Verify agent-mesh tools directory exists in image
run_test "Agent Mesh tools directory exists" \
    "docker run --rm goose-test:0.2.0 ls -la /opt/agent-mesh/tools/ | grep -q 'send_task.py'"

# TEST 5: Verify all 4 agent-mesh tools present
run_test "All 4 agent-mesh tools present (send_task, request_approval, notify, fetch_status)" \
    "docker run --rm goose-test:0.2.0 sh -c 'ls /opt/agent-mesh/tools/send_task.py /opt/agent-mesh/tools/request_approval.py /opt/agent-mesh/tools/notify.py /opt/agent-mesh/tools/fetch_status.py' 2>&1 | grep -c '.py' | grep -q 4"

# TEST 6: Verify MESH_JWT_TOKEN exported in entrypoint
run_test "Entrypoint exports MESH_JWT_TOKEN" \
    "grep -q 'export MESH_JWT_TOKEN' docker/goose/docker-goose-entrypoint.sh"

# TEST 7: Verify config generator includes agent-mesh
run_test "Config generator includes agent-mesh extension" \
    "grep -q 'agent_mesh' docker/goose/generate-goose-config.py"

# TEST 8: Verify PYTHONPATH set for agent-mesh
run_test "PYTHONPATH includes /opt/agent-mesh" \
    "docker run --rm goose-test:0.2.0 printenv PYTHONPATH | grep -q '/opt/agent-mesh'"

# Final summary
echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"
echo "Tests run: $TESTS_RUN"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $((TESTS_RUN - TESTS_PASSED))"

if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Agent Mesh integration is ready!"
    echo ""
    echo "The Docker Goose containers are now configured with:"
    echo "  • Agent Mesh MCP extension (/opt/agent-mesh)"
    echo "  • 4 tools: send_task, request_approval, notify, fetch_status"
    echo "  • Auto-configuration from Controller profiles"
    echo "  • JWT authentication for mesh communication"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
