# Phase 6 Resume Prompt - For New Agent Sessions

**Use this prompt when starting a NEW Goose session to resume Phase 6 work.**

---

## 📋 Copy-Paste Resume Prompt

```
I'm resuming Phase 6 work on the Goose Orchestrator project.

CRITICAL - READ THESE DOCUMENTS IN ORDER:

1. Technical Project Plan/master-technical-project-plan.md
   - Overall project context and phase breakdown

2. docs/operations/COMPLETE-SYSTEM-REFERENCE.md
   - Current system state (all services, volumes, profiles)
   - Quick reference for endpoints, credentials, troubleshooting

3. docs/operations/SYSTEM-ARCHITECTURE-MAP.md
   - Complete architecture (modules, services, database, volumes)
   - Where everything lives (source code, configs, migrations)

4. Technical Project Plan/PM Phases/Phase-6/PHASE-6-MAIN-PROMPT.md
   - Phase 6 goals, workstreams, deliverables
   - Task breakdown for all 5 workstreams (A, B, C, D, V)

5. Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json
   - Current progress (which workstream, which task)
   - What's complete, in-progress, pending

6. Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md
   - Detailed task checklist (marked with [x] when complete)
   - Subtasks and acceptance criteria

7. docs/tests/phase6-progress.md
   - Timestamped progress log
   - Latest updates, decisions, blockers

AFTER READING ALL DOCUMENTS:

1. Summarize current Phase 6 progress:
   - Which workstream is active? (A, B, C, D, or V)
   - Which task is in progress?
   - What's complete? What's pending?
   - Any blockers or issues?

2. Verify system state:
   - Are all 7 services running and healthy? (docker ps)
   - Is Vault unsealed? (if not, ask me for unseal keys)
   - Are all 8 profiles in database? (check via Controller API)

3. Ask me: "Which workstream should I focus on today?"

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

### 1. Progress Summary
```
Phase 6 Progress Summary:
- Current Workstream: A (Lifecycle Integration)
- Current Task: A.2 (Database Persistence)
- Tasks Complete: A.1 (Route Integration)
- Tasks Pending: A.3, B.1-B.5, C.1-C.4, D.1-D.4, V.1-V.5

Latest Update (from phase6-progress.md):
- 2025-11-10 14:00: Completed session routes (A.1)
- Unit tests: 5/5 passing
- Next: Create migration 0007 for session FSM support

Blockers: None
```

### 2. System Health Check
```
System Health:
✅ All 7 services running (postgres, keycloak, vault, ollama, privacy-guard, redis, controller)
✅ Vault unsealed
✅ 8 profiles in database (analyst, developer, finance, hr, legal, manager, marketing, support)
✅ All migrations applied (001, 0002, 0004, 0005, 0006)

Ready to continue work.
```

### 3. Question to User
```
Which workstream should I focus on?

Options:
A. Lifecycle Integration (current - 1/3 tasks complete)
B. Privacy Guard Proxy (not started)
C. Multi-Goose Test Environment (not started)
D. Agent Mesh E2E Testing (not started)
V. Full Integration Validation (not started)

Or: "Continue A.2 (Database Persistence)"?
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

## 🛠️ Common Resume Scenarios

### Scenario 1: Services Not Running
```
Agent: "I see services are not running. Should I start them?"

User: "Yes, follow STARTUP-GUIDE.md"

Agent: 
1. Starts postgres, keycloak, vault
2. Asks user to unseal Vault (provides script location)
3. Waits for user confirmation
4. Runs migrations
5. Starts remaining services
6. Verifies health
```

### Scenario 2: Mid-Task Resume
```
Agent: "Phase-6-Agent-State.json shows Task A.2 in progress.
Looking at phase6-progress.md, last update was:
'2025-11-10 14:30 - Started migration 0007, not yet tested'

Should I:
a) Complete migration 0007 testing
b) Start fresh on A.2
c) Switch to different workstream"

User: "Complete migration testing (option a)"

Agent: Runs migration, tests, updates state files
```

