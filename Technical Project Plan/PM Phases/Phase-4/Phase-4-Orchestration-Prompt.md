# Phase 4 Orchestration Prompt

**Copy this entire prompt to a new goose session to execute Phase 4**

---

## 🔄 Quick Resume Block — Copy this if resuming Phase 4

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
   - `Technical Project Plan/PM Phases/Phase-4/Phase-4-Orchestration-Prompt.md` (this file)
   - `Technical Project Plan/PM Phases/Phase-4/PHASE-4-RESUME-PROMPT.md`

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
-Respect full integration testing and backward compatibility validation between phases, do not defer to future phases, stack must work end-to-end.
-When close to context window of current session limits, stop and update progress logs (/Docs/tests/) and update current phase state json, and checklist files( /PM Phases/Phase-x)
```

---

## 🚀 Master Orchestration Prompt — Copy this for new session

You are executing **Phase 4: Storage/Metadata + Session Persistence** for the goose-org-twin project.

## 📋 Context

**Project:** goose-org-twin (Multi-agent orchestration system)  
**Repository:** git@github.com:JEFH507/org-chart-goose-orchestrator.git  
**Current Branch:** main  
**Phase:** 4 (Storage/Metadata + Session Persistence - M2 Milestone)  
**Priority:** 🔴 HIGH (Unblocks Phase 5 - Directory/Policy)  
**Estimated Effort:** 3-4 days (M)

### Prerequisites (Must be Completed)
- ✅ Phase 0: Infrastructure bootstrap
- ✅ Phase 1: Basic Controller skeleton
- ✅ Phase 1.2: JWT verification middleware  
- ✅ Phase 2: Vault integration
- ✅ Phase 2.2: Privacy Guard with Ollama model
- ✅ Phase 2.5: Dependency upgrades
- ✅ Phase 3: Controller API + Agent Mesh (5 routes, 4 MCP tools, cross-agent demo)

### Blocks
- ⏸️ Phase 5: Directory/Policy + Profiles + Simple UI (waiting for this phase)

---

## 🎯 Objectives

### Primary Goal
Implement session persistence and idempotency deduplication to enable stateful cross-agent workflows.

### Success Criteria (v0.4.0)
- ✅ Postgres schema deployed (4 tables: sessions, tasks, approvals, audit)
- ✅ Session CRUD routes functional (POST/GET/PUT /sessions)
- ✅ fetch_status tool returns real data (not 501)
- ✅ Idempotency deduplication working (Redis-backed)
- ✅ Integration tests 6/6 passing
- ✅ Tagged release: v0.4.0

### Why This Phase Matters
- **Blocks Phase 5:** Directory/Policy needs session storage for approval workflows
- **Grant Milestone:** Q1 deliverable (Week 3-4, Month 3)
- **User-Facing Impact:** fetch_status tool finally works (no more 501 errors)
- **Production Readiness:** Idempotency prevents duplicate task execution

---

## 📦 User Decisions Applied

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

## 🔧 Execution Plan (4 Workstreams + Checkpoint)

### Workstream A: Postgres Schema Design (~2 days)

Execute in order:

**A1. Database Schema Design (~4h)**
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Create schema documentation
cat > docs/database/SCHEMA.md <<'EOF'
# Postgres Database Schema

## Tables

### sessions
- id (UUID, PK) - Session identifier
- role (VARCHAR(50), NOT NULL) - Agent role (Finance, Manager, etc.)
- task_id (UUID, FK → tasks.id, nullable) - Associated task
- status (VARCHAR(20), NOT NULL) - pending/active/completed/failed/expired
- created_at (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW())
- updated_at (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW())
- metadata (JSONB, default '{}') - Flexible context storage

**Indexes:**
- PRIMARY KEY (id)
- INDEX idx_sessions_task_id ON sessions(task_id)
- INDEX idx_sessions_status ON sessions(status)
- INDEX idx_sessions_created_at ON sessions(created_at DESC)

### tasks
- id (UUID, PK) - Task identifier
- task_type (VARCHAR(50), NOT NULL) - notification/approval/routing
- description (TEXT, NOT NULL) - Human-readable summary
- from_role (VARCHAR(50), NOT NULL) - Source agent role
- to_role (VARCHAR(50), NOT NULL) - Target agent role
- data (JSONB, default '{}') - Task payload
- trace_id (UUID, NOT NULL) - Distributed tracing ID
- idempotency_key (UUID, UNIQUE, NOT NULL) - Deduplication key
- created_at (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW())

**Indexes:**
- PRIMARY KEY (id)
- UNIQUE INDEX idx_tasks_idempotency_key ON tasks(idempotency_key)
- INDEX idx_tasks_trace_id ON tasks(trace_id)
- INDEX idx_tasks_created_at ON tasks(created_at DESC)

### approvals
- id (UUID, PK) - Approval identifier
- task_id (UUID, FK → tasks.id, NOT NULL) - Associated task
- approver_role (VARCHAR(50), NOT NULL) - Role that approved/rejected
- status (VARCHAR(20), NOT NULL) - pending/approved/rejected
- decision_at (TIMESTAMP WITH TIME ZONE, nullable) - When decision made
- notes (TEXT, default '') - Approval comments
- created_at (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW())

**Indexes:**
- PRIMARY KEY (id)
- INDEX idx_approvals_task_id ON approvals(task_id)
- INDEX idx_approvals_status ON approvals(status)

### audit_events
- id (UUID, PK) - Event identifier
- event_type (VARCHAR(50), NOT NULL) - task_routed/approval_requested/etc
- role (VARCHAR(50), NOT NULL) - Agent role that triggered event
- timestamp (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW())
- trace_id (UUID, NOT NULL) - Links to distributed trace
- metadata (JSONB, default '{}') - Event-specific data

**Indexes:**
- PRIMARY KEY (id)
- INDEX idx_audit_events_trace_id ON audit_events(trace_id)
- INDEX idx_audit_events_timestamp ON audit_events(timestamp DESC)
- INDEX idx_audit_events_event_type ON audit_events(event_type)
EOF

# Verify schema documented
wc -l docs/database/SCHEMA.md
```

