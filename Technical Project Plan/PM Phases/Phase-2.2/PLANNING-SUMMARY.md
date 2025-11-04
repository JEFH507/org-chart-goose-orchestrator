# Phase 2.2 Planning Summary

**Date:** 2025-11-04  
**Status:** ✅ Planning Complete — Ready for Execution  
**Commit:** 7c5874e

---

## What Was Created

### Planning Documents (7 files, ~2,756 lines)

1. **Phase-2.2-Agent-Prompts.md** (~500 lines)
   - Complete orchestrator guide (modeled after Phase 2)
   - Master orchestrator prompt for new sessions
   - Resume prompt for continuing work
   - 7 detailed sub-prompts (A1-A3, B1-B2, C1-C2)
   - State persistence protocol
   - Git workflow guidelines
   - Guardrails and constraints

2. **Phase-2.2-Execution-Plan.md** (~350 lines)
   - Objectives and success criteria
   - Workstream breakdown (A: Model, B: Docs, C: Testing)
   - Architecture diagrams (before/after)
   - Timeline (7-11 hours, ≤ 2 days)
   - Risk analysis
   - Acceptance criteria

3. **Phase-2.2-Agent-State.json** (initial state)
   - Phase metadata
   - Workstreams and branches
   - User inputs (defaults from Phase 2)
   - Checklist structure (7 tasks)
   - Performance baselines from Phase 2
   - Empty results fields (to be filled during execution)

4. **Phase-2.2-Checklist.md**
   - 7 major tasks with sub-items
   - Workstream A: 3 tasks (Ollama client, hybrid detection, config)
   - Workstream B: 2 tasks (update guides)
   - Workstream C: 2 tasks (accuracy, smoke tests)
   - Completion tracking (0/7 = 0%)

5. **Phase-2.2-Assumptions-and-Open-Questions.md**
   - 13 technical/operational assumptions
   - 10 open questions (4 pre-execution, 3 during, 3 post)
   - Decision log (5 pre-execution decisions made)
   - Risk analysis (5 risks with mitigations)
   - User input checklist

6. **README.md** (directory guide)
   - Quick overview
   - Document index
   - How to start
   - Workstream summary
   - Quick start command

7. **docs/tests/phase2.2-progress.md** (progress log)
   - Initial planning entry
   - Resume instructions
   - Template for future entries

---

## Analysis Performed

### Phase 2 Review
- ✅ Reviewed completion summary (100% done)
- ✅ Analyzed deliverables (145+ tests, 90.1MB image, P50=16ms)
- ✅ Studied orchestrator model (Phase-2-Agent-Prompts.md)
- ✅ Reviewed deviations log (lessons learned)
- ✅ Validated tracking protocol (state JSON, progress log, checklist)

### Requirements Analysis
- ✅ Master plan Phase 2.2 definition reviewed
- ✅ ADR-0015 analyzed (model selection: llama3.2:1b)
- ✅ ADR-0002 analyzed (local-first requirement)
- ✅ Ollama CE defaults confirmed (from Phase 0)
- ✅ Product goals reviewed (privacy-first, org-aware)

### Design Decisions
- ✅ Architecture: Hybrid detection (regex + NER model)
- ✅ Integration: Ollama HTTP client (async)
- ✅ Fallback: Graceful degradation to regex-only
- ✅ Configuration: Environment-based, opt-in
- ✅ Compatibility: No breaking changes (backward compatible)
- ✅ Performance: 200ms increase acceptable for accuracy gain

---

## Phase 2.2 Scope

### Objectives
1. Add local NER model (Ollama) to improve detection accuracy
2. Preserve all Phase 2 functionality (HTTP API, modes, strategies)
3. Maintain local-only posture (no cloud exposure)
4. Keep backward compatible (no breaking changes)
5. Small effort (≤ 2 days)

### Workstreams
- **A: Model Integration** (4-6h) - Ollama client, hybrid detection, config
- **B: Documentation** (1-2h) - Update guides
- **C: Testing** (2-3h) - Accuracy validation, smoke tests

