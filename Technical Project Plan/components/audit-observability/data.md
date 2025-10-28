# Data

- Postgres: audit_events(id, ts, tenantId, actor jsonb, action, target, result, redactions jsonb, cost jsonb, traceId, hashPrev)
- Retention: 90 days default; export then purge.
