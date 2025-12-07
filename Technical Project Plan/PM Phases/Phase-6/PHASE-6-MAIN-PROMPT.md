# Phase 6: Backend Integration & Multi-Agent Testing - Main Prompt

**VERSION:** 2.0 (Comprehensive Restructure - 2025-11-10)  
**STATUS:** Ready to start  
**EXPECTED DURATION:** 4-6 weeks

---

## 🎯 Copy-Paste Prompt for New Agent Sessions

```
You are continuing Phase 6 of the goose Orchestrator project.

CONTEXT DOCUMENTS (read in order):
1. Technical Project Plan/master-technical-project-plan.md - Overall project context
2. docs/operations/SYSTEM-ARCHITECTURE-MAP.md - System architecture
3. docs/operations/COMPLETE-SYSTEM-REFERENCE.md - Current system state
4. Technical Project Plan/PM Phases/Phase-6/PHASE-6-MAIN-PROMPT.md - This phase goals
5. Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json - Current progress
6. Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md - Task checklist
7. docs/tests/phase6-progress.md - Detailed progress log

BEFORE STARTING ANY WORK:
1. Read Phase-6-Agent-State.json to understand current progress
2. Read Phase-6-Checklist.md to see what's complete/pending
3. Read docs/tests/phase6-progress.md for latest updates
4. Ask user which workstream to focus on (A, B, C, D, or V)

CRITICAL RULES:
- Update Phase-6-Agent-State.json after EVERY milestone
- Update Phase-6-Checklist.md as tasks complete
- Append to docs/tests/phase6-progress.md with timestamped entries
- Ask user before making architectural changes
- Run tests after every significant change
- When stuck, ask user for guidance (don't assume or change without permission)

WORKFLOW UNDERSTANDING (must internalize):
1. Admin uploads CSV org chart
2. Admin assigns profiles to users (NOT users choosing)
3. User installs goose → signs in → Controller auto-pushes assigned profile
4. Privacy Guard Proxy intercepts ALL LLM calls (mask → LLM → unmask)
5. Agent Mesh enables cross-agent communication (Finance ↔ Manager ↔ Legal)

SYSTEM STARTUP (if services not running):
Follow docs/operations/STARTUP-GUIDE.md exactly:
1. Start postgres, keycloak, vault
2. Unseal Vault (ask user for keys - 3 of 5 required)
3. Run database migrations (001, 0002, 0004, 0005, 0006)
4. Start ollama, privacy-guard, redis, controller
5. Verify all services healthy: docker ps

ALL PARTS MUST BE FULLY INTEGRATED - no gaps before UI work (Phase 7).

Ready to continue? Ask me which workstream to focus on.
```

---

## 📋 Phase 6 Overview

### Goal
Complete MVP-ready backend integration with full cross-agent communication, Privacy Guard Proxy, and multi-goose testing capabilities.

### Strategy
**Integration-first approach** - ALL backend components fully working together BEFORE any UI work.

### Success Criteria
3+ goose agents (Finance, Manager, Legal) communicating via Agent Mesh, all requests intercepted by Privacy Guard Proxy, full demo workflow operational.

---

## 🏗️ Phase 6 Workstreams

### Workstream A: Lifecycle Integration (Week 1-2)
**Goal:** Wire Lifecycle module into Controller routes, enable session management FSM.

**Key Deliverables:**
- Session endpoints (POST /sessions, PUT /sessions/{id}/events, etc.)
- Session state persistence to database
- Migration 0007 (update sessions table for FSM)
- Session lifecycle tests (8 tests minimum)

**Success Criteria:**
- [x] Session endpoints accessible via Controller API
- [x] Session states persist to database
- [x] Session lifecycle tests: 8/8 passing
- [x] Controller restart preserves session states

---

### Workstream B: Privacy Guard Proxy (Week 2-3)
**Goal:** Build HTTP proxy to intercept ALL LLM calls for PII masking/unmasking.

