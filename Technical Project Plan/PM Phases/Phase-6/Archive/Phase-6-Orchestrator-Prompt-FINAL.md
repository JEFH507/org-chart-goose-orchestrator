# Phase 6 Orchestrator Prompt - Production Hardening + Admin UI + Privacy Proxy

**Version:** 3.0 (Architecture-Aligned)  
**Approach:** Privacy Guard Proxy + Profile Setup Scripts  
**Timeline:** 14 days (3 weeks calendar)  
**Target:** v0.6.0 Production-Ready MVP

---

# ═══════════════════════════════════════════════════════════════════
# SECTION 1: INITIAL SESSION PROMPT
# Copy everything from here to "END OF INITIAL PROMPT" for first session
# ═══════════════════════════════════════════════════════════════════

## 📋 COPY FROM HERE - Initial Session Prompt

```markdown
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
**Decision:** Privacy Guard Proxy + Profile Setup Scripts

**Why:** Follows proven Phases 1-5 service pattern (see docs/architecture/SRC-ARCHITECTURE-AUDIT.md)
- Add new service: privacy-guard-proxy (port 8090)
- Add automation scripts: setup-profile.sh
- No goose Desktop fork needed
- 14 days timeline (vs 19 days for fork approach)

## Your Working Directory
/home/papadoc/Gooseprojects/goose-org-twin

## Key Documents (Read ONLY When Needed)
**Checklist (Primary Guide):**
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md

**Architecture Context:**
- docs/architecture/SRC-ARCHITECTURE-AUDIT.md (understand /src structure)
- docs/architecture/PHASE5-ARCHITECTURE.md (current system)

**Progress Tracking (Update After Each Workstream):**
- docs/tests/phase6-progress.md
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md

**Decision Context (Reference Only):**
- Technical Project Plan/PM Phases/Phase-6/DECISION-TREE.md
- Technical Project Plan/PM Phases/Phase-6/ARCHITECTURE-ALIGNED-RECOMMENDATIONS.md

## IMPORTANT: First Action (USER Will Do This)
**BEFORE YOU START ANY WORK:**

The USER will run the validation script to ensure Privacy Guard is working:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
docker compose -f deploy/compose/ce.dev.yml up -d privacy-guard
sleep 5
./scripts/privacy-goose-validate.sh
```

**Expected Output:** 6/6 tests pass ✅

**Your Role:** WAIT for user to report validation results

**If 6/6 pass:** Proceed to Workstream A (Vault Production)
**If any fail:** Help user debug Privacy Guard issues first

## Workstream Execution Order

Execute in sequence (DO NOT skip ahead):

1. ⏸️ **VALIDATION** (User runs, you wait for results)
2. **Workstream A:** Vault Production (2 days)
3. **Workstream B:** Admin UI (3 days)
4. **Workstream C:** Privacy Guard Proxy (3 days)
5. **Workstream D:** Profile Setup Scripts (1 day)
6. **Workstream E:** Wire Lifecycle (1 day)
7. **Workstream F:** Security Hardening (1 day)
8. **Workstream G:** Integration Testing (2 days)
9. **Workstream H:** Documentation (1 day)
10. **Final:** Commit, tag v0.6.0, complete Phase 6

## After Each Workstream (MANDATORY):

**Update Tracking Files:**
```bash
# 1. Update progress log
vim docs/tests/phase6-progress.md
# Add timestamped entry with: tasks completed, files created, tests passed

# 2. Update state JSON
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
# Update: current_workstream, completed_workstreams, tests_passing

# 3. Update checklist
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md"
# Mark completed tasks with ✅

# 4. Commit if tests passed
git add .
git commit -m "feat(phase-6): Workstream X complete - <description>"
git push origin main
```

**CRITICAL:** Do NOT proceed to next workstream without updating all 3 tracking files!

## Information Sources (Prioritized)

**Tier 1 (Always Check First):**
- Phase-6-Checklist-FINAL.md (task details)
- Existing code patterns (grep, rg for similar implementations)
- Your knowledge (Rust, Axum, SvelteKit, Docker)

