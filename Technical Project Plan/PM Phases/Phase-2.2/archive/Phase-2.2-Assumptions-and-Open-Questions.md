# Phase 2.2 — Privacy Guard Enhancement — Assumptions and Open Questions

**Date:** 2025-11-04  
**Phase:** Phase 2.2 - Privacy Guard Enhancement  
**Status:** Pre-Execution

---

## Assumptions

### Technical Assumptions

1. **Ollama Service Availability**
   - Assumption: Ollama container is available in CE defaults (from Phase 0)
   - Validation: Confirmed in `deploy/compose/ce.dev.yml`
   - Risk: LOW - Ollama already deployed

2. **Model Performance**
   - Assumption: llama3.2:1b provides P50 ≤ 700ms on typical prompts (~1000 chars)
   - Validation: Will measure during testing
   - Risk: MEDIUM - If too slow, can switch to tinyllama:1.1b or disable by default

3. **Model Accuracy**
   - Assumption: llama3.2:1b improves NER detection by ≥ 10% over regex-only
   - Validation: Will measure during C1 (accuracy tests)
   - Risk: MEDIUM - If < 10%, document and keep as opt-in enhancement

4. **Ollama API Stability**
   - Assumption: Ollama `/api/generate` endpoint is stable and documented
   - Validation: API is standard and well-documented
   - Risk: LOW - Mature API, widely used

5. **Rust HTTP Client**
   - Assumption: `reqwest` crate (already in use) supports Ollama API needs
   - Validation: Confirmed - same pattern as controller guard client
   - Risk: LOW - Proven in Phase 2

6. **No Phase 2 Breaking Changes**
   - Assumption: Can enhance detection without changing external API
   - Validation: Hybrid detection is internal implementation detail
   - Risk: LOW - Design preserves API contract

### Operational Assumptions

7. **User Opt-In**
   - Assumption: Model disabled by default (backward compatible)
   - Validation: `GUARD_MODEL_ENABLED=false` by default
   - Risk: NONE - Conservative approach

8. **Docker Compose Environment**
   - Assumption: Users run via `docker compose` with privacy-guard profile
   - Validation: Standard deployment from Phase 2
   - Risk: LOW - Established pattern

9. **Local-Only Execution**
   - Assumption: Ollama runs in local container, no cloud calls
   - Validation: ADR-0002 requirement, enforced by docker network
   - Risk: NONE - Design constraint

10. **Phase 2 Test Fixtures Sufficient**
    - Assumption: Existing `pii_samples.txt` and `clean_samples.txt` adequate for accuracy testing
    - Validation: 150+ PII samples across 8 entity types
    - Risk: LOW - Comprehensive test data

### Resource Assumptions

11. **CPU-Only Inference**
    - Assumption: llama3.2:1b runs acceptably on CPU (no GPU required)
    - Validation: ADR-0015 explicitly chose CPU-friendly models
    - Risk: LOW - Model designed for CPU inference

12. **Memory Footprint**
    - Assumption: Ollama + llama3.2:1b fits in CE defaults RAM (~8GB total)
    - Validation: Model ~1GB, Ollama overhead ~500MB
    - Risk: LOW - Well within budget

13. **Disk Space**
    - Assumption: llama3.2:1b model (~1GB) acceptable for CE deployment
    - Validation: Standard for local models
    - Risk: NONE - Expected

---

## Open Questions

### Pre-Execution (Answer Before Starting)

1. **Default Model Behavior**
   - **Question:** Should GUARD_MODEL_ENABLED default to `true` or `false`?
   - **Options:**
     - `false` (backward compatible, users opt-in) ← **RECOMMENDED**
     - `true` (new feature by default, users opt-out)
   - **Impact:** User experience, performance, backward compatibility
   - **Decision Needed:** Before A3
   - **Current Assumption:** `false` for backward compatibility

2. **Performance Acceptable Range**
   - **Question:** Is P50 ≤ 700ms acceptable for model-enhanced mode?
   - **Options:**
     - Accept 200ms increase (16ms → 700ms)
     - Require tighter bound (e.g., P50 ≤ 500ms)
   - **Impact:** Model selection, user adoption
   - **Decision Needed:** Before C1 (accuracy tests)
   - **Current Assumption:** 700ms acceptable (10-20% accuracy gain worth it)

3. **Accuracy Threshold**
   - **Question:** Is +10% detection improvement sufficient to declare success?
   - **Options:**
     - Accept +10% as success
     - Require +15% or +20%
   - **Impact:** Model selection, feature value
   - **Decision Needed:** Before C1
   - **Current Assumption:** +10% sufficient

### During Execution (Resolve As Needed)

4. **Model Selection**
   - **Question:** If llama3.2:1b doesn't meet targets, which fallback?
   - **Options:**
     - llama3.2:3b (better accuracy, slower, more memory)
     - tinyllama:1.1b (faster, less accurate)
     - Keep llama3.2:1b as opt-in only
   - **Impact:** Performance, accuracy, resource usage
   - **Decision Needed:** After initial testing
   - **Current Assumption:** llama3.2:1b will work

5. **Prompt Engineering**
   - **Question:** Is initial NER prompt sufficient or needs tuning?
   - **Options:**
     - Use simple prompt (current design)
     - Iterate with few-shot examples
     - Add structured output format
   - **Impact:** Accuracy, maintenance complexity
   - **Decision Needed:** After A1 testing
   - **Current Assumption:** Simple prompt sufficient