**A2. Database Migration Setup (~4h)**
```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Create migrations directory
mkdir -p deploy/migrations

# Create initial migration
cat > deploy/migrations/001_create_schema.sql <<'EOF'
-- Create sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    role VARCHAR(50) NOT NULL,
    task_id UUID,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_sessions_task_id ON sessions(task_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);

-- Create tasks table
CREATE TABLE tasks (
    id UUID PRIMARY KEY,
    task_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    from_role VARCHAR(50) NOT NULL,
    to_role VARCHAR(50) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    trace_id UUID NOT NULL,
    idempotency_key UUID UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_tasks_idempotency_key ON tasks(idempotency_key);
CREATE INDEX idx_tasks_trace_id ON tasks(trace_id);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);

-- Create approvals table
CREATE TABLE approvals (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    approver_role VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    decision_at TIMESTAMP WITH TIME ZONE,
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_approvals_task_id ON approvals(task_id);
CREATE INDEX idx_approvals_status ON approvals(status);

-- Create audit_events table
CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    role VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    trace_id UUID NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_audit_events_trace_id ON audit_events(trace_id);
CREATE INDEX idx_audit_events_timestamp ON audit_events(timestamp DESC);
CREATE INDEX idx_audit_events_event_type ON audit_events(event_type);
EOF

# Run migration
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/orchestrator
sqlx migrate run --source deploy/migrations

# Verify tables created
psql $DATABASE_URL -c '\dt'
```

