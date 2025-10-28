# Controller API

Overview: Minimal HTTP controller that routes tasks, handles approvals, session aggregation, profile lookup proxy, and audit ingest. Publishes minimal OpenAPI.

## KPIs
- 99.5% availability
- p95 route latency ≤ 200ms (excluding agent processing)
- OpenAPI client codegen successful in CI
