# Documentation Index - Org Chart Goose Orchestrator

**Project**: org-chart-goose-orchestrator  
**Version**: 0.5.0 (Phase 5)  
**Last Updated**: 2025-11-07

---

## 🚀 Quick Start

**New to the project?** Start here:
1. **Product Description**: [`docs/product/productdescription.md`](product/productdescription.md)
2. **Architecture Overview**: [`docs/HOW-IT-ALL-FITS-TOGETHER.md`](HOW-IT-ALL-FITS-TOGETHER.md)
3. **Quick Start Guide**: [`docs/QUICK-START-TESTING.md`](QUICK-START-TESTING.md)
4. **Build Instructions**: [`docs/BUILD_QUICK_START.md`](BUILD_QUICK_START.md)

---

## 📋 Core Documentation

### System Architecture
- **Phase 5 Architecture Diagrams**: [`architecture/PHASE5-ARCHITECTURE.md`](architecture/PHASE5-ARCHITECTURE.md) ⭐ **NEW** - Complete system (4 views)
- **Overall Architecture**: [`HOW-IT-ALL-FITS-TOGETHER.md`](HOW-IT-ALL-FITS-TOGETHER.md) (28KB - comprehensive)
- **Service Versions**: [`../VERSION_PINS.md`](../VERSION_PINS.md) (Vault 1.18.3, Postgres 17.2, etc.)
- **Build Process**: [`BUILD_PROCESS.md`](BUILD_PROCESS.md)
- **Docker Compose Guide**: [`guides/compose-ce.md`](guides/compose-ce.md)

### Profile System ⭐ NEW (Phase 5)
- **Profile Specification**: [`profiles/SPEC.md`](profiles/SPEC.md) - Complete schema + deserialization guide
- **Example Profiles**: [`../profiles/`](../profiles/) - finance.yaml, legal.yaml, developer.yaml, hr.yaml, executive.yaml, support.yaml

### Vault Integration ⭐ NEW (Phase 5)
- **Vault Operations Guide**: [`guides/VAULT.md`](guides/VAULT.md) - Dev mode + Phase 6 production upgrade
- **ADR**: [`adr/0016-ce-profile-signing-key-management.md`](adr/0016-ce-profile-signing-key-management.md)

### Migration Guides ⭐ NEW (Phase 5)
- **Phase 4 → Phase 5**: [`MIGRATION-PHASE5.md`](MIGRATION-PHASE5.md) - Upgrade guide (v0.4.0 → v0.5.0)

### Privacy Guard
- **Integration Guide**: [`guides/privacy-guard-integration.md`](guides/privacy-guard-integration.md)
- **Configuration**: [`guides/privacy-guard-config.md`](guides/privacy-guard-config.md)
- **User Override UI**: [`privacy/USER-OVERRIDE-UI.md`](privacy/USER-OVERRIDE-UI.md)

### API Reference
- **Controller Endpoints**: [`api/controller/README.md`](api/controller/README.md)
- **Audit API**: [`api/audit/README.md`](api/audit/README.md)
- **Schemas**: [`api/schemas/`](api/schemas/)

---

## 🏗️ Architectural Decision Records (ADRs)

**Directory**: [`adr/`](adr/)

### Identity & Security
- [0003](adr/0003-secrets-and-key-management.md) - Secrets and key management
- [0004](adr/0004-identity-and-auth.md) - Identity and auth
- [0006](adr/0006-identity-auth-bridge.md) - Identity auth bridge
- [0016](adr/0016-ce-profile-signing-key-management.md) - Profile signing (Vault)
- [0019](adr/0019-auth-bridge-jwt-verification.md) - JWT verification
- [0020](adr/0020-vault-oss-wiring.md) - Vault OSS wiring

### Privacy & Data Protection
- [0002](adr/0002-privacy-guard-placement.md) - Privacy Guard placement
- [0005](adr/0005-data-retention-and-redaction.md) - Data retention and redaction
- [0008](adr/0008-audit-schema-and-redaction.md) - Audit schema
- [0009](adr/0009-deterministic-pseudonymization-keys.md) - Pseudonymization
- [0015](adr/0015-guard-model-policy-and-selection.md) - Guard model policy
- [0021](adr/0021-privacy-guard-rust-implementation.md) - Privacy Guard Rust
- [0022](adr/0022-pii-detection-rules-and-fpe.md) - PII detection rules

