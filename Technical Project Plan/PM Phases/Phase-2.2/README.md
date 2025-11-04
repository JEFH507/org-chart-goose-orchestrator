# Phase 2.2 — Privacy Guard Enhancement

**Status:** 📋 Ready to Execute  
**Effort:** Small (S = ≤ 2 days)  
**Date Created:** 2025-11-04

---

## Overview

Phase 2.2 enhances the Privacy Guard service (delivered in Phase 2) with local NER model support via Ollama to improve PII detection accuracy while preserving all existing functionality.

**Key Enhancement:**
- Hybrid detection combining regex (fast, high precision) + NER model (better recall)
- Local-only execution (Ollama container, no cloud)
- Graceful fallback to regex-only if model unavailable
- Backward compatible (no API changes)

**Builds On:**
- Phase 2: Privacy Guard baseline (regex, pseudonymization, FPE, HTTP API)
- Phase 0: Ollama service in CE defaults

---

## Objectives

1. **Improve Detection Accuracy** - Add local NER model for better person/org name detection
2. **Maintain Local-First Posture** - Use Ollama (local container only, no cloud)
3. **Preserve Functionality** - All Phase 2 features work unchanged
4. **Backward Compatible** - No breaking changes, model disabled by default
5. **Small Scope** - ≤ 2 days effort, minimal complexity

---

## Key Documents

### For Execution
- **START HERE:** `Phase-2.2-Agent-Prompts.md` - Complete orchestrator guide
- **Execution Plan:** `Phase-2.2-Execution-Plan.md` - Workstreams and tasks
- **Checklist:** `Phase-2.2-Checklist.md` - Track progress
- **State:** `Phase-2.2-Agent-State.json` - Current state (update after each task)
- **Progress Log:** `docs/tests/phase2.2-progress.md` - Chronological log

### For Context
- **Assumptions:** `Phase-2.2-Assumptions-and-Open-Questions.md` - Pre-execution decisions
- **Phase 2 Baseline:** `../Phase-2/Phase-2-Completion-Summary.md` - What we're building on
- **ADR-0015:** `docs/adr/0015-guard-model-policy-and-selection.md` - Model selection guidance
- **ADR-0002:** `docs/adr/0002-privacy-guard-placement.md` - Local-first requirement

---

## How to Start

### Option 1: Use Master Orchestrator Prompt (Recommended)

Copy and paste the "Master Orchestrator Prompt" from `Phase-2.2-Agent-Prompts.md` into a new Goose session.

### Option 2: Manual Execution

1. Review all documents in this directory
2. Confirm user inputs in assumptions document
3. Create feature branch: `feat/phase2.2-ollama-detection`
4. Follow execution plan task-by-task
5. Update state JSON and progress log after each task

---

## Workstreams

### Workstream A: Model Integration (4-6 hours)
**Branch:** `feat/phase2.2-ollama-detection`

- **A1:** Ollama HTTP Client (1-2h)
- **A2:** Hybrid Detection Logic (2-3h)
- **A3:** Configuration & Fallback (1-2h)

**Deliverables:** Updated Rust code, unit tests, compose config

---

### Workstream B: Documentation (1-2 hours)
**Branch:** `docs/phase2.2-guides`

- **B1:** Update Configuration Guide (30-60min)
- **B2:** Update Integration Guide (30-60min)

**Deliverables:** Updated guides with model configuration

---

### Workstream C: Testing & Validation (2-3 hours)
**Branch:** `test/phase2.2-validation`

- **C1:** Accuracy Validation Tests (1-2h)
- **C2:** Smoke Tests (1h)

**Deliverables:** Accuracy scripts, smoke tests, results

---

## Success Metrics

**Accuracy:**
- ✅ Detection improvement: ≥ +10% (measured on Phase 2 fixtures)
- ✅ False positive rate: < 5% (unchanged)

**Performance:**
- ✅ P50 ≤ 700ms with model enabled
- ✅ P95 ≤ 1000ms with model enabled
- ✅ Regex-only fallback: P50 ~16ms (unchanged)

**Quality:**
- ✅ All unit tests pass
- ✅ Integration tests pass
- ✅ Smoke tests pass (5/5)
- ✅ No breaking changes to API

**Deliverables:**
- ✅ Ollama client code (~150 lines)
- ✅ Hybrid detection logic (~150 lines)
- ✅ Updated documentation (~120 lines)
- ✅ Accuracy tests
- ✅ Smoke tests
- ✅ Completion summary

---

## Timeline

**Estimated Total:** 7-11 hours (within 2 days)

**Day 1:**
- Morning: A1 + A2 (Ollama client, hybrid detection)
- Afternoon: A3 (config, fallback)

**Day 2:**
- Morning: B1 + B2 (docs), C1 (accuracy tests)
- Afternoon: C2 (smoke tests), completion summary

---

## Architecture Change

### Before (Phase 2)
```
Privacy Guard:
  ├─ Regex Detection → Masking
  └─ HTTP API (5 endpoints)
```

### After (Phase 2.2)
```
Privacy Guard:
  ├─ Hybrid Detection:
  │    ├─ Regex (baseline)
  │    └─ Ollama NER (optional)
  │         └─ http://ollama:11434
  ├─ Merge Results (consensus logic)
  ├─ Masking (unchanged)
  └─ HTTP API (5 endpoints, unchanged)
```

**Impact:** Internal implementation detail, external API unchanged

---

## References

### Current Phase
- All documents in this directory (`Phase-2.2/`)

### Prior Phases
- Phase 2: `../Phase-2/` (baseline implementation)
- Phase 1.2: `../Phase-1.2/` (JWT, Vault)
- Phase 1: `../Phase-1/` (controller)
- Phase 0: `../Phase-0/` (infra)

### ADRs
- ADR-0015: Guard Model Policy and Selection
- ADR-0021: Privacy Guard Rust Implementation
- ADR-0022: PII Detection Rules and FPE
- ADR-0002: Privacy Guard Placement

### Code
- Privacy Guard: `src/privacy-guard/`
- Test Fixtures: `tests/fixtures/`

---

## Quick Start Command

```bash
# Copy this into new Goose session to start Phase 2.2
cat "Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-Prompts.md"

# Then paste the "Master Orchestrator Prompt" section
```

---

**Created:** 2025-11-04  
**Status:** Ready for execution  
**Next:** Review with user, confirm assumptions, begin Workstream A
