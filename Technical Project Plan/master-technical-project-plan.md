# Org-Chart Orchestrated AI Framework — Technical Project Plan
Version: v2.0 (Grant-Aligned)
Date: 2025-11-05
Previous: v1.0 (2025-10-27)
Alignment: Goose v1.12.00 architecture, Block Goose Innovation Grant ($100K/12mo)

## Objectives and Success Criteria
- Deliver a privacy-first, org-aware orchestration MVP that coordinates role-based “digital twin” agents using HTTP-only flows and OIDC SSO.
- Prove cross-agent task routing and approvals, deterministic pseudonymization, and auditable runs.
- Reuse Goose v1.12 capabilities (MCP-first, lead/worker models, OTLP-ready observability).

Success (MVP):
- E2E demo of multi-agent approval workflow with policy enforcement and audit trail.
- Privacy guard: mask-and-forward enforced, measurable accuracy/latency (≤500ms P50).
- Controller APIs published (minimal OpenAPI), Agent Mesh MCP verbs working.
- Deployable with CE defaults (Keycloak, Vault OSS, Postgres, Ollama; optional S3-compatible object storage — SeaweedFS default option; MinIO/Garage optional).
- Availability doc’d at 99.5% SLO; P50 ≤ 5s interactive agent tasks.

## Scope (MVP vs Post-MVP)
MVP (v1.2):
- Components: identity-auth-gateway, agent-mesh-mcp, directory-policy, controller-api, privacy-guard, audit-observability, model-orchestration, storage-metadata, packaging-deployment, security-secrets.
- Constraints: HTTP-only (no message bus), single-tenant per org, minimal metadata server-side, desktop-first agents.

Post-MVP (v1.3+):
- Policy composition/graph engine, message bus adapter, SCIM, mTLS, dashboards, multi-tenant hardening, advanced approvals, analytics.

## Architecture Overview and Alignment to Goose v1.12
Alignment:
- MCP-first tools and extension allowlists match governance needs.
- Lead/worker provider orchestration supports guard→planner/worker flows.
- Axum-based server (goosed) with OpenAPI and OTLP exporter re-used.
- Local-first state and OS keychain align with privacy-by-design.

Additions:
- Directory/Policy (signed role profiles, ABAC eval), HTTP Controller, Agent Mesh MCP verbs, Session Broker notions (scoped context) as minimal HTTP services.
- Identity via OIDC/JWT with bridge to goosed (X-Secret-Key compatibility).

Exclusions:
- No message bus in MVP; async handled via HTTP and polling.

## Work Breakdown Structure (WBS) and Timeline (Grant-Aligned: 12 months)

### Phases 0-3: COMPLETE ✅ (2 weeks actual vs 4 weeks estimated)
**Status:** Merged to main, tagged v0.3.0

Phase 0: Project Setup (S) ✅ COMPLETE
- Repo scaffolding, docker-compose CE defaults (Keycloak, Vault, Postgres, Ollama).
- Actual: 1 day

Phase 1: Identity & Security (M) ✅ COMPLETE
- OIDC login, JWT minting, Vault OSS wiring.
- Actual: 2 days

Phase 1.2: Controller JWT Verification (S) ✅ COMPLETE
- RS256 signature validation, JWKS caching, clock skew tolerance.
- Controller-side middleware integration.
- Actual: 1 day

Phase 2: Privacy Guard (M) ✅ COMPLETE
- Local runtime (regex + NER), deterministic pseudonymization (Vault keys), mask-and-forward.
- Actual: 3 days

Phase 2.2: Privacy Guard Enhancement (S) ✅ COMPLETE
- Local model integration (Ollama), preserve modes (Off/Detect/Mask/Strict).
- Actual: 2 days

Phase 2.5: Dependency Upgrades (S) ✅ COMPLETE
- Quarterly audit (Rust 1.83.0, Keycloak 26.0.4, Python 3.13.9, Vault 1.18.3).
- CVE remediation, compatibility validation.
- Actual: 1 day

Phase 3: Controller API + Agent Mesh (L) ✅ COMPLETE
- Controller API: 5 routes (tasks, sessions, approvals, profiles, audit), 21 unit tests, OpenAPI spec, JWT auth.
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status), 977 lines code, 5/6 integration tests.
- Cross-agent demo: Finance → Manager workflow, 5/5 test cases pass, 6/6 smoke tests.
- Actual: 2 days (estimated 9 days) — 78% faster

