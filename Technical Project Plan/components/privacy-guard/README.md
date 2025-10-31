# Privacy Guard

Overview: Local pre/post masking and deterministic pseudonymization; supports detect-only, mask-and-forward, block modes. Integrates with Goose providers as wrapper and as an MCP tool.

## KPIs
- Overhead ≤ 500ms P50
- False negative rate for key PII classes ≤ 3% in test set
- Zero raw PII in audit logs

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - ADR-0002: ../../docs/adr/0002-privacy-guard-placement.md\n- AuditEvent schema (redactions metadata): ../../docs/audit/audit-event.schema.json
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - Document redaction metadata expectations; runtime enforcement deferred
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
