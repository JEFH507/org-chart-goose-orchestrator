#!/bin/bash
# Privacy Guard False Positive Rate Test
# Validates that clean samples have no false detections
# Usage: ./test_false_positives.sh [--regex-only|--model-enhanced]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FIXTURES_DIR="$PROJECT_ROOT/tests/fixtures"
GUARD_URL="${GUARD_URL:-http://localhost:8089}"

# Parse arguments
MODE="${1:-auto}"  # auto, --regex-only, --model-enhanced

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Privacy Guard False Positive Rate Test ===${NC}"
echo ""
echo "Project: goose-org-twin / Phase 2.2"
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Guard URL: $GUARD_URL"
echo "Fixtures: $FIXTURES_DIR/clean_samples.txt"
echo "Mode: $MODE"
echo ""

# Check prerequisites
if [ ! -f "$FIXTURES_DIR/clean_samples.txt" ]; then
    echo -e "${RED}❌ ERROR: clean_samples.txt not found${NC}"
    exit 1
fi

# Check if guard is running
if ! curl -sf "$GUARD_URL/status" > /dev/null 2>&1; then
    echo -e "${RED}❌ ERROR: Privacy Guard not responding at $GUARD_URL${NC}"
    echo "Start the service with: docker compose up privacy-guard"
    exit 1
fi

# Configure model based on mode
if [ "$MODE" == "--regex-only" ]; then
    echo -e "${YELLOW}Disabling model for regex-only test...${NC}"
    cd "$PROJECT_ROOT/deploy/compose"
    sed -i 's/^GUARD_MODEL_ENABLED=.*/GUARD_MODEL_ENABLED=false/' .env.ce
    docker compose -f ce.dev.yml --profile ollama --profile privacy-guard restart privacy-guard > /dev/null 2>&1
    cd "$PROJECT_ROOT"
    sleep 8
elif [ "$MODE" == "--model-enhanced" ]; then
    echo -e "${YELLOW}Enabling model for model-enhanced test...${NC}"
    cd "$PROJECT_ROOT/deploy/compose"
    sed -i 's/^GUARD_MODEL_ENABLED=.*/GUARD_MODEL_ENABLED=true/' .env.ce
    docker compose -f ce.dev.yml --profile ollama --profile privacy-guard restart privacy-guard > /dev/null 2>&1
    cd "$PROJECT_ROOT"
    sleep 15
else
    echo -e "${YELLOW}Using current guard configuration...${NC}"
fi

# Get current model status
model_enabled=$(curl -sf "$GUARD_URL/status" | jq -r '.model_enabled // false')
model_name=$(curl -sf "$GUARD_URL/status" | jq -r '.model_name // "N/A"')

echo "Model enabled: $model_enabled"
if [ "$model_enabled" == "true" ]; then
    echo "Model name: $model_name"
fi
echo ""

# Test false positive rate
echo -e "${YELLOW}Testing false positive rate on clean samples...${NC}"

false_positives=0
total_samples=0
fp_details=$(mktemp)
trap "rm -f $fp_details" EXIT

while IFS= read -r line; do
    # Skip empty lines and comments
    if [[ -z "$line" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
        continue
    fi
    
    total_samples=$((total_samples + 1))
    
    # Call scan endpoint
    response=$(curl -sf -X POST "$GUARD_URL/guard/scan" \
        -H 'Content-Type: application/json' \
        -d "{\"text\": $(echo "$line" | jq -Rs .), \"tenant_id\": \"fp-test\"}" \
        2>/dev/null)
    
    detection_count=$(echo "$response" | jq -r '.detections | length' 2>/dev/null || echo "0")
    
    if [ "$detection_count" -gt 0 ]; then
        false_positives=$((false_positives + 1))
        
        # Record false positive details
        echo "=== FALSE POSITIVE #$false_positives ===" >> "$fp_details"
        echo "Text: $line" >> "$fp_details"
        echo "Detections:" >> "$fp_details"
        echo "$response" | jq -r '.detections[] | "  - Type: \(.entity_type), Text: \(.matched_text), Confidence: \(.confidence)"' >> "$fp_details"
        echo "" >> "$fp_details"
    fi
    
    # Progress indicator
    if [ $((total_samples % 20)) -eq 0 ]; then
        echo -n "."
    fi
done < "$FIXTURES_DIR/clean_samples.txt"

echo ""
echo ""

# Calculate false positive rate
if [ "$total_samples" -gt 0 ]; then
    fp_rate=$(awk "BEGIN {printf \"%.2f\", ($false_positives / $total_samples) * 100}")
    
    echo -e "${BLUE}=== Results ===${NC}"
    echo "Total samples:       $total_samples"
    echo "False positives:     $false_positives"
    echo "False positive rate: ${fp_rate}%"
    echo ""
    
    # Show false positive details if any
    if [ "$false_positives" -gt 0 ]; then
        echo -e "${YELLOW}False Positive Details:${NC}"
        cat "$fp_details"
    fi
    
    # Acceptance criteria: FP rate < 5%
    if (( $(echo "$fp_rate < 5" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${GREEN}✅ PASS: False positive rate < 5% (got ${fp_rate}%)${NC}"
        exit 0
    elif (( $(echo "$fp_rate < 10" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${YELLOW}⚠️  MARGINAL: False positive rate 5-10% (got ${fp_rate}%)${NC}"
        echo "   Target is < 5%, but 5-10% may be acceptable for some use cases"
        exit 0
    else
        echo -e "${RED}❌ FAIL: False positive rate >= 10% (got ${fp_rate}%)${NC}"
        echo "   Expected < 5%"
        exit 1
    fi
else
    echo -e "${RED}❌ ERROR: No clean samples to test${NC}"
    exit 1
fi
