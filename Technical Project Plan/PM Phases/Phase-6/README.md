# Phase 6: Backend Integration & Multi-Agent Testing

**VERSION:** 2.0 (Restructured 2025-11-10)  
**STATUS:** Ready to Start  
**TIMELINE:** 4-6 weeks  
**TARGET:** MVP-ready backend integration (v0.6.0)

---

## 📁 Phase 6 Directory Structure

```
Technical Project Plan/PM Phases/Phase-6/
├── README.md                          ← You are here (navigation & overview)
├── PHASE-6-MAIN-PROMPT.md             ← Copy-paste prompt for new agents
├── PHASE-6-RESUME-PROMPT.md           ← Resume prompt for returning agents
├── Phase-6-Agent-State.json           ← Current progress tracking (JSON)
├── Phase-6-Checklist.md               ← Comprehensive task checklist
├── Archive-Old-Plan/                  ← Old Phase 6 plan (UI-first approach, deprecated)
│   ├── ARCHITECTURE-ALIGNED-RECOMMENDATIONS.md
│   ├── DECISION-SUMMARY.md
│   ├── DECISION-TREE.md
│   ├── PHASE-6-DECISION-DOCUMENT.md
│   ├── Phase-6-Orchestrator-Prompt.md
│   ├── QUICK-START.md
│   ├── README.md
│   └── RESUME-A5-BUG-FIX.md
└── Archive/                           ← Even older archived files
    ├── ARCHITECTURE-CLARIFICATION.md
    ├── Phase-6-Checklist.md
    ├── Phase-6-Orchestrator-Prompt-FINAL.md
    ├── Phase-6-Orchestrator-Prompt.md
    ├── QUESTIONS-ANSWERED.md
    └── REVISED-SCOPE.md
```

---

## 🎯 Quick Start for New Agents

### 1. **Read Phase Context (in order):**
1. `Technical Project Plan/master-technical-project-plan.md` - Overall project plan
2. `docs/operations/COMPLETE-SYSTEM-REFERENCE.md` - Current system state
3. `docs/operations/SYSTEM-ARCHITECTURE-MAP.md` - Architecture details
4. `PHASE-6-MAIN-PROMPT.md` - This phase's goals and strategy
5. `Phase-6-Agent-State.json` - Current progress
6. `Phase-6-Checklist.md` - Task breakdown
7. `docs/tests/phase6-progress.md` - Detailed progress log

### 2. **Verify System State:**
```bash
# Check all services running
docker ps

# If not, follow docs/operations/STARTUP-GUIDE.md
# Ask user to unseal Vault (3 of 5 keys required)
```

### 3. **Ask User Which Workstream:**
```
I've read the Phase 6 context. Current progress:
- Workstream A (Lifecycle): Not started
- Workstream B (Privacy Proxy): Not started
- Workstream C (Multi-Goose): Not started
- Workstream D (Agent Mesh E2E): Not started
- Workstream V (Full Validation): Not started

Which workstream should I focus on?
```

---

## 📊 Phase 6 Overview

### Strategy
**Integration-first approach** - ALL backend components must work together BEFORE any UI work.

### Critical Requirements (User-Defined)
1. ✅ Admin assigns profiles (users do NOT choose their roles)
2. ✅ Privacy Guard Proxy intercepts ALL LLM calls (mask → LLM → unmask)
3. ✅ Agent Mesh E2E is core value (Finance ↔ Manager ↔ Legal)
4. ✅ No UI work until Phase 7 (backend must be proven first)
5. ✅ All 81+ tests must pass before complete

### 5 Workstreams

#### **A. Lifecycle Integration** (Week 1-2)
Wire Lifecycle module into Controller routes, enable session FSM.

**Key Deliverables:**
- Session endpoints (POST /sessions, PUT /sessions/{id}/events, etc.)
- Migration 0007 (update sessions table for FSM)
- Session lifecycle tests (8 tests)

**Dependencies:** None (Lifecycle module already complete from Phase 5)

---

#### **B. Privacy Guard Proxy** (Week 2-3)
Build HTTP proxy to intercept ALL LLM calls for PII masking/unmasking.