### Configuration & Orchestration
- [0011](adr/0011-directory-policy-profile-bundles.md) - Directory policy bundles
- [0013](adr/0013-model-orchestration-lead-worker-cost-aware.md) - Lead-worker model
- [0014](adr/0014-ce-object-storage-default-and-provider-policy.md) - Object storage

### Infrastructure
- [0001](adr/0001-mvp-message-bus.md) - MVP message bus
- [0007](adr/0007-agent-mesh-mcp.md) - Agent mesh MCP
- [0010](adr/0010-controller-openapi-and-http-interfaces.md) - Controller OpenAPI
- [0012](adr/0012-storage-and-metadata-model.md) - Storage model
- [0017](adr/0017-controller-language-and-runtime-choice.md) - Controller language (Rust)
- [0018](adr/0018-controller-healthchecks-and-compose-profiles.md) - Health checks

**Total**: 23 ADRs (22 active + 1 template)

---

## 📖 User Guides

**Directory**: [`guides/`](guides/)

### Setup & Configuration
- **Dev Setup**: [`guides/dev-setup.md`](guides/dev-setup.md)
- **Compose CE**: [`guides/compose-ce.md`](guides/compose-ce.md)
- **Keycloak Guide**: [`guides/keycloak.md`](guides/keycloak.md)
- **Vault Operations**: [`guides/VAULT.md`](guides/VAULT.md) ⭐ NEW

### Privacy & Security
- **Privacy Guard Integration**: [`guides/privacy-guard-integration.md`](guides/privacy-guard-integration.md)
- **Privacy Guard Config**: [`guides/privacy-guard-config.md`](guides/privacy-guard-config.md)

---

## 🧪 Testing Documentation

**Directory**: [`tests/`](tests/)

### Phase 5 Test Results
- **Comprehensive Results**: [`tests/phase5-test-results.md`](tests/phase5-test-results.md) (21KB, 1,100 lines)
- **Progress Log**: [`tests/phase5-progress.md`](phase5-progress.md) (221KB, detailed history)

### Test Suites (Integration)
Located in: `../tests/integration/`
- **H2**: Profile Loading (10 tests) - `test_profile_loading.sh`
- **H3**: Privacy Guard (18 tests) - `test_finance_pii_jwt.sh`, `test_legal_local_jwt.sh`
- **H4**: Org Chart (12 tests) - `test_org_chart_jwt.sh`
- **H6**: E2E Workflow (10 tests) - `test_e2e_workflow.sh`
- **H6.1**: All Profiles (20 tests) - `test_all_profiles_comprehensive.sh`
- **H7**: Performance (7 tests) - `../tests/perf/api_latency_benchmark.sh`
- **Vault**: Admin Profiles (3 tests) - `test_admin_profiles.sh`

**Total**: 60/60 tests passing (100%)

### Historical Logs
- **Phase 0-4 Progress**: `tests/phase0-progress.md` through `tests/phase4-progress.md`
- **Archived Smoke Tests**: `archive/smoke-tests/` (Phase 0-4)

---

## 📊 Project Management

### Phase Artifacts
**Directory**: [`../Technical Project Plan/PM Phases/`](../Technical Project Plan/PM Phases/)

- **Phase 0**: Foundation (Repo structure, planning)
- **Phase 1**: Auth + Privacy Guard skeleton
- **Phase 1.2**: Privacy Guard HTTP API
- **Phase 2**: Controller foundation
- **Phase 2.2**: Keycloak integration
- **Phase 3**: Privacy Guard Rust migration
- **Phase 4**: E2E integration
- **Phase 5** (CURRENT): Profiles + Org Chart + Vault

### Master Plan
**File**: [`../Technical Project Plan/master-technical-project-plan.md`](../Technical Project Plan/master-technical-project-plan.md)

---

## 🔐 Security & Compliance