**Phases 0-3 Total:** 2 weeks (estimated 4 weeks)

---

### Phase 4: Storage/Metadata + Session Persistence (M) — GRANT Q1 MILESTONE
**Timeline:** Week 3-4 (1 week)  
**Target:** Grant application ready (v0.4.0)

Workstreams:
- A. Postgres Schema Design (~2 days)
  - Sessions table (id, role, task_id, status, created_at, updated_at, metadata)
  - Tasks table (id, type, from_role, to_role, payload, trace_id, idempotency_key)
  - Approvals table (id, task_id, approver_role, status, decision_at, notes)
  - Audit index (id, event_type, role, timestamp, trace_id, metadata)
  - Migrations (Diesel ORM for Rust, or SQL scripts)

- B. Session CRUD Operations (~2 days)
  - Controller routes: POST /sessions (create), GET /sessions/{id} (fetch), PUT /sessions/{id} (update)
  - Replace 501 responses with real data from Postgres
  - Session lifecycle: pending → active → completed/failed
  - Retention policies (7 days default, configurable)

- C. fetch_status Tool Completion (~1 day)
  - Update Agent Mesh fetch_status to call GET /sessions/{task_id}
  - Return real session status (not 501)
  - Integration tests updated (6/6 passing)

- D. Idempotency Deduplication (~1 day)
  - Redis cache for idempotency keys (TTL: 24 hours)
  - Controller middleware: check Redis before processing, return 200 if duplicate
  - Test duplicate request handling

- E. Progress Tracking (~15 min) 🚨 CHECKPOINT
  - Update Phase-4-Agent-State.json
  - Update phase4-progress.md
  - Commit to git, report to user

**Deliverables:**
- ✅ Postgres schema deployed (4 tables + migrations)
- ✅ Session persistence operational (POST/GET/PUT /sessions)
- ✅ fetch_status tool functional (no more 501 errors)
- ✅ Idempotency deduplication working (Redis-backed)
- ✅ Integration tests 6/6 passing
- ✅ Tagged release: v0.4.0

**Effort:** M (~1 week, 3-4 days)

---

### Phase 5: Directory/Policy + Profiles + Simple UI (M) — GRANT Q1 MILESTONE
**Timeline:** Week 5-6 (1 week)  
**Target:** Grant application ready (v0.5.0)

Workstreams:
- A. Profile Bundle Format (~1 day)
  - YAML/JSON spec: role, display_name, description, extensions, recipes, policies, env_vars
  - Signing mechanism (Vault transit key or HMAC)
  - Validation schema (JSON Schema or Rust types)

- B. Role Profiles (5 roles) (~2 days)
  - Finance: Budget approvals, reporting, compliance tools
  - Manager: Team oversight, approvals, delegation
  - Engineering: PR reviews, deployments, on-call
  - Marketing: Campaign management, analytics, content
  - Support: Ticket triage, escalation, knowledge base

- C. RBAC/ABAC Policy Engine (~2 days)
  - Policy evaluation: can_use_tool(role, tool_name), can_access_data(role, data_type)
  - Extension allowlists per role (Finance: no code execution, Manager: no PII access, etc.)
  - Deny-by-default, explicit allow rules
  - Policy storage (Postgres or Vault)

- D. GET /profiles/{role} Implementation (~1 day)
  - Replace mock profiles with real profile bundles from Postgres/Vault
  - Controller route returns profile JSON with extensions, recipes, policies
  - Agent Mesh tools use profiles for routing decisions

