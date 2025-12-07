# ADR 0008: Audit Schema and Redaction Maps

Status: Accepted (MVP)
Date: 2025-10-27

## Context
We need consistent auditability across agents and services without leaking PII/secret data. goose provides observability hooks but no turnkey audit schema or redaction map service.

## Decision
- Adopt a common AuditEvent schema: {id, ts, tenantId, actor{type,id,role}, action, target, result, redactions[], cost{tokens,$}, traceId, hashPrev}.
- Only masked data stored; include redaction counts/summaries. Ingest rejects events containing raw PII via validation.
- Provide ndjson export; keep retention defaults ~90 days.

## Consequences
- Safer audits with limited forensic detail; re-identification handled locally under authorization.

## Alternatives
- Store PII encrypted centrally with access controls (defer to post-MVP).
