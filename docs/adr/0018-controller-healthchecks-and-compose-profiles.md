# ADR 0018: Controller Healthchecks and Compose Profiles (Phase 1)

## Context
During Phase 1, the minimal controller (Rust, axum 0.7) exposes:
- GET /status → 200 {"status":"ok","version":"0.1.0"}
- POST /audit/ingest → 202 (metadata-only, no persistence)

Compose integration initially showed the controller stuck in "starting" due to an eager healthcheck and transient startup timing. Healthchecks were incorrectly targeting Postgres in earlier iterations.

## Decision
- Healthchecks MUST probe the application endpoint that reflects readiness (controller /status), not a dependency.
- Compose healthcheck parameters SHOULD be conservative to avoid flapping:
  - start_period: 10s, interval: 5s, timeout: 3s, retries: 20
- A local override file (deploy/compose/local.controller.override.yml) SHOULD be used to:
  - Swap in a local image (goose-controller:local)
  - Avoid host port conflicts (no published ports)
  - Limit the profile to controller (+postgres dependency) for faster validation.

## Technical details
- deploy/compose/ce.dev.yml:
  ```yaml
  healthcheck:
    test: ["CMD-SHELL", "curl -fsS http://localhost:8088/status || exit 1"]
    start_period: 10s
    interval: 5s
    timeout: 3s
    retries: 20
  ```
- deploy/compose/healthchecks/controller.sh uses fast curl timeouts for local checks.
- Validation is done with:
  ```bash
  docker build -t goose-controller:local -f src/controller/Dockerfile .
  docker compose -f deploy/compose/ce.dev.yml -f deploy/compose/local.controller.override.yml --profile controller up -d --build
  docker compose -f deploy/compose/ce.dev.yml -f deploy/compose/local.controller.override.yml --profile controller ps
  docker exec <controller> curl -fsS http://localhost:8088/status
  ```

## Security & privacy impact
- No change. Healthcheck queries do not expose data and carry no secrets.

## Operational impact
- Improved reliability of local compose validation.
- CI remains scoped to Postgres-only compose-health for stability.

## Consequences
- Slightly longer warmup window, but stable healthy state.
- Clear separation of local validation vs CI posture.

## Alternatives considered
- Keeping Postgres-focused healthcheck: rejected; does not reflect controller readiness.
- Aggressive retries with zero start_period: rejected; observed flapping.

## Decision lifecycle
- Status: Accepted for Phase 1.
- Revisit in Phase 2 for Kubernetes readiness/liveness separation and potential probe endpoints.

## References
- PR #27: Relax controller healthcheck and validate compose (B2)
- docs/tests/repo-audit-phase1.md
- docs/tests/smoke-phase1.md
