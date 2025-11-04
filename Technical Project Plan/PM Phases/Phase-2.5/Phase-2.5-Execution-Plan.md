# Phase 2.5 Execution Plan — Dependency Security & LTS Upgrades

**Phase:** 2.5 (Unplanned - Security & Maintenance)  
**Type:** Small (S) - ~5-6 hours  
**Priority:** HIGH (Security CVEs in Keycloak)  
**Date Created:** 2025-11-04  
**Prerequisites:** Phases 0, 1, 1.2, 2, 2.2 complete  
**Blocks:** Phase 3 (Controller API + Agent Mesh)

---

## 🎯 Objectives

### Primary Goal
Upgrade infrastructure dependencies to latest LTS/stable versions for:
1. **Security:** Patch critical CVEs (especially Keycloak CVE-2024-8883 HIGH)
2. **Performance:** Benefit from improvements in Postgres 17, Keycloak 26, Vault 1.18
3. **Maintainability:** Reduce technical debt; avoid version lag issues

### Success Criteria
- ✅ All dependencies upgraded to latest LTS/stable versions
- ✅ Phase 1.2 tests pass (JWT auth with Keycloak 26.0.4)
- ✅ Phase 2.2 tests pass (Privacy Guard with Vault 1.18.3 + Ollama)
- ✅ No breaking changes in existing functionality
- ✅ Documentation updated (VERSION_PINS.md, CHANGELOG.md, ADR)

---

## 📦 Upgrade Matrix

| Component | Current | Target | Priority | Risk |
|-----------|---------|--------|----------|------|
| **Keycloak** | 24.0.4 | **26.0.4** | 🔴 HIGH | 🟢 LOW |
| **Vault** | 1.17.6 | **1.18.3** | 🟡 MEDIUM | 🟢 LOW |
| **Postgres** | 16.4-alpine | **17.2-alpine** | 🟢 LOW | 🟢 LOW |
| **Ollama** | 0.12.9 | **0.12.9** | ⚪ KEEP | 🟢 N/A |

### Ollama Version Note
- Current 0.12.9 is likely a custom/future version for qwen3:0.6b support
- Latest public release (GitHub): 0.5.4
- **Decision:** KEEP 0.12.9 (validates correctly in Phase 2.2 smoke tests)
- **Action:** Document version discrepancy in VERSION_PINS.md

---

## 🔧 Workstream Breakdown

### Workstream A: Infrastructure Upgrade (~2 hours)

**Objective:** Update Docker images and configuration files

**Tasks:**

**A1. Update VERSION_PINS.md** (~15 min)
- Update Keycloak: 24.0.4 → 26.0.4
- Update Vault: 1.17.6 → 1.18.3
- Update Postgres: 16.4-alpine → 17.2-alpine
- Add note on Ollama 0.12.9 (custom version for qwen3 support)

**A2. Update docker-compose configuration** (~30 min)
- File: `deploy/compose/ce.dev.yml`
- Update image tags:
  ```yaml
  keycloak:
    image: quay.io/keycloak/keycloak:26.0.4
  
  vault:
    image: hashicorp/vault:1.18.3
  
  postgres:
    image: postgres:17.2-alpine
  
  ollama:
    image: ollama/ollama:0.12.9  # Keep current
  ```

**A3. Pull new Docker images** (~15 min)
```bash
docker compose -f deploy/compose/ce.dev.yml pull keycloak vault postgres
```

**A4. Restart services** (~30 min)
```bash
docker compose -f deploy/compose/ce.dev.yml down
docker compose -f deploy/compose/ce.dev.yml up -d
```

**A5. Verify health checks** (~15 min)
```bash
docker compose -f deploy/compose/ce.dev.yml ps
# All services should show (healthy)
```

**Deliverables:**
- ✅ Updated VERSION_PINS.md
- ✅ Updated ce.dev.yml
- ✅ All services running and healthy

---

