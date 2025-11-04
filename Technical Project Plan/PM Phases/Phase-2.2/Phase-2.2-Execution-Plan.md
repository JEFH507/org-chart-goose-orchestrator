# Phase 2.2 — Privacy Guard Enhancement — Execution Plan

**Phase:** Phase 2.2 - Privacy Guard Enhancement  
**Effort:** Small (S = ≤ 2 days)  
**Status:** Ready to Execute  
**Date:** 2025-11-04

---

## Objectives

Enhance the existing Privacy Guard service (Phase 2) with local NER model support to improve PII detection accuracy while maintaining:
- All existing functionality (HTTP API, modes, strategies)
- Local-only posture (no cloud exposure)
- Backward compatibility (no breaking changes)
- Small scope (≤ 2 days effort)

---

## Success Criteria

**Functional:**
- ✅ Local NER model (Ollama) integrated with Privacy Guard
- ✅ Hybrid detection (regex + model) working
- ✅ Graceful fallback to regex-only if model unavailable
- ✅ Configuration via environment variables
- ✅ All Phase 2 functionality preserved

**Non-Functional:**
- ✅ Detection accuracy improvement: +10-20% (measured)
- ✅ Performance: P50 ≤ 700ms with model (200ms increase acceptable)
- ✅ False positive rate: < 5% (unchanged from Phase 2)
- ✅ Backward compatible API (no client changes needed)

**Deliverables:**
- ✅ Updated privacy-guard service code (Rust)
- ✅ Ollama client integration
- ✅ Hybrid detection logic
- ✅ Configuration documentation
- ✅ Accuracy validation tests
- ✅ Smoke tests
- ✅ Completion summary

---

## Scope

### In Scope (Phase 2.2)
- Ollama HTTP client for NER calls
- Hybrid detection combining regex + model results
- Environment-based configuration (model enable/disable, model selection)
- Graceful fallback to regex-only
- Documentation updates (config guide, integration guide)
- Accuracy measurement and validation
- Smoke tests for model-enhanced detection

### Out of Scope (Future)
- Custom NER model training
- Cloud-based models (OpenAI, Anthropic)
- Multiple model ensemble
- GPU optimization
- Provider middleware integration
- Agent-side model calls (Phase 3+)
- Image/file content analysis
- Real-time model tuning

---

## Architecture

### Current (Phase 2)
```
Client → Privacy Guard (Rust HTTP service)
           ├─ Regex Detection → Masking → Response
           └─ In-memory State
```

### Enhanced (Phase 2.2)
```
Client → Privacy Guard (Rust HTTP service)
           ├─ Hybrid Detection:
           │    ├─ Regex (fast, high precision)
           │    └─ Ollama NER (slower, better recall)
           │         └─ http://ollama:11434 (local container)
           ├─ Merge Results (consensus → HIGH confidence)
           └─ Masking (unchanged) → Response
```

**Key Design Decisions:**
- Ollama as HTTP service (already in CE defaults)
- Async HTTP calls (reqwest crate, already in use)
- Fail-open: if Ollama unavailable, fall back to regex-only
- Model configurable via env var (default: llama3.2:1b per ADR-0015)
- No changes to external API (same endpoints, same responses)

---

## Workstream Breakdown

### Workstream A: Model Integration (4-6 hours)
**Branch:** `feat/phase2.2-ollama-detection`

**A1: Ollama HTTP Client** (1-2 hours)
- Create `ollama_client.rs` module
- HTTP client for `/api/generate` endpoint
- Environment-based configuration (GUARD_MODEL_ENABLED, OLLAMA_URL, OLLAMA_MODEL)
- NER prompt template
- Response parsing
- Unit tests

**A2: Hybrid Detection Logic** (2-3 hours)
- Update `detection.rs` with `detect_hybrid()` function
- Merge regex and model results
- Consensus detection (both methods → HIGH confidence)
- Model-only detection (HIGH confidence)
- Regex-only fallback
- Integration tests

**A3: Configuration & Fallback** (1-2 hours)
- Update compose files with model env vars
- Add model status to `/status` endpoint
- Ollama health check on startup
- Graceful degradation logic
- Fallback tests

