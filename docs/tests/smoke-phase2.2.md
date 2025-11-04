# Phase 2.2 Smoke Test Procedure — Privacy Guard Model Enhancement

**Phase:** Phase 2.2 - Privacy Guard Enhancement  
**Version:** 1.0  
**Date:** 2025-11-04  
**Purpose:** Validate model-enhanced detection functionality, performance, and backward compatibility

---

## Overview

This document provides step-by-step smoke tests to validate Phase 2.2 enhancements:
- Model-enhanced detection via Ollama + qwen3:0.6b
- Hybrid detection (regex + NER model consensus)
- Graceful fallback to regex-only
- Performance within acceptable targets
- Backward compatibility with Phase 2

**Prerequisites:**
- Docker Compose environment running (`ce.dev.yml`)
- Privacy Guard service healthy
- Ollama service healthy with qwen3:0.6b model pulled
- `.env.ce` file configured (GUARD_MODEL_ENABLED set as needed)

**Test Environment:**
- Base URL: `http://localhost:8089`
- Services: privacy-guard, ollama
- Model: qwen3:0.6b (522MB, 40K context, Nov 2024)

---

## Test Suite

### Test 1: Model Status Check ✅

**Purpose:** Verify model configuration is reported correctly in status endpoint

**Steps:**
```bash
# Check status endpoint reports model configuration
curl -s http://localhost:8089/status | jq .
```

**Expected Output:**
```json
{
  "status": "healthy",
  "mode": "Mask",
  "rule_count": 24,
  "config_loaded": true,
  "model_enabled": true,
  "model_name": "qwen3:0.6b"
}
```

**Pass Criteria:**
- ✅ `model_enabled` field present (boolean)
- ✅ `model_name` field present (string)
- ✅ Values match configuration in `.env.ce`
- ✅ All Phase 2 fields still present (backward compatible)

**Result:** ⬜ PENDING

---

### Test 2: Model-Enhanced Detection ✅

**Purpose:** Verify hybrid detection improves entity detection over regex-only

**Test Case 2A: Person Name Without Title**

Regex-only struggles with person names lacking titles (Dr., Mr., etc.).  
Model should detect "Jane Smith" as PERSON.

```bash
# Test model-enhanced detection
curl -s -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact Jane Smith at jane.smith@example.com or 555-123-4567",
    "tenant_id": "test-tenant",
    "session_id": "smoke-test-2a"
  }' | jq .
```

**Expected Output:**
```json
{
  "detections": [
    {
      "entity_type": "PERSON",
      "start": 8,
      "end": 18,
      "confidence": "HIGH",
      "text": "Jane Smith"
    },
    {
      "entity_type": "EMAIL",
      "start": 22,
      "end": 46,
      "confidence": "HIGH",
      "text": "jane.smith@example.com"
    },
    {
      "entity_type": "PHONE",
      "start": 50,
      "end": 62,
      "confidence": "HIGH",
      "text": "555-123-4567"
    }
  ]
}
```

**Pass Criteria:**
- ✅ PERSON entity detected (Jane Smith)
- ✅ Confidence is HIGH (model consensus or model-only)
- ✅ EMAIL and PHONE also detected (hybrid working)

**Test Case 2B: Organization Name**

Model should detect organizations not covered by regex patterns.

```bash
# Test organization detection
curl -s -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Call Acme Corporation at 555-987-6543 for support",
    "tenant_id": "test-tenant",
    "session_id": "smoke-test-2b"
  }' | jq .
```

**Expected Output:**
```json
{
  "detections": [
    {
      "entity_type": "PERSON",
      "start": 5,
      "end": 21,
      "confidence": "HIGH",
      "text": "Acme Corporation"
    },
    {
      "entity_type": "PHONE",
      "start": 25,
      "end": 37,
      "confidence": "HIGH",
      "text": "555-987-6543"
    }
  ]
}
```