### Scenario 3: Cross-Session Coordination
```
Agent: "I see another agent worked on Workstream B yesterday.
Their progress: Privacy Guard Proxy scaffold created (B.1 complete).
Current status: Building, tests pending.

Should I:
a) Continue B.1 (complete tests)
b) Start B.2 (PII masking integration)
c) Switch to different workstream"

User: "Continue B.1, finish the tests"

Agent: Completes tests, updates checklist, asks about B.2
```

---

## 📝 State File Update Examples

### After Completing a Task

#### Update 1: Phase-6-Agent-State.json
```json
{
  "phase": "6",
  "current_workstream": "A",
  "current_task": "A.3",
  "workstreams": {
    "A": {
      "name": "Lifecycle Integration",
      "status": "in_progress",
      "tasks_complete": ["A.1", "A.2"],
      "tasks_in_progress": ["A.3"],
      "tasks_pending": []
    }
  },
  "last_updated": "2025-11-10T15:00:00Z",
  "last_agent": "agent-session-abc123",
  "notes": "Migration 0007 complete, tested successfully. Starting A.3 (testing)."
}
```

#### Update 2: Phase-6-Checklist.md
```markdown
## Workstream A: Lifecycle Integration

### Task A.1: Route Integration ✅ COMPLETE
- [x] Create src/controller/src/routes/sessions.rs
- [x] Wire endpoints into main.rs
- [x] Unit tests passing (5/5)
- [x] Integration tests passing (3/3)

### Task A.2: Database Persistence ✅ COMPLETE
- [x] Migration 0007 created (update sessions table)
- [x] SessionManager updated to persist state
- [x] State recovery on Controller restart tested
- [x] Tests passing (4/4)

### Task A.3: Testing 🔄 IN PROGRESS
- [ ] Create test_session_lifecycle.sh
- [ ] Session state diagram created
- [ ] Update TESTING-GUIDE.md
```

#### Update 3: docs/tests/phase6-progress.md
```markdown
## 2025-11-10 15:00 - Task A.2 Complete ✅

**Agent:** agent-session-abc123  
**Workstream:** A (Lifecycle Integration)  
**Task:** A.2 (Database Persistence)

**Completed:**
- Created migration 0007 (`db/migrations/metadata-only/0007_update_sessions_for_lifecycle.sql`)
- Updated SessionManager to persist state transitions
- Added state recovery logic (Controller restart preserves sessions)
- Tests passing: 4/4

**Test Results:**
```bash
Test 1: Migration applies cleanly ✅
Test 2: Session state persists to database ✅
Test 3: Controller restart recovers sessions ✅
Test 4: Concurrent session updates don't conflict ✅
```

**Next:** Task A.3 - Create comprehensive session lifecycle tests

**Branch:** feature/phase6-lifecycle-persistence  
**Commit:** def456abc (migration 0007)  
**Commit:** ghi789def (SessionManager updates)
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

### Must-Read Documents
- Master Plan: `Technical Project Plan/master-technical-project-plan.md`
- System Architecture: `docs/operations/SYSTEM-ARCHITECTURE-MAP.md`
- Current State: `docs/operations/COMPLETE-SYSTEM-REFERENCE.md`
- Phase 6 Main: `Technical Project Plan/PM Phases/Phase-6/PHASE-6-MAIN-PROMPT.md`

### State Tracking
- Progress: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`
- Checklist: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md`
- Log: `docs/tests/phase6-progress.md`

### Operations
- Startup: `docs/operations/STARTUP-GUIDE.md`
- Testing: `docs/operations/TESTING-GUIDE.md`
- Troubleshooting: `docs/operations/COMPLETE-SYSTEM-REFERENCE.md`

### Product
- Vision: `docs/product/productdescription.md`
- Architecture: `docs/architecture/PHASE5-ARCHITECTURE.md`

---

**END OF RESUME PROMPT**

**Remember:** Read context first, check state files, ask user which workstream, update state after milestones.