**A3. Progress Tracking (~15 min)**
```bash
# Update state JSON
cd "Technical Project Plan/PM Phases/Phase-4"

jq '.workstreams.A.tasks_completed = 2 |
    .workstreams.A.status = "COMPLETE" |
    .current_workstream = "B" |
    .progress.completed_tasks = 2 |
    .progress.percentage = ((2 / 15) * 100 | round)' \
  Phase-4-Agent-State.json > tmp.json && mv tmp.json Phase-4-Agent-State.json

# Append to progress log
cat >> docs/tests/phase4-progress.md <<'EOF'

### [YYYY-MM-DD HH:MM] - Workstream A: Postgres Schema Design (COMPLETE)

**Duration:** ~4 hours  
**Status:** ✅ COMPLETE  

#### Tasks Completed:
- [x] A1: Database Schema Design
- [x] A2: Database Migration Setup

#### Deliverables:
- ✅ docs/database/SCHEMA.md (4 tables documented)
- ✅ deploy/migrations/001_create_schema.sql (migration script)
- ✅ Postgres tables created (sessions, tasks, approvals, audit_events)

**Next:** Workstream B (Session CRUD Operations)
EOF

# Commit progress
git add "Technical Project Plan/PM Phases/Phase-4/" docs/tests/phase4-progress.md docs/database/ deploy/migrations/
git commit -m "feat(phase-4): workstream A complete - postgres schema deployed

- 4 tables created (sessions, tasks, approvals, audit_events)
- Indexes optimized for queries
- Migration script with sqlx
- Schema documentation complete

Progress: 2/15 tasks (13%)

Refs: #phase4 #workstream-a #checkpoint"
```

---

### ⚠️ CHECKPOINT 1: After Workstream A

**🛑 STOP HERE. Do not proceed to Workstream B until user confirms.**

**Before proceeding, complete ALL steps below:**

#### Step 1: Update State Files (~5 min)

```bash
cd "Technical Project Plan/PM Phases/Phase-4"

jq '.workstreams.A.status = "COMPLETE" |
    .workstreams.A.checkpoint_complete = true |
    .current_workstream = "B" |
    .pending_user_confirmation = true |
    .checkpoint_reason = "Workstream A complete - awaiting confirmation to proceed to B"' \
  Phase-4-Agent-State.json > tmp.json && mv tmp.json Phase-4-Agent-State.json
```

#### Step 2: Update Checklist (~2 min)

Open `Phase-4-Checklist.md` and mark Workstream A tasks `[x]`.

#### Step 3: Update Progress Log (~10 min)

Append to `docs/tests/phase4-progress.md` with Workstream A summary (see progress tracking command above).

#### Step 4: Commit Progress (~3 min)

Use commit command from A3 above.

#### Step 5: Report to User & WAIT (~2 min)

```
🎉 Workstream A (Postgres Schema Design) is COMPLETE ✅

**Summary:**
- ✅ 4 tables created (sessions, tasks, approvals, audit_events)
- ✅ Indexes optimized for performance
- ✅ Migration script deployed (sqlx)
- ✅ Schema documentation complete (docs/database/SCHEMA.md)

**Files Updated:**
- Phase-4-Agent-State.json (workstream A status = COMPLETE)
- Phase-4-Checklist.md (2/15 tasks = 13%)
- docs/tests/phase4-progress.md (Workstream A summary)

**Git:**
- Commit: [sha] feat(phase-4): workstream A complete

**Next: Workstream B (Session CRUD Operations) - ~2 days**
- B1: Session Model + Repository
- B2: Controller Session Routes (POST/GET/PUT /sessions)
- B3: Session Lifecycle Management
- B4: Unit Tests
- B5: Progress Tracking

---

**⏸️ WAITING FOR YOUR CONFIRMATION**

Type **"proceed"** to continue to Workstream B  
Type **"review"** to inspect files first  
Type **"pause"** to stop and save progress
```

**🛑 DO NOT PROCEED until user responds.**

---

### Workstream B: Session CRUD Operations (~2 days)

Execute in order:

