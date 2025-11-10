# Org-Chart Orchestrated AI Framework — Technical Project Plan

**Version:** 3.0 (Cleaned & Restructured)  
**Date:** 2025-11-10  
**Previous:** v2.0 (2025-11-05)  
**Alignment:** Goose v1.12.00, Block Goose Innovation Grant ($100K/12mo)

---

## Objectives

Deliver a **privacy-first, org-aware orchestration MVP** that coordinates role-based AI agents using:
- **HTTP-only flows** + OIDC SSO
- **Cross-agent task routing** and approvals
- **Deterministic PII protection** (Privacy Guard Proxy)
- **Auditable execution** (all operations logged)

---

## Success Criteria (MVP - After Phase 7)

- ✅ E2E demo: Multi-agent approval workflow (Finance ↔ Manager ↔ Legal)
- ✅ Privacy Guard Proxy: All LLM calls intercepted, PII masked/unmasked, latency ≤500ms P50
- ✅ Agent Mesh: Cross-agent communication working
- ✅ Controller APIs: OpenAPI spec published
- ✅ Deployable: CE defaults (Keycloak, Vault, Postgres, Ollama, Docker)
- ✅ Performance: P50 ≤ 5s for interactive tasks, 99.5% availability

---

## Scope

### MVP (v1.0 - After Phase 7)
- **Components:** Identity/Auth, Agent Mesh MCP, Directory/Policy, Controller API, Privacy Guard Proxy, Audit/Observability, Storage/Metadata
- **Constraints:** HTTP-only, single-tenant, minimal server-side metadata, desktop-first agents

### Post-MVP (v1.1+)
- Policy composition, message bus adapter, SCIM, mTLS, dashboards, multi-tenant, advanced approvals, analytics

---

## Architecture

### Core Components

```
┌──────────────┐
│ Goose Agent  │ (Finance, Manager, Legal, HR, Developer, Support)
└──────┬───────┘
       │
       ▼
┌────────────────────┐
│ Privacy Guard      │ ← Intercepts ALL LLM calls
│ Proxy (Port 8090)  │   Mask PII → LLM → Unmask PII
└──────┬─────────────┘
       │
       ├─────────────────────┐
       │                     │
       ▼                     ▼
┌──────────────┐      ┌───────────────┐
│Privacy Guard │      │ LLM Provider  │
│Service (8089)│      │ (OpenRouter)  │
└──────────────┘      └───────────────┘
       │
       ▼
┌────────────────────┐
│ Controller API     │ ← Orchestrates agents, routes tasks
│ (Port 8088)        │
└──────┬─────────────┘
       │
       ├─────────────────────┬─────────────────────┬─────────────────┐
       ▼                     ▼                     ▼                 ▼
┌──────────┐         ┌──────────┐         ┌──────────┐      ┌──────────┐
│ Postgres │         │ Keycloak │         │  Vault   │      │  Redis   │
│ (5432)   │         │ (8080)   │         │ (8200/1) │      │ (6379)   │
└──────────┘         └──────────┘         └──────────┘      └──────────┘
```

### Alignment to Goose v1.12
- **MCP-first:** Extension allowlists, tool governance
- **Lead/worker:** Multi-provider orchestration
- **Axum server:** OpenAPI, OTLP observability
- **Local-first:** State management, OS keychain

### Additions
- **Privacy Guard Proxy:** HTTP proxy for LLM call interception
- **Agent Mesh:** Multi-agent coordination (send_task, request_approval, notify, fetch_status)
- **Lifecycle Module:** Session FSM (INIT → ACTIVE → PAUSED → COMPLETED)
- **Profile System:** Role-based configuration (YAML, Vault-signed)

---

## Phase Summary

### Phases 0-5: COMPLETE ✅ (7 weeks)

**Phase 0:** Project Setup (1 day) ✅  
**Phase 1:** Identity & Security (2 days) ✅  
**Phase 1.2:** Controller JWT Verification (1 day) ✅  
**Phase 2:** Privacy Guard (3 days) ✅  
**Phase 2.2:** Privacy Guard Enhancement (2 days) ✅  
**Phase 2.5:** Dependency Upgrades (1 day) ✅  
**Phase 3:** Controller API + Agent Mesh (2 days) ✅  
**Phase 4:** Storage/Metadata + Session Persistence (1 week) ✅  
**Phase 5:** Profile System + Privacy Guard MCP (2 weeks) ✅

**Tagged Releases:** v0.3.0 (Phase 3), v0.4.0 (Phase 4), v0.5.0 (Phase 5)