### Success Criteria
- **Accuracy:** +10-20% detection improvement
- **Performance:** P50 ≤ 700ms with model (vs 16ms baseline)
- **Quality:** All tests pass, no breaking changes
- **Documentation:** Guides updated, smoke tests complete

---

## Key Design Choices

### 1. Hybrid Detection (Regex + Model)
**Why:** Best of both worlds
- Regex: Fast, high precision, structured patterns
- Model: Better recall, unstructured text (person names)
- Consensus: Both methods agree → HIGH confidence

**Alternative Considered:** Model-only detection
**Rejected:** Slower, may miss structured patterns (SSN, phone)

### 2. Ollama HTTP Client
**Why:** Simplest integration
- Ollama already in CE defaults (Phase 0)
- HTTP API well-documented
- Async calls with reqwest (already in use)
- No new infrastructure needed

**Alternative Considered:** Embedded model library
**Rejected:** Complex integration, GPU dependencies

### 3. Model Disabled by Default
**Why:** Backward compatibility
- Users opt-in to enhanced detection
- No performance regression for existing users
- Clear migration path

**Alternative Considered:** Enabled by default
**Rejected:** Unexpected latency increase

### 4. Graceful Fallback
**Why:** Reliability
- Service continues if Ollama unavailable
- Regex-only is proven (Phase 2)
- No downtime risk

**Alternative Considered:** Fail-closed (error if model unavailable)
**Rejected:** Violates availability requirements

### 5. Target P50 ≤ 700ms
**Why:** Acceptable trade-off
- 10-20% accuracy improvement worth latency increase
- Users can disable model if latency critical
- Still well under 1-second SLA

**Alternative Considered:** Maintain P50 ≤ 500ms
**Rejected:** Unrealistic with model inference

---

## Implementation Strategy

### Phase 2.2 Builds On Phase 2
- ✅ Existing Rust codebase (`src/privacy-guard/`)
- ✅ Proven HTTP API (5 endpoints)
- ✅ Test fixtures (150+ PII samples, clean samples)
- ✅ Docker deployment (90.1MB image)
- ✅ Compose integration (privacy-guard service)
- ✅ Performance baseline (P50=16ms)

### Minimal Changes Required
- **New:** ~300 lines of Rust code (Ollama client, hybrid detection)
- **Modified:** ~50 lines (detection.rs, main.rs, compose config)
- **Documentation:** ~120 lines added to guides
- **Tests:** ~200 lines (accuracy tests, smoke tests)

**Total New Code:** ~670 lines (vs Phase 2: ~5,000+ lines)

**Scope Ratio:** Phase 2.2 is ~13% of Phase 2 effort ✅ (aligns with "Small" classification)

---

## Progressive Development Approach

### Phase Progression
```
Phase 0: Infrastructure Setup (S)
  └─ Ollama in CE defaults ✅

Phase 1: Controller Baseline (M)
  └─ HTTP API, healthchecks ✅

Phase 1.2: Identity & Security (S)
  └─ JWT, Vault wiring ✅

Phase 2: Privacy Guard Baseline (M)
  └─ Regex detection, pseudonymization, FPE, HTTP API ✅
       Performance: P50=16ms, 145+ tests

Phase 2.2: Privacy Guard Enhancement (S) ← WE ARE HERE
  └─ Add NER model, hybrid detection
       Expected: +10-20% accuracy, P50 ≤ 700ms

Phase 3: Controller API + Agent Mesh (L) ← NEXT
  └─ OpenAPI, MCP verbs, cross-agent tasks
```

### Leveraging Prior Work
- **Phase 0:** Ollama container already configured ✅
- **Phase 1:** HTTP-only architecture pattern established ✅
- **Phase 1.2:** JWT auth pattern (for /reidentify endpoint) ✅
- **Phase 2:** Complete Privacy Guard service to enhance ✅

**No Rework:** Phase 2.2 purely additive, no refactoring needed

---

## Risk Mitigation

### Top Risks Identified

