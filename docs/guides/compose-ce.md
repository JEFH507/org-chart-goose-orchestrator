# CE Compose (Phase 0)

Decision highlights
- Object storage is opt-in. Default profile runs without S3.
- Provide multiple S3 options via profiles/env (SeaweedFS default ALv2 option; MinIO/Garage alternatives; Ozone for scale later).
- Standard ports with .env overrides (see docs/guides/ports.md). Run preflight checks before `up`.

What this profile includes now (in Phase 0)
- Keycloak (OIDC): ${KEYCLOAK_PORT:-8080}
- Vault OSS (dev): ${VAULT_PORT:-8200}
- Postgres: ${POSTGRES_PORT:-5432}
- Ollama (local models): ${OLLAMA_PORT:-11434}
- Object storage: disabled by default; see below

Profiles (to be added in Phase 1)
- seaweedfs: enables SeaweedFS S3 gateway
- minio: enables MinIO
- garage: enables Garage

Preflight
- Run `scripts/dev/preflight_ports.sh` to detect port conflicts.

Model provisioning
- Models are not bundled. On first use, `ollama pull <model>` with explicit consent. See docs/guides/guard-model-selection.md.

License & notices
- See docs/THIRD_PARTY.md for license and links of optional services.