- E. Simple Web UI (~2 days) 🎨 NEW REQUIREMENT
  - Technology: Svelte/SvelteKit or React (lightweight, fast)
  
  **Admin UI Features:**
  - Settings Page:
    - Edit variables (SESSION_RETENTION_DAYS, IDEMPOTENCY_TTL)
    - Privacy Guard configuration:
      - Toggle modes: Off/Detect/Mask/Strict (user-selectable)
      - Model status: Show Ollama NER enabled/disabled (✅ ALREADY WORKING - Phase 2.2)
      - Detection preview: Test PII detection with sample text
    - Push policy updates to agents (real-time profile refresh via WebSocket or polling)
    - Assign profiles to users by email (directory integration)
    - View system health (Controller/Keycloak/Vault/Privacy Guard/Ollama status)
  - Profile Management:
    - Create/edit role profiles (YAML editor with validation)
    - Test policy evaluation (simulate can_use_tool, can_access_data)
    - Publish profiles (sign with Vault, push to agents)
  
  **User Features (Goose Client UI):**
  - Privacy Guard Settings:
    - Users select Privacy Guard mode in Goose client settings (Off/Detect/Mask/Strict)
    - Mode is stored per-user, sent with API requests
    - UI shows masked content warnings when Strict mode active
  - SSO Integration:
    - Users do SSO via Keycloak OIDC (redirect flow)
    - JWT token stored in OS keychain (secure)
    - Auto-refresh before expiry (seamless UX)
  - MCP Tools Auto-Configuration:
    - Users receive MCP tools via extension config (pushed from admin)
    - Extension manifest updated when admin assigns new profile
    - Goose client reloads extensions automatically
  
  **Pages:**
    1. Dashboard: Org chart visualization (D3.js or vis.js), agent status indicators
    2. Sessions: List recent sessions, click to view details, status badges
    3. Profiles: Browse available roles, view profile details, download YAML
    4. Audit: Search/filter audit events, trace ID linking, export CSV
    5. Settings (Admin): Edit variables, policy updates, user-profile assignment
  - Deployment: Static build served by Controller (Axum + tower-http serve-dir)
  - Auth: JWT token from Keycloak (same as API)
  - Styling: Tailwind CSS or simple.css (minimal, accessible)

- F. Progress Tracking (~15 min) 🚨 CHECKPOINT
  - Update Phase-5-Agent-State.json
  - Update phase5-progress.md
  - Commit to git, report to user

**Deliverables:**
- ✅ Profile bundle spec documented (docs/profiles/SPEC.md)
- ✅ 5 role profiles operational (Finance, Manager, Engineering, Marketing, Support)
- ✅ RBAC/ABAC policy engine functional
- ✅ GET /profiles/{role} returns real profiles
- ✅ Simple web UI deployed (4 pages: Dashboard, Sessions, Profiles, Audit)
- ✅ Tagged release: v0.5.0
- ✅ Grant application ready

**Effort:** M (~1 week, 5-6 days with UI)

---

### Phase 5.5: Grant Application Demo (~1 week) — GRANT Q1 FINAL DELIVERABLE
**Timeline:** Week 7  
**Target:** Grant application submitted

Tasks:
- A. Demo Video (~2 days)
  - 5-minute walkthrough: Finance → Manager → Engineering cross-agent workflow
  - Show UI: org chart, session tracking, audit trail
  - Narration: problem, solution, impact
  - Tools: OBS Studio (screen recording), DaVinci Resolve (editing)

- B. Documentation (~2 days)
  - Architecture diagrams (system overview, deployment topologies, data flow)
  - API documentation (OpenAPI spec refinement, MCP tool reference)
  - User guide (quickstart, role profiles, troubleshooting)
  - Contributor guide (CONTRIBUTING.md, PR templates, issue templates)

- C. Performance Benchmarks (~1 day)
  - Latency: P50/P95/P99 for API routes (target: P50 < 5s)
  - Throughput: Requests/second (target: > 10 req/s)
  - Cost: Token usage per workflow (budget tracking)
  - Load testing: Apache Bench or wrk

- D. GitHub Repository Polish (~1 day)
  - README.md update (badges, demo GIF, features list, installation)
  - LICENSE file (Apache-2.0)
  - CODE_OF_CONDUCT.md (Contributor Covenant)
  - SECURITY.md (responsible disclosure policy)
  - Issue templates (bug report, feature request)
  - PR template (checklist, testing, docs)

- E. Grant Application Submission (~1 day)
  - Fill out Block Goose Innovation Grant form
  - Upload demo video, link GitHub repo
  - Submit answers (see docs/grant/GRANT-APPLICATION-ANALYSIS.md)
  - Post to Goose Discord/community

**Deliverables:**
- ✅ 5-minute demo video (YouTube/Vimeo public)
- ✅ Comprehensive documentation (architecture, API, user guide, contributor guide)
- ✅ Performance benchmarks documented (latency, throughput, cost)
- ✅ GitHub repository polished (README, LICENSE, COC, SECURITY, templates)
- ✅ Grant application submitted

