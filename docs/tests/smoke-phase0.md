# Phase 0 Smoke Checks

## Preflight ports
./scripts/dev/preflight_ports.sh

## Compose bring-up (infra only)
cp deploy/compose/.env.ce.example deploy/compose/.env.ce
# Adjust ports if needed, then:
docker compose -f deploy/compose/ce.dev.yml up -d

## Health verification
./deploy/compose/healthchecks/keycloak.sh
./deploy/compose/healthchecks/vault.sh
./deploy/compose/healthchecks/postgres.sh
# Optional profiles:
./deploy/compose/healthchecks/ollama.sh
./deploy/compose/healthchecks/minio.sh

## OpenAPI lint (warn-only)
./scripts/dev/openapi_lint.sh || true

## Presence checks
ls -1 docs/audit/audit-event.schema.json
ls -1 docs/policy/profile-bundle.schema.yaml
ls -1 db/migrations/metadata-only/0001_init.sql

