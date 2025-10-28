# Org-Chart Orchestrated AI Framework — Technical Requirements (v1.2/MVP)

This document defines the technical requirements for a company-agnostic, org-chart–aware AI orchestration framework built on Goose. It is intended for architects to scaffold the system and for engineers to implement the MVP (v1.2) with a clear path to enterprise scale.

## Scope & Assumptions

### In-Scope (v1.2/MVP)
- Role-based "digital twin" assistants mapped to the org chart via role profiles.
- Desktop agents (macOS/Linux) with a local Privacy Guard at the agent pre/post boundary (mask-and-forward) before any cloud LLM calls; optional provider middleware as defense-in-depth.
- Cross-agent task routing and approvals over HTTP (no message bus in MVP).
- Identity: OIDC SSO for users; controller mints short-lived JWTs for agent/service calls.
- Secrets: Vault + KMS from MVP (CE: Vault OSS; SaaS: managed Vault). Desktop uses OS keychain/sops for local secrets.
- Cost-aware model routing (local guard + cloud worker); ability to downshift for summaries.
- Governance: extension allowlists, RBAC/ABAC policy enforcement at tool and data boundaries.
- Observability and audit: structured logs by default; OTLP/OTel-ready optional for traces/metrics (ndjson export for audits).
- Optional tiny controller (single-process HTTP) for routing/approvals when running beyond pure desktop.

### Out-of-Scope (MVP)
- Full multi-tenant SaaS (assume single-tenant per org); advanced policy composition.
- Pub/Sub bus (NATS/Kafka), mTLS, SCIM, advanced SSO features beyond basic OIDC, and data residency enforcement.
- Advanced analytics dashboards (Grafana/Loki/Tempo/Prometheus) beyond basic metrics/logs.
- Windows desktop packaging (post-MVP); Kubernetes Helm charts (optional P1).

### Key Assumptions & Constraints
- Deployment: single-tenant per org; desktop-first; optional small controller on Ubuntu.
- Identity: OIDC SSO in MVP; controller mints short-lived JWTs for agent/service calls; SCIM optional later.
- Secrets: Vault + KMS from MVP (CE: Vault OSS; SaaS: managed Vault). Desktop uses OS keychain/sops for local secrets; BYOK supported.
- Budget: target $200–$500/month (MVP). Timeline: 6–8 weeks (solo) with Prototype → Pilot → Department rollout path.
- Compliance posture: privacy-by-design defaults (mask-and-forward), auditable actions; SOC2-aligned practices without certification in MVP.

## Non-Functional Requirements (Targets)

### Performance
- Interactive single-agent actions: P50 ≤ 5s, P95 ≤ 15s (end-to-end including guard).
- General single-agent tasks: P50 < 15s, P95 < 60s.
- Cross-agent workflows: async with progress; step-level SLA targets P95 ≤ 90s; end-to-end depends on human approvals.
- Privacy Guard overhead: ≤ 500ms P50, ≤ 2s P95 per request on commodity CPU.

### Scalability
- MVP capacity: up to 100 concurrent agent instances; 300 concurrent sessions; 30 requests/sec aggregate across agents.
- Scale path: horizontal scale via additional agent instances; controller routable behind a load balancer; later bus for fan-out.

### Availability & Reliability
- Orchestrator/controller SLO (MVP): 99.5% monthly availability.
- Failure domains: agent crash isolated per user/role; controller restart non-disruptive to running agents.
- Resilience: retries with backoff, idempotent endpoints, backpressure/circuit breakers, graceful degradation to local-only when cloud unavailable.
- DR targets (MVP): RTO ≤ 4h, RPO ≤ 24h (file/state or simple DB backups if used).

### Security, Privacy, Compliance
- Identity: short-lived JWT (≤ 30m) for agent/controller; OIDC/SAML SSO post-MVP; SCIM optional later.
- Authorization: RBAC/ABAC policies at Directory/Policy layer; extension allowlists per role; least-privilege defaults.
- Data classes: PII and secrets masked by default; PHI out of scope unless explicitly enabled; configurable policies.
- Privacy Guard: deterministic pseudonymization with per-tenant keys; de-identification mapped back only at authorized endpoints.
- Encryption: TLS in transit; encryption at rest for local agent state and any server-side storage.
- Secrets: app-managed secrets in MVP; move to Vault/KMS + BYOK for enterprise.
- Audit scope: model calls, tool invocations, approvals, redaction decisions, policy denials, cost usage.

### Cost Guardrails
- Budget: $200–$500/month in MVP environments.
- Policies: per-tenant model budgets; automatic downshift to cheaper models for summarization; caching of repeated prompts where safe; guard-first to minimize cloud tokens.

## System Decomposition & Responsibilities

Reuse Goose where possible; add minimal new services for orchestration.

- Goose Agent (reuse)
  - Runs locally or containerized; executes recipes and tools (MCP); exposes ACP-compatible endpoint.
  - Enforces extension allowlists and permission prompts; produces structured logs and traces.
