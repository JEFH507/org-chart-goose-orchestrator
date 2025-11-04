# Phase 2.5 Orchestration Prompt

**Copy this entire prompt to a new Goose session to execute Phase 2.5**

---

## 🔄 Resume Prompt — Copy this block if resuming Phase 2.5

```markdown
You are resuming Phase 2.5 orchestration for goose-org-twin.

**Context:**
- Phase: 2.5 — Dependency Security & LTS Upgrades (Small - ~6 hours)
- Repository: /home/papadoc/Gooseprojects/goose-org-twin

**Required Actions:**
1. Read state from: `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Agent-State.json`
2. Read last progress entry from: `docs/tests/phase2.5-progress.md` (if exists)
3. Re-read authoritative documents:
   - `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Execution-Plan.md`
   - `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Checklist.md`
   - `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Orchestration-Prompt.md` (this file)
   - `Technical Project Plan/PM Phases/Phase-2.5/DEPENDENCY-RESEARCH.md`

**Summarize for me:**
- Current workstream and task from state JSON
- Last step completed (from progress.md or state JSON)
- Checklist progress (X/22 tasks complete)
- Pending questions (if any in state JSON)

**Then proceed with:**
- If pending_questions exist: ask them and wait for my answers
- Otherwise: continue with the next unchecked task in the checklist
- Update state JSON and progress log after each task/milestone

**Guardrails:**
- HTTP-only orchestrator; metadata-only server model
- No secrets in git; .env samples only
- Update state JSON and progress log after each milestone
- Create ADR-0023 before marking phase complete
```

---

## 🚀 Master Orchestration Prompt — Copy this block for new session

You are executing **Phase 2.5: Dependency Security & LTS Upgrades** for the goose-org-twin project.

## 📋 Context

**Project:** goose-org-twin (Multi-agent orchestration system)  
**Repository:** git@github.com:JEFH507/org-chart-goose-orchestrator.git  
**Current Branch:** main  
**Phase:** 2.5 (Dependency Security & LTS Upgrades)  
**Priority:** 🔴 HIGH (Keycloak has critical CVE-2024-8883)  
**Estimated Effort:** 6 hours (can complete same day)

### Prerequisites (Completed)
- ✅ Phase 0: Infrastructure bootstrap
- ✅ Phase 1: Basic Controller skeleton
- ✅ Phase 1.2: JWT verification middleware
- ✅ Phase 2: Vault integration
- ✅ Phase 2.2: Privacy Guard with Ollama model

### Blocks
- ⏸️ Phase 3: Controller API + Agent Mesh (waiting for this phase)

---

## 🎯 Objectives

### Primary Goal
Upgrade infrastructure and development dependencies to latest LTS/stable versions:

**Infrastructure (Runtime):**
1. **Security:** Patch critical Keycloak CVEs (CVE-2024-8883 HIGH, CVE-2024-7318 MED, CVE-2024-8698 MED)
2. **Performance:** Postgres 17.2 improvements, Vault 1.18.3 stability
3. **Maintainability:** Reduce technical debt, ensure 5-year LTS for Postgres

**Development Tools (Phase 3 Prep):**
4. **Python 3.13.9:** Latest stable for Agent Mesh MCP server (Phase 3)
5. **Rust 1.91.0:** Latest stable for Controller API development (8 versions upgrade)

### Success Criteria
- ✅ All Docker services upgraded and healthy (Keycloak, Vault, Postgres, Ollama)
- ✅ Phase 1.2 smoke tests pass (JWT auth with Keycloak 26.0.4)
- ✅ Phase 2.2 smoke tests pass (Privacy Guard with Vault 1.18.3 + Postgres 17.2)
- ✅ Development tools verified (Python 3.13, Rust 1.91 Docker images)
- ✅ Controller compiles successfully with Rust 1.91.0
- ✅ VERSION_PINS.md updated (infrastructure + dev tools sections)
- ✅ CHANGELOG.md updated
- ✅ **ADR-0023 created: "Dependency LTS Policy"**
- ✅ No performance regression (P50 latency within 10%)

---

## 📦 Upgrade Matrix

