# Phase 2.5 Checklist — Dependency Security & LTS Upgrades

**Status:** 📋 READY  
**Total Tasks:** 22  
**Estimated Effort:** ~6 hours  

---

## Workstream A: Infrastructure Upgrade (~2 hours)

- [ ] A1. Update VERSION_PINS.md (15 min)
  - [ ] Keycloak: 24.0.4 → 26.0.4
  - [ ] Vault: 1.17.6 → 1.18.3
  - [ ] Postgres: 16.4-alpine → 17.2-alpine
  - [ ] Document Ollama 0.12.9 rationale

- [ ] A2. Update ce.dev.yml (30 min)
  - [ ] Update Keycloak image tag
  - [ ] Update Vault image tag
  - [ ] Update Postgres image tag
  - [ ] Validate YAML syntax

- [ ] A3. Pull new Docker images (15 min)
  - [ ] `docker compose pull keycloak vault postgres`

- [ ] A4. Restart services (30 min)
  - [ ] `docker compose down`
  - [ ] `docker compose up -d`

- [ ] A5. Verify health checks (15 min)
  - [ ] All services show (healthy)

**Progress:** 0% (0/5 tasks complete)

---

## Workstream B: Phase 1.2 Validation (~1 hour)

- [ ] B1. Re-run Phase 1.2 smoke tests (30 min)
  - [ ] Test OIDC login flow
  - [ ] Test JWT verification
  - [ ] Test JWKS caching

- [ ] B2. Test /status endpoint (10 min)
  - [ ] Verify 200 OK response

- [ ] B3. Test /audit/ingest with JWT (20 min)
  - [ ] Get token from Keycloak
  - [ ] POST with Bearer token
  - [ ] Verify 202 Accepted

**Progress:** 0% (0/3 tasks complete)

---

## Workstream C: Phase 2.2 Validation (~1.5 hours)

- [ ] C1. Verify Vault pseudo_salt (15 min)
  - [ ] `vault kv get secret/pseudonymization`

- [ ] C2. Re-run Phase 2.2 smoke tests (45 min)
  - [ ] Test 1: Model Status Check
  - [ ] Test 2: Model-Enhanced Detection
  - [ ] Test 3: Graceful Fallback
  - [ ] Test 4: Performance Benchmarking
  - [ ] Test 5: Backward Compatibility

- [ ] C3. Test /status endpoint (10 min)
  - [ ] Verify model_enabled=true

- [ ] C4. Test deterministic pseudonymization (20 min)
  - [ ] Mask text twice, compare pseudonyms

**Progress:** 0% (0/4 tasks complete)

---

## Workstream D: Development Tools Upgrade (~30 min)

- [ ] D1. Update VERSION_PINS.md with dev tools (10 min)
  - [ ] Add "Development Tools (Phase 3+)" section
  - [ ] Document Python 3.13.9 (python:3.13-slim)
  - [ ] Document Rust 1.91.0 (rust:1.91.0-bookworm)

- [ ] D2. Pull dev tool Docker images (10 min)
  - [ ] `docker pull python:3.13-slim`
  - [ ] `docker pull rust:1.91.0-bookworm`

- [ ] D3. Test Rust 1.91.0 compilation (10 min)
  - [ ] Run `cargo check` with Rust 1.91.0
  - [ ] Verify Controller compiles successfully

**Progress:** 0% (0/3 tasks complete)

---

## Workstream E: Documentation (~1 hour)

- [ ] E1. Update CHANGELOG.md (15 min)
  - [ ] Document infrastructure upgrades
  - [ ] Document security CVE fixes
  - [ ] Note dev tool versions

- [ ] E2. Create ADR-0023 (30 min)
  - [ ] Title: "Dependency LTS Policy"
  - [ ] Decision rationale
  - [ ] Quarterly review process

- [ ] E3. Create validation summary (10 min)
  - [ ] Test results from Workstream B & C
  - [ ] Performance metrics
  - [ ] Issues found (if any)

- [ ] E4. Final VERSION_PINS.md review (5 min)
  - [ ] Ensure all sections complete
  - [ ] Verify version numbers

**Progress:** 0% (0/4 tasks complete)

---

## Overall Progress

**Total:** 0% (0/22 tasks complete)  
**Time Spent:** 0 hours  
**Time Remaining:** ~6 hours
