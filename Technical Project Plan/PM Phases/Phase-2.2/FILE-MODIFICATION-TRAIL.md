# Phase 2.2 — File Modification Trail

**Purpose:** Track all file changes across Phase 2.2 for easy reversal/tracking  
**Date:** 2025-11-04  
**Status:** Updated for C1 (90% complete - blocked)  

---

## Modified Files (Committed)

### 1. deploy/compose/ce.dev.yml
**Lines Changed:** 55 (healthcheck)  
**Commit:** 29aea13  
**Date:** 2025-11-04

**Change:**
```yaml
# OLD (line 55):
healthcheck:
  test: ["CMD-SHELL", "curl -fsS http://localhost:11434/api/tags || exit 1"]

# NEW (line 55):
healthcheck:
  test: ["CMD-SHELL", "ollama list || exit 1"]
```

**Reason:** Ollama image doesn't include curl; use ollama CLI instead  
**Impact:** Ollama container now healthy, privacy-guard can start  
**Status:** ✅ WORKING - No revert needed

**PENDING CHANGE (for next session):**
```yaml
# Line 47 - WILL CHANGE after user decision:
ollama:
  image: ollama/ollama:0.3.14  # ← TO BE UPGRADED to 0.5.1+ for qwen3:0.6b
```

---

### 2. src/privacy-guard/src/ollama_client.rs
**Lines Changed:** 17 (timeout)  
**Commit:** NOT YET COMMITTED  
**Date:** Will be modified next session

**Change Needed:**
```rust
// Line 17 (inside impl OllamaClient::new()):

// CURRENT:
.timeout(Duration::from_secs(5))

// CHANGE TO:
.timeout(Duration::from_secs(30))  // Or 60 depending on user preference
```

**Reason:** Model inference + loading requires >5s on CPU-only hardware  
**Impact:** Model calls will complete instead of timing out  
**Status:** ⚠️ NOT YET CHANGED - Required for C1 completion  
**Priority:** CRITICAL

**After Change:**
- Rebuild: `docker compose build privacy-guard`
- Restart: `docker compose up -d privacy-guard`
- Test: Run accuracy scripts

---

### 3. deploy/compose/.env.ce.example
**Lines Changed:** 12-14 (added model env vars)  
**Commit:** 3edeb40 (Task A3)  
**Date:** 2025-11-04

**Added:**
```bash
# Privacy Guard Model (Phase 2.2)
GUARD_MODEL_ENABLED=false # true|false - Enable NER model for improved accuracy (opt-in)
OLLAMA_URL=http://ollama:11434  # Ollama service URL (internal Docker network)
OLLAMA_MODEL=qwen3:0.6b   # NER model: qwen3:0.6b (recommended), llama3.2:1b, llama3.2:3b
```

**Status:** ✅ COMMITTED - No changes needed  
**Note:** Default values are correct

---

### 4. src/privacy-guard/src/main.rs
**Lines Changed:** Multiple (AppState, status endpoint, handlers)  
**Commits:** d67f953 (A2), 3edeb40 (A3)  
**Date:** 2025-11-04

**Changes:**
1. **AppState** (added ollama_client field)
2. **status_handler** (added model_enabled, model_name fields)
3. **scan_handler** (uses detect_hybrid)
4. **mask_handler** (uses detect_hybrid)

**Status:** ✅ COMMITTED - No changes needed

---

### 5. src/privacy-guard/src/detection.rs
**Lines Changed:** +160 lines (hybrid detection)  
**Commit:** d67f953 (A2)  
**Date:** 2025-11-04

**Added Functions:**
- `detect_hybrid()` - async hybrid detection
- `merge_detections()` - combines regex + model results
- `overlaps()` - overlap detection
- `map_ner_type()` - entity type mapping

**Tests Added:** 11 new tests  
**Status:** ✅ COMMITTED - No changes needed

---

### 6. docs/guides/privacy-guard-config.md
**Lines Changed:** +451 lines (Model-Enhanced Detection section)  
**Commit:** 779b1fd (B1)  
**Date:** 2025-11-04  
**Version:** 1.0 → 1.1

**Added Section:**
- Model-Enhanced Detection (Phase 2.2+)
- Configuration env vars
- Supported models table
- Hybrid detection explanation
- Performance characteristics
- Enablement guide
- Troubleshooting

**Status:** ✅ COMMITTED - No changes needed

---

### 7. docs/guides/privacy-guard-integration.md
**Lines Changed:** +49 lines, -6 lines (status endpoint, performance section)  
**Commit:** 0f1939a (B2)  
**Date:** 2025-11-04  
**Version:** 1.0 → 1.1

**Updated:**
- GET /status response (added model_enabled, model_name)
- Performance Considerations (added detection modes comparison)
- Latency targets (added model-enhanced expectations)

**Status:** ✅ COMMITTED - No changes needed

---

## Created Files (New Artifacts)

### 8. src/privacy-guard/src/ollama_client.rs
**Size:** ~290 lines  
**Commit:** a5391a1 (A1)  
**Date:** 2025-11-04

**Contents:**
- OllamaClient struct
- HTTP client for /api/generate
- NER prompt builder
- Response parser
- Health check
- 8 unit tests

