# ADR-0023: Dependency LTS Policy

**Date:** 2025-11-04  
**Status:** Accepted  
**Context:** Phase 2.5 (Dependency Security & LTS Upgrades)  
**Deciders:** Product Team, Engineering Team

---

## Context

The goose-org-twin project relies on multiple infrastructure and development dependencies. During Phase 2.5 planning, we identified version lag across the stack:

### Version Lag Identified (2025-11-04)

| Component | Current | Latest | Lag | Priority |
|-----------|---------|--------|-----|----------|
| **Keycloak** | 24.0.4 | 26.0.4 | 6 months | 🔴 HIGH (CVE-2024-8883 HIGH) |
| **Vault** | 1.17.6 | 1.18.3 | 4 months | 🟡 MEDIUM (LTS upgrade) |
| **Postgres** | 16.4 | 17.2 | 3 months | 🟢 LOW (performance + LTS) |
| **Ollama** | 0.12.9 | 0.12.9 | 0 | ✅ Current |
| **Rust (dev)** | 1.83.0 | 1.91.0 | 8 versions | 🟡 MEDIUM (dev tool) |
| **Python (dev)** | 3.12.3 | 3.13.9 | 1 minor | 🟢 LOW (dev tool) |

### Risks of Version Lag

1. **Security vulnerabilities:** Unpatched CVEs (e.g., Keycloak CVE-2024-8883 HIGH severity)
2. **Performance degradation:** Missing optimizations (e.g., Postgres 17 JSON improvements)
3. **Technical debt accumulation:** Harder to upgrade later (breaking changes compound)
4. **Support concerns:** Approaching/exceeding EOL dates (e.g., Postgres 16 LTS shorter than 17)
5. **Ecosystem drift:** Incompatibility with newer libraries and tooling

### Current Process Gaps

- **No formal upgrade policy:** Ad-hoc upgrades when blockers occur
- **No review cadence:** Dependencies checked only during phase planning
- **No upgrade triggers:** No defined criteria for when to upgrade
- **No LTS tracking:** Inconsistent awareness of LTS release schedules

---

## Decision

We will maintain dependencies at **latest LTS or stable versions** with quarterly reviews and defined upgrade triggers.

### Dependency Targets

#### Infrastructure (Runtime Dependencies)

| Component | Target | Rationale | LTS/Stable |
|-----------|--------|-----------|------------|
| **Keycloak** | Latest stable | Fast security releases, no LTS concept | Stable |
| **Vault** | Latest LTS | HashiCorp LTS tracker (12-month cycles) | LTS |
| **Postgres** | Latest stable | 5-year LTS per version | LTS (5 years) |
| **Ollama** | Latest stable | Rapid AI model ecosystem evolution | Stable |
| **SeaweedFS/MinIO** | Latest stable | S3 compatibility requirements | Stable |

**Policy:** Upgrade to latest LTS or stable within **1 quarter of release**, unless blockers exist.

---

#### Development Tools

| Component | Target | Rationale | Support Window |
|-----------|--------|-----------|----------------|
| **Rust** | Latest stable | 6-week rolling releases, excellent backward compatibility | Stable |
| **Python** | Latest stable | 5-year support per version | 5 years |
| **Docker** | Latest stable | Critical for dev/prod parity | Stable |
| **Node.js (future)** | Latest LTS | 3-year LTS cycles (even versions) | LTS (3 years) |

**Policy:** Upgrade to latest stable within **2 quarters of release**, test early to identify breaking changes.

---

### Review Cadence

**Quarterly Dependency Review** (every 3 months):

**When:** Last week of each quarter (March, June, September, December)

**Process:**
1. **Inventory:** Check current versions vs latest LTS/stable
2. **Prioritize:** Categorize by urgency (HIGH/MEDIUM/LOW)
3. **Plan:** Schedule upgrades in next phase if within targets
4. **Document:** Update VERSION_PINS.md with findings

