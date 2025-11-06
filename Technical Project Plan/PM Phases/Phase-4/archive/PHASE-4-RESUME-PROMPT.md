# Phase 4 Resume Prompt — Storage/Metadata + Session Persistence

**Purpose:** This document provides context for resuming Phase 4 work in a new session or after interruption.  
**Target:** AI orchestrator agent (Goose) or human developer  
**Phase:** 4 of 12 (Grant-aligned timeline)  
**Status:** Ready to begin

---

## 🔄 Quick Resume Block — Copy-Paste for New Sessions

```markdown
You are resuming Phase 4 orchestration for goose-org-twin.

**Context:**
- Phase: 4 — Storage/Metadata + Session Persistence (Medium - ~3-4 days)
- Repository: /home/papadoc/Gooseprojects/goose-org-twin

**Required Actions:**
1. Read state from: `Technical Project Plan/PM Phases/Phase-4/Phase-4-Agent-State.json`
2. Read last progress entry from: `docs/tests/phase4-progress.md` (if exists)
3. Re-read authoritative documents:
   - `Technical Project Plan/master-technical-project-plan.md` (Phase 4 section)
   - `Technical Project Plan/PM Phases/Phase-4/Phase-4-Checklist.md`
   - `Technical Project Plan/PM Phases/Phase-4/Phase-4-Orchestration-Prompt.md`
   - `Technical Project Plan/PM Phases/Phase-4/PHASE-4-RESUME-PROMPT.md` (this file)

**Summarize for me:**
- Current workstream and task from state JSON (A/B/C/D/E)
- Last step completed (from progress.md or state JSON)
- Checklist progress (X/15 tasks complete)
- Pending questions (if any in state JSON)
- Blockers (if any)

**Then proceed with:**
- If pending_questions exist: ask them and wait for my answers
- Otherwise: continue with the next unchecked task in the checklist
- Update state JSON and progress log after each task/milestone

**Guardrails:**
- Database ORM: sqlx (user decision)
- Session retention: 7 days (SESSION_RETENTION_DAYS env var)
- Idempotency TTL: 24 hours (IDEMPOTENCY_TTL_SECONDS env var)
- Update state JSON and progress log after each workstream checkpoint
- No secrets in git; .env.ce contains sensitive config (never commit)
```

---

## 🎯 Phase 4 Objectives

**Primary Goal:** Implement session persistence and idempotency deduplication to enable stateful cross-agent workflows.

**Success Criteria:**
1. ✅ Postgres schema deployed (4 tables: sessions, tasks, approvals, audit)
2. ✅ Session CRUD routes functional (POST/GET/PUT /sessions)
3. ✅ fetch_status tool returns real data (not 501)
4. ✅ Idempotency deduplication working (Redis-backed)
5. ✅ Integration tests 6/6 passing
6. ✅ Tagged release: v0.4.0

**Why This Phase Matters:**
- **Blocks Phase 5:** Directory/Policy needs session storage for approval workflows
- **Grant Milestone:** Q1 deliverable (Week 3-4, Month 3)
- **User-Facing Impact:** fetch_status tool finally works (no more 501 errors)
- **Production Readiness:** Idempotency prevents duplicate task execution

---

## 📊 User Decisions Applied

### Database ORM: sqlx
**Decision:** Use sqlx (compile-time checked SQL) instead of Diesel ORM  
**Rationale:**
- Simpler learning curve (raw SQL with safety)
- Async-first design (matches Axum)
- Compile-time query validation
- No heavy ORM runtime

### Session Retention: 7 Days
**Decision:** SESSION_RETENTION_DAYS=7 (default)  
**Rationale:**
- Balances audit needs with storage costs
- Configurable via environment variable
- Background cleanup deferred to Phase 7

### Idempotency TTL: 24 Hours
**Decision:** IDEMPOTENCY_TTL_SECONDS=86400 (24 hours)  
**Rationale:**
- Prevents stale duplicate detection (requests >24h apart are new)
- Redis memory efficient (auto-expiration)
- Configurable via environment variable