**Deliverables:**
- Updated Rust code (~300 lines)
- Unit and integration tests
- Compose configuration
- Working hybrid detection

---

### Workstream B: Documentation (1-2 hours)
**Branch:** `docs/phase2.2-guides`

**B1: Update Configuration Guide** (30-60 min)
- Add "Model-Enhanced Detection" section to `privacy-guard-config.md`
- Document supported models (ADR-0015)
- Explain hybrid detection
- Performance trade-offs
- Troubleshooting

**B2: Update Integration Guide** (30-60 min)
- Update `/status` response with new fields
- Update performance characteristics
- Note backward compatibility
- Client impact (none)

**Deliverables:**
- Updated configuration guide (+80 lines)
- Updated integration guide (+40 lines)

---

### Workstream C: Testing & Validation (2-3 hours)
**Branch:** `test/phase2.2-validation` (or same as A)

**C1: Accuracy Validation** (1-2 hours)
- Create comparison test (regex vs model)
- Measure detection improvement on Phase 2 fixtures
- Validate false positive rate unchanged
- Document results

**C2: Smoke Tests** (1 hour)
- Create `smoke-phase2.2.md` (5 tests)
- Model status check
- Enhanced detection test
- Fallback test
- Performance benchmark
- Backward compatibility test
- Execute and document results

**Deliverables:**
- Accuracy test scripts
- Smoke test procedure
- Validation results (improvement %, performance)

---

## Dependencies

### Phase Dependencies
- Phase 2 complete (baseline privacy-guard service) ✅
- Phase 0 (Ollama in CE defaults) ✅

### Infrastructure Dependencies
- Ollama container running (already in `ce.dev.yml`)
- llama3.2:1b model pulled (or will pull on first use)
- Docker Compose with privacy-guard profile

### External Dependencies
- None (all local)

---

## Risk Analysis

### Risk 1: Model Latency Too High
**Probability:** Medium  
**Impact:** Medium  
**Mitigation:**
- Use smallest recommended model (llama3.2:1b, CPU-friendly)
- Make model optional (GUARD_MODEL_ENABLED flag)
- Target P50 ≤ 700ms (200ms increase acceptable)
- Document fallback to regex-only for high-volume use cases

### Risk 2: Model Availability Issues
**Probability:** Low  
**Impact:** Low  
**Mitigation:**
- Graceful fallback to regex-only (fail-open)
- Health check on startup (non-blocking)
- Clear logging when model unavailable
- No breaking changes to API

### Risk 3: Accuracy Not Improved
**Probability:** Low  
**Impact:** Medium  
**Mitigation:**
- Use proven NER models (Llama 3.2 has good NER capabilities)
- Measure improvement on Phase 2 test fixtures
- Accept +10% as success threshold
- If < 10%, document and defer to larger model or different approach

### Risk 4: Integration Complexity
**Probability:** Low  
**Impact:** Low  
**Mitigation:**
- Small scope (single HTTP client, ~300 LOC)
- Reuse existing reqwest dependency
- Similar pattern to controller guard client
- No persistence or state changes needed

---

## Testing Strategy

### Unit Tests
- Ollama client initialization and configuration
- NER response parsing
- Hybrid detection merge logic
- Fallback scenarios

### Integration Tests
- Hybrid detection with real Ollama calls (or mocked)
- Consensus detection (both methods agree)
- Model-only detection
- Regex-only fallback

### Accuracy Tests
- Compare regex-only vs model-enhanced on Phase 2 fixtures
- Measure improvement %
- Validate false positive rate unchanged

### Smoke Tests (E2E)
- Model status reporting
- Enhanced detection (person names without titles)
- Fallback when model disabled
- Performance benchmark (P50, P95)
- Backward compatibility (Phase 2 clients)

---

## Performance Targets

### Baseline (Phase 2 - Regex Only)
- P50: 16ms
- P95: 22ms
- P99: 23ms

### Target (Phase 2.2 - With Model)
- P50: ≤ 700ms (acceptable: 16ms → 700ms, ~44x increase)
- P95: ≤ 1000ms
- P99: ≤ 2000ms (unchanged from Phase 2 target)

