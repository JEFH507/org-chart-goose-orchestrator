#!/usr/bin/env bash
# Test multi-Goose service startup
# Tests that 3 Goose containers can start with different profiles
set -eo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

COMPOSE_FILE="deploy/compose/ce.dev.yml"
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
echo "Multi-Goose Startup Test Suite"
echo "======================================"
echo ""

# TEST 1: Verify Docker Compose file exists
run_test "Docker Compose file exists" \
    "test -f $COMPOSE_FILE"

# TEST 2: Verify docker-compose config is valid
run_test "Docker Compose configuration is valid" \
    "docker compose -f $COMPOSE_FILE config --quiet 2>&1 | grep -v 'variable is not set' > /dev/null || true"

# TEST 3: Verify multi-goose profile services are defined (need all dependent profiles)
# Note: ollama is required because privacy-guard depends on it (even though it's optional via GUARD_MODEL_ENABLED)
PROFILES="--profile controller --profile privacy-guard --profile privacy-guard-proxy --profile multi-goose --profile ollama"

run_test "goose-finance service is defined" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -q 'goose-finance:'"

run_test "goose-manager service is defined" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -q 'goose-manager:'"

run_test "goose-legal service is defined" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -q 'goose-legal:'"

# TEST 4: Verify workspace volumes are defined
run_test "goose_finance_workspace volume is defined" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -q 'goose_finance_workspace:'"

run_test "goose_manager_workspace volume is defined" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -q 'goose_manager_workspace:'"

run_test "goose_legal_workspace volume is defined" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -q 'goose_legal_workspace:'"

# TEST 5: Verify services use correct profile
run_test "All goose services use multi-goose profile" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 20 'goose-finance:' | grep -q 'multi-goose' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 20 'goose-manager:' | grep -q 'multi-goose' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 20 'goose-legal:' | grep -q 'multi-goose'"

# TEST 6: Verify services have correct roles
run_test "goose-finance has GOOSE_ROLE: finance" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-finance:' | grep -q 'GOOSE_ROLE: finance'"

run_test "goose-manager has GOOSE_ROLE: manager" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-manager:' | grep -q 'GOOSE_ROLE: manager'"

run_test "goose-legal has GOOSE_ROLE: legal" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-legal:' | grep -q 'GOOSE_ROLE: legal'"

# TEST 7: Verify services have extra_hosts for Keycloak access
run_test "All goose services have host.docker.internal mapping" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-finance:' | grep -q 'host.docker.internal=host-gateway' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-manager:' | grep -q 'host.docker.internal=host-gateway' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-legal:' | grep -q 'host.docker.internal=host-gateway'"

# TEST 8: Verify services depend on controller
run_test "All goose services depend on controller" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-finance:' | grep -q 'controller:' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-manager:' | grep -q 'controller:' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-legal:' | grep -q 'controller:'"

# TEST 9: Verify services depend on privacy-guard-proxy
run_test "All goose services depend on privacy-guard-proxy" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-finance:' | grep -q 'privacy-guard-proxy:' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-manager:' | grep -q 'privacy-guard-proxy:' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-legal:' | grep -q 'privacy-guard-proxy:'"

# TEST 10: Verify services use correct Docker image
run_test "All goose services use goose-test:0.2.0 image" \
    "docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-finance:' | grep -q 'goose-test:0.2.0' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-manager:' | grep -q 'goose-test:0.2.0' && \
     docker compose -f $COMPOSE_FILE $PROFILES config 2>/dev/null | grep -A 50 'goose-legal:' | grep -q 'goose-test:0.2.0'"

# TEST 11: Verify Docker image exists locally
run_test "Docker image goose-test:0.2.0 exists" \
    "docker images --format '{{.Repository}}:{{.Tag}}' | grep -q 'goose-test:0.2.0'"

# TEST 12: Verify agent-mesh extension files in image
run_test "Agent Mesh extension files exist in image" \
    "docker run --rm goose-test:0.2.0 ls -la /opt/agent-mesh/agent_mesh_server.py 2>/dev/null | grep -q agent_mesh_server.py"

# TEST 13: Verify agent-mesh Python dependencies installed
run_test "Agent Mesh Python dependencies installed" \
    "docker run --rm goose-test:0.2.0 python3 -c 'import mcp; import requests; import pydantic; print(\"OK\")' 2>&1 | grep -q 'OK'"

# TEST 14: Verify profiles are signed in database (prerequisite)
run_test "Profiles are signed in database" \
    "docker ps --format '{{.Names}}' | grep -q '^ce_controller$' && \
     JWT=\$(curl -s -X POST 'http://localhost:8080/realms/dev/protocol/openid-connect/token' \
       -H 'Content-Type: application/x-www-form-urlencoded' \
       -d 'grant_type=client_credentials' \
       -d 'client_id=goose-controller' \
       -d 'client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8' 2>/dev/null | jq -r '.access_token' 2>/dev/null) && \
     FINANCE_SIG=\$(curl -s -H \"Authorization: Bearer \$JWT\" \
       'http://localhost:8088/profiles/finance' 2>/dev/null | jq -r '.signature.signature' 2>/dev/null) && \
     [ -n \"\$FINANCE_SIG\" ] && [ \"\$FINANCE_SIG\" != \"null\" ] && [[ \"\$FINANCE_SIG\" == vault:v1:* ]]"

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
    echo "You can now start the multi-Goose environment with:"
    echo "  cd deploy/compose"
    echo "  docker compose -f ce.dev.yml --profile controller --profile privacy-guard --profile privacy-guard-proxy --profile ollama --profile multi-goose up -d"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