- Privacy Guard (new wrapper + local runtime)
  - Pre/post-processing around provider calls: PII/secret detection, deterministic masking, re-identification on return.
  - Modes: off, detect-only, mask-and-forward, strict block for disallowed classes (e.g., credentials).
- Org Directory & Policy Service (new; "Directory/Policy")
  - Stores org chart (DirectoryNode graph), role profiles, policies, and tool/extension allowlists.
  - Issues signed profile bundles to agents; evaluates RBAC/ABAC for routing and approvals.
- Task Router & Skills Graph (new; "Router")
  - Maintains skills/tags derived from extensions/recipes; routes tasks to suitable agents by role/skill/load/locale.
  - Supports fan-out and simple load balancing; emits routing/audit events.
- Cross-Agent Session Broker (new; "Session Broker")
  - Creates multi-agent sessions; manages scoped context shards and redaction boundaries; coordinates hand-offs and aggregates status.
- Audit & Observability Service (new; "Audit")
  - Centralizes OTEL traces, structured logs, and audit events; provides export APIs and retention policies.
- Agent Mesh Extension (new MCP extension)
  - Tools: send_task, request_approval, notify, fetch_status; uses Directory/Router for resolution and policy checks.
- Model Gateway / Provider Wrapper (reuse + extend)
  - Integrates cloud models and local models (Ollama) with policy-aware selection and cost downshift.
- Storage Layer (reuse + configure)
  - Postgres for metadata (sessions, policies, approvals); object storage for artifacts; encrypted local files for desktop.

## Responsibilities Matrix (per component)
- Goose Agent: tool execution; recipe runner; local state; ACP API; observability emit; enforce allowlists.
- Privacy Guard: PII detection; masking; token mapping; block/allow decisions; redact logs.
- Directory/Policy: org chart; profiles; RBAC/ABAC evaluation; secrets reference (not custody in MVP).
- Router: skills registry; task dispatch; cost/latency-aware routing; retries; simple queuing.
- Session Broker: multi-agent context sharding; hand-offs; approval workflow state; status aggregation.
- Audit: OTEL ingestion; audit store; query/export; retention jobs; tamper-evident hashing (hash chain) optional.
- Agent Mesh: inter-agent verbs; auth token propagation; request/response schemas.
- Model Gateway: model registry; selection policy; caching; token accounting.
- Storage: metadata DB; object store; encryption and TTLs.

## Interfaces & Contracts

### External APIs (high level; OpenAPI to be authored during implementation)
- Orchestrator Controller HTTP (v1)
  - POST /api/v1/tasks/route {task, context, policyHints}
  - POST /api/v1/sessions {participants, scope}
  - POST /api/v1/approvals {sessionId, stepId, approverRole, payload}
  - GET  /api/v1/status/{id}
  - GET  /api/v1/profiles/{role}
  - POST /api/v1/audit/ingest
- ACP-compatible Agent API (reuse Goose)
  - /v1/chat, /v1/tools, /v1/sessions (per Goose/ACP).

### MCP Tools (Agent Mesh + Guard)
- send_task(targetRole|agentId, task, context, policyHints) → taskId
- request_approval(targetRole|agentId, sessionId, stepId, payload) → approvalId
- notify(targetRole|agentId, message, severity?) → ack
- fetch_status(taskId|sessionId) → status
- privacy_guard.detect(text) → entities
- privacy_guard.process(text, policy) → {masked, mapRef}

### Internal Service APIs
- Directory/Policy
  - GET /directory/{nodeId}; GET /roles/{role}/profile; POST /evaluate {subject, action, resource}
- Router
  - POST /route {task, skills, constraints} → {agentTargets}
- Session Broker
  - POST /sessions; PATCH /sessions/{id}/handoff; GET /sessions/{id}
- Audit
  - POST /events; GET /events?filters; GET /export (ndjson)

### Events & Schemas
- AuditEvent: {id, ts, tenantId, actor{type,id,role}, action, target, result, redactions[], cost{tokens,$}, traceId, hashPrev}
- ApprovalEvent: {id, ts, sessionId, stepId, approver, decision, notes}
- RouteEvent: {id, ts, taskId, fromAgent, toAgent[], policyEval, reason}
- CrossAgentMessage: {id, ts, from, to, type(task|note|approval), payload, auth{jwt}, traceId}

## Data Model & Storage

### Core Entities
- Agent, Profile, Policy, Session, Task, Approval, AuditEvent, DirectoryNode, Tool, Extension, Recipe, Memory, Hint, Ignore

### Storage Choices
- Postgres (metadata): profiles, policies, directory graph (with ltree/adjacency), sessions, tasks, approvals, audit event index.
- Object store (artifacts/transcripts): S3-compatible buckets with server-side encryption and lifecycle TTL.
- Local agent state: encrypted files (config, sessions.jsonl, logs) per device.
- Indexing: btree indexes on tenantId, role, status, created_at; GIN/JSONB for event payloads; optional graph index for directory.
- Retention/TTL: audit log default 90 days (configurable); session transcripts 30–90 days; artifacts 30 days unless pinned; redaction maps kept only as long as necessary for re-identification.
- Redaction: store only masked text in centralized stores when possible; keep token maps encrypted with per-tenant keys and access-scoped.

