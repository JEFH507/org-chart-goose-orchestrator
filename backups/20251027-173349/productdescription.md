# Org-Chart Orchestrated AI Framework

This repository contains the product design and possible structure and components for a hierarchical, org-chart–aware AI orchestration framework built on goose. It is customer-first: focused on enterprise value, privacy, and governance. Designed for both local and endpoint deployments, with a privacy-aware model strategy and role-based digital twins.

## Product Design (Customer-First)

### Executive summary
- Problem: Enterprises struggle to turn AI into measurable productivity without risking data privacy, compliance, and governance. One-size-fits-all copilots don’t fit complex org structures, access rules, and departmental workflows.
- Solution: A hierarchical, org-chart–aware AI orchestration framework that gives every employee and team a “digital twin” assistant tailored to their role, tools, and policies. It scales from individual desktop agents to organization-wide orchestrated agents with strong privacy, governance, and auditability.
- Outcome: Faster execution across departments, standardized processes via recipes, safer AI with data minimization and audit trails, and a path to enterprise-wide AI adoption that respects org structure and compliance.

### Who it’s for
- CIO/CTO, CISO/Compliance, Department Leaders (Marketing/Finance/Engineering/Support/Legal), IT Ops/Platform Teams, and Individual Contributors.

### Problems we solve
- Fragmented AI usage, lack of role relevance, privacy/compliance risk, tool sprawl, no organizational memory, limited observability.

### Value proposition
- Digital twins for each role, org-aware orchestration, privacy by design (local guard), standardization with flexibility, unified governance, open ecosystem.

### Differentiators
- Hierarchical orchestration mapped to the org chart; per-role digital twins; data minimization pipeline; vendor-neutral and open; land-and-expand deployment.

### Core capabilities
- Role profiles, orchestrated tasks, privacy guard, multi-model strategies, governance & audit, MCP integrations, flexible deployment.

### Proposed architecture (customer view)
See docs/architecture/reference_onepager.html

### Deployment modes
- Individual Desktop, Department Endpoint, Organization-wide, Hybrid.

### Security, privacy, and compliance
- BYOK + SSO; policy-driven handling; local-first privacy guard; audit/observability; extension allowlists; isolation options.

### Business model (open core + enterprise)
- Open-source core; enterprise orchestrator features; managed SaaS or self-hosted; pricing by seat/agent with add-ons; marketplace for extensions and profile packs.

### Open-source strategy
- Apache-2.0 core; open SDKs/templates; commercial orchestrator/audit/SSO/SCIM bundles; grants and community engagement.

### Success metrics
- Time savings, adoption, coverage, quality & safety, cost efficiency, collaboration.

### Customer journey
- Pilot → Department rollout → Org rollout (with orchestrator and central governance).

### Risks & mitigations
- Guard errors, vendor lock-in, integration sprawl, change management.

### Assumptions to validate later
- SSO/secrets approach, initial role profiles, must-have MCP integrations, preferred deployment, compliance scope.

## Getting Started (Project)
- This phase is design-only; implementation will follow after we finalize technical blueprint and MVP.

## License
- To be finalized (planned: Apache-2.0 for core). Enterprise components licensed separately.

# Product Possible Structure

## Guiding principles
- Hierarchical orchestration: reflect the company org chart; each node (C‑suite, department, manager, IC) has its own “goose twin” with role-specific config.
- Strong tenancy and policy: clear isolation and permissions per org, per role, per user.
- Privacy by design: optional local LLM preprocessing (“privacy guard”) to anonymize before cloud calls, deterministic re-identification on return.
- Modular and standards-based: reuse goose’s MCP tool framework and ACP client compatibility. Add orchestration as first-class primitives.
- Flexible deployment: desktop-local for individuals; containerized endpoints for teams/org; hybrid allowed.

## Core components and how they map to goose
See docs/architecture/reference_onepager.html

1) Agent instance (per role/user/department)
- Based on goose:
  - UI/CLI or pure API endpoint (ACP-compatible).
  - goose-server (goosed) exposes API; Agent Engine coordinates LLM and tools.
  - MCP extensions provide capabilities (Developer, Drive, GitHub, Postgres, etc.).