**Effort:** S (~1 week, 5-7 days)

---

### Phases 6-12: Post-Grant Milestones (Months 4-12)
**Funded by $100K Block Goose Innovation Grant**

Phase 6: Privacy Guard Production Hardening (M) — Q2 Month 4-5
- **Ollama NER Model Integration:** ✅ ALREADY WORKING (Phase 2.2 complete)
  - Model: llama3.2:latest (via Ollama 0.12.9)
  - Detection: Hybrid approach (regex + NER for person names, organizations)
  - Enabled by default: `GUARD_MODEL_ENABLED=true` in production deployments
  - Performance: P50=22.8s (CPU-only, acceptable for backend compliance checks)
  
- **Production Hardening Focus (this phase):**
  - Smart model triggering (selective usage based on regex confidence) — 240x speedup
  - Model warm-up on startup (eliminate cold start latency)
  - Comprehensive integration tests (MCP → Controller → Guard → Response)
  - Performance optimization (target P50 < 500ms for 80-90% of requests)
  - Load testing with realistic corporate PII datasets
  - Deterministic pseudonymization (Vault-backed keys production-ready)

- **Note:** Phase 2.2 delivered working Ollama NER integration. This phase optimizes it for production scale.
- Effort: M (~4-5 days)

Phase 7: Audit/Observability Enhancement (S) — Q2 Month 5-6
- OTLP export to Grafana/Prometheus
- Dashboard templates (org-level oversight, cost tracking)
- Audit event schema extended (compliance fields)
- ndjson export for external SIEM
- Effort: S (~2 days)

Phase 8: Model Orchestration (M) — Q3 Month 7-8
- Lead/worker selection (guard → planner → worker)
- Cost-aware downshift (GPT-4 → GPT-3.5 for summaries)
- Policy constraints (Legal role: local-only models)
- Token accounting and budget alerts
- Effort: M (~3-5 days)

Phase 9: 10 Role Profiles Library (M) — Q3 Month 8-9
- Expand from 5 to 10 roles (Sales, Legal, HR, Executive, IC)
- Community templates (open library, contributions welcome)
- Profile marketplace (revenue share 80/20 creator)
- Effort: M (~3-5 days)

Phase 10: Kubernetes Deployment (M) — Q3 Month 9
- Helm charts for all services
- Auto-scaling, network policies, pod security
- Production runbooks (deployment, monitoring, troubleshooting)
- Effort: M (~3-5 days)

Phase 11: Advanced Features (M) — Q4 Month 10-11
- Approval workflows (configurable multi-stage approvals)
- Policy composition (role inheritance, override rules)
- SCIM integration (auto-provision agents from IdP)
- Compliance packs (GDPR, SOC2, HIPAA templates)
- Effort: M (~5-7 days)

Phase 12: Community & Sustainability (M) — Q4 Month 11-12
- Blog posts, conference talks (FOSDEM, KubeCon, AI Engineer Summit)
- Upstream PRs to Goose (Agent Mesh, Privacy Guard, profile spec)
- GitHub Discussions active, external contributors onboarded
- Business model documentation (open core, managed SaaS)
- Effort: M (~5-7 days)

---

### Grant-Aligned Timeline Summary

**Months 1-3 (Q1): Foundation & Grant Application**
- Weeks 1-2: Phase 0-3 complete ✅
- Weeks 3-4: Phase 4 (Storage/Metadata + Session Persistence)
- Weeks 5-6: Phase 5 (Directory/Policy + Profiles + Simple UI)
- Week 7: Phase 5.5 (Grant Application Demo & Submission)
- **Deliverable:** Grant application ready (v0.5.0)

**Months 4-6 (Q2): Production Hardening**
- Phase 6: Privacy Guard production-ready
- Phase 7: Audit/Observability enhanced
- First upstream PRs to Goose core
- **Deliverable:** Production-grade orchestration (v0.6.0)

**Months 7-9 (Q3): Scale & Features**
- Phase 8: Model Orchestration
- Phase 9: 10 Role Profiles library
- Phase 10: Kubernetes deployment
- **Deliverable:** Enterprise-ready platform (v0.7.0)

