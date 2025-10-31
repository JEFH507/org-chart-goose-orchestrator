# 0017 — Controller Language and Runtime Choice (Rust)

Status: Accepted
Date: 2025-10-31

## Context
Phase 1 requires a minimal Controller runtime aligned with the OpenAPI stub (HTTP-only, metadata-only). We must choose a language/runtime that supports security, performance, and long-term maintainability, and aligns with Goose ecosystem practices.

## Decision
Use Rust for the Controller runtime baseline in Phase 1.

## Rationale
- Security/Robustness: Strong typing and memory safety reduce classes of runtime errors.
- Performance/Footprint: Predictable latency and small static binaries simplify CE deployments.
- Ecosystem: axum/actix-web, serde, tracing, OpenTelemetry crates, sqlx for metadata when needed.
- Alignment: Goose upstream is Rust-heavy; improves reuse and contributor familiarity.

## Consequences
- Slightly higher initial ramp vs Node/Python; mitigated with templates and docs.
- CI builds may be longer; mitigated with caching.
- Container images can be minimal (alpine/distroless) with a static binary.

## Alternatives Considered
- Node (TypeScript): Faster iteration, broader familiarity; higher runtime footprint and supply-chain diligence.
- Python (FastAPI): Rapid prototyping; lower throughput and deployment considerations.

## References
- docs/api/controller/openapi.yaml
- Technical Project Plan/PM Phases/Phase-1/Phase-1-Execution-Plan.md
