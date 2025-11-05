# Postgres Database Schema

## Overview

This document describes the database schema for the goose-org-twin orchestration system.
The schema supports session persistence, task routing, approval workflows, and audit logging.

**Design Decisions:**
- **ORM Choice:** sqlx (compile-time checked SQL, async-first)
- **Session Retention:** 7 days (configurable via `SESSION_RETENTION_DAYS`)
- **Idempotency TTL:** 24 hours (configurable via `IDEMPOTENCY_TTL_SECONDS`)

---

## Tables

### sessions

Stores agent session state for cross-agent workflows.

**Columns:**
- `id` (UUID, PRIMARY KEY) - Session identifier
- `role` (VARCHAR(50), NOT NULL) - Agent role (Finance, Manager, etc.)
- `task_id` (UUID, nullable) - Associated task (FK → tasks.id)
- `status` (VARCHAR(20), NOT NULL) - Session status
  - Valid values: `pending`, `active`, `completed`, `failed`, `expired`
- `created_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW()) - Creation timestamp
- `updated_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW()) - Last update timestamp
- `metadata` (JSONB, NOT NULL, default '{}') - Flexible context storage

**Indexes:**
- `PRIMARY KEY (id)` - Primary key index
- `idx_sessions_task_id ON sessions(task_id)` - Foreign key lookups
- `idx_sessions_status ON sessions(status)` - Status filtering
- `idx_sessions_created_at ON sessions(created_at DESC)` - Time-based queries (pagination, cleanup)

**Notes:**
- `metadata` JSONB column allows flexible storage without schema migrations
- `updated_at` auto-updates via application logic (not database trigger)
- Sessions older than `SESSION_RETENTION_DAYS` can be marked as `expired` (cleanup deferred to Phase 7)

---

### tasks

Stores cross-agent task routing information.

**Columns:**
- `id` (UUID, PRIMARY KEY) - Task identifier
- `task_type` (VARCHAR(50), NOT NULL) - Task category
  - Valid values: `notification`, `approval`, `routing`
- `description` (TEXT, NOT NULL) - Human-readable summary
- `from_role` (VARCHAR(50), NOT NULL) - Source agent role
- `to_role` (VARCHAR(50), NOT NULL) - Target agent role
- `data` (JSONB, NOT NULL, default '{}') - Task payload (arbitrary JSON)
- `trace_id` (UUID, NOT NULL) - Distributed tracing ID (links related operations)
- `idempotency_key` (UUID, UNIQUE, NOT NULL) - Deduplication key (prevents duplicate processing)
- `created_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW()) - Creation timestamp

**Indexes:**
- `PRIMARY KEY (id)` - Primary key index
- `idx_tasks_idempotency_key ON tasks(idempotency_key) UNIQUE` - Enforces idempotency constraint
- `idx_tasks_trace_id ON tasks(trace_id)` - Trace-based queries
- `idx_tasks_created_at ON tasks(created_at DESC)` - Time-based queries

**Notes:**
- `idempotency_key` must be unique across all tasks (prevents duplicate task creation)
- `trace_id` links tasks across distributed workflows (used for debugging/auditing)
- `data` JSONB column stores task-specific payloads (e.g., approval details, routing info)

---

### approvals

Stores approval workflow state (pending/approved/rejected decisions).

**Columns:**
- `id` (UUID, PRIMARY KEY) - Approval identifier
- `task_id` (UUID, NOT NULL) - Associated task (FK → tasks.id)
- `approver_role` (VARCHAR(50), NOT NULL) - Role that approved/rejected
- `status` (VARCHAR(20), NOT NULL) - Approval status
  - Valid values: `pending`, `approved`, `rejected`
- `decision_at` (TIMESTAMP WITH TIME ZONE, nullable) - When decision was made (NULL if pending)
- `notes` (TEXT, NOT NULL, default '') - Approval comments/rationale
- `created_at` (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW()) - Creation timestamp

**Indexes:**
- `PRIMARY KEY (id)` - Primary key index
- `idx_approvals_task_id ON approvals(task_id)` - Task-based lookups
- `idx_approvals_status ON approvals(status)` - Status filtering (e.g., find all pending approvals)

**Constraints:**
- `FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE` - Cascade delete when task is deleted

**Notes:**
- Multiple approvals can exist for the same task (multi-step approval workflows)
- `decision_at` is NULL until status changes from `pending`
- `notes` field allows approvers to explain their decision

---

### audit_events

Stores audit trail for compliance and debugging.

**Columns:**
- `id` (UUID, PRIMARY KEY) - Event identifier
- `event_type` (VARCHAR(50), NOT NULL) - Event category
  - Examples: `task_routed`, `approval_requested`, `session_created`, `privacy_violation_detected`
- `role` (VARCHAR(50), NOT NULL) - Agent role that triggered event
- `timestamp` (TIMESTAMP WITH TIME ZONE, NOT NULL, default NOW()) - Event timestamp
- `trace_id` (UUID, NOT NULL) - Links to distributed trace
- `metadata` (JSONB, NOT NULL, default '{}') - Event-specific data

**Indexes:**
- `PRIMARY KEY (id)` - Primary key index
- `idx_audit_events_trace_id ON audit_events(trace_id)` - Trace-based queries
- `idx_audit_events_timestamp ON audit_events(timestamp DESC)` - Time-based queries (recent events)
- `idx_audit_events_event_type ON audit_events(event_type)` - Event type filtering

**Notes:**
- Immutable table (no updates, only inserts)
- Retention policy TBD (may archive old events to cold storage in Phase 7)
- `metadata` JSONB stores event-specific details (e.g., privacy violations, approval reasons)

---

## Relationships

```
sessions ──────> tasks (via task_id, optional)
                   │
                   ├──> approvals (via task_id, one-to-many)
                   │
                   └──> audit_events (via trace_id, many-to-many)
