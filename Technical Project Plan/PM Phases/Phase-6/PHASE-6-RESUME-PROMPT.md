# Phase 6 Resume Prompt - For New Agent Sessions (MVP Demo Focus)

**Use this prompt when starting a NEW Goose session to resume Phase 6 work.**

**⚡ SCOPE REVISION (2025-11-11):** Phase 6 is now **MVP DEMO FOCUSED** - 6-hour implementation to get functional demo FAST. Automated testing deferred to Phase 7.

---

## 📋 Copy-Paste Resume Prompt

```
I'm resuming Phase 6 work on the Goose Orchestrator project.

🎯 PHASE 6 SCOPE: MVP Demo Focus (75% Complete - 15/20 tasks)
- Goal: Functional 6-window demo showing Agent Mesh + Privacy Guard working end-to-end
- Timeline: 6 hours remaining implementation
- Deferred: 81+ automated tests, advanced UI, deployment docs → Phase 7

CRITICAL - READ THESE DOCUMENTS IN ORDER:

1. Technical Project Plan/PM Phases/Phase-6/PHASE-6-MVP-SCOPE.md ⭐ PRIMARY
   - MVP scope definition (IN SCOPE vs OUT OF SCOPE)
   - Demo workflow (5 phases documented)
   - 6-hour implementation timeline
   - User decisions approved (task persistence, Privacy Guard architecture)

2. Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json
   - Current progress: 75% (15/20 tasks)
   - User decisions approved (all 7 documented)
   - MVP demo tasks breakdown (D.3, D.4, Admin.1-2, Demo.1)
   - Demo windows layout (6-window visual proof)

3. Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md
   - Reorganized: MVP tasks after D.2, deferred tasks at end
   - Current task: D.3 (Task Persistence) - NEXT UP
   - 20 total tasks (5 remaining for MVP demo)

4. docs/tests/phase6-progress.md
   - Latest: 2025-11-11 15:00-15:30 "Phase 6 Scope Revision"
   - Timestamped progress log with all user decisions
   - Implementation plan documented

5. Technical Project Plan/master-technical-project-plan.md
   - Phase 6 section rewritten for MVP demo focus
   - Status: 75% Complete
   - Demo windows layout documented

6. docs/operations/COMPLETE-SYSTEM-REFERENCE.md
   - Current system state (7+ services running)
   - Quick reference for endpoints, credentials

7. docs/operations/SYSTEM-ARCHITECTURE-MAP.md
   - Complete architecture reference

AFTER READING ALL DOCUMENTS:

1. Summarize current Phase 6 MVP status:
   - Current progress: 75% (15/20 tasks)
   - Completed: Workstreams A, B, C, D.1, D.2
   - Next task: D.3 (Task Persistence) or other MVP task
   - Remaining: D.3, D.4, Admin.1-2, Demo.1 (6 hours total)

2. Verify system state:
   - Are 7+ services running? (docker ps | grep -E "controller|privacy-guard|proxy|postgres|vault|keycloak|redis|ollama|goose")
   - Is Vault unsealed? (if not, ask me for unseal keys)
   - Are all 8 profiles in database? (check via Controller API)

3. Ask me: "Ready to proceed with D.3 (Task Persistence) or other MVP task?"

CRITICAL RULES:
- Update Phase-6-Agent-State.json after EVERY milestone
- Update Phase-6-Checklist.md as tasks complete
- Append to docs/tests/phase6-progress.md with timestamped entries
- Run tests after significant changes
- Ask me before making architectural changes
- When stuck, ask me (don't assume or change code without permission)

PRODUCT WORKFLOW (must understand):
1. Admin uploads CSV org chart
2. Admin assigns profiles to users (NOT users choosing their roles)
3. User installs Goose → signs in → Controller auto-pushes assigned profile
4. Privacy Guard Proxy intercepts ALL LLM calls (mask PII → LLM → unmask PII)
5. Agent Mesh enables cross-agent communication (Finance ↔ Manager ↔ Legal)

PHASE 6 GOAL:
Complete MVP-ready backend integration - ALL components fully working together BEFORE UI work (Phase 7).

Ready to resume. Please read all documents and report current status.
```

---

## 🔍 What the Agent Should Do

After reading the documents, the agent should provide:

### 1. Progress Summary (MVP Demo Focus)
```
Phase 6 MVP Progress Summary:
- Overall Progress: 75% (15/20 tasks complete)
- Completed Workstreams: A (Lifecycle), B (Privacy Proxy), C (Multi-Goose)
- Completed Tasks in D: D.1 (Tool Implementation), D.2 (Cross-Agent Communication)
- Next MVP Tasks: D.3 (Task Persistence), D.4 (Privacy Validation), Admin.1-2, Demo.1

Latest Update (from phase6-progress.md):
- 2025-11-11 15:00-15:30: Phase 6 Scope Revision to MVP Demo Focus
- User decisions approved (7 total): Task persistence, Privacy Guard architecture, deployment models
- 6 hours remaining implementation (D.3, D.4, Admin, Demo)
- Deferred to Phase 7: 81+ automated tests, deployment docs, performance benchmarks

Blockers: None
```