**Key Deliverables:**
- Privacy Guard Proxy service (Rust/Axum, port 8090)
- PII masking before LLM, unmasking after LLM
- All 8 profile YAMLs updated (api_base: http://privacy-guard-proxy:8090/v1)
- Proxy tests (8 tests)

**Architecture:**
```
Goose Agent → Privacy Guard Proxy → Mask PII → LLM (OpenRouter)
                     ↓                               ↓
              Privacy Guard (8089)            Response
                     ↑                               ↓
                Unmask PII ← Privacy Guard Proxy ←─┘
```

**Dependencies:** None

---

#### **C. Multi-Goose Test Environment** (Week 3-4)
Set up Docker Goose containers for testing 3+ agents simultaneously.

**Key Deliverables:**
- Docker Goose image (Dockerfile, config scripts)
- 3 Goose containers in ce.dev.yml (Finance, Manager, Legal)
- Agent Mesh configuration for multi-Goose
- Multi-Goose tests (8 tests)

**Architecture:**
```
Docker Network: goose-orchestrator-network
├─ goose-finance (container) → Finance profile
├─ goose-manager (container) → Manager profile
├─ goose-legal (container) → Legal profile
└─ Controller (routes messages between agents)
```

**Dependencies:** Workstream A (Lifecycle Integration)

---

#### **D. Agent Mesh E2E Testing** (Week 4-5)
Cross-agent communication tests with real Goose instances.

**Key Deliverables:**
- E2E test framework (Python)
- 3 E2E scenarios (Expense Approval, Legal Review, Cross-Department)
- Privacy isolation validation (each agent sees only what profile allows)
- Agent Mesh E2E tests (19 steps across 3 scenarios)

**Test Scenarios:**
1. **Expense Approval:** Finance → Manager (PII masked for Manager)
2. **Legal Review:** Finance → Legal → Manager (attorney-client privilege)
3. **Cross-Department:** HR → Finance → Manager (role-based access)

**Dependencies:** Workstream C (Multi-Goose Environment)

---

#### **V. Full Integration Validation** (Week 5-6)
End-to-end testing of complete workflow.

**Key Deliverables:**
- Full workflow test (30 tests: admin setup, user onboarding, privacy proxy, agent mesh, session lifecycle, data validation)
- Performance testing (load test, benchmarks)
- Security audit (18 checks)

**Demo Workflow:**
1. Admin uploads CSV org chart (50 employees)
2. Admin assigns profiles to users
3. User installs Goose → signs in → gets assigned profile
4. All LLM calls intercepted by Privacy Guard Proxy
5. Multi-agent collaboration (Finance ↔ Manager ↔ Legal)
6. Privacy boundaries enforced, all access logged

**Dependencies:** All other workstreams (A, B, C, D)

---

## 📋 Deliverables Summary

### Code (7 items)
1. Session Lifecycle Routes (`src/controller/src/routes/sessions.rs`)
2. Privacy Guard Proxy Service (`src/privacy-guard-proxy/`)
3. Docker Goose Image (`docker/goose/Dockerfile`)
4. Multi-Goose Compose Config (updated `ce.dev.yml`)
5. Agent Mesh Routes (`src/controller/src/routes/agent_mesh.rs`)
6. E2E Test Framework (`tests/e2e/framework/`)
7. Migration 0007 (`db/migrations/metadata-only/0007_update_sessions_for_lifecycle.sql`)

### Tests (7 suites, 81+ tests)
1. Session Lifecycle Tests (8 tests)
2. Privacy Guard Proxy Tests (8 tests)
3. Multi-Goose Tests (8 tests)
4. Agent Mesh E2E Tests (19 steps across 3 scenarios)
5. Full Integration Tests (30 tests)
6. Performance Tests (load testing, benchmarks)
7. Security Audit (18 checks)

### Documentation (7 documents)
1. Updated STARTUP-GUIDE.md
2. Updated SYSTEM-ARCHITECTURE-MAP.md
3. Updated TESTING-GUIDE.md
4. NEW: MULTI-GOOSE-SETUP.md
5. NEW: PRIVACY-GUARD-PROXY.md
6. NEW: AGENT-MESH-E2E.md
7. NEW: PHASE-6-COMPLETION-SUMMARY.md

---

## ✅ Acceptance Criteria

Phase 6 is complete when:

1. ✅ All 5 workstreams complete (A, B, C, D, V)
2. ✅ All 81+ tests passing
3. ✅ Demo workflow operational (CSV → Profile → Multi-agent)
4. ✅ Privacy Guard Proxy intercepting all LLM calls
5. ✅ 3 Goose agents collaborating via Agent Mesh
6. ✅ Security audit passing (all checks)
7. ✅ Documentation complete (7 new/updated docs)
8. ✅ Performance benchmarks published
9. ✅ User onboarding tested (50 test users)
10. ✅ Ready for Phase 7 (UI development)

---

## 🚀 What Gets Deferred to Phase 7

**All UI Work:**
- Admin Dashboard (CSV upload UI, user management, audit viewer)
- User Portal (profile view, session history, privacy preferences)
- Goose Desktop Integration (auto-sign-in, profile sync, collaboration panel)
- Full UX design and frontend development

**Also Deferred to Phase 8+:**
- Production deployment (Kubernetes, cloud infrastructure)
- Security hardening (secrets rotation, pentesting)
- Performance optimization (caching, query optimization)
- Improve Ollama NER (better PII detection models)
- Monitoring & alerting (Prometheus, Grafana)
- Backup & disaster recovery
- Multi-tenant support

---

## 📖 For More Details

- **Main Prompt:** `PHASE-6-MAIN-PROMPT.md` (comprehensive task breakdown)
- **Resume Prompt:** `PHASE-6-RESUME-PROMPT.md` (for new sessions)
- **State Tracking:** `Phase-6-Agent-State.json` (current progress)
- **Task Checklist:** `Phase-6-Checklist.md` (detailed tasks)
- **Progress Log:** `docs/tests/phase6-progress.md` (timestamped updates)
- **Master Plan:** `Technical Project Plan/master-technical-project-plan.md` (overall project)

---

## ⚠️ Critical Notes for Agents

### DO:
- ✅ Update `Phase-6-Agent-State.json` after EVERY milestone
- ✅ Mark tasks in `Phase-6-Checklist.md` as complete
- ✅ Append to `docs/tests/phase6-progress.md` with timestamps
- ✅ Run tests after significant changes
- ✅ Ask user when stuck or unclear

### DON'T:
- ❌ Start UI work (deferred to Phase 7)
- ❌ Skip testing (every deliverable needs tests)
- ❌ Change architecture without asking user
- ❌ Assume things work without verification
- ❌ Leave gaps in integration

---

**Last Updated:** 2025-11-10  
**Status:** Ready to start - awaiting user confirmation on which workstream to begin  
**Recommended Start:** Workstream A (Lifecycle Integration) - no dependencies