**Months 10-12 (Q4): Community & Sustainability**
- Phase 11: Advanced features (approval workflows, SCIM, compliance)
- Phase 12: Community engagement, blog posts, talks
- Business model validation (2 paid pilots)
- **Deliverable:** Sustainable OSS project (v1.0.0)

**Total Effort:** 12 months, $100K grant funding

**Effort Scale:** S ≤ 2d, M ~ 3–5d, L ~ 1–2w, XL > 2w

## Environment/Infra Plan
- CE defaults: Keycloak (OIDC), Vault OSS + KMS (dev: file KMS), Postgres, Ollama (guard); optional S3-compatible object storage (SeaweedFS default option; MinIO/Garage optional). Object storage is deferred until needed for large artifacts; it is not required to complete MVP flows.
- Topologies:
  - Desktop-only: Goose desktop with local guard and Mesh; optional local Keycloak (dev).
  - Dept/Org: Docker compose with controller + directory-policy + audit ingest + Postgres; agents desktop or container.
- Security zones:
  - Desktop zone (agent+guard), Org zone (controller/policy/audit), Secrets zone (Vault/KMS).

## Governance/Security/Compliance Posture
- Identity: OIDC SSO, short-lived JWT (≤30m), optional refresh via OIDC.
- Authorization: RBAC/ABAC-lite in directory-policy; extension allowlists enforce least privilege.
- Privacy: Mask-and-forward by default; deterministic mapping keys per-tenant; strict no-content server rule.
- Auditability: Structured logs, OTEL traces, audit events with redaction maps; exportable ndjson.
- Compliance-ready: SOC2-aligned practices; GDPR/CPRA pack later.

## Risks and Mitigations (Top 10)
1) Guard accuracy → Start conservative rules + evaluate; manual overrides; defense-in-depth.
2) Auth mismatch (OIDC/JWT vs X-Secret-Key) → Gateway bridge, standardized middleware.
3) Latency from guard + orchestration → Local guard optimization, prompt trimming, caching.
4) Security of inter-agent calls → JWT-bound requests, role claims, signed profiles; mTLS later.
5) Integration sprawl (MCP) → Curated allowlists per role; staged rollout; sandbox risky tools.
6) Data custody creep → Stateless content rule; redaction at source; periodic reviews.
7) Observability signal overload → Minimal baseline; dashboards later; sampling.
8) Cost overruns → Budget policy, downshift, token accounting; guard-first.
9) Change management → Seed role profiles, clear runbooks, training plan.
10) Single-tenant limits scale → Document path to multi-tenant; quotas post-MVP.

## Dependencies
- Goose v1.12 (goosed, MCP, providers, OTLP).
- Keycloak (OIDC), Vault OSS + KMS, Postgres, Ollama + small models, optional S3-compatible object storage (SeaweedFS default option; MinIO/Garage optional), OpenRouter/one-API (optional).
- Linux/macOS desktops.

## Milestones and Acceptance Criteria (Grant-Aligned)

### Q1 Milestones (Months 1-3): Foundation & Grant Application
- **M1 (Week 2) ✅ ACHIEVED:** OIDC + JWT complete, Privacy Guard MVP operational, Vault integrated
- **M2 (Week 4) ✅ ACHIEVED:** Controller API + Agent Mesh complete, cross-agent demo working, session persistence added
- **M3 (Week 7) 🎯 TARGET:** Grant application ready
  - Postgres metadata complete (sessions, tasks, approvals, audit)
  - 5 role profiles operational (Finance, Manager, Engineering, Marketing, Support)
  - Simple web UI deployed (4 pages: Dashboard, Sessions, Profiles, Audit)
  - Demo video recorded (5 minutes, Finance → Manager → Engineering workflow)
  - Performance benchmarks documented (P50 < 5s, throughput > 10 req/s)
  - Tagged release: v0.5.0
  - Grant application submitted to Block

### Q2 Milestones (Months 4-6): Production Hardening
- **M4 (Month 5):** Privacy Guard production-ready
  - Ollama NER model loaded (accurate PII detection)
  - Comprehensive integration tests (MCP → Controller → Guard)
  - Performance: P50 < 500ms validated
  - Deterministic pseudonymization production-ready
  - Tagged release: v0.6.0

