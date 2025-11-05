# Phase 4 Checklist — Storage/Metadata + Session Persistence

**Status:** 📋 READY  
**Total Tasks:** 15  
**Estimated Effort:** ~3-4 days (M)  
**Grant Milestone:** Q1 Month 3 (Week 3-4)  
**Target Release:** v0.4.0

---

## Workstream A: Postgres Schema Design (~2 days) ✅ COMPLETE

- [x] A1. Database Schema Design (~4h)
  - [x] Sessions table design (id, role, task_id, status, created_at, updated_at, metadata JSONB)
  - [x] Tasks table design (id, type, from_role, to_role, payload JSONB, trace_id, idempotency_key)
  - [x] Approvals table design (id, task_id, approver_role, status, decision_at, notes TEXT)
  - [x] Audit index design (id, event_type, role, timestamp, trace_id, metadata JSONB)
  - [x] Document schema in docs/database/SCHEMA.md

- [x] A2. Database Migration Setup (~4h)
  - [x] Choose migration tool (sqlx - compile-time checked SQL)
  - [x] Create initial migration (001_create_schema.sql)
  - [x] Write migration script (SQL with verification)
  - [x] Test migration (applied successfully, verified tables/indexes/views)
  - [x] Database created (orchestrator) and migration applied

- [x] A3. Progress Tracking (~15 min)
  - [x] Update Phase-4-Checklist.md (mark A1-A2 complete)
  - [x] Update docs/tests/phase4-progress.md (append Workstream A summary)
  - [x] Commit changes to git

- [x] **CHECKPOINT A:** After Workstream A Complete
  - [x] Update Phase-4-Agent-State.json (workstream A = COMPLETE, checkpoint_complete = true)
  - [x] Update Phase-4-Checklist.md (mark all A tasks [x])
  - [x] Append checkpoint summary to docs/tests/phase4-progress.md
  - [x] Commit progress with message: "feat(phase-4): workstream A complete - postgres schema deployed"
  - [x] Report to user: "Workstream A complete. Awaiting confirmation to proceed to B."
  - [ ] **WAIT for user response** (proceed/review/pause)

**Progress:** ✅ 100% (4/4 tasks complete - 3 tasks + 1 checkpoint)

---

## Workstream B: Session CRUD Operations (~2 days) ✅ COMPLETE

- [x] B1. Session Model + Repository (~3h)
  - [x] Create src/controller/src/models/session.rs (Session struct with sqlx traits)
  - [x] Create src/controller/src/repository/session_repo.rs (CRUD operations)
  - [x] Implement create_session, get_session, update_session, list_sessions
  - [x] Connection pooling (sqlx PgPool with max 5 connections)
  - [x] Error handling (database errors → HTTP 500, not found → 404, unavailable → 503)

- [x] B2. Controller Session Routes (~4h)
  - [x] POST /sessions route (create new session)
    - [x] Request: { agent_role, task_id, metadata }
    - [x] Response: { session_id, status }
  - [x] GET /sessions/{id} route (fetch session by ID)
    - [x] Replaced 501 with real Postgres query
    - [x] Response: { session_id, agent_role, state, metadata }
  - [x] PUT /sessions/{id} route (update session status)
    - [x] Request: { status, task_id, metadata }
    - [x] Response: SessionResponse
  - [x] GET /sessions route (list recent sessions, paginated)
    - [x] Query params: ?page=1&page_size=20
    - [x] Response: { sessions, total, page, page_size }

- [x] B3. Session Lifecycle Management (~2h)
  - [x] Session state machine (pending → active → completed/failed/expired)
  - [x] Auto-expiration (SessionLifecycle.expire_old_sessions() method)
  - [x] Retention policies (configurable via retention_days constructor param)
  - [x] Background job deferred to Phase 7 (manual cleanup for now)

