# Phase 0 Execution Plan (Detailed)

## Scope, Goals, Non-goals

- Scope (Phase 0: 1–2 weeks)
  - Repo scaffolding and hygiene aligned to ADRs 0001–0013 and Goose v1.12.
  - Environment bootstrap: local developer setup docs, version pinning strategy, secrets bootstrap patterns.
  - CE defaults docker-compose baseline: Keycloak (OIDC), Vault OSS, Postgres, Ollama (CPU by default), optional MinIO; health checks and env overrides only.
  - Placeholder artifacts: OpenAPI stub file structure for Controller/API (no implementation), profile/policy bundle templates, audit schema file, migration stubs for metadata-only model.
  - CI hygiene and conventions: conventional commits, PR template, branch protections, basic CI placeholders.

- Goals
  - Establish a predictable, low-friction dev environment consistent with HTTP-only and metadata-only constraints (ADR-0001, ADR-0012).
  - Prepare CE deployment baseline that can be “docker compose up” healthy without app code.
  - Produce templates/stubs that downstream phases fill in: Controller OpenAPI stub, Directory/Policy bundle format stub, AuditEvent schema, DB migration stubs.
  - Document decisions (short, linked to ADRs) and acceptance checks.

- Non-goals (Phase 0)
  - No app/service implementation code (Controller, Gateway, Directory/Policy, Mesh, Guard code) beyond scaffolding and placeholders.
  - No message bus or gRPC; strictly HTTP-only as per ADR-0001.
  - No persistence of raw content server-side; only metadata model stubs per ADR-0005/0012.
  - No production TLS automation; keep dev-only instructions.

## Alignment to Overall Phases and MVP Constraints

- ADR alignment:
  - HTTP-only orchestration (ADR-0001).
  - Privacy Guard agent-side placement and defense-in-depth later (ADR-0002).
  - Secrets via Vault OSS + KMS patterns (ADR-0003).
  - OIDC SSO (Keycloak CE) and JWT bridge plan (ADR-0004/0006).
  - Data retention: metadata-only on server (ADR-0005/0012).
  - Agent Mesh via Controller API and MCP verbs (ADR-0007/0010).
  - Audit schema/redaction maps baseline (ADR-0008).
  - Deterministic pseudonymization keys (ADR-0009).
  - Lead/worker cost-aware orchestration (ADR-0013).
- Goose v1.12 alignment:
  - Reuse goosed as-is with X-Secret-Key initially, fronted by identity gateway later (ADR-0006).
  - Keep OTLP-ready observability expectations and OpenAPI generation in mind.
  - MCP-first tools, HTTP-only flows.

## Deliverables/Artifacts (Phase 0)

- Repo scaffolding and hygiene
  - .github/ templates: PULL_REQUEST_TEMPLATE.md, ISSUE_TEMPLATE/bug.md, feature.md
  - CONTRIBUTING.md (links to ADRs, commit message conventions)
  - CODEOWNERS (optional)
  - docs/conventions/commit-style.md (conventional commits)
  - scripts/dev/ (bootstrap scripts placeholders)
- Environment bootstrap
  - docs/guides/dev-setup.md (Linux/macOS), including prerequisites and ports map
  - .env.example populated with CE service defaults and overridable ports
  - docs/security/secrets-bootstrap.md (Vault OSS dev mode flow; OS keychain local-only)
  - VERSION_PINS.md (Keycloak, Vault, Postgres, Ollama, MinIO versions)
- CE defaults docker-compose
  - deploy/compose/ce.dev.yml (Keycloak, Vault, Postgres, Ollama, optional MinIO)
  - deploy/compose/.env.ce (example)
  - deploy/compose/healthchecks/ (curl-based checks)
  - docs/guides/compose-ce.md (how to run, seed Keycloak realm, seed Vault dev)
- Placeholder APIs/schemas/config
  - docs/api/controller/openapi.yaml (stubbed paths per ADR-0010)
  - docs/policy/profile-bundle.schema.yaml and example: config/profiles/sample/marketing.yaml.sig (placeholder)
  - docs/audit/audit-event.schema.json per ADR-0008
  - db/migrations/metadata-only/0001_init.sql (stub with TODOs; tables: sessions_meta, tasks_meta, approvals_meta, audit_index)
