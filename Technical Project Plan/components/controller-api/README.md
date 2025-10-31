# Controller API

Overview: Minimal HTTP controller that routes tasks, handles approvals, session aggregation, profile lookup proxy, and audit ingest. Publishes minimal OpenAPI.

## KPIs
- 99.5% availability
- p95 route latency ≤ 200ms (excluding agent processing)
- OpenAPI client codegen successful in CI

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - OpenAPI stub: ../../docs/api/controller/openapi.yaml\n- Schemas: ../../docs/api/schemas/README.md\n- AuditEvent schema: ../../docs/audit/audit-event.schema.json
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - Minimal controller runtime (status, audit ingest stubs)\n- Spectral lint + linkcheck in CI\n- Compose integration and healthcheck
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
