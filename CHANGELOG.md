# Changelog

All notable changes to this project will be documented in this file.

## Unreleased
- Phase 3+ components (Controller API, Agent Mesh, Directory/Policy, etc.)

## 2025-11-03 — Phase 2: Privacy Guard ✅
**Summary:** Rust HTTP service with regex-based PII detection, deterministic pseudonymization, and format-preserving encryption. Performance exceeds targets by 30-87x.

### Added
- **Privacy Guard Service** (Rust/Axum on port 8089):
  - PII detection for 8 entity types (SSN, EMAIL, PHONE, CREDIT_CARD, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER)
  - 25+ regex patterns with confidence scoring (HIGH/MEDIUM/LOW)
  - HMAC-SHA256 deterministic pseudonymization with per-tenant salt
  - Format-preserving encryption (FPE) for phone (4 formats) and SSN (2 formats)
  - Strategy-based masking: Pseudonym, FPE, Redact
  - Policy modes: OFF, DETECT, MASK (default), STRICT
  - HTTP API: `/guard/scan`, `/guard/mask`, `/guard/reidentify`, `/status`, `/internal/flush-session`
  - Session-scoped in-memory state (no persistence)
  - Structured audit logging (counts only, no raw PII)
- **Configuration:**
  - `deploy/compose/guard-config/rules.yaml` (24 patterns across 8 entity types)
  - `deploy/compose/guard-config/policy.yaml` (modes, strategies, audit settings)
  - Test fixtures (382 lines: 219 PII samples, 163 clean samples)
- **Deployment:**
  - Dockerfile with multi-stage build (90.1MB image)
  - Docker Compose service with healthcheck
  - Healthcheck script: `deploy/compose/healthchecks/guard_health.sh`
- **Controller Integration:**
  - Optional guard client (`GUARD_ENABLED` flag)
  - Fail-open error handling
  - Redaction count logging
- **Documentation:**
  - Configuration guide (891 lines): `docs/guides/privacy-guard-config.md`
  - Integration guide (1,157 lines): `docs/guides/privacy-guard-integration.md`
  - Smoke test procedure (943 lines): `docs/tests/smoke-phase2.md`
  - Test results report: `docs/tests/phase2-test-results.md`
- **ADRs:**
  - ADR-0021: Privacy Guard Rust Implementation
  - ADR-0022: PII Detection Rules and FPE

### Performance
- **Benchmarked (100 requests):**
  - P50: 16ms (target: 500ms) → **31x better** ⚡
  - P95: 22ms (target: 1s) → **45x better** ⚡
  - P99: 23ms (target: 2s) → **87x better** ⚡
  - Success rate: 100%

### Testing
- **Test Coverage:** 145+ tests across 7 modules
  - Detection: 13 tests
  - Pseudonymization: 11 tests
  - State management: 9 tests
  - FPE: 48 tests (26 FPE + 22 masking integration)
  - Policy: 46 tests (38 unit + 8 E2E)
  - Audit: 9 tests
  - HTTP API: 16 tests (5 unit + 11 integration)
- **Smoke Tests:** 9/10 passed, 2 skipped (documented)
  - ✅ Healthcheck, Detection, Masking, FPE, Determinism, Tenant Isolation
  - ✅ Audit logs (no PII), Performance, Session management
  - ⏭️ Reidentification (requires Phase 1.2 JWT)
  - ⏭️ Controller integration (compilation errors documented)

### Fixed
- Compilation errors: ~40 entity type variant fixes, borrow checker errors
- Vault healthcheck: Changed to `vault status` CLI command
- Dockerfile build hang: Removed `--version` check

### Branches
- `feat/phase2-guard-core` (Workstream A - Core Implementation)
- `feat/phase2-guard-config` (Workstream B - Configuration)
- `feat/phase2-guard-deploy` (Workstream C - Deployment)
- `docs/phase2-guides` (Workstream D - Documentation)

**Total:** 28 commits, 18/19 tasks (95% complete)

**See:** `docs/tests/phase2-progress.md`, `Technical Project Plan/PM Phases/Phase-2/`

---

## 2025-11-03 — Phase 1.2: Identity & Security Realignment ✅
**Summary:** JWT verification middleware, Keycloak OIDC integration, Vault wiring documentation

### Added
- JWT verification middleware in controller (RS256 signature validation)
- JWKS caching with issuer/audience validation
- Keycloak realm seeding: dev realm, test user, role assignments
- Vault wiring guide for PSEUDO_SALT management
- Reverse proxy auth pattern documentation
- Smoke test procedure: `docs/tests/smoke-phase1.2.md`
- ADR-0019: Auth Bridge JWT Verification
- ADR-0020: Vault OSS Wiring

### Changed
- Controller `/audit/ingest` now requires Bearer JWT
- `/status` endpoint remains public
- Graceful degradation when OIDC config missing

**See:** `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Completion-Summary.md`

---

## 2025-11-01 — Phase 1: Initial Runtime ✅
**Summary:** Minimal controller runtime with audit ingestion, Docker Compose integration, CI skeleton

### Added
- Controller baseline (Rust/Axum): `/status`, `/audit/ingest`
- Docker Compose integration with healthchecks
- CI skeleton (linkcheck, Spectral, compose health)
- Keycloak/Vault dev seeding scripts (idempotent)
- DB Phase 1 migrations (indexes/FKs)
- Observability docs (structured logs, redaction)
- Smoke tests documentation

**See:** `Technical Project Plan/PM Phases/Phase-1/Phase-1-Completion-Summary.md`

---

## 2025-10-31 — Phase 0: Project Setup ✅
**Summary:** Repo hygiene, dev environment, Docker Compose baseline, OpenAPI stubs, metadata-only migrations

### Added
- Repo hygiene: branch protections, conventional commits, PR template
- Dev environment bootstrap docs (Linux/macOS)
- CE defaults: version pinning (Keycloak, Vault, Postgres, Ollama)
- Docker Compose baseline (infra only)
- OpenAPI stub with schema placeholders
- DB migration stubs (metadata-only)
- Repository reorganization (Workstream G)

**See:** `Technical Project Plan/PM Phases/Phase-0/Phase-0-Summary.md`