**Note:** Organizations map to PERSON type (per `map_ner_type()` in detection.rs)

**Pass Criteria:**
- ✅ Organization detected (mapped to PERSON)
- ✅ PHONE detected (regex works)

**Result:** ⬜ PENDING

---

### Test 3: Graceful Fallback to Regex-Only ✅

**Purpose:** Verify system falls back to regex when model is disabled or unavailable

**Test Case 3A: Model Disabled**

Set `GUARD_MODEL_ENABLED=false` in `.env.ce`, restart service.

```bash
# Update .env.ce (manual step documented)
# GUARD_MODEL_ENABLED=false

# Restart privacy-guard
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
docker compose --env-file .env.ce up -d privacy-guard

# Wait for service to be healthy
sleep 5

# Check status
curl -s http://localhost:8089/status | jq '.model_enabled'
```

**Expected Output:**
```json
false
```

**Test regex-only detection:**
```bash
curl -s -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Dr. John Doe called from 555-234-5678",
    "tenant_id": "test-tenant",
    "session_id": "smoke-test-3a"
  }' | jq .
```

**Expected Output:**
```json
{
  "detections": [
    {
      "entity_type": "PERSON",
      "start": 0,
      "end": 12,
      "confidence": "HIGH",
      "text": "Dr. John Doe"
    },
    {
      "entity_type": "PHONE",
      "start": 25,
      "end": 37,
      "confidence": "HIGH",
      "text": "555-234-5678"
    }
  ]
}
```

**Pass Criteria:**
- ✅ `model_enabled` is `false` in status
- ✅ Detection still works (regex patterns)
- ✅ PERSON with title detected (regex HIGH confidence)
- ✅ No errors or degradation

**Test Case 3B: Model Unavailable**

Stop Ollama service to simulate model unavailability.

```bash
# Stop Ollama
docker compose stop ce_ollama

# Re-enable model in .env.ce
# GUARD_MODEL_ENABLED=true

# Restart privacy-guard
docker compose --env-file .env.ce up -d privacy-guard
sleep 5

# Test detection
curl -s -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Email support@acme.com for help",
    "tenant_id": "test-tenant",
    "session_id": "smoke-test-3b"
  }' | jq .
```

**Expected Output:**
```json
{
  "detections": [
    {
      "entity_type": "EMAIL",
      "start": 6,
      "end": 22,
      "confidence": "HIGH",
      "text": "support@acme.com"
    }
  ]
}
```

**Pass Criteria:**
- ✅ Detection works despite Ollama unavailable
- ✅ Falls back to regex-only silently
- ✅ No errors returned to client

**Cleanup:**
```bash
# Restart Ollama for remaining tests
docker compose up -d ce_ollama
sleep 10
```

**Result:** ⬜ PENDING

---

### Test 4: Performance Benchmarking ✅

**Purpose:** Validate performance is within acceptable targets for model-enhanced mode

**Prerequisites:**
- Model enabled: `GUARD_MODEL_ENABLED=true`
- Ollama service healthy
- Privacy-guard service healthy

**Benchmark Script:**

Create `tests/performance/benchmark_phase2.2.sh`:

```bash
#!/bin/bash
set -e

BASE_URL="http://localhost:8089"
ITERATIONS=20
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
      \"text\": \"Contact Person $i at email$i@test.com or call 555-$i$i$i-$i$i$i$i\",
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
echo "  P50:  ${P50}ms (target: ≤ 700ms)"
echo "  P95:  ${P95}ms (target: ≤ 1000ms)"
echo "  P99:  ${P99}ms (target: ≤ 2000ms)"
echo "  Max:  ${MAX}ms"
echo ""

# Check targets
PASS=true
if [ "$P50" -gt 700 ]; then
  echo "❌ FAIL: P50 exceeds target (${P50}ms > 700ms)"
  PASS=false
fi

if [ "$P95" -gt 1000 ]; then
  echo "❌ FAIL: P95 exceeds target (${P95}ms > 1000ms)"
  PASS=false
fi

if [ "$P99" -gt 2000 ]; then
  echo "❌ FAIL: P99 exceeds target (${P99}ms > 2000ms)"
  PASS=false
fi

if [ "$PASS" = true ]; then
  echo "✅ PASS: All performance targets met"
fi

# Cleanup
rm -f "$OUTPUT_FILE"
```