| Component | Current | Target | Priority | Type |
|-----------|---------|--------|----------|------|
| **Keycloak** | 24.0.4 | **26.0.4** | 🔴 HIGH | Runtime (Security CVEs) |
| **Vault** | 1.17.6 | **1.18.3** | 🟡 MEDIUM | Runtime (Latest LTS) |
| **Postgres** | 16.4-alpine | **17.2-alpine** | 🟢 LOW | Runtime (Performance + LTS) |
| **Ollama** | 0.12.9 | **0.12.9** | ✅ KEEP | Runtime (Already latest) |
| **Python** | 3.12.3 (sys) | **3.13.9** (Docker) | 🟡 MEDIUM | Dev Tool (Phase 3) |
| **Rust** | 1.83.0 (Docker) | **1.91.0** (Docker) | 🟡 MEDIUM | Dev Tool (8 versions behind) |

---

## 🔧 Execution Plan

### Workstream A: Infrastructure Upgrade (~2 hours)

**A1. Update VERSION_PINS.md infrastructure section** (~15 min)
- Update Keycloak: 24.0.4 → 26.0.4
- Update Vault: 1.17.6 → 1.18.3
- Update Postgres: 16.4-alpine → 17.2-alpine
- Document Ollama 0.12.9 rationale (already latest, verified 2025-10-31)

**A2. Update deploy/compose/ce.dev.yml** (~30 min)
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
cd /home/papadoc/Gooseprojects/goose-org-twin
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

# Verify individual services
curl -f http://localhost:8080/health/ready  # Keycloak
curl -f http://localhost:8200/v1/sys/health  # Vault (200 or 429)
docker exec ce_postgres pg_isready  # Postgres
curl -f http://localhost:11434/api/version  # Ollama
```

**Deliverables:**
- ✅ Updated VERSION_PINS.md (infrastructure section)
- ✅ Updated ce.dev.yml
- ✅ All services running and healthy

---

### Workstream B: Phase 1.2 Validation (~1 hour)

**B1. Re-run Phase 1.2 smoke tests** (~30 min)

Reference: `docs/tests/smoke-phase1.2.md`

Test OIDC login flow and JWT verification:
```bash
# Get JWT token from Keycloak 26.0.4
TOKEN=$(curl -X POST \
  http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "client_id=goose-controller" \
  -d "grant_type=client_credentials" \
  -d "client_secret=<secret>" \
  | jq -r '.access_token')

echo "JWT Token obtained: ${TOKEN:0:50}..."
```

**B2. Test Controller /status endpoint** (~10 min)
```bash
curl -f http://localhost:8088/status
# Expected: 200 OK
```

**B3. Test Controller /audit/ingest with JWT** (~20 min)
```bash
curl -X POST http://localhost:8088/audit/ingest \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "source": "phase-2.5-validation",
    "category": "test",
    "action": "keycloak_26_verification"
  }'
# Expected: 202 Accepted
```

**Expected Results:**
- ✅ OIDC token endpoint responsive
- ✅ JWT verification middleware works with Keycloak 26.0.4
- ✅ JWKS caching functional
- ✅ No OIDC/JWT errors in controller logs

**Deliverables:**
- ✅ Create validation report: `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Keycloak-Validation.md`

---

### Workstream C: Phase 2.2 Validation (~1.5 hours)

**C1. Verify Vault pseudo_salt path** (~15 min)
```bash
docker exec ce_vault vault kv get secret/pseudonymization
# Should show pseudo_salt key
```

**C2. Re-run Phase 2.2 smoke tests** (~45 min)

Reference: `docs/tests/smoke-phase2.2.md`

Run all 5 tests:
1. Model Status Check (model_enabled=true, model_name=qwen3:0.6b)
2. Model-Enhanced Detection (person names detected)
3. Graceful Fallback (works when model disabled)
4. Performance Benchmarking (P50 ~23s acceptable)
5. Backward Compatibility (Phase 2 functionality intact)

**C3. Test Privacy Guard /status endpoint** (~10 min)
```bash
curl http://localhost:8089/status
# Expected: {"model_enabled": true, "model_name": "qwen3:0.6b", ...}
```

**C4. Test deterministic pseudonymization** (~20 min)
```bash
# Test 1: Mask text with PII
RESULT1=$(curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "John Smith SSN: 123-45-6789",
    "tenant_id": "test-tenant",
    "session_id": "test-session"
  }')

