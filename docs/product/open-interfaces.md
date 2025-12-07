# Open Interfaces

Adapters and standards that keep you portable and in control.

## Provider interfaces
- AuthProvider (OIDC), SecretsProvider (Vault/KMS), StorageProvider (Postgres/S3), ModelProvider (OpenAI‑compatible & local), BusProvider (future)
- Config‑not‑code: swap vendors in YAML/TOML; no rebuilds
- Conformance tests for adapters; OSS reference implementations

## Standards
- Identity: OIDC (Keycloak in CE; any OIDC in SaaS)
- Storage: Postgres and S3‑compatible object stores (SeaweedFS/MinIO/Garage)
- Models: OpenAI‑compatible API; local models via Ollama
- Tools: MCP (goose extensions)
- Telemetry (optional): OpenTelemetry, Prometheus/Loki/Grafana

---
Quick nav: [Docs Home](../README.md) • [Community Edition](./ce.md) • [Architecture One‑Pager](../architecture/one-pager.md)
