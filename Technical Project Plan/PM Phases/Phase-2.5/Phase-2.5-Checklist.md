# Phase 2.5 Checklist — Dependency Security & LTS Upgrades

**Status:** 📋 READY  
**Total Tasks:** 22  
**Estimated Effort:** ~6 hours  

---

## Workstream A: Infrastructure Upgrade (~2 hours)

- [x] A1. Update VERSION_PINS.md (15 min)
  - [x] Keycloak: 24.0.4 → 26.0.4
  - [x] Vault: 1.17.6 → 1.18.3
  - [x] Postgres: 16.4-alpine → 17.2-alpine
  - [x] Document Ollama 0.12.9 rationale

- [x] A2. Update ce.dev.yml (30 min)
  - [x] Update Keycloak image tag
  - [x] Update Vault image tag
  - [x] Update Postgres image tag
  - [x] Validate YAML syntax
  - [x] Fix Keycloak health check (TCP port instead of HTTP)

- [x] A3. Pull new Docker images (15 min)
  - [x] `docker compose pull keycloak vault postgres`

- [x] A4. Restart services (30 min)
  - [x] `docker compose down`
  - [x] `docker compose up -d`

- [x] A5. Verify health checks (15 min)
  - [x] All services show (healthy)

**Progress:** 100% (5/5 tasks complete)

---

## Workstream B: Phase 1.2 Validation (~1 hour)

- [x] B1. Re-run Phase 1.2 smoke tests (30 min)
  - [x] Test OIDC endpoint accessibility (master + dev realm)
  - [x] Test JWT token issuance (client_credentials grant)
  - [x] Test JWKS endpoint

- [x] B2. Verify Keycloak 26.0.4 compatibility (30 min)
  - [x] Seed dev realm with keycloak_seed.sh
  - [x] Create confidential client for token issuance
  - [x] Validate RS256 JWT token

- [x] B3. Document Phase 1.2 validation results (10 min)
  - [x] Created Phase-2.5-Keycloak-Validation.md
  - [x] Full E2E with Controller deferred (Controller not started)

**Progress:** 100% (3/3 tasks complete)

---

## Workstream C: Phase 2.2 Validation (~1.5 hours)

- [x] C1. Verify Vault pseudo_salt (15 min)
  - [x] Created pseudo_salt manually (Vault dev mode restart)
  - [x] Validated KV v2 access working

- [x] C2. Re-run Phase 2.2 smoke tests (45 min)
  - [x] Test 1: Model Status Check (✅ PASS)
  - [x] Test 2: Model-Enhanced Detection (✅ PASS with CPU limitations)
  - [x] Test 3: Deterministic Pseudonymization (✅ PASS)
  - [x] Test 4: Backward Compatibility (✅ PASS)

- [x] C3. Test /status endpoint (10 min)
  - [x] Verified model_enabled=true, model_name=qwen3:0.6b

- [x] C4. Test deterministic pseudonymization (20 min)
  - [x] Masked text twice, confirmed same output (Vault salt working)
  - [x] Created Phase-2.5-Privacy-Guard-Validation.md

**Progress:** 100% (4/4 tasks complete)

---

## Workstream D: Development Tools Upgrade (~30 min)

- [x] D1. Update VERSION_PINS.md with dev tools (10 min)
  - [x] Added "Development Tools (Phase 3+)" section
  - [x] Documented Python 3.13.9 (python:3.13-slim, EOL 2029-10)
  - [x] Documented Rust 1.83.0 (current) and 1.91.0 deferral

- [x] D2. Pull dev tool Docker images (10 min)
  - [x] Pulled python:3.13-slim (already latest)
  - [x] Pulled rust:1.91.0-bookworm for testing

- [x] D3. Test Rust 1.91.0 compilation (10 min)
  - [x] Ran cargo check with Rust 1.91.0
  - [x] Found compilation errors (Clone derives needed)
  - [x] Decided to defer upgrade to post-Phase 3

**Progress:** 100% (3/3 tasks complete)

---

## Workstream E: Documentation (~1 hour)

- [x] E1. Update CHANGELOG.md (15 min)
  - [x] Documented infrastructure upgrades (Keycloak, Vault, Postgres)
  - [x] Documented security CVE fixes (CVE-2024-8883, CVE-2024-7318, CVE-2024-8698)
  - [x] Noted dev tool versions (Python 3.13.9, Rust 1.83.0)

- [x] E2. Create ADR-0023 (30 min)
  - [x] Title: "Dependency LTS Policy"
  - [x] Decision rationale (security, stability, cost)
  - [x] Quarterly review process (March, June, September, December)

- [x] E3. Create validation summary (10 min)
  - [x] Created Phase-2.5-Validation-Summary.md
  - [x] Test results from Workstream B & C
  - [x] Performance metrics (no regression)
  - [x] Issues found and resolutions documented

- [x] E4. Final VERSION_PINS.md review (5 min)
  - [x] Ensured all sections complete (Infrastructure + Dev Tools)
  - [x] Verified version numbers and EOL dates

**Progress:** 100% (4/4 tasks complete)

---

## Overall Progress

**Total:** 100% (19/19 tasks complete) ✅  
**Status:** COMPLETE  
**Actual Time:** ~4.5 hours  
**Estimated Time:** ~6 hours (under budget!)