echo "First result: $RESULT1"

# Test 2: Same input should give same pseudonym
RESULT2=$(curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "John Smith SSN: 123-45-6789",
    "tenant_id": "test-tenant",
    "session_id": "test-session"
  }')

echo "Second result: $RESULT2"

# Compare pseudonyms (should be identical)
```

**Expected Results:**
- ✅ Vault integration works (pseudo_salt accessible via KV engine)
- ✅ Ollama model detection functional (qwen3:0.6b)
- ✅ Performance acceptable (P50 ~23s CPU-only)
- ✅ Deterministic pseudonymization (same input → same output)
- ✅ All 5 smoke tests pass

**Deliverables:**
- ✅ Create validation report: `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Privacy-Guard-Validation.md`

---

### Workstream D: Development Tools Upgrade (~30 min)

**D1. Update VERSION_PINS.md with dev tools section** (~10 min)

Add new section to VERSION_PINS.md:

```markdown
## Development Tools (Phase 3+)

### Python Runtime - Agent Mesh MCP Server
- **Docker Image:** python:3.13-slim
- **Version:** Python 3.13.9 (released 2025-11-04)
- **EOL:** 2029-10 (5-year support)
- **Use:** Agent Mesh MCP server (Phase 3), future Python-based extensions
- **Note:** System Python 3.12.3 compatible but Docker image preferred for consistency

### Rust Toolchain - Controller API & Extensions
- **Docker Image:** rust:1.91.0-bookworm (or rust:1.91.0-slim for smaller builds)
- **Version:** rustc 1.91.0 (f8297e351 2025-10-28)
- **Release Cycle:** 6-week rolling stable releases
- **Use:** Controller API, Privacy Guard, Rust-based MCP extensions
- **Cargo Edition:** 2021 (Cargo.toml edition field)
- **Note:** Upgraded from local rust:1.83.0 image (8 minor versions behind)
```

**D2. Pull dev tool Docker images** (~10 min)
```bash
docker pull python:3.13-slim
docker pull rust:1.91.0-bookworm
```

**D3. Test Rust 1.91.0 compilation** (~10 min)
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Test Controller build with Rust 1.91.0
docker run --rm \
  -v $(pwd):/workspace \
  -w /workspace/src/controller \
  rust:1.91.0-bookworm \
  cargo check --release

# Expected: Cargo check completes without errors
```

**Expected Results:**
- ✅ Python 3.13-slim Docker image available locally
- ✅ Rust 1.91.0-bookworm Docker image available locally
- ✅ Controller code compiles successfully with Rust 1.91.0
- ✅ VERSION_PINS.md documents dev tool versions

**If Rust 1.91.0 Compilation Fails:**
- Document breaking changes in validation summary
- Note required code updates for Phase 3
- Consider keeping Rust 1.83.0 if critical blockers exist

**Deliverables:**
- ✅ Updated VERSION_PINS.md (dev tools section)
- ✅ Docker images pulled and validated
- ✅ Rust compilation test results documented

---

### Workstream E: Documentation (~1 hour)

**E1. Update CHANGELOG.md** (~15 min)

Add to `CHANGELOG.md`:

```markdown
## [Unreleased]

### Changed (Phase 2.5 - 2025-11-04)
- Upgraded Keycloak 24.0.4 → 26.0.4 (security CVE fixes)
- Upgraded Vault 1.17.6 → 1.18.3 (latest LTS)
- Upgraded Postgres 16.4 → 17.2 (latest stable, 5-year LTS)
- Upgraded Python dev tools: 3.12.3 → 3.13.9 (Docker image)
- Upgraded Rust dev tools: 1.83.0 → 1.91.0 (Docker image)
- Validated Ollama 0.12.9 is latest stable (released 2025-10-31)
- Validated Phase 1.2 (JWT auth) functionality with new dependencies
- Validated Phase 2.2 (Privacy Guard) functionality with new dependencies

### Security (Phase 2.5)
- **Fixed:** Keycloak CVE-2024-8883 (HIGH severity - session fixation)
- **Fixed:** Keycloak CVE-2024-7318 (MEDIUM severity - authorization bypass)
- **Fixed:** Keycloak CVE-2024-8698 (MEDIUM severity - XSS vulnerability)
```

