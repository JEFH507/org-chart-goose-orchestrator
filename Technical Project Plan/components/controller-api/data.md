# Data

Minimal metadata in Postgres:
- tasks(id, tenantId, target, status, created_at)
- approvals(id, sessionId, approverRole, status, ts)
- sessions(id, participants jsonb, scope jsonb)

## Retention
90 days (configurable).