- Role profile:
  - A bundle of config: prompts, recipes, extensions allowlist/config, environment variables, policies.
  - Templates per domain: Marketing, Finance, Engineering, Support, Sales, etc.
  - Pre-installed recipes: common workflows by role (e.g., “Monthly close” for Finance, “Campaign reporting” for Marketing).
- Privacy guard extension:
  - A pre-processing/post-processing pipeline:
    - Inbound: detect/label PII, deterministically mask (pseudonymize) using a per-tenant secret; optionally use a local LLM for ambiguous detection.
    - Outbound: map masked tokens back to real values after the cloud LLM response returns.
  - Primary: Agent-level pre/post extension (local-first; open models via Ollama + rules). Optional: provider middleware wrapper as defense-in-depth.
  - Supports modes: off, detect-only, mask-and-forward, strict block if unsafe.

2) Orchestrator (new services)
- Org Directory & Policy Service:
  - Stores the org chart (graph of roles and their relationships).
  - Holds role profiles; ensures each agent instance is configured with the right templates, tools, and data access policies.
  - Manages tenancy, SSO integration, secrets (Vault/KMS), extension allowlists per role/tenant.
- Task Router & Skills Graph:
  - Knows what each agent “can do” (skills/tags derived from extensions/recipes).
  - Routes tasks to the right agent(s) based on role, skill, load, locale, etc.
  - Provides fan-out (e.g., “Finance approve this; Legal review that”).
- Cross-Agent Session Broker:
  - Creates “org sessions” that span multiple agents.
  - Maintains scoped context shards per agent, with redaction boundaries (e.g., IC doesn’t see HR PII).
  - Handles hand-offs and status aggregation (e.g., a manager view of progress).
- Audit & Observability:
  - OTEL traces across agents; per-tenant logs; action audit (who did what, where, with which tool).
  - Summaries and dashboards for org-level oversight.

3) Communication and standards
- MCP for tools: reuse goose’s extensions ecosystem.
- ACP for agent endpoints: allow external clients to talk to agent instances.
- For agent-to-agent calls:
  - Option A: a lightweight “Agent Mesh” extension exposing “send_task”/“send_note”/“request_approval” as tools, with secure HTTP/gRPC underneath.
  - Option B: Pub/sub bus (NATS/Kafka) for async coordination; messages routed by directory/router; each agent has an inbox/outbox tool.

4) Identity, access, and governance
- SSO integration (OIDC/SAML) at the Orchestrator; short-lived tokens for agent calls.
- RBAC/ABAC policies at directory level, enforced in:
  - Agent extension allowlists (what tools and endpoints are permitted).
  - Data access (MCP servers configured with scoped credentials).
- Secrets management:
  - Central: Vault/KMS for cloud credentials; agents receive ephemeral tokens.
  - Local: per-agent secrets when running on desktops; encrypted at rest.

5) Model strategy (lead/worker + “guard”)
- Support mixed models:
  - Guard model (local): anonymization, classification, cheap summarization, preliminary planning.
  - Planner (cloud or local): optional plan generation.
  - Worker (cloud): heavy reasoning; tool calling.
- Policy:
  - Sensitive text → guard preprocess → cloud worker.
  - Allow full local-only for sensitive departments (legal/HR) when needed.
- Deterministic masking:
  - Cryptographically keyed mapping for PII tokens (consistent across session/tenant); re-identify only at authorized endpoints.

6) Storage and memory
- Local (desktop) mode:
  - Keep goose defaults (config.yaml, sessions.jsonl, logs), plus per-agent encryption.
- Org mode:
  - Central Postgres for session metadata; object storage for artifacts.
  - Per-tenant keys, at-rest encryption.
  - Retention/TTL policies and redaction.

7) Deployment options
- Individual/local:
  - Desktop goose with the “Agent Mesh” turned on; privacy guard enabled; synced profile from directory.
- Team/Org:
  - Kubernetes: 1 container per agent instance (or per team) using goosed + config; UI optional (headless API).
  - Autoscaling at team/manager level; dedicated nodes for guard LLMs (GPU/CPU).
- Hybrid:
  - Employees use Desktop for their IC agents; Departments have containerized departmental agents; Orchestrator in cloud/VPC.