**B1. Session Model + Repository (~3h)**
```bash
cd src/controller

# Add sqlx dependency to Cargo.toml
cat >> Cargo.toml <<'EOF'
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json"] }
EOF

# Create session model
cat > src/models/session.rs <<'EOF'
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub role: String,
    pub task_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(json)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub role: String,
    pub task_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub status: String,
    pub metadata: Option<serde_json::Value>,
}
EOF

# Create session repository
cat > src/repository/session_repo.rs <<'EOF'
use crate::models::session::{CreateSessionRequest, Session, UpdateSessionRequest};
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateSessionRequest) -> Result<Session> {
        let id = Uuid::new_v4();
        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, role, task_id, status, metadata)
            VALUES ($1, $2, $3, 'pending', $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.role)
        .bind(req.task_id)
        .bind(req.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn update(&self, id: Uuid, req: UpdateSessionRequest) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = $1, metadata = COALESCE($2, metadata), updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&req.status)
        .bind(req.metadata)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn list(&self, role: Option<String>, limit: i64, offset: i64) -> Result<Vec<Session>> {
        let sessions = if let Some(r) = role {
            sqlx::query_as::<_, Session>(
                r#"
                SELECT * FROM sessions
                WHERE role = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(&r)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Session>(
                r#"
                SELECT * FROM sessions
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(sessions)
    }
}
EOF

# Update main.rs to initialize Postgres pool
# (Add PgPool to AppState, create pool in main())
cargo build
```

**B2. Controller Session Routes (~4h)**
```bash
# Update src/routes/sessions.rs with real implementations
# POST /sessions → call session_repo.create()
# GET /sessions/{id} → call session_repo.get() (replace 501)
# PUT /sessions/{id} → call session_repo.update()
# GET /sessions → call session_repo.list() (replace empty array)

cargo build
cargo test
```

**B3. Session Lifecycle Management (~2h)**
```bash
# Create src/lifecycle/session_lifecycle.rs
# - State machine transitions (pending → active → completed/failed)
# - Auto-expiration query (mark sessions older than SESSION_RETENTION_DAYS as expired)
# - Background cleanup (optional, defer to Phase 7)

# Document lifecycle in docs/architecture/SESSION-LIFECYCLE.md
```

**B4. Unit Tests (~2h)**
```bash
# Create tests/session_crud_test.rs
# Test: POST /sessions (valid input, missing role, duplicate ID)
# Test: GET /sessions/{id} (exists, not found, invalid UUID)
# Test: PUT /sessions/{id} (valid transition, invalid status, not found)
# Test: GET /sessions (pagination, role filtering)

cargo test
```

**B5. Progress Tracking (~15 min)**
```bash
# Same pattern as A3: update state JSON, append to progress log, commit
```

---

### ⚠️ CHECKPOINT 2: After Workstream B

**🛑 STOP HERE. Do not proceed to Workstream C until user confirms.**

Same checkpoint procedure as Checkpoint 1.

**Report to user:**
```
🎉 Workstream B (Session CRUD Operations) is COMPLETE ✅

**Summary:**
- ✅ Session model + repository created (sqlx)
- ✅ POST /sessions route functional (create sessions)
- ✅ GET /sessions/{id} route functional (no more 501!)
- ✅ PUT /sessions/{id} route functional (update status)
- ✅ GET /sessions route functional (list with pagination)
- ✅ Unit tests passing (cargo test)

**Next: Workstream C (fetch_status Tool Completion)**
```

---

### Workstream C: fetch_status Tool Completion (~1 day)

**C1. Update fetch_status Tool (~3h)**
```bash
cd src/agent-mesh

# Modify tools/fetch_status.py
# Change: GET /sessions/{task_id} (currently returns 501)
# To: Real API call that parses session data from Postgres

# Update success response format to match Session model
# Update error handling (404 → "session not found", 500 → "database error")

python -m pytest tests/test_integration.py::test_fetch_status -v
```

**C2. Integration Tests Update (~2h)**
```bash
# Update tests/test_integration.py
# - Create session via POST /sessions
# - Call fetch_status with session ID
# - Verify status matches (pending → active → completed)
# - All 6/6 tests should pass (fetch_status was 5/6 before)

python -m pytest tests/test_integration.py -v
# Expected: 6/6 PASS
```

