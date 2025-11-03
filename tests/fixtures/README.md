# Privacy Guard Test Fixtures

**Purpose:** Synthetic test data for validating PII detection and masking functionality.

**Date Created:** 2025-11-03  
**Owner:** Phase 2 Team  
**Status:** Active

---

## Files

### `pii_samples.txt`
Synthetic text containing **known PII** across all 8 entity types.

- **Lines:** 100+
- **Expected Detections:** 150+ entities
- **Entity Types Covered:** SSN, CREDIT_CARD, EMAIL, PHONE, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER
- **Use Cases:**
  - Validate detection engine accuracy (recall)
  - Test masking logic
  - Performance benchmarking
  - Regression testing

**Example Content:**
```
My SSN is 123-45-6789 for tax purposes.
Contact john.doe@example.com for details.
Call 555-123-4567 for support.
```

**Expected Behavior:**
- Guard should detect all PII entities listed
- High-confidence patterns should match reliably (>95% precision)
- Medium/Low confidence patterns should match with context

---

### `clean_samples.txt`
Synthetic text containing **NO PII** (negative test cases).

- **Lines:** 50+
- **Expected Detections:** 0
- **Content:** Normal business/technical text, generic numbers, dates without birth context
- **Use Cases:**
  - Validate false positive rate (precision)
  - Ensure guard doesn't over-detect
  - Baseline performance testing (fast path)

**Example Content:**
```
The project timeline includes three phases over six months.
Application started successfully on port 8080.
Version 1.2.3.4 includes bug fixes.
```

**Expected Behavior:**
- Guard should detect ZERO entities
- Acceptable: <5% false positive rate (e.g., version numbers matching IPv4)

---

### `expected_detections.json`
Detailed documentation of expected detection results.

- **Format:** JSON
- **Contents:**
  - Expected counts per entity type
  - Confidence level distribution
  - Known false positives/negatives
  - Edge cases and overlapping entities
  - Validation criteria
  - Test procedures

**Use Cases:**
- Reference for test assertions
- Track detection accuracy over time
- Document known limitations
- Guide manual smoke testing

---

## Usage

### 1. Integration Tests (Rust)

**Location:** `src/privacy-guard/tests/integration_tests.rs`

**Example:**
```rust
#[tokio::test]
async fn test_pii_samples_detection() {
    let samples = std::fs::read_to_string("../../tests/fixtures/pii_samples.txt").unwrap();
    let response = client.post("/guard/scan")
        .json(&json!({ "text": samples, "tenant_id": "test" }))
        .send()
        .await
        .unwrap();
    
    let result: ScanResponse = response.json().await.unwrap();
    
    // Should detect at least 150 entities
    assert!(result.detections.len() >= 150, 
        "Expected >=150 detections, got {}", result.detections.len());
    
    // Verify each entity type is detected
    let ssn_count = result.detections.iter()
        .filter(|d| d.entity_type == EntityType::SSN)
        .count();
    assert!(ssn_count >= 15, "Expected >=15 SSNs, got {}", ssn_count);
    
    // ... similar assertions for other types
}

#[tokio::test]
async fn test_clean_samples_no_detections() {
    let samples = std::fs::read_to_string("../../tests/fixtures/clean_samples.txt").unwrap();
    let response = client.post("/guard/scan")
        .json(&json!({ "text": samples, "tenant_id": "test" }))
        .send()
        .await
        .unwrap();
    
    let result: ScanResponse = response.json().await.unwrap();
    
    // Should detect zero entities (allow <5% false positive rate)
    assert!(result.detections.len() < 3, 
        "Expected 0 detections, got {}", result.detections.len());
}
```

---

### 2. Manual Testing (curl)

**Scan for PII (detection only):**
```bash
# Single line test
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact John Doe (SSN: 123-45-6789) at john.doe@example.com",
    "tenant_id": "test"
  }' | jq

# Full file test
PII_TEXT=$(cat tests/fixtures/pii_samples.txt)
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d "{\"text\": $(jq -Rs . <<< "$PII_TEXT"), \"tenant_id\": \"test\"}" \
  | jq '.detections | length'
# Expected output: 150+
```

**Mask PII:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "My SSN is 123-45-6789 and email is john@example.com",
    "tenant_id": "test",
    "mode": "MASK"
  }' | jq