- **M5 (Month 6):** Audit/Observability enhanced, first upstream PRs
  - OTLP export to Grafana/Prometheus operational
  - Audit event schema extended for compliance
  - Dashboard templates deployed (org-level oversight, cost tracking)
  - Agent Mesh + Privacy Guard PRs submitted to Goose core
  - External contributor onboarding (1-2 contributors active)

### Q3 Milestones (Months 7-9): Scale & Features
- **M6 (Month 8):** Model Orchestration complete
  - Lead/worker selection operational (guard → planner → worker)
  - Cost-aware downshift implemented (GPT-4 → GPT-3.5)
  - Policy constraints enforced (Legal role: local-only models)
  - Token accounting and budget alerts functional
  - Tagged release: v0.7.0

- **M7 (Month 9):** Enterprise-ready platform
  - 10 role profiles library complete (all departments covered)
  - Kubernetes deployment operational (Helm charts, auto-scaling)
  - Production runbooks finalized (deployment, monitoring, troubleshooting)
  - 2 paid pilots underway ($10K each — business validation)

### Q4 Milestones (Months 10-12): Community & Sustainability
- **M8 (Month 11):** Advanced features deployed
  - Approval workflows configurable (multi-stage approvals)
  - SCIM integration operational (auto-provision agents from IdP)
  - Compliance packs available (GDPR, SOC2, HIPAA templates)
  - Policy composition working (role inheritance, override rules)

- **M9 (Month 12):** Sustainable OSS project
  - 100 production deployments (tracked via opt-in telemetry)
  - 10 external contributors (non-grant-funded PRs merged)
  - 5 upstreamed PRs to Goose core (Agent Mesh, Privacy Guard, docs)
  - 3 conference talks/blog posts (FOSDEM, KubeCon, AI Engineer Summit)
  - Business model validated (2 paid pilots complete, renewal contracts signed)
  - Tagged release: v1.0.0
  - Grant deliverables complete

**Success Criteria (12 Months):**
- ✅ Grant application submitted (Q1)
- ✅ Production-grade orchestration (Q2)
- ✅ Enterprise-ready platform (Q3)
- ✅ Sustainable OSS project (Q4)
- ✅ 100 deployments, 10 contributors, 5 upstream PRs
- ✅ Business model validated ($20K pilot revenue)

## RACI and Stakeholder Map
- R: Tech Lead (core orchestration), Security Lead (identity/keys), Infra Lead (deployments), PM (scope/priorities).
- A: CTO/Architect for final decisions (ADRs).
- C: Department champions (Marketing/Finance/Eng) for role profiles.
- I: Compliance/IT Ops (audit and SLOs).

## Now/Next/Later Roadmap
- Now (MVP): Identity bridge, Mesh MCP, Privacy Guard, Controller OpenAPI, Profiles/Policy-lite, Audit baseline, Cost-aware routing.
- Next (v1.3): Policy composition/graph, dashboards, Bus adapter interface, SCIM.
- Later (v1.4): Multi-tenant hardening, advanced approvals, compliance packs, analytics, mTLS.

---

## Upstream PR Opportunities — Contributing to Goose Core

**Purpose:** Document innovations built in this project that could benefit the broader Goose ecosystem. These contributions align with the Block Goose Innovation Grant's "previous collaboration" question and demonstrate community value.

### 🎯 Target: 5 Upstreamed PRs by Month 12

---

### 1. Privacy Guard MCP Integration Patterns (Phase 6 - Q2 Month 5)

**What We're Building:**
- MCP tool wrapper that masks PII before sending to LLM
- Generic middleware pattern: any MCP tool → Privacy Guard → LLM
- Configurable modes: Off/Detect/Mask/Strict (user-selectable)
- Deterministic pseudonymization (Vault-backed keys)

**Upstream PR Proposal:**
- **Component:** goose-mcp crate (Rust)
- **Change:** Add optional `privacy_guard` middleware to MCP tool registration
- **Example:**
  ```rust
  server.register_tool(
      "web_search",
      Tool::new(web_search_handler)
          .with_privacy_guard(PrivacyGuardConfig {
              mode: PrivacyMode::Mask,
              vault_url: env::var("VAULT_URL")?,
              pii_patterns: vec![Pattern::Email, Pattern::SSN, Pattern::CreditCard],
          })
  );
  ```