### 2. System Health Check
```
System Health (MVP Demo Stack):
✅ 7+ services running (postgres, keycloak, vault, redis, controller, ollama, privacy-guard, privacy-guard-proxy, goose-finance, goose-manager, goose-legal)
✅ Vault unsealed
✅ 8 profiles in database (analyst, developer, finance, hr, legal, manager, marketing, support)
✅ All migrations applied (0001-0007)
✅ Agent Mesh tools: 3/4 working (send_task, notify, request_approval ✅; fetch_status ❌ 404)

Ready for MVP implementation.
```

### 3. Question to User
```
Ready to proceed with MVP tasks?

Remaining MVP Tasks (6 hours total):
D.3: Task Persistence - Fix fetch_status 404 (2 hours)
D.4: Privacy Guard Validation - Remove redundancy, per-instance setup (2 hours)
Admin.1-2: Minimal Admin Dashboard - HTML/JS + API routes (2 hours)
Demo.1: Demo Validation - 6-window layout, manual testing (1 hour)

Which task should I start with?
(Default: D.3 Task Persistence - most critical for Agent Mesh)
```

---

## 📊 Expected Agent Behavior

### ✅ Good Agent Behavior
- Reads ALL documents before asking questions
- Summarizes progress accurately
- Checks system health
- Asks which workstream to focus on
- Updates state files after milestones
- Runs tests frequently
- Asks user when unclear

### ❌ Bad Agent Behavior
- Jumps into coding without reading context
- Doesn't check Phase-6-Agent-State.json
- Doesn't update state files
- Makes architectural changes without asking
- Skips tests
- Assumes things work without verification

---

## 🛠️ Common Resume Scenarios (MVP Demo Focus)

### Scenario 1: Starting MVP Implementation
```
Agent: "I've read all documents. Phase 6 is 75% complete (15/20 tasks).

MVP Tasks Remaining (6 hours):
- D.3: Task Persistence (2h) - Fix fetch_status 404
- D.4: Privacy Guard Validation (2h) - Remove redundancy, per-instance setup
- Admin.1-2: Minimal Admin Dashboard (2h)
- Demo.1: Demo Validation (1h) - 6-window layout

All services running, Vault unsealed, Agent Mesh 3/4 tools working.

Should I start with D.3 (Task Persistence)?"

User: "Yes, proceed with D.3"

Agent: Creates migration 0008, implements routes, tests, updates state files
```

### Scenario 2: Mid-Task Resume (MVP Context)
```
Agent: "Phase-6-Agent-State.json shows D.3 (Task Persistence) in progress.
Looking at phase6-progress.md, last update was:
'2025-11-11 16:00 - Created migration 0008, not yet applied'

Should I:
a) Complete migration 0008 (apply + test)
b) Start fresh on D.3
c) Switch to D.4 (Privacy Guard Validation)"

User: "Complete migration 0008 testing (option a)"

Agent: Applies migration, tests fetch_status, updates state files
```

### Scenario 3: Demo Validation Phase
```
Agent: "All MVP implementation tasks complete (D.3, D.4, Admin.1-2).
Ready for Demo.1 validation.

Demo Workflow (6 windows):
- Terminal 1: Finance Goose
- Terminal 2: Manager Goose  
- Terminal 3: Legal Goose
- Browser 1: Finance Control Panel (8090)
- Browser 2: Manager Control Panel (8091)
- Browser 3: Legal Control Panel (8092)

Should I:
a) Create demo validation script
b) Manual testing with your guidance
c) Both (script + manual)"

User: "Both - create script first"

Agent: Creates demo validation script, documents steps, asks for manual testing
```

---

## 📝 State File Update Examples (MVP Tasks)

### After Completing D.3 (Task Persistence)

#### Update 1: Phase-6-Agent-State.json
```json
{
  "phase_name": "Backend Integration & MVP Demo (Revised Scope)",
  "overall_progress": "80%",
  "current_task": "D.4 - Privacy Guard Architecture Validation",
  "mvp_demo_tasks": {
    "D.3": {
      "name": "Task Persistence",
      "status": "complete",
      "subtasks": {
        "migration_0008": "✅ Complete",
        "tasks_routes": "✅ Complete",
        "fetch_status_fix": "✅ Complete",
        "tests": "✅ 3/3 passing"
      }
    },
    "D.4": {
      "name": "Privacy Guard Architecture Validation",
      "status": "in_progress",
      "subtasks": {
        "D.4.1": "⏳ Remove Proxy redundancy",
        "D.4.2": "⏳ Per-instance setup"
      }
    }
  },
  "last_updated": "2025-11-11T16:30:00Z",
  "notes": "Migration 0008 complete, fetch_status now returns task data. Starting D.4.1."
}
```

