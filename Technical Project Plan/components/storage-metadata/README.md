# Storage & Metadata

Overview: Postgres metadata for tasks/sessions/approvals/audit index. Object store optional (artifacts). Enforces data minimization and TTL.

## KPIs
- DB p95 query ≤ 50ms for primary indexes
- Retention jobs complete daily
- Zero raw content persisted server-side
