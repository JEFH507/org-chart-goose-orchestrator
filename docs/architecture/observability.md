# Observability — Phase 1 (Controller)

Phase 1 documents the logging model, redaction posture, and OTLP stubs for the controller. The controller is HTTP-only and metadata-only per ADR-0010 and ADR-0008.

## Logging model (structured JSON)
- Transport: stdout (structured JSON via `tracing_subscriber`)
- Level: INFO default (configurable via `RUST_LOG`)
- Event fields (no PII):
  - ts: timestamp (emitted by runtime)
  - level: log level
  - component: "controller"
  - message: short message (e.g., "controller starting", "audit.ingest")
  - endpoint: e.g., "/status", "/audit/ingest"
  - method: e.g., GET, POST (if added later)
  - status_code: numeric HTTP status (for responses; optional)
  - version: controller version (on startup/status)
  - category: from AuditEvent (metadata-only)
  - action: from AuditEvent (metadata-only)
  - source: from AuditEvent (metadata-only)
  - subject: optional opaque identifier (MUST be pseudonymized or masked; avoid raw PII)
  - traceId: optional inbound trace identifier for correlation
  - has_metadata: boolean, true if metadata object present (no content logged)

Notes:
- Do not log request bodies. Avoid any content-bearing fields.
- Use explicit fields over free-text; prefer stable keys to ease parsing.

## Redaction posture
- No PII or content-bearing data is logged in Phase 1.
- `subject` must be an opaque id (pseudonymized when sourced from agents) and is optional.
- If adding new endpoints, ensure fields are metadata-only and add redaction unit tests.

## OTLP stubs (future integration)
- Library: OpenTelemetry Rust (`opentelemetry`, `opentelemetry-otlp`).
- Exporters: OTLP over gRPC/HTTP to an OTLP collector (not wired in Phase 1).
- Suggested attributes:
  - service.name = "goose-controller"
  - service.version = <controller version>
  - http.method, http.target, http.status_code
  - trace_id from inbound requests when provided
- Phase 2+: add span events for `/audit/ingest` with attributes: source, category, action, has_metadata.

## Example logs
```json
{"ts":"2025-11-01T00:00:00Z","level":"INFO","component":"controller","message":"controller starting","port":8088}
{"ts":"2025-11-01T00:00:02Z","level":"INFO","component":"controller","message":"audit.ingest","source":"test","category":"ci","action":"ingest","subject":null,"traceId":"abc-123","has_metadata":true}
```

## Acceptance (Phase 1)
- Logs contain only metadata fields as listed above and no PII.
- Redaction posture documented here and referenced from smoke tests.
