# Audit & Observability

Overview: OTLP traces, structured logs, audit event ingest and export (ndjson). Redaction-aware logging.

## KPIs
- 100% audit events redacted as needed
- Export job completion < 2m for 100k events
- Trace sampling 5–10% with minimal overhead

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - AuditEvent schema: ../../docs/audit/audit-event.schema.json\n- Compose baseline and health scripts: ../../deploy/compose/healthchecks/
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - Observability doc with logging fields (traceId, actor, redactions)\n- Ensure no PII logs
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