1. **Model Latency Too High (Medium/Medium)**
   - Mitigation: Use smallest model (llama3.2:1b), make optional, document fallback
   - Contingency: Keep regex-only as default

2. **Accuracy Not Improved (Low/Medium)**
   - Mitigation: Test on Phase 2 fixtures, accept +10% threshold
   - Contingency: Document as experimental, still deliver for completion

3. **Ollama Unavailability (Low/Low)**
   - Mitigation: Graceful fallback, health check, clear logging
   - Contingency: Service continues with regex-only

**All Risks:** Low-to-Medium probability, all have clear mitigations

---

## Next Steps

### For User (Pre-Execution)

1. **Review Planning Documents:**
   - Read: `Technical Project Plan/PM Phases/Phase-2.2/README.md` (start here)
   - Read: `Phase-2.2-Execution-Plan.md` (scope and workstreams)
   - Skim: `Phase-2.2-Agent-Prompts.md` (orchestrator guide)

2. **Confirm Assumptions:**
   - Review: `Phase-2.2-Assumptions-and-Open-Questions.md`
   - Answer pre-execution questions:
     - Model disabled by default? (Recommended: Yes)
     - P50 ≤ 700ms acceptable? (Recommended: Yes)
     - +10% accuracy sufficient? (Recommended: Yes)

3. **Approve to Start:**
   - If approved: Copy "Master Orchestrator Prompt" from Agent-Prompts.md
   - Paste into new Goose session to begin execution
   - Orchestrator will create branches, implement code, run tests

### For Orchestrator (Execution)

1. **Confirm user inputs** (from assumptions document)
2. **Create feature branch** `feat/phase2.2-ollama-detection`
3. **Execute Workstream A** (tasks A1-A3)
4. **Switch to docs branch** for Workstream B
5. **Execute Workstream C** (testing)
6. **Write completion summary**
7. **Prepare PRs**

---

## Expected Outcomes

### By End of Phase 2.2

**Code:**
- ✅ Ollama HTTP client integrated (~150 lines)
- ✅ Hybrid detection logic (~150 lines)
- ✅ Configuration and fallback (~100 lines)
- ✅ All unit tests passing
- ✅ Integration tests passing

**Configuration:**
- ✅ New env vars: GUARD_MODEL_ENABLED, OLLAMA_URL, OLLAMA_MODEL
- ✅ Compose updated with ollama dependency
- ✅ Model disabled by default (backward compat)

**Documentation:**
- ✅ Configuration guide updated (+80 lines)
- ✅ Integration guide updated (+40 lines)
- ✅ Smoke test procedure created

**Testing:**
- ✅ Accuracy improvement measured (≥ +10%)
- ✅ Performance validated (P50 ≤ 700ms)
- ✅ Smoke tests pass (5/5)
- ✅ False positive rate < 5% maintained

**Deliverables:**
- ✅ Working enhanced guard service
- ✅ Backward compatible API
- ✅ Comprehensive documentation
- ✅ Validation tests and results
- ✅ Completion summary

---

## Comparison: Phase 2 vs Phase 2.2

| Aspect | Phase 2 (Baseline) | Phase 2.2 (Enhancement) |
|--------|-------------------|------------------------|
| **Effort** | Medium (3-5 days) | Small (≤ 2 days) |
| **Tasks** | 19 major tasks | 7 major tasks |
| **Code** | ~5,000+ lines | ~300 new lines |
| **Detection** | Regex-only | Hybrid (regex + NER) |
| **Performance** | P50=16ms | P50 ≤ 700ms |
| **Accuracy** | Baseline | +10-20% expected |
| **Model** | None | Ollama (llama3.2:1b) |
| **API Changes** | New service | None (backward compat) |
| **Workstreams** | 4 (A, B, C, D) | 3 (A, B, C) |
| **Scope** | Complete new service | Enhancement only |

**Phase 2.2 Efficiency:** ~13% of Phase 2 effort for meaningful enhancement ✅

---

## Documentation Quality