### Security Documentation
- **Security Concerns**: [`security/`](security/)
- **Compliance**: [`compliance/`](compliance/)
- **Audit Logs**: [`audit/`](audit/)

### Privacy
- **Privacy Policy**: [`privacy/`](privacy/)
- **PII Detection**: [`adr/0022-pii-detection-rules-and-fpe.md`](adr/0022-pii-detection-rules-and-fpe.md)
- **User Override UI**: [`privacy/USER-OVERRIDE-UI.md`](privacy/USER-OVERRIDE-UI.md)

---

## 🗄️ Database

**Directory**: [`database/`](database/)

- **Schema Documentation**: Database structure and migrations
- **Migrations**: `../src/controller/migrations/` (SQL files)

### Tables (Phase 5)
- **profiles**: Role-based configurations (JSONB)
- **org_users**: Organizational hierarchy
- **org_imports**: CSV import audit trail

---

## 📝 Grant Application

**Directory**: [`grant/`](grant/) or [`grants/`](grants/)

- Grant proposal documents
- Project pitch materials

---

## 🌐 Product Documentation

**Directory**: [`product/`](product/)

- **Product Description**: [`product/productdescription.md`](product/productdescription.md)
- **Requirements**: [`product/requirements.md`](product/requirements.md)
- **Open Interfaces**: [`product/open-interfaces.md`](product/open-interfaces.md)
- **CE Edition**: [`product/ce.md`](product/ce.md)
- **Approvals**: [`product/approvals.md`](product/approvals.md)
- **Privacy**: [`product/privacy.md`](product/privacy.md)

---

## 📦 Archived Documentation

**Directory**: [`archive/`](archive/)

### Session Summaries
- Session artifacts from development phases
- Historical planning documents

### Smoke Tests
- **Location**: [`archive/smoke-tests/`](archive/smoke-tests/)
- **Contents**: Phase 0-4 smoke test results (7 files)

### Obsolete
- **Location**: [`archive/obsolete/`](archive/obsolete/)
- **Contents**: Superseded documentation

### Workstream Summaries
- **workstream-d-test-summary.md**: D-series endpoint tests (superseded by phase5-test-results.md)
- **workstream-f-test-plan.md**: Unit test specifications
- **phase5-resume-report.md**: Session continuation artifact

---

## 🔍 Related Resources

### External Documentation
- **Upstream Goose Docs**: See [DOCS_INDEX.md](../DOCS_INDEX.md) (auto-generated, 2,188 lines)
- **Goose v1.12.00 References**: `../goose-versions-references/gooseV1.12.00/`
- **How Goose Works**: `../goose-versions-references/how-goose-works-docs/docs/`