---

### Phase 6: Backend Integration & Multi-Agent Testing (4-6 weeks) **IN PROGRESS**

**Restructured:** 2025-11-10 (Integration-first approach)  
**Status:** Not Started  
**Target:** MVP-ready backend integration (v0.6.0)

#### 5 Workstreams:

**A. Lifecycle Integration** (Week 1-2)
- Wire Lifecycle module into Controller routes
- Enable session FSM (INIT → ACTIVE → PAUSED → COMPLETED)
- Session persistence to database
- Tests: 8

**B. Privacy Guard Proxy** (Week 2-3)
- Build HTTP proxy service (Rust/Axum, port 8090)
- Intercept ALL LLM calls (Goose → Proxy → Mask → LLM → Unmask → Goose)
- Support OpenRouter, Anthropic, OpenAI providers
- Update all 8 profile YAMLs (api_base points to proxy)
- Tests: 8

**C. Multi-Goose Test Environment** (Week 3-4)
- Docker Goose image (Dockerfile, config scripts)
- 3 Goose containers (Finance, Manager, Legal) in ce.dev.yml
- Agent Mesh configuration for multi-Goose
- Agent registration/discovery in Controller
- Tests: 8

**D. Agent Mesh E2E Testing** (Week 4-5)
- E2E test framework (Python)
- 3 scenarios: Expense Approval, Legal Review, Cross-Department
- Privacy isolation validation (19 steps total)
- Tests: 19

**V. Full Integration Validation** (Week 5-6)
- Full workflow test (30 tests: admin setup, user onboarding, privacy proxy, agent mesh, session lifecycle, data validation)
- Performance testing (load test, benchmarks)
- Security audit (18 checks)
- Tests: 48

**Total Tests:** 81+

**Deliverables:**
- 7 code deliverables (routes, services, configs, migrations)
- 7 test suites (81+ tests)
- 7 documentation updates

**Acceptance Criteria:**
- All 81+ tests passing
- 3 Goose agents collaborating (Finance ↔ Manager ↔ Legal)
- Privacy Guard Proxy intercepting ALL LLM calls
- Demo workflow operational (CSV → Profile → Multi-agent)
- Ready for Phase 7 (UI development)

**Phase 6 Details:** See `Technical Project Plan/PM Phases/Phase-6/README.md`

---

### Phase 7: Admin UI + User Experience (2-3 weeks) **DEFERRED**

**UI Work Deferred from Phase 6:**

**Admin Dashboard:**
- CSV upload UI (org chart import)
- User management (assign/revoke profiles)
- Audit log viewer (filters, export)
- System health monitoring (service status)

**User Portal:**
- Self-service profile view
- Session history
- Privacy preferences
- Agent Mesh collaboration panel

**Goose Desktop Integration:**
- Auto-sign-in with Keycloak
- Profile auto-sync
- Multi-agent collaboration UI

**Deliverables:**
- 5 admin pages (Dashboard, Profiles, Org Chart, Audit, Settings)
- 3 user pages (Profile, Chat, Sessions)
- D3.js org chart visualization
- Monaco YAML editor
- JWT auth integration

**Target:** MVP complete (v1.0.0)

---

### Phase 8+: Deployment, Hardening, Optimization **DEFERRED**

**Production Deployment:**
- Kubernetes deployment (Helm charts, auto-scaling)
- Cloud infrastructure (AWS/GCP/Azure)
- Multi-environment support (dev, staging, prod)

**Security Hardening:**
- Secrets rotation (Keycloak, Vault credentials)
- Pentesting and vulnerability assessment
- Compliance packs (GDPR, SOC2, HIPAA)

**Performance Optimization:**
- Caching strategies (Redis, CDN)
- Query optimization (database indices)
- Load balancing (multiple Controller instances)

**Ollama NER Improvement:**
- Fine-tune model with corporate PII dataset
- Improve precision/recall (>95% target)
- Reduce latency (P50 < 200ms target)

**Monitoring & Alerting:**
- Prometheus metrics
- Grafana dashboards
- PagerDuty integration

**Backup & Disaster Recovery:**
- Automated backups (Postgres, Vault, Keycloak)
- Point-in-time recovery
- Disaster recovery runbooks

**Multi-Tenant Support:**
- Tenant isolation
- Per-tenant quotas
- Billing integration

---

## Timeline (12-Month Grant)

