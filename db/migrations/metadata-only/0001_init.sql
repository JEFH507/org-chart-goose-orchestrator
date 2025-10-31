-- Metadata-only migrations (Phase 0)

CREATE TABLE IF NOT EXISTS sessions_meta (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  actor_id TEXT,
  trace_id TEXT
);

CREATE TABLE IF NOT EXISTS tasks_meta (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  type TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  cost_json JSON,
  hash_prev TEXT
);

CREATE TABLE IF NOT EXISTS approvals_meta (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  approver_role TEXT NOT NULL,
  status TEXT NOT NULL,
  decided_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS audit_index (
  id TEXT PRIMARY KEY,
  ts TIMESTAMP NOT NULL,
  tenant_id TEXT NOT NULL,
  actor_id TEXT,
  action TEXT NOT NULL,
  target_id TEXT,
  redactions_count INTEGER DEFAULT 0,
  trace_id TEXT
);

-- TODO (Phase 7): Indexes and foreign keys between meta tables
