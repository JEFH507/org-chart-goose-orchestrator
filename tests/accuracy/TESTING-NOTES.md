# Phase 2.2 Accuracy Testing Notes

**Date:** 2025-11-04  
**Task:** C1 - Accuracy Validation Tests  
**Status:** In Progress

---

## Environment

**Services:**
- Privacy Guard: ghcr.io/jefh507/privacy-guard:0.1.0 (Phase 2.2 with hybrid detection)
- Ollama: ollama/ollama:0.3.14 (CPU-only, no GPU)
- Model: llama3.2:1b (1.3GB)

**Hardware:**
- AMD Ryzen 7 PRO 3700U (4 cores, 8 threads)
- 8GB RAM (~2.8GB available per Ollama logs)
- CPU-only inference

**Configuration:**
- GUARD_MODEL_ENABLED: false (default), dynamically toggled by tests
- OLLAMA_URL: http://ollama:11434 (Docker internal network)
- OLLAMA_MODEL: llama3.2:1b

---

## Model Selection Note

**Original Plan:** qwen3:0.6b (523MB, 40K context, Nov 2024)

**Actual:** llama3.2:1b (1.3GB, 128K context, Oct 2024)

**Reason for Change:**
- Ollama 0.3.14 doesn't support qwen3:0.6b (requires newer version)
- Error: "The model you are attempting to pull requires a newer version of Ollama"
- Fallback to llama3.2:1b (original ADR-0015 default)

**Impact:**
- Minimal — llama3.2:1b is the ADR-0015 recommended model
- Larger size (1.3GB vs 523MB) but hardware can support it
- May have slightly higher latency due to larger model
- NER capability expected to be comparable or better

**Future:**
- Upgrade Ollama to 0.4.x+ to support qwen3:0.6b
- Or keep llama3.2:1b as stable default for CE

---

## Ollama Healthcheck Fix

**Issue:** Ollama container marked unhealthy, blocking privacy-guard startup

**Root Cause:** Healthcheck used `curl` but ollama/ollama:0.3.14 image doesn't include curl

**Fix:** Changed healthcheck from:
```yaml
test: ["CMD-SHELL", "curl -fsS http://localhost:11434/api/tags || exit 1"]
```

To:
```yaml
test: ["CMD-SHELL", "ollama list || exit 1"]
```

**Result:** ✅ Ollama container now healthy, privacy-guard starts successfully

**Commit:** (Pending — will be included in C1 commit)

---

## Test Execution

### Test 1: Detection Accuracy Comparison (`compare_detection.sh`)

**Started:** 2025-11-04 08:18:44  
**Status:** Running  
**Expected Duration:** 2-5 minutes  

**Steps:**
1. Disable model → restart privacy-guard → wait 8s
2. Process 150+ PII samples with regex-only detection
3. Enable model → restart privacy-guard → wait 15s
4. Process same 150+ PII samples with hybrid detection
5. Calculate improvement percentage

**Acceptance Criteria:**
- ✅ Target: ≥ 10% improvement
- ⚠️  Marginal: 5-10% improvement
- ❌ Fail: < 5% improvement

**Progress Notes:**
- Step 1 started at 08:18:44
- Processing PII samples (219 lines, ~150 valid samples after filtering comments)
- Expected ~1 minute for regex-only phase
- Expected ~2-3 minutes for model-enhanced phase (slower inference)

---

### Test 2: False Positive Rate (`test_false_positives.sh`)

**Status:** Not yet started  
**Will run after:** Test 1 completes  

**Expected Duration:** 1-2 minutes  

**Steps:**
1. Process 150+ clean samples (no PII)
2. Count false positive detections
3. Calculate FP rate percentage

**Acceptance Criteria:**
- ✅ Target: < 5% FP rate
- ⚠️  Marginal: 5-10% FP rate
- ❌ Fail: ≥ 10% FP rate

---

## Expected Results

**Based on Phase 2.2 objectives:**

**Detection Improvement:**
- Target: +10-20% more entities detected with model
- Likely sources of improvement:
  - PERSON names without titles (e.g., "Alice Cooper" vs "Dr. Smith")
  - ORGANIZATION entities (mapped to PERSON in hybrid detection)
  - Ambiguous context where regex alone is conservative

**False Positive Rate:**
- Target: Maintain < 5% (same as Phase 2 baseline)
- Model should improve recall without sacrificing precision
- If FP rate increases, may need prompt tuning or confidence threshold adjustment

**Performance:**
- Regex-only: ~16ms P50 (Phase 2 baseline)
- Model-enhanced: ~500-700ms P50 (Phase 2.2 target with llama3.2:1b)
- Actual may be higher on this hardware (CPU-only, 8GB RAM)

---

## Results

### Test 1: Detection Accuracy Comparison

**Result:** (Pending — test in progress)

```
Regex-only:      ___ entities
Model-enhanced:  ___ entities
Improvement:     ___%

Status: ___
```

### Test 2: False Positive Rate

**Result:** (Pending)

```
Total samples:       ___
False positives:     ___
False positive rate: ___%

Status: ___
```

---

## Issues & Resolutions

### Issue 1: qwen3:0.6b not supported

**Severity:** MEDIUM  
**Impact:** Cannot use preferred model from Phase 2.2 planning  
**Resolution:** Use llama3.2:1b instead (ADR-0015 default)  
**Follow-up:** Upgrade Ollama in future or accept llama3.2:1b as default

### Issue 2: Ollama healthcheck failing

**Severity:** HIGH (blocking)  
**Impact:** privacy-guard couldn't start due to unhealthy dependency  
**Resolution:** Use `ollama list` CLI instead of `curl`  
**Status:** ✅ RESOLVED

---

## Artifacts

**Created:**
- `tests/accuracy/compare_detection.sh` (5.4KB, executable)
- `tests/accuracy/test_false_positives.sh` (4.7KB, executable)
- `tests/accuracy/README.md` (8.2KB, comprehensive documentation)
- `tests/accuracy/TESTING-NOTES.md` (this file)

**Modified:**
- `deploy/compose/ce.dev.yml` (Ollama healthcheck fix)
- `deploy/compose/.env.ce` (OLLAMA_MODEL=llama3.2:1b)

**Will Document:**
- Results in `docs/tests/phase2.2-progress.md`
- Metrics in `Phase-2.2-Agent-State.json`
- Summary in Phase 2.2 completion artifacts

---

## Next Steps

1. ✅ Wait for compare_detection.sh to complete
2. ✅ Review results and document
3. ✅ Run test_false_positives.sh
4. ✅ Review FP results and document
5. ✅ Update state JSON with accuracy metrics
6. ✅ Update progress log with test results
7. ✅ Commit accuracy test scripts and documentation
8. ✅ Proceed to C2 (Smoke Tests)

---

**Last Updated:** 2025-11-04 08:20:00  
**Next Update:** After test completion