**Quarterly Review Checklist:**
- [ ] Check Keycloak latest stable release notes
- [ ] Check Vault LTS tracker (https://developer.hashicorp.com/vault/docs/updates/lts-tracker)
- [ ] Check Postgres release calendar
- [ ] Check Ollama GitHub releases
- [ ] Check Rust release schedule (https://releases.rs/)
- [ ] Check Python release schedule (https://devguide.python.org/versions/)
- [ ] Review EOL calendars (https://endoflife.date/)
- [ ] Update VERSION_PINS.md with review date and findings

---

### Upgrade Triggers

Upgrades are **mandatory** if ANY of these triggers occur:

#### 1. Security (Immediate)

- **HIGH or CRITICAL CVE** in current version
- **Publicly disclosed exploit** with proof-of-concept
- **Zero-day vulnerability** announced

**Action:** Emergency upgrade within **1 week**, out-of-band from quarterly schedule.

**Example:** Keycloak CVE-2024-8883 (HIGH) → triggered Phase 2.5 upgrade.

---

#### 2. LTS Transition (Next Quarter)

- **New LTS version released** (e.g., Vault 1.18 → 1.19 LTS)
- **Current version approaching EOL** (<6 months remaining)
- **Vendor deprecation warning** for current version

**Action:** Plan upgrade in next development phase.

**Example:** Postgres 17 LTS (5-year support) vs Postgres 16 (shorter support).

---

#### 3. Performance (Next Quarter)

- **Significant performance improvements** (>20% throughput gain)
- **New features critical to roadmap** (e.g., Postgres JSON operators for Phase 3)
- **Resource efficiency gains** (>15% memory/CPU reduction)

**Action:** Plan upgrade in next development phase, benchmark in dev environment first.

---

#### 4. Deprecation (Within 2 Quarters)

- **Current version nearing EOL** (<12 months remaining)
- **Breaking changes accumulating** (>2 minor versions behind)
- **Ecosystem incompatibility** (tools/libraries require newer version)

**Action:** Plan upgrade within 2 development phases to avoid emergency upgrade.

---

### Why Latest LTS/Stable?

**Benefits:**
- ✅ **Security:** Timely patches for known vulnerabilities
- ✅ **Performance:** Benefit from optimizations and improvements
- ✅ **Support:** Vendor support guaranteed for 3-5 years minimum
- ✅ **Compatibility:** Avoid breaking changes from multi-version jumps
- ✅ **Community:** Broader testing, faster issue resolution

**Why NOT bleeding edge?**
- ❌ **Stability:** Nightly/beta versions have unpredictable breaking changes
- ❌ **Testing:** Latest stable has broader community validation
- ❌ **Support:** Pre-release versions lack vendor support commitments

---

### Why Quarterly Reviews?

**Balance:**
- **Not too frequent:** Avoids upgrade fatigue and disruption
- **Not too infrequent:** Prevents dangerous version lag accumulation
- **Predictable:** Aligns with typical LTS release schedules (quarterly or semi-annual)

**Phase Alignment:**
- Quarterly reviews coincide with phase planning
- Upgrades can be integrated into new development phases
- No surprise emergency upgrades (except HIGH/CRITICAL CVEs)

---

## Rationale

### Problem: Version Lag Accumulation

**Example from Phase 2.5:**
- Keycloak 6 months behind → HIGH CVE discovered
- Vault 4 months behind → missed KV v2 performance improvements
- Postgres 3 months behind → delayed access to JSON performance for Phase 3

**Cost of delay:**
- Emergency upgrade mid-development (disruptive)
- Longer testing cycles (more breaking changes to validate)
- Higher risk (untested multi-version jumps)

### Solution: Proactive Quarterly Reviews

**Benefits:**
- **Predictable:** Scheduled upgrade planning, no surprises
- **Lower risk:** Smaller version jumps, incremental testing
- **Better timing:** Upgrades planned during phase transitions, not mid-development
- **Security:** Timely CVE patches within 1 quarter max (unless emergency)

---

## Consequences

### Positive

✅ **Reduced security risk:** Timely CVE patches (max 3 months lag, except emergencies)  
✅ **Improved performance:** Access to latest optimizations within 3-6 months  
✅ **Better support:** Stay within vendor LTS windows (3-5 year guarantees)  
✅ **Easier upgrades:** Smaller version jumps, fewer breaking changes  
✅ **Predictable overhead:** Known quarterly review effort (~1 day per quarter)

---

### Negative

❌ **Upgrade overhead:** ~1 day per quarter for review, testing, validation  
❌ **Potential disruption:** Breaking changes may require code updates (e.g., Rust 1.91 Clone derives)  
❌ **Testing burden:** Regression testing required for Phase 1.2 + Phase 2.2 after each upgrade  
❌ **Emergency work:** HIGH/CRITICAL CVEs may force out-of-band upgrades

---

### Mitigations

**Overhead Mitigation:**
- **Automation:** Scripted health checks, automated smoke tests
- **Documentation:** Standard upgrade playbooks, validation checklists
- **Batching:** Group compatible upgrades in single phase (e.g., Phase 2.5: Keycloak + Vault + Postgres)

**Risk Mitigation:**
- **Testing:** Always re-run smoke tests for affected phases (Phase 1.2, Phase 2.2)
- **Rollback:** Keep previous Docker image tags for quick rollback
- **Staging:** Test in dev environment before production
- **Documentation:** Update VERSION_PINS.md + CHANGELOG.md every upgrade

**Disruption Mitigation:**
- **Timing:** Plan upgrades during phase transitions (not mid-sprint)
- **Feature flags:** Graceful degradation when dependencies unavailable
- **Backward compatibility:** Maintain API contracts across upgrades

---

## Alternatives Considered

### Alternative 1: Pin to Specific Versions (Never Upgrade)

**Approach:** Lock dependencies at known-good versions, never upgrade unless forced.

**Rejected because:**
- ❌ Accumulates security debt (CVEs pile up)
- ❌ Increases EOL risk (support drops unexpectedly)
- ❌ Creates emergency upgrade scenarios (forced migrations under pressure)
- ❌ Blocks performance improvements (no access to optimizations)

**Example:** Keycloak 24.0.4 → CVE-2024-8883 (HIGH) discovered → forced emergency upgrade.

---

### Alternative 2: Upgrade Only on Breaking Issues

**Approach:** Upgrade when current version blocks new features or has critical bugs.

**Rejected because:**
- ❌ **Reactive** (not proactive, driven by emergencies)
- ❌ **Harder multi-version jumps** (more breaking changes accumulate)
- ❌ **Security lag** (CVEs may not block features but are still critical)
- ❌ **Unpredictable timing** (disrupts development schedules)

**Example:** Waiting until Postgres 16 EOL approaches → forced migration during Phase 3 development.

---

### Alternative 3: Always Use Nightly/Beta

**Approach:** Track bleeding-edge versions for earliest access to features.

**Rejected because:**
- ❌ **Too unstable** for production system (unpredictable breaking changes)
- ❌ **No vendor support** (pre-release versions lack support commitments)
- ❌ **High test burden** (constant regression testing needed)
- ❌ **Ecosystem incompatibility** (community tools lag behind bleeding edge)

**Example:** Rust nightly → breaking changes daily, incompatible with stable ecosystem.

---

### Alternative 4: Continuous Dependency Monitoring

**Approach:** Automated dependency scanning, weekly updates via Dependabot/Renovate.

**Partially Adopted:**
- ✅ Use for **development tools** (Rust, Python) with automated testing
- ❌ **Not suitable for infrastructure** (runtime changes need manual validation)
- ❌ **Too frequent** (weekly updates create upgrade fatigue)

**Mitigation:**
- Use quarterly manual review for infrastructure (Keycloak, Vault, Postgres)
- Consider automated scanning for dev tools in future (post-MVP)

---

## Implementation

### Phase 2.5 (Current)

**Executed:** 2025-11-04

✅ Keycloak 24.0.4 → 26.0.4 (security CVE fixes)  
✅ Vault 1.17.6 → 1.18.3 (latest LTS)  
✅ Postgres 16.4 → 17.2 (latest stable, 5-year LTS)  
✅ Ollama 0.12.9 (verified latest stable)  
✅ Python 3.12.3 → 3.13.9 (dev tool)  
⚠️ Rust 1.83.0 → 1.91.0 (tested, deferred due to code changes needed)

**Validation:**
- ✅ Phase 1.2 smoke tests (OIDC/JWT functional)
- ✅ Phase 2.2 smoke tests (Privacy Guard with Vault 1.18.3 + Postgres 17.2 + Ollama 0.12.9)
- ✅ Created VERSION_PINS.md with LTS/stable targets
- ✅ Created ADR-0023 (this document)

---

### Ongoing Process

**Quarterly Dependency Review** (starting Q1 2026):

**Schedule:**
- **Q1 2026 (March):** First scheduled review
- **Q2 2026 (June):** Second review
- **Q3 2026 (September):** Third review
- **Q4 2026 (December):** Fourth review

**Deliverables per review:**
1. Updated VERSION_PINS.md (review date + findings)
2. CHANGELOG.md entry (if upgrades performed)
3. ADR addendum (if major version changes, e.g., Postgres 17 → 18)
4. Phase-X-Validation-Summary.md (if upgrades integrated into phase)

**Effort estimate:** ~1 day per quarter (~4 days per year)

---

### Upgrade Checklist (Template)

Use this checklist for each dependency upgrade:

```markdown
## Dependency Upgrade Checklist

**Component:** [e.g., Keycloak]  
**Version:** [e.g., 24.0.4 → 26.0.4]  
**Priority:** [HIGH/MEDIUM/LOW]  
**Trigger:** [Security CVE / LTS transition / Performance / Deprecation]

### Pre-Upgrade
- [ ] Review release notes for breaking changes
- [ ] Check VERSION_PINS.md for current version
- [ ] Identify affected phases (Phase 1.2, Phase 2.2, etc.)
- [ ] Pull new Docker image (`docker pull <image>:<tag>`)

### Upgrade
- [ ] Update VERSION_PINS.md
- [ ] Update ce.dev.yml (or relevant compose file)
- [ ] Restart services (`docker compose down && docker compose up -d`)
- [ ] Verify health checks (all services healthy)

### Validation
- [ ] Re-run affected phase smoke tests
  - [ ] Phase 1.2 smoke tests (if auth/OIDC component)
  - [ ] Phase 2.2 smoke tests (if Vault/Postgres/Ollama component)
- [ ] Performance regression check (P50 within 10% of baseline)
- [ ] Breaking changes documented (if any)

### Documentation
- [ ] Update CHANGELOG.md (version changes, CVE fixes, performance notes)
- [ ] Create validation report (Phase-X-Validation-Summary.md)
- [ ] Update ADR if major version change (e.g., ADR-0023 addendum)
- [ ] Git commit with conventional commit message

### Rollback Plan (if needed)
- [ ] Previous Docker image tag noted: [e.g., keycloak:24.0.4]
- [ ] Rollback command ready: `git checkout VERSION_PINS.md && docker compose up -d`
```

---

## Examples

### Example 1: Quarterly Review (Q1 2026)

**Date:** 2026-03-28  
**Findings:**
- Keycloak: 26.0.4 → 27.0.1 (2 minor versions, no HIGH CVEs)
- Vault: 1.18.3 → 1.19.0 (LTS transition)
- Postgres: 17.2 → 17.4 (patch updates)
- Ollama: 0.12.9 → 0.13.2 (new model support)
- Rust: 1.83.0 → 1.95.0 (12 versions behind)

**Decisions:**
- **Keycloak:** Upgrade to 27.0.1 in Q1 phase (MEDIUM priority)
- **Vault:** Upgrade to 1.19.0 LTS in Q1 phase (MEDIUM priority, LTS transition)
- **Postgres:** Upgrade to 17.4 in Q1 phase (LOW priority, patch release)
- **Ollama:** Upgrade to 0.13.2 in Q1 phase (MEDIUM priority, model ecosystem)
- **Rust:** Defer to Q2 (code changes needed for 1.91+, track breaking changes)

**Action:** Plan "Phase X.Y: Q1 2026 Dependency Refresh" (~1 day work)

---

### Example 2: Emergency Upgrade (CVE-2026-XXXXX)

**Date:** 2026-05-15 (out-of-band, mid-Q2)  
**Trigger:** HIGH severity CVE in Vault 1.19.0  
**Impact:** Vault KV v2 authentication bypass (exploitable)

**Actions:**
1. **Immediate (Day 1):**
   - Check Vault 1.19.3 release notes (CVE patched)
   - Pull new image: `docker pull hashicorp/vault:1.19.3`
   - Update VERSION_PINS.md
   - Update ce.dev.yml
   - Restart services

2. **Validation (Day 2):**
   - Re-run Phase 2.2 smoke tests (Vault integration)
   - Verify pseudo_salt accessible
   - Verify deterministic pseudonymization working
   - Performance check (no regression)

3. **Documentation (Day 2-3):**
   - Update CHANGELOG.md (emergency CVE patch)
   - Create validation summary
   - Merge to main immediately (skip PR for security urgency)

**Total effort:** 2-3 days (acceptable for HIGH CVE response)

---

## Success Metrics

Track these metrics quarterly to measure policy effectiveness:

| Metric | Target | Rationale |
|--------|--------|-----------|
| **CVE Exposure Time** | <90 days (max) | Max 1 quarter lag for non-emergency CVEs |
| **Emergency Upgrades** | <2 per year | Indicates proactive quarterly reviews working |
| **Breaking Changes** | <10% of upgrades | Latest LTS/stable should minimize breaking changes |
| **Upgrade Effort** | ~1 day per quarter | Sustainable overhead, not disruptive |
| **Rollback Rate** | <5% | Indicates good testing/validation process |

**Review Annually:** Assess metrics in Q4 review, adjust policy if needed.

---

## References

### External References

- **Keycloak Release Notes:** https://www.keycloak.org/docs/latest/release_notes/
- **Vault LTS Tracker:** https://developer.hashicorp.com/vault/docs/updates/lts-tracker
- **Postgres Support Policy:** https://www.postgresql.org/support/versioning/
- **Rust Release Schedule:** https://releases.rs/
- **Python Release Schedule:** https://devguide.python.org/versions/
- **End of Life Date Tracker:** https://endoflife.date/

### Internal References

- **Phase 2.5 Execution Plan:** `Technical Project Plan/PM Phases/Phase-2.5/Phase-2.5-Execution-Plan.md`
- **VERSION_PINS.md:** Project dependency inventory
- **CHANGELOG.md:** Upgrade history
- **Phase Validation Reports:**
  - `Phase-2.5-Keycloak-Validation.md`
  - `Phase-2.5-Privacy-Guard-Validation.md`

### CVE References

- **CVE-2024-8883 (Keycloak):** NVD database (HIGH severity, session fixation)
- **CVE-2024-7318 (Keycloak):** NVD database (MEDIUM severity, authorization bypass)
- **CVE-2024-8698 (Keycloak):** NVD database (MEDIUM severity, XSS)

---

## Revision History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | 2025-11-04 | Initial ADR | Goose Orchestrator Agent |

---

**Approved by:** Engineering Team  
**Implementation:** Phase 2.5 (2025-11-04)  
**Next Review:** Q1 2026 (March 2026)
