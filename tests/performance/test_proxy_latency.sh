#!/bin/bash
# Privacy Guard Proxy Performance Benchmark
# Measures latency overhead of proxy + Privacy Guard

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

PROXY_URL="http://localhost:8090"
PRIVACY_GUARD_URL="http://localhost:8089"

echo "=========================================="
echo "PRIVACY GUARD PROXY LATENCY BENCHMARK"
echo "=========================================="
echo ""

# Test 1: Proxy API latency (status endpoint - no masking)
echo -e "${YELLOW}Test 1: Proxy API latency (status endpoint)${NC}"
TOTAL=0
for i in {1..10}; do
    LATENCY=$(curl -s -w "%{time_total}\n" -o /dev/null $PROXY_URL/api/status)
    # Convert to milliseconds
    LATENCY_MS=$(echo "$LATENCY * 1000" | bc)
    TOTAL=$(echo "$TOTAL + $LATENCY_MS" | bc)
    echo "  Request $i: ${LATENCY_MS} ms"
done
AVG=$(echo "scale=2; $TOTAL / 10" | bc)
echo -e "  ${GREEN}Average: ${AVG} ms${NC}"
echo ""

# Test 2: Privacy Guard masking latency
echo -e "${YELLOW}Test 2: Privacy Guard masking latency${NC}"
TEXT_WITH_PII="Employee John Doe with SSN 123-45-6789 and email john@example.com"
TOTAL=0
for i in {1..10}; do
    LATENCY=$(curl -s -w "%{time_total}\n" -o /dev/null -X POST $PRIVACY_GUARD_URL/guard/mask \
        -H "Content-Type: application/json" \
        -d "{\"tenant_id\": \"benchmark\", \"text\": \"$TEXT_WITH_PII\"}")
    LATENCY_MS=$(echo "$LATENCY * 1000" | bc)
    TOTAL=$(echo "$TOTAL + $LATENCY_MS" | bc)
    echo "  Request $i: ${LATENCY_MS} ms"
done
AVG=$(echo "scale=2; $TOTAL / 10" | bc)
echo -e "  ${GREEN}Average: ${AVG} ms${NC}"
MASK_AVG=$AVG
echo ""

# Test 3: Privacy Guard reidentify latency
echo -e "${YELLOW}Test 3: Privacy Guard reidentify latency${NC}"
# First, get a masked text with session_id
MASK_RESULT=$(curl -s -X POST $PRIVACY_GUARD_URL/guard/mask \
    -H "Content-Type: application/json" \
    -d "{\"tenant_id\": \"benchmark\", \"text\": \"$TEXT_WITH_PII\"}")
SESSION_ID=$(echo "$MASK_RESULT" | jq -r '.session_id')
MASKED_TEXT=$(echo "$MASK_RESULT" | jq -r '.masked_text')

TOTAL=0
for i in {1..10}; do
    LATENCY=$(curl -s -w "%{time_total}\n" -o /dev/null -X POST $PRIVACY_GUARD_URL/guard/reidentify \
        -H "Content-Type: application/json" \
        -d "{\"tenant_id\": \"benchmark\", \"session_id\": \"$SESSION_ID\", \"masked_text\": \"$MASKED_TEXT\"}")
    LATENCY_MS=$(echo "$LATENCY * 1000" | bc)
    TOTAL=$(echo "$TOTAL + $LATENCY_MS" | bc)
    echo "  Request $i: ${LATENCY_MS} ms"
done
AVG=$(echo "scale=2; $TOTAL / 10" | bc)
echo -e "  ${GREEN}Average: ${AVG} ms${NC}"
UNMASK_AVG=$AVG
echo ""

# Test 4: Combined overhead (mask + unmask)
echo -e "${YELLOW}Test 4: Combined Privacy Guard overhead${NC}"
COMBINED=$(echo "$MASK_AVG + $UNMASK_AVG" | bc)
echo -e "  Masking: ${MASK_AVG} ms"
echo -e "  Unmasking: ${UNMASK_AVG} ms"
echo -e "  ${GREEN}Total: ${COMBINED} ms${NC}"
echo ""

# Evaluate against target
TARGET=200
echo "=========================================="
echo "PERFORMANCE EVALUATION"
echo "=========================================="
echo "Target: < ${TARGET} ms"
echo "Actual: ${COMBINED} ms"

if (( $(echo "$COMBINED < $TARGET" | bc -l) )); then
    echo -e "${GREEN}✓ PASS - Within target${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}✗ FAIL - Exceeds target by $(echo "$COMBINED - $TARGET" | bc) ms${NC}"
    EXIT_CODE=1
fi
echo ""

# Summary
echo "=========================================="
echo "SUMMARY"
echo "=========================================="
echo "Proxy API latency: Low (status endpoint)"
echo "Privacy Guard mask: ${MASK_AVG} ms"
echo "Privacy Guard unmask: ${UNMASK_AVG} ms"
echo "Combined overhead: ${COMBINED} ms"
echo "Target: < ${TARGET} ms"
echo ""

exit $EXIT_CODE
