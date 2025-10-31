# Ports Registry (Phase 0)

Default ports:
- Keycloak: 8080
- Vault: 8200
- Postgres: 5432
- Ollama: 11434
- SeaweedFS: 8333 (S3), 9333 (master), 8081 (filer)
- MinIO: 9000 (API), 9001 (Console)

Override strategy:
- Use `deploy/compose/.env.ce` to set environment variables consumed by compose files.
- Example: set `KEYCLOAK_PORT=8088` to change Keycloak’s host port.

See also: `deploy/compose/.env.ce.example` and docs/guides/dev-setup.md.
