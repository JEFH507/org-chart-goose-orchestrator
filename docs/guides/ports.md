# Ports (Defaults and Overrides)

Standardize defaults for a predictable developer experience. Override via environment.

Defaults (override via a local `.env.ce` you create under `deploy/compose/` — not tracked by repo):
- Keycloak: 8080
- Vault: 8200
- Postgres: 5432
- Ollama: 11434
- MinIO (opt-in): 9000 (S3), 9001 (console)
- SeaweedFS (opt-in): 8333 (S3), 9333 (master UI), 8081 (Filer API)

Preflight check
- Use `scripts/dev/preflight_ports.sh` to detect conflicts and suggest alternatives before `docker compose up`.

Overrides
- Create a local `deploy/compose/.env.ce` (ignored by repo policy) and adjust ports, e.g.: `KEYCLOAK_PORT=18080`.