### Follows Phase 2 Model
- ✅ Comprehensive agent prompts (500+ lines like Phase 2)
- ✅ Detailed execution plan with workstreams
- ✅ State JSON schema for pause/resume
- ✅ Checklist for tracking progress
- ✅ Assumptions and risk analysis
- ✅ Progress log template

### Improvements Over Phase 2
- ✅ README.md for directory navigation (Phase 2 didn't have this)
- ✅ Clear references to Phase 2 baseline (builds on prior work)
- ✅ Pre-execution decision log (5 decisions documented)
- ✅ More explicit backward compatibility guidance

---

## Key Features of Phase 2.2 Orchestrator

### 1. Pause/Resume Capable
- State JSON tracks current position
- Progress log records all steps
- Resume prompt reconstructs context
- Validated protocol from Phase 2

### 2. Progressive Build
- Workstream A: Implement model integration
- Workstream B: Update documentation
- Workstream C: Validate accuracy and performance
- Each workstream builds on previous

### 3. Git-Aware
- Feature branches per workstream
- Conventional commits
- SSH-first with GNOME askpass
- PR preparation automated

### 4. Guardrails Enforced
- HTTP-only architecture
- Metadata-only storage
- No PII in logs
- Local-only model execution
- Backward compatibility

### 5. Testing-Driven
- Unit tests required for each task
- Integration tests for hybrid logic
- Accuracy measurement mandatory
- Smoke tests before completion

---

## Technical Highlights

### Architecture
```
Current (Phase 2):
  Client → Privacy Guard (Rust) → Regex Detection → Masking → Response

Enhanced (Phase 2.2):
  Client → Privacy Guard (Rust) → Hybrid Detection → Masking → Response
                                    ├─ Regex (fast)
                                    └─ Ollama NER (accurate)
                                         └─ http://ollama:11434 (local)
```

### Hybrid Detection Algorithm
1. Run regex detection (fast, high precision)
2. Call Ollama NER (slower, better recall)
3. Merge results:
   - Both methods detect same entity → HIGH confidence (consensus)
   - Model-only detection → HIGH confidence (model trusted)
   - Regex-only detection → Original confidence

### Graceful Fallback
- If `GUARD_MODEL_ENABLED=false` → regex-only (Phase 2 behavior)
- If Ollama unavailable → automatic fallback to regex-only
- No API errors, transparent to clients

---

## User Decisions Required

### Before Execution (Pre-A1)

1. **Model Default Behavior** (Question #1)
   - Recommended: `GUARD_MODEL_ENABLED=false` (backward compatible)
   - Alternative: `true` (enhanced by default)
   - **Your Decision:** __________

2. **Performance Acceptable?** (Question #2)
   - Accept: P50 ≤ 700ms with model (vs 16ms baseline)?
   - **Your Decision:** __________

3. **Accuracy Threshold** (Question #3)
   - Accept: +10% improvement as success?
   - **Your Decision:** __________

**Recommended Answers:** 
- #1: `false` (backward compatible)
- #2: Yes (200ms increase acceptable)
- #3: Yes (+10% sufficient)

These are already set as defaults in state JSON. Confirm or update before starting.

---

## How to Execute

### Quick Start (Recommended)

1. **Review this summary** (you're reading it now) ✅

2. **Confirm assumptions:**
   ```bash
   # Open and review:
   cat "Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Assumptions-and-Open-Questions.md"
   ```

3. **Start new Goose session:**
   - Copy "Master Orchestrator Prompt" from `Phase-2.2-Agent-Prompts.md`
   - Paste into new Goose session
   - Orchestrator will handle everything (branches, commits, tests, docs)

4. **Monitor progress:**
   ```bash
   # Check state
   cat "Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-State.json" | jq '.current_task_id, .status'
   
   # Check progress log
   tail -20 "docs/tests/phase2.2-progress.md"
   
   # Check checklist
   grep "Status:" "Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Checklist.md"
   ```

### Manual Start (Alternative)

1. Review all planning documents
2. Create branch: `git checkout -b feat/phase2.2-ollama-detection`
3. Follow execution plan task-by-task
4. Update tracking documents after each task

---

## Expected Timeline

### Day 1 (5-6 hours)
- **Morning (3h):** A1 (Ollama client) + start A2
- **Afternoon (2-3h):** Finish A2 (hybrid detection) + A3 (config)
- **EOD:** Workstream A complete, code compiling

### Day 2 (2-5 hours)
- **Morning (2h):** B1 + B2 (update guides) + C1 (accuracy tests)
- **Afternoon (1-3h):** C2 (smoke tests), completion summary, PR prep
- **EOD:** Phase 2.2 complete, ready for merge

**Total:** 7-11 hours (within Small = ≤ 2 days) ✅

---

## Success Indicators

**You'll know Phase 2.2 succeeded when:**
- ✅ All 7 checklist items marked "done"
- ✅ Accuracy tests show ≥ +10% improvement
- ✅ Smoke tests pass (5/5)
- ✅ Performance within target (P50 ≤ 700ms)
- ✅ No breaking changes (Phase 2 clients work unchanged)
- ✅ Documentation updated
- ✅ Completion summary written
- ✅ PRs ready for review

---

## Files Created (Summary)

```
Technical Project Plan/PM Phases/Phase-2.2/
├── README.md                                    [Directory guide]
├── Phase-2.2-Agent-Prompts.md                   [Orchestrator master prompt]
├── Phase-2.2-Execution-Plan.md                  [Workstreams and tasks]
├── Phase-2.2-Agent-State.json                   [State tracking]
├── Phase-2.2-Checklist.md                       [Progress checklist]
├── Phase-2.2-Assumptions-and-Open-Questions.md  [Decisions and risks]
└── PLANNING-SUMMARY.md                          [This file]

docs/tests/
└── phase2.2-progress.md                         [Progress log]
```

**Total:** 8 files, ~2,900 lines

---

## Commit

```
commit 7c5874e
Author: Javier
Date:   2025-11-04

    docs(phase2.2): create Phase 2.2 planning documents
    
    Phase 2.2 — Privacy Guard Enhancement (Small effort: ≤ 2 days)
    
    [Full commit message details in git log]
```

**Status:** ✅ Committed to main branch

---

## What's Next

### Immediate Next Steps

1. **User Review** (you)
   - Review this summary ✅
   - Review assumptions document
   - Confirm or adjust decisions
   - Approve to proceed

2. **Execution Kickoff**
   - Copy Master Orchestrator Prompt
   - Start new Goose session
   - Begin Workstream A

3. **Completion**
   - Execute all 7 tasks
   - Validate results
   - Write completion summary
   - Prepare PRs

### After Phase 2.2

**Next Phase:** Phase 3 — Controller API + Agent Mesh (Large effort)
- Scope: OpenAPI expansion, MCP extension, cross-agent tasks
- Builds on: Phases 0, 1, 1.2, 2, 2.2
- Estimated: 1-2 weeks

---

## Summary

✅ **Phase 2.2 planning complete and ready for execution**

**Created:**
- 7 planning documents (~2,756 lines)
- Comprehensive orchestrator guide (modeled after Phase 2)
- Clear workstreams (3), tasks (7), timeline (7-11h)
- State tracking (JSON, progress log, checklist)
- Risk analysis and mitigations

**Analyzed:**
- Phase 2 completion (baseline understanding)
- Master plan requirements (Phase 2.2 definition)
- ADRs (model selection, local-first posture)
- Progressive build approach (leveraging prior phases)

**Designed:**
- Hybrid detection architecture (regex + NER)
- Ollama HTTP client integration
- Graceful fallback strategy
- Backward compatible implementation

**Ready For:**
- User review and confirmation
- Execution via orchestrator prompt
- 2-day implementation sprint
- Meaningful accuracy enhancement

---

**Planning Complete:** 2025-11-04  
**Status:** ✅ READY FOR EXECUTION  
**Next:** User review → execution kickoff → delivery
