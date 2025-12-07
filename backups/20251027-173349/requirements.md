# Org-Chart Orchestrated AI Framework — Requirements (MVP)

This document captures concise, company-agnostic requirements for an org-chart–aware AI orchestration framework that delivers role-based "digital twin" assistants with strong privacy, governance, and auditability.

## Personas and target markets
- CIO/CTO (strategy and platform ownership)
- CISO/Compliance Officer (risk, privacy, governance)
- Department Leaders (Marketing, Finance, Engineering, Support, Legal)
- IT Operations / Platform Engineering (deployment, SSO, networking, observability)
- Individual Contributors (role-specific assistants and workflows)

## Value proposition
- Deliver measurable productivity by mapping AI assistants to the org chart and role profiles.
- Enforce privacy-by-design via local guard, least-privilege tool access, and audit trails.
- Standardize processes with reusable recipes while allowing department-level customization.
- Deploy flexibly (desktop, endpoint, org-wide, hybrid) with vendor-neutral model strategy.

## Primary user journeys (MVP-first)
1) Role onboarding and access
   - Admin connects SSO, imports org structure, assigns role profiles (policies, tools, recipes).
2) IC executes a guided workflow
   - An IC uses a desktop agent configured by their role profile to run a prebuilt recipe (e.g., compile a report), with privacy guard masking sensitive data before any cloud call.
3) Cross-agent task routing and approvals
   - A manager initiates a multi-step workflow that routes tasks to department agents, collects approvals, and aggregates status.
4) Governance and audit
   - Compliance reviews an auditable trace of actions, tool usage, model calls, and redactions across agents.
5) Operations visibility
   - Platform team views health, cost, and performance dashboards (OTEL traces, logs, metrics) and adjusts policies.

## Constraints (budget, timeline, platform, compliance)
- Budget
  - Optimize for cost: prefer open-source components and cost-aware model routing; enable basic cost caps and telemetry.
- Timeline
  - Target MVP within a quarter (assumption), phased: Prototype → Pilot → Department rollout.
- Platform
  - Desktop: Windows/macOS/Linux for IC agents. Server: containerized services (Kubernetes optional) with Postgres for metadata and object storage for artifacts.
  - Integrations via standards (MCP tools) and ACP-compatible agent endpoints; optional pub/sub (e.g., NATS/Kafka) for async orchestration.
- Compliance
  - Privacy-by-design defaults (mask-and-forward); SSO (OIDC/SAML), RBAC/ABAC, BYOK; configurable data residency; auditable logs with configurable retention.

## Non-functional requirements (performance, security, reliability)
- Performance
  - Typical single-agent task P50 < 15s, P95 < 60s; cross-agent workflows run asynchronously with progress updates; local guard adds minimal overhead.
- Security
  - SSO, short-lived tokens, RBAC/ABAC; extension allowlists; encryption in transit (TLS) and at rest; secrets in KMS/Vault; deterministic pseudonymization keys per tenant.
- Reliability & resilience
  - Orchestrator targets 99.5% availability (MVP); retries with backoff, idempotent operations, backpressure/circuit breakers; graceful degradation to local-only when cloud unavailable.
- Observability
  - End-to-end tracing (OTEL), structured logs, metrics, and audit events; dashboards for org-level oversight and cost monitoring.
- Scalability
  - MVP supports up to 100 concurrent agent instances with horizontal scale paths defined.

## Risks and assumptions
- Risks
  - Privacy guard accuracy tradeoffs (masking vs. utility) affecting UX and quality.
  - Integration sprawl and maintenance burden across tools and departments.
  - Latency and cost increases from multi-agent/guard orchestration.
  - Security of agent-to-agent communications and policy enforcement.
  - Change management and adoption friction across departments.
- Assumptions
  - MVP runs single-tenant per org; deployable in customer VPC or on-prem.
  - SSO via OIDC; SCIM provisioning optional for MVP.
  - Deterministic pseudonymization with per-tenant keys in KMS/Vault; default policy masks PII and secrets.
  - Allowed models include local and major cloud LLMs; router can downshift to cheaper models for summaries.
  - Initial focus roles: Finance, Marketing, Engineering; core MCP integrations for files, repos, docs, and common SaaS.
  - Audit log retention default 90 days (configurable); data residency configurable by tenant.

