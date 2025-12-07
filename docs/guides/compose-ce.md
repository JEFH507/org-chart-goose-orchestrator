# CE Compose (Phase 0)

Steps:
1. Copy env: `cp deploy/compose/.env.ce.example deploy/compose/.env.ce` and adjust ports.
   - **Important:** Update `OIDC_CLIENT_SECRET` with actual value from Keycloak UI
   - **Important:** Verify `DATABASE_URL` points to `orchestrator` database (not `postgres`)
2. Create symlink for docker-compose auto-loading: `cd deploy/compose && ln -sf .env.ce .env`
3. Bring up infra only:
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

## Port conflicts (Ollama)

- Ollama defaults to host port 11434. If you already run a host Ollama (e.g., goose Desktop), do not enable the compose `ollama` profile at the same time.
- Alternatively, override OLLAMA_PORT in deploy/compose/.env.ce and re-run compose.
- See also: docs/guides/dev-setup.md (Known Issues) and docs/guides/ports.md.
