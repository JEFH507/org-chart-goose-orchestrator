# Project TODO — Execution Checklist (Derived from Master Plan WBS)

This checklist mirrors the Technical Project Plan (MVP 6–8 weeks). Keep items short and actionable.

**Status Legend:**
- ✅ Complete — Phase closed with summary/tag
- ⚙️ In Progress — Currently executing
- 📋 Planned — Ready to start
- ⏸️ Deferred — Post-MVP

---

## Phase 0 — Project Setup (S) — ✅ COMPLETE
**Closed:** 2025-10-31 | **Tag:** `phase0-complete` | **PRs:** #3–#15

### Summary
- [x] Repo hygiene: branch protections, conventional commits, PR template
- [x] Dev env bootstrap docs (Linux/macOS)
- [x] CE defaults: version pinning approach (Keycloak, Vault OSS, Postgres, Ollama)
- [x] Docker compose baseline (infra only)
- [x] OpenAPI stub with schema placeholders
- [x] DB migration stubs (metadata-only)
- [x] Optional Workstream G (repo reorg) completed

**Detailed completion documented in:**
- `Technical Project Plan/PM Phases/Phase-0/Phase-0-Summary.md`
- `docs/tests/phase0-progress.md`

---

## Phase 1 — Initial Runtime (M) — ✅ COMPLETE
**Closed:** 2025-11-01 | **Summary:** `Technical Project Plan/PM Phases/Phase-1/Phase-1-Completion-Summary.md`

### Delivered
- [x] Minimal controller runtime in Rust (Axum-based)
  - [x] GET /status → 200 with version
  - [x] POST /audit/ingest → 202 (logs metadata, no persistence)
  - [x] Other endpoints → 501
- [x] Controller integrated into compose with healthchecks
- [x] CI skeleton (linkcheck, Spectral, compose health)
- [x] Keycloak/Vault dev seeding scripts (partial)
- [x] DB Phase 1 migrations (indexes/FKs)
- [x] Observability docs (structured logs, redaction)

### Gap Identified → Phase 1.2
- [ ] OIDC SSO (Keycloak CE) JWT flow ← **Moved to Phase 1.2**
- [ ] JWT verification middleware ← **Moved to Phase 1.2**
- [ ] Vault wiring validation ← **Moved to Phase 1.2**

**Note:** Phase 1 original scope included Identity & Security, but OIDC/JWT implementation was incomplete. Phase 1.2 completes this work.

---

## Phase 1.2 — Identity & Security Realignment (M) — ⚙️ IN PROGRESS
**Started:** 2025-11-01 | **Prep Complete:** 2025-11-02 | **Est. Duration:** 3–5 days

### Purpose
Complete Phase 1's original Identity & Security scope (OIDC/JWT) before moving to Phase 2 (Privacy Guard).

### Checklist (from `Phase-1.2-Checklist.md`)
- [x] Initialize Phase-1.2 scaffolding and state
- [x] ADRs finalized (0019: Auth Bridge, 0020: Vault Wiring)
- [x] Phase-1.2 prompt with concrete OIDC values
- [x] .env.ce.example updated (OIDC_*, PSEUDO_SALT)
- [x] Smoke test doc created (docs/tests/smoke-phase1.2.md)
- [ ] Update Keycloak seed (realm/client/user/roles) and docs with JWT curl
- [ ] Implement controller JWT verification middleware (JWKS, iss/aud checks)
- [ ] Document or add gateway auth bridge (compose profile optional)
- [ ] Validate Vault dev wiring; docs for reading/writing pseudo_salt and env export
- [ ] Update compose healthchecks if needed
- [ ] Run smoke tests (JWT-protected ingest flow)
- [ ] Write Phase-1.2-Completion-Summary.md and update progress/state

### Key Decisions Recorded
- Controller-side JWT verification (no dedicated gateway for MVP)
- RS256, JWKS caching, small clock skew (~60s)
- /status public; /audit/ingest protected
- Vault KV v2 path: `secret/pseudonymization:pseudo_salt`
- Dev realm: `dev`, client: `goose-controller`, audience: `goose-controller`

**State JSON:** `Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Agent-State.json`

---

## Phase 2 — Privacy Guard (M) — 📋 PLANNED
**Scope:** Rules + regex + NER; deterministic masking; default mode = mask-and-forward

- [ ] PII regex/ruleset (baseline) committed
- [ ] Deterministic mapping (HMAC) with per-tenant keys
- [ ] Provider wrapper hooks (pre/post) integrated
- [ ] Redaction logs with counts; no raw PII in logs
- [ ] Guard P50 ≤ 500ms on commodity laptop (bench result)

## Phase 2.2 — Privacy Guard Enhancement (S) — 📋 PLANNED
**Scope:** Add small local model to improve detection accuracy

