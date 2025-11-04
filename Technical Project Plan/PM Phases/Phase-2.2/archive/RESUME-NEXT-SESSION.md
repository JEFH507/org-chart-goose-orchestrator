# Phase 2.2 — Resume Instructions for Next Session

**Phase:** 2.2 - Privacy Guard Enhancement  
**Status:** IN PROGRESS (82% complete)  
**Current Task:** C1 (Accuracy Validation Tests) - 🚧 BLOCKED  
**Date:** 2025-11-04  

---

## Quick Status

- ✅ **Workstream A (Model Integration): COMPLETE** (4/4 tasks)
- ✅ **Workstream B (Documentation): COMPLETE** (2/2 tasks)
- 🚧 **Workstream C (Testing & Validation): BLOCKED** (C1 90% complete)

**What's Done:**
- Ollama HTTP client implemented ✅
- Hybrid detection logic working ✅
- Configuration and fallback complete ✅
- Documentation comprehensive ✅
- Test infrastructure created ✅
- All 141 unit tests passing ✅

**What's Blocked:**
- Accuracy test execution (model inference timing out)
- Blocker: Ollama version incompatibility

---

## 🚨 CRITICAL: Start Next Session With This Question

**MANDATORY FIRST QUESTION TO USER:**

```
Phase 2.2 Task C1 is 90% complete but blocked by Ollama version compatibility.

We discovered that Ollama 0.3.14 (current version in ce.dev.yml) does NOT 
support qwen3:0.6b model. When we tried to pull it, we got:

  Error 412: "The model you are attempting to pull requires a newer 
  version of Ollama. Please download the latest version."

Your preference is qwen3:0.6b (523MB, Nov 2024, 40K context) over 
llama3.2:1b (Oct 2024) because it's more recent and lightweight.

Options to resolve:

1. **Upgrade Ollama** (RECOMMENDED)
   - Change: ollama/ollama:0.3.14 → ollama/ollama:0.5.1 (latest stable)
   - Location: deploy/compose/ce.dev.yml line 47
   - Benefit: Get your preferred qwen3:0.6b model
   - Risk: Minimal (good backward compatibility)

2. **Select Alternative Modern Model**
   - Criteria: <1GB, 2024 release, Ollama 0.3.14 compatible
   - Source: https://ollama.com/search
   - Note: Limited modern options for older Ollama version

Which do you prefer? I recommend Option 1 (Ollama upgrade).
```

**DO NOT PROCEED** until user answers this question.

---

## Files to Read FIRST (Before Asking Question)

Read these in order to understand the full context:

1. **`Technical Project Plan/PM Phases/Phase-2.2/C1-FINDINGS.md`**  
   → Complete analysis of all blockers, remediation plan, change trail

2. **`Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-State.json`**  
   → Check `pending_questions` section for detailed context
   → Check `notes` array for C1 findings summary

3. **`docs/tests/phase2.2-progress.md`**  
   → Latest entry (C1) has blocker details and remediation steps

4. **`tests/accuracy/TESTING-NOTES.md`**  
   → Implementation notes, model selection history, test results

5. **`Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Checklist.md`**  
   → C1 section shows what's done, what's blocked

---

## After User Responds

### If User Chooses: Option 1 (Upgrade Ollama)

**Execute these steps in order:**

1. **Upgrade Ollama version:**
   ```bash
   # Edit file: deploy/compose/ce.dev.yml line 47
   # Change: ollama/ollama:0.3.14
   # To: ollama/ollama:0.5.1  (or latest from https://hub.docker.com/r/ollama/ollama/tags)
   ```

2. **Pull new Ollama and qwen3:0.6b:**
   ```bash
   cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
   docker compose pull ollama
   docker compose up -d ollama
   docker exec ce_ollama ollama pull qwen3:0.6b
   docker exec ce_ollama ollama list  # Verify
   ```

3. **Update VERSION_PINS.md:**
   Add Ollama version upgrade note

