# ADR 0012: Minimal Server-Side Metadata Model

Status: Accepted (MVP)
Date: 2025-10-27

## Context
The project enforces a metadata-only server posture to reduce data custody risk.

## Decision
- Store only metadata for sessions, tasks, approvals, and an audit index. Do not persist raw content server-side. Implement TTL-based retention jobs (default 90 days for audit index).

## Consequences
- Strong privacy posture; reduces central debugging capabilities.

## Alternatives
- Store transcripts encrypted; strict access governance (post-MVP consideration).