---

## Appendix A — MVP implementation choices (PO alignment)

The following choices keep the MVP lean (low ops) while preserving a clean path to enterprise. They align with goose guides (Permissions, Security, Logging System, Multi‑Model Config, Providers) and the goose v1.12.00 snapshot (MCP‑first, extension allowlists, OTel‑ready, CLI/Desktop parity).

- Tenancy and hosting
  - Single‑tenant per org for MVP.
  - Runtime: Desktop‑only or Desktop + tiny controller (single‑process HTTP) on Ubuntu.
  - Networking: HTTP‑only for MVP; no message bus yet.
  - Persistence: If login or multi‑user persistence is needed, use a managed DB/Auth (see below). Otherwise, keep file/local state only.

- Identity and provisioning
  - MVP: OIDC SSO for users; controller mints short‑lived JWT for agent/service calls.
  - SCIM: Optional; add when needed.
  - Agent‑to‑agent auth: JWT for MVP; add mTLS + JWT in enterprise.

- Compliance and audit
  - Frameworks: SOC2‑aligned practices and GDPR principles (not certification) for MVP; HIPAA/PCI out of scope.
  - Data residency: Not enforced in MVP; document as TBD for post‑MVP.
  - Audit: Structured logs + session history per goose’s Logging System; retain 3–12 months (TBD exact).

- Privacy guard policy
  - Mask PII and secrets by default; PHI only if explicitly in scope later.
  - Unsafe content: mask‑and‑forward by default; strict block for credentials/API keys/PAN‑like data.
  - Keys: Per‑tenant secrets; Vault + KMS from MVP (CE: Vault OSS; SaaS: managed Vault), desktop uses OS keychain/sops for local secrets.
  - Implementation: Agent-level pre/post guard (local-first with Ollama + regex/rules) before any cloud call; optional provider middleware as defense-in-depth; re‑identify post‑response for allowed fields.

- Model policy and cost/latency targets
  - One‑API provider: Tetrate Agent Router (recommended in goose) or OpenRouter to access multiple models.
  - Cloud model: Favor strong tool‑calling (Claude 4 family) for goose’s extension/tool usage.
  - Local model: Lightweight model via Ollama for guard tasks (e.g., Llama 3.2 3B/Qwen 2.5 3B, quantized), plus regex/rules.
  - Budget: $200–$500/month for MVP.
  - Latency: Interactive p50 ≤ 5s, p95 ≤ 15s; background tasks may exceed.

- Platforms and infra
  - Desktop: macOS + Linux for MVP; Windows later.
  - Server OS (if controller used): Ubuntu.
  - Components (MVP‑basic): No NATS; no heavy observability; structured logs only.
  - Managed DB/Auth (optional): If the demo needs login and persistent multi‑user state, use a managed option (e.g., Supabase: Postgres + JWT Auth). If not, skip to minimize ops.
  - goose alignment: Use extension allowlist and permission modes (Ask/Approve); configure via `goose configure`; use `.gooseignore` for safety.

- Initial MVP scope (re‑stated)
  - Roles: Marketing (IC + Manager) and Finance (IC + Approver).
  - Journeys: Task routing; Cross‑dept review; Approval flow.

- Timeline and capacity
  - 6–8 weeks (solo). Milestones:
    - M0: Agent ↔ local guard ↔ cloud model path
    - M1: Minimal router decisions + handoffs (HTTP‑only)
    - M2: Identity (JWT) + simple secrets + audit events
    - M3: Demo scenario + documentation

- Not‑yet‑defined (post‑MVP)
  - NATS/Kafka bus, advanced policy composition, enterprise SSO (OIDC/SAML), SCIM, OTel/Prom/Grafana/Loki/Tempo stack, data residency specifics, retention beyond demo defaults.

References
- goose guides: Providers, Multi‑Model Config, Permissions, Security, Logging System, Using Gooseignore, Managing Tools.
- Project docs: docs/architecture/mvp.md, docs/adr/0001–0005.
