# Phase 6 Orchestrator Prompt

**Version:** 3.1 (Simplified Structure)  
**Approach:** Privacy Guard Proxy + Profile Setup Scripts  
**Timeline:** 14 days (3 weeks calendar)  
**Target:** v0.6.0 Production-Ready MVP

---

# ═══════════════════════════════════════════════════════════════════════
# SECTION 1: INITIAL SESSION PROMPT  
# 
# USE THIS ONCE - At the very start of Phase 6
# Copy everything between START and END markers below
# 
# ═══════════════════════════════════════════════════════════════════════

```
─────────────────────────────────────────────────────────────────────
 START OF INITIAL PROMPT - COPY FROM HERE ↓
─────────────────────────────────────────────────────────────────────

# Phase 6: Production Hardening + Admin UI + Privacy Proxy

## Mission
Execute Phase 6 to deliver production-ready v0.6.0 MVP with:
1. ✅ Users sign in → Profiles auto-load → Chat with PII protection
2. ✅ Admin UI (profile management, org chart, audit logs)
3. ✅ Vault production-ready (TLS, AppRole, Raft, audit)
4. ✅ Privacy Guard Proxy (intercepts LLM requests, masks PII)
5. ✅ Security hardened (no secrets, .env.example, SECURITY.md)
6. ✅ 92/92 integration tests passing

## Architecture Approach
**Decision:** Privacy Guard Proxy + Profile Setup Scripts (validated ✅)

**Why:** Follows proven Phases 1-5 service pattern
- Add new service: privacy-guard-proxy (port 8090)
- Add automation scripts: setup-profile.sh
- No goose Desktop fork needed
- 14 days timeline (faster than fork approach)

See: docs/architecture/SRC-ARCHITECTURE-AUDIT.md (proof that service pattern works)

## Working Directory
/home/papadoc/Gooseprojects/goose-org-twin

## Key Documents

**Primary Guide (Your Main Reference):**
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md ← Follow this!

**Progress Tracking (Update After EVERY Workstream):**
- docs/tests/phase6-progress.md ← Timestamped entries
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json ← Current state
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md ← Mark tasks ✅

**Architecture Reference (Read Only When Confused):**
- docs/architecture/SRC-ARCHITECTURE-AUDIT.md ← Understand /src structure
- docs/architecture/PHASE5-ARCHITECTURE.md ← Current system overview

**Code Reference (Grep These for Patterns):**
- src/controller/ ← Axum routing patterns
- src/privacy-guard/ ← Privacy Guard API reference
- src/vault/ ← Vault client (upgrade in Workstream A)
- src/lifecycle/ ← Session lifecycle (wire in Workstream E)

## 🚨 CRITICAL: First Action is USER's Responsibility (Not Yours)

**YOU (AI Agent): DO NOT START WORK YET!**

**WAIT for user to complete validation:**

### USER Will Run This Command:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
docker compose -f deploy/compose/ce.dev.yml up -d privacy-guard
sleep 5
./scripts/privacy-goose-validate.sh
```

### Expected User Output:
```
═══════════════════════════════════════════════════════════
   Privacy Guard Validation Script
═══════════════════════════════════════════════════════════

Checking Privacy Guard service... ✓ Running

Test 1: SSN
✓ PASSED: Original PII replaced with tokens

Test 2: Email
✓ PASSED: Original PII replaced with tokens

Test 3: Phone
✓ PASSED: Original PII replaced with tokens

Test 4: Multiple PII
✓ PASSED: Original PII replaced with tokens

Test 5: Credit Card
✓ PASSED: Original PII replaced with tokens

Test 6: No PII
✓ PASSED: No PII detected (as expected)

═══════════════════════════════════════════════════════════
   Validation Summary
═══════════════════════════════════════════════════════════

Total Tests: 6
Passed: 6
Failed: 0

✓✓✓ ALL TESTS PASSED! ✓✓✓

Privacy Guard is working correctly!
Ready to proceed with Proxy + Scripts approach for Phase 6.
```

### Your Response After User Reports Results:

**If user reports "6/6 tests passed":**
→ Say: "✅ Validation complete! Starting Workstream A (Vault Production). First task: A1 (TLS/HTTPS Setup)."
→ Begin Workstream A from checklist

**If user reports test failures:**
→ Say: "⚠️ Validation failed. Let me help debug Privacy Guard."
→ Ask user for error output
→ Help debug
→ User retries validation

## Workstream Execution Order (Sequential - DO NOT Skip)

