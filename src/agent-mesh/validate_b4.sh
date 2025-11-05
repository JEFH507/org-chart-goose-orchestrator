#!/bin/bash
# Validation script for B4 (notify tool)

set -e

echo "============================================================"
echo "B4 VALIDATION: notify Tool Structure Tests"
echo "============================================================"
echo

# Build Docker image with updated code
echo "Building Docker image with notify tool..."
docker build -t agent-mesh:latest . > /dev/null 2>&1
echo "✓ Docker image built successfully"
echo

# Run validation tests in Docker
echo "Running validation tests..."
docker run --rm agent-mesh:latest python tests/test_notify.py

echo
echo "============================================================"
echo "B4 VALIDATION COMPLETE"
echo "============================================================"
