#!/bin/bash
# Agent Mesh MCP Server - Setup Script
# Supports both native Python and Docker-based setup

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Agent Mesh MCP Server Setup"
echo "================================"

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}' | cut -d. -f1,2)
echo "Detected Python version: $PYTHON_VERSION"

if [ "$1" == "docker" ] || [ ! -x "$(command -v python3)" ]; then
    echo "📦 Using Docker-based setup..."
    
    # Build Docker image
    echo "Building Docker image with Python 3.13..."
    docker build -t agent-mesh:latest .
    
    echo "✅ Docker image built successfully"
    echo ""
    echo "Usage:"
    echo "  docker run -it --rm agent-mesh:latest"
    echo "  # Or with .env file:"
    echo "  docker run -it --rm --env-file .env agent-mesh:latest"
    
elif [ -x "$(command -v python3)" ]; then
    echo "🐍 Using native Python setup..."
    
    # Check if venv exists
    if [ ! -d ".venv" ]; then
        echo "Creating virtual environment..."
        python3 -m venv .venv || {
            echo "❌ Failed to create venv. Install python3-venv:"
            echo "   sudo apt install python3-venv  # Debian/Ubuntu"
            echo "   sudo dnf install python3-virtualenv  # Fedora/RHEL"
            echo ""
            echo "Or use Docker setup: ./setup.sh docker"
            exit 1
        }
    fi
    
    # Activate venv
    source .venv/bin/activate
    
    # Upgrade pip
    echo "Upgrading pip..."
    pip install --upgrade pip setuptools wheel
    
    # Install package
    echo "Installing agent-mesh package..."
    pip install -e .
    
    # Optional: Install dev dependencies
    if [ "$1" == "dev" ]; then
        echo "Installing dev dependencies..."
        pip install -e ".[dev]"
    fi
    
    echo "✅ Setup complete"
    echo ""
    echo "Activate the virtual environment:"
    echo "  source .venv/bin/activate"
    echo ""
    echo "Run the server:"
    echo "  python agent_mesh_server.py"
    echo ""
    echo "Run tests:"
    echo "  pytest tests/"
    
else
    echo "❌ Python 3 not found"
    echo "Please install Python 3.13+ or use Docker setup:"
    echo "  ./setup.sh docker"
    exit 1
fi

echo ""
echo "📝 Don't forget to configure .env file:"
echo "  cp .env.example .env"
echo "  # Edit .env with your CONTROLLER_URL and MESH_JWT_TOKEN"