**Tier 2 (Check When Needed):**
- docs/guides/VAULT.md (Section 5 ONLY for Vault production)
- docs/profiles/SPEC.md (for profile schema reference)
- src/controller/ code (for Axum patterns)
- src/privacy-guard/ code (for Privacy Guard API)

**Tier 3 (Rarely Needed):**
- docs/architecture/PHASE5-ARCHITECTURE.md (if confused about system)
- docs/tests/phase5-progress.md (if need to see test patterns)

**Never Read:**
- Historical progress logs (too verbose)
- Archive/ folder documents (outdated)
- Full master plan (you have this prompt)

## Cost Optimization Rules

1. **Read minimally:** Checklist first, then code, then docs
2. **Use existing patterns:** Grep for similar code instead of reading docs
3. **Cache knowledge:** Don't re-read same sections
4. **Ask before reading:** "Do I need this doc or can I infer from checklist?"

## Git Workflow

**Branch Strategy:**
- Work on: main (Phase 6 is sequential, no parallel work)
- Commit after: Each workstream completion (if tests pass)
- Push: Immediately after commit

**Commit Message Format:**
```
feat(phase-6): Workstream X complete - <description>

- Task X1: <what was done>
- Task X2: <what was done>
- Tests: X/X passing ✅

Files:
- src/path/to/file.rs (XXX lines)
- docs/path/to/doc.md

Updates:
- Phase-6-Agent-State.json (workstream X complete)
- Phase-6-Checklist-FINAL.md (marked tasks ✅)
- docs/tests/phase6-progress.md (timestamped entry)
```

## Success Criteria (Phase 6 Complete When)

