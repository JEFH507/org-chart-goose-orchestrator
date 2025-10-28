# ADR 0001: MVP Messaging Approach (HTTP-only; no bus)

- Status: Accepted (MVP)
- Date: 2025-10-26
- Authors: @owner
- Decision Drivers:
  - Business: MVP timeline; minimize operational overhead
  - Technical: Keep orchestration simple and traceable
  - Compliance/Security: Reduce moving parts and egress surfaces
  - Cost/Latency: Minimal infra footprint; acceptable latency at MVP scale
- Assumptions
  - MVP scale is modest; synchronous and simple async patterns suffice

## Context
We need cross-agent handoffs and notifications. Options considered: HTTP-only orchestration, a lightweight bus (NATS), or a heavier broker (Kafka).

## Decision
For MVP, do not introduce a message bus. Use HTTP-only orchestration between agents/services (controller and desktop agents). Re-evaluate adding a lightweight bus (e.g., NATS) post-MVP if scale or decoupling needs emerge.

## Technical details
- Use HTTP endpoints for handoffs and callbacks (e.g., approval hooks)
- Idempotent operations; retries with backoff handled by caller
- Optional long-polling or simple webhook callbacks for progress
- Keep payloads small and pass references/IDs for larger artifacts

## Security & privacy impact
- TLS for all HTTP calls; short-lived JWT for authZ between agents/services
- Avoid PII in URLs; rely on masked payloads and IDs

## Operational impact
- No additional infra beyond the tiny controller (when used)
- Easier local/dev and desktop-only operation

## Consequences
- Benefits: Lowest operational complexity; easy to reason about
- Risks/Trade-offs: No async decoupling or fan-out; potential coupling between services
- Mitigations: Keep interfaces narrow; add NATS later if needed

## Alternatives considered
- NATS — Pros: low-latency pub/sub; Cons: adds infra/ops not needed for MVP
- Kafka — Pros: durable streams; Cons: heavy for MVP needs

## Decision lifecycle
- Revisit at M2/M3 milestone or upon hitting scale/latency limits to decide on introducing NATS (JetStream) or similar

## References
- ../../productdescription.md
- ../../requirements.md
- ../architecture/mvp.md
