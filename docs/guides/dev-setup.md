# Developer Setup (Phase 0)

This guide covers Linux/macOS prerequisites, repo overview, default ports, override mechanism, and Phase 0 smoke steps.

## Prerequisites
- Docker (CPU-only; no GPU required)
- Git (SSH recommended)
- Optional: Spectral (`npm i -g @stoplight/spectral-cli`) for OpenAPI lint (warn-only)

## OS Support
- Linux and macOS are supported for Phase 0 scaffolding.

## Repository Layout (high level)
- services/ (reserved for future runtime services)
- deploy/compose/ (docker compose files, env examples)
- docs/ (architecture, ADRs, guides, api, security)
- config/ (templates and configurations)
- db/ (migrations and notes)
- scripts/ (dev helpers)

## Default Ports and Overrides
See [Ports Registry](./ports.md) for the authoritative list and strategy.
Defaults (Phase 0): Keycloak 8080, Vault 8200, Postgres 5432, Ollama 11434,
SeaweedFS 8333/9333/8081, MinIO 9000/9001.

Override via `deploy/compose/.env.ce` (local-only, not committed).

## Local env file
1. Copy: `cp deploy/compose/.env.ce.example deploy/compose/.env.ce`
2. Edit values as needed.

## Smoke checks (Phase 0)
- Read [docs/tests/smoke-phase0.md] for commands to validate:
  - Preflight ports
  - Compose bring-up (infra only) and health verification
  - OpenAPI lint (warn-only)
  - Presence of schemas and migrations

## Known Issues / Troubleshooting
- If ports are occupied, adjust in `.env.ce` and retry
- If `gh` CLI is missing, use Git web UI to open PRs
- S3-compatible object storage is OFF by default per ADR-0014; enabling is optional and documented.

## Secrets and Keys (dev only)
See [docs/security/secrets-bootstrap.md] for Vault dev mode notes and key handling. Do not commit secrets.
