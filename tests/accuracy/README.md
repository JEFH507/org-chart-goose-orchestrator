# Privacy Guard Accuracy Tests

**Phase:** 2.2 - Privacy Guard Enhancement  
**Purpose:** Measure detection accuracy improvement with model-enhanced detection  
**Date:** 2025-11-04

---

## Overview

This directory contains accuracy validation tests for comparing regex-only detection (Phase 2 baseline) with model-enhanced hybrid detection (Phase 2.2).

**Test Scripts:**
1. `compare_detection.sh` - Compares detection accuracy between modes
2. `test_false_positives.sh` - Validates false positive rate on clean samples

---

## Test 1: Detection Accuracy Comparison

**Script:** `compare_detection.sh`

**Purpose:** Measure improvement in PII detection when using hybrid detection (regex + NER model)

**Method:**
1. Run all PII samples through guard with model disabled (regex-only)
2. Count total entities detected
3. Restart guard with model enabled (hybrid detection)
4. Count total entities detected
5. Calculate improvement percentage

**Acceptance Criteria:**
- ✅ **Target:** ≥ 10% improvement in detection count
- ⚠️  **Marginal:** 5-10% improvement (may be acceptable)
- ❌ **Fail:** < 5% improvement

**Usage:**
```bash
# Start privacy-guard service
cd /home/papadoc/Gooseprojects/goose-org-twin
docker compose up -d privacy-guard ollama

# Run comparison test
./tests/accuracy/compare_detection.sh
```

**Expected Output:**
```
=== Privacy Guard Detection Accuracy Comparison ===
...
Step 1: Testing regex-only detection...
  Model disabled: ✓
  Regex-only: 145 entities detected across 150 samples

Step 2: Testing model-enhanced detection...
  Model enabled: ✓ (qwen3:0.6b)
  Model-enhanced: 165 entities detected across 150 samples

=== Results ===
Regex-only:      145 entities
Model-enhanced:  165 entities
Improvement:     13.8%

✅ PASS: Accuracy improvement >= 10% (got 13.8%)
```

**What It Tests:**
- Model improves recall on ambiguous PERSON names (no titles)
- Model catches ORGANIZATION entities (mapped to PERSON)
- Consensus detection increases confidence
- Graceful fallback when model unavailable

---

## Test 2: False Positive Rate

**Script:** `test_false_positives.sh`

**Purpose:** Ensure model-enhanced detection doesn't increase false positives

**Method:**
1. Run all clean samples (no PII) through guard
2. Count false positive detections
3. Calculate false positive rate
4. Compare to Phase 2 baseline (< 5%)

**Acceptance Criteria:**
- ✅ **Target:** < 5% false positive rate
- ⚠️  **Marginal:** 5-10% (may need tuning)
- ❌ **Fail:** ≥ 10% false positive rate

**Usage:**
```bash
# Test with current configuration
./tests/accuracy/test_false_positives.sh

# Test regex-only mode
./tests/accuracy/test_false_positives.sh --regex-only

# Test model-enhanced mode
./tests/accuracy/test_false_positives.sh --model-enhanced
```

**Expected Output:**
```
=== Privacy Guard False Positive Rate Test ===
...
Model enabled: true
Model name: qwen3:0.6b

Testing false positive rate on clean samples...
.....

=== Results ===
Total samples:       150
False positives:     3
False positive rate: 2.00%

✅ PASS: False positive rate < 5% (got 2.00%)
```

**What It Tests:**
- Model doesn't misclassify common business terms
- Clean technical text doesn't trigger false alarms
- False positive rate comparable to Phase 2 baseline
- Model adds recall without sacrificing precision

---

## Test Fixtures

**PII Samples:** `tests/fixtures/pii_samples.txt`
- 219 lines total
- ~150 test samples (after filtering comments/blanks)
- 8 entity types: SSN, EMAIL, PHONE, CREDIT_CARD, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER
- Mix of HIGH, MEDIUM, and LOW confidence patterns

**Clean Samples:** `tests/fixtures/clean_samples.txt`
- 163 lines total
- ~150 test samples
- Business/technical text with NO PII
- Baseline for false positive testing

---

## Performance Notes

**Execution Time:**
- Regex-only: ~30-60 seconds (150 samples × 200-400ms avg)
- Model-enhanced: ~2-3 minutes (150 samples × 500-700ms avg with model)

**Resource Usage:**
- Ollama container: ~1-2GB RAM (qwen3:0.6b model loaded)
- Privacy Guard: ~50MB RAM
- CPU: Moderate load during inference