**E2. Create ADR-0023: Dependency LTS Policy** (~30 min)

**CRITICAL:** Create `docs/adr/0023-dependency-lts-policy.md`

```markdown
# ADR-0023: Dependency LTS Policy

**Date:** 2025-11-04  
**Status:** Accepted  
**Context:** Phase 2.5 (Dependency Security & LTS Upgrades)  
**Deciders:** Product Team, Engineering Team

## Context

The goose-org-twin project relies on multiple infrastructure and development dependencies. During Phase 2.5 planning, we identified version lag:

- Keycloak: 6 months behind (24.0.4 vs 26.0.4) with HIGH severity CVEs
- Vault: 4 months behind (1.17.6 vs 1.18.3 LTS)
- Postgres: 3 months behind (16.4 vs 17.2 stable)
- Rust: 8 minor versions behind (1.83.0 vs 1.91.0)
- Python: 1 minor version behind (3.12.3 vs 3.13.9)

Version lag creates:
- **Security risks:** Unpatched CVEs (e.g., Keycloak CVE-2024-8883 HIGH)
- **Performance issues:** Missing optimizations (e.g., Postgres 17 improvements)
- **Technical debt:** Harder to upgrade later (breaking changes accumulate)
- **Support concerns:** LTS timelines (e.g., Postgres 17 has 5-year support)

## Decision

We will maintain dependencies at **latest LTS or stable versions** as follows:

### Infrastructure (Runtime Dependencies)
- **Keycloak:** Latest stable release
- **Vault:** Latest LTS version (per HashiCorp LTS tracker)
- **Postgres:** Latest stable release with 5+ year LTS
- **Ollama:** Latest stable release (or custom version if needed for specific models)

### Development Tools
- **Rust:** Latest stable toolchain (6-week release cycle)
- **Python:** Latest stable release with 5-year support window

### Review Cadence
- **Quarterly:** Review all dependencies for new LTS/stable releases
- **Ad-hoc:** Immediate upgrade if HIGH/CRITICAL CVE discovered
- **Pre-Phase:** Check versions before starting major development phases

### Upgrade Triggers
1. **Security:** HIGH or CRITICAL CVE in current version
2. **LTS Transition:** New LTS release available (e.g., Vault 1.18 → 1.19)
3. **Performance:** Significant performance improvements (>20%)
4. **Deprecation:** Current version nearing EOL (<6 months)

## Rationale

### Why Latest LTS/Stable?
- **Security:** Patches for known vulnerabilities
- **Performance:** Benefit from optimizations
- **Support:** Vendor support for 3-5 years minimum
- **Compatibility:** Avoid breaking changes from multi-version jumps

### Why Not Bleeding Edge?
- **Stability:** Avoid nightly/beta versions (unpredictable breaking changes)
- **Testing:** Latest stable has broader community testing

### Why Quarterly Reviews?
- **Balance:** Not too frequent (disruptive) nor too infrequent (lag accumulates)
- **Predictable:** Aligns with typical LTS release schedules
- **Phase-aligned:** Coincides with planning for new development phases

## Consequences

### Positive
- ✅ Reduced security risk (timely CVE patches)
- ✅ Improved performance (latest optimizations)
- ✅ Better support (within vendor LTS windows)
- ✅ Easier upgrades (smaller version jumps)

### Negative
- ❌ Upgrade overhead (~1 day per quarter for testing/validation)
- ❌ Potential breaking changes (though mitigated by LTS choice)
- ❌ Need for regression testing (Phase 1.2 + Phase 2.2 validation)

### Mitigations
- **Testing:** Always re-run smoke tests for affected phases
- **Rollback:** Keep previous Docker image tags for quick rollback
- **Documentation:** Update VERSION_PINS.md + CHANGELOG.md every upgrade
- **Validation:** Include upgrade in phase planning (e.g., Phase 2.5)

## Alternatives Considered

### Alternative 1: Pin to Specific Versions (Never Upgrade)
- ❌ **Rejected:** Accumulates security debt, EOL risk

### Alternative 2: Upgrade Only on Breaking Issues
- ❌ **Rejected:** Reactive (not proactive), harder multi-version jumps

### Alternative 3: Always Use Nightly/Beta
- ❌ **Rejected:** Too unstable for production system

## Implementation

### Phase 2.5 (Current)
- Upgrade Keycloak, Vault, Postgres, Python, Rust per this ADR
- Create VERSION_PINS.md with LTS/stable targets
- Validate with Phase 1.2 + Phase 2.2 smoke tests

### Ongoing
- Add quarterly dependency review to project calendar
- Update VERSION_PINS.md after each review
- Create ADR addendums for major version changes (e.g., Postgres 17 → 18)

## References

- **Keycloak Release Notes:** https://www.keycloak.org/docs/latest/release_notes/
- **Vault LTS Tracker:** https://developer.hashicorp.com/vault/docs/updates/lts-tracker
- **Postgres Support Policy:** https://www.postgresql.org/support/versioning/
- **Rust Release Schedule:** https://releases.rs/
- **Python Release Schedule:** https://devguide.python.org/versions/
- **Phase 2.5 Execution Plan:** Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Execution-Plan.md
- **CVE-2024-8883 (Keycloak):** NVD database

---

**Approved by:** Engineering Team  
**Implementation:** Phase 2.5 (2025-11-04)
```

