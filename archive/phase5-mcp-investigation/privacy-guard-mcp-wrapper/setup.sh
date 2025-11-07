#!/bin/bash
# Setup script for Privacy Guard MCP Wrapper

set -e

echo "🔧 Privacy Guard MCP Wrapper Setup"
echo "===================================="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    echo "❌ Please run as normal user (not sudo)"
    echo "   The script will ask for sudo password when needed"
    exit 1
fi

# Step 1: Install system dependencies
echo "📦 Step 1: Installing system dependencies..."
echo "   - python3.12-venv"
echo "   - python3-pip"
echo ""
sudo apt update
sudo apt install -y python3.12-venv python3-pip

echo ""
echo "✅ System dependencies installed"
echo ""

# Step 2: Create virtual environment
echo "🐍 Step 2: Creating Python virtual environment..."
cd "$(dirname "$0")"
python3 -m venv .venv

echo "✅ Virtual environment created"
echo ""

# Step 3: Install Python package
echo "📦 Step 3: Installing privacy-guard-mcp package..."
source .venv/bin/activate
pip install -e .

echo ""
echo "✅ Package installed"
echo ""

# Step 4: Verify installation
echo "🧪 Step 4: Verifying installation..."
python -c "import privacy_guard_mcp; print(f'Version: {privacy_guard_mcp.__version__}')"

echo ""
echo "✅ Installation verified"
echo ""

# Step 5: Instructions
echo "🎉 Setup complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Next Steps:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Open Goose Desktop → Settings → Extensions"
echo ""
echo "2. Click '+ Add custom extension' button"
echo ""
echo "3. Fill in the form:"
echo "   Extension Name: privacy-guard"
echo "   Type: STDIO"
echo "   Description: Privacy Guard - PII detection and masking"
echo "   Command: $(pwd)/.venv/bin/python"
echo "   Command Args:"
echo "     - -m"
echo "     - privacy_guard_mcp"
echo "   Timeout: 300"
echo "   Environment Variables:"
echo "     PRIVACY_GUARD_URL = http://localhost:8089"
echo "     TENANT_ID = test-tenant"
echo ""
echo "4. Click 'Add Extension'"
echo ""
echo "5. Enable the extension (toggle switch)"
echo ""
echo "6. Start a new chat and try:"
echo "   'Scan this for PII: My SSN is 123-45-6789'"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 Full path for Command field:"
echo "   $(pwd)/.venv/bin/python"
echo ""
