#!/bin/bash
# Integration Test Runner for Agent Mesh MCP Server
# 
# This script:
# 1. Checks if Controller API is running
# 2. Obtains JWT token from Keycloak (if not provided)
# 3. Runs pytest integration tests
# 4. Reports results

set -e  # Exit on error

# ============================================================
# Configuration
# ============================================================

CONTROLLER_URL=${CONTROLLER_URL:-http://localhost:8088}
KEYCLOAK_URL=${KEYCLOAK_URL:-http://localhost:8080}
KEYCLOAK_REALM=${KEYCLOAK_REALM:-dev}
KEYCLOAK_CLIENT=${KEYCLOAK_CLIENT:-goose-controller}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================
# Functions
# ============================================================

print_header() {
    echo -e "${BLUE}============================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}============================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

check_controller() {
    print_info "Checking Controller API health at $CONTROLLER_URL..."
    
    if curl -s -f -m 5 "$CONTROLLER_URL/status" > /dev/null 2>&1; then
        print_success "Controller API is running"
        return 0
    else
        print_error "Controller API is not reachable at $CONTROLLER_URL"
        print_info "Please start the Controller API:"
        echo "  cd src/controller"
        echo "  cargo run --release"
        return 1
    fi
}

get_jwt_token() {
    if [ -n "$MESH_JWT_TOKEN" ]; then
        print_success "Using JWT token from MESH_JWT_TOKEN environment variable"
        return 0
    fi
    
    print_info "Obtaining JWT token from Keycloak..."
    
    # Check if KEYCLOAK_CLIENT_SECRET is set
    if [ -z "$KEYCLOAK_CLIENT_SECRET" ]; then
        print_warning "KEYCLOAK_CLIENT_SECRET not set"
        print_info "Please set MESH_JWT_TOKEN or KEYCLOAK_CLIENT_SECRET environment variable"
        print_info "Example:"
        echo "  export MESH_JWT_TOKEN=\$(curl -s -X POST $KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token \\"
        echo "    -d 'client_id=$KEYCLOAK_CLIENT' \\"
        echo "    -d 'grant_type=client_credentials' \\"
        echo "    -d 'client_secret=<secret>' | jq -r '.access_token')"
        return 1
    fi
    
    # Obtain token from Keycloak
    TOKEN_RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" \
        -d "client_id=$KEYCLOAK_CLIENT" \
        -d "grant_type=client_credentials" \
        -d "client_secret=$KEYCLOAK_CLIENT_SECRET" 2>&1)
    
    if [ $? -ne 0 ]; then
        print_error "Failed to obtain JWT token from Keycloak"
        print_info "Response: $TOKEN_RESPONSE"
        return 1
    fi
    
    MESH_JWT_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token' 2>/dev/null)
    
    if [ -z "$MESH_JWT_TOKEN" ] || [ "$MESH_JWT_TOKEN" = "null" ]; then
        print_error "Failed to extract access_token from Keycloak response"
        print_info "Response: $TOKEN_RESPONSE"
        return 1
    fi
    
    export MESH_JWT_TOKEN
    print_success "JWT token obtained from Keycloak"
}

check_python_env() {
    print_info "Checking Python environment..."
    
    if [ ! -d ".venv" ]; then
        print_warning "Virtual environment not found"
        print_info "Creating virtual environment..."
        python3 -m venv .venv
    fi
    
    print_info "Activating virtual environment..."
    source .venv/bin/activate
    
    print_info "Installing dependencies..."
    pip install -q -e ".[dev]"
    
    print_success "Python environment ready"
}

run_tests() {
    print_header "Running Integration Tests"
    
    # Set environment variables
    export CONTROLLER_URL
    export MESH_JWT_TOKEN
    
    # Run pytest with verbose output
    if pytest tests/test_integration.py -v --tb=short; then
        print_success "All integration tests passed!"
        return 0
    else
        print_error "Some integration tests failed"
        return 1
    fi
}

# ============================================================
# Main Execution
# ============================================================

main() {
    print_header "Agent Mesh Integration Test Runner"
    
    # Change to script directory
    cd "$(dirname "$0")"
    
    # Step 1: Check Controller API
    if ! check_controller; then
        exit 1
    fi
    
    # Step 2: Get JWT token
    if ! get_jwt_token; then
        exit 1
    fi
    
    # Step 3: Check Python environment
    if ! check_python_env; then
        exit 1
    fi
    
    # Step 4: Run tests
    if ! run_tests; then
        exit 1
    fi
    
    print_header "Integration Tests Complete"
    print_success "All systems operational!"
}

# Run main function
main "$@"
