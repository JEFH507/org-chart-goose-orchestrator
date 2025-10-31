# Phase 1 Assumptions and Open Questions

## Assumptions
- Runtime language: Rust (ADR-0017) with HTTP-only orchestration per ADR-0010
- Default branch: main; feature branches per workstream
- Controller port: 8088 (configurable via CONTROLLER_PORT)
- Local dev DB: Postgres via compose; DATABASE_URL provided via local .env.ce
- Object storage provider: off by default (ADR-0014); profiles can enable SeaweedFS/MinIO/Garage later
- Logs are structured, metadata-only, with optional traceId propagation
- CI uses spectral for OpenAPI lint and a linkcheck for internal docs and $ref

## Open Questions
1. Which linkcheck tool is canonical in CI (lychee vs. markdown-link-check)?
2. Spectral ruleset: do we pin to a local .spectral.yaml or use recommended default?
3. DB migration runner: which toolchain for Rust integration (sqlx-cli vs. refinery vs. simple psql scripts)?
4. Compose service naming and network: standardize service names expected by controller (e.g., postgres) vs. env remapping.
5. Acceptance environment: do we require compose-only runs in CI, or allow host Postgres for speed?

## Risks
- CI flakiness from external link checks
- Version skew across compose profiles and host tools
- Over-logging risk; ensure PII redaction and metadata-only fields

## Resolution Plan
- Start with lychee for linkcheck; can swap if needed
- Add a pinned .spectral.yaml; iterate based on OpenAPI feedback
- Use sqlx-cli or plain SQL migrations with a simple runner script for Phase 1; evaluate later
- Compose service names: prefer postgres as service host; controller reads DATABASE_URL
- CI acceptance: run compose services headless; cache layers to speed up

## Tracking
- Progress and decisions appended to docs/tests/phase1-progress.md
- State in Technical Project Plan/PM Phases/Phase-1/Phase-1-Agent-State.json