## Tech Stack Options & Trade-offs (for ADRs)
- Languages/runtimes
  - Rust or Go for services (performance, safety); Node/TypeScript for Desktop/Electron integrations; Python acceptable for guard/rules.
- Datastores
  - Postgres for metadata; S3-compatible object storage; SQLite acceptable for pure desktop.
- Message bus
  - None in MVP; NATS vs Kafka later (NATS for lightweight routing, Kafka for durable analytics).
- Secrets
  - MVP: env/files or simple secret store; later: HashiCorp Vault or cloud KMS; BYOK per tenant.
- Tracing/Logs/Metrics
  - OpenTelemetry SDKs; exporters to file/stdout in MVP; later Tempo/Jaeger/Prometheus/Grafana/Loki.
- Deployment
  - Docker containers; optional Kubernetes; Helm for P1; CI/CD via GitHub Actions.
- Security
  - mTLS between services (post-MVP), JWT/OIDC for auth; mTLS + JWT in enterprise; CSP and sandboxing for Desktop.
- Models
  - Cloud: major providers via aggregator (OpenRouter/one-API). Local: Ollama with Llama/Qwen small models for guard tasks. Planner optional.

## Deployment Topology & Security Zones
- Desktop Zone (User Workstation)
  - Goose Agent + Privacy Guard local; encrypted local state; keys scoped to device or tenant.
- Department Node (Optional)
  - Containerized agent(s) for team workflows; may host tiny controller; restricted network egress; ephemeral credentials.
- Org Controller (Single Tenant)
  - Directory/Policy, Router, Session Broker, Audit; behind LB; network ACLs; secret store. Identity boundary with JWT (MVP), OIDC later.
- Key custody
  - MVP: app-managed per-tenant secret; enterprise: BYOK in KMS/Vault; guard mapping keys never leave tenant boundary.

## Observability & Audit Baseline
- Metrics: request rate, latency (P50/P95), error rates, token usage, cache hit rate, routing decisions, approval times.
- Logs: structured JSON with tenantId, actor, action, result, redactions, traceId/spanId.
- Traces: OTEL spans across agent → guard → model → tools → orchestrator; sampling 5–10% in MVP.
- Audit event schema: see Events; export formats ndjson/CSV; signed hash chain optional to detect tampering.

## Risks, Mitigations, Open Questions

### Key Risks
- Privacy Guard accuracy (false positives/negatives) reduces utility or safety.
- Integration sprawl across MCP tools increases maintenance and attack surface.
- Latency and cost inflation from multi-agent + guard orchestration.
- Security of inter-agent communication and policy enforcement gaps.
- Adoption/change management across departments.

### Mitigations
- Start with conservative rules + regex + small local LLM; add allow/deny lists; continuous evaluation.
- Maintain curated, versioned tool allowlists per role; sandbox high-risk tools; staged rollout.
- Cost-aware routing, caching, batch where possible; measure token cost KPIs; tune models.
- JWT with short TTL; signed profile bundles; explicit policy checks on every route and approval; mTLS later.
- Provide prebuilt role profiles/recipes and clear guides; phased rollout.

### Open Questions
- Multi-tenant SaaS vs single-tenant per org for scale-out? (Assume single-tenant MVP.)
- Preferred SSO (Okta, Azure AD, Auth0) and SCIM needs? Timeline to enable?
- Exact data residency and retention policies per tenant?
- Allowed/blocked model vendors; on-prem GPU for local models?
- Approval SLAs, retries, and escalation paths (manager notifications)?
- Which departments and MCP integrations are day-1 must-haves beyond Finance/Marketing/Eng?

## Acceptance Criteria (v1.2 Technical Definition)
- Architecture
  - Components and responsibilities implemented or scaffolded as defined (Agent, Privacy Guard, Directory/Policy, Router, Session Broker, Audit).
- Interfaces
  - Controller HTTP endpoints stubbed with OpenAPI; Agent Mesh MCP tools available; ACP compatibility verified.
- Security & Privacy
  - Privacy Guard enforces mask-and-forward with deterministic pseudonymization; per-tenant key in app-managed store; JWT with ≤ 30m TTL; extension allowlists enforced.
- NFRs (measurable)
  - Latency targets met in a reference scenario (interactive P50 ≤ 5s, P95 ≤ 15s) on commodity hardware; orchestrator availability target documented at 99.5%.
- Observability & Audit
  - Structured logs and basic metrics emitted; OTEL traces captured locally; audit events persisted/exportable as ndjson.
- Cost Controls
  - Downshift policy available; per-tenant budget thresholds configurable; token usage recorded per task.
- Deliverables
  - This technical-requirements.md; seed profiles for Finance and Marketing; example cross-agent approval flow; quick-start scripts and docs.
