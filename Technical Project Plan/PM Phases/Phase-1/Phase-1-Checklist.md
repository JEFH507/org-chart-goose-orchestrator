# Phase 1 Checklist — goose-org-twin

## Scope
Phase 1 establishes the minimal, testable MVP runtime and CI posture for the org-chart orchestrator:
- CI skeleton (linkcheck, OpenAPI spectral lint, compose health)
- Minimal controller runtime (Rust) with /status and /audit/ingest
- Compose integration with healthcheck profile
- Dev seeding scripts (Keycloak, Vault) — idempotent, no secrets committed
- Phase 1 DB metadata migrations (indexes/FKs only)
- Observability docs (structured logs, redaction, OTLP stubs)
- Smoke/acceptance checks and CHANGELOG entry

## Tasks
- A1: Phase 1 planning docs; README roadmap update
- A2: CI workflow (linkcheck, spectral, compose health)
- B1: Controller runtime baseline (Rust; HTTP-only; metadata-only)
- B2: Compose integration (controller service + healthcheck script)
- C1: Keycloak dev seeding script + guide
- C2: Vault dev bootstrap script + guide
- D1: DB Phase 1 migrations + runner docs (no content-bearing columns)
- E: Observability docs (logging fields, redaction, OTLP stubs)
- F: Acceptance and smoke checks; CHANGELOG update
- G (optional): Repo-wide docs audit and cleanup (approval required)

## Acceptance
- CI workflow runs successfully on PR for Phase 1 branch
- Controller responds:
  - GET /status → 200 {"status":"ok","version":"x.y.z"}
  - POST /audit/ingest → 202; validates AuditEvent schema; logs metadata only
- Compose profile "controller" passes healthcheck
- Seeding scripts re-runnable, log actions, commit no secrets
- DB migrations apply cleanly; no content-bearing columns added
- Logs documented; no PII; redaction metadata included
- Smoke doc validates endpoints and CI checks; progress log updated

## Dependencies
- ADRs 0001–0017 (esp. ADR-0010 OpenAPI, ADR-0014 storage, ADR-0017 language/runtime)
- OpenAPI stub: docs/api/controller/openapi.yaml
- Docker with compose plugin; Postgres service for dev
- Optional: Keycloak and Vault containers for scripts

## Notes
- HTTP-only posture; metadata-only server model
- Object storage OFF by default (enable via profiles)
- No secrets committed; local .env.ce is ignored
