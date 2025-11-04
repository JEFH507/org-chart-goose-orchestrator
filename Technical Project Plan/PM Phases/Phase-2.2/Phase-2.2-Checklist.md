# Phase 2.2 — Privacy Guard Enhancement — Checklist

**Status:** Ready to Execute  
**Last Updated:** 2025-11-04

This checklist tracks completion of all Phase 2.2 deliverables.

---

## Workstream A: Model Integration

### A1: Ollama HTTP Client
- [ ] Create `src/privacy-guard/src/ollama_client.rs`
- [ ] Add OllamaClient struct with HTTP client
- [ ] Implement `from_env()` for configuration (GUARD_MODEL_ENABLED, OLLAMA_URL, OLLAMA_MODEL)
- [ ] Implement `extract_entities()` method
- [ ] Add NER prompt template
- [ ] Parse Ollama response to extract entities
- [ ] Write unit tests (parsing, initialization)
- [ ] Tests pass: `cargo test --package privacy-guard`
- [ ] Commit with conventional message

**Estimated:** 1-2 hours  
**Status:** 📋 TODO

---

### A2: Hybrid Detection Logic
- [ ] Update `src/privacy-guard/src/detection.rs`
- [ ] Implement `detect_hybrid()` function
- [ ] Combine regex and NER model results
- [ ] Merge logic: consensus → HIGH confidence
- [ ] Add model-only detections as HIGH confidence
- [ ] Implement overlap detection
- [ ] Map NER entity types to EntityType enum
- [ ] Update `scan_handler` to use hybrid detection
- [ ] Update `mask_handler` to use hybrid detection
- [ ] Write integration tests
- [ ] Tests pass
- [ ] Commit with conventional message

**Estimated:** 2-3 hours  
**Status:** 📋 TODO  
**Depends on:** A1

---

### A3: Configuration & Fallback Logic
- [ ] Update `deploy/compose/.env.ce.example` with model env vars
- [ ] Update `deploy/compose/ce.dev.yml` privacy-guard service
- [ ] Add ollama dependency to privacy-guard service
- [ ] Update `/status` endpoint with model status fields
- [ ] Add `health_check()` method to OllamaClient
- [ ] Check Ollama health on startup (non-blocking)
- [ ] Log warnings if model unavailable
- [ ] Write fallback tests
- [ ] Tests pass
- [ ] Commit with conventional message

**Estimated:** 1-2 hours  
**Status:** 📋 TODO  
**Depends on:** A1, A2

---

## Workstream B: Documentation

### B1: Update Configuration Guide
- [ ] Add "Model-Enhanced Detection" section to `privacy-guard-config.md`
- [ ] Document environment variables (GUARD_MODEL_ENABLED, OLLAMA_URL, OLLAMA_MODEL)
- [ ] List supported models from ADR-0015
- [ ] Explain hybrid detection logic
- [ ] Document performance trade-offs
- [ ] Add "When to Use" guidance
- [ ] Add troubleshooting section
- [ ] Update existing env vars section
- [ ] Review and validate examples
- [ ] Commit with conventional message

**Estimated:** 30-60 minutes  
**Status:** 📋 TODO  
**Depends on:** A1, A2, A3

---

### B2: Update Integration Guide
- [ ] Update `/status` endpoint documentation
- [ ] Add new response fields (model_enabled, model_name)
- [ ] Update performance characteristics section
- [ ] Add regex-only vs model-enhanced comparison
- [ ] Note backward compatibility (API unchanged)
- [ ] Update controller integration section (no changes needed)
- [ ] Review and validate examples
- [ ] Commit with conventional message

**Estimated:** 30-60 minutes  
**Status:** 📋 TODO  
**Depends on:** A3

---

## Workstream C: Testing & Validation

### C1: Accuracy Validation Tests
- [ ] Create `tests/accuracy/compare_detection.sh`
- [ ] Test regex-only detection on Phase 2 fixtures
- [ ] Test model-enhanced detection on same fixtures
- [ ] Calculate improvement percentage
- [ ] Validate improvement ≥ 10% target
- [ ] Create `tests/accuracy/test_false_positives.sh`
- [ ] Test FP rate on clean samples
- [ ] Validate FP rate < 5% target
- [ ] Make scripts executable
- [ ] Run accuracy tests
- [ ] Document results in progress log
- [ ] Commit with conventional message

**Estimated:** 1-2 hours  
**Status:** 📋 TODO  
**Depends on:** A1, A2, A3

---

### C2: Smoke Tests
- [ ] Create `docs/tests/smoke-phase2.2.md`
- [ ] Document Test 1: Model status check
- [ ] Document Test 2: Model-enhanced detection
- [ ] Document Test 3: Fallback to regex (model disabled)
- [ ] Document Test 4: Performance with model
- [ ] Document Test 5: Backward compatibility
- [ ] Include expected outputs and pass criteria
- [ ] Run all smoke tests
- [ ] Document results in progress log
- [ ] Update state JSON with performance measurements
- [ ] Commit with conventional message

**Estimated:** 1 hour  
**Status:** 📋 TODO  
**Depends on:** A1, A2, A3

---

## Final Deliverables

### Completion Tasks
- [ ] All workstreams (A, B, C) complete
- [ ] All tests passing
- [ ] Accuracy improvement ≥ 10% achieved
- [ ] Performance within targets (P50 ≤ 700ms)
- [ ] Run full smoke test suite
- [ ] Write Phase-2.2-Completion-Summary.md
- [ ] Update state JSON (status=COMPLETE, results filled)
- [ ] Update progress log with completion
- [ ] Update PROJECT_TODO.md
- [ ] Update CHANGELOG.md
- [ ] Prepare PR(s) for review

**Estimated:** 1 hour  
**Status:** 📋 TODO  
**Depends on:** All workstreams

---

## Progress Tracking

**Completion:** 0% (0/7 major tasks)  
**Completed:** None  
**In Progress:** None  
**Blocked:** None

**Commits:** 0

**Branches:**
- feat/phase2.2-ollama-detection (not created yet)
- docs/phase2.2-guides (not created yet)
- test/phase2.2-validation (not created yet)

**Next Action:** Confirm user inputs and begin Task A1 (Ollama HTTP Client)

---

**Last Update:** 2025-11-04  
**Current Branch:** main (not started)  
**Current Workstream:** INIT  
**Current Task:** INIT
