# Phase 4 — Storage/Metadata + Session Persistence

**Status:** 📋 READY TO BEGIN  
**Timeline:** Week 3-4 (1 week estimated)  
**Grant Milestone:** Q1 Month 3  
**Target Release:** v0.4.0  
**Depends On:** Phase 3 complete ✅

---

## Quick Start

### For AI Agent (goose):
1. Read: `PHASE-4-RESUME-PROMPT.md` (comprehensive context)
2. Read: `Phase-4-Checklist.md` (task breakdown)
3. Read: `Phase-4-Agent-State.json` (progress tracker)
4. Begin: Workstream A (Postgres Schema Design)

### For Human Developer:
1. Review master plan: `../../master-technical-project-plan.md`
2. Check Phase 3 completion: `../Phase-3/Phase-3-Completion-Summary.md`
3. Understand grant context: `../../../docs/grant/GRANT-APPLICATION-ANALYSIS.md`
4. Start with checklist: `Phase-4-Checklist.md`

---

## Phase 4 Objectives

**Primary Goal:** Enable stateful cross-agent workflows through session persistence and idempotency.

**Success Criteria:**
1. ✅ Postgres schema deployed (4 tables: sessions, tasks, approvals, audit)
2. ✅ Session CRUD routes functional (POST/GET/PUT /sessions)
3. ✅ fetch_status tool returns real data (not 501)
4. ✅ Idempotency deduplication working (Redis-backed)
5. ✅ Integration tests 6/6 passing
6. ✅ Tagged release: v0.4.0

**Why This Matters:**
- **Enables Phase 5:** Directory/Policy needs session storage for approval workflows
- **User-Facing:** fetch_status tool finally works (no more 501 errors)
- **Production-Ready:** Idempotency prevents duplicate task execution
- **Grant Milestone:** Q1 deliverable (part of grant application demo)

---

## Workstreams (15 Tasks Total)

### A. Postgres Schema Design (3 tasks, ~2 days)
- Database schema design (sessions, tasks, approvals, audit tables)
- Migration setup (Diesel or sqlx)
- Documentation (docs/database/SCHEMA.md)

### B. Session CRUD Operations (5 tasks, ~2 days)
- Session model + repository (Rust structs, CRUD functions)
- Controller session routes (POST/GET/PUT /sessions)
- Session lifecycle management (pending → active → completed/failed)
- Unit tests (session route testing)

### C. fetch_status Tool Completion (3 tasks, ~1 day)
- Update fetch_status tool (call real session API, not 501)
- Integration tests update (6/6 passing)

### D. Idempotency Deduplication (4 tasks, ~1 day)
- Redis setup (docker-compose, connection config)
- Idempotency middleware (Redis-backed duplicate detection)
- Test duplicate handling (same Idempotency-Key → cached response)

### E. Final Checkpoint (1 task, ~15 min)
- Update tracking documents (Agent State, Checklist, Progress Log)
- Create completion summary
- Git workflow (commit, push, merge, tag v0.4.0)

---

## Deliverables

### Code:
- [ ] `src/controller/src/models/session.rs` (Session model)
- [ ] `src/controller/src/repository/session_repo.rs` (CRUD operations)
- [ ] `src/controller/src/routes/sessions.rs` (updated routes)
- [ ] `src/controller/src/middleware/idempotency.rs` (Redis deduplication)
- [ ] `src/agent-mesh/tools/fetch_status.py` (updated tool)
- [ ] `deploy/migrations/001_create_sessions_tasks_approvals_audit.sql` (schema)

### Configuration:
- [ ] `docker-compose.yml` (add Redis service)
- [ ] `src/controller/src/config.rs` (Postgres + Redis config)
- [ ] `.env.example` (SESSION_RETENTION_DAYS, IDEMPOTENCY_TTL_SECONDS)

### Documentation:
- [ ] `docs/database/SCHEMA.md` (Postgres schema docs)
- [ ] `docs/api/SESSIONS.md` (session routes API docs)
- [ ] `docs/tests/phase4-progress.md` (progress log)
- [ ] `Phase-4-Completion-Summary.md` (final summary)

### Tests:
- [ ] `src/controller/src/routes/sessions_test.rs` (unit tests)
- [ ] `src/agent-mesh/tests/test_integration.py` (updated, 6/6 passing)
- [ ] `scripts/test-idempotency.sh` (manual testing script)

---

## Dependencies

### Required Before Phase 4:
- ✅ Phase 3 complete (Controller API, Agent Mesh MCP)
- ✅ Postgres 15.x running (docker-compose up)
- ✅ Keycloak + JWT auth working

### New Dependencies (Phase 4):
- Redis 7.x (for idempotency cache)
- Diesel ORM or sqlx (for Rust database access)
- Migration tool (diesel-cli or sqlx-cli)

### Rust Crates to Add:
```toml
# Cargo.toml additions
diesel = { version = "2.1", features = ["postgres", "r2d2", "uuid", "chrono"] }
# OR
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono"] }

redis = { version = "0.24", features = ["tokio-comp"] }
r2d2 = "0.8" # connection pooling
```

---

## Architecture Changes