### Workstream B: Validation - Phase 1.2 (JWT Auth) (~1 hour)

**Objective:** Ensure Keycloak 26.0.4 upgrade didn't break OIDC/JWT flows

**Tasks:**

**B1. Re-run Phase 1.2 smoke tests** (~30 min)
- File: `docs/tests/smoke-phase1.2.md`
- Test: OIDC login flow
- Test: JWT token verification (RS256)
- Test: JWKS endpoint caching
- Test: Clock skew tolerance (60s)

**B2. Controller /status endpoint** (~10 min)
```bash
curl http://localhost:8088/status
# Should return 200 OK
```

**B3. Controller /audit/ingest with JWT** (~20 min)
```bash
# Get JWT token from Keycloak
TOKEN=$(curl -X POST \
  http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "client_id=goose-controller" \
  -d "grant_type=client_credentials" \
  -d "client_secret=<secret>" \
  | jq -r '.access_token')

# Test protected endpoint
curl -X POST http://localhost:8088/audit/ingest \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "source": "test",
    "category": "test",
    "action": "test"
  }'
# Should return 202 Accepted
```

**Expected Results:**
- ✅ All Phase 1.2 smoke tests pass
- ✅ JWT verification middleware works
- ✅ JWKS caching functional
- ✅ No OIDC-related errors in logs

**Deliverables:**
- ✅ Validation report: `Phase-2.5-Keycloak-Validation.md`

---

### Workstream C: Validation - Phase 2.2 (Privacy Guard) (~1.5 hours)

**Objective:** Ensure Vault 1.18.3 + Postgres 17.2 upgrades didn't break Privacy Guard

**Tasks:**

**C1. Verify Vault pseudo_salt path** (~15 min)
```bash
docker exec ce_vault vault kv get secret/pseudonymization
# Should show pseudo_salt key
```

**C2. Re-run Phase 2.2 smoke tests** (~45 min)
- File: `docs/tests/smoke-phase2.2.md`
- Test 1: Model Status Check (model_enabled, model_name)
- Test 2: Model-Enhanced Detection (person names)
- Test 3: Graceful Fallback (model disabled)
- Test 4: Performance Benchmarking (P50 ~23s acceptable)
- Test 5: Backward Compatibility (Phase 2 functionality)

**C3. Privacy Guard /status endpoint** (~10 min)
```bash
curl http://localhost:8089/status
# Should show model_enabled=true, model_name=qwen3:0.6b
```

**C4. Deterministic pseudonymization test** (~20 min)
```bash
# Test 1: Mask text
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "John Smith SSN: 123-45-6789",
    "tenant_id": "test-tenant",
    "session_id": "test-session"
  }'
# Should return masked text with pseudonyms

# Test 2: Same input should give same pseudonym
# (run Test 1 again, compare pseudonyms)
```

**Expected Results:**
- ✅ All Phase 2.2 smoke tests pass (5/5)
- ✅ Vault integration works (pseudo_salt accessible)
- ✅ Model detection functional (qwen3:0.6b via Ollama)
- ✅ Performance acceptable (P50 ~23s CPU-only)

**Deliverables:**
- ✅ Validation report: `Phase-2.5-Privacy-Guard-Validation.md`

---

### Workstream D: Development Tools Upgrade (~30 min)

**Objective:** Verify and upgrade Python/Rust toolchain for Phase 3 development

**Tasks:**

**D1. Update VERSION_PINS.md with dev tools** (~10 min)
- Add section: "Development Tools (Phase 3+)"
- Document Python 3.13.9 (python:3.13-slim Docker image)
- Document Rust 1.91.0 (rust:1.91.0-bookworm Docker image)
- Note: System Python 3.12.3 compatible but Docker preferred

**D2. Pull latest dev tool Docker images** (~10 min)
```bash
docker pull python:3.13-slim
docker pull rust:1.91.0-bookworm
```

