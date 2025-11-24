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
- **Privacy Guard (Phase 2 ✅):** HTTP service with regex-based PII detection, deterministic pseudonymization, FPE for phone/SSN, session-scoped state
- Identity: OIDC (SSO) at Orchestrator, JWT for agents; mTLS post-MVP
- Messaging: HTTP-only orchestration; no bus in MVP
- Storage: Postgres (metadata), object storage optional (artifacts) with local-first defaults; no raw content on controller

## Components (As-Built)

### Privacy Guard Service (Phase 2 - Completed)
**Purpose:** Detect and mask PII before data leaves local environment

**Implementation:**
- Language: Rust (Axum HTTP framework)
- Port: 8089
- Deployment: Docker Compose profile `privacy-guard`
- State: In-memory session-scoped mappings (no persistence)

**Capabilities:**
- **Detection:** 8 entity types via regex (SSN, EMAIL, PHONE, CREDIT_CARD, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER)
- **Masking Strategies:**
  - Pseudonymization: HMAC-SHA256 deterministic mapping (EMAIL, PERSON, IP_ADDRESS, DOB, ACCOUNT)
  - FPE (Format-Preserving Encryption): Preserves format for PHONE and SSN
  - Redaction: Static masking with last-4 preservation for CREDIT_CARD
- **Modes:** OFF, DETECT (detection only), MASK (default), STRICT (fail on PII)
- **API:** `/guard/scan`, `/guard/mask`, `/guard/reidentify`, `/status`, `/internal/flush-session`
- **Performance:** P50=16ms, P95=22ms, P99=23ms (exceeds targets by 30-87x)

**Integration:**
- Controller integration via `GUARD_ENABLED` flag (optional)
- Agent-side wrapper (Phase 3+)
- Configuration: `rules.yaml` (patterns), `policy.yaml` (modes/strategies)

**References:**
- Implementation Guide: `docs/guides/privacy-guard-integration.md`
- Configuration Guide: `docs/guides/privacy-guard-config.md`
- ADR-0021: Rust Implementation
- ADR-0022: Detection Rules and FPE

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

Note: For the MVP implementation choices that keep ops low and align with Goose v1.12.00, see “Appendix A — MVP implementation choices (PO alignment)” in requirements.md.
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
- ../../product/productdescription.md
- ../../architecture/technical-requirements.md
- ../adr/