**All 10 critical path tests work:**
1. ✅ Vault HTTPS (curl https://localhost:8200 works)
2. ✅ Vault AppRole (Controller auth without root token)
3. ✅ Profile Loading (./setup-profile.sh finance works)
4. ✅ Privacy Protection (LLM sees masked PII)
5. ✅ Admin Login (http://localhost:8088/admin loads)
6. ✅ Profile Editing (Admin can publish profiles)
7. ✅ Org Chart Upload (CSV import works)
8. ✅ Signature Verification (Tampered profile → 403)
9. ✅ Lifecycle Validation (Invalid transition → 400)
10. ✅ 92/92 Tests Pass

**Deliverables:**
- Vault production-ready
- Admin UI deployed (5 pages)
- Privacy Guard Proxy service running
- Profile setup scripts (6 roles)
- Lifecycle integrated into routes
- Security hardened
- Documentation complete (6 guides)
- Tagged release: v0.6.0

## When to Pause and Resume

**Pause if:**
- Approaching context window limits (>80% token usage)
- Major blocker encountered (need user input)
- Workstream complete (good stopping point)

**Before pausing:**
1. Update all 3 tracking files (progress.md, state.json, checklist)
2. Commit changes (if tests passed)
3. Report status to user

**Resume using:** Resume Prompt section below

---

# FIRST ACTION FOR USER (Not AI Agent)

**USER: Before starting Phase 6, run validation:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Start Privacy Guard
docker compose -f deploy/compose/ce.dev.yml up -d privacy-guard

# Wait for startup
sleep 5

# Run validation
./scripts/privacy-goose-validate.sh
```

**Expected Output:**
```
✓✓✓ ALL TESTS PASSED! ✓✓✓

Privacy Guard is working correctly!
Ready to proceed with Proxy + Scripts approach for Phase 6.
```

**After validation passes:**
1. Report results to AI agent
2. AI agent will start Workstream A (Vault Production)

**If validation fails:**
1. Share error output with AI agent
2. AI agent will help debug Privacy Guard
3. Retry validation
```

---
---
---

## 📋 RESUME PROMPT (Copy This for New Sessions)

```markdown
# Resume: Phase 6 - Production Hardening + Admin UI + Privacy Proxy

## Context
You are resuming Phase 6 work for the org-chart-goose-orchestrator project.

**Phase:** 6  
**Target:** v0.6.0 Production-Ready MVP  
**Approach:** Privacy Guard Proxy + Profile Setup Scripts  
**Timeline:** 14 days total

## What Phase 6 Delivers
1. Users sign in → Profiles auto-load → Chat with PII protection
2. Admin UI for profile management
3. Vault production-ready (TLS, AppRole, Raft)
4. Privacy Guard Proxy intercepts LLM requests
5. 92/92 tests passing

## Working Directory
/home/papadoc/Gooseprojects/goose-org-twin

## Check Current State (Do This First)

**Step 1:** Read current state:
```bash
cat "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
```

**Step 2:** Read latest progress:
```bash
tail -50 docs/tests/phase6-progress.md
```

**Step 3:** Check checklist:
```bash
grep -A 2 "\[ \]" "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md" | head -20
```

**This tells you:**
- Which workstream you're on
- What tasks are incomplete
- What was last completed

## Workstream Order (For Reference)
1. Validation (User runs validation script)
2. A. Vault Production (2 days)
3. B. Admin UI (3 days)
4. C. Privacy Guard Proxy (3 days)
5. D. Profile Setup Scripts (1 day)
6. E. Wire Lifecycle (1 day)
7. F. Security Hardening (1 day)
8. G. Integration Testing (2 days)
9. H. Documentation (1 day)

## Primary Task Source
**Follow:** Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md

**For each unchecked task:**
1. Read task description from checklist
2. Implement the task
3. Test the task
4. Mark task complete in checklist ✅

## After Completing Tasks (MANDATORY)

**Update all 3 tracking files:**

1. **Progress Log:**
```bash
vim docs/tests/phase6-progress.md
# Add timestamped entry:
### [YYYY-MM-DD HH:MM] - Workstream X Task Y Complete
- Completed: <task description>
- Files: <list files created/modified>
- Tests: <pass/fail status>
- Next: <next task>
```

2. **State JSON:**
```bash
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
# Update:
{
  "current_workstream": "X",
  "current_task": "XY",
  "completed_tasks": ["X1", "X2", ...],
  "tests_passing": "XX/92"
}
```

3. **Checklist:**
```bash
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md"
# Mark completed task:
- [x] XY: Task description
```

**Then commit if tests passed:**
```bash
git add .
git commit -m "feat(phase-6): Workstream X Task Y complete"
git push origin main
```

## Key Architecture Principles (From Audit)

**Service Pattern (Add New Services):**
- controller (8088) ✅ Exists
- privacy-guard (8089) ✅ Exists
- privacy-guard-proxy (8090) 🆕 Build in Workstream C
- admin-ui (at /admin) 🆕 Build in Workstream B

**Module Pattern (Import in Controller):**
- lifecycle ✅ Exists (wire into routes in Workstream E)
- profile ✅ Exists (used in routes)
- vault ✅ Exists (upgrade to production in Workstream A)

**Script Pattern (Automation):**
- scripts/setup-profile.sh 🆕 Build in Workstream D
- scripts/goose-*.sh 🆕 Build in Workstream D (convenience wrappers)

**This pattern worked in Phases 1-5, continue it!**

## Information Hierarchy

**When you need information, check in this order:**

1. **Checklist** (task details) ← START HERE
2. **Existing code** (grep for similar patterns)
3. **Your knowledge** (Rust, Axum, SvelteKit, Docker)
4. **Specific docs** (only sections referenced in checklist)

**Example:**
- Need Vault TLS setup? → Checklist has commands → Use those (don't read full Vault guide)
- Need Axum routing? → Grep src/controller/src/main.rs for pattern → Copy pattern
- Need profile schema? → Already know it from Phase 5 → Reuse knowledge

## Progress Tracking (Critical)

**After EVERY workstream completion:**

1. Update docs/tests/phase6-progress.md (timestamped entry)
2. Update Phase-6-Agent-State.json (current state)
3. Update Phase-6-Checklist-FINAL.md (mark tasks ✅)
4. Commit changes (git commit + push)

**Do NOT skip this!** These files are your resume point if session ends.

## Testing Requirements

**After each workstream:**
- Run workstream-specific tests
- Verify tests pass before moving to next workstream
- Update test results in progress.md

**After all workstreams:**
- Run full integration test suite (92 tests)
- Run end-to-end workflow test
- Verify performance targets met

## Current Status
**Read state.json to determine where to start**

## Next Action
**Check Phase-6-Checklist-FINAL.md for first unchecked task**

---

**Ready to execute Phase 6!**
```

---
---
---

## 📋 RESUME PROMPT - Copy This for Each New Session

```markdown
# Resume: Phase 6 - Production Hardening + Admin UI + Privacy Proxy

## Quick Context
- **Phase:** 6 (Production Hardening)
- **Target:** v0.6.0 MVP
- **Approach:** Privacy Guard Proxy + Profile Setup Scripts
- **Timeline:** 14 days total
- **Working Dir:** /home/papadoc/Gooseprojects/goose-org-twin

## Step 1: Check Current State (Read These 3 Files)

### 1A. Read State JSON (Tells You Where You Are)
```bash
cat "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
```

**Look for:**
- `"current_workstream"` → Which workstream you're on (A, B, C, D, E, F, G, or H)
- `"current_task"` → Which task within workstream
- `"completed_workstreams"` → What's done already
- `"tests_passing"` → How many tests pass (target: 92/92)

### 1B. Read Latest Progress (Tells You What Was Last Done)
```bash
tail -100 docs/tests/phase6-progress.md
```

**Look for:**
- Last timestamped entry
- Which tasks were completed
- Which files were created
- What's next

### 1C. Check Next Task (Tells You What to Do)
```bash
grep -A 3 "^\- \[ \]" "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md" | head -20
```

**Look for:**
- First unchecked `[ ]` task
- Task description
- Deliverable expected

## Step 2: Resume Work

**Based on what you found:**

**If state.json says `"current_workstream": "A"`:**
→ Continue Workstream A tasks from checklist

**If state.json says `"current_workstream": "B"`:**
→ Continue Workstream B tasks from checklist

**And so on...**

## Step 3: Execute Current Task

**Follow checklist task exactly:**
1. Read task description
2. Implement solution (code, scripts, docs)
3. Test implementation
4. Mark task complete in checklist ✅
5. Move to next task

**If workstream complete:**
→ Update all 3 tracking files (progress.md, state.json, checklist)
→ Commit and push
→ Move to next workstream

## Step 4: Update Tracking (After Each Workstream)

**MANDATORY after each workstream completion:**

### 4A. Update Progress Log
```bash
vim docs/tests/phase6-progress.md

# Add entry:
### [YYYY-MM-DD HH:MM] - Workstream X Complete ✅

**Status:** ✅ COMPLETE

**Completed Tasks:**
- [x] X1: <description>
- [x] X2: <description>

**Files Created/Modified:**
- src/path/file.rs (XXX lines)
- docs/path/doc.md

**Tests:**
- Test suite: X/X passing ✅

**Commits:**
- <commit-hash> "<commit-message>"

**State Updates:**
- [x] Phase-6-Agent-State.json updated
- [x] Phase-6-Checklist-FINAL.md marked ✅
- [x] Progress log updated (this file)

**Next:** Workstream Y
```

### 4B. Update State JSON
```bash
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"

# Update:
{
  "phase": "6",
  "status": "IN_PROGRESS",
  "current_workstream": "Y",  ← NEXT workstream
  "current_task": "Y1",  ← FIRST task of next workstream
  "completed_workstreams": ["V", "A", "B", ...],  ← ADD completed
  "completed_tasks": ["A1", "A2", ..., "X6"],  ← ADD all completed tasks
  "tests_passing": "XX/92",  ← UPDATE count
  "last_updated": "YYYY-MM-DD HH:MM"
}
```

### 4C. Update Checklist
```bash
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md"

# Mark all completed tasks:
- [x] X1: Task description ✅
- [x] X2: Task description ✅
...
- [x] X6: Task description ✅

**Workstream X Complete** → Update progress log, mark checklist ✅  ← ADD THIS
```

### 4D. Commit and Push
```bash
git add .
git commit -m "feat(phase-6): Workstream X complete - <summary>

Completed tasks:
- X1: <description>
- X2: <description>

Tests: X/X passing ✅

Files created:
- src/path/file.rs (XXX lines)

Updates:
- Phase-6-Agent-State.json
- Phase-6-Checklist-FINAL.md
- docs/tests/phase6-progress.md
"

git push origin main
```

## Workstream Summary (For Quick Reference)

| Workstream | Duration | Key Deliverable |
|------------|----------|----------------|
| V. Validation | 10 min | Privacy Guard validated (6/6 tests) |
| A. Vault Production | 2 days | TLS, AppRole, Raft, Audit, Verify |
| B. Admin UI | 3 days | 5 pages (Dashboard, Profiles, Org, Audit, Settings) |
| C. Privacy Proxy | 3 days | Rust service (port 8090, mask/unmask) |
| D. Setup Scripts | 1 day | setup-profile.sh + 6 role wrappers |
| E. Lifecycle | 1 day | Wire lifecycle into session routes |
| F. Security | 1 day | No secrets, .env.example, SECURITY.md |
| G. Testing | 2 days | 92/92 integration tests passing |
| H. Documentation | 1 day | 6 guides (Vault, Proxy, Setup, Admin, Security, Migration) |

**Total:** 14 days

## Architecture Reference (Quick Lookup)

**Services (Docker containers):**
- controller (8088) - Main API
- privacy-guard (8089) - PII detection
- privacy-guard-proxy (8090) - LLM interceptor 🆕
- admin-ui (/admin) - Web UI 🆕

**Modules (Rust libraries):**
- lifecycle - Session state machine (wire into routes in E)
- profile - Schema/validation/signing (used in routes)
- vault - Vault client (upgrade to AppRole in A)

**Scripts (Bash automation):**
- setup-profile.sh 🆕 - Profile loading
- goose-*.sh 🆕 - Convenience wrappers

## Key Files Locations

**Tracking (Update These):**
- docs/tests/phase6-progress.md
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md

**Execution Guide:**
- Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md

**Architecture:**
- docs/architecture/SRC-ARCHITECTURE-AUDIT.md (understand /src)
- docs/architecture/PHASE5-ARCHITECTURE.md (current system)

**Code:**
- src/controller/ - Main API (reference for Axum patterns)
- src/privacy-guard/ - Privacy Guard API (reference for integration)
- src/vault/ - Vault client (upgrade in Workstream A)
- src/lifecycle/ - Session lifecycle (wire in Workstream E)

## Remember: Update Tracking Files After Each Workstream!

**The 3 files that MUST be updated:**
1. docs/tests/phase6-progress.md (timestamped entry)
2. Phase-6-Agent-State.json (current state)
3. Phase-6-Checklist-FINAL.md (mark tasks ✅)

**Then commit and push!**

## Now: Check Current State and Resume Work

**Run Step 1 commands above to find where you are, then continue!**
```

---

## End of Resume Prompt

---

## 📝 Notes for Orchestrator

**Key Principles:**
1. **Read state.json first** (tells you where you are)
2. **Follow checklist** (tells you what to do)
3. **Update tracking after each workstream** (progress, state, checklist)
4. **Commit after each workstream** (if tests pass)
5. **Don't skip ahead** (workstreams have dependencies)

**Cost Optimization:**
- Checklist has task details (don't read extra docs)
- Grep existing code for patterns (don't re-read docs)
- Cache knowledge from previous sessions (don't re-read)

**Quality Gates:**
- Tests must pass before moving to next workstream
- All 3 tracking files must be updated
- Git commit must be pushed

**Success Metric:**
- 92/92 tests passing at end
- v0.6.0 tagged and released
- All 10 critical path tests work

---

**This prompt structure ensures continuity across session restarts!**
