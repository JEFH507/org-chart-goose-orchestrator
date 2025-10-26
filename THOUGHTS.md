# Thoughts & Context (scratchpad)

This is a non-binding scratchpad capturing the conversation and design intentions so far. Use it for reference; it’s not an official TODO.

## Vision
- Org-chart orchestrated AI framework: each role/team gets a “digital twin” assistant built on Goose.
- Hierarchical orchestration mirrors company org structure; cross-agent workflows and approvals.
- Privacy by design with local LLM guard (mask PII before cloud calls; re-identify on return when authorized).
- Open, vendor-neutral integrations via MCP and ACP; land-and-expand deployment (desktop → department → org-wide).

## Customer-first design
- Problems solved: fragmented AI usage, lack of role relevance, privacy risk, tool sprawl, missing org memory, limited observability.
- Value: role-specific twins, orchestration, standardization via profiles/recipes, unified governance, auditability.
- Differentiators: org-aware hierarchy, data minimization pipeline, open ecosystem, flexible deployment.

## Proposed architecture (product view)
- Orchestrator: Directory & Policy, Task Router & Skills Graph, Cross-Agent Session Broker, Audit & Observability.
- Agents (per role/user/dept): UI/CLI/API, Goose-based agent, Privacy Guard, MCP extensions, memory.
- Models: local guard/planner; cloud worker; policy-driven routing.
- Data: MCP servers for SaaS/DB; local FS.

## Deployment modes
- Individual desktop; department endpoint; organization-wide; hybrid.

## Open-core business model (idea)
- Open: core agent runtime, extension SDKs, role profile templates, reference guard rules.
- Enterprise: orchestrator services (directory, router, broker), SSO/SCIM, central audit, compliance packs, premium extensions.

## Tech thoughts (later phases)
- Agent Mesh extension for agent-to-agent communication.
- Provider wrapper to insert Privacy Guard pre/post around model calls.
- Profile bundles for roles/departments (extensions, policies, recipes, prompts, env vars).
- Cross-Agent Session Broker API and context segmentation rules.
- Observability with OTEL across agents and orchestrator services.

## MVP sketch (to be refined later)
- Single department pilot: departmental endpoint + 3–5 IC desktop agents.
- Core MCP integrations for the department.
- Basic privacy guard (regex + rules + small local LLM for ambiguous cases).
- Simple cross-agent handoff (HTTP/gRPC) and audit log.

## Reference
- Upstream Goose: https://github.com/block/goose
- Our Goose reference snapshots live in: ./goose-versions-references/

## Misc
- Use .goose/.goosehints to keep session context local to this project.
- Avoid committing secrets; follow .env.example; keep .gooseignore conservative.