- **Benefit:** Enterprise users get PII protection out-of-the-box
- **Documentation:** Add privacy-first MCP patterns to Goose architecture docs
- **Timeline:** PR draft by Month 5, merged by Month 6
- **Estimated Effort:** 3-5 days (extract, generalize, document, PR review cycle)

---

### 2. OIDC/JWT Middleware for MCP Servers (Phase 5 - Q1 Week 6)

**What We're Building:**
- Keycloak OIDC integration for Controller API
- JWT verification middleware (RS256, JWKS caching, clock skew tolerance)
- Role-based access control (RBAC) using JWT claims

**Upstream PR Proposal:**
- **Component:** goosed (Rust server framework)
- **Change:** Add `oidc_middleware` module for JWT verification
- **Example:**
  ```rust
  let app = Router::new()
      .route("/tasks/route", post(route_task))
      .layer(OidcMiddleware::new(OidcConfig {
          issuer_url: "https://keycloak.example.com/realms/prod",
          jwks_url: "https://keycloak.example.com/realms/prod/protocol/openid-connect/certs",
          audience: "goose-server",
          clock_skew_secs: 60,
      }));
  ```
- **Benefit:** Enterprise deployments get OIDC SSO without custom middleware
- **Documentation:** Add OIDC setup guide to Goose server docs
- **Timeline:** PR draft by Week 6, merged by Month 4
- **Estimated Effort:** 2-3 days (extract, generalize, document, PR review cycle)

---

### 3. Agent Mesh Protocol — Multi-Agent Coordination (Phase 5 - Q1 Week 7)

**What We're Building:**
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status)
- Cross-agent communication protocol (JSON over HTTP)
- Session tracking and approval workflows

**Upstream PR Proposal:**
- **Component:** New MCP extension or core feature
- **Change:** Add `agent_mesh` MCP tools to Goose core
- **Spec:**
  ```yaml
  tools:
    - name: send_task
      description: Route a task to another agent role
      params:
        - target: string (required) - target agent role
        - task: object (required) - task payload
        - context: object (optional) - additional context
    
    - name: request_approval
      description: Request approval from another agent
      params:
        - task_id: string (required) - task identifier
        - approver_role: string (required) - approver's role
        - reason: string (required) - approval reason
    
    - name: notify
      description: Send notification to another agent
      params:
        - target: string (required) - target agent role
        - message: string (required) - notification message
        - priority: string (optional) - low/normal/high
    
    - name: fetch_status
      description: Get task/session status
      params:
        - task_id: string (required) - task identifier
  ```
- **Benefit:** Enable multi-agent workflows without custom infrastructure
- **Documentation:** Add multi-agent orchestration guide to Goose docs
- **Timeline:** PR draft by Week 7, merged by Month 7
- **Estimated Effort:** 5-7 days (extract, generalize, document, community feedback, PR review cycle)

---

### 4. Session Persistence Patterns (Phase 4 - Q1 Week 4)

**What We're Building:**
- Postgres-backed session storage (sessions, tasks, approvals, audit)
- Session lifecycle management (pending → active → completed/failed)
- Idempotency deduplication (Redis-backed, 24h TTL)

**Upstream PR Proposal:**
- **Component:** goosed (server framework)
- **Change:** Add optional `session_persistence` module with pluggable backends
- **Example:**
  ```rust
  // Postgres backend
  let session_store = PostgresSessionStore::new(pool);
  
  // Redis cache (optional)
  let cache = RedisCache::new(redis_pool);
  
  let app = Router::new()
      .route("/sessions", get(list_sessions).post(create_session))
      .with_state(AppState { session_store, cache });
  ```
- **Benefit:** Production deployments get session persistence without custom implementation
- **Documentation:** Add stateful workflows guide to Goose docs
- **Timeline:** PR draft by Week 4, merged by Month 4
- **Estimated Effort:** 4-6 days (extract, generalize, support multiple backends, document, PR review cycle)

---

### 5. Role Profile Specification (Phase 5 - Q1 Week 6)

**What We're Building:**
- YAML/JSON profile spec: role, display_name, description, extensions, recipes, policies, env_vars
- Signed profiles (Vault transit key or HMAC)
- Policy engine: can_use_tool(role, tool_name), can_access_data(role, data_type)

