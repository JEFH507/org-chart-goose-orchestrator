#!/bin/bash
# Privacy Guard Validation Script
# Purpose: Test Privacy Guard PII masking before implementing full Phase 6 proxy
# Usage: ./scripts/privacy-goose-validate.sh

set -e

PRIVACY_GUARD_URL="${PRIVACY_GUARD_URL:-http://localhost:8089}"
TENANT_ID="${TENANT_ID:-validation-test}"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   Privacy Guard Validation Script${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Check if Privacy Guard is running
echo -n "Checking Privacy Guard service... "
if curl -s -f "$PRIVACY_GUARD_URL/status" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo ""
    echo "Please start Privacy Guard service:"
    echo "  cd /home/papadoc/Gooseprojects/goose-org-twin"
    echo "  docker-compose up privacy-guard"
    exit 1
fi

echo ""

# Test scenarios (aligned with Phase 5 tested PII types)
declare -a test_cases=(
    "SSN|My SSN is 123-45-6789|SSN"
    "Email|Contact john@acme.com for details|EMAIL"
    "Phone|Call me at 555-123-4567|PHONE"
    "Multiple PII|Email john@acme.com, SSN 123-45-6789, phone 555-123-4567|EMAIL,SSN,PHONE"
    "No PII|Analyze Q4 budget trends and forecast|NONE"
)

total_tests=0
passed_tests=0

for test_case in "${test_cases[@]}"; do
    IFS='|' read -r test_name user_input expected_entities <<< "$test_case"
    
    total_tests=$((total_tests + 1))
    
    echo -e "${BLUE}Test $total_tests: $test_name${NC}"
    echo "Input: \"$user_input\""
    
    # Scan for PII
    SCAN_RESULT=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
        -H "Content-Type: application/json" \
        -d "{\"text\":\"$user_input\",\"mode\":\"hybrid\",\"tenant_id\":\"$TENANT_ID\"}" 2>/dev/null)
    
    if [ $? -ne 0 ] || [ -z "$SCAN_RESULT" ]; then
        echo -e "${RED}✗ FAILED: Privacy Guard scan request failed${NC}"
        echo ""
        continue
    fi
    
    DETECTION_COUNT=$(echo "$SCAN_RESULT" | jq -r '.detections | length' 2>/dev/null || echo "0")
    
    if [ "$expected_entities" == "NONE" ]; then
        if [ "$DETECTION_COUNT" -eq 0 ]; then
            echo -e "${GREEN}✓ PASSED: No PII detected (as expected)${NC}"
            passed_tests=$((passed_tests + 1))
        else
            echo -e "${RED}✗ FAILED: Expected no PII, but detected $DETECTION_COUNT${NC}"
            echo "Detections: $(echo $SCAN_RESULT | jq -r '.detections')"
        fi
        echo ""
        continue
    fi
    
    if [ "$DETECTION_COUNT" -eq 0 ]; then
        echo -e "${RED}✗ FAILED: Expected PII ($expected_entities), but none detected${NC}"
        echo ""
        continue
    fi
    
    echo -e "${YELLOW}⚠ Detected $DETECTION_COUNT PII item(s)${NC}"
    
    # Show detected entities
    ENTITIES=$(echo "$SCAN_RESULT" | jq -r '.detections[] | "\(.entity_type): \(.text)"')
    echo "$ENTITIES" | while read -r line; do
        echo "  - $line"
    done
    
    # Mask PII
    MASK_RESULT=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/mask" \
        -H "Content-Type: application/json" \
        -d "{\"text\":\"$user_input\",\"method\":\"pseudonym\",\"mode\":\"hybrid\",\"tenant_id\":\"$TENANT_ID\"}" 2>/dev/null)
    
    if [ $? -ne 0 ] || [ -z "$MASK_RESULT" ]; then
        echo -e "${RED}✗ FAILED: Privacy Guard mask request failed${NC}"
        echo ""
        continue
    fi
    
    MASKED_TEXT=$(echo "$MASK_RESULT" | jq -r '.masked_text' 2>/dev/null)
    
    if [ -z "$MASKED_TEXT" ] || [ "$MASKED_TEXT" == "null" ]; then
        echo -e "${RED}✗ FAILED: Masked text is empty${NC}"
        echo ""
        continue
    fi
    
    echo -e "${GREEN}✓ Masked: \"$MASKED_TEXT\"${NC}"
    
    # Verify masked text doesn't contain original PII
    if echo "$MASKED_TEXT" | grep -E "123-45-6789|john@acme.com|555-123-4567|4532-1234-5678-9010" > /dev/null; then
        echo -e "${RED}✗ FAILED: Masked text still contains original PII!${NC}"
        echo ""
        continue
    fi
    
    # Verify masked text contains tokens OR format-preserving replacements
    # Tokens: SSN_, EMAIL_, PHONE_, CREDIT_CARD_, PERSON_
    # Format-preserving: 999-XX-XXXX (SSN), 5XX-XXX-XXXX (phone), EMAIL_hash
    if echo "$MASKED_TEXT" | grep -E "SSN_|EMAIL_|PHONE_|CREDIT_CARD_|PERSON_|999-|555-" > /dev/null; then
        echo -e "${GREEN}✓ PASSED: Original PII replaced with masked values${NC}"
        passed_tests=$((passed_tests + 1))
    else
        echo -e "${RED}✗ FAILED: No masked values found in text${NC}"
    fi
    
    echo ""
done

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   Validation Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "Total Tests: $total_tests"
echo "Passed: $passed_tests"
echo "Failed: $((total_tests - passed_tests))"
echo ""

if [ "$passed_tests" -eq "$total_tests" ]; then
    echo -e "${GREEN}✓✓✓ ALL TESTS PASSED! ✓✓✓${NC}"
    echo ""
    echo "Privacy Guard is working correctly!"
    echo "Ready to proceed with Proxy + Scripts approach for Phase 6."
    exit 0
else
    echo -e "${RED}✗✗✗ SOME TESTS FAILED ✗✗✗${NC}"
    echo ""
    echo "Privacy Guard needs fixes before Phase 6."
    echo "Review failures above and fix Privacy Guard service."
    exit 1
fi
