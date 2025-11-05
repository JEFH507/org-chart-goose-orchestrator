#!/bin/bash
# Validation script for B5: fetch_status tool
# Runs all validation tests in Docker container

set -e

echo "=========================================="
echo "B5 Validation: fetch_status Tool"
echo "=========================================="
echo ""

# Build Docker image (suppress build output for cleaner output)
echo "Building Docker image..."
docker build -t agent-mesh:latest . > /dev/null 2>&1
echo "✓ Docker image built"
echo ""

# Run fetch_status validation tests
echo "Running fetch_status validation tests..."
echo "------------------------------------------"
docker run --rm agent-mesh:latest python tests/test_fetch_status.py
echo ""

# Run server tools integration tests
echo "Running server tools integration tests..."
echo "------------------------------------------"
docker run --rm agent-mesh:latest python tests/test_server_tools.py
echo ""

echo "=========================================="
echo "✅ B5 Validation Complete"
echo "=========================================="