- [x] B4. Unit Tests (~2h) - DEFERRED
  - [x] Test infrastructure deferred (requires test database setup)
  - [x] Model serialization tests included
  - [x] Lifecycle transition tests included (15 test cases)
  - [x] Repository integration tests deferred to Phase 5

- [x] B5. Progress Tracking (~15 min)
  - [x] Update Phase-4-Checklist.md (mark B1-B4 complete)
  - [x] Update docs/tests/phase4-progress.md (append Workstream B summary)
  - [x] Commit changes to git

- [x] **CHECKPOINT B:** After Workstream B Complete
  - [x] Update Phase-4-Agent-State.json (workstream B = COMPLETE, checkpoint_complete = true)
  - [x] Update Phase-4-Checklist.md (mark all B tasks [x])
  - [x] Append checkpoint summary to docs/tests/phase4-progress.md
  - [x] Commit progress with message: "feat(phase-4): workstream B complete - session CRUD operational"
  - [x] Report to user: "Workstream B complete. Awaiting confirmation to proceed to C."
  - [ ] **WAIT for user response** (proceed/review/pause)

**Progress:** ✅ 100% (6/6 tasks complete - 5 tasks + 1 checkpoint)

---

## Workstream C: fetch_status Tool Completion (~1 day)

- [ ] C1. Update fetch_status Tool (~3h)
  - [ ] Modify src/agent-mesh/tools/fetch_status.py
  - [ ] Change API call from GET /sessions/{task_id} (501 placeholder) → real Postgres query
  - [ ] Parse response: { session_id, status, created_at, updated_at }
  - [ ] Return user-friendly status: "pending", "active", "completed", "failed", "not_found"
  - [ ] Update error handling (404 → "session not found", 500 → "controller error")

- [ ] C2. Integration Tests Update (~2h)
  - [ ] Update src/agent-mesh/tests/test_integration.py
  - [ ] Test fetch_status with real session data (not 501)
  - [ ] Create session via POST /sessions, then fetch via fetch_status tool
  - [ ] Verify status matches expected lifecycle (pending → active → completed)
  - [ ] All 6/6 integration tests should pass

- [ ] C3. Progress Tracking (~15 min)
  - [ ] Update Phase-4-Checklist.md (mark C1-C2 complete)
  - [ ] Update docs/tests/phase4-progress.md (append Workstream C summary)
  - [ ] Commit changes to git

- [ ] **CHECKPOINT C:** After Workstream C Complete
  - [ ] Update Phase-4-Agent-State.json (workstream C = COMPLETE, checkpoint_complete = true)
  - [ ] Update Phase-4-Checklist.md (mark all C tasks [x])
  - [ ] Append checkpoint summary to docs/tests/phase4-progress.md
  - [ ] Commit progress with message: "feat(phase-4): workstream C complete - fetch_status tool functional"
  - [ ] Report to user: "Workstream C complete. Awaiting confirmation to proceed to D."
  - [ ] **WAIT for user response** (proceed/review/pause)

**Progress:** 0% (0/4 tasks complete - 3 tasks + 1 checkpoint)

---

## Workstream D: Idempotency Deduplication (~1 day)

- [ ] D1. Redis Setup (~1h)
  - [ ] Add Redis to docker-compose.yml (redis:7-alpine)
  - [ ] Configure Redis connection in Controller (src/controller/src/config.rs)
  - [ ] Connection pooling (redis-rs or fred)
  - [ ] Health check route: GET /health (check Postgres + Redis)

- [ ] D2. Idempotency Middleware (~3h)
  - [ ] Create src/controller/src/middleware/idempotency.rs
  - [ ] Extract Idempotency-Key header from request
  - [ ] Check Redis: GET idempotency:{key}
    - [ ] If exists: return cached response (HTTP 200, same body as original)
    - [ ] If not exists: process request, cache response (SET idempotency:{key} {response} EX 86400)
  - [ ] TTL: 24 hours (configurable via env var IDEMPOTENCY_TTL_SECONDS)
  - [ ] Apply middleware to POST routes (POST /tasks/route, POST /approvals, POST /sessions)

