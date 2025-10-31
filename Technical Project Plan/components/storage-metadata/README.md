# Storage & Metadata

Overview: Postgres metadata for tasks/sessions/approvals/audit index. Object store optional (artifacts). Enforces data minimization and TTL.

## KPIs
- DB p95 query ≤ 50ms for primary indexes
- Retention jobs complete daily
- Zero raw content persisted server-side

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - Metadata-only migrations: ../../db/migrations/metadata-only/0001_init.sql\n- DB README: ../../db/README.md
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - Indexes/FKs where low-risk; migration runner docs
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
