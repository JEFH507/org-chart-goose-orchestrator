# Packaging & Deployment

Overview: Desktop agents + services (controller, directory-policy, audit) with CE defaults. Provide docker-compose and desktop packaging notes.

## KPIs
- Quickstart time ≤ 30 minutes
- Demo script runs end-to-end first time
- CI build success ≥ 95%

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - Compose baseline: ../../deploy/compose/ce.dev.yml\n- Health scripts: ../../deploy/compose/healthchecks/
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - CI workflow: linkcheck, spectral, compose health
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