### Q1 (Months 1-3): Foundation ✅ COMPLETE
- **Week 1-2:** Phases 0-3 (Identity, Privacy Guard, Controller API, Agent Mesh)
- **Week 3-4:** Phase 4 (Storage, Session Persistence)
- **Week 5-6:** Phase 5 (Profiles, Privacy Guard MCP)
- **Week 7:** Phase 5.5 (Grant Application Demo)
- **Milestone:** Grant application ready (v0.5.0)

### Q2 (Months 4-6): Backend Integration & Testing 🔄 IN PROGRESS
- **Week 8-13:** Phase 6 (Lifecycle, Privacy Proxy, Multi-Goose, Agent Mesh E2E, Full Validation)
- **Week 14-16:** Phase 7 (Admin UI, User Portal, Goose Desktop Integration)
- **Milestone:** MVP complete (v1.0.0)

### Q3 (Months 7-9): Scale & Features
- Phase 8: Model Orchestration (lead/worker, cost-aware routing)
- Phase 9: 10 Role Profiles library
- Phase 10: Kubernetes deployment
- **Milestone:** Enterprise-ready platform (v1.5.0)

### Q4 (Months 10-12): Production & Community
- Phase 11: Advanced features (approvals, SCIM, compliance)
- Phase 12: Community engagement (blog posts, talks, contributors)
- **Milestone:** Sustainable OSS project (v2.0.0)

---

## Infrastructure

### CE Defaults (Docker Compose)
- **Keycloak:** OIDC SSO, JWT minting
- **Vault:** Secrets management, profile signing
- **Postgres:** Data persistence (sessions, profiles, audit)
- **Ollama:** Local LLM for NER (qwen3:0.6b)
- **Redis:** Caching, idempotency
- **Controller:** Axum API (Rust)
- **Privacy Guard:** PII detection service (Rust)
- **Privacy Guard Proxy:** LLM call interceptor (Rust) *NEW in Phase 6*

### Topologies
- **Desktop-only:** Goose Desktop + local Privacy Guard + Agent Mesh MCP
- **Org/Dept:** Docker Compose (controller, privacy guard, postgres) + desktop agents
- **Production:** Kubernetes (Helm charts, auto-scaling) *Phase 8+*

### Security Zones
- **Desktop:** Agent + Privacy Guard (local)
- **Org:** Controller + Policy + Audit (centralized)
- **Secrets:** Vault + KMS (isolated)

---

## Governance & Security

- **Identity:** OIDC SSO, short-lived JWT (≤30m)
- **Authorization:** RBAC/ABAC (extension allowlists, tool policies)
- **Privacy:** Mask-and-forward by default, deterministic pseudonymization
- **Audit:** Structured logs, OTEL traces, exportable audit events
- **Compliance:** SOC2-aligned practices

---

## Risks & Mitigations

1. **Privacy Guard accuracy** → Conservative rules + manual overrides
2. **Auth complexity** → OIDC/JWT bridge, standardized middleware
3. **Latency** → Local guard optimization, caching, async processing
4. **Security** → JWT-bound requests, signed profiles, mTLS later
5. **Integration sprawl** → Curated allowlists, staged rollout
6. **Data custody** → Stateless content rule, redaction at source
7. **Cost** → Budget policies, token accounting, downshift
8. **Scale** → Document multi-tenant path post-MVP

---

## Key Decisions

