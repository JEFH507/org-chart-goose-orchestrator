# ADR 0005: Data Retention and Storage (Stateless Orchestrator; minimal data)

- Status: Accepted (MVP)
- Date: 2025-10-26
- Authors: @owner
- Decision Drivers:
  - Business: Minimize data custody; avoid vendor lock-in
  - Technical: Keep server stateless for content; desktop-local engine data
  - Compliance/Security: Data minimization; right-to-erasure
  - Cost/Latency: Reduce storage footprint; keep ops light
- Assumptions
  - Single-tenant per org; desktop holds content by default

## Context
We aim to store as little data as possible on the server. LLM providers maintain their own policy relationship with the client; we avoid holding raw content centrally.

## Decision
- Stateless orchestrator for content: server stores only minimal metadata; desktop holds prompts, responses, and artifacts by default.
- Server-side stored (metadata only): identity/session issuance, policies/profiles/recipes (small text), approvals/workflow state (IDs/status/timestamps), audit metadata (event type, hashes/pointers), usage counters (no content).
- Not stored server-side: raw prompts/responses, files, PII/transcripts.
- Data residency: configurable; export/import tools for portability.

## Technical details
- DB: Postgres-compatible; metadata only. Object store optional; if used, S3-compatible (MinIO for CE).
- TTLs: sessions metadata 30–90d default; audit 3–12m configurable; artifacts local-only by default.
- Audit: structured, redacted logs; content-addressed references for integrity without content.

## Security & privacy impact
- Reduced custody and breach surface; supports erasure by deleting local artifacts and server pointers

## Operational impact
- Lightweight server ops; relies on desktop-local storage

## Consequences
- Benefits: Strong privacy posture; easy CE self-hosting; portability
- Risks/Trade-offs: Reduced server-side debuggability; limited cross-device continuity unless users sync local stores

## Decision lifecycle
- Revisit after pilots to tune TTLs and portability tooling

## References
- ../../productdescription.md
- ../../requirements.md
- ../architecture/mvp.md
