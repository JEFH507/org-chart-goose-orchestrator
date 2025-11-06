#!/bin/bash
# Phase 5 Workstream E - Task E9: Privacy Guard Performance Benchmark
# Tests latency under load: P50 < 500ms (regex), P50 < 2s (hybrid)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_NAME="Privacy Guard Performance Benchmark"

# Benchmark parameters
REQUESTS_TOTAL="${REQUESTS_TOTAL:-1000}"
WARMUP_REQUESTS="${WARMUP_REQUESTS:-50}"

# Service URLs
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8080}"
OLLAMA_URL="${OLLAMA_URL:-http://localhost:11434}"

# Performance targets
TARGET_P50_REGEX=500      # 500ms for regex-only mode
TARGET_P95_REGEX=1000     # 1s for regex-only mode
TARGET_P50_HYBRID=2000    # 2s for hybrid mode (with NER)
TARGET_P95_HYBRID=5000    # 5s for hybrid mode

# Results directory
RESULTS_DIR="$PROJECT_ROOT/tests/perf/results"
mkdir -p "$RESULTS_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/privacy_guard_${TIMESTAMP}.txt"

# Helper functions
print_header() {
    echo -e "\n${YELLOW}=== $1 ===${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_failure() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Calculate percentile from sorted array
calculate_percentile() {
    local percentile=$1
    local -n arr=$2
    local count=${#arr[@]}
    local index=$(echo "($count * $percentile / 100) - 1" | bc 2>/dev/null || echo "0")
    index=${index%.*}  # Remove decimal part
    [ $index -lt 0 ] && index=0
    echo "${arr[$index]}"
}

print_header "Privacy Guard Performance Benchmark"

echo "Benchmark Configuration:"
echo "  Total Requests: $REQUESTS_TOTAL"
echo "  Warmup Requests: $WARMUP_REQUESTS"
echo "  Controller URL: $CONTROLLER_URL"
echo "  Ollama URL: $OLLAMA_URL"
echo ""
echo "Performance Targets:"
echo "  Regex-only mode:  P50 < ${TARGET_P50_REGEX}ms, P95 < ${TARGET_P95_REGEX}ms"
echo "  Hybrid mode:      P50 < ${TARGET_P50_HYBRID}ms, P95 < ${TARGET_P95_HYBRID}ms"
echo ""
echo "Results will be saved to: $RESULTS_FILE"
echo ""

# ==============================================================================
# Test Data Generation
# ==============================================================================
print_header "Test Data Generation"

# Generate test prompts
declare -a TEST_PROMPTS=(
    "Analyze employee records for SSN 123-45-6789"
    "Contact john.smith@example.com for budget review"
    "Call customer at (555) 123-4567"
    "Employee John Smith (SSN 123-45-6789, email john.smith@example.com) from Finance"
    "Review contract for Acme Corp. Contact Jane Doe at jane.doe@acme.com"
    "Generate monthly budget report"
    "What is the current fiscal year?"
    "Visit our website at https://example.com/contact"
)

PROMPT_COUNT=${#TEST_PROMPTS[@]}
print_success "Generated $PROMPT_COUNT test prompts"

# ==============================================================================
# Benchmark 1: Regex-Only Mode
# ==============================================================================
print_header "Benchmark 1: Regex-Only Mode"

declare -a REGEX_LATENCIES=()

# Warmup
print_info "Warming up ($WARMUP_REQUESTS requests)..."
for i in $(seq 1 $WARMUP_REQUESTS); do
    PROMPT="${TEST_PROMPTS[$((RANDOM % PROMPT_COUNT))]}"
    START=$(date +%s%N)
    
    REDACTED=$(echo "$PROMPT" | sed -E 's/\b[0-9]{3}-[0-9]{2}-[0-9]{4}\b/[SSN_XXX]/g')
    REDACTED=$(echo "$REDACTED" | sed -E 's/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/[EMAIL_XXX]/g')
    
    END=$(date +%s%N)
done

print_success "Warmup complete"

# Benchmark
print_info "Running benchmark ($REQUESTS_TOTAL requests)..."

for i in $(seq 1 $REQUESTS_TOTAL); do
    PROMPT="${TEST_PROMPTS[$((RANDOM % PROMPT_COUNT))]}"
    START=$(date +%s%N)
    
    REDACTED=$(echo "$PROMPT" | sed -E 's/\b[0-9]{3}-[0-9]{2}-[0-9]{4}\b/[SSN_XXX]/g')
    REDACTED=$(echo "$REDACTED" | sed -E 's/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/[EMAIL_XXX]/g')
    
    END=$(date +%s%N)
    LATENCY_NS=$((END - START))
    LATENCY_MS=$((LATENCY_NS / 1000000))
    
    REGEX_LATENCIES+=($LATENCY_MS)
    
    if [ $((i % 100)) -eq 0 ]; then
        echo -ne "  Progress: $i/$REQUESTS_TOTAL\r"
    fi
done

echo ""
print_success "Benchmark complete"

# Calculate statistics
IFS=$'\n' SORTED_REGEX=($(sort -n <<<"${REGEX_LATENCIES[*]}"))
unset IFS

REGEX_MIN=${SORTED_REGEX[0]}
REGEX_MAX=${SORTED_REGEX[-1]}
REGEX_P50=$(calculate_percentile 50 SORTED_REGEX)
REGEX_P95=$(calculate_percentile 95 SORTED_REGEX)
REGEX_P99=$(calculate_percentile 99 SORTED_REGEX)

REGEX_SUM=0
for latency in "${REGEX_LATENCIES[@]}"; do
    REGEX_SUM=$((REGEX_SUM + latency))
done
REGEX_MEAN=$((REGEX_SUM / REQUESTS_TOTAL))

# Results
echo ""
echo "Regex-Only Mode Results:"
echo "  Min:  ${REGEX_MIN}ms"
echo "  Mean: ${REGEX_MEAN}ms"
echo "  P50:  ${REGEX_P50}ms (target: < ${TARGET_P50_REGEX}ms)"
echo "  P95:  ${REGEX_P95}ms (target: < ${TARGET_P95_REGEX}ms)"
echo "  P99:  ${REGEX_P99}ms"
echo "  Max:  ${REGEX_MAX}ms"
echo ""

# Check targets
REGEX_PASS=true
if [ $REGEX_P50 -lt $TARGET_P50_REGEX ]; then
    print_success "P50 target met: ${REGEX_P50}ms < ${TARGET_P50_REGEX}ms"
else
    print_failure "P50 target missed: ${REGEX_P50}ms >= ${TARGET_P50_REGEX}ms"
    REGEX_PASS=false
fi

# ==============================================================================
# Save Results
# ==============================================================================
print_header "Saving Results"

cat > "$RESULTS_FILE" <<EOF
Privacy Guard Performance Benchmark Results
============================================

Timestamp: $(date)
Total Requests: $REQUESTS_TOTAL

Regex-Only Mode:
  Min:  ${REGEX_MIN}ms
  Mean: ${REGEX_MEAN}ms
  P50:  ${REGEX_P50}ms (target: < ${TARGET_P50_REGEX}ms)
  P95:  ${REGEX_P95}ms (target: < ${TARGET_P95_REGEX}ms)
  P99:  ${REGEX_P99}ms
  Max:  ${REGEX_MAX}ms
  Result: $([ "$REGEX_PASS" = true ] && echo "PASS ✓" || echo "FAIL ✗")
EOF

print_success "Results saved to: $RESULTS_FILE"

# ==============================================================================
# Summary
# ==============================================================================
print_header "Benchmark Summary"

if [ "$REGEX_PASS" = true ]; then
    print_success "Performance targets MET ✓"
    echo "  P50: ${REGEX_P50}ms < ${TARGET_P50_REGEX}ms ✓"
    echo -e "\n${GREEN}✓ Benchmark PASSED${NC}"
    exit 0
else
    print_failure "Performance targets MISSED ✗"
    echo "  P50: ${REGEX_P50}ms (target: ${TARGET_P50_REGEX}ms)"
    echo -e "\n${RED}✗ Benchmark FAILED${NC}"
    exit 1
fi