8) Observability and audit
- OTEL traces across agent calls (end-to-end across agents and providers).
- Structured tool logs (who/what/when/results).
- Security events (prompt injection detections, denied tools, masked data).
- Approval workflows logged for compliance.

9) Packaging role profiles
- Ship as “profiles”:
  - profile.yml includes:
    - extensions + tool permissions, recipes, prompts, env vars, router settings, allowlists.
  - Org Directory pushes profiles to agents on join/update.
  - Library of profiles for common departments; customize per org.

## How this extends goose practically
- New “Agent Mesh” extension:
  - Tools: send_task, request_approval, notify, fetch_status
  - Uses org directory to resolve endpoints; enforces policies.
- Provider wrapper in goose:
  - Hook a “privacy guard” pre/post around model calls (lead/worker/guard orchestration).
- Cross-Agent Session Broker:
  - New service; goose agents call it to create/join org sessions; broker decides context routing/redaction.
- Profiles and tenancy:
  - Bootstrap agent config from a profile service; extend goose’s config loader to accept signed profile bundles.

## Phased rollout plan
- Phase 0: Prototype
  - Single org; small team; 1 orchestrator + 3–5 agent instances (dept/manager/IC).
  - Implement “Agent Mesh” as HTTP/gRPC calls with service tokens. Use local privacy guard with regex + NER + rules.
- Phase 1: Privacy guard + simple router
  - Add deterministic PII masking; build provider wrapper; basic cross-agent session linking.
- Phase 2: Full orchestrator
  - Org Directory, Skills Router, Session Broker, audit pipelines; role profile library; SSO + Vault.
- Phase 3: Scale and polish
  - Kubernetes scaling, multi-tenant isolation, advanced analytics, cost controls, burst protection, caching.

## Key risks and tradeoffs
- Privacy guard accuracy: masking vs usefulness is a tradeoff; false positives/negatives impact UX.
- Complexity of cross-agent sessions: keeping context scoped yet coherent is non-trivial.
- Cost and latency: multiple agents + guard + planner can increase latency; router and caching help.
- Security: agent-to-agent calls must be strictly authorized and auditable.

## Relevant to define
1) Tenancy and deployment
- Are you targeting multi-tenant SaaS (one orchestrator for many orgs) or single-tenant per org?
- Preferred runtime: Kubernetes in your cloud/VPC, or on-prem?

2) Identity and auth
- Which SSO provider(s) (Okta, Azure AD, Auth0)? Need SCIM for user provisioning?
- Should agent-to-agent calls use mTLS, JWT, or both? Any compliance requirements (SOC2, HIPAA, PCI)?

3) Privacy guard
- What data classes must be masked (PII, PHI, secrets)? Should we provide prebuilt policies (GDPR/CPRA)?
- Is deterministic pseudonymization sufficient, or do you need reversible encryption for specific fields?
- Where should de-identification keys live (per-tenant in KMS/Vault)?

4) Model policy
- Which vendors/models are allowed? Any preference for local models (e.g., Llama/Qwen) and hardware?
- Budget/tokens constraints? Should router automatically downshift to cheaper models for summaries?

5) Orchestrator behavior
- Should tasks flow strictly top-down (manager approvals) or can peers collaborate laterally?
- Do you want SLAs, retries, escalation paths (e.g., manager notified after N failures)?

6) Profiles and data sources
- Which departments to start with (Marketing, Finance, Eng)? Which MCP integrations are must-have?
- Any custom/internal tools we should wrap as MCP servers?

7) Observability and audit
- What audit artifacts do you need (who/what/when/why)? How long to retain?
- Preferred telemetry stack (OTEL -> Grafana/Loki/Tempo/Prometheus/Jaeger, or vendor)?

## Grant alignment
- License: Apache-2.0 for core
- Community Edition (CE): self-hostable docker-compose (Keycloak, Vault OSS, Postgres, MinIO, Ollama)
- Business validation: pursue 1–2 paid pilots during MVP to validate PMF
- Novel interaction (TBD emphasis): Whiteboard-to-Workflow or Voice/Meeting Approvals (tracked in PROJECT_TODO)
