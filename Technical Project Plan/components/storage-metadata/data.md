# Data

## Tables (summary)
- sessions(id, tenantId, participants jsonb, scope jsonb, created_at)
- tasks(id, sessionId?, tenantId, target jsonb, status, created_at)
- approvals(id, sessionId, approverRole, status, payload jsonb, ts)
- audit_events(...)

Indexes on tenantId, created_at, status.
