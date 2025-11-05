#!/bin/bash
# Start Finance Agent for Multi-Agent Testing
#
# This script:
# 1. Gets a fresh JWT token from Keycloak
# 2. Updates the Agent Mesh MCP server environment with Finance role
# 3. Starts a Goose session configured for Finance agent
#
# Usage: ./scripts/start-finance-agent.sh
#
# The Finance agent will have access to Agent Mesh MCP tools:
# - send_task: Send tasks to other agents (e.g., Manager)
# - request_approval: Request approvals from other roles
# - notify: Send notifications to other agents
# - fetch_status: Check status of tasks/sessions

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
error() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    exit 1
}

success() {
    echo -e "${GREEN}$1${NC}"
}

info() {
    echo -e "${BLUE}$1${NC}"
}

warn() {
    echo -e "${YELLOW}$1${NC}"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

info "🏦 Starting Finance Agent for Multi-Agent Testing"
echo ""

# Check if Controller API is running
info "📡 Checking Controller API..."
if ! curl -sf http://localhost:8088/status >/dev/null 2>&1; then
    warn "⚠️  Controller API is not running"
    info "Starting Controller API..."
    cd "$PROJECT_ROOT/deploy/compose"
    docker compose -f ce.dev.yml up controller -d
    sleep 3
    if ! curl -sf http://localhost:8088/status >/dev/null 2>&1; then
        error "Failed to start Controller API"
    fi
fi
success "✅ Controller API is running"
echo ""

# Get JWT token
info "🔑 Acquiring JWT token from Keycloak..."
JWT_TOKEN=$("$SCRIPT_DIR/get-jwt-token.sh" 2>&1 | tail -1)
if [ -z "$JWT_TOKEN" ] || [ "$JWT_TOKEN" = "null" ]; then
    error "Failed to get JWT token. Is Keycloak running?"
fi
success "✅ JWT token acquired"
echo ""

# Set environment variables for Agent Mesh MCP server
export ROLE="finance"
export CONTROLLER_URL="http://localhost:8088"
export MESH_JWT_TOKEN="$JWT_TOKEN"

info "📋 Finance Agent Configuration:"
echo "   Role: $ROLE"
echo "   Controller URL: $CONTROLLER_URL"
echo "   JWT Token: ${JWT_TOKEN:0:30}...${JWT_TOKEN: -10}"
echo ""

# Check if Agent Mesh virtual environment exists
AGENT_MESH_DIR="$PROJECT_ROOT/src/agent-mesh"
if [ ! -d "$AGENT_MESH_DIR/.venv" ]; then
    warn "⚠️  Agent Mesh virtual environment not found"
    info "Creating virtual environment..."
    cd "$AGENT_MESH_DIR"
    python3 -m venv .venv
    source .venv/bin/activate
    pip install -q --upgrade pip
    pip install -q -e .
    success "✅ Virtual environment created"
    echo ""
fi

# Start Agent Mesh MCP server in background
info "🚀 Starting Agent Mesh MCP server (Finance role)..."
cd "$AGENT_MESH_DIR"
source .venv/bin/activate

# Start MCP server in background
python -m agent_mesh_server &
MCP_PID=$!

# Give MCP server time to start
sleep 2

# Check if MCP server is running
if ! kill -0 $MCP_PID 2>/dev/null; then
    error "Agent Mesh MCP server failed to start"
fi

success "✅ Agent Mesh MCP server running (PID: $MCP_PID)"
echo ""

info "═══════════════════════════════════════════════════════════"
info "🏦 Finance Agent Ready!"
info "═══════════════════════════════════════════════════════════"
echo ""
info "You can now use Goose Desktop or CLI to interact with this agent."
echo ""
info "Example workflows:"
echo ""
info "1️⃣  Send a budget approval request to Manager:"
echo "   'Use agent_mesh__send_task to send a budget approval request"
echo "    to the manager role for \$50,000 for Engineering department'"
echo ""
info "2️⃣  Send a notification:"
echo "   'Use agent_mesh__notify to send a high-priority notification"
echo "    to the manager about Q4 budget deadline'"
echo ""
info "3️⃣  Check task status:"
echo "   'Use agent_mesh__fetch_status to check the status of task"
echo "    <task-id-from-previous-response>'"
echo ""
info "═══════════════════════════════════════════════════════════"
echo ""
warn "⚠️  Press Ctrl+C to stop the MCP server (PID: $MCP_PID)"
echo ""

# Wait for MCP server process
wait $MCP_PID