**Key Deliverables:**
- Privacy Guard Proxy service (new Rust/Axum service, port 8090)
- PII masking before LLM, unmasking after LLM
- OpenRouter/Anthropic/OpenAI provider support
- Update all 8 profile YAMLs to use proxy (api_base: http://privacy-guard-proxy:8090/v1)
- Proxy tests (8 tests minimum)

**Architecture:**
```
goose Agent → Privacy Guard Proxy → Mask PII → LLM (OpenRouter)
                     ↓                               ↓
              Privacy Guard (8089)            Response
                     ↑                               ↓
                Unmask PII ← Privacy Guard Proxy ←─┘
```

**Success Criteria:**
- [x] Privacy Guard Proxy running on port 8090
- [x] All LLM calls routed through proxy
- [x] PII masked before LLM, unmasked after
- [x] All tests passing (8/8)
- [x] Latency overhead < 200ms per request

---

### Workstream C: Multi-goose Test Environment (Week 3-4)
**Goal:** Set up Docker-based goose containers for testing 3+ agents simultaneously.

**Key Deliverables:**
- Docker goose image (Dockerfile, config script)
- 3 goose containers (Finance, Manager, Legal) in ce.dev.yml
- Agent Mesh configuration for multi-goose
- Agent registration/discovery in Controller
- Multi-goose tests (8 tests minimum)

**Architecture:**
```
Docker Network: goose-orchestrator-network
├─ goose-finance (container)
├─ goose-manager (container)
├─ goose-legal (container)
└─ Controller (routes messages between agents)
```

**Success Criteria:**
- [x] 3 goose containers running (Finance, Manager, Legal)
- [x] Each goose has correct profile loaded
- [x] Agent Mesh discovers all 3 agents
- [x] All tests passing (8/8)
- [x] Documentation complete

---

### Workstream D: Agent Mesh E2E Testing (Week 4-5)
**Goal:** Cross-agent communication tests with real goose instances.

**Key Deliverables:**
- E2E test framework (Python)
- 3 E2E scenarios:
  1. Expense Approval (Finance → Manager)
  2. Legal Review (Finance → Legal → Manager)
  3. Cross-Department (HR → Finance → Manager)
- Privacy isolation validation (each agent sees only what profile allows)
- Agent Mesh E2E tests (19 steps across 3 scenarios)

**Success Criteria:**
- [x] 3 E2E scenarios implemented
- [x] All scenarios passing (19/19 steps)
- [x] Privacy isolation validated (0 violations)
- [x] Audit logging working
- [x] Documentation complete

---

### Workstream V: Full Integration Validation (Week 5-6)
**Goal:** End-to-end testing of complete workflow.

**Key Deliverables:**
- Full workflow test (30 tests):
  - Admin setup (CSV import, profile assignment)
  - User onboarding (login, profile fetch, session creation)
  - Privacy Guard Proxy (interception, masking, audit)
  - Agent Mesh (discovery, messaging, privacy)
  - Session lifecycle (active, pause, resume, complete)
  - Data validation (integrity, audit, compliance)
- Performance testing (load test, benchmarks)
- Security audit (18 checks)

**Demo Workflow:**
1. Admin uploads CSV org chart (50 employees)
2. Admin assigns profiles to users
3. User installs goose → signs in → gets assigned profile
4. All LLM calls intercepted by Privacy Guard Proxy
5. Multi-agent collaboration (Finance ↔ Manager ↔ Legal)
6. Privacy boundaries enforced, all access logged

**Success Criteria:**
- [x] Full workflow test: 30/30 passing
- [x] Performance: All metrics within targets
- [x] Security audit: All checks passing
- [x] 50 test users successfully onboarded
- [x] 3 agents collaborating successfully
- [x] Documentation complete

---

## 📊 Phase 6 Deliverables Summary

### Code Deliverables (7 items)
1. Session Lifecycle Routes (`src/controller/src/routes/sessions.rs`)
2. Privacy Guard Proxy Service (`src/privacy-guard-proxy/`)
3. Docker goose Image (`docker/goose/Dockerfile`)
4. Multi-goose Compose Config (updated `ce.dev.yml`)
5. Agent Mesh Routes (`src/controller/src/routes/agent_mesh.rs`)
6. E2E Test Framework (`tests/e2e/framework/`)
7. Migration 0007 (`db/migrations/metadata-only/0007_update_sessions_for_lifecycle.sql`)

### Test Deliverables (7 test suites, 81+ tests)
1. Session Lifecycle Tests (8 tests)
2. Privacy Guard Proxy Tests (8 tests)
3. Multi-goose Tests (8 tests)
4. Agent Mesh E2E Tests (19 steps across 3 scenarios)
5. Full Integration Tests (30 tests)
6. Performance Tests (load testing, benchmarks)
7. Security Audit (18 checks)

### Documentation Deliverables (7 documents)
1. Updated STARTUP-GUIDE.md (multi-goose startup)
2. Updated SYSTEM-ARCHITECTURE-MAP.md (Privacy Guard Proxy)
3. Updated TESTING-GUIDE.md (E2E testing)
4. NEW: MULTI-GOOSE-SETUP.md (Docker goose setup)
5. NEW: PRIVACY-GUARD-PROXY.md (Proxy architecture)
6. NEW: AGENT-MESH-E2E.md (E2E scenarios)
7. NEW: PHASE-6-COMPLETION-SUMMARY.md (achievements)

---

## ⚠️ Critical Requirements (DO NOT SKIP)

### 1. All Parts Fully Integrated
- **NO gaps** between components
- **NO deferred integration** work
- **NO "we'll connect this later"**
- Every component must work with every other component

### 2. Agent Mesh E2E is Core Value
- Multi-agent communication is THE differentiator
- Must be proven working before UI
- 3 scenarios minimum (Finance ↔ Manager ↔ Legal)

### 3. Privacy Guard Proxy is Non-Negotiable
- ALL LLM calls MUST go through proxy
- PII masked before LLM, unmasked after
- No direct LLM calls allowed

### 4. Admin Assigns Profiles (Not Users)
- Admin uploads CSV org chart
- Admin assigns profiles to users
- Users get auto-configured on goose startup
- Users do NOT choose their own profiles

### 5. Testing is Mandatory
- Minimum 81 tests across all workstreams
- All tests must pass before phase complete
- No "we'll test this later"

### 6. Documentation Must Be Complete
- Every new component documented
- Every test procedure documented
- Every integration point documented
- Future agents must not get stuck

---

## 🚨 What NOT to Do

### ❌ DO NOT Start UI Work
- Admin UI deferred to Phase 7
- User portal deferred to Phase 7
- All frontend work deferred to Phase 7

### ❌ DO NOT Skip Testing
- Every deliverable must have tests
- Tests must pass before marking complete
- No "manual testing only"

### ❌ DO NOT Change Architecture Without Asking
- User has specific workflow in mind
- Privacy Guard Proxy architecture is decided
- Agent Mesh is core (not optional)

### ❌ DO NOT Assume Things Work
- Test every integration point
- Verify every service communicates correctly
- Don't assume Docker networking works without testing

### ❌ DO NOT Leave Gaps
- All components must be wired together
- No "TODO: integrate this" comments
- Complete integration before Phase 6 ends

---

## 📝 State Management (Bulletproof Logging)

### After Every Milestone, Update 3 Files:

#### 1. Phase-6-Agent-State.json
```json
{
  "phase": "6",
  "current_workstream": "A",
  "current_task": "A.1",
  "workstreams": {
    "A": {
      "name": "Lifecycle Integration",
      "status": "in_progress",
      "tasks_complete": ["A.1"],
      "tasks_in_progress": ["A.2"],
      "tasks_pending": ["A.3"]
    }
  },
  "last_updated": "2025-11-10T12:00:00Z",
  "last_agent": "agent-session-xyz"
}
```

#### 2. Phase-6-Checklist.md
Mark tasks complete with `[x]`:
```markdown
## Workstream A: Lifecycle Integration
- [x] Task A.1: Route Integration
  - [x] Create src/controller/src/routes/sessions.rs
  - [x] Wire endpoints into main.rs
  - [x] Unit tests passing
- [ ] Task A.2: Database Persistence
  - [ ] Migration 0007 created
  - [ ] SessionManager updated
```

#### 3. docs/tests/phase6-progress.md
Append timestamped entries:
```markdown
## 2025-11-10 12:00 - Task A.1 Complete

**Completed:**
- Created session routes (sessions.rs)
- Wired into Controller main.rs
- Unit tests: 5/5 passing

**Next:** Task A.2 - Database persistence

**Branch:** feature/phase6-lifecycle-routes
**Commit:** abc123def
```

---

## 🔄 Resume Protocol for New Agents

When a new agent starts working on Phase 6:

### Step 1: Read Context (in order)
1. `master-technical-project-plan.md` - Overall project
2. `docs/operations/SYSTEM-ARCHITECTURE-MAP.md` - Architecture
3. `docs/operations/COMPLETE-SYSTEM-REFERENCE.md` - Current state
4. `Phase-6-Agent-State.json` - Current progress
5. `Phase-6-Checklist.md` - What's done/pending
6. `docs/tests/phase6-progress.md` - Latest updates

### Step 2: Verify System State
```bash
# Check all services running
docker ps

# If not running, follow STARTUP-GUIDE.md
# Ask user to unseal Vault (3 of 5 keys required)
```

### Step 3: Ask User
```
I've read the Phase 6 context. Current progress:
- Workstream A: [status]
- Workstream B: [status]
- ...

Which workstream should I focus on?
```

### Step 4: Work on Assigned Workstream
- Follow task breakdown in this document
- Update state files after milestones
- Run tests frequently
- Ask user when stuck

---

## 🎯 Phase 6 Acceptance Criteria

Phase 6 is complete when:

1. ✅ All 5 workstreams complete (A, B, C, D, V)
2. ✅ All 81+ tests passing
3. ✅ Demo workflow operational (CSV → Profile → Multi-agent)
4. ✅ Privacy Guard Proxy intercepting all LLM calls
5. ✅ 3 goose agents collaborating via Agent Mesh
6. ✅ Security audit passing (all checks)
7. ✅ Documentation complete (7 new/updated docs)
8. ✅ Performance benchmarks published
9. ✅ User onboarding tested (50 test users)
10. ✅ Ready for Phase 7 (UI development)

---

## 🚀 Next Phase Preview

**Phase 7: Admin UI + User Experience**

Deferred items:
- Admin Dashboard (CSV upload UI, user management, audit viewer)
- User Portal (profile view, session history, privacy preferences)
- goose Desktop Integration (auto-sign-in, profile sync, collaboration panel)
- Full UX design and frontend development

**Phase 8+: Deployment & Hardening** (TBD)

Deferred items:
- Production deployment (Kubernetes, cloud infrastructure)
- Security hardening (secrets rotation, pentesting, compliance)
- Performance optimization (caching, query optimization, CDN)
- Improve Ollama NER (better PII detection models, custom training)
- Monitoring & alerting (Prometheus, Grafana, PagerDuty)
- Backup & disaster recovery
- Multi-tenant support

---

## 📚 Reference Documents

### Essential Reading (must read before starting)
- `docs/operations/STARTUP-GUIDE.md` - How to start all services
- `docs/operations/SYSTEM-ARCHITECTURE-MAP.md` - Where everything lives
- `docs/operations/TESTING-GUIDE.md` - How to run tests
- `docs/operations/COMPLETE-SYSTEM-REFERENCE.md` - Quick reference

### Phase Context
- `Technical Project Plan/master-technical-project-plan.md` - Overall plan
- `Technical Project Plan/PM Phases/Phase-0/` - Project inception
- `Technical Project Plan/PM Phases/Phase-3/` - Agent Mesh testing
- `Technical Project Plan/PM Phases/Phase-5/` - Privacy Guard development

### Product Requirements
- `docs/product/productdescription.md` - Product vision
- `docs/architecture/PHASE5-ARCHITECTURE.md` - Privacy Guard architecture

---

## ⚙️ Development Workflow

### For Each Task:

1. **Plan**
   - Read task description in this document
   - Review related code/docs
   - Ask user if architecture unclear

2. **Implement**
   - Write code following existing patterns
   - Add comprehensive comments
   - Follow Rust/Python best practices

3. **Test**
   - Write unit tests
   - Write integration tests
   - Run all tests (`./scripts/run-all-tests.sh`)

4. **Document**
   - Update relevant docs
   - Add code comments
   - Update TESTING-GUIDE.md if new tests

5. **Update State**
   - Update `Phase-6-Agent-State.json`
   - Mark tasks in `Phase-6-Checklist.md`
   - Append to `docs/tests/phase6-progress.md`

6. **Commit & Push**
   - Conventional commit message
   - Reference task ID (e.g., "feat(lifecycle): add session routes [A.1]")
   - Push to feature branch

7. **Ask User**
   - "Task A.1 complete. Tests passing. Ready for A.2?"

---

## 🛠️ Troubleshooting

### Services Won't Start
- Follow `docs/operations/STARTUP-GUIDE.md` exactly
- Check Vault unsealed (ask user for keys)
- Check logs: `docker logs ce_controller --tail 50`

### Tests Failing
- Read test output carefully
- Check service health: `docker ps`
- Check database: `docker exec ce_postgres psql -U postgres -d orchestrator -c "SELECT COUNT(*) FROM profiles;"`
- Ask user if stuck

### Can't Find Files
- Use `docs/operations/SYSTEM-ARCHITECTURE-MAP.md` as reference
- Use `rg` to search: `rg "SessionManager" src/`
- Ask user if file location unclear

### Architecture Questions
- **DO NOT assume** - ask user
- Reference existing code patterns
- Check ADRs in `docs/adr/`

---

## 📞 When to Ask User

### Always Ask When:
1. Architecture is unclear or ambiguous
2. Multiple implementation approaches possible
3. Tests failing and root cause unknown
4. Need secrets/credentials (Vault keys, API keys, etc.)
5. About to make breaking changes
6. Stuck for >15 minutes
7. Need clarification on product workflow

### Never Assume:
- How services should communicate
- What data to persist
- What privacy rules to apply
- How profiles should be assigned
- What UI should look like (deferred to Phase 7)

---

**END OF MAIN PROMPT**

---

**Next Steps:**
1. Read all context documents
2. Check Phase-6-Agent-State.json for current progress
3. Ask user which workstream to focus on
4. Update state files after every milestone
5. Run tests frequently
6. Ask when stuck

**Remember:** Integration-first, test everything, document thoroughly, ask when unclear.
