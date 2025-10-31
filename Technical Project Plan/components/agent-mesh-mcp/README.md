# Agent Mesh MCP

Overview: Standard MCP extension offering cross-agent verbs: send_task, request_approval, notify, fetch_status. Secure HTTP calls across agents via controller endpoints; enforces policy hints.

## KPIs
- Task delivery success ≥ 99%
- Median call latency ≤ 300ms intra-VPC
- Policy violation rate < 1% (caught)

## Phase Alignment

- Phase 0 (completed)
  - Summary: ../PM Phases/Phase-0/Phase-0-Summary.md
  - ADR-0007: ../../docs/adr/0007-agent-mesh-mcp.md
- Phase 1 (planned)
  - Plan: ../PM Phases/Phase-1/Phase-1-Execution-Plan.md
  - No runtime changes; planning docs only
- Later phases
  - Refer to master plan: ../master-technical-project-plan.md
