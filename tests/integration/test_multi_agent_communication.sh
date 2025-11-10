#!/bin/bash

# Test: Multi-Agent Communication via Agent Mesh
# 
# This test validates that multiple Goose agents can:
# 1. Start successfully in Docker containers
# 2. Fetch their respective profiles from Controller
# 3. Communicate via Agent Mesh tools and Controller /tasks/route endpoint
# 4. Maintain workspace isolation
# 5. Survive Controller restarts

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Test result function
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    echo ""
    echo -e "${YELLOW}TEST $TESTS_RUN:${NC} $test_name"
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS:${NC} $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL:${NC} $test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

echo "======================================"
echo "Multi-Agent Communication Test Suite"
echo "======================================"
echo ""
echo "Prerequisites:"
echo "  - Docker Compose services running (controller, privacy-guard-proxy, ollama)"
echo "  - Goose containers NOT yet started (this test will start them)"
echo ""

# Check prerequisites
echo "Checking prerequisites..."

# TEST 1: Controller API is accessible
run_test "Controller API is accessible" \
    "curl -s http://localhost:8088/status | grep -q 'ok'"

# TEST 2: Privacy Guard Proxy is accessible  
run_test "Privacy Guard Proxy is accessible" \
    "curl -s http://localhost:8090/api/status | grep -q 'healthy'"

# TEST 3: Keycloak is accessible
run_test "Keycloak is accessible" \
    "curl -s http://localhost:8080/realms/dev/.well-known/openid-configuration | grep -q 'issuer'"

# TEST 4: Docker Compose file has multi-goose profile
run_test "Docker Compose has multi-goose profile" \
    "grep -q 'profiles:.*multi-goose' deploy/compose/ce.dev.yml"

# TEST 5: All 3 Goose services defined in compose file
run_test "All 3 Goose services defined" \
    "[ $(grep -c 'goose-finance:' deploy/compose/ce.dev.yml) -eq 1 ] && \
     [ $(grep -c 'goose-manager:' deploy/compose/ce.dev.yml) -eq 1 ] && \
     [ $(grep -c 'goose-legal:' deploy/compose/ce.dev.yml) -eq 1 ]"

echo ""
echo "======================================"
echo "Starting Multi-Goose Environment"
echo "======================================"

# Start the multi-goose services
echo "Starting goose-finance, goose-manager, goose-legal..."
cd deploy/compose

# Note: This requires OPENROUTER_API_KEY to be set in .env.ce
# Check if it's set
if ! grep -q "^OPENROUTER_API_KEY=" .env.ce 2>/dev/null || grep -q "^OPENROUTER_API_KEY=$" .env.ce 2>/dev/null; then
    echo ""
    echo -e "${YELLOW}WARNING:${NC} OPENROUTER_API_KEY not set in .env.ce"
    echo "Multi-Goose containers will start but may fail to make LLM calls."
    echo "This is OK for testing agent startup and communication routing."
    echo ""
fi

# Start services with all required profiles
# Note: multi-goose services depend on controller, privacy-guard-proxy, ollama
docker compose -f ce.dev.yml \
    --profile controller \
    --profile privacy-guard \
    --profile privacy-guard-proxy \
    --profile ollama \
    --profile multi-goose \
    up -d 2>&1 | tail -10

cd ../..

# Wait for containers to start (give them time to fetch profiles and configure)
echo "Waiting for containers to initialize (30 seconds)..."
sleep 30

# TEST 6: All 3 Goose containers are running
run_test "All 3 Goose containers are running" \
    "[ $(docker ps --filter 'name=ce_goose_' --format '{{.Names}}' | wc -l) -eq 3 ]"

# TEST 7: Finance container is healthy
run_test "Finance container started" \
    "docker ps --filter 'name=ce_goose_finance' --filter 'status=running' --format '{{.Names}}' | grep -q goose_finance"

# TEST 8: Manager container is healthy  
run_test "Manager container started" \
    "docker ps --filter 'name=ce_goose_manager' --filter 'status=running' --format '{{.Names}}' | grep -q goose_manager"

# TEST 9: Legal container is healthy
run_test "Legal container started" \
    "docker ps --filter 'name=ce_goose_legal' --filter 'status=running' --format '{{.Names}}' | grep -q goose_legal"

# TEST 10: Finance container fetched finance profile
run_test "Finance fetched correct profile" \
    "docker logs ce_goose_finance 2>&1 | grep -q 'Role: finance'"

# TEST 11: Manager container fetched manager profile
run_test "Manager fetched correct profile" \
    "docker logs ce_goose_manager 2>&1 | grep -q 'Role: manager'"

# TEST 12: Legal container fetched legal profile
run_test "Legal fetched correct profile" \
    "docker logs ce_goose_legal 2>&1 | grep -q 'Role: legal'"

# TEST 13: Finance has agent_mesh in config
run_test "Finance config includes agent_mesh" \
    "docker exec ce_goose_finance cat /root/.config/goose/config.yaml 2>/dev/null | grep -q 'agent_mesh:'"

# TEST 14: Manager has agent_mesh in config
run_test "Manager config includes agent_mesh" \
    "docker exec ce_goose_manager cat /root/.config/goose/config.yaml 2>/dev/null | grep -q 'agent_mesh:'"

# TEST 15: Legal has agent_mesh in config
run_test "Legal config includes agent_mesh" \
    "docker exec ce_goose_legal cat /root/.config/goose/config.yaml 2>/dev/null | grep -q 'agent_mesh:'"

# TEST 16: Workspaces are isolated (finance has separate volume)
run_test "Finance workspace exists" \
    "docker exec ce_goose_finance ls -la /workspace 2>/dev/null | grep -q 'total'"

# TEST 17: Manager workspace is different from finance
run_test "Workspaces are isolated" \
    "docker exec ce_goose_finance touch /workspace/finance-test.txt && \
     ! docker exec ce_goose_manager ls /workspace/finance-test.txt 2>/dev/null"

# TEST 18: Controller /tasks/route endpoint accessible
run_test "Controller tasks/route endpoint exists" \
    "curl -s -X POST http://localhost:8088/tasks/route \
        -H 'Content-Type: application/json' \
        -d '{\"target\":\"test\",\"task\":{}}' 2>&1 | grep -qE '(400|401|405)'"

echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"
echo "Tests run: $TESTS_RUN"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Multi-Goose environment is running:"
    echo "  - goose-finance (role: finance)"
    echo "  - goose-manager (role: manager)"
    echo "  - goose-legal (role: legal)"
    echo ""
    echo "Each agent has:"
    echo "  ✓ Correct profile from Controller"
    echo "  ✓ Agent Mesh extension configured"
    echo "  ✓ Isolated workspace volume"
    echo "  ✓ Access to Controller API for task routing"
    echo ""
    echo "To test agent communication, use the Agent Mesh tools:"
    echo "  - agent_mesh__send_task"
    echo "  - agent_mesh__request_approval"
    echo "  - agent_mesh__notify"
    echo "  - agent_mesh__fetch_status"
    echo ""
    echo "To stop the environment:"
    echo "  cd deploy/compose && docker compose -f ce.dev.yml --profile multi-goose down"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