1. ⏸️ **VALIDATION** ← User runs first (you wait)
2. **Workstream A:** Vault Production (2 days)
3. **Workstream B:** Admin UI (3 days)
4. **Workstream C:** Privacy Guard Proxy (3 days)
5. **Workstream D:** Profile Setup Scripts (1 day)
6. **Workstream E:** Wire Lifecycle (1 day)
7. **Workstream F:** Security Hardening (1 day)
8. **Workstream G:** Integration Testing (2 days)
9. **Workstream H:** Documentation (1 day)
10. **FINAL:** Commit, tag v0.6.0, complete Phase 6

## After Each Task/Workstream (MANDATORY PROCESS)

### After Individual Task:
1. Implement task (code/scripts/docs)
2. Test task (verify it works)
3. Mark task ✅ in checklist

### After Complete Workstream (CRITICAL):
1. **Update progress log** (docs/tests/phase6-progress.md) ← Timestamped entry
2. **Update state JSON** (Phase-6-Agent-State.json) ← Current state
3. **Update checklist** (Phase-6-Checklist-FINAL.md) ← Mark tasks ✅
4. **Run workstream tests** (verify all passing)
5. **Commit and push** (if tests pass)

**DO NOT SKIP STEP 1-5!** This is how you resume if session ends.

### Commit Message Template:
```
feat(phase-6): Workstream X complete - <one-line summary>

Completed:
- X1: <task description>
- X2: <task description>
[... all tasks]

Files created/modified:
- src/path/to/file.rs (XXX lines)
- docs/path/to/doc.md

Tests:
- Workstream tests: X/X passing ✅
- Total progress: XX/92 tests passing

Tracking updates:
- Phase-6-Agent-State.json (workstream X → Y)
- Phase-6-Checklist-FINAL.md (X1-X6 marked ✅)
- docs/tests/phase6-progress.md (timestamped entry added)

Next: Workstream Y
```

## Information Sources (Priority Order)

**Tier 1 - Always Check First:**
1. Phase-6-Checklist-FINAL.md (has task details)
2. Existing code (grep for patterns)
3. Your knowledge (Rust, Axum, SvelteKit)

**Tier 2 - When Checklist References Them:**
1. docs/guides/VAULT.md (Section 5 ONLY)
2. src/controller/src/ (for Axum patterns)
3. src/privacy-guard/src/ (for API reference)

**Tier 3 - Rarely:**
1. docs/architecture/PHASE5-ARCHITECTURE.md (if confused)
2. docs/tests/phase5-progress.md (for test patterns)

**Never:**
- Archive/ folder (outdated)
- Full master plan (too long)
- Historical progress logs (use latest only)

## Git Workflow

**Branch:** main (sequential work)  
**Commit:** After each workstream (if tests pass)  
**Push:** Immediately after commit

## Success Criteria

**Phase 6 complete when ALL true:**
- ✅ 92/92 tests passing
- ✅ All 10 critical path scenarios work
- ✅ All 8 workstreams complete (V, A-H)
- ✅ All tracking files updated
- ✅ Tagged release: v0.6.0
- ✅ Documentation complete (6 guides)

## What to Report to User

**After each workstream:**
```
✅ Workstream X Complete!

Completed:
- X1-X6: <summary>

Tests: X/X passing ✅
Files: XXX lines created
Commits: <hash>

Next: Workstream Y (starts with task Y1)
```

**When blocked:**
```
⚠️ Blocker in Workstream X Task Y

Issue: <description>
Need: <what you need from user>
Progress: XX% of workstream complete
```

**At phase end:**
```
🎉 Phase 6 Complete!

✅ All 8 workstreams done
✅ 92/92 tests passing
✅ v0.6.0 tagged
✅ Production-ready MVP delivered

Ready for Phase 7!
```

---

**Now wait for user validation results, then begin Phase 6!**

─────────────────────────────────────────────────────────────────────
 END OF INITIAL PROMPT - COPY TO HERE ↑
─────────────────────────────────────────────────────────────────────
```

---
---
---

# ═══════════════════════════════════════════════════════════════════════
# SECTION 2: RESUME PROMPT  
# 
# USE THIS FOR ALL RESUMED SESSIONS - Every time you come back
# Copy everything between START and END markers below
# 
# ═══════════════════════════════════════════════════════════════════════

```
─────────────────────────────────────────────────────────────────────
 START OF RESUME PROMPT - COPY FROM HERE ↓
─────────────────────────────────────────────────────────────────────

# Resume: Phase 6 - Production Hardening + Admin UI + Privacy Proxy

## Quick Context
- **Phase:** 6 (Production Hardening)
- **Target:** v0.6.0 Production-Ready MVP
- **Approach:** Privacy Guard Proxy + Profile Setup Scripts
- **Timeline:** 14 days total
- **Working Dir:** /home/papadoc/Gooseprojects/goose-org-twin