#### Update 2: Phase-6-Checklist.md
```markdown
## MVP DEMO TASKS (After D.2)

### Task D.3: Task Persistence ✅ COMPLETE
**Goal:** Fix fetch_status 404 error - tasks must persist to database
- [x] Create migration 0008 (new tasks table)
- [x] Implement POST /api/tasks route (create task)
- [x] Implement GET /api/tasks/:id route (fetch_status)
- [x] Update Agent Mesh tools to use new routes
- [x] Tests passing (3/3)

### Task D.4: Privacy Guard Architecture Validation 🔄 IN PROGRESS
**Goal:** Remove redundancy, prove per-instance isolation

#### D.4.1: Remove Proxy Redundancy ⏳ PENDING
- [ ] Refactor privacy-guard-proxy to pure router
- [ ] Remove src/privacy-guard-proxy/src/masking.rs
- [ ] Update proxy.rs to delegate ALL masking to Service
- [ ] Tests passing

#### D.4.2: Per-Instance Privacy Guard Setup ⏳ PENDING
- [ ] Update ce.dev.yml (3 Ollama + 3 Service + 3 Proxy)
- [ ] Configure ports (8089-8094, 11434-11436)
- [ ] Test isolation (Legal AI doesn't block Finance Rules)
- [ ] Document in COMPLETE-SYSTEM-REFERENCE.md
```

#### Update 3: docs/tests/phase6-progress.md
```markdown
## 2025-11-11 16:30 - Task D.3 Complete ✅

**Agent:** agent-session-xyz789  
**Task:** D.3 (Task Persistence)  
**Duration:** 1.5 hours (under 2h estimate)

**Completed:**
- Created migration 0008 (`db/migrations/metadata-only/0008_create_tasks_table.sql`)
- Implemented tasks routes in Controller (`src/controller/src/routes/tasks.rs`)
- Updated Agent Mesh fetch_status tool to use GET /api/tasks/:id
- All tests passing: 3/3

**Test Results:**
```bash
Test 1: Migration 0008 applies cleanly ✅
Test 2: POST /api/tasks creates task in database ✅
Test 3: GET /api/tasks/:id returns task (no more 404) ✅
```

**Agent Mesh Status:** 4/4 tools working (send_task, notify, request_approval, fetch_status ✅)

**Next:** Task D.4.1 - Remove Privacy Guard Proxy redundancy (30 mins)

**Branch:** feature/phase6-task-persistence  
**Commits:**
- abc123def (migration 0008)
- def456ghi (tasks routes)
- ghi789jkl (Agent Mesh tool updates)
```

---

## 🚨 Critical Reminders

### Before Coding
1. Read Phase-6-Agent-State.json
2. Read Phase-6-Checklist.md
3. Read docs/tests/phase6-progress.md
4. Understand what's done, what's pending
5. Ask user which workstream to focus on

### During Coding
1. Follow task breakdown in PHASE-6-MAIN-PROMPT.md
2. Write tests for every feature
3. Run tests frequently
4. Ask user when unclear

### After Milestones
1. Update Phase-6-Agent-State.json
2. Update Phase-6-Checklist.md
3. Append to docs/tests/phase6-progress.md
4. Commit changes with conventional commit message
5. Ask user: "Task X.Y complete. Ready for X.Z?"

---

## 📚 Quick Reference Links

### Must-Read Documents (MVP Demo Focus)
- **⭐ MVP Scope:** `Technical Project Plan/PM Phases/Phase-6/PHASE-6-MVP-SCOPE.md` (PRIMARY)
- Master Plan: `Technical Project Plan/master-technical-project-plan.md`
- System Architecture: `docs/operations/SYSTEM-ARCHITECTURE-MAP.md`
- Current State: `docs/operations/COMPLETE-SYSTEM-REFERENCE.md`
- Phase 6 Main: `Technical Project Plan/PM Phases/Phase-6/PHASE-6-MAIN-PROMPT.md`

### State Tracking
- Agent State: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`
- Checklist: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md` (reorganized: MVP tasks first)
- Progress Log: `docs/tests/phase6-progress.md`

### Operations
- Startup: `docs/operations/STARTUP-GUIDE.md`
- Testing: `docs/operations/TESTING-GUIDE.md`
- Troubleshooting: `docs/operations/COMPLETE-SYSTEM-REFERENCE.md`

### Product
- Vision: `docs/product/productdescription.md`
- Architecture: `docs/architecture/PHASE5-ARCHITECTURE.md`

### Architecture Decisions (User Approved)
- D2 Completion: `Technical Project Plan/PM Phases/Phase-6/docs/D2_COMPLETION_SUMMARY.md`
- Decisions Needed: `Technical Project Plan/PM Phases/Phase-6/docs/ARCHITECTURE-DECISIONS-NEEDED.md`

---

**END OF RESUME PROMPT**

**Remember:** Read context first, check state files, ask user which workstream, update state after milestones.
