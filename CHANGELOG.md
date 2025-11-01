# Changelog\n\nAll notable changes to this project will be documented in this file.

## Unreleased
- Phase 0 scaffolding: repo hygiene, dev setup, version pins, compose baseline, OpenAPI and schema stubs, metadata-only migrations.
- See progress log: docs/tests/phase0-progress.md

## 2025-11-01 — Phase 1 (MVP)
- CI skeleton stabilized (linkcheck, spectral, compose health)
- Controller baseline (Rust): /status, /audit/ingest; structured logs
- Compose integration: controller profile + healthcheck; Dockerfile
- Dev seeding scripts: Keycloak and Vault (idempotent; no secrets)
- DB migrations: metadata-only; runner docs
- Observability docs (structured logging, redaction posture, OTLP stubs)
- Smoke tests doc