**D3. Test Rust 1.91.0 compilation** (~10 min)
```bash
# Test Controller build with Rust 1.91.0
docker run --rm -v $(pwd):/workspace -w /workspace/src/controller \
  rust:1.91.0-bookworm \
  cargo check --release

# Should complete without errors
```

**Expected Results:**
- ✅ Python 3.13-slim Docker image available locally
- ✅ Rust 1.91.0 Docker image available locally
- ✅ Controller code compiles successfully with Rust 1.91.0
- ✅ VERSION_PINS.md documents dev tool versions

**Deliverables:**
- ✅ Updated VERSION_PINS.md (dev tools section)
- ✅ Docker images pulled and validated

---

### Workstream E: Documentation & ADR (~1 hour)

**Objective:** Update project documentation and create architectural decision record

**Tasks:**

**D1. Update VERSION_PINS.md** (~15 min)
- Document new versions
- Add upgrade reasoning
- Note Ollama version discrepancy

**D2. Update CHANGELOG.md** (~15 min)
```markdown
## [Unreleased]

### Changed (Phase 2.5)
- Upgraded Keycloak 24.0.4 → 26.0.4 (security CVE fixes)
- Upgraded Vault 1.17.6 → 1.18.3 (latest LTS)
- Upgraded Postgres 16.4 → 17.2 (latest stable, 5-year LTS)
- Validated Phase 1.2 (JWT auth) and Phase 2.2 (Privacy Guard) functionality

### Security (Phase 2.5)
- Fixed Keycloak CVE-2024-8883 (HIGH severity)
- Fixed Keycloak CVE-2024-7318 (MEDIUM severity)
- Fixed Keycloak CVE-2024-8698 (MEDIUM severity)
```

**D3. Create ADR-0023: Dependency LTS Policy** (~30 min)
- File: `docs/adr/0023-dependency-lts-policy.md`
- Content:
  - Decision: Use latest LTS/stable versions for all infrastructure
  - Rationale: Security, performance, avoid version lag
  - Consequences: Quarterly review of versions
  - Alternatives considered: Pin to older versions, manual upgrades only

**D4. Create validation summary** (~10 min)
- File: `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Validation-Summary.md`
- Include: Test results, performance metrics, issues found (if any)

**Deliverables:**
- ✅ Updated VERSION_PINS.md
- ✅ Updated CHANGELOG.md
- ✅ ADR-0023 created
- ✅ Validation summary document

---

## 📊 Timeline

**Total Estimated Effort:** ~6 hours (0.75 days)

```
Hour 1-2: Workstream A (Infrastructure Upgrade)
  - Update VERSION_PINS.md (infrastructure section)
  - Update docker-compose files
  - Pull images, restart services
  - Verify health checks

Hour 2-3: Workstream B (Phase 1.2 Validation)
  - Re-run OIDC/JWT smoke tests
  - Test controller endpoints
  - Validate JWKS caching

Hour 3-4.5: Workstream C (Phase 2.2 Validation)
  - Verify Vault integration
  - Re-run Privacy Guard smoke tests (5 tests)
  - Performance benchmarking
  - Model detection validation

Hour 4.5-5: Workstream D (Development Tools Upgrade)
  - Update VERSION_PINS.md (dev tools section)
  - Pull Python 3.13-slim and Rust 1.91.0 Docker images
  - Test Rust 1.91.0 compilation

Hour 5-6: Workstream E (Documentation)
  - Update CHANGELOG.md
  - Create ADR-0023
  - Write validation summary
```

---

## 🎯 Milestones

**M1 (Hour 2):** All infrastructure services upgraded and healthy  
**M2 (Hour 3):** Phase 1.2 validation complete (Keycloak 26.0.4 works)  
**M3 (Hour 4.5):** Phase 2.2 validation complete (Vault 1.18.3 + Postgres 17.2 work)  
**M4 (Hour 5):** Development tools verified (Python 3.13, Rust 1.91)  
**M5 (Hour 6):** Documentation complete, Phase 2.5 ready to merge