4. **Commit Ollama upgrade:**
   ```bash
   git add deploy/compose/ce.dev.yml VERSION_PINS.md
   git commit -m "build(deps): upgrade Ollama to 0.5.1 for qwen3:0.6b support"
   ```

5. **Fix Ollama client timeout:**
   ```rust
   // File: src/privacy-guard/src/ollama_client.rs
   // Line: 17
   // Change: .timeout(Duration::from_secs(5))
   // To: .timeout(Duration::from_secs(30))
   ```

6. **Rebuild privacy-guard:**
   ```bash
   docker compose build privacy-guard
   docker compose up -d privacy-guard
   ```

7. **Run accuracy tests:**
   ```bash
   cd /home/papadoc/Gooseprojects/goose-org-twin
   ./tests/accuracy/compare_detection.sh
   ./tests/accuracy/test_false_positives.sh --model-enhanced
   ```

8. **Document results:**
   - Update Phase-2.2-Agent-State.json (performance_results section)
   - Mark C1 as "done" in checklist
   - Append to phase2.2-progress.md with test results

9. **Commit:**
   ```bash
   git add src/privacy-guard/src/ollama_client.rs "Technical Project Plan/" docs/tests/
   git commit -m "fix(guard): increase Ollama timeout to 30s for model inference

   test(guard): complete C1 accuracy validation - [X.X%] improvement
   
   Results:
   - Regex-only: 123 entities
   - Model-enhanced: [XXX] entities
   - Improvement: [X.X]%
   - FP rate: [X.X]%
   
   Refs: Phase 2.2 Task C1"
   ```

10. **Proceed to C2** (Smoke Tests)

---

### If User Chooses: Option 2 (Alternative Model)

**Execute these steps:**

1. **Research options:**
   - Visit https://ollama.com/search
   - Filter: size <1GB, release 2024, compatible with Ollama 0.3.14
   - Check model cards for NER capability
   - Present 2-3 options to user with specs

2. **Get user approval** for specific model selection

3. **Update configuration:**
   - deploy/compose/.env.ce.example (OLLAMA_MODEL default)
   - src/privacy-guard/src/ollama_client.rs (default in from_env if needed)

4. **Pull and test model:**
   ```bash
   docker exec ce_ollama ollama pull <selected-model>
   docker exec ce_ollama ollama list
   ```

5. **Continue with steps 5-10 from Option 1** (timeout fix, rebuild, test, document, commit)

---

## Key Files Modified (Change Trail)

**Committed:**
- `tests/accuracy/` (all 5 files) - Commit: 29aea13
- `deploy/compose/ce.dev.yml` (Ollama healthcheck) - Commit: 29aea13
- `Phase-2.2-Agent-State.json` (C1 status, findings) - Commit: b2be6ee
- `Phase-2.2-Checklist.md` (C1 blockers) - Commit: b2be6ee
- `phase2.2-progress.md` (C1 entry) - Commit: b2be6ee
- `C1-FINDINGS.md` (analysis doc) - Commit: b2be6ee

**Local Changes (NOT committed - .gooseignore):**
- `deploy/compose/.env.ce`:
  - GUARD_MODEL_ENABLED=true (for testing)
  - OLLAMA_MODEL=llama3.2:1b (TEMPORARY - revert to qwen3:0.6b after Ollama upgrade)

**Needs Modification Next Session:**
- `src/privacy-guard/src/ollama_client.rs` line 17 (timeout: 5s → 30s)
- `deploy/compose/ce.dev.yml` line 47 (Ollama version upgrade - if user chooses Option 1)
- `VERSION_PINS.md` (Ollama version documentation - if upgraded)

---

## Test Results Summary

### Regex-Only Baseline ✅
```
Entities detected: 123
Test samples: 106 (valid)
Success rate: 100%
Performance: P50 ~16ms (Phase 2 baseline confirmed)
```