---

## 📚 Required Reading (Must Read Before Starting)

### Core Documents (Read in Order):
1. **Master Technical Plan** (project blueprint)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/master-technical-project-plan.md`
   - Section: "Phase 4: Storage/Metadata + Session Persistence"
   - Key info: Grant-aligned timeline, 12-month roadmap, success criteria

2. **Phase 4 Checklist** (task breakdown)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/PM Phases/Phase-4/Phase-4-Checklist.md`
   - What to do: Review all workstreams (A-E), understand dependencies

3. **Phase 4 Agent State** (progress tracker)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/PM Phases/Phase-4/Phase-4-Agent-State.json`
   - What to do: Check current_workstream, current_task, blockers

4. **Phase 3 Completion Summary** (what we built)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/PM Phases/Phase-3/Phase-3-Completion-Summary.md`
   - Why: Understand Controller API routes, Agent Mesh tools, current state

5. **Grant Application Analysis** (big picture)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/docs/grant/GRANT-APPLICATION-ANALYSIS.md`
   - Why: Understand MVP scope, grant milestones, Phase 4's role

### Database Schema References:
6. **Controller API OpenAPI Spec** (current routes)
   - URL: http://localhost:8088/api-docs/openapi.json
   - What to do: Understand existing POST /tasks/route, GET /sessions (501 placeholder)

7. **Privacy Guard MCP Integration** (data flows)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/docs/adr/0024-agent-mesh-python-implementation.md`
   - Why: Understand how sessions fit into cross-agent workflows

### Optional (If Deeper Context Needed):
8. **Product Description** (customer-first vision)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/docs/product/productdescription.md`

9. **Master Plan Revision Analysis** (lessons learned Phases 0-3)
   - Path: `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/MASTER-PLAN-REVISION-ANALYSIS.md`

---

## 🔄 Resuming from Interruption

### If Starting Phase 4 Fresh:
1. Read Phase-4-Agent-State.json → confirm `status: "NOT_STARTED"`
2. Read Phase-4-Checklist.md → start with Workstream A (Postgres Schema Design)
3. Create progress log: `docs/tests/phase4-progress.md`
4. Begin A1: Database Schema Design

### If Resuming Mid-Phase:
1. Read Phase-4-Agent-State.json → check:
   - `current_workstream` (e.g., "B")
   - `current_task` (e.g., "B2")
   - `blockers` (any issues preventing progress?)
2. Read last entry in `docs/tests/phase4-progress.md` → understand recent work
3. Continue from current_task (don't redo completed work)

### If Blocked or Stuck:
1. Check `blockers` array in Phase-4-Agent-State.json
2. Review dependencies: Postgres running? Redis installed?
3. Check previous phase: Phase 3 complete? Controller API working?
4. Ask user for clarification (don't assume or guess)

---

## 🛠️ Technical Context

### What Already Exists (Phase 3 Complete):
- ✅ Controller API (Rust/Axum): 5 routes, 21 unit tests
  - POST /tasks/route (task routing)
  - GET /sessions/{id} (returns 501 - TO BE IMPLEMENTED IN PHASE 4)
  - POST /sessions (returns 501 - TO BE IMPLEMENTED IN PHASE 4)
  - POST /approvals (approval handling)
  - GET /profiles/{role} (mock profiles)

- ✅ Agent Mesh MCP (Python): 4 tools, 977 lines code
  - send_task: Working
  - request_approval: Working
  - notify: Working
  - fetch_status: **Returns 501** (TO BE FIXED IN PHASE 4)

- ✅ Infrastructure:
  - Keycloak 26.0.4 (OIDC SSO)
  - Vault 1.18.3 (secrets management)
  - Postgres 15.x (database - no schema yet)
  - Ollama (local models for Privacy Guard)
  - **Redis: NOT YET ADDED** (Phase 4 dependency)

### What Needs to Be Built (Phase 4):
1. **Postgres Schema:** 4 tables (sessions, tasks, approvals, audit)
2. **Migrations:** Diesel or sqlx migrations for schema versioning
3. **Session Routes:** POST/GET/PUT /sessions (replace 501 with real data)
4. **fetch_status Tool:** Update to call real session API
5. **Redis Setup:** Add to docker-compose, configure connection
6. **Idempotency Middleware:** Redis-backed duplicate request detection

---

## 📂 File Structure (Where to Work)

### Rust Controller (src/controller/):
```
src/controller/
├── src/
│   ├── models/
│   │   └── session.rs (NEW - create Session struct)
│   ├── repository/
│   │   └── session_repo.rs (NEW - CRUD operations)
│   ├── routes/
│   │   └── sessions.rs (MODIFY - implement POST/GET/PUT)
│   ├── middleware/
│   │   └── idempotency.rs (NEW - Redis-backed deduplication)
│   ├── config.rs (MODIFY - add Postgres + Redis config)
│   └── main.rs (MODIFY - wire up new routes/middleware)
├── Cargo.toml (MODIFY - add diesel/sqlx, redis-rs)
└── migrations/ (NEW - Diesel/sqlx migrations)
    └── 001_create_sessions_tasks_approvals_audit/
        ├── up.sql
        └── down.sql