---

## ⚠️ Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Keycloak OIDC breaking change** | LOW | HIGH | Phase 1.2 tests validate; rollback to 24.0.4 if fails |
| **Postgres 16→17 schema incompatibility** | LOW | MEDIUM | No data/schema yet; fresh install safe |
| **Vault KV engine API change** | LOW | MEDIUM | Phase 2.2 tests validate; rollback to 1.17.6 if fails |
| **Ollama model compatibility** | LOW | MEDIUM | qwen3:0.6b format unchanged; Phase 2.2 tests validate |
| **Docker image pull failures** | LOW | LOW | Retry, use Docker Hub mirror if needed |

**Rollback Strategy:**
```bash
# If validation fails, rollback to previous versions
git checkout VERSION_PINS.md
git checkout deploy/compose/ce.dev.yml
docker compose -f deploy/compose/ce.dev.yml down
docker compose -f deploy/compose/ce.dev.yml up -d
```

---

## 📝 Acceptance Criteria

### Must Pass
- ✅ All Docker services start and pass health checks
- ✅ Phase 1.2 smoke tests: 100% pass (OIDC/JWT)
- ✅ Phase 2.2 smoke tests: 100% pass (Privacy Guard, 5/5 tests)
- ✅ No performance regression (P50 latency within 10% of baseline)
- ✅ VERSION_PINS.md updated
- ✅ CHANGELOG.md updated
- ✅ ADR-0023 created

### Nice to Have
- ⭐ Performance improvement (Postgres 17.2 faster queries)
- ⭐ Keycloak startup time improvement (26.0.4 ~15% faster)

---

## 🔗 Dependencies

### Upstream (Completed)
- ✅ Phase 0: Infrastructure bootstrap
- ✅ Phase 1.2: JWT verification middleware
- ✅ Phase 2.2: Privacy Guard with Ollama model

### Downstream (Blocked Until Complete)
- ⏸️ Phase 3: Controller API + Agent Mesh
  - **Requirement:** Ensure Keycloak 26.0.4 JWT verification works
  - **Requirement:** Ensure Privacy Guard Vault integration stable

---

## 📚 Reference Documents

### External Documentation
- **Keycloak 26.0.4 Release Notes:** https://www.keycloak.org/docs/latest/release_notes/
- **Vault 1.18.3 Release Notes:** https://developer.hashicorp.com/vault/docs/updates/release-notes
- **Postgres 17.2 Release Notes:** https://www.postgresql.org/about/news/postgresql-18-released-3142/

### Internal Documentation
- **VERSION_PINS.md:** Current version pins
- **Phase 1.2 Smoke Tests:** `docs/tests/smoke-phase1.2.md`
- **Phase 2.2 Smoke Tests:** `docs/tests/smoke-phase2.2.md`
- **Phase 1.2 Completion Summary:** `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Completion-Summary.md`
- **Phase 2.2 Completion Summary:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`

---

## 🚀 Execution Workflow

### 1. Pre-Execution Checklist
- [ ] Review this execution plan
- [ ] Ensure git working directory is clean
- [ ] Backup current docker-compose state
- [ ] Note current Docker image hashes (for rollback)

### 2. Execute Workstreams
- [ ] Workstream A: Infrastructure Upgrade
- [ ] Workstream B: Phase 1.2 Validation
- [ ] Workstream C: Phase 2.2 Validation
- [ ] Workstream D: Development Tools Upgrade
- [ ] Workstream E: Documentation

### 3. Post-Execution
- [ ] Create Phase-2.5-Completion-Summary.md
- [ ] Commit changes (conventional commit format)
- [ ] Optional: Create PR for review (or merge directly to main)
- [ ] Update Phase-2.5-Agent-State.json (final state)

---

**Prepared by:** Goose AI Agent  
**Date:** 2025-11-04  
**Status:** READY FOR EXECUTION  
**Next Action:** User review & approval, then execute workstreams