6. **Consensus Logic**
   - **Question:** How to handle regex+model disagreement?
   - **Options:**
     - Take union (both detections)
     - Require consensus (intersection)
     - Model overrides regex (when conflicts)
   - **Impact:** Accuracy, false positives
   - **Decision Needed:** During A2
   - **Current Assumption:** Union with consensus boost to HIGH confidence

7. **Entity Type Mapping**
   - **Question:** How to map model's entity types to our EntityType enum?
   - **Options:**
     - Strict mapping (unmapped types ignored)
     - Map ORGANIZATION → PERSON
     - Add new entity types
   - **Impact:** Detection coverage
   - **Decision Needed:** During A2
   - **Current Assumption:** Map ORGANIZATION → PERSON, ignore LOCATION

### Post-Execution (Future Phases)

8. **Multiple Model Support**
   - **Question:** Should we support multiple models simultaneously?
   - **Impact:** Complexity, resource usage
   - **Defer To:** Phase 3+ (if user demand)

9. **Model Fine-Tuning**
   - **Question:** Should we support custom model training?
   - **Impact:** Scope, complexity, user experience
   - **Defer To:** Phase 4+ (if enterprise demand)

10. **GPU Optimization**
    - **Question:** Should we optimize for GPU if available?
    - **Impact:** Performance, complexity, deployment options
    - **Defer To:** Post-MVP (optional enhancement)

---

## Validation Plan

### How We'll Validate Assumptions

1. **A1 (Ollama Client):** Verify Ollama API works, parsing succeeds
2. **A2 (Hybrid Detection):** Measure consensus logic effectiveness
3. **A3 (Fallback):** Confirm graceful degradation works
4. **C1 (Accuracy):** Measure actual improvement % on Phase 2 fixtures
5. **C2 (Smoke Tests):** Validate performance targets (P50, P95, P99)

### Success Criteria
- Accuracy improvement ≥ 10% (validate assumption #3)
- P50 ≤ 700ms with model (validate assumption #2)
- FP rate < 5% unchanged (validate assumption #10)
- Graceful fallback works (validate assumption #6)
- No breaking changes (validate assumption #6)

---

## Risks

### High-Priority Risks

**R1: Model latency exceeds target**
- **Probability:** Medium (30%)
- **Impact:** Medium (users won't enable feature)
- **Mitigation:** Make opt-in, document trade-offs, provide smaller model option
- **Contingency:** Keep regex-only as default, document model as experimental

**R2: Accuracy improvement < 10%**
- **Probability:** Low (20%)
- **Impact:** Medium (feature value questioned)
- **Mitigation:** Test with multiple models, tune prompt, accept as opt-in enhancement
- **Contingency:** Document as "experimental", still deliver for Phase 2.2 completion

**R3: Ollama unavailability**
- **Probability:** Low (10%)
- **Impact:** Low (graceful fallback)
- **Mitigation:** Health check on startup, fail-open design, clear logging
- **Contingency:** Service continues with regex-only (no downtime)

### Low-Priority Risks

**R4: Memory pressure from model**
- **Probability:** Low (15%)
- **Impact:** Low (performance degradation)
- **Mitigation:** Use smallest model (llama3.2:1b), document requirements
- **Contingency:** Recommend disabling model or using tinyllama

**R5: NER prompt needs tuning**
- **Probability:** Medium (40%)
- **Impact:** Low (can iterate)
- **Mitigation:** Simple initial prompt, measure and iterate
- **Contingency:** Accept iteration as part of small scope

---

## Decision Log

### Decisions Made (Pre-Execution)

| Date | Decision | Rationale | Impact |
|------|----------|-----------|--------|
| 2025-11-04 | Use llama3.2:1b default | ADR-0015 recommendation, CPU-friendly | Model selection |
| 2025-11-04 | Model disabled by default | Backward compatibility | User experience |
| 2025-11-04 | P50 ≤ 700ms acceptable | Accuracy gain worth latency increase | Performance targets |
| 2025-11-04 | +10% accuracy sufficient | Realistic improvement threshold | Success criteria |
| 2025-11-04 | Fail-open on model error | Service availability priority | Reliability |

### Decisions Deferred

| Question | Target Resolution | Owner |
|----------|-------------------|-------|
| Prompt tuning approach | During A1 testing | Orchestrator |
| Consensus logic details | During A2 implementation | Orchestrator |
| Entity type mappings | During A2 implementation | Orchestrator |
| Model fallback strategy | After initial testing | User + Orchestrator |

---

## User Input Checklist

Before starting Phase 2.2, confirm:

- [ ] OS: Linux (assumed from Phase 2) ✅
- [ ] Docker available (assumed from Phase 2) ✅
- [ ] Ollama model preference: llama3.2:1b (default) or other?
- [ ] Enable model by default: No (backward compat) or Yes?
- [ ] Performance target acceptable: P50 ≤ 700ms? ✅ (assumed acceptable)
- [ ] Accuracy target acceptable: +10% improvement? ✅ (assumed sufficient)
- [ ] Git identity: Javier / 132608441+JEFH507@users.noreply.github.com ✅
- [ ] GitHub remote: git@github.com:JEFH507/org-chart-goose-orchestrator.git ✅

---

## References

- **Master Plan:** `Technical Project Plan/master-technical-project-plan.md`
- **Phase 2 Baseline:** `Technical Project Plan/PM Phases/Phase-2/Phase-2-Completion-Summary.md`
- **ADR-0015:** `docs/adr/0015-guard-model-policy-and-selection.md` (model selection)
- **ADR-0002:** `docs/adr/0002-privacy-guard-placement.md` (local-first requirement)

---

**Status:** Ready for user confirmation  
**Next:** Answer open questions, confirm assumptions, begin execution