**Status:** ✅ COMMITTED  
**Pending Change:** Line 17 timeout (see #2 above)

---

### 9. tests/accuracy/compare_detection.sh
**Size:** 5.5KB (executable)  
**Commit:** 29aea13 (C1)  
**Date:** 2025-11-04

**Purpose:** Compare regex vs model-enhanced detection accuracy

**Status:** ✅ COMMITTED - Script ready, awaiting model fix to execute

---

### 10. tests/accuracy/test_false_positives.sh
**Size:** 4.7KB (executable)  
**Commit:** 29aea13 (C1)  
**Date:** 2025-11-04

**Purpose:** Validate false positive rate on clean samples

**Status:** ✅ COMMITTED - Script ready, awaiting model fix to execute

---

### 11. tests/accuracy/README.md
**Size:** 8.2KB  
**Commit:** 29aea13 (C1)  
**Date:** 2025-11-04

**Contents:** Complete documentation for accuracy test suite

**Status:** ✅ COMMITTED

---

### 12. tests/accuracy/TESTING-NOTES.md
**Size:** ~5KB  
**Commit:** 29aea13 (C1)  
**Date:** 2025-11-04

**Contents:** Implementation notes, findings, model selection history

**Status:** ✅ COMMITTED

---

### 13. tests/accuracy/.gitignore
**Commit:** 29aea13 (C1)  
**Date:** 2025-11-04

**Purpose:** Exclude temporary test result files

**Status:** ✅ COMMITTED

---

### 14. Technical Project Plan/PM Phases/Phase-2.2/C1-FINDINGS.md
**Size:** ~17KB  
**Commit:** b2be6ee  
**Date:** 2025-11-04

**Contents:**
- Complete analysis of 4 critical findings
- Remediation plan with detailed steps
- Test results (partial)
- Change trail
- Next session checklist

**Status:** ✅ COMMITTED

---

### 15. Technical Project Plan/PM Phases/Phase-2.2/RESUME-NEXT-SESSION.md
**Size:** ~5KB  
**Commit:** b6f393c  
**Date:** 2025-11-04

**Contents:** Quick start guide for next session with mandatory first question

**Status:** ✅ COMMITTED

---

## Modified Files (Local Only - NOT Committed)

### 16. deploy/compose/.env.ce
**Status:** ⚠️ LOCAL CHANGES ONLY (.gooseignore)

**Current Contents (relevant lines):**
```bash
# Pseudonymization - Phase 1.2
PSEUDO_SALT=CHANGE_ME_DEV_ONLY

# Privacy Guard - Phase 2.2
GUARD_MODEL_ENABLED=true         # ← For testing (change from false)
OLLAMA_MODEL=llama3.2:1b         # ← TEMPORARY (should be qwen3:0.6b)
```

**Why Not Committed:**
- .env.ce is in .gooseignore (local environment only)
- Values are for testing only
- Production values come from .env.ce.example

**Action for Next Session:**
After Ollama upgrade, reset to:
```bash
GUARD_MODEL_ENABLED=false  # Default (opt-in)
OLLAMA_MODEL=qwen3:0.6b    # Revert from llama3.2:1b
```

---

## Files to Revert/Update (Next Session)

### If Ollama Upgraded to 0.5.x+

**File:** deploy/compose/ce.dev.yml  
**Line:** 47  
**Change:** `ollama/ollama:0.3.14` → `ollama/ollama:0.5.1`  
**Commit:** New commit for Ollama upgrade

**File:** VERSION_PINS.md  
**Section:** Ollama (Container)  
**Add:** Version upgrade note  
**Commit:** Same as above

**File:** deploy/compose/.env.ce (local)  
**Change:** `OLLAMA_MODEL=llama3.2:1b` → `OLLAMA_MODEL=qwen3:0.6b`  
**Commit:** Not committed (local only)

---

### If Alternative Model Selected

**Files to Update:**
1. deploy/compose/.env.ce.example (OLLAMA_MODEL default)
2. docs/guides/privacy-guard-config.md (model references)
3. docs/guides/privacy-guard-integration.md (examples)
4. ADR-0015 (if significant policy change)
5. VERSION_PINS.md (Guard Models section)

**Template for Model Change:**
```bash
# Old references to: qwen3:0.6b
# New references to: <selected-model>
```

---

## Revert Instructions (If Needed)

### Revert Ollama Healthcheck Change
```bash
git revert 29aea13  # Reverts healthcheck fix
# OR manually edit ce.dev.yml line 55 back to curl
```

### Revert All C1 Changes
```bash
git reset --hard 0f1939a  # Go back before C1
# Loses: Test scripts, C1 findings docs, tracking updates
```

### Revert Only Test Scripts
```bash
git rm -r tests/accuracy/
git commit -m "revert: remove C1 test scripts"
```

**⚠️ WARNING:** Only revert if fundamentally changing approach. Current work is 90% complete and correct.

---

## Summary of Changes by Phase

### Phase 2.2 - Workstream A (Model Integration)
- ✅ ollama_client.rs (new file)
- ✅ detection.rs (+160 lines hybrid detection)
- ✅ main.rs (AppState, handlers, status endpoint)
- ✅ .env.ce.example (model env vars)
- ✅ ce.dev.yml (privacy-guard service config)

### Phase 2.2 - Workstream B (Documentation)
- ✅ privacy-guard-config.md (+451 lines)
- ✅ privacy-guard-integration.md (+49 lines)

### Phase 2.2 - Workstream C (Testing - In Progress)
- ✅ tests/accuracy/ (5 files created)
- ✅ ce.dev.yml (Ollama healthcheck)
- ⏳ ollama_client.rs (timeout fix pending)
- ⏳ Test execution pending (model fix)

---

## Total Lines of Code/Docs Added

**Code:**
- ollama_client.rs: ~290 lines
- detection.rs: +160 lines
- main.rs: ~30 lines
- Test scripts: ~200 lines (shell)
- **Total Code: ~680 lines**

**Documentation:**
- privacy-guard-config.md: +451 lines
- privacy-guard-integration.md: +49 lines
- tests/accuracy/README.md: ~350 lines
- C1-FINDINGS.md: ~700 lines
- TESTING-NOTES.md: ~250 lines
- RESUME-NEXT-SESSION.md: ~200 lines
- **Total Docs: ~2,000 lines**

**Total: ~2,680 lines added in Phase 2.2 (excluding tracking docs)**

---

## Quick Grep Commands (Find Changes)

**Find model references:**
```bash
rg "qwen3:0.6b|llama3.2:1b" --type rust --type yaml --type md
```

**Find Ollama version references:**
```bash
rg "ollama/ollama:0.3.14" deploy/
```

**Find timeout references:**
```bash
rg "Duration::from_secs\(5\)" src/privacy-guard/
```

**Find GUARD_MODEL_ENABLED:**
```bash
rg "GUARD_MODEL_ENABLED" --type yaml --type rust
```

---

## Commit History Summary

```
b6f393c docs: add comprehensive resume instructions for next session
b2be6ee chore: document C1 blockers and findings - comprehensive analysis
29aea13 test(guard): add accuracy validation tests for Phase 2.2
a959685 chore: update Phase 2.2 tracking - Task B2 complete
0f1939a docs(guard): update integration guide with Phase 2.2
001cc78 chore: update Phase 2.2 tracking - Task B1 complete
779b1fd docs(guard): add model-enhanced detection section to config guide
7bdc38b chore: update Phase 2.2 tracking - Task A3 complete
3edeb40 feat(guard): add model configuration and status endpoint
91f2c4f chore: update Phase 2.2 tracking - Task A2 complete
d67f953 feat(guard): implement hybrid detection (regex + NER model)
f92536d fix: resolve remaining 5 test failures - 100% tests passing
5570a92 chore: update tracking - A0 investigation complete
ae8d605 fix(guard): add PSEUDO_SALT test default
426c7ed fix(guard): resolve Phase 2 regex and validation issues
...
```

**Total Phase 2.2 Commits:** 13 (including tracking updates)

---

## Files NOT Modified (Reference)

These files were considered but not changed:

- `src/privacy-guard/src/redaction.rs` - No changes needed (masking logic unchanged)
- `src/privacy-guard/src/pseudonym.rs` - No changes needed
- `src/privacy-guard/src/policy.rs` - No changes needed
- `src/privacy-guard/src/state.rs` - No changes needed
- `src/privacy-guard/src/audit.rs` - Only Phase 2 bug fixes (A0)
- `src/privacy-guard/Cargo.toml` - Only dependency move (A1)
- `deploy/compose/guard-config/rules.yaml` - No changes (Phase 2 patterns work)
- `deploy/compose/guard-config/policy.yaml` - No changes

---

## Reverting Specific Changes

### Revert Ollama Healthcheck Only
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
git diff 0f1939a..29aea13 ce.dev.yml | git apply --reverse
# OR manually edit line 55 back to curl
```

### Revert Model Integration (All of Workstream A)
```bash
git revert --no-commit d67f953  # A2
git revert --no-commit 3edeb40  # A3
git revert --no-commit a5391a1  # A1
git commit -m "revert: Phase 2.2 Workstream A (model integration)"
```

**⚠️ WARNING:** This is destructive. Only use if fundamentally changing approach.

---

## Next Session Change Checklist

After user decides on Ollama/model:

- [ ] Update `deploy/compose/ce.dev.yml` (Ollama version if upgrading)
- [ ] Update `VERSION_PINS.md` (Ollama version note if upgrading)
- [ ] Update `src/privacy-guard/src/ollama_client.rs` line 17 (timeout: 5s → 30s)
- [ ] Rebuild privacy-guard Docker image
- [ ] Pull correct model in Ollama (qwen3:0.6b or alternative)
- [ ] Update local `.env.ce` if model changed from qwen3:0.6b
- [ ] Run accuracy tests
- [ ] Document results in state JSON, checklist, progress log
- [ ] Commit all changes with test results

---

**Version:** 1.0  
**Created:** 2025-11-04  
**Last Updated:** 2025-11-04  
**Purpose:** Comprehensive change tracking for Phase 2.2