```

### Python Agent Mesh (src/agent-mesh/):
```
src/agent-mesh/
├── tools/
│   └── fetch_status.py (MODIFY - call real session API)
└── tests/
    └── test_integration.py (MODIFY - expect 200 not 501)
```

### Database Migrations (deploy/migrations/):
```
deploy/migrations/ (NEW directory)
└── 001_create_sessions_tasks_approvals_audit.sql
```

### Documentation (docs/):
```
docs/
├── database/
│   └── SCHEMA.md (NEW - document Postgres schema)
├── api/
│   └── SESSIONS.md (NEW - session routes API docs)
└── tests/
    └── phase4-progress.md (NEW - progress log)
```

---

## 🚦 Starting Checklist

Before beginning Workstream A, verify:

- [ ] Phase 3 complete and merged to main (check: `git log --oneline | grep v0.3.0`)
- [ ] Postgres running (check: `docker ps | grep postgres`)
- [ ] Controller API healthy (check: `curl http://localhost:8088/health`)
- [ ] Phase-4-Checklist.md read and understood
- [ ] Phase-4-Agent-State.json confirms `status: "NOT_STARTED"`
- [ ] Ready to create docs/tests/phase4-progress.md (progress log)

---

## 📝 Work Log Template

When resuming Phase 4, create/update progress log:

```markdown
# Phase 4 Progress Log — Storage/Metadata + Session Persistence

**Started:** YYYY-MM-DD HH:MM  
**Phase Status:** IN_PROGRESS  
**Current Workstream:** A (Postgres Schema Design)  
**Current Task:** A1 (Database Schema Design)

---

## Session 1: [Date] - Workstream A Start

**Timestamp:** YYYY-MM-DD HH:MM  
**Duration:** Xh  
**Tasks Completed:**
- [x] A1: Database schema design complete
- [x] sessions table: id (UUID), role (VARCHAR), task_id (UUID), status (VARCHAR), created_at, updated_at, metadata (JSONB)
- [x] tasks table: id, type, from_role, to_role, payload, trace_id, idempotency_key
- [x] Schema documented in docs/database/SCHEMA.md

**Blockers:** None  
**Next:** A2 (Database Migration Setup)

---
```

---

## 🎓 Key Concepts to Understand

### Session Lifecycle:
1. **pending**: Session created, task not yet routed
2. **active**: Task routed to agent, awaiting completion
3. **completed**: Task finished successfully
4. **failed**: Task failed with error
5. **expired**: Session older than retention period