**E3. Create validation summary** (~10 min)

Create `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Validation-Summary.md`:

```markdown
# Phase 2.5 Validation Summary

**Date:** [Completion date]  
**Phase:** 2.5 (Dependency Security & LTS Upgrades)  
**Status:** [PASS/FAIL]

## Test Results

### Infrastructure Upgrades
- ✅/❌ Keycloak 24.0.4 → 26.0.4: [PASS/FAIL]
- ✅/❌ Vault 1.17.6 → 1.18.3: [PASS/FAIL]
- ✅/❌ Postgres 16.4 → 17.2: [PASS/FAIL]
- ✅/❌ Ollama 0.12.9 (kept): [PASS/FAIL]

### Development Tools
- ✅/❌ Python 3.13.9 Docker image: [PASS/FAIL]
- ✅/❌ Rust 1.91.0 Docker image: [PASS/FAIL]
- ✅/❌ Controller compiles with Rust 1.91.0: [PASS/FAIL]

### Phase 1.2 Validation (JWT Auth)
- ✅/❌ OIDC token endpoint: [PASS/FAIL]
- ✅/❌ JWT verification middleware: [PASS/FAIL]
- ✅/❌ JWKS caching: [PASS/FAIL]
- **Overall:** [PASS/FAIL]

### Phase 2.2 Validation (Privacy Guard)
- ✅/❌ Vault pseudo_salt access: [PASS/FAIL]
- ✅/❌ Model detection (qwen3:0.6b): [PASS/FAIL]
- ✅/❌ Performance (P50): [XX.Xs] (baseline: ~23s)
- ✅/❌ Deterministic pseudonymization: [PASS/FAIL]
- ✅/❌ All 5 smoke tests: [X/5] passed
- **Overall:** [PASS/FAIL]

## Performance Metrics
- Keycloak startup time: [XX]s (baseline: ~30s)
- Vault health check: [XX]ms
- Postgres connection: [XX]ms
- Privacy Guard P50 latency: [XX.X]s (baseline: 22.8s)

## Issues Found
[List any issues discovered during validation]

## Recommendations
[Any follow-up actions needed]

## Deliverables Completed
- ✅ Updated VERSION_PINS.md
- ✅ Updated ce.dev.yml
- ✅ Updated CHANGELOG.md
- ✅ Created ADR-0023
- ✅ All services healthy
- ✅ Phase 1.2 tests pass
- ✅ Phase 2.2 tests pass

**Phase 2.5 Status:** [COMPLETE/INCOMPLETE]
```

