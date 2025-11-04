#!/bin/bash

# Create results directory
mkdir -p benchmark_results

echo "Starting Privacy Guard Performance Benchmark..."
echo "Running 100 requests..."

for i in {1..100}; do
  start=$(date +%s%N)
  curl -s -X POST http://localhost:8089/guard/mask \
    -H 'Content-Type: application/json' \
    -d '{
      "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789. Credit card: 4532015112830366. From IP: 192.168.1.100",
      "tenant_id": "test-org"
    }' > /dev/null
  end=$(date +%s%N)
  
  # Calculate duration in milliseconds
  duration=$(( (end - start) / 1000000 ))
  echo $duration
done | tee benchmark_results/latencies.txt | sort -n | awk '
  BEGIN {
    print "=== Privacy Guard Performance Results ==="
  }
  {
    arr[NR]=$1
    sum+=$1
  }
  END {
    print "Total requests: " NR
    print "Mean: " int(sum/NR) " ms"
    print "Min: " arr[1] " ms"
    print "Max: " arr[NR] " ms"
    print "P50 (median): " arr[int(NR*0.50)] " ms"
    print "P90: " arr[int(NR*0.90)] " ms"
    print "P95: " arr[int(NR*0.95)] " ms"
    print "P99: " arr[int(NR*0.99)] " ms"
    print ""
    print "=== Target Validation ==="
    if (arr[int(NR*0.50)] <= 500) print "✅ P50 <= 500ms: PASS"; else print "❌ P50 > 500ms: FAIL"
    if (arr[int(NR*0.95)] <= 1000) print "✅ P95 <= 1000ms: PASS"; else print "❌ P95 > 1000ms: FAIL"
    if (arr[int(NR*0.99)] <= 2000) print "✅ P99 <= 2000ms: PASS"; else print "❌ P99 > 2000ms: FAIL"
  }
'
