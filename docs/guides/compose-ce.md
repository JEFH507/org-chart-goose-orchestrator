# CE Compose (Phase 0)

Steps:
1. Copy env: `cp deploy/compose/.env.ce.example deploy/compose/.env.ce` and adjust ports.
2. Bring up infra only:
   - Basic: `docker compose -f deploy/compose/ce.dev.yml up -d`
   - Enable Ollama: `docker compose -f deploy/compose/ce.dev.yml --profile ollama up -d`
   - Enable SeaweedFS: `docker compose -f deploy/compose/ce.dev.yml --profile s3-seaweedfs up -d`
   - Enable MinIO: `docker compose -f deploy/compose/ce.dev.yml --profile s3-minio up -d`
3. Verify health:
   - `deploy/compose/healthchecks/keycloak.sh`
   - `deploy/compose/healthchecks/vault.sh`
   - `deploy/compose/healthchecks/postgres.sh`
   - `deploy/compose/healthchecks/ollama.sh` (if profile enabled)
   - `deploy/compose/healthchecks/minio.sh` (if profile enabled)

S3 is OFF by default. See ADR-0014 for policy details.