### Development
- **.goosehints**: Project-level hints for AI agents
- **Justfile**: Common development tasks
- **scripts/**: Build and test automation

---

## 📂 Documentation Directory Structure

```
docs/
├── README.md                        # ← This file (quick navigation)
├── QUICK-START-TESTING.md           # Testing guide
├── BUILD_PROCESS.md                  # Build instructions
├── BUILD_QUICK_START.md              # Quick build guide
├── HOW-IT-ALL-FITS-TOGETHER.md       # Architecture overview (28KB)
├── UPSTREAM-CONTRIBUTION-STRATEGY.md  # Future upstream contributions
├── MIGRATION-PHASE5.md               # ⭐ Phase 4 → 5 upgrade guide
│
├── profiles/                         # ⭐ Profile system docs (Phase 5)
│   └── SPEC.md                       # Complete schema + deserialization guide
│
├── guides/                           # User guides
│   ├── compose-ce.md
│   ├── dev-setup.md
│   ├── keycloak.md
│   ├── privacy-guard-integration.md
│   ├── privacy-guard-config.md
│   └── VAULT.md                      # ⭐ Vault operations (Phase 5)
│
├── adr/                              # Architectural Decision Records (23 files)
│   ├── 0001-mvp-message-bus.md
│   ├── ...
│   └── 0022-pii-detection-rules-and-fpe.md
│
├── tests/                            # Test documentation
│   ├── phase5-test-results.md        # Comprehensive H results (1,100 lines)
│   ├── phase5-progress.md            # Detailed progress log (221KB)
│   ├── phase0-4-progress.md          # Historical progress
│   └── repo-audit-phase*.md
│
├── api/                              # API documentation
│   ├── controller/
│   ├── audit/
│   └── schemas/
│
├── archive/                          # Archived documentation
│   ├── smoke-tests/                  # Phase 0-4 smoke tests
│   ├── session-summaries/            # Session artifacts
│   ├── planning/                     # Planning docs
│   └── obsolete/                     # Superseded files
│
├── product/                          # Product documentation
├── privacy/                          # Privacy & PII protection
├── security/                         # Security documentation
├── compliance/                       # Compliance docs
├── database/                         # Database schemas
├── grant/ or grants/                 # Grant proposals
├── architecture/                     # Architecture diagrams
├── decisions/                        # Decision documents
├── conventions/                      # Coding conventions
└── [other directories...]

```

---

## 📚 Documentation by Topic

### For Developers
1. Profile Schema & Deserialization → [`profiles/SPEC.md`](profiles/SPEC.md)
2. Vault Integration → [`guides/VAULT.md`](guides/VAULT.md)
3. API Endpoints → [`api/controller/README.md`](api/controller/README.md)
4. Build Process → [`BUILD_PROCESS.md`](BUILD_PROCESS.md)
5. ADRs → [`adr/`](adr/)

### For Admins
1. Deployment Guide → [`guides/compose-ce.md`](guides/compose-ce.md)
2. Keycloak Setup → [`guides/keycloak.md`](guides/keycloak.md)
3. Vault Operations → [`guides/VAULT.md`](guides/VAULT.md)
4. Migration Guide → [`MIGRATION-PHASE5.md`](MIGRATION-PHASE5.md)

### For Grant Reviewers
1. Product Description → [`product/productdescription.md`](product/productdescription.md)
2. Architecture Overview → [`HOW-IT-ALL-FITS-TOGETHER.md`](HOW-IT-ALL-FITS-TOGETHER.md)
3. Test Results → [`tests/phase5-test-results.md`](tests/phase5-test-results.md)
4. Progress Log → [`tests/phase5-progress.md`](phase5-progress.md)
5. Profile Specification → [`profiles/SPEC.md`](profiles/SPEC.md)

### For AI Agents
1. Architecture Diagrams → [`architecture/PHASE5-ARCHITECTURE.md`](architecture/PHASE5-ARCHITECTURE.md) - 4 visual views
2. Profile Specification → [`profiles/SPEC.md`](profiles/SPEC.md)
3. Vault Guide → [`guides/VAULT.md`](guides/VAULT.md)
4. Migration Guide → [`MIGRATION-PHASE5.md`](MIGRATION-PHASE5.md)
5. Test Results → [`tests/phase5-test-results.md`](tests/phase5-test-results.md)

---

## 📈 Phase 5 Summary

### What's New
- **Profile System**: 6 roles (finance, legal, developer, hr, executive, support)
- **Org Chart Management**: CSV import, tree API, user lookup
- **Vault Integration**: HMAC-SHA256 profile signing (dev mode)
- **13 New Endpoints**: D1-D12 (profiles + org chart) + E5 (Privacy Guard)
- **60/60 Tests Passing**: 100% test coverage

### Key Files
- [`profiles/SPEC.md`](profiles/SPEC.md) - Profile system specification
- [`guides/VAULT.md`](guides/VAULT.md) - Vault operations guide
- [`MIGRATION-PHASE5.md`](MIGRATION-PHASE5.md) - Upgrade guide
- [`tests/phase5-test-results.md`](tests/phase5-test-results.md) - Test results
- [`../VERSION_PINS.md`](../VERSION_PINS.md) - Service versions

---

## 🔗 External Links

- **Upstream Goose Repo**: https://github.com/block/goose
- **Project Repo**: https://github.com/JEFH507/org-chart-goose-orchestrator
- **Goose Documentation**: https://block.github.io/goose/
- **Vault Documentation**: https://developer.hashicorp.com/vault

---

**For the complete auto-generated upstream file listing, see**: [`../DOCS_INDEX.md`](../DOCS_INDEX.md) (2,188 lines)

---

**End of Documentation Index**
