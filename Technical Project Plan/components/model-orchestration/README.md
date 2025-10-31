# Model Orchestration

Overview: Lead/worker provider routing with guard-first; cost-aware downshift; provider allow/deny by policy.

## KPIs
- Downshift rate for summaries ≥ 50%
- Cost variance within budget ±10%
- p95 plan+execute within targets

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - Guard model selection doc: ../../docs/guides/guard-model-selection.md\n- ADR-0015: ../../docs/adr/0015-guard-model-policy-and-selection.md
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - No runtime changes; planning docs only
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