## What Phase 6 Delivers (For Context)
1. Users sign in → Profiles auto-load → Chat with PII protection
2. Admin UI (SvelteKit) for profile management, org chart, audit
3. Vault production-ready (TLS, AppRole, Raft, audit)
4. Privacy Guard Proxy service (intercepts LLM, masks PII)
5. Profile setup automation (scripts for 6 roles)
6. Lifecycle state machine wired into routes
7. Security hardened (no secrets, .env.example)
8. 92/92 integration tests passing

## STEP 1: Check Current State (Read These 3 Files - MANDATORY)

### 1A. Read State JSON (Tells You Where You Are)
```bash
cat "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
```

**Extract from JSON:**
- `current_workstream` → Which workstream (V, A, B, C, D, E, F, G, or H)
- `current_task` → Which task (e.g., "A2", "B3")
- `completed_workstreams` → Array of done workstreams
- `completed_tasks` → Array of done tasks
- `tests_passing` → "XX/92" (target: 92/92)
- `status` → "IN_PROGRESS" or "COMPLETE"

**Example:**
```json
{
  "current_workstream": "B",
  "current_task": "B3",
  "completed_workstreams": ["V", "A"],
  "completed_tasks": ["V1", "A1", "A2", "A3", "A4", "A5", "A6", "B1", "B2"],
  "tests_passing": "11/92"
}
```
→ Means: You're on Workstream B, Task B3. V and A are done. 11 tests passing so far.

---

### 1B. Read Latest Progress (Tells You What Was Last Done)
```bash
tail -100 docs/tests/phase6-progress.md
```

**Look for in output:**
- Last timestamped entry (e.g., `### [2025-11-08 14:30] - Workstream A Complete ✅`)
- Which tasks were completed
- Which files were created/modified
- Test results
- Git commit hash
- What's stated as "Next"

**This tells you:** What the previous session accomplished and where it stopped.

---

### 1C. Check Next Task (Tells You What to Do Now)
```bash
grep -B 1 -A 3 "^\- \[ \]" "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md" | head -30
```

**Look for:**
- First unchecked `[ ]` task
- Task description and commands
- Expected deliverable

**Example output:**
```
### B3: Profiles Page (6 hours)

- [ ] Create `src/routes/profiles/+page.svelte`:
  - [ ] Profile list sidebar (6 roles)
  - [ ] Monaco YAML editor (for selected profile)
```
→ Means: Next task is B3, create profiles page with Monaco editor.

---

## STEP 2: Resume Work

**Based on state.json current_workstream:**

- **If "VALIDATION":** Wait for user to run validation script
- **If "A":** Continue Workstream A tasks from checklist
- **If "B":** Continue Workstream B tasks from checklist  
- **If "C":** Continue Workstream C tasks from checklist
- **... and so on**

**Then:**
1. Open checklist: `Phase-6-Checklist-FINAL.md`
2. Find workstream section (e.g., "Workstream B")
3. Find first unchecked `[ ]` task
4. Execute that task
5. Test the task
6. Mark task ✅ in checklist
7. Move to next task

**When all tasks in workstream done:**
→ Update all 3 tracking files (see Step 4 below)
→ Commit and push
→ Move to next workstream

---

## STEP 3: Execute Tasks from Checklist

**For each unchecked task in current workstream:**

1. **Read task from checklist** (has code examples, commands)
2. **Implement** (create files, write code, run commands)
3. **Test** (verify it works)
4. **Mark complete** (update checklist with ✅)

**Example:**
```
Checklist says:
- [ ] A1: Generate TLS certificates
  ```bash
  openssl req -newkey rsa:2048 ...
  ```

You do:
1. Run the openssl command
2. Verify cert file created
3. Test: curl --cacert vault.crt https://localhost:8200
4. Mark: - [x] A1: Generate TLS certificates ✅
```

**Don't overthink!** Checklist has the details, just execute.

---

## STEP 4: Update Tracking Files (MANDATORY After Each Workstream)

**After completing ALL tasks in a workstream, update these 3 files:**

### 4A. Update Progress Log (Timestamped Entry)

```bash
vim docs/tests/phase6-progress.md

# Add at end of file:
### [2025-11-XX HH:MM] - Workstream X Complete ✅

**Status:** ✅ COMPLETE

**Completed Tasks:**
- [x] X1: <task description>
- [x] X2: <task description>
- [x] X3: <task description>
[... all tasks from workstream]

**Files Created/Modified:**
- src/new-file.rs (XXX lines) - <description>
- docs/new-doc.md (XXX lines) - <description>
- config/updated.yml (modified)

**Tests:**
- Test suite: X/X passing ✅
- Example: "Vault production tests: 5/5 passing ✅"

**Commits:**
- <git-commit-hash> "feat(phase-6): Workstream X complete - <summary>"
- Example: a1b2c3d "feat(phase-6): Workstream A complete - Vault production"

**State Updates:**
- [x] Phase-6-Agent-State.json updated (current_workstream: X → Y)
- [x] Phase-6-Checklist-FINAL.md marked (X1-X6 all ✅)
- [x] docs/tests/phase6-progress.md updated (this entry)

**Next:** Workstream Y (first task: Y1)

---
```