**Tips:**
- Ensure Ollama model is pulled: `docker exec ollama ollama pull qwen3:0.6b`
- Wait for services to stabilize after restart (sleep 8-15s)
- Use `docker compose logs -f privacy-guard` to monitor

---

## Interpreting Results

### Detection Improvement

**High Improvement (>15%):**
- Model is catching significantly more entities
- Likely due to PERSON names without titles (e.g., "Alice Cooper" vs "Dr. Smith")
- ORGANIZATION entities (mapped to PERSON)
- May indicate regex patterns were too conservative

**Moderate Improvement (10-15%):**
- Expected range for Phase 2.2
- Model adds value on ambiguous cases
- Consensus detection working as designed

**Low Improvement (5-10%):**
- Marginal gain, may need model tuning
- Check if test fixtures have mostly HIGH confidence patterns (less room for improvement)
- Consider different model (llama3.2:3b for better accuracy)

**No Improvement (<5%):**
- Model not providing significant value
- Possible issues:
  - Model not loading correctly
  - Fallback to regex-only happening silently
  - Test fixtures don't stress model strengths

### False Positive Rate

**Excellent (<2%):**
- Better than Phase 2 baseline
- Model improving precision

**Good (2-5%):**
- Acceptable for production
- Monitor specific false positive types

**Marginal (5-10%):**
- May need pattern tuning
- Check false positive details in output
- Consider adjusting confidence thresholds

**Poor (>10%):**
- Not acceptable
- Review model prompts
- Check for overly broad patterns

---

## Troubleshooting

### Issue: Model not enabling

**Symptoms:**
```
⚠️  WARNING: Model not enabled (status: false)
```

**Resolution:**
1. Check environment variable: `docker compose config | grep GUARD_MODEL_ENABLED`
2. Verify Ollama running: `docker compose ps ollama`
3. Check Ollama logs: `docker compose logs ollama`
4. Pull model manually: `docker exec ollama ollama pull qwen3:0.6b`

### Issue: Low detection improvement

**Symptoms:**
```
❌ FAIL: Accuracy improvement < 5%
```

**Resolution:**
1. Verify model is actually enabled: `curl http://localhost:8089/status | jq '.model_enabled'`
2. Check if fallback is happening: `docker compose logs privacy-guard | grep "model extraction failed"`
3. Try larger model: Update `.env.ce` with `OLLAMA_MODEL=llama3.2:3b`
4. Review test fixtures: Ensure mix of HIGH/MEDIUM/LOW confidence patterns

### Issue: High false positive rate

**Symptoms:**
```
❌ FAIL: False positive rate >= 10%
```

**Resolution:**
1. Review false positive details in test output
2. Check for common patterns (e.g., product codes matching SSN regex)
3. Adjust confidence threshold: Update `policy.yaml` → `confidence_threshold: HIGH`
4. Refine regex patterns in `rules.yaml`
5. Check model prompt in `src/privacy-guard/src/ollama_client.rs`

### Issue: Tests timing out

**Symptoms:**
```
curl: (28) Operation timed out
```

**Resolution:**
1. Increase sleep time after restart: Edit scripts, change `sleep 8` → `sleep 15`
2. Check guard healthcheck: `curl http://localhost:8089/status`
3. Increase Ollama timeout: Update `ollama_client.rs` timeout from 5s → 10s
4. Check system resources: `docker stats`

---

## Results Documentation

After running tests, document results in:
1. `docs/tests/phase2.2-progress.md` (append test results entry)
2. `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-State.json` (update `performance_results`)
3. Create summary in Phase 2.2 completion artifacts

**Example Entry:**
```markdown
### 2025-11-04 — C1 Complete: Accuracy Validation Tests

**Accuracy Comparison:**
- Regex-only: 145 entities
- Model-enhanced: 165 entities
- Improvement: 13.8% ✅ (target: ≥10%)

**False Positive Rate:**
- Total samples: 150
- False positives: 3
- FP rate: 2.0% ✅ (target: <5%)

**Verdict:** Phase 2.2 accuracy targets MET
```

---

## References

- **Phase 2 Baseline:** `docs/tests/phase2-test-results.md`
- **Phase 2.2 Plan:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Execution-Plan.md`
- **Test Fixtures:** `tests/fixtures/README.md`
- **ADR-0015:** Model selection policy
- **ADR-0022:** PII detection rules

---

**Version:** 1.0  
**Created:** 2025-11-04  
**Last Updated:** 2025-11-04