**E4. Final VERSION_PINS.md review** (~5 min)
- Ensure all sections complete (infrastructure + dev tools)
- Verify version numbers match upgrade matrix
- Check formatting and links

**Deliverables:**
- ✅ Updated CHANGELOG.md
- ✅ **Created ADR-0023: Dependency LTS Policy**
- ✅ Created Phase-2.5-Validation-Summary.md
- ✅ VERSION_PINS.md finalized

---

## 📊 Timeline & Milestones

**Total Effort:** ~6 hours (same day execution)

```
Hour 0-2:   Workstream A (Infrastructure Upgrade)
Hour 2-3:   Workstream B (Phase 1.2 Validation)
Hour 3-4.5: Workstream C (Phase 2.2 Validation)
Hour 4.5-5: Workstream D (Development Tools Upgrade)
Hour 5-6:   Workstream E (Documentation + ADR-0023)
```

### Milestones
- **M1 (Hour 2):** All infrastructure services upgraded and healthy
- **M2 (Hour 3):** Phase 1.2 validation complete (Keycloak 26.0.4 works)
- **M3 (Hour 4.5):** Phase 2.2 validation complete (Vault 1.18.3 + Postgres 17.2 work)
- **M4 (Hour 5):** Development tools verified (Python 3.13, Rust 1.91)
- **M5 (Hour 6):** Documentation complete + **ADR-0023 created**, Phase 2.5 ready to merge

---

## 🚦 Progress Tracking

Update `Phase-2.5-Agent-State.json` after each task:

```bash
# Example: After completing A1
jq '.workstreams.A.tasks_completed += 1 | .progress.completed_tasks += 1' \
  Phase-2.5-Agent-State.json > tmp.json && mv tmp.json Phase-2.5-Agent-State.json
```

Update `Phase-2.5-Checklist.md` by checking off completed tasks.

---

## ⚠️ Important Notes

### Rollback Strategy
If ANY validation fails:
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
git checkout VERSION_PINS.md deploy/compose/ce.dev.yml
docker compose -f deploy/compose/ce.dev.yml down
docker compose -f deploy/compose/ce.dev.yml up -d
```

### Rust 1.91.0 Compatibility
If Controller fails to compile with Rust 1.91.0:
- Document exact error in validation summary
- Note required code changes for Phase 3
- Keep Rust 1.83.0 for now, defer upgrade

### Critical Files to Update
1. **VERSION_PINS.md** (infrastructure + dev tools sections)
2. **deploy/compose/ce.dev.yml** (Docker image tags)
3. **CHANGELOG.md** (upgrade notes + security fixes)
4. **docs/adr/0023-dependency-lts-policy.md** ← **MUST CREATE**

### ADR Reminder
**ADR-0023 is MANDATORY for Phase 2.5 completion.** It documents the policy that justifies this entire phase and sets precedent for future dependency management.

---

## 📝 Git Workflow

### Branch Strategy
```bash
git checkout -b chore/phase-2.5-dependency-upgrades
```

### Commit Messages (Conventional Commits)
```bash
git add VERSION_PINS.md deploy/compose/ce.dev.yml
git commit -m "chore(deps): upgrade Keycloak, Vault, Postgres to latest LTS

- Keycloak: 24.0.4 → 26.0.4 (fixes CVE-2024-8883 HIGH)
- Vault: 1.17.6 → 1.18.3 (latest LTS)
- Postgres: 16.4 → 17.2 (latest stable, 5-year LTS)
- Python: 3.12.3 → 3.13.9 (Docker image for Phase 3)
- Rust: 1.83.0 → 1.91.0 (Docker image, 8 versions upgrade)

Validated with Phase 1.2 (JWT auth) and Phase 2.2 (Privacy Guard) tests.

Closes #[issue-number] (if applicable)
Refs: Technical Project Plan/PM Phases/Phase-2.5/"

git add CHANGELOG.md docs/adr/0023-dependency-lts-policy.md
git commit -m "docs(adr): add ADR-0023 dependency LTS policy

