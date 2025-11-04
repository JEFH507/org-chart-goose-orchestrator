#!/bin/bash
set -e

BASE_URL="http://localhost:8089"
ITERATIONS=10
OUTPUT_FILE="/tmp/phase2.2_benchmark_results.txt"

echo "Phase 2.2 Performance Benchmark (Model-Enhanced)"
echo "Model: qwen3:0.6b (CPU-only)"
echo "Iterations: $ITERATIONS"
echo "---"

# Clean previous results
> "$OUTPUT_FILE"

# Warmup (first request may be slower)
echo "Warmup request..."
curl -s -X POST "$BASE_URL/guard/scan" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact Jane Smith at jane@example.com",
    "tenant_id": "warmup",
    "session_id": "warmup"
  }' > /dev/null

sleep 2

# Run benchmark
echo "Running $ITERATIONS iterations..."
for i in $(seq 1 $ITERATIONS); do
  START=$(date +%s%N)
  
  curl -s -X POST "$BASE_URL/guard/scan" \
    -H "Content-Type: application/json" \
    -d "{
      \"text\": \"Contact Person Number $i at email$i@test.com or call 555-111-222$i\",
      \"tenant_id\": \"benchmark\",
      \"session_id\": \"bench-$i\"
    }" > /dev/null
  
  END=$(date +%s%N)
  DURATION_NS=$((END - START))
  DURATION_MS=$((DURATION_NS / 1000000))
  
  echo "$DURATION_MS" >> "$OUTPUT_FILE"
  echo "  Request $i: ${DURATION_MS}ms"
done

# Calculate percentiles
echo "---"
echo "Calculating statistics..."

SORTED=$(sort -n "$OUTPUT_FILE")
COUNT=$(wc -l < "$OUTPUT_FILE")

P50_INDEX=$(((COUNT + 1) / 2))
P95_INDEX=$(((COUNT * 95 + 99) / 100))
P99_INDEX=$(((COUNT * 99 + 99) / 100))

P50=$(echo "$SORTED" | sed -n "${P50_INDEX}p")
P95=$(echo "$SORTED" | sed -n "${P95_INDEX}p")
P99=$(echo "$SORTED" | sed -n "${P99_INDEX}p")
MIN=$(echo "$SORTED" | head -1)
MAX=$(echo "$SORTED" | tail -1)

echo ""
echo "Results:"
echo "  Min:  ${MIN}ms"
echo "  P50:  ${P50}ms (target: 8000-15000ms for CPU)"
echo "  P95:  ${P95}ms (target: < 20000ms for CPU)"
echo "  P99:  ${P99}ms (target: < 30000ms for CPU)"
echo "  Max:  ${MAX}ms"
echo ""

# Check targets (CPU-only acceptable range)
PASS=true
if [ "$P50" -gt 15000 ]; then
  echo "⚠️  WARN: P50 exceeds expected range (${P50}ms > 15000ms)"
  if [ "$P50" -gt 20000 ]; then
    echo "❌ FAIL: P50 significantly exceeds target"
    PASS=false
  fi
fi

if [ "$P95" -gt 20000 ]; then
  echo "⚠️  WARN: P95 exceeds expected range (${P95}ms > 20000ms)"
  if [ "$P95" -gt 30000 ]; then
    echo "❌ FAIL: P95 significantly exceeds target"
    PASS=false
  fi
fi

if [ "$P99" -gt 30000 ]; then
  echo "❌ FAIL: P99 exceeds target (${P99}ms > 30000ms)"
  PASS=false
fi

if [ "$PASS" = true ]; then
  echo "✅ PASS: Performance within acceptable range for CPU-only inference"
fi

echo ""
echo "Note: CPU-only inference (no GPU) is expected to be 10-15s per request."
echo "This is ACCEPTABLE per user decision (2025-11-04)."

# Cleanup
rm -f "$OUTPUT_FILE"