# Expected output:
# {
#   "masked_text": "My SSN is 555-XXX-XXXX and email is EMAIL_a3f7b2c8@redacted.local",
#   "redactions": { "SSN": 1, "EMAIL": 1 },
#   "session_id": "sess_..."
# }
```

**Verify determinism:**
```bash
# Run mask twice with same input, compare outputs
RESULT1=$(curl -s -X POST http://localhost:8089/guard/mask -d '{"text":"SSN: 123-45-6789","tenant_id":"test"}' | jq -r '.masked_text')
RESULT2=$(curl -s -X POST http://localhost:8089/guard/mask -d '{"text":"SSN: 123-45-6789","tenant_id":"test"}' | jq -r '.masked_text')

if [ "$RESULT1" == "$RESULT2" ]; then
  echo "✅ Determinism verified"
else
  echo "❌ Outputs differ: $RESULT1 vs $RESULT2"
fi
```

---

### 3. Smoke Testing

**Procedure:** See `docs/tests/smoke-phase2.md`

**Quick Smoke Test:**
```bash
# 1. Start guard service
docker compose --profile privacy-guard up -d

# 2. Verify health
curl http://localhost:8089/status | jq

# 3. Test PII detection
curl -X POST http://localhost:8089/guard/scan \
  -d '{"text":"SSN: 123-45-6789, Email: test@example.com","tenant_id":"test"}' \
  | jq '.detections | length'
# Expected: 2

# 4. Test clean samples (no PII)
CLEAN_TEXT=$(cat tests/fixtures/clean_samples.txt)
curl -X POST http://localhost:8089/guard/scan \
  -d "{\"text\": $(jq -Rs . <<< "$CLEAN_TEXT"), \"tenant_id\": \"test\"}" \
  | jq '.detections | length'
# Expected: 0 (or <3 for <5% false positive rate)

# 5. Test masking
curl -X POST http://localhost:8089/guard/mask \
  -d '{"text":"Call 555-123-4567","tenant_id":"test"}' \
  | jq '.masked_text'
# Expected: Phone number masked with FPE
```

---

### 4. Performance Benchmarking

**Measure P50/P95/P99 latency:**

```bash
# Use Apache Bench or similar tool
ab -n 1000 -c 10 -p payload.json -T application/json \
  http://localhost:8089/guard/scan

# Or custom script
for i in {1..100}; do
  START=$(date +%s%N)
  curl -s -X POST http://localhost:8089/guard/mask \
    -d @pii_samples_payload.json > /dev/null
  END=$(date +%s%N)
  DURATION=$(( (END - START) / 1000000 ))  # milliseconds
  echo "$DURATION"
done | sort -n | awk '
  BEGIN { sum=0; count=0; }
  { 
    arr[NR]=$1; 
    sum+=$1; 
    count++; 
  }
  END {
    asort(arr);
    p50_idx = int(count * 0.5);
    p95_idx = int(count * 0.95);
    p99_idx = int(count * 0.99);
    printf "P50: %d ms\n", arr[p50_idx];
    printf "P95: %d ms\n", arr[p95_idx];
    printf "P99: %d ms\n", arr[p99_idx];
    printf "Avg: %d ms\n", sum/count;
  }
'

# Expected targets:
# P50: <500ms
# P95: <1000ms
# P99: <2000ms
```

---

## Validation Criteria

### Detection Accuracy
- **Precision:** >95% (detected PII is actually PII)
- **Recall:** >90% (actual PII is detected)
- **False Positive Rate:** <5% on clean_samples.txt

### Confidence Levels
- **HIGH:** >98% precision
- **MEDIUM:** >90% precision
- **LOW:** >80% precision (context-dependent)

### Performance
- **pii_samples.txt:** P50 <500ms, P95 <1s
- **clean_samples.txt:** P50 <100ms (no detections, faster)

---

## Known Limitations

### False Positives (Acceptable)
- **Version numbers:** `1.2.3.4` matches IPv4 pattern
  - **Mitigation:** Filter via context or post-processing
- **Generic numbers:** `1234567890` may match ACCOUNT_NUMBER
  - **Mitigation:** Tune confidence threshold to MEDIUM or HIGH

### False Negatives (Acceptable for Phase 2)
- **Unicode names:** `José García` (non-ASCII characters)
  - **Planned:** Phase 2.2 with ML model or enhanced regex
- **Internationalized emails:** `user@münchen.de`
  - **Planned:** IDN-aware regex or ML model

### Edge Cases
- **Overlapping entities:** Guard should handle gracefully (higher confidence wins)
- **Partial redactions:** Already-masked text (`xxx-xx-6789`) should not re-detect
- **Case sensitivity:** Patterns should be case-insensitive

---

## Maintenance

### When to Update

1. **New entity types added:** Add samples to `pii_samples.txt`, update `expected_detections.json`
2. **Pattern refinements:** Re-run tests, update expected counts if changed
3. **False positive/negative discovered:** Add edge case to fixtures
4. **Performance regression:** Add benchmark samples

### Versioning

Update `expected_detections.json` version field when:
- Expected detection counts change >10%
- New entity types added
- Major pattern refactoring

### Review Frequency

- **Phase completion:** Full review of accuracy and performance
- **Quarterly:** Check for new PII patterns in industry standards
- **On user feedback:** Add real-world examples that failed detection

---

## Related Documentation

- **Rules Configuration:** `deploy/compose/guard-config/rules.yaml`
- **Policy Configuration:** `deploy/compose/guard-config/policy.yaml`
- **Integration Tests:** `src/privacy-guard/tests/integration_tests.rs`
- **Smoke Test Guide:** `docs/tests/smoke-phase2.md`
- **ADR-0022:** PII Detection Rules and FPE

---

**Changelog:**
- 2025-11-03: Initial creation with 8 entity types, 100+ PII samples, 50+ clean samples