Documents quarterly review cadence and upgrade triggers.
Part of Phase 2.5 dependency security upgrades."

git add Technical\ Project\ Plan/PM\ Phases/Phase-2.5/
git commit -m "docs(phase-2.5): add validation reports and summary

- Phase-2.5-Keycloak-Validation.md
- Phase-2.5-Privacy-Guard-Validation.md
- Phase-2.5-Validation-Summary.md"
```

### Merge to Main
```bash
# If no PR needed (or after PR approval)
git checkout main
git merge --squash chore/phase-2.5-dependency-upgrades
git commit -m "chore(phase-2.5): dependency security & LTS upgrades [COMPLETE]

Summary:
- Infrastructure: Keycloak 26.0.4, Vault 1.18.3, Postgres 17.2
- Dev Tools: Python 3.13.9, Rust 1.91.0 (Docker images)
- Security: Fixed Keycloak CVE-2024-8883 (HIGH)
- Validation: Phase 1.2 + Phase 2.2 tests pass
- ADR-0023: Dependency LTS policy established

Phase 2.5 complete. Unblocks Phase 3 (Controller API + Agent Mesh)."

git push origin main
```

---

## ✅ Completion Checklist

Before marking Phase 2.5 complete:

- [ ] All 5 workstreams executed (A, B, C, D, E)
- [ ] All Docker services healthy
- [ ] Phase 1.2 tests pass (100%)
- [ ] Phase 2.2 tests pass (5/5 smoke tests)
- [ ] Rust 1.91.0 compilation tested
- [ ] VERSION_PINS.md updated (infrastructure + dev tools)
- [ ] CHANGELOG.md updated
- [ ] **ADR-0023 created and committed** ← CRITICAL
- [ ] Validation reports created (Keycloak, Privacy Guard, summary)
- [ ] Phase-2.5-Agent-State.json final state: status="COMPLETE"
- [ ] Phase-2.5-Checklist.md: 22/22 tasks complete
- [ ] Git commits created (conventional format)
- [ ] Merged to main branch
- [ ] Create Phase-2.5-Completion-Summary.md

---

## 📚 Reference Documents

### Execution Plan
- **Full Details:** `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Execution-Plan.md`
- **Checklist:** `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Checklist.md`
- **State Tracking:** `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Agent-State.json`

### Validation References
- **Phase 1.2 Smoke Tests:** `docs/tests/smoke-phase1.2.md`
- **Phase 2.2 Smoke Tests:** `docs/tests/smoke-phase2.2.md`
- **Phase 1.2 Summary:** `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Completion-Summary.md`
- **Phase 2.2 Summary:** `Technical Project Plan/PM Phases/Phase-2.2/Phase-2.2-Completion-Summary.md`

### External Documentation
- **Keycloak 26.0.4:** https://www.keycloak.org/docs/latest/release_notes/
- **Vault 1.18.3:** https://developer.hashicorp.com/vault/docs/updates/release-notes
- **Postgres 17.2:** https://www.postgresql.org/about/news/postgresql-18-released-3142/
- **Python 3.13:** https://devguide.python.org/versions/
- **Rust 1.91:** https://releases.rs/

---

## 🎯 Success Criteria (Final Check)

At the end of Phase 2.5, confirm:

- ✅ **Security:** Keycloak CVE-2024-8883 (HIGH) patched
- ✅ **Stability:** All services healthy, no errors in logs
- ✅ **Validation:** Phase 1.2 + Phase 2.2 functionality intact
- ✅ **Performance:** No regression (P50 within 10% of baseline)
- ✅ **Documentation:** VERSION_PINS.md, CHANGELOG.md, **ADR-0023**, validation reports
- ✅ **Readiness:** Phase 3 unblocked (stable infrastructure + latest dev tools)

**If all ✅, Phase 2.5 is COMPLETE.** Proceed to Phase 3 (Controller API + Agent Mesh).

---

**Orchestrated by:** Goose AI Agent  
**Date:** 2025-11-04  
**Execution Time:** ~6 hours  
**Next Phase:** Phase 3 (check Phase-2.5/ folder for any changes first)
