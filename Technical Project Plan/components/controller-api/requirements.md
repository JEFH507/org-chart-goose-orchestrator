# Requirements

## Functional
- POST /tasks/route; POST /sessions; POST /approvals; GET /status/{id}
- GET /profiles/{role} (proxy to directory); POST /audit/ingest.

## Non-functional
- Idempotency keys; request size limits (1MB); structured logs.