- [ ] Select and integrate small local model (e.g., spaCy or Rust NER)
- [ ] Preserve existing modes (Off/Detect/Mask/Strict)
- [ ] Keep guard local-only (no cloud exposure)
- [ ] Maintain mask-and-forward default
- [ ] UI integration with lead/worker model selection

## Phase 3 — Controller API + Agent Mesh (L) — 📋 PLANNED
- [ ] OpenAPI v1 published (tasks, approvals, sessions, profiles proxy, audit ingest)
- [ ] Controller routes with JWT auth middleware (← from Phase 1.2)
- [ ] Agent Mesh MCP tools (send_task, request_approval, notify, fetch_status)
- [ ] Idempotency + retry w/ jitter + request size limits
- [ ] Integration test: cross-agent approval demo (stub OK)

## Phase 4 — Directory/Policy + Profiles (M) — 📋 PLANNED
- [ ] Profile bundle schema (YAML) + signature (Ed25519)
- [ ] GET /profiles/{role} and POST /policy/evaluate
- [ ] Enforce extension allowlists per role
- [ ] Policy default-deny with explainable deny reasons

## Phase 5 — Audit & Observability (S) — 📋 PLANNED
- [ ] AuditEvent schema adopted and documented
- [ ] POST /audit/ingest with Postgres index
- [ ] ndjson export implemented
- [ ] OTLP config examples (local dev)

## Phase 6 — Model Orchestration (M) — 📋 PLANNED
- [ ] Model registry config (models.yaml) + pricing
- [ ] Lead/worker selection wiring (guard-first)
- [ ] Policy hook: sensitivity → local-only routing
- [ ] Usage accounting recorded in audit cost

## Phase 7 — Storage/Metadata (S) — 📋 PLANNED
- [ ] Migrations for sessions/tasks/approvals/audit index
- [ ] Retention job (TTL) for audit index
- [ ] Verify metadata-only (no raw content) persists server-side

## Phase 8 — Packaging/Deployment + Docs (M) — 📋 PLANNED
- [ ] docker-compose (Keycloak, Vault, Postgres, controller, directory)
- [ ] Desktop packaging guidance (Electron/Goose)
- [ ] .env.example + secrets bootstrap guidance (dev)
- [ ] Health checks + smoke tests docs

---

## Cross-Cutting Deliverables

### Acceptance & Demo
- [ ] Smoke E2E: login → agent → guard → simple route → audit event
- [ ] Full demo scenario: multi-agent approval with policy enforcement
- [ ] Performance checks: interactive P50 ≤ 5s, P95 ≤ 15s
- [ ] Compliance posture doc: privacy-by-design, data retention, roles & responsibilities

### ADRs (Decisions Tracking)
- [x] 0001 MVP Message Bus (deferred)
- [x] 0002 Privacy Guard Placement
- [x] 0003 Secrets and Key Management
- [x] 0004 Identity and Auth
- [x] 0005 Data Retention and Redaction
- [x] 0006 Identity/Auth Bridge
- [x] 0007 Agent Mesh MCP
- [x] 0008 Audit Schema & Redaction
- [x] 0009 Pseudonymization Keys
- [x] 0010 Controller OpenAPI
- [x] 0011 Signed Profiles/Policy Evaluate
- [x] 0012 Metadata-only Storage
- [x] 0013 Lead/Worker Model Orchestration
- [x] 0014 CE Object Storage Default and Provider Policy
- [x] 0015 Guard Model Policy and Selection
- [x] 0016 CE Profile Signing Key Management
- [x] 0017 Controller Language and Runtime Choice (Rust)
- [x] 0018 Controller Healthchecks and Compose Profiles
- [x] 0019 Auth Bridge JWT Verification
- [x] 0020 Vault OSS Wiring

**Note:** ADRs 0001–0020 authored; ADRs 0006–0013 track MVP feature decisions; implementation in phases 1.2–8.

---

## Repository Housekeeping & Governance

### GitHub Repository Health (added 2025-11-03)
- [x] Clean up stale merged branches (completed 2025-11-03)
- [ ] Enable branch protection for `main` branch
  - [ ] Require pull request reviews before merging (1 approver)
  - [ ] Require status checks to pass before merging
  - [ ] Require branches to be up to date before merging
  - [ ] Include administrators
- [ ] Add repository topics/tags for discoverability
  - Suggested: `ai`, `orchestration`, `goose`, `privacy-guard`, `mcp`, `rust`, `llm`, `enterprise-ai`, `org-chart`, `digital-twins`
- [ ] Add repository description in GitHub "About" section
  - Suggested: "Hierarchical, org-chart-aware AI orchestration framework built on Goose. Privacy-first digital twin assistants for enterprise teams."
- [ ] Add GitHub Actions badges to README once CI is running

### Ownership & Dates
- [ ] Assign owners for each Phase/Component
- [ ] Add target dates (Weeks 1–6) and link to PRs

---

## References
- Technical Project Plan/master-technical-project-plan.md
- Technical Project Plan/components/*
- docs/adr/0006–0013
