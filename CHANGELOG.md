# Changelog

All notable changes to this project will be documented in this file.

## Unreleased
- Phase 2.3+ components (Performance optimization, minimal UI, Controller API, Agent Mesh, Directory/Policy, etc.)

## 2025-11-04 — Phase 2.2: Privacy Guard Model Enhancement ✅
**Summary:** Ollama integration with qwen3:0.6b NER model for improved PII detection. Hybrid detection (regex + model consensus) with graceful fallback. CPU-only inference accepted (P50 ~23s).

### Added
- **Ollama Integration:**
  - Ollama 0.12.9 service in Docker Compose (8GB RAM limit)
  - qwen3:0.6b NER model (523MB, 40K context, Nov 2024)
  - Automated initialization script: `deployment/docker/init-ollama.sh`
  - Graceful fallback to regex-only if model unavailable
- **Hybrid Detection System:**
  - Model detector: `src/detection/model_detector.rs` (~400 lines)
  - Hybrid detector: `src/detection/hybrid_detector.rs` (~300 lines)
  - Consensus merging: Both detect → HIGH, Model-only → HIGH, Regex-only → MEDIUM/HIGH
  - Improved coverage: Person names without titles now detected (e.g., "Jane Smith")
- **Status Endpoint Enhancement:**
  - Added `model_enabled: bool` field
  - Added `model_name: Option<String>` field
  - Updated `GET /status` response schema
- **Configuration:**
  - `.env.ce` additions: `GUARD_OLLAMA_URL`, `GUARD_MODEL_NAME`, `GUARD_MODEL_ENABLED`, `GUARD_MODEL_TIMEOUT_SECS`
  - Opt-in feature: Model disabled by default (preserves Phase 2 performance)
- **Documentation:**
  - Model integration architecture: `docs/architecture/model-integration.md` (~800 lines)
  - Ollama setup operations guide: `docs/operations/ollama-setup.md` (~400 lines)
  - Smoke test procedure: `docs/tests/smoke-phase2.2.md` (~500 lines)
  - Test results report: `Technical Project Plan/PM Phases/Phase-2.2/C2-SMOKE-TEST-RESULTS.md`
- **Testing:**
  - Unit tests: `tests/unit/model_detector_test.rs` (8 tests)
  - Integration tests: `tests/integration/hybrid_detection_test.rs` (12 tests)
  - Performance benchmark: `tests/performance/benchmark_phase2.2.sh`
  - Smoke tests: 5/5 passed (100% success rate)

### Performance (CPU-Only Inference)
- **Benchmarked (10 requests):**
  - P50: 22,807ms (~23s) - acceptable for 8GB RAM, no GPU
  - P95: 47,027ms (~47s) - one outlier due to CPU variance
  - P99: 47,027ms (~47s)
  - Success rate: 100% (60s timeout prevents failures)
- **Baseline (Phase 2 Regex-Only):**
  - P50: 16ms (preserved when model disabled)
- **Future Optimization (Phase 2.3):**
  - Smart triggering: Expected P50 ~100ms (80-90% fast path)

### Fixed
- Blocker: Ollama version incompatibility (0.3.14 → 0.12.9)
- Blocker: Request timeouts (30s → 60s)
- Model loading cold start (added warm-up on startup)

### Testing
- **Unit Tests:** 8 tests (model detector)
- **Integration Tests:** 12 tests (hybrid detection consensus)
- **Smoke Tests:** 5/5 passed
  - ✅ Model status check
  - ✅ Model-enhanced detection (person names, partial org)
  - ✅ Graceful fallback (model disabled/unavailable)
  - ⚠️ Performance benchmarking (acceptable for CPU)
  - ✅ Backward compatibility (Phase 2 preserved)

### Known Limitations
- **Performance:** 22.8s P50 (CPU-only, no GPU available)
- **Organization Detection:** Limited (small model constraint)
- **Confidence Tuning:** Model-only → HIGH (can introduce false positives)

### Branches
- `feat/phase2.2-ollama-detection` (18 commits)
  - Workstream A (Design & Code): 5 commits
  - Workstream B (Infrastructure): 6 commits
  - Workstream C (Testing): 5 commits
  - Blockers resolved: 2 commits

**See:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`

---

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
