# Smoke Tests — Phase 1

This document defines the smoke checks for Phase 1.

## CI checks
- Linkcheck (internal-only; excludes archived/backups/vendor refs)
- Spectral lint of `docs/api/controller/openapi.yaml`
- Compose health (Postgres only) for CI speed; full stack validated locally

## Local compose
```bash
# Bring up metadata DB and controller
export CONTROLLER_PORT=8088
export DATABASE_URL=postgresql://postgres:postgres@postgres:5432/postgres

docker compose -f deploy/compose/ce.dev.yml --profile controller up -d postgres controller

# Wait for controller to be healthy
watch -n 2 "docker inspect -f '{{json .State.Health.Status}}' ce_controller"
```

## Endpoints
```bash
# /status should return 200 and {"status":"ok","version": "x.y.z"}
curl -sf http://127.0.0.1:8088/status | jq .

# /audit/ingest should return 202 and log metadata-only fields
curl -i -X POST http://127.0.0.1:8088/audit/ingest \
  -H 'Content-Type: application/json' \
  -d '{"source":"smoke","category":"test","action":"ingest","traceId":"smoke-trace","metadata":{"k":"v"}}'
```

## Teardown
```bash
docker compose -f deploy/compose/ce.dev.yml down -v
```

## Acceptance
- CI green on PRs to main (linkcheck, spectral, compose-health)
- Controller: `/status` 200 with payload; `/audit/ingest` 202 and metadata-only logs
- Progress logged in `docs/tests/phase1-progress.md`