---

### 4B. Update State JSON (Current State)

```bash
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
```

**Update these fields:**
```json
{
  "status": "IN_PROGRESS",
  "current_state": {
    "current_workstream": "Y",  ← NEXT workstream letter
    "current_task": "Y1",  ← FIRST task of next workstream
    "awaiting_user_action": false
  },
  "workstreams": {
    "X": {
      "status": "COMPLETE",  ← Mark current workstream COMPLETE
      "tasks_completed": 6  ← Update count
    },
    "Y": {
      "status": "IN_PROGRESS"  ← Mark next workstream IN_PROGRESS
    }
  },
  "completed_workstreams": ["V", "A", ..., "X"],  ← ADD X to array
  "completed_tasks": ["A1", "A2", ..., "X6"],  ← ADD all X tasks
  "testing": {
    "vault_tests": {  ← UPDATE relevant test suite
      "passed": 5,
      "status": "PASSING"
    },
    "total_passed": 71  ← UPDATE total (60 + new tests)
  },
  "git": {
    "last_commit": "<commit-hash>",  ← ADD commit hash
    "commits_this_phase": 3  ← INCREMENT count
  },
  "last_updated": "2025-11-XX HH:MM"  ← UPDATE timestamp
}
```

---

### 4C. Update Checklist (Mark Tasks Complete)

```bash
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md"
```

**Find workstream section, mark ALL tasks:**
```markdown
## Workstream X: Name (N days)

### X1: Task Name (N hours)

- [x] Subtask 1  ← Change [ ] to [x]
- [x] Subtask 2  ← Change [ ] to [x]

**Deliverable:** Description ✅  ← Add ✅

---

### X2: Next Task

- [x] Subtask 1  ← Mark complete
...
```

---

### 4D. Git Commit and Push

```bash
git add .

git commit -m "feat(phase-6): Workstream X complete - <summary>

Completed tasks:
- X1: <description>
- X2: <description>
[... list all]

Tests: X/X passing ✅
Total: XX/92 tests now passing

Files created:
- src/path/file.rs (XXX lines)
- docs/path/doc.md (XXX lines)

Tracking updates:
- Phase-6-Agent-State.json (workstream X → Y, tests XX/92)
- Phase-6-Checklist-FINAL.md (X1-X6 marked ✅)
- docs/tests/phase6-progress.md (timestamped entry)

Next: Workstream Y
"

git push origin main
```

**Verify push succeeded before continuing!**

---

## Information Hierarchy (When You Need Help)

**Question:** "How do I implement X?"

**Check in this order:**
1. **Checklist** → Does it have code example? Use it!
2. **Similar code** → `rg "pattern" src/` → Copy pattern
3. **Your knowledge** → Do you know Rust/Axum? Use it!
4. **Docs (last resort)** → Read ONLY section checklist references

---

## Workstream Quick Reference

| ID | Name | Duration | Key Deliverable |
|----|------|----------|----------------|
| V | Validation | 10 min | Privacy Guard validated (user runs) |
| A | Vault Production | 2 days | TLS, AppRole, Raft, Audit, Verify (5 tests) |
| B | Admin UI | 3 days | 5 pages (Dashboard, Profiles, Org, Audit, Settings) |
| C | Privacy Proxy | 3 days | Rust service port 8090 (mask/unmask logic) |
| D | Setup Scripts | 1 day | setup-profile.sh + 6 role wrappers |
| E | Lifecycle | 1 day | Wire lifecycle into session routes |
| F | Security | 1 day | No secrets, .env.example, SECURITY.md |
| G | Testing | 2 days | 92/92 integration tests passing |
| H | Documentation | 1 day | 6 guides complete |

---

## Key Principles (Remember These)

1. **State.json = Source of truth** (where you are)
2. **Checklist = Task details** (what to do)
3. **Progress.md = History** (what was done)
4. **Update all 3 after workstream** (MANDATORY)
5. **Commit after workstream** (if tests pass)
6. **Sequential execution** (don't skip workstreams)

---

## Now: Execute Resume Workflow

1. **Run Step 1 commands** (read 3 files)
2. **Identify current position** (from state.json)
3. **Resume work** (from checklist)
4. **Update tracking** (after each workstream)
5. **Repeat** (until Phase 6 complete)

---

**Start by checking state.json now!**

─────────────────────────────────────────────────────────────────────
 END OF RESUME PROMPT - COPY TO HERE ↑
─────────────────────────────────────────────────────────────────────
```