- Acceptance and smoke definitions
  - docs/tests/smoke-phase0.md (what “healthy” means for Phase 0 compose; curl checks)
  - docs/runbooks/README.md (placeholders)

## Work Breakdown Structure (WBS)

- Workstream A: Repo hygiene and conventions (0.5–1 day)
  - Setup PR templates, commit convention docs, branch protections draft.
  - Dependencies: none.
- Workstream B: Dev environment bootstrap docs (1 day)
  - dev-setup.md, ports table, prerequisites, make targets or script placeholders.
  - Dependencies: A.
- Workstream C: CE docker-compose baseline (1.5–2 days)
  - ce.dev.yml with Keycloak, Vault, Postgres, Ollama, optional MinIO.
  - Healthcheck scripts and .env overrides; version pinning doc.
  - Dependencies: B.
- Workstream D: Placeholder schemas and stubs (1 day)
  - Controller OpenAPI stub (endpoints only); AuditEvent schema JSON; profile bundle schema and signed file placeholder; migrations stub.
  - Dependencies: A, B.
- Workstream E: Secrets bootstrap docs (0.5 day)
  - Vault OSS dev mode guidance; local OS keychain notes.
  - Dependencies: C.
- Workstream F: Acceptance criteria and smoke checks (0.5 day)
  - Smoke plan; curl checks; checklist.
  - Dependencies: C, D.

Estimates: ~4.5–6 days net effort (single engineer); with review/iteration fits 1–2 weeks.

Key dependencies
- Docker installed; ports free.
- No GPU required; Ollama CPU models only for Phase 0.
- Access to ADRs and Goose v1.12 architecture (already present).

## Milestones and Acceptance Criteria

- M0.1 Repo ready (Day 2–3)
  - Acceptance: PR template active; CONTRIBUTING.md present; conventional commit doc present.
- M0.2 CE compose up (Day 4–5)
  - Acceptance: `docker compose -f deploy/compose/ce.dev.yml up -d` starts all services; curl health checks pass; version pins recorded.
- M0.3 Placeholders in place (Day 6–7)
  - Acceptance: OpenAPI stub validates with openapi-lint; audit schema file present; profile bundle schema present with example signed filename; migration stub files exist.
- M0.4 Docs pass (Day 7–9)
  - Acceptance: dev-setup.md and compose-ce.md validated by a second person; smoke-phase0.md reproducible.

## Risks and Mitigations (Phase 0 specific)

- Service version drift or breaking changes → Mitigation: explicit VERSION_PINS.md; hash/tag specific images.
- Port conflicts on developer machines → Mitigation: .env.ce overrides and documented alternate port mapping.
- Keycloak/Vault initial configuration complexity → Mitigation: provide dev-mode defaults, seed scripts later; document minimum.
- Ollama resource usage on laptops → Mitigation: use smallest model tags; CPU-only; document how to disable Ollama service.
- Team variability in Docker support (e.g., macOS issues) → Mitigation: fallback to Podman notes; optional “compose without Ollama.”
- Over-configuring Phase 0 (scope creep) → Mitigation: placeholders only, no code; stick to acceptance list.

## RACI and Roles (Phase 0)

- Responsible (R)
  - Infra Lead: compose baseline, health checks, version pinning.
  - Docs Lead: dev-setup/guides, acceptance docs.
  - Security Lead: secrets bootstrap guidance, metadata-only review.
- Accountable (A)
  - Tech Lead: approves structure, alignment to ADRs, final acceptance.
- Consulted (C)
  - PM: prioritization, scope boundaries.
  - Compliance/IT: metadata-only and privacy posture sign-off.
- Informed (I)
  - All contributors via README/PR templates.

## Timeline (1–2 weeks target)

- Days 1–2: Workstream A + B
- Days 3–4: Workstream C (compose baseline)
- Day 5: Workstream D (schemas/stubs)
- Day 6: Workstream E (secrets bootstrap) + start F
- Day 7–8: Complete F, polish docs, internal review
- Buffer (Day 9–10): Fixes from review; acceptance sign-off
