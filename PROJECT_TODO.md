# Project TODO — Execution Checklist (Derived from Master Plan WBS)

This checklist mirrors the Technical Project Plan (MVP 6–8 weeks). Keep items short and actionable.

## Phase 0 — Project setup (S)
- [ ] Repo hygiene: branch protections, conventional commits, PR template
- [ ] Dev env bootstrap docs (Linux/macOS)
- [ ] CE defaults: version pinning approach (Keycloak, Vault OSS, Postgres, Ollama)

### Phase 0 — Detailed checklist and current status (for Orchestrator continuity)

- Workstream A: Repo scaffolding & hygiene
  - [ ] .github templates (issues + PR): ISSUE_TEMPLATE/bug_report.md, ISSUE_TEMPLATE/feature_request.md, PULL_REQUEST_TEMPLATE.md, optional CODEOWNERS
  - [ ] CONTRIBUTING.md
  - [ ] docs/conventions/commit-style.md (Conventional Commits)
  - [x] VERSION_PINS.md (Keycloak, Vault, Postgres, Ollama, S3 options)

- Workstream B: Environment bootstrap (docs)
  - [x] docs/guides/dev-setup.md
  - [ ] docs/guides/keycloak-dev.md (local CE IdP how-to)
  - [x] docs/security/profile-bundle-signing.md
  - [x] docs/guides/ports.md
  - [ ] Update dev-setup.md with system-wide installers policy and link to official Docker docs

- Workstream C: CE docker-compose (infra only)
  - [ ] deploy/compose/ce.dev.yml (Keycloak, Vault, Postgres, Ollama; S3 OFF by default)
  - [ ] deploy/compose/healthchecks/*.sh
  - [ ] .env.ce.example with overridable ports; ensure deploy/compose/.env.ce is git-ignored
  - [x] docs/guides/compose-ce.md (Phase 0 scaffold)

- Workstream D: Placeholders and schemas
  - [ ] docs/api/controller/openapi.yaml (stub MVP endpoints per ADR-0010)
  - [ ] docs/api/schemas/README.md
  - [ ] docs/audit/audit-event.schema.json (stub; ADR-0008)
  - [ ] docs/policy/profile-bundle.schema.yaml (stub; ADR-0016)
  - [ ] config/profiles/sample/marketing.yaml.sig (placeholder)
  - [ ] db/migrations/metadata-only/0001_init.sql (stub) and db/README.md
  - [x] docs/tests/smoke-phase0.md

- Workstream E: Secrets & security docs
  - [x] docs/security/profile-bundle-signing.md (key mgmt)
  - [ ] Document dev secrets handling in dev-setup.md and compose-ce.md

- Workstream F: Reviews and acceptance
  - [ ] Review against ADRs 0001–0013; reference 0014–0016
  - [ ] Record acceptance sign-off in CHANGELOG.md

Notes
- Optional Workstream G (repo reorg) was completed via PRs and merged; README updated accordingly.
- S3-compatible storage is OFF by default (ADR-0014). Guard model policy documented (ADR-0015). Profile signing key mgmt documented (ADR-0016).

## Phase 1 — Identity & Security (M)
- [ ] OIDC SSO (Keycloak CE) working locally
- [ ] JWT minting/validation libs in gateway
- [ ] Role claims mapping (IdP groups → roles)
- [ ] goosed bridge (JWT→X-Secret-Key) operational
- [ ] Admin config: client IDs, redirect URIs, JWKS exposure
- [ ] Auth audit events emitted (login, token exchange)

## Phase 2 — Privacy Guard (M)
- [ ] PII regex/ruleset (baseline) committed
- [ ] Deterministic mapping (HMAC) with per-tenant keys
- [ ] Provider wrapper hooks (pre/post) integrated
- [ ] Redaction logs with counts; no raw PII in logs
- [ ] Guard P50 ≤ 500ms on commodity laptop (bench result)

## Phase 3 — Controller API + Agent Mesh (L)
- [ ] OpenAPI v1 published (tasks, approvals, sessions, profiles proxy, audit ingest)
- [ ] Controller routes with JWT auth middleware
- [ ] Agent Mesh MCP tools (send_task, request_approval, notify, fetch_status)
- [ ] Idempotency + retry w/ jitter + request size limits
- [ ] Integration test: cross-agent approval demo (stub OK)

## Phase 4 — Directory/Policy + Profiles (M)
- [ ] Profile bundle schema (YAML) + signature (Ed25519)
- [ ] GET /profiles/{role} and POST /policy/evaluate
- [ ] Enforce extension allowlists per role
- [ ] Policy default-deny with explainable deny reasons

## Phase 5 — Audit & Observability (S)
- [ ] AuditEvent schema adopted and documented
- [ ] POST /audit/ingest with Postgres index
- [ ] ndjson export implemented
- [ ] OTLP config examples (local dev)

## Phase 6 — Model Orchestration (M)
- [ ] Model registry config (models.yaml) + pricing
- [ ] Lead/worker selection wiring (guard-first)
- [ ] Policy hook: sensitivity → local-only routing
- [ ] Usage accounting recorded in audit cost

## Phase 7 — Storage/Metadata (S)
- [ ] Migrations for sessions/tasks/approvals/audit index
- [ ] Retention job (TTL) for audit index
- [ ] Verify metadata-only (no raw content) persists server-side

## Phase 8 — Packaging/Deployment + Docs (M)
- [ ] docker-compose (Keycloak, Vault, Postgres, controller, directory)
- [ ] Desktop packaging guidance (Electron/Goose)
- [ ] .env.example + secrets bootstrap guidance (dev)
- [ ] Health checks + smoke tests docs

## Cross-cutting — Acceptance & Demo
- [ ] Smoke E2E: login → agent → guard → simple route → audit event
- [ ] Full demo scenario: multi-agent approval with policy enforcement
- [ ] Performance checks: interactive P50 ≤ 5s, P95 ≤ 15s
- [ ] Compliance posture doc: privacy-by-design, data retention, roles & responsibilities

## ADRs — Decisions (MVP)
- [ ] 0006 Identity/Auth Bridge
- [ ] 0007 Agent Mesh MCP
- [ ] 0008 Audit Schema & Redaction
- [ ] 0009 Pseudonymization Keys
- [ ] 0010 Controller OpenAPI
- [ ] 0011 Signed Profiles/Policy Evaluate
- [ ] 0012 Metadata-only Storage
- [ ] 0013 Lead/Worker Model Orchestration
- [x] 0014 CE Object Storage Default and Provider Policy
- [x] 0015 Guard Model Policy and Selection
- [x] 0016 CE Profile Signing Key Management

## Ownership & Dates
- [ ] Assign owners for each Phase/Component
- [ ] Add target dates (Weeks 1–6) and link to PRs

## References
- Technical Project Plan/master-technical-project-plan.md
- Technical Project Plan/components/*
- docs/adr/0006–0013