### Phase 6 Restructure (2025-11-10)
- **Integration-first:** Backend complete BEFORE UI work
- **Agent Mesh E2E:** Core value, not optional (Finance ↔ Manager ↔ Legal)
- **Privacy Guard Proxy:** ALL LLM calls intercepted (non-negotiable)
- **Admin workflow:** Admin assigns profiles (users don't choose)
- **UI deferred:** Phase 7 (after backend proven stable)

---

## Current Status (2025-11-10)

### Completed Phases (0-5)
- ✅ Identity/Auth (Keycloak OIDC, JWT verification)
- ✅ Privacy Guard (regex + NER, Vault pseudonymization)
- ✅ Controller API (5 routes, 21 unit tests)
- ✅ Agent Mesh MCP (4 tools, 5/6 integration tests)
- ✅ Session Persistence (Postgres, Redis idempotency)
- ✅ Profile System (8 profiles, Vault-signed, database seeding)

### Current Phase (6)
- **Status:** Ready to start
- **Workstreams:** 5 (A: Lifecycle, B: Privacy Proxy, C: Multi-Goose, D: Agent Mesh E2E, V: Validation)
- **Tasks:** 21 total (0 complete)
- **Tests:** 81+ planned (0 created)

### Infrastructure Status
- **Services:** 7 running (controller, privacy-guard, redis, ollama, keycloak, postgres, vault)
- **Database:** 8 tables, 5 migrations applied
- **Profiles:** 8 loaded (analyst, developer, finance, hr, legal, manager, marketing, support)
- **Volumes:** 6 persistent (postgres_data, keycloak_data, vault_raft, vault_logs, ollama_models, redis_data)
- **Tests Passing:** 28/28 (Finance PII: 8/8, Comprehensive Profiles: 20/20)

---

## Documentation

### Essential Docs
- **This File:** High-level technical plan (summary only)
- **System Architecture:** `docs/operations/SYSTEM-ARCHITECTURE-MAP.md`
- **Startup Guide:** `docs/operations/STARTUP-GUIDE.md`
- **Testing Guide:** `docs/operations/TESTING-GUIDE.md`
- **System Reference:** `docs/operations/COMPLETE-SYSTEM-REFERENCE.md`

### Phase-Specific Docs
- **Phase 0-3:** `Technical Project Plan/PM Phases/Phase-0/` through `Phase-3/`
- **Phase 4:** `Technical Project Plan/PM Phases/Phase-4/`
- **Phase 5:** `Technical Project Plan/PM Phases/Phase-5/`
- **Phase 6:** `Technical Project Plan/PM Phases/Phase-6/` ← Current phase
  - See `Phase-6/README.md` for detailed workstream breakdown
  - See `Phase-6/PHASE-6-MAIN-PROMPT.md` for copy-paste prompt
  - See `Phase-6/Phase-6-Checklist.md` for task tracking

### Product Docs
- **Product Description:** `docs/product/productdescription.md`
- **Architecture:** `docs/architecture/PHASE5-ARCHITECTURE.md`
- **Privacy Guide:** `docs/privacy/PRIVACY-GUARD-NER.md`

---

## Milestones (Grant-Aligned)

### Q1 (Months 1-3): Foundation ✅ COMPLETE
- **M1 (Week 2):** OIDC + Privacy Guard + Vault ✅
- **M2 (Week 4):** Controller API + Agent Mesh ✅
- **M3 (Week 7):** Grant application ready (v0.5.0) ✅

### Q2 (Months 4-6): Backend Integration 🔄 IN PROGRESS
- **M4 (Week 13):** Backend integration complete (Phase 6)
- **M5 (Week 16):** MVP with UI complete (Phase 7)
- **Target:** v1.0.0 (MVP ready for demo)

### Q3 (Months 7-9): Scale & Features
- **M6:** Model Orchestration (lead/worker, cost routing)
- **M7:** 10 Role Profiles library
- **M8:** Kubernetes deployment
- **Target:** v1.5.0 (Enterprise-ready)

### Q4 (Months 10-12): Production & Community
- **M9:** Advanced features (approvals, SCIM, compliance)
- **M10:** Community engagement (talks, blog posts)
- **M11:** Business validation (2 paid pilots)
- **M12:** Sustainable OSS project (v2.0.0)

---

## Upstream Contributions (Planned)

**Target:** 5 PRs to Goose core by Month 12

1. **Privacy Guard MCP** (Month 6) - PII masking middleware
2. **OIDC/JWT Middleware** (Month 4) - Keycloak SSO integration
3. **Agent Mesh Protocol** (Month 7) - Multi-agent coordination
4. **Session Persistence** (Month 4) - Postgres-backed sessions
5. **Role Profiles Spec** (Month 8) - YAML/JSON profile format

---

## Next Steps

**Current Phase:** Phase 6 (Backend Integration & Multi-Agent Testing)

**For New Agents:**
1. Read `Technical Project Plan/PM Phases/Phase-6/PHASE-6-MAIN-PROMPT.md`
2. Read `Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`
3. Read `Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md`
4. Read `docs/tests/phase6-progress.md`
5. Verify system state (docker ps, services healthy)
6. Ask user which workstream to focus on (A, B, C, D, or V)

**Recommended Start:** Workstream A (Lifecycle Integration) - no dependencies

---

## Version History

- **v3.0 (2025-11-10):** Cleaned up excessive detail, Phase 6 restructured
- **v2.0 (2025-11-05):** Grant-aligned, Phase 5 details added
- **v1.0 (2025-10-27):** Initial technical plan

---

**This document is a HIGH-LEVEL SUMMARY only.**  
**For detailed execution plans, see phase-specific directories.**  
**For operational procedures, see docs/operations/**

**Last Updated:** 2025-11-10  
**Next Review:** After Phase 6 completion
