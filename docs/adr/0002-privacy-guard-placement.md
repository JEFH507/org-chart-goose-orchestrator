# ADR 0002: Privacy Guard Placement (Agent pre/post primary; provider middleware optional)

- Status: Accepted (MVP)
- Date: 2025-10-26
- Authors: @owner
- Decision Drivers:
  - Business: Privacy-by-design promise
  - Technical: Minimize cloud exposure and latency; enforce guard consistently
  - Compliance/Security: Deterministic masking, key custody, defense-in-depth
  - Cost/Latency: Local preprocessing preferred
- Assumptions
  - Guard must run before any egress to cloud or external tools

## Context
Privacy Guard can be implemented as (A) agent pre/post extension, or (B) provider middleware (pre/post around model calls). We prefer a local-first model with open-source components.

## Decision
- Primary: Agent-level pre-filter/post-filter extension is required in all flows (local-first, open models via Ollama + rules).
- Secondary (optional): Provider middleware wrapper as a safety belt to enforce masking at the last mile before model calls.

## MVP implementation details
- Policies as files (YAML/JSON), versioned in Git; rule packs open and auditable
- Local guard: lightweight model (e.g., Llama/Qwen via Ollama) + regex/rules; re-identification only for allowed fields
- Provider wrapper: disabled by default in CE; enabled by default in SaaS for defense-in-depth; same open policy files

## Security & privacy impact
- Mask before any egress; deterministic pseudonymization keyed per tenant
- Keys via Vault/KMS (see ADR 0003); no PII in URLs; logs are redacted

## Operational impact
- Consistent enforcement with minimal central ops; per-department overrides via policy files

## Consequences
- Benefits: Strong privacy posture; consistent masking; flexible deployment (CE/SaaS)
- Risks/Trade-offs: Slight duplication of checks; manageable via shared policies

## Decision lifecycle
- Revisit post-MVP after compliance review and red-team tests

## References
- ../../productdescription.md
- ../../requirements.md
- ../architecture/mvp.md