### Before Phase 4:
```
Agent (Desktop) → Controller API → 501 Not Implemented
                ↓
          Agent Mesh MCP
          └── fetch_status → HTTP 501 (no session data)
```

### After Phase 4:
```
Agent (Desktop) → Controller API → Postgres (session data)
                ↓                 ↓
          Agent Mesh MCP      Redis (idempotency cache)
          └── fetch_status → HTTP 200 (real session data)
```

---

## Database Schema (Preview)

### sessions Table:
| Column      | Type      | Constraints          |
|-------------|-----------|----------------------|
| id          | UUID      | PRIMARY KEY          |
| role        | VARCHAR   | NOT NULL             |
| task_id     | UUID      | NOT NULL             |
| status      | VARCHAR   | NOT NULL             |
| created_at  | TIMESTAMP | DEFAULT NOW()        |
| updated_at  | TIMESTAMP | DEFAULT NOW()        |
| metadata    | JSONB     | DEFAULT '{}'         |

### tasks Table:
| Column           | Type      | Constraints          |
|------------------|-----------|----------------------|
| id               | UUID      | PRIMARY KEY          |
| type             | VARCHAR   | NOT NULL             |
| from_role        | VARCHAR   | NOT NULL             |
| to_role          | VARCHAR   | NOT NULL             |
| payload          | JSONB     | NOT NULL             |
| trace_id         | UUID      | NOT NULL             |
| idempotency_key  | VARCHAR   | UNIQUE               |
| created_at       | TIMESTAMP | DEFAULT NOW()        |

### approvals Table:
| Column        | Type      | Constraints          |
|---------------|-----------|----------------------|
| id            | UUID      | PRIMARY KEY          |
| task_id       | UUID      | FOREIGN KEY (tasks)  |
| approver_role | VARCHAR   | NOT NULL             |
| status        | VARCHAR   | NOT NULL             |
| decision_at   | TIMESTAMP | DEFAULT NOW()        |
| notes         | TEXT      | NULL                 |

### audit Table:
| Column      | Type      | Constraints          |
|-------------|-----------|----------------------|
| id          | UUID      | PRIMARY KEY          |
| event_type  | VARCHAR   | NOT NULL             |
| role        | VARCHAR   | NOT NULL             |
| timestamp   | TIMESTAMP | DEFAULT NOW()        |
| trace_id    | UUID      | NOT NULL             |
| metadata    | JSONB     | DEFAULT '{}'         |

---

## Progress Tracking

### Mandatory Checkpoints:
1. **After Workstream A:** Postgres schema deployed, migrations applied
2. **After Workstream B:** Session CRUD routes functional
3. **After Workstream C:** fetch_status tool working (not 501)
4. **After Workstream D:** Idempotency deduplication working
5. **After Workstream E:** Phase 4 complete, v0.4.0 tagged

### Update on Each Checkpoint:
- [ ] `Phase-4-Agent-State.json` (current progress, blockers)
- [ ] `Phase-4-Checklist.md` (mark tasks complete)
- [ ] `docs/tests/phase4-progress.md` (append timestamped entry)
- [ ] Commit changes to git (checkpoint commits)

---

## Next Phase Preview

**Phase 5:** Directory/Policy + Profiles + Simple UI  
**Timeline:** Week 5-6 (1 week estimated)  
**Depends On Phase 4:** Session storage, Postgres schema, audit index

**Phase 5 Workstreams:**
- A. Profile Bundle Format (YAML/JSON spec, signing)
- B. 5 Role Profiles (Finance, Manager, Engineering, Marketing, Support)
- C. RBAC/ABAC Policy Engine (can_use_tool, can_access_data)
- D. GET /profiles/{role} Implementation (real profile bundles)
- E. Simple Web UI (4 pages: Dashboard, Sessions, Profiles, Audit) 🎨 NEW!
- F. Progress Tracking (checkpoint)

**Phase 5 Deliverable:** Grant application ready (v0.5.0)

---

## Files in This Directory

- `README.md` (this file - quick reference)
- `PHASE-4-RESUME-PROMPT.md` (comprehensive context for AI agent)
- `Phase-4-Checklist.md` (task breakdown, 15 tasks)
- `Phase-4-Agent-State.json` (progress tracker, JSON format)

**To Be Created During Phase 4:**
- `Phase-4-Completion-Summary.md` (final summary, created in Workstream E)
- `docs/tests/phase4-progress.md` (progress log, created at start)

---

## Ready to Begin?

### Confirmation Checklist:
- [ ] Phase 3 complete and merged to main
- [ ] Postgres running (`docker ps | grep postgres`)
- [ ] Controller API healthy (`curl http://localhost:8088/health`)
- [ ] Read PHASE-4-RESUME-PROMPT.md
- [ ] Read Phase-4-Checklist.md
- [ ] Ready to create docs/tests/phase4-progress.md

### User Decisions Needed:
1. **Database ORM:** Diesel or sqlx? (Recommend: sqlx for simplicity)
2. **Session Retention:** 7 days? 30 days? (Recommend: 7 days default, configurable)
3. **Idempotency TTL:** 24 hours? 48 hours? (Recommend: 24 hours)

**Once confirmed, begin Workstream A: Postgres Schema Design!** 🚀
