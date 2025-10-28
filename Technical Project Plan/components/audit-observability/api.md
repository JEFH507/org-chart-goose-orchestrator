# API

## AuditEvent
- {id, ts, tenantId, actor:{type,id,role}, action, target, result, redactions:[{type,count}], cost:{tokens,usd}, traceId, hashPrev}

## Endpoints
- POST /api/v1/audit/ingest [{AuditEvent}]
- GET /api/v1/audit/export → ndjson stream
