# Org-Chart Orchestrated AI Framework — Technical Project Plan
Version: v1.0 (Draft)
Date: 2025-10-27
Alignment: Goose v1.12.00 architecture, Org Chart Goose analysis (2025-10-27)

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

## Work Breakdown Structure (WBS) and Timeline (6–8 weeks)
Phase 0: Project setup (S)
- Repo scaffolding, env bootstrap, CE defaults docker-compose.

Phase 1: Identity & Security (M)
- OIDC login, JWT minting, gateway-to-goosed auth bridge; Vault OSS wiring.

Phase 2: Privacy Guard (M)
- Local runtime (regex + rules + small LLM), deterministic pseudonymization keys, logs redaction.

Phase 3: Controller API + Agent Mesh (L)
- Minimal OpenAPI (tasks, approvals, sessions, profiles, audit ingest), MCP extension verbs (send_task/request_approval/notify/fetch_status).

Phase 4: Directory/Policy + Profiles (M)
- Role profile bundle format (signed), policy evaluation (RBAC/ABAC-lite), allowlists.

Phase 5: Audit/Observability (S)
- OTLP export config; audit event schema; ndjson export.

Phase 6: Model Orchestration (M)
- Lead/worker selection, cost-aware downshift; policy constraints.

Phase 7: Storage/Metadata (S)
- Postgres schema for sessions/tasks/approvals/audit index; retention baseline.

Phase 8: Packaging/Deployment + Docs (M)
- Desktop packaging guidance, docker compose for services, runbooks, demo script.

Buffer & Hardening (S)
- Latency tuning, policy tweaks, acceptance tests.

Indicative timeline:
- Weeks 1–2: Phases 0–2
- Weeks 2–4: Phases 3–4
- Weeks 4–5: Phases 5–6
- Weeks 5–6: Phases 7–8 + buffer

Effort scale: S ≤ 2d, M ~ 3–5d, L ~ 1–2w, XL > 2w.

## Environment/Infra Plan
- CE defaults: Keycloak (OIDC), Vault OSS + KMS (dev: file KMS), Postgres, Ollama (guard); optional S3-compatible object storage (SeaweedFS default option; MinIO/Garage optional).
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

## Milestones and Acceptance Criteria
- M1 (Week 2): OIDC login → JWT; guard prototype hits P50 ≤ 500ms; initial profiles/allowlists.
- M2 (Week 4): Controller API + Mesh verbs functional; cross-agent approval demo; audit events emitted.
- M3 (Week 6): Cost-aware model routing; Postgres metadata; ndjson audit export; docs/runbooks; acceptance tests pass.

## RACI and Stakeholder Map
- R: Tech Lead (core orchestration), Security Lead (identity/keys), Infra Lead (deployments), PM (scope/priorities).
- A: CTO/Architect for final decisions (ADRs).
- C: Department champions (Marketing/Finance/Eng) for role profiles.
- I: Compliance/IT Ops (audit and SLOs).

## Now/Next/Later Roadmap
- Now (MVP): Identity bridge, Mesh MCP, Privacy Guard, Controller OpenAPI, Profiles/Policy-lite, Audit baseline, Cost-aware routing.
- Next (v1.3): Policy composition/graph, dashboards, Bus adapter interface, SCIM.
- Later (v1.4): Multi-tenant hardening, advanced approvals, compliance packs, analytics, mTLS.
