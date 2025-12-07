# MVP (v1.2) — Definition

Purpose: a minimal, demonstrable slice that proves org-aware orchestration with privacy-by-design and governance.

## Goals
- Orchestrate tasks across role-based agent instances using a minimal Orchestrator.
- Enforce privacy guard at agent pre/post boundary (local-first), with optional provider middleware as defense-in-depth.
- Provide baseline audit/observability for agent activity and approvals.
- Keep deployment simple for pilot (desktop + small org controller).

## In Scope
- Minimal Orchestrator components:
  - Directory & Policy (basic): roles, profiles, allowlists
  - Task Router (capability-/permission-aware, simple rules)
  - Session Broker (basic handoff context, scoped)
  - Audit baseline (events timeline; structured logs; OTel-ready optional)
- Privacy Guard: agent pre/post (local-first), optional provider middleware as defense-in-depth
- Identity: OIDC (SSO) at Orchestrator, JWT for agents; mTLS post-MVP
- Messaging: HTTP-only orchestration; no bus in MVP
- Storage: Postgres (metadata), object storage optional (artifacts) with local-first defaults

## Out of Scope
- Advanced policy composition/graph evaluation
- Marketplace/profile packs
- Complex schedulers; temporal integrations beyond minimal
- Rich analytics dashboards; advanced approval chains

## Success Criteria
- End-to-end demo of task routed across ≥2 agents with policy enforcement
- Measurable privacy guard behavior (masking before egress)
- Audit trail and traces for the demo run
- MVP deployable on a small cluster or local desktop+controller setup

## Risks & Mitigations
- Guard accuracy → test rules + fallback; add manual overrides
- Latency from guard + routing → cache/prompt trimming; local guard
- Identity/certs complexity → dev mode with local IdP and self-signed certs

## Dependencies

Note: For the MVP implementation choices that keep ops low and align with goose v1.12.00, see “Appendix A — MVP implementation choices (PO alignment)” in requirements.md.
- productdescription.md (value pillars)
- requirements.md (PO output)
- plan.mmd (Planner output)
- technical-requirements.md (targets and interfaces)
- ADRs 0001–0005 (initial decisions)

## Milestones (indicative)
- M0: Prototype path (agent ↔ guard ↔ worker model)
- M1: Router + Session Broker baseline (HTTP-only)
- M2: Identity (OIDC SSO) + Secrets (Vault+KMS) + Audit baseline
- M3: Demo scenario + packaging + docs

## References
- ../architecture/index.html
- ../../productdescription.md
- ../../technical-requirements.md
- ../adr/