```

**Notes:**
- Sessions can exist without tasks (standalone agent sessions)
- Tasks can have multiple approvals (multi-step workflows)
- Audit events link to tasks via `trace_id` (not direct FK)

---

## Migration Strategy

**Tool:** `sqlx-cli` (compile-time query validation)

**Migration Files:**
- `deploy/migrations/001_create_schema.sql` - Initial schema creation
- Future migrations follow pattern: `NNN_description.sql`

**Rollback Strategy:**
- sqlx supports down migrations (not used in Phase 4)
- For Phase 4, schema is immutable after creation
- Future schema changes will use numbered migrations

**Environment Variables:**
- `DATABASE_URL=postgresql://postgres:postgres@localhost:5432/orchestrator`

---

## Performance Considerations

**Index Strategy:**
- All primary keys use UUID v4 (random distribution, prevents hotspots)
- Foreign key columns indexed (task_id lookups)
- Status columns indexed (common filter queries)
- Timestamp columns indexed DESC (recent-first pagination)

**Query Patterns:**
- List sessions: `ORDER BY created_at DESC LIMIT 50` (uses idx_sessions_created_at)
- Find pending approvals: `WHERE status = 'pending'` (uses idx_approvals_status)
- Trace debugging: `WHERE trace_id = $1` (uses idx_audit_events_trace_id)

**Connection Pooling:**
- sqlx `PgPool` with `max_connections=10` (configurable)
- Idle connections timeout after 30s

---

## Security

**SQL Injection Prevention:**
- All queries use parameterized prepared statements (sqlx compile-time verification)

**Data Retention:**
- Sessions expire after 7 days (configurable)
- Audit events retained indefinitely (Phase 7 may add archiving)

**Access Control:**
- Database user `postgres` has full access (dev environment)
- Production deployments should use restricted role (Phase 6+)

---

## Schema Version

**Version:** 0.4.0  
**Created:** Phase 4 (Storage/Metadata + Session Persistence)  
**Last Updated:** 2025-11-05  

---

## References

- **sqlx Documentation:** <https://docs.rs/sqlx/latest/sqlx/>
- **Postgres UUID Type:** <https://www.postgresql.org/docs/15/datatype-uuid.html>
- **JSONB Performance:** <https://www.postgresql.org/docs/15/datatype-json.html>
