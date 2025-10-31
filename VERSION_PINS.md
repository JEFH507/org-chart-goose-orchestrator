# VERSION_PINS (Phase 0)

Pin all images explicitly (no :latest). Tags may evolve later; update via PR.

- Keycloak: quay.io/keycloak/keycloak:24.0.4
- Vault: hashicorp/vault:1.17.6
- Postgres: postgres:16.4-alpine
- Ollama: ollama/ollama:0.3.14
- SeaweedFS: chrislusf/seaweedfs:3.68
- MinIO: minio/minio:RELEASE.2024-09-22T00-00-00Z
- Garage (optional): dxflrs/garage:0.9.3

Notes:
- Guard model tags: document selections in docs/guides/guard-model-selection.md; do not bundle weights in repo.
- HTTP-only posture and metadata-only storage per ADR-0010/0012.