**Upstream PR Proposal:**
- **Component:** Goose configuration + docs
- **Change:** Add `profiles/` directory with role profile spec
- **Spec Format:**
  ```yaml
  # profiles/finance.yaml
  role: finance
  display_name: "Finance Team Agent"
  description: "Budget approvals, reporting, compliance"
  
  extensions:
    - name: github
      allowed_tools: ["list_issues", "create_issue", "add_comment"]
    - name: agent_mesh
      allowed_tools: ["send_task", "request_approval", "notify", "fetch_status"]
  
  policies:
    - allow_tool: "github__list_issues"
      conditions:
        - repo: "finance/*"
    - deny_tool: "developer__shell"
      reason: "No code execution for Finance role"
  
  env_vars:
    PRIVACY_GUARD_MODE: "Strict"
    SESSION_RETENTION_DAYS: "90"  # Compliance requirement
  ```
- **Benefit:** Organizations get role-based agent configuration without custom code
- **Documentation:** Add role profiles guide to Goose docs
- **Timeline:** PR draft by Week 6, merged by Month 8
- **Estimated Effort:** 5-7 days (spec design, validation schema, examples, documentation, PR review cycle)

---

### Grant Application Mapping

**Block Goose Innovation Grant Question:**
> "Have you contributed to the Goose project before? If so, please provide links to previous pull requests, issues, or other contributions."

**Answer (Post-Phase 12):**
> Yes. As part of the org-chart-goose-orchestrator project (funded by Block Goose Innovation Grant #2026-Q1), we have contributed the following to Goose core:
>
> 1. **Privacy Guard MCP Integration** (PR #XXXX) - Merged Month 6
>    - Middleware pattern for PII masking in MCP tools
>    - Enterprise-ready privacy protection
>
> 2. **OIDC/JWT Middleware** (PR #XXXX) - Merged Month 4
>    - Keycloak SSO integration for goosed
>    - Role-based access control
>
> 3. **Agent Mesh Protocol** (PR #XXXX) - Merged Month 7
>    - Multi-agent coordination tools
>    - Cross-agent communication protocol
>
> 4. **Session Persistence Patterns** (PR #XXXX) - Merged Month 4
>    - Postgres-backed session storage
>    - Pluggable backend architecture
>
> 5. **Role Profile Specification** (PR #XXXX) - Merged Month 8
>    - YAML/JSON profile format
>    - Policy engine for tool allowlists
>
> All PRs include comprehensive documentation, tests, and examples. Total lines contributed: ~2,500 (code) + ~5,000 (docs).
>
> GitHub repository: https://github.com/JEFH507/org-chart-goose-orchestrator

---

### Success Metrics (12 Months)

- ✅ 5 PRs merged to Goose core (target: all 5 by Month 8)
- ✅ 2,500+ lines of code contributed
- ✅ 5,000+ lines of documentation contributed
- ✅ 3 conference talks/blog posts referencing contributions (FOSDEM, KubeCon, AI Engineer Summit)
- ✅ External contributors adopting patterns (tracked via GitHub stars, forks, issues)
- ✅ Goose maintainers recognize project as "exemplar" for enterprise deployments

---

### Community Engagement Strategy

**Months 4-6 (Q2): Early Upstream Work**
- Draft PRs for Privacy Guard + OIDC middleware
- Engage with Goose maintainers on design review
- Iterate based on feedback
- Merge 2 PRs (Privacy Guard, OIDC)

**Months 7-9 (Q3): Core Contributions**
- Submit Agent Mesh protocol PR (most impactful)
- Submit Session Persistence PR
- Merge 2 PRs (Agent Mesh, Session Persistence)

**Months 10-12 (Q4): Documentation + Community**
- Submit Role Profile Specification PR
- Write blog posts on each contribution
- Present at conferences (FOSDEM, KubeCon)
- Engage with external contributors (GitHub Discussions)
- Merge 1 PR (Role Profiles)

---

**Total Upstream Contribution Effort:** ~25-35 days (across 12 months)

**Timeline:**
- Month 4: OIDC middleware PR merged
- Month 5: Privacy Guard PR drafted
- Month 6: Privacy Guard PR merged
- Month 7: Agent Mesh PR merged + Session Persistence PR merged
- Month 8: Role Profiles PR merged
- Months 9-12: Blog posts, talks, community engagement