### Model-Enhanced ❌ BLOCKED
```
Entities detected: 123 (same as regex - fallback behavior)
Actual improvement: 0.0% (NOT real - model timed out)
Reason: Ollama client 5s timeout too short
Status: Test infrastructure ready, model integration not working
```

**Expected After Fix:**
```
Model-enhanced: 135-160 entities
Improvement: 10-25%
FP rate: <5%
Performance: P50 500-700ms
```

---

## Critical Context for Next Session

**User Preferences (IMPORTANT):**
- ✅ qwen3:0.6b (523MB, Nov 2024, 40K context)
- ❌ llama3.2:1b (Oct 2024 - "old model", rejected)
- Requirement: Modern models from https://ollama.com/search ONLY
- No compromise on model recency

**Technical Requirements:**
- Local-only (no cloud)
- Lightweight (<1GB preferred, <600MB ideal)
- Good NER capability (PERSON, ORGANIZATION detection)
- CPU-friendly (AMD Ryzen 7 PRO 3700U, 8GB RAM)

**Blockers:**
1. Ollama 0.3.14 ← incompatible with qwen3:0.6b
2. Ollama client timeout 5s ← too short for inference

**Resolution Path:**
- Upgrade Ollama (recommended) OR alternative model
- Fix timeout (1-line code change)
- Rebuild and test (30 min)

---

## Quick Reference - File Locations

**Analysis:**
- C1-FINDINGS.md: `Technical Project Plan/PM Phases/Phase-2.2/C1-FINDINGS.md`

**Tracking:**
- State: `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Agent-State.json`
- Checklist: `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Checklist.md`
- Progress: `docs/tests/phase2.2-progress.md`

**Tests:**
- Scripts: `tests/accuracy/compare_detection.sh`, `tests/accuracy/test_false_positives.sh`
- Docs: `tests/accuracy/README.md`, `tests/accuracy/TESTING-NOTES.md`

**Code to Fix:**
- Timeout: `src/privacy-guard/src/ollama_client.rs` line 17
- Ollama version: `deploy/compose/ce.dev.yml` line 47

**Config:**
- Template: `deploy/compose/.env.ce.example`
- Local (not committed): `deploy/compose/.env.ce`

---

## Expected Timeline After Resolution

- **Resolve blocker:** 30-60 min (upgrade Ollama + fix timeout + rebuild)
- **Run tests:** 5-10 min (both accuracy scripts)
- **Document:** 15-20 min (update state, checklist, progress log)
- **Commit:** 5 min
- **Total:** ~1 hour to complete C1
- **Then:** Proceed to C2 (Smoke Tests) - 1 hour
- **Phase 2.2 completion:** ~2 hours total remaining

---

## Commit References

**Latest Commits:**
- `29aea13` - test(guard): add accuracy validation tests for Phase 2.2
- `b2be6ee` - chore: document C1 blockers and findings - comprehensive analysis
- `0f1939a` - docs(guard): update integration guide with Phase 2.2 model-enhanced detection
- `779b1fd` - docs(guard): add model-enhanced detection section to config guide

**Branch:**
- feat/phase2.2-ollama-detection (all work on this branch)

---

## Success Criteria (Remaining)

To complete Phase 2.2:

- [ ] Resolve Ollama/model compatibility (user decision)
- [ ] Fix Ollama client timeout (30s)
- [ ] Complete C1 test execution
  - [ ] Accuracy improvement ≥ 10%
  - [ ] FP rate < 5%
- [ ] Complete C2 smoke tests (5 tests)
- [ ] Write completion summary
- [ ] Update project docs (TODO, CHANGELOG)
- [ ] Create PR for review

**Estimated Remaining:** 2-3 hours

---

**Version:** 1.0  
**Created:** 2025-11-04  
**For:** Next Goose Session Resume  
**Start with:** Read this file → Read C1-FINDINGS.md → Ask user model question
