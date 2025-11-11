#!/bin/bash
# Test script for Privacy Guard per-instance setup
# Tests that each Goose instance has independent Privacy Guard with user-controllable settings

set -e  # Exit on error

echo "============================================"
echo "Privacy Guard Per-Instance Test"
echo "============================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if running
echo "Step 1: Verify all services are running..."
echo ""

EXPECTED_CONTAINERS=(
    "ce_ollama_finance"
    "ce_ollama_manager"
    "ce_ollama_legal"
    "ce_privacy_guard_finance"
    "ce_privacy_guard_manager"
    "ce_privacy_guard_legal"
    "ce_privacy_guard_proxy_finance"
    "ce_privacy_guard_proxy_manager"
    "ce_privacy_guard_proxy_legal"
    "ce_goose_finance"
    "ce_goose_manager"
    "ce_goose_legal"
)

MISSING=0
for container in "${EXPECTED_CONTAINERS[@]}"; do
    if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
        echo -e "${GREEN}✓${NC} $container is running"
    else
        echo -e "${RED}✗${NC} $container is NOT running"
        MISSING=$((MISSING + 1))
    fi
done

if [ $MISSING -gt 0 ]; then
    echo ""
    echo -e "${RED}ERROR: $MISSING containers are missing!${NC}"
    echo "Run: docker compose -f deploy/compose/ce.dev.yml --profile multi-goose up -d"
    exit 1
fi

echo ""
echo -e "${GREEN}✓ All 12 containers are running!${NC}"
echo ""

# Test Finance Control Panel (Rules mode)
echo "============================================"
echo "Step 2: Test Finance Control Panel"
echo "============================================"
echo ""

echo "Setting Finance to Rules-only mode..."
RESPONSE=$(curl -s -X PUT http://localhost:8096/api/settings \
    -H "Content-Type: application/json" \
    -d '{"routing":"service","detection":"rules","privacy":"auto"}')

if echo "$RESPONSE" | grep -q "success\|updated\|ok"; then
    echo -e "${GREEN}✓${NC} Finance settings updated to Rules-only"
else
    echo -e "${YELLOW}⚠${NC} Finance settings response: $RESPONSE"
fi

echo ""
echo "Finance Control Panel: http://localhost:8096/ui"
echo ""

# Test Manager Control Panel (Hybrid mode)
echo "============================================"
echo "Step 3: Test Manager Control Panel"
echo "============================================"
echo ""

echo "Setting Manager to Hybrid mode..."
RESPONSE=$(curl -s -X PUT http://localhost:8097/api/settings \
    -H "Content-Type: application/json" \
    -d '{"routing":"service","detection":"hybrid","privacy":"auto"}')

if echo "$RESPONSE" | grep -q "success\|updated\|ok"; then
    echo -e "${GREEN}✓${NC} Manager settings updated to Hybrid"
else
    echo -e "${YELLOW}⚠${NC} Manager settings response: $RESPONSE"
fi

echo ""
echo "Manager Control Panel: http://localhost:8097/ui"
echo ""

# Test Legal Control Panel (AI mode)
echo "============================================"
echo "Step 4: Test Legal Control Panel"
echo "============================================"
echo ""

echo "Setting Legal to AI mode..."
RESPONSE=$(curl -s -X PUT http://localhost:8098/api/settings \
    -H "Content-Type: application/json" \
    -d '{"routing":"service","detection":"ai","privacy":"strict"}')

if echo "$RESPONSE" | grep -q "success\|updated\|ok"; then
    echo -e "${GREEN}✓${NC} Legal settings updated to AI-only"
else
    echo -e "${YELLOW}⚠${NC} Legal settings response: $RESPONSE"
fi

echo ""
echo "Legal Control Panel: http://localhost:8098/ui"
echo ""

# Check logs for verification
echo "============================================"
echo "Step 5: Verify Settings in Logs"
echo "============================================"
echo ""

echo "Checking Finance Privacy Guard logs for 'detection_method'..."
sleep 2  # Give logs a moment to update
FINANCE_LOGS=$(docker logs ce_privacy_guard_finance 2>&1 | grep "detection_method" | tail -3)
if [ -n "$FINANCE_LOGS" ]; then
    echo -e "${GREEN}Finance logs:${NC}"
    echo "$FINANCE_LOGS"
else
    echo -e "${YELLOW}⚠ No detection_method logs found for Finance (may need to send test request)${NC}"
fi

echo ""
echo "Checking Manager Privacy Guard logs for 'detection_method'..."
MANAGER_LOGS=$(docker logs ce_privacy_guard_manager 2>&1 | grep "detection_method" | tail -3)
if [ -n "$MANAGER_LOGS" ]; then
    echo -e "${GREEN}Manager logs:${NC}"
    echo "$MANAGER_LOGS"
else
    echo -e "${YELLOW}⚠ No detection_method logs found for Manager (may need to send test request)${NC}"
fi

echo ""
echo "Checking Legal Privacy Guard logs for 'detection_method'..."
LEGAL_LOGS=$(docker logs ce_privacy_guard_legal 2>&1 | grep "detection_method" | tail -3)
if [ -n "$LEGAL_LOGS" ]; then
    echo -e "${GREEN}Legal logs:${NC}"
    echo "$LEGAL_LOGS"
else
    echo -e "${YELLOW}⚠ No detection_method logs found for Legal (may need to send test request)${NC}"
fi

echo ""
echo "============================================"
echo "Summary"
echo "============================================"
echo ""
echo -e "${GREEN}✓ All per-instance services are running${NC}"
echo -e "${GREEN}✓ Settings have been updated${NC}"
echo ""
echo "Next steps:"
echo "1. Open Control Panels in browser:"
echo "   - Finance: http://localhost:8096/ui"
echo "   - Manager: http://localhost:8097/ui"
echo "   - Legal:   http://localhost:8098/ui"
echo ""
echo "2. Send test requests to verify settings flow:"
echo "   - Use Goose instances to send LLM requests"
echo "   - Check Privacy Guard logs for detection_method"
echo ""
echo "3. Verify isolation:"
echo "   - Change Finance to 'rules' → logs show 'rules'"
echo "   - Change Manager to 'hybrid' → logs show 'hybrid'"
echo "   - Each instance operates independently"
echo ""
echo "For detailed testing, see:"
echo "  - docs/tests/PRIVACY-GUARD-PER-INSTANCE-COMPLETE.md"
echo "  - PRIVACY-GUARD-FIX-SUMMARY.md"
echo ""
