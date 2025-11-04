# Master Orchestration Guide — Phases 2.5 & 3

**Date:** 2025-11-04  
**Project:** goose-org-twin  
**Phases:** 2.5 (Dependency Upgrades) + 3 (Controller API + Agent Mesh)

---

## 🎯 Quick Start

### Phase 2.5: Execute FIRST (~5.5 hours, same day)
```
Copy Phase-2.5-Orchestration-Prompt.md to new Goose session → Execute → Merge
```

### Phase 3: Execute AFTER Phase 2.5 (~8-9 days)
```
Copy Phase-3-Orchestration-Prompt.md to new Goose session → Execute → Merge
```

---

## 📁 Artifact Locations

### Phase 2.5 Artifacts
```
Technical Project Plan/PM Phases/Phase-2.5/
├── DEPENDENCY-RESEARCH.md           # Version research
├── Phase-2.5-Execution-Plan.md      # 4 workstreams
├── Phase-2.5-Checklist.md           # 19 tasks
├── Phase-2.5-Agent-State.json       # State tracking
└── Phase-2.5-Orchestration-Prompt.md # COPY THIS to new session
```

### Phase 3 Artifacts
```
Technical Project Plan/PM Phases/Phase-3/
├── Phase-3-Execution-Plan.md        # 3 workstreams
├── Phase-3-Checklist.md             # Tasks with estimates
├── Phase-3-Agent-State.json         # State tracking
└── Phase-3-Orchestration-Prompt.md  # COPY THIS to new session (after Phase 2.5)
```

---

## ⚙️ Execution Workflow

### 1. Phase 2.5 (Dependency Upgrades)

**Prerequisites:** Phases 0, 1, 1.2, 2, 2.2 complete ✅

**Goal:** Upgrade Keycloak, Vault, Postgres to latest LTS

**Steps:**
1. Open `Phase-2.5-Orchestration-Prompt.md`
2. Copy entire contents
3. Paste into NEW Goose session
4. Agent executes 4 workstreams:
   - A: Infrastructure Upgrade (2h)
   - B: Phase 1.2 Validation (1h)
   - C: Phase 2.2 Validation (1.5h)
   - D: Documentation (1h)
5. Review completion summary
6. Merge PR to main

**Duration:** ~5.5 hours (same day)

### 2. Phase 3 (Controller API + Agent Mesh)

**Prerequisites:** Phase 2.5 complete ✅

**Goal:** Multi-agent orchestration via Controller API + Python MCP extension

**Important:** Check `Phase-2.5/` folder for any changes from dependency upgrades before starting

**Steps:**
1. Open `Phase-3-Orchestration-Prompt.md`
2. Copy entire contents
3. Paste into NEW Goose session
4. Agent executes 3 workstreams:
   - A: Controller API (Rust, ~3 days)
   - B: Agent Mesh MCP (Python, ~4-5 days)
   - C: Cross-Agent Demo (~1 day)
5. Review completion summary
6. Merge PR to main

**Duration:** ~8-9 days

---

## 📊 Progress Tracking

### Phase 2.5
- State file: `Phase-2.5-Agent-State.json`
- Checklist: `Phase-2.5-Checklist.md`
- Progress log: `docs/tests/phase2.5-progress.md` (created during execution)

### Phase 3
- State file: `Phase-3-Agent-State.json`
- Checklist: `Phase-3-Checklist.md`
- Progress log: `docs/tests/phase3-progress.md` (created during execution)

---

## ✅ Success Criteria

### Phase 2.5
- ✅ All services upgraded (Keycloak 26.0.4, Vault 1.18.3, Postgres 17.2)
- ✅ Phase 1.2 tests pass (JWT auth)
- ✅ Phase 2.2 tests pass (Privacy Guard)
- ✅ ADR-0023 created
- ✅ No breaking changes

### Phase 3
- ✅ Controller API routes implemented (OpenAPI published)
- ✅ Agent Mesh MCP tools working (4 tools: send_task, request_approval, notify, fetch_status)
- ✅ Cross-agent approval demo successful
- ✅ ADR-0024 & ADR-0025 created
- ✅ Integration tests pass

---

## 🔗 Reference Documents

- **Master Plan:** `Technical Project Plan/master-technical-project-plan.md`
- **Phase 2.2 Summary:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`
- **Phase 3 Pre-Flight:** `Technical Project Plan/PM Phases/Phase-3-PRE-FLIGHT-ANALYSIS.md`
- **PROJECT_TODO:** Future work tracker
- **VERSION_PINS:** Current dependency versions

---

## 🚨 Important Notes

1. **Execute Phase 2.5 BEFORE Phase 3** (security CVEs in Keycloak)
2. **Check Phase-2.5 folder** before starting Phase 3 (for any upgrade changes)
3. **Use separate Goose sessions** for each phase (better context management)
4. **Update state JSON** after each task (enables session resume)
5. **Commit frequently** with conventional commits

---

**Prepared by:** Goose AI Agent  
**Date:** 2025-11-04  
**Status:** READY FOR USER REVIEW
