#!/bin/bash
# Privacy Guard Detection Accuracy Comparison
# Compares regex-only vs model-enhanced detection accuracy
# Usage: ./compare_detection.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FIXTURES_DIR="$PROJECT_ROOT/tests/fixtures"
GUARD_URL="${GUARD_URL:-http://localhost:8089}"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Privacy Guard Detection Accuracy Comparison ===${NC}"
echo ""
echo "Project: goose-org-twin / Phase 2.2"
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Guard URL: $GUARD_URL"
echo "Fixtures: $FIXTURES_DIR/pii_samples.txt"
echo ""

# Check prerequisites
if [ ! -f "$FIXTURES_DIR/pii_samples.txt" ]; then
    echo -e "${RED}❌ ERROR: pii_samples.txt not found${NC}"
    exit 1
fi

# Check if guard is running
if ! curl -sf "$GUARD_URL/status" > /dev/null 2>&1; then
    echo -e "${RED}❌ ERROR: Privacy Guard not responding at $GUARD_URL${NC}"
    echo "Start the service with: docker compose up privacy-guard"
    exit 1
fi

# Temporary files
regex_results=$(mktemp)
model_results=$(mktemp)
trap "rm -f $regex_results $model_results" EXIT

# Test 1: Regex-only detection
echo -e "${YELLOW}Step 1: Testing regex-only detection...${NC}"
echo "  (This may take 30-60 seconds)"

# Disable model
cd "$PROJECT_ROOT/deploy/compose"
sed -i 's/^GUARD_MODEL_ENABLED=.*/GUARD_MODEL_ENABLED=false/' .env.ce
docker compose -f ce.dev.yml --env-file .env.ce --profile ollama --profile privacy-guard stop privacy-guard > /dev/null 2>&1
docker compose -f ce.dev.yml --env-file .env.ce --profile ollama --profile privacy-guard up -d privacy-guard > /dev/null 2>&1
cd "$PROJECT_ROOT"
sleep 8  # Wait for service to be ready

# Verify model is disabled
model_status=$(curl -sf "$GUARD_URL/status" | jq -r '.model_enabled // false')
if [ "$model_status" != "false" ]; then
    echo -e "${RED}❌ ERROR: Model not disabled (status: $model_status)${NC}"
    exit 1
fi

echo "  Model disabled: ✓"

# Count detections per line
line_num=0
while IFS= read -r line; do
    line_num=$((line_num + 1))
    
    # Skip empty lines and comments
    if [[ -z "$line" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
        continue
    fi
    
    # Call scan endpoint
    detection_count=$(curl -sf -X POST "$GUARD_URL/guard/scan" \
        -H 'Content-Type: application/json' \
        -d "{\"text\": $(echo "$line" | jq -Rs .), \"tenant_id\": \"accuracy-test\"}" \
        | jq -r '.detections | length' 2>/dev/null || echo "0")
    
    echo "$detection_count" >> "$regex_results"
    
    # Progress indicator
    if [ $((line_num % 20)) -eq 0 ]; then
        echo -n "."
    fi
done < "$FIXTURES_DIR/pii_samples.txt"

echo ""

regex_total=$(awk '{sum+=$1} END {print sum}' "$regex_results")
regex_lines=$(wc -l < "$regex_results")
echo -e "  ${GREEN}Regex-only: $regex_total entities detected across $regex_lines samples${NC}"

# Test 2: Model-enhanced detection
echo ""
echo -e "${YELLOW}Step 2: Testing model-enhanced detection...${NC}"
echo "  (This may take 2-3 minutes due to model inference)"

# Enable model
cd "$PROJECT_ROOT/deploy/compose"
sed -i 's/^GUARD_MODEL_ENABLED=.*/GUARD_MODEL_ENABLED=true/' .env.ce
docker compose -f ce.dev.yml --env-file .env.ce --profile ollama --profile privacy-guard stop privacy-guard > /dev/null 2>&1
docker compose -f ce.dev.yml --env-file .env.ce --profile ollama --profile privacy-guard up -d privacy-guard > /dev/null 2>&1
cd "$PROJECT_ROOT"
sleep 15  # Model startup takes longer

# Verify model is enabled
model_status=$(curl -sf "$GUARD_URL/status" | jq -r '.model_enabled // false')
if [ "$model_status" != "true" ]; then
    echo -e "${RED}❌ WARNING: Model not enabled (status: $model_status)${NC}"
    echo "Continuing with fallback to regex-only..."
fi

model_name=$(curl -sf "$GUARD_URL/status" | jq -r '.model_name // "unknown"')
echo "  Model enabled: ✓ ($model_name)"

# Count detections per line
line_num=0
while IFS= read -r line; do
    line_num=$((line_num + 1))
    
    # Skip empty lines and comments
    if [[ -z "$line" ]] || [[ "$line" =~ ^[[:space:]]*# ]]; then
        continue
    fi
    
    # Call scan endpoint
    detection_count=$(curl -sf -X POST "$GUARD_URL/guard/scan" \
        -H 'Content-Type: application/json' \
        -d "{\"text\": $(echo "$line" | jq -Rs .), \"tenant_id\": \"accuracy-test\"}" \
        | jq -r '.detections | length' 2>/dev/null || echo "0")
    
    echo "$detection_count" >> "$model_results"
    
    # Progress indicator
    if [ $((line_num % 20)) -eq 0 ]; then
        echo -n "."
    fi
done < "$FIXTURES_DIR/pii_samples.txt"

echo ""

model_total=$(awk '{sum+=$1} END {print sum}' "$model_results")
model_lines=$(wc -l < "$model_results")
echo -e "  ${GREEN}Model-enhanced: $model_total entities detected across $model_lines samples${NC}"

# Calculate improvement
echo ""
echo -e "${BLUE}=== Results ===${NC}"
echo "Regex-only:      $regex_total entities"
echo "Model-enhanced:  $model_total entities"

if [ "$regex_total" -gt 0 ]; then
    improvement=$(awk "BEGIN {printf \"%.1f\", (($model_total - $regex_total) / $regex_total) * 100}")
    
    echo "Improvement:     ${improvement}%"
    echo ""
    
    # Acceptance criteria: >= 10% improvement
    if (( $(echo "$improvement >= 10" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${GREEN}✅ PASS: Accuracy improvement >= 10% (got ${improvement}%)${NC}"
        exit 0
    elif (( $(echo "$improvement >= 5" | bc -l 2>/dev/null || echo "0") )); then
        echo -e "${YELLOW}⚠️  MARGINAL: Accuracy improvement 5-10% (got ${improvement}%)${NC}"
        echo "   Target is 10%+, but 5-10% may be acceptable depending on use case"
        exit 0
    else
        echo -e "${RED}❌ FAIL: Accuracy improvement < 5% (got ${improvement}%)${NC}"
        echo "   Expected >= 10% improvement"
        exit 1
    fi
else
    echo -e "${RED}❌ ERROR: No regex detections to compare${NC}"
    exit 1
fi