**C3. Progress Tracking (~15 min)**
```bash
# Update state JSON, append to progress log, commit
```

---

### ⚠️ CHECKPOINT 3: After Workstream C

**🛑 STOP HERE. Do not proceed to Workstream D until user confirms.**

**Report to user:**
```
🎉 Workstream C (fetch_status Tool Completion) is COMPLETE ✅

**Summary:**
- ✅ fetch_status tool updated (calls real session API)
- ✅ Integration tests updated (6/6 passing)
- ✅ No more 501 errors!
- ✅ Session lifecycle validated end-to-end

**Next: Workstream D (Idempotency Deduplication)**
```

---

### Workstream D: Idempotency Deduplication (~1 day)

**D1. Redis Setup (~1h)**
```bash
# Add Redis to docker-compose
cat >> deploy/compose/ce.dev.yml <<'EOF'
  redis:
    image: redis:7-alpine
    container_name: org-chart-goose-orchestrator-redis
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped
EOF

# Start Redis
docker compose -f deploy/compose/ce.dev.yml up -d redis

# Verify healthy
docker exec org-chart-goose-orchestrator-redis redis-cli ping
# Expected: PONG
```

**D2. Idempotency Middleware (~3h)**
```bash
cd src/controller

# Add redis dependency to Cargo.toml
cat >> Cargo.toml <<'EOF'
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
EOF

# Create src/middleware/idempotency.rs
# - Extract Idempotency-Key header
# - Check Redis: GET idempotency:{key}
# - If exists: return cached response (HTTP 200)
# - If not exists: process request, cache response (SET ... EX 86400)
# - Apply to POST /tasks/route, POST /approvals, POST /sessions

cargo build
```

**D3. Test Duplicate Handling (~2h)**
```bash
# Test duplicate POST /tasks/route with same Idempotency-Key
# - First request: HTTP 202, task routed
# - Second request: HTTP 200, same response body
# - Third request (different key): HTTP 202, new task

# Test expired key (TTL elapsed, treat as new request)
# Test missing key (process as new, don't cache)

# Document in docs/tests/test-idempotency.sh
```

**D4. Progress Tracking (~15 min)**
```bash
# Update state JSON, append to progress log, commit
```

---

### ⚠️ CHECKPOINT 4: After Workstream D

**🛑 STOP HERE. Do not proceed to Workstream E until user confirms.**

**Report to user:**
```
🎉 Workstream D (Idempotency Deduplication) is COMPLETE ✅

**Summary:**
- ✅ Redis deployed (docker-compose)
- ✅ Idempotency middleware functional
- ✅ Duplicate requests handled correctly (HTTP 200 cached)
- ✅ TTL enforced (24 hours)
- ✅ Integration tests updated

**Next: Workstream E (Final Checkpoint)**
```

---

### Workstream E: Final Checkpoint (~15 min) 🚨 MANDATORY

**E1. Update Tracking Documents**
```bash
# Update Phase-4-Agent-State.json (status = COMPLETE, progress = 100%)
cd "Technical Project Plan/PM Phases/Phase-4"

jq '.status = "COMPLETE" |
    .end_date = (now | strftime("%Y-%m-%d")) |
    .current_workstream = null |
    .workstreams.E.status = "COMPLETE" |
    .progress.completed_tasks = 15 |
    .progress.percentage = 100' \
  Phase-4-Agent-State.json > tmp.json && mv tmp.json Phase-4-Agent-State.json

# Update Phase-4-Checklist.md (mark all tasks [x])
# Update docs/tests/phase4-progress.md (append final summary)
```

**E2. Verify Deliverables**
```bash
# Checklist:
- [ ] Postgres schema deployed (4 tables)
- [ ] Session CRUD routes functional (POST/GET/PUT /sessions)
- [ ] fetch_status tool returns real data (not 501)
- [ ] Idempotency deduplication working (Redis-backed)
- [ ] Integration tests 6/6 passing
- [ ] All smoke tests passing
```