### Idempotency Pattern:
```
Request 1: POST /tasks/route + Idempotency-Key: abc123
  → Check Redis: GET idempotency:abc123 (not found)
  → Process request, route task
  → Cache response: SET idempotency:abc123 {response} EX 86400
  → Return: HTTP 202 Accepted

Request 2: POST /tasks/route + Idempotency-Key: abc123 (duplicate)
  → Check Redis: GET idempotency:abc123 (found!)
  → Return cached response: HTTP 200 OK (same body as Request 1)
  → Do NOT process task again
```

### Database Migration Strategy:
- Use Diesel (ORM) or sqlx (raw SQL)
- Migrations versioned: 001, 002, 003...
- Each migration has up.sql (apply) and down.sql (rollback)
- Apply on startup: `diesel migration run` or `sqlx migrate run`

---

## 🚨 Common Pitfalls to Avoid

1. **Don't modify Phase 3 code unless necessary**
   - Phase 3 is complete and working
   - Only update GET /sessions and POST /sessions routes (currently return 501)

2. **Don't skip migrations**
   - Manual SQL changes are not tracked
   - Always use migration tool (Diesel or sqlx)

3. **Don't hardcode database URLs**
   - Use env vars: DATABASE_URL, REDIS_URL
   - Update .env.example with new vars

4. **Don't forget to update tests**
   - Integration tests expect 501 → update to expect 200
   - Add new unit tests for session CRUD

5. **Don't rush idempotency**
   - Test duplicate requests thoroughly
   - Verify TTL works (keys expire after 24 hours)

---

## 🎯 Success Indicators

### After Workstream A (Postgres Schema):
- [ ] `docker exec -it org-chart-goose-orchestrator-postgres-1 psql -U postgres -d orchestrator -c '\dt'`
  - Should show: sessions, tasks, approvals, audit tables

### After Workstream B (Session CRUD):
- [ ] `curl http://localhost:8088/sessions/test-123`
  - Should return: HTTP 200 (or 404 if not found), NOT 501

### After Workstream C (fetch_status):
- [ ] `cd src/agent-mesh && python -m pytest tests/test_integration.py -v`
  - Should show: 6/6 tests passing (not 5/6)

### After Workstream D (Idempotency):
- [ ] Send duplicate request with same Idempotency-Key
  - First request: HTTP 202
  - Second request: HTTP 200 (cached response)

---

## 📞 When to Ask for Help

**Ask user if:**
- Postgres schema design unclear (how many columns? what types?)
- Diesel vs sqlx choice needed (user preference?)
- Redis deployment topology (Docker Compose? Separate server?)
- Session retention period (7 days? 30 days?)
- Idempotency TTL (24 hours? 48 hours?)

**Don't ask about:**
- Basic Rust/Python syntax (use best judgment)
- Standard database types (UUID, TIMESTAMP, JSONB)
- Common middleware patterns (extract header, check cache, return response)

---

## ✅ Final Checkpoint (Workstream E)

Before marking Phase 4 complete:

1. **All workstreams A-D complete** (15/15 tasks)
2. **Integration tests passing** (6/6)
3. **Smoke tests updated** (if needed)
4. **Phase-4-Agent-State.json updated** (`status: "COMPLETE"`, `progress: 100%`)
5. **Phase-4-Completion-Summary.md created** (comprehensive summary)
6. **Git workflow complete** (commit, push, merge, tag v0.4.0)
7. **User confirmation** (review and approve before moving to Phase 5)

---

## 🚀 Ready to Begin?

**Next Action:**
1. Confirm you've read required documents (Master Plan, Checklist, Agent State, Phase 3 Summary)
2. Verify Postgres is running and accessible
3. Create docs/tests/phase4-progress.md (copy template above)
4. Begin Workstream A, Task A1: Database Schema Design

**User Confirmation Required:**
- Do you want to use Diesel ORM or sqlx for database access? (Recommend: sqlx for simplicity)
- Session retention period? (Recommend: 7 days default, configurable)
- Idempotency TTL? (Recommend: 24 hours)

**Let's build Phase 4!** 🎯