**Rationale:** Accuracy improvement worth modest latency increase for use cases where recall is critical.

**Configuration:** Users can disable model (GUARD_MODEL_ENABLED=false) to get Phase 2 performance.

---

## Configuration

### Environment Variables (New)
```bash
# Enable model-enhanced detection (default: false for backward compat)
GUARD_MODEL_ENABLED=true

# Ollama service URL (default: docker compose service name)
OLLAMA_URL=http://ollama:11434

# Model to use for NER (default: llama3.2:1b per ADR-0015)
OLLAMA_MODEL=llama3.2:1b  # Options: llama3.2:1b, llama3.2:3b, tinyllama:1.1b
```

### Supported Models (ADR-0015)
- **llama3.2:1b** (recommended) - CPU-friendly, ~1GB, good NER
- **llama3.2:3b** - Better accuracy, more resources, ~3GB
- **tinyllama:1.1b** - Smallest, lowest accuracy, ~637MB

### Backward Compatibility
- Default: GUARD_MODEL_ENABLED=false (preserves Phase 2 behavior)
- Users opt-in to model-enhanced detection
- No breaking changes to API

---

## Timeline

**Total Effort:** 7-11 hours (Small - within 2 days)

**Day 1 (Morning):**
- A1: Ollama Client (2 hours)
- A2: Hybrid Detection (3 hours)

**Day 1 (Afternoon):**
- A3: Configuration & Fallback (2 hours)

**Day 2 (Morning):**
- B1: Update Config Guide (1 hour)
- B2: Update Integration Guide (1 hour)

**Day 2 (Afternoon):**
- C1: Accuracy Validation (2 hours)
- C2: Smoke Tests (1 hour)
- Final review and completion summary

---

## Acceptance Criteria

**Code:**
- [ ] Ollama client implemented and tested
- [ ] Hybrid detection logic working
- [ ] Graceful fallback tested
- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Code review clean (no TODOs, proper error handling)

**Performance:**
- [ ] P50 ≤ 700ms with model enabled
- [ ] P95 ≤ 1000ms with model enabled
- [ ] Regex-only fallback performance unchanged (P50 ~16ms)

**Accuracy:**
- [ ] Detection improvement ≥ 10% (measured on Phase 2 fixtures)
- [ ] False positive rate < 5% (unchanged)

**Documentation:**
- [ ] Configuration guide updated
- [ ] Integration guide updated
- [ ] Model selection explained (ADR-0015)
- [ ] Performance trade-offs documented

**Testing:**
- [ ] Accuracy test scripts created and run
- [ ] Smoke tests pass (5/5)
- [ ] Results documented in progress log

**Compatibility:**
- [ ] No breaking changes to API
- [ ] Phase 2 clients work without changes
- [ ] Default behavior preserves Phase 2 (model disabled by default)

---

## References

### Master Plan
- `Technical Project Plan/master-technical-project-plan.md` (Phase 2.2 definition)

### Phase 2 (Baseline)
- `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`
- `src/privacy-guard/` (existing code)

### ADRs
- ADR-0002: Privacy Guard Placement (local-first requirement)
- ADR-0015: Guard Model Policy and Selection (model choice guidance)
- ADR-0021: Privacy Guard Rust Implementation (Phase 2 baseline)
- ADR-0022: PII Detection Rules and FPE (detection strategy)

### Guides
- `docs/guides/guard-model-selection.md`
- `docs/guides/privacy-guard-config.md`
- `docs/guides/privacy-guard-integration.md`

### Test Data
- `tests/fixtures/pii_samples.txt` (150+ PII samples)
- `tests/fixtures/clean_samples.txt` (clean samples for FP testing)

---

## Notes

- Phase 2.2 is intentionally small (S effort) to keep momentum
- Builds directly on Phase 2 success (90.1MB image, 145+ tests)
- Preserves all Phase 2 guarantees (HTTP-only, metadata-only, local-first)
- Optional enhancement (users opt-in via config)
- Enables future Phase 3 agent-side integration

---

**Version:** 1.0  
**Date:** 2025-11-04  
**Status:** Ready for Execution