**E3. Git Workflow**
```bash
# Commit all Phase 4 changes
git add .
git commit -m "feat(phase-4): storage/metadata + session persistence [COMPLETE]

Summary:
- Postgres schema deployed (4 tables: sessions, tasks, approvals, audit_events)
- Session CRUD routes functional (POST/GET/PUT /sessions)
- fetch_status tool updated (no more 501 errors)
- Idempotency deduplication working (Redis-backed, 24h TTL)
- Integration tests: 6/6 passing
- Session retention: 7 days (configurable)

Phase 4 complete. Unblocks Phase 5 (Directory/Policy).

Refs: #phase4 #complete"

# Merge to main
git checkout main
git merge --squash feature/phase-4-storage-session
git commit -m "feat(phase-4): storage/metadata + session persistence [COMPLETE]"
git push origin main

# Tag release
git tag -a v0.4.0 -m "Phase 4: Storage/Metadata + Session Persistence"
git push origin v0.4.0
```

**E4. Create Completion Summary**
```bash
# Create Phase-4-Completion-Summary.md
# - Executive summary
# - Workstream-by-workstream achievements
# - Technical decisions (sqlx, 7 days, 24h TTL)
# - Database schema
# - Git commit history
# - Phase 5 readiness
```

**E5. Report to User**
```
🎉🎉🎉 Phase 4 (Storage/Metadata + Session Persistence) is COMPLETE ✅

**Summary:**
✅ **All 4 workstreams complete** (A, B, C, D, E)  
✅ **All 15 tasks complete** (100%)  
✅ **All deliverables created**

**Postgres Schema:**
- ✅ 4 tables deployed (sessions, tasks, approvals, audit_events)
- ✅ Indexes optimized for performance
- ✅ Migration script with sqlx

**Session CRUD:**
- ✅ POST /sessions functional (create sessions)
- ✅ GET /sessions/{id} functional (no more 501!)
- ✅ PUT /sessions/{id} functional (update status)
- ✅ GET /sessions functional (list with pagination)
- ✅ Unit tests: ALL PASS (cargo test)

**fetch_status Tool:**
- ✅ Updated to use real session data
- ✅ Integration tests: 6/6 PASS
- ✅ No more 501 errors!

**Idempotency Deduplication:**
- ✅ Redis deployed (docker-compose)
- ✅ Middleware functional (24h TTL)
- ✅ Duplicate requests handled correctly

**Documentation:**
- ✅ Schema documentation (docs/database/SCHEMA.md)
- ✅ Progress log complete (docs/tests/phase4-progress.md)
- ✅ Completion summary created

**Git Status:**
- ✅ All commits merged to main
- ✅ Tagged release: v0.4.0
- ✅ Pushed to GitHub

---

**Phase 4 COMPLETE. Ready for Phase 5 (Directory/Policy + Profiles + Simple UI).** ✅

**Would you like me to:**
1. Review Phase 4 completion summary
2. Begin Phase 5 preparation
3. Other action?
```

---

## 📝 Progress Tracking Commands

### After Each Task
```bash
# Update state JSON
jq '.workstreams.<WORKSTREAM>.tasks_completed += 1 |
    .progress.completed_tasks += 1 |
    .progress.percentage = ((.progress.completed_tasks / .progress.total_tasks) * 100 | round)' \
  Phase-4-Agent-State.json > tmp.json && mv tmp.json Phase-4-Agent-State.json

# Append to progress log
cat >> docs/tests/phase4-progress.md <<'EOF'

### [YYYY-MM-DD HH:MM] - Task <TASK_ID> Complete

**Task:** <Task description>  
**Duration:** ~Xh  
**Status:** ✅ COMPLETE  

**Deliverables:**
- <file1>
- <file2>

**Next:** <next_task>
EOF

# Commit
git add "Technical Project Plan/PM Phases/Phase-4/" docs/tests/phase4-progress.md
git commit -m "feat(phase-4): task <TASK_ID> complete - <brief summary>"
```

