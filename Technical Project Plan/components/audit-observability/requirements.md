# Requirements

## Functional
- POST /audit/ingest (bulk ok)
- GET /audit/export?from..to..filters

## Non-functional
- Privacy: never store raw PII; store masked + redaction metadata.
- Performance: p95 ingest ≤ 100ms per event in batch.