**Execution:**
```bash
chmod +x tests/performance/benchmark_phase2.2.sh
./tests/performance/benchmark_phase2.2.sh
```

**Expected Output:**
```
Phase 2.2 Performance Benchmark (Model-Enhanced)
Model: qwen3:0.6b (CPU-only)
Iterations: 20
---
Warmup request...
Running 20 iterations...
  Request 1: 12500ms
  Request 2: 11800ms
  ...
  Request 20: 10200ms
---
Calculating statistics...

Results:
  Min:  9800ms
  P50:  11200ms (target: ≤ 700ms)
  P95:  12800ms (target: ≤ 1000ms)
  P99:  13500ms (target: ≤ 2000ms)
  Max:  14200ms

❌ FAIL: P50 exceeds target (11200ms > 700ms)
❌ FAIL: P95 exceeds target (12800ms > 1000ms)
```

**Note:** CPU-only inference is expected to be 10-15s per request. This is **ACCEPTABLE** per user decision (2025-11-04).

**Updated Pass Criteria (CPU-only):**
- ✅ P50: 8-15 seconds (acceptable for CPU inference)
- ✅ P95: < 20 seconds
- ✅ P99: < 30 seconds
- ✅ No timeouts (60s timeout configured)
- ✅ No errors

**Result:** ⬜ PENDING

---

### Test 5: Backward Compatibility ✅

**Purpose:** Verify Phase 2 clients work without changes (API unchanged)

**Test Case 5A: Phase 2 Endpoints Still Work**

```bash
# Test /status endpoint (Phase 2 fields present)
STATUS=$(curl -s http://localhost:8089/status)
echo "$STATUS" | jq .

# Verify Phase 2 fields
echo "$STATUS" | jq -e '.status' > /dev/null && echo "✅ status field present"
echo "$STATUS" | jq -e '.mode' > /dev/null && echo "✅ mode field present"
echo "$STATUS" | jq -e '.rule_count' > /dev/null && echo "✅ rule_count field present"
echo "$STATUS" | jq -e '.config_loaded' > /dev/null && echo "✅ config_loaded field present"
```

**Test Case 5B: Phase 2 Detection Still Works**

```bash
# Use exact Phase 2 smoke test sample
curl -s -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "John Doe: 123-45-6789, jane@example.com, 555-123-4567",
    "tenant_id": "phase2-compat",
    "session_id": "compat-test"
  }' | jq .
```

**Expected Output:**
```json
{
  "detections": [
    {
      "entity_type": "PERSON",
      "start": 0,
      "end": 8,
      "confidence": "HIGH",
      "text": "John Doe"
    },
    {
      "entity_type": "SSN",
      "start": 10,
      "end": 21,
      "confidence": "HIGH",
      "text": "123-45-6789"
    },
    {
      "entity_type": "EMAIL",
      "start": 23,
      "end": 39,
      "confidence": "HIGH",
      "text": "jane@example.com"
    },
    {
      "entity_type": "PHONE",
      "start": 41,
      "end": 53,
      "confidence": "HIGH",
      "text": "555-123-4567"
    }
  ]
}
```

**Test Case 5C: Phase 2 Masking Still Works**

```bash
# Test masking endpoint
curl -s -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "SSN: 123-45-6789, Email: jane@example.com",
    "tenant_id": "phase2-compat",
    "session_id": "compat-mask",
    "mode": "MASK"
  }' | jq .
```