### After Each Checkpoint
```bash
# Update state JSON (mark checkpoint complete, set pending_user_confirmation)
jq '.workstreams.<WORKSTREAM>.checkpoint_complete = true |
    .pending_user_confirmation = true |
    .checkpoint_reason = "Workstream <X> complete - awaiting confirmation"' \
  Phase-4-Agent-State.json > tmp.json && mv tmp.json Phase-4-Agent-State.json

# Append checkpoint summary to progress log
# Commit
# Report to user and WAIT for response
```

---

## ✅ Completion Checklist

Before marking Phase 4 complete:

### Workstream A: Postgres Schema
- [ ] 4 tables deployed (sessions, tasks, approvals, audit_events)
- [ ] Indexes created
- [ ] Migration script tested (apply + rollback)
- [ ] Schema documented (docs/database/SCHEMA.md)

### Workstream B: Session CRUD
- [ ] Session model + repository created
- [ ] POST /sessions functional
- [ ] GET /sessions/{id} functional (no more 501)
- [ ] PUT /sessions/{id} functional
- [ ] GET /sessions functional (pagination)
- [ ] Unit tests pass (cargo test)

### Workstream C: fetch_status Tool
- [ ] Tool updated to use real session data
- [ ] Integration tests updated
- [ ] 6/6 tests passing
- [ ] No more 501 errors

### Workstream D: Idempotency
- [ ] Redis deployed (docker-compose)
- [ ] Middleware functional
- [ ] Duplicate requests handled (HTTP 200 cached)
- [ ] TTL enforced (24 hours)
- [ ] Integration tests updated

### Workstream E: Final Checkpoint
- [ ] Phase-4-Agent-State.json status=COMPLETE
- [ ] Phase-4-Checklist.md 15/15 tasks complete
- [ ] docs/tests/phase4-progress.md complete with final summary
- [ ] Phase-4-Completion-Summary.md created
- [ ] All commits merged to main
- [ ] Tagged release: v0.4.0

---

## 📚 Reference Documents

### Execution Artifacts
- **Checklist:** `Technical Project Plan/PM Phases/Phase-4/Phase-4-Checklist.md`
- **State Tracking:** `Technical Project Plan/PM Phases/Phase-4/Phase-4-Agent-State.json`
- **Resume Prompt:** `Technical Project Plan/PM Phases/Phase-4/PHASE-4-RESUME-PROMPT.md`

### Phase 3 Reference
- **Phase 3 Completion:** `Technical Project Plan/PM Phases/Phase-3/Phase-3-Completion-Summary.md`
- **Controller API:** `src/controller/src/routes/sessions.rs` (currently returns 501)
- **Agent Mesh:** `src/agent-mesh/tools/fetch_status.py` (currently fails with 501)

### Database
- **sqlx Documentation:** https://docs.rs/sqlx/latest/sqlx/
- **Postgres Documentation:** https://www.postgresql.org/docs/15/

---

## 🎯 Success Criteria (Final Check)

At the end of Phase 4, confirm:

- ✅ **Postgres:** 4 tables deployed, indexed, documented
- ✅ **Session CRUD:** POST/GET/PUT /sessions functional, unit tests pass
- ✅ **fetch_status:** Returns real data (not 501), integration tests 6/6 pass
- ✅ **Idempotency:** Redis-backed deduplication working, TTL enforced
- ✅ **Testing:** Unit tests pass, integration tests pass, smoke tests pass
- ✅ **Documentation:** Schema docs, progress log, completion summary
- ✅ **Git:** All commits merged, tagged v0.4.0

**If all ✅, Phase 4 is COMPLETE.** Proceed to Phase 5 (Directory/Policy + Profiles + Simple UI).

---

**Orchestrated by:** goose AI Agent  
**Execution Time:** ~3-4 days  
**Next Phase:** Phase 5 (after Phase 4 complete)