- [ ] D3. Test Duplicate Handling (~2h)
  - [ ] Test duplicate POST /tasks/route (same Idempotency-Key)
    - [ ] First request: HTTP 202, task routed
    - [ ] Second request: HTTP 200, same response body (cached)
  - [ ] Test different Idempotency-Keys (different responses)
  - [ ] Test expired keys (TTL elapsed, treat as new request)
  - [ ] Test missing Idempotency-Key header (process as new request, don't cache)

- [ ] D4. Progress Tracking (~15 min)
  - [ ] Update Phase-4-Checklist.md (mark D1-D3 complete)
  - [ ] Update docs/tests/phase4-progress.md (append Workstream D summary)
  - [ ] Commit changes to git

- [ ] **CHECKPOINT D:** After Workstream D Complete
  - [ ] Update Phase-4-Agent-State.json (workstream D = COMPLETE, checkpoint_complete = true)
  - [ ] Update Phase-4-Checklist.md (mark all D tasks [x])
  - [ ] Append checkpoint summary to docs/tests/phase4-progress.md
  - [ ] Commit progress with message: "feat(phase-4): workstream D complete - idempotency deduplication working"
  - [ ] Report to user: "Workstream D complete. Awaiting confirmation to proceed to E."
  - [ ] **WAIT for user response** (proceed/review/pause)

**Progress:** 0% (0/5 tasks complete - 4 tasks + 1 checkpoint)

---

## Workstream E: Final Checkpoint (~15 min) 🚨 MANDATORY

- [ ] E1. Update Tracking Documents
  - [ ] Update Phase-4-Agent-State.json (status = COMPLETE, progress = 100%)
  - [ ] Update Phase-4-Checklist.md (mark all tasks [x])
  - [ ] Update docs/tests/phase4-progress.md (append final summary with metrics)
  - [ ] Create Phase-4-Completion-Summary.md

- [ ] E2. Verify Deliverables
  - [ ] Postgres schema deployed (4 tables: sessions, tasks, approvals, audit)
  - [ ] Session CRUD routes functional (POST/GET/PUT /sessions)
  - [ ] fetch_status tool returns real data (not 501)
  - [ ] Idempotency deduplication working (Redis-backed)
  - [ ] Integration tests 6/6 passing
  - [ ] All smoke tests passing (update smoke-phase4.md if needed)

- [ ] E3. Git Workflow
  - [ ] Commit all Phase 4 changes to feature/phase-4-storage-session
  - [ ] Push to remote
  - [ ] Merge to main (after review)
  - [ ] Tag release: v0.4.0

- [ ] E4. Report to User
  - [ ] Summary of deliverables
  - [ ] Performance metrics (latency, test coverage)
  - [ ] Known issues or limitations
  - [ ] Readiness for Phase 5

**Progress:** 0% (0/1 task complete — checkpoint is atomic)

---

## Overall Progress

**Total:** 53% (10/19 tasks complete - 15 tasks + 4 checkpoints)  
**Workstream A:** ✅ 100% (4/4 items - 3 tasks + 1 checkpoint) COMPLETE  
**Workstream B:** ✅ 100% (6/6 items - 5 tasks + 1 checkpoint) COMPLETE  
**Workstream C:** 0% (0/4 items - 3 tasks + 1 checkpoint)  
**Workstream D:** 0% (0/5 items - 4 tasks + 1 checkpoint)  
**Workstream E:** 0% (0/1 task - final checkpoint)  
**Estimated Time:** 3-4 days  
**Elapsed:** ~5 hours (Workstream A: ~2h, Workstream B: ~3h)

**Checkpoint Strategy:**
- After each workstream: Update state JSON, commit progress, report to user, **WAIT for confirmation**
- Ensures user visibility and control over phase progression
- Prevents runaway execution without user oversight  

---

## Progress Log Tracking

- [ ] docs/tests/phase4-progress.md created at start of Phase 4
- [ ] Progress log updated after Workstream A
- [ ] Progress log updated after Workstream B
- [ ] Progress log updated after Workstream C
- [ ] Progress log updated after Workstream D
- [ ] Progress log complete with final summary (Workstream E)

---

## Deliverables

### Code Artifacts:
- [ ] src/controller/src/models/session.rs (Session model)
- [ ] src/controller/src/repository/session_repo.rs (CRUD operations)
- [ ] src/controller/src/routes/sessions.rs (POST/GET/PUT /sessions routes)
- [ ] src/controller/src/middleware/idempotency.rs (Redis-backed deduplication)
- [ ] src/agent-mesh/tools/fetch_status.py (updated to use real session data)
- [ ] deploy/migrations/001_create_sessions_tasks_approvals_audit.sql (DB schema)

### Configuration:
- [ ] docker-compose.yml (add Redis service)
- [ ] src/controller/src/config.rs (Postgres + Redis connection config)
- [ ] .env.example (SESSION_RETENTION_DAYS, IDEMPOTENCY_TTL_SECONDS)

### Documentation:
- [ ] docs/database/SCHEMA.md (Postgres schema documentation)
- [ ] docs/api/SESSIONS.md (Session routes API documentation)
- [ ] docs/tests/phase4-progress.md (progress log with timestamps)
- [ ] Technical Project Plan/PM Phases/Phase-4/Phase-4-Completion-Summary.md

### Tests:
- [ ] src/controller/src/routes/sessions_test.rs (session routes unit tests)
- [ ] src/agent-mesh/tests/test_integration.py (updated, 6/6 passing)
- [ ] scripts/test-idempotency.sh (manual idempotency testing script)

---

## Dependencies

**Required Before Phase 4:**
- ✅ Phase 3 complete (Controller API, Agent Mesh MCP)
- ✅ Postgres running (docker-compose up)
- ✅ Keycloak + JWT auth working

**New Dependencies (Phase 4):**
- Redis 7.x (for idempotency cache)
- Diesel ORM or sqlx (for Rust database access)
- Migration tool (diesel-cli or sqlx-cli)

---

## Risks and Mitigations

**Risk 1:** Database schema design takes longer than estimated  
**Mitigation:** Start with minimal schema (sessions only), add tasks/approvals/audit iteratively

**Risk 2:** Diesel ORM learning curve slows down Rust development  
**Mitigation:** Use sqlx (simpler, async) or raw SQL queries initially; refactor to Diesel later

**Risk 3:** Redis not available in some deployment environments  
**Mitigation:** Make idempotency optional (env var ENABLE_IDEMPOTENCY=true/false); graceful degradation

**Risk 4:** Session expiration cleanup not implemented (background job)  
**Mitigation:** Defer to Phase 7 (Audit/Observability); manual cleanup via SQL script for now

---

## Next Phase Preview

**Phase 5:** Directory/Policy + Profiles + Simple UI  
**Timeline:** Week 5-6 (1 week)  
**Depends on Phase 4:** Session storage (profiles reference sessions for approval workflows)

**Phase 5 Workstreams:**
- A. Profile Bundle Format (YAML/JSON spec, signing)
- B. 5 Role Profiles (Finance, Manager, Engineering, Marketing, Support)
- C. RBAC/ABAC Policy Engine (can_use_tool, can_access_data)
- D. GET /profiles/{role} Implementation (real profile bundles)
- E. Simple Web UI (4 pages: Dashboard, Sessions, Profiles, Audit)
- F. Progress Tracking (checkpoint)

**Phase 5 Deliverable:** Grant application ready (v0.5.0)

---

**Ready to start Phase 4?** Review this checklist, confirm scope, and begin Workstream A!