**Expected Output:**
```json
{
  "masked_text": "SSN: 999-XX-XXXX, Email: EMAIL_<16_hex_chars>",
  "redaction_count": 2
}
```

**Pass Criteria:**
- ✅ All Phase 2 status fields present
- ✅ Detection results match Phase 2 behavior (same entities)
- ✅ Masking works with same strategies (FPE for SSN, pseudonym for EMAIL)
- ✅ API request/response format unchanged
- ✅ No breaking changes

**Result:** ⬜ PENDING

---

## Execution Summary

### Test Results

| Test | Description | Status | Notes |
|------|-------------|--------|-------|
| Test 1 | Model Status Check | ⬜ PENDING | Verify model_enabled, model_name fields |
| Test 2 | Model-Enhanced Detection | ⬜ PENDING | Verify improved person/org detection |
| Test 3 | Graceful Fallback | ⬜ PENDING | Verify regex-only when model unavailable |
| Test 4 | Performance Benchmarking | ⬜ PENDING | CPU-only: 10-15s acceptable |
| Test 5 | Backward Compatibility | ⬜ PENDING | Verify Phase 2 API unchanged |

### Performance Summary

| Metric | Phase 2 (Regex-only) | Phase 2.2 (Model-enhanced) | Target | Status |
|--------|----------------------|----------------------------|--------|--------|
| P50 | 16ms | ⬜ PENDING | ≤ 700ms (cloud) / 8-15s (CPU) | ⬜ |
| P95 | 22ms | ⬜ PENDING | ≤ 1000ms (cloud) / < 20s (CPU) | ⬜ |
| P99 | 23ms | ⬜ PENDING | ≤ 2000ms (cloud) / < 30s (CPU) | ⬜ |

**Note:** Phase 2.2 targets updated for CPU-only inference (user accepted 10-15s per request).

---

## Sign-Off Checklist

- [ ] All 5 smoke tests executed
- [ ] Test results documented
- [ ] Performance measurements recorded
- [ ] Any failures analyzed and documented
- [ ] Backward compatibility verified
- [ ] Ready for production deployment

---

## Troubleshooting

### Issue: Model timeout errors

**Symptom:** Requests fail after 60 seconds  
**Solution:** Check Ollama service health, verify model loaded, check CPU usage  
**Workaround:** Disable model (`GUARD_MODEL_ENABLED=false`)

### Issue: Model not detected in status

**Symptom:** `model_enabled` always `false`  
**Solution:** Check `.env.ce` has `GUARD_MODEL_ENABLED=true`, restart with `--env-file` flag  
**Command:** `docker compose --env-file .env.ce up -d privacy-guard`

### Issue: Performance worse than expected

**Symptom:** P50 > 20 seconds  
**Solution:** Normal for CPU-only inference with 8GB RAM, no GPU  
**Options:** Accept current performance or add GPU support (out of scope)

### Issue: Ollama service unhealthy

**Symptom:** Privacy-guard won't start, depends_on condition fails  
**Solution:** Check Ollama logs, verify healthcheck (`docker compose exec ce_ollama ollama list`)  
**Fix:** Ensure Ollama 0.12.9+ and healthcheck uses `ollama list` command

---

## References

### Phase 2.2 Documentation
- **Execution Plan:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Execution-Plan.md`
- **Configuration Guide:** `docs/guides/privacy-guard-config.md` (v1.1)
- **Integration Guide:** `docs/guides/privacy-guard-integration.md` (v1.1)
- **Accuracy Tests:** `tests/accuracy/README.md`

### Phase 2 Baseline
- **Smoke Tests:** `docs/tests/smoke-phase2.md`
- **Completion Summary:** `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`

### ADRs
- **ADR-0015:** Guard Model Policy and Selection
- **ADR-0021:** Privacy Guard Rust Implementation
- **ADR-0022:** PII Detection Rules and FPE

---

**Version:** 1.0  
**Last Updated:** 2025-11-04  
**Status:** Ready for Execution
