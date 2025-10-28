# ADR 0010: Controller OpenAPI and HTTP Interfaces

Status: Accepted (MVP)
Date: 2025-10-27

## Context
Agents and UI/automation clients need a stable, documented HTTP interface for routing, approvals, sessions, and audit.

## Decision
- Publish minimal OpenAPI covering /tasks/route, /sessions, /approvals, /status, /profiles (proxy), /audit/ingest.
- Require idempotency keys; enforce size limits; structured logs with traceId.

## Consequences
- Faster integration and client generation; consistent surface for Mesh.

## Alternatives
- GraphQL; gRPC-only; both deferred until post-MVP.
