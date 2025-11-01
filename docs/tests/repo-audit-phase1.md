# Repository Audit — Phase 1 (MVP)

Date: 2025-11-01
Scope: Org-Chart Orchestrated AI Framework (goose-org-twin) — Phase 1 bootstrap

This report summarizes health checks performed across the repository to validate Phase 1 readiness. It documents checks, findings, decisions, and recommended follow-ups.

## Summary
- Overall: PASS for Phase 1 objectives. CI skeleton, minimal controller, compose integration, dev seeds, metadata-only DB migration, observability docs, and smoke tests are in place.
- Blocking issues: None for Phase 1.
- Notable nits: Dockerfile bases not pinned by digest; dev-only credentials in compose (intentional); a few TODO placeholders in docs/schemas; CODEOWNERS pending.

## Checks and Findings

### 1) CI and Automation
- Workflows: .github/workflows/phase1-ci.yml present
  - Jobs: linkcheck (lychee offline), spectral lint, compose-health (scoped to Postgres) with path filter.
  - Status: Green on main after PR #21 and subsequent merges (#25, #26, #27).
- Lychee config: .github/lychee.toml excludes archived/vendor/backup content. Uses workflow globs to select files.
- Spectral config: .spectral.yaml minimal; JSON Schemas added to satisfy $refs.

Decision: CI scope is intentionally minimal and stable for MVP. Compose-health stays Postgres-only in CI.

Follow-ups:
- Optional: Add a nightly matrix for full-compose health, behind a profile flag.

### 2) Controller and Healthchecks
- Implementation: src/controller (Rust, axum 0.7) with GET /status, POST /audit/ingest.
- Logs: Structured JSON via tracing-subscriber.
- Healthcheck: Relaxed to poll /status with start_period=10s, retries=20. Compose controller now flips healthy reliably.

Decision (documented via ADR-0018): Healthchecks target app readiness (not dependency), with cautious timing to avoid flaps. Compose profiles isolate controller for local validation.

Follow-ups:
- Consider readiness vs liveness split if we adopt k8s later.

### 3) Docker and Compose
- Compose: deploy/compose/ce.dev.yml with pinned service images and corrected healthchecks per service.
- Controller local override: deploy/compose/local.controller.override.yml to use goose-controller:local and avoid host port conflicts during validation.
- Dockerfile: src/controller/Dockerfile multi-stage; base images by tag (rust:1.81-bookworm, debian:bookworm-slim).

Findings:
- Base images not pinned by digest (nit). Service images in compose are pinned to explicit tags (good).

Follow-ups:
- Pin Dockerfile FROM images by digest or track minor with periodic renovate (Phase 2+).

### 4) Secrets and Configuration Hygiene
- .gitignore includes .env and deploy/compose/.env.ce; no secrets committed.
- Dev-only defaults: Keycloak admin/admin, Vault dev root token — acceptable for CE dev posture; documented in guides.

Follow-ups:
- Reiterate “no secrets in git” in docs/security/secrets-bootstrap.md (already captured).

### 5) Database (Metadata-Only)
- db/migrations/0001_init_metadata.sql creates audit_metadata with indexes on trace_id and ts.
- db/README.md provides manual runner instructions.

Findings:
- TODOs for future indexes/relationships are acceptable (Phase 7 per plan).

### 6) Documentation and Specs
- Observability: docs/architecture/observability.md covers structured logs, redaction, and OTLP stubs.
- Smoke tests: docs/tests/smoke-phase1.md validates CI and runtime endpoints.
- OpenAPI: docs/api/controller/openapi.yaml references added schemas; spectral passes.

Findings:
- TODO schema placeholder remains by design (for future controller methods).

### 7) Codebase TODOs/Nits Scan
- Markers found in reference subtree and planning docs; project-owned actionable items include:
  - CODEOWNERS has TODO to update maintainers’ handles.
  - db/migrations/metadata-only/0001_init.sql TODO indexes for Phase 7.
  - docs/api/schemas/TODO-task.schema.json intentionally present for spectral/$ref coverage.

Follow-ups:
- Add CODEOWNERS entries when maintainer list is final.

## Recommendations (Phase 2+)
- Pin Dockerfile base images by digest or employ Renovate/Dependabot to bump tags proactively.
- Add a periodic (nightly) job for full compose health if desired.
- Consider adding a light controller unit test for /status shape.
- Expand OpenAPI and add CI spectral ruleset tightening.

## Acceptance
- Phase 1 repo health: ACCEPTED.
- Residual nits are tracked above; none block Phase 1 close-out.

---

Appendix: Commands used
- docker build -t goose-controller:local -f src/controller/Dockerfile .
- docker compose -f deploy/compose/ce.dev.yml -f deploy/compose/local.controller.override.yml --profile controller up -d --build
- docker compose -f deploy/compose/ce.dev.yml -f deploy/compose/local.controller.override.yml --profile controller ps
- docker exec <controller> curl -fsS http://localhost:8088/status
- docker exec <controller> curl -fsS -i -X POST http://localhost:8088/audit/ingest -H 'Content-Type: application/json' -d '{"source":"smoke","category":"test","action":"ping"}'
