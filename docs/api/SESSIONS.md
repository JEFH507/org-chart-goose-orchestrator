# Session API Documentation

**Version:** v0.4.0  
**Base URL:** `http://localhost:8088`  
**Authentication:** JWT Bearer Token (Keycloak OIDC)

---

## Overview

The Session API provides endpoints for managing agent sessions in the goose-org-twin orchestrator. Sessions track the lifecycle of agent interactions, including task assignments, state transitions, and metadata storage.

### Key Features

- ✅ **Database-backed persistence** (Postgres with sqlx)
- ✅ **Session lifecycle management** (5 states: pending, active, completed, failed, expired)
- ✅ **JWT authentication** (all endpoints protected)
- ✅ **Pagination support** (list endpoint)
- ✅ **Idempotency** (via Idempotency-Key header)
- ✅ **Health monitoring** (GET /health endpoint)

---

## Session Model

### Session Object

```json
{
  "session_id": "uuid",
  "agent_role": "string",
  "task_id": "uuid | null",
  "state": "pending | active | completed | failed | expired",
  "created_at": "timestamp",
  "updated_at": "timestamp",
  "metadata": {}
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `session_id` | UUID | Unique session identifier (auto-generated) |
| `agent_role` | String | Agent role (e.g., "finance", "manager", "engineering") |
| `task_id` | UUID (nullable) | Associated task identifier (for cross-agent workflows) |
| `state` | String | Session lifecycle state (see State Machine below) |
| `created_at` | Timestamp | Session creation time (UTC) |
| `updated_at` | Timestamp | Last update time (UTC) |
| `metadata` | JSON | Flexible key-value storage for session context |

### State Machine

```
pending ──┬──> active ──┬──> completed
          │             ├──> failed
          │             └──> expired
          └──> expired

Terminal states: completed, failed, expired (cannot transition)
```

**Valid Transitions:**
- ✅ `pending → active` (session starts)
- ✅ `pending → expired` (timeout before start)
- ✅ `active → completed` (session finishes successfully)
- ✅ `active → failed` (session encounters error)
- ✅ `active → expired` (timeout during execution)
- ❌ `completed → *` (terminal state)
- ❌ `failed → *` (terminal state)
- ❌ `expired → *` (terminal state)

---

## Endpoints

### 1. Create Session

**POST** `/sessions`

Creates a new session with `pending` state.

#### Headers

```
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json
Idempotency-Key: <unique_key> (optional, recommended)
```

#### Request Body

```json
{
  "agent_role": "string",
  "task_id": "uuid | null",
  "metadata": {}
}
```

**Field Requirements:**
- `agent_role`: **Required** - Agent role identifier
- `task_id`: Optional - Associated task UUID
- `metadata`: Optional - JSON object (default: `{}`)

#### Response

**Status:** `201 Created`

```json
{
  "session_id": "9f654837-2d15-4ed3-8aae-c97eb9b68a7b",
  "status": "pending"
}
```

#### Error Responses

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid request body (missing `agent_role`, invalid JSON) |
| `401 Unauthorized` | Missing or invalid JWT token |
| `503 Service Unavailable` | Database not configured or unavailable |

#### Example

```bash
curl -X POST http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: create-finance-session-123" \
  -d '{
    "agent_role": "finance",
    "metadata": {
      "workflow": "budget_approval",
      "user_id": "user-456"
    }
  }'
```

---

### 2. Get Session by ID

**GET** `/sessions/{id}`

Retrieves a specific session by UUID.

#### Headers

```
Authorization: Bearer <JWT_TOKEN>
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Session identifier |

#### Response

**Status:** `200 OK`

```json
{
  "session_id": "9f654837-2d15-4ed3-8aae-c97eb9b68a7b",
  "agent_role": "finance",
  "state": "active",
  "task_id": "task-uuid-123",
  "created_at": "2025-11-05T07:30:00Z",
  "updated_at": "2025-11-05T08:00:00Z",
  "metadata": {
    "workflow": "budget_approval",
    "user_id": "user-456"
  }
}
```

#### Error Responses

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid UUID format |
| `401 Unauthorized` | Missing or invalid JWT token |
| `404 Not Found` | Session does not exist |
| `503 Service Unavailable` | Database not configured or unavailable |

#### Example

```bash
curl http://localhost:8088/sessions/9f654837-2d15-4ed3-8aae-c97eb9b68a7b \
  -H "Authorization: Bearer $JWT_TOKEN"
```

---

### 3. Update Session

**PUT** `/sessions/{id}`

Updates an existing session's state and/or metadata.

#### Headers

```
Authorization: Bearer <JWT_TOKEN>
Content-Type: application/json
Idempotency-Key: <unique_key> (optional, recommended)
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | UUID | Session identifier |

#### Request Body

```json
{
  "status": "active | completed | failed | expired",
  "task_id": "uuid | null",
  "metadata": {}
}
```

**Field Requirements:**
- `status`: Optional - New session state (must be valid transition)
- `task_id`: Optional - Update associated task
- `metadata`: Optional - Merge with existing metadata

**Note:** All fields are optional. Omitted fields retain their current values.

#### Response

**Status:** `200 OK`

```json
{
  "session_id": "9f654837-2d15-4ed3-8aae-c97eb9b68a7b",
  "agent_role": "finance",
  "state": "active",
  "task_id": "task-uuid-123",
  "created_at": "2025-11-05T07:30:00Z",
  "updated_at": "2025-11-05T08:00:00Z",
  "metadata": {
    "workflow": "budget_approval",
    "user_id": "user-456",
    "progress": "50%"
  }
}
```

#### Error Responses

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid status value or invalid transition |
| `401 Unauthorized` | Missing or invalid JWT token |
| `404 Not Found` | Session does not exist |
| `422 Unprocessable Entity` | Invalid state transition (e.g., `completed → active`) |
| `503 Service Unavailable` | Database not configured or unavailable |

#### Example

```bash
curl -X PUT http://localhost:8088/sessions/9f654837-2d15-4ed3-8aae-c97eb9b68a7b \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: update-session-to-active-123" \
  -d '{
    "status": "active",
    "metadata": {
      "progress": "50%",
      "started_at": "2025-11-05T08:00:00Z"
    }
  }'
```

---

### 4. List Sessions

**GET** `/sessions`

Lists sessions with pagination support.

#### Headers

```
Authorization: Bearer <JWT_TOKEN>
```

#### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | Integer | `1` | Page number (1-indexed) |
| `page_size` | Integer | `20` | Items per page (max: 100) |
| `agent_role` | String | (all) | Filter by agent role (optional) |

#### Response

**Status:** `200 OK`

```json
{
  "sessions": [
    {
      "session_id": "9f654837-2d15-4ed3-8aae-c97eb9b68a7b",
      "agent_role": "finance",
      "state": "active",
      "task_id": "task-uuid-123",
      "created_at": "2025-11-05T07:30:00Z",
      "updated_at": "2025-11-05T08:00:00Z",
      "metadata": {}
    },
    {
      "session_id": "8e543726-1c04-3bd2-7a9d-b86da8b57a6c",
      "agent_role": "manager",
      "state": "completed",
      "task_id": null,
      "created_at": "2025-11-04T14:00:00Z",
      "updated_at": "2025-11-04T16:30:00Z",
      "metadata": {}
    }
  ],
  "total": 42,
  "page": 1,
  "page_size": 20
}
```

#### Error Responses

| Status | Description |
|--------|-------------|
| `400 Bad Request` | Invalid pagination parameters (e.g., page < 1) |
| `401 Unauthorized` | Missing or invalid JWT token |
| `503 Service Unavailable` | Database not configured or unavailable |

#### Example

```bash
# Get first page (default)
curl http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT_TOKEN"

# Get page 2 with 50 items per page
curl "http://localhost:8088/sessions?page=2&page_size=50" \
  -H "Authorization: Bearer $JWT_TOKEN"

# Filter by agent role
curl "http://localhost:8088/sessions?agent_role=finance" \
  -H "Authorization: Bearer $JWT_TOKEN"
```

---

### 5. Health Check

**GET** `/health`

Returns health status of the controller and its dependencies.

#### Headers

None required (unauthenticated endpoint).

#### Response

**Status:** `200 OK`

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "database": "connected",
  "redis": "connected"
}
```

**Field Descriptions:**
- `status`: Overall health (`"healthy"` or `"degraded"`)
- `version`: Controller version
- `database`: Postgres connection status (`"connected"` or `"disconnected"`)
- `redis`: Redis connection status (`"connected"` or `"disconnected"`)

#### Degraded Status

If database or Redis are unavailable:

```json
{
  "status": "degraded",
  "version": "0.1.0",
  "database": "disconnected",
  "redis": "connected"
}
```

**Note:** The controller continues to operate in degraded mode, but persistence operations (POST/GET/PUT /sessions) will return `503 Service Unavailable`.

#### Example

```bash
curl http://localhost:8088/health
```

---

## Idempotency

All mutation endpoints (POST, PUT) support idempotency via the `Idempotency-Key` header.

### How It Works

1. **First request:** Process request, cache response in Redis with 24-hour TTL
2. **Duplicate request (same key):** Return cached response immediately (no database write)
3. **Expired key (>24h):** Treat as new request

### Configuration

```bash
# Enable idempotency (default: false)
IDEMPOTENCY_ENABLED=true

# TTL in seconds (default: 86400 = 24 hours)
IDEMPOTENCY_TTL_SECONDS=86400
```

### Example

```bash
# First request
curl -X POST http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Idempotency-Key: unique-key-123" \
  -H "Content-Type: application/json" \
  -d '{"agent_role": "finance"}'

# Response: 201 Created
# {
#   "session_id": "abc-123",
#   "status": "pending"
# }

# Duplicate request (same key)
curl -X POST http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Idempotency-Key: unique-key-123" \
  -H "Content-Type: application/json" \
  -d '{"agent_role": "finance"}'

# Response: 200 OK (cached)
# {
#   "session_id": "abc-123",
#   "status": "pending"
# }
# Same session ID returned!
```

---

## Authentication

All endpoints (except `/health`) require JWT authentication via Keycloak OIDC.

### Getting a JWT Token

```bash
# Client credentials grant (service accounts)
curl -X POST "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=YOUR_CLIENT_SECRET" | jq -r '.access_token'
```

### Using the Token

```bash
curl http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT_TOKEN"
```

### Token Requirements

- **Issuer:** `http://localhost:8080/realms/dev`
- **Audience:** `goose-controller`
- **Algorithm:** RS256
- **Expiration:** Tokens expire after 5 minutes (Keycloak default)

---

## Error Handling

### Error Response Format

```json
{
  "error": "Error message",
  "details": "Optional detailed error information"
}
```

### Common Error Codes

| Status | Reason | Solution |
|--------|--------|----------|
| `400 Bad Request` | Invalid input (missing fields, invalid JSON, invalid UUID) | Check request body/parameters |
| `401 Unauthorized` | Missing or invalid JWT token | Obtain valid token from Keycloak |
| `404 Not Found` | Session does not exist | Verify session ID |
| `422 Unprocessable Entity` | Invalid state transition | Check session state machine |
| `503 Service Unavailable` | Database or Redis unavailable | Check service health via `/health` endpoint |

---

## Database Schema

### sessions Table

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    role VARCHAR(50) NOT NULL,
    task_id UUID,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_sessions_task_id ON sessions(task_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);
```

**See:** `docs/database/SCHEMA.md` for full schema documentation.

---

## Configuration

### Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `DATABASE_URL` | String | (required) | Postgres connection string |
| `REDIS_URL` | String | (optional) | Redis connection string |
| `IDEMPOTENCY_ENABLED` | Boolean | `false` | Enable idempotency middleware |
| `IDEMPOTENCY_TTL_SECONDS` | Integer | `86400` | Idempotency cache TTL (24 hours) |
| `SESSION_RETENTION_DAYS` | Integer | `7` | Session auto-expiration period |
| `OIDC_ISSUER_URL` | String | (required) | Keycloak issuer URL |
| `OIDC_AUDIENCE` | String | `goose-controller` | JWT audience claim |

### Example .env

```bash
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/orchestrator
REDIS_URL=redis://redis:6379
IDEMPOTENCY_ENABLED=true
IDEMPOTENCY_TTL_SECONDS=86400
SESSION_RETENTION_DAYS=7
OIDC_ISSUER_URL=http://localhost:8080/realms/dev
OIDC_AUDIENCE=goose-controller
```

---

## Rate Limiting

**Not implemented in Phase 4.** All endpoints are unlimited.

**Recommendation for production:**
- Add rate limiting middleware (e.g., governor crate)
- Suggested limits: 100 requests/minute per client

---

## Versioning

**Current Version:** v0.4.0

**API Versioning Strategy:** URL-based (future)
- v0.4.0: No version prefix (backwards compatible)
- v1.0.0+: `/v1/sessions` (breaking changes)

---

## Examples

### Complete Workflow Example

```bash
# 1. Get JWT token
JWT_TOKEN=$(curl -s -X POST "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=YOUR_SECRET" | jq -r '.access_token')

# 2. Create session
SESSION_ID=$(curl -s -X POST http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: workflow-session-$(date +%s)" \
  -d '{
    "agent_role": "finance",
    "metadata": {
      "workflow": "budget_approval",
      "amount": 50000
    }
  }' | jq -r '.session_id')

echo "Session created: $SESSION_ID"

# 3. Retrieve session
curl -s http://localhost:8088/sessions/$SESSION_ID \
  -H "Authorization: Bearer $JWT_TOKEN" | jq .

# 4. Update to active
curl -s -X PUT http://localhost:8088/sessions/$SESSION_ID \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "active",
    "metadata": {
      "workflow": "budget_approval",
      "amount": 50000,
      "started_at": "'$(date -Iseconds)'"
    }
  }' | jq .

# 5. Complete session
curl -s -X PUT http://localhost:8088/sessions/$SESSION_ID \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "completed",
    "metadata": {
      "workflow": "budget_approval",
      "amount": 50000,
      "completed_at": "'$(date -Iseconds)'",
      "approved": true
    }
  }' | jq .

# 6. List all sessions
curl -s http://localhost:8088/sessions?page=1&page_size=10 \
  -H "Authorization: Bearer $JWT_TOKEN" | jq .
```

---

## Performance Considerations

### Database

- **Connection pooling:** Max 5 connections per controller instance
- **Index usage:** All queries use indexed columns (session_id, status, created_at)
- **JSONB metadata:** Flexible but not indexed (avoid large queries on metadata)

### Redis (Idempotency)

- **Memory usage:** ~1 KB per cached response
- **Eviction policy:** allkeys-lru (oldest entries evicted when memory full)
- **Max memory:** 256 MB (configurable)

### Recommendations

- **Pagination:** Use `page_size` ≤ 100 for list operations
- **Idempotency:** Always use `Idempotency-Key` for mutation operations
- **Monitoring:** Track `/health` endpoint for database/Redis connectivity

---

## Changelog

### v0.4.0 (2025-11-05)

- ✅ Initial release (Phase 4)
- ✅ Session CRUD endpoints (POST/GET/PUT/LIST)
- ✅ Database persistence (Postgres + sqlx)
- ✅ Idempotency middleware (Redis-backed)
- ✅ Health endpoint
- ✅ JWT authentication (Keycloak OIDC)
- ✅ Session lifecycle management (5 states)
- ✅ Pagination support

---

## Support & Troubleshooting

### Common Issues

**1. "503 Service Unavailable" on /sessions endpoints**
- **Cause:** Database not configured or unavailable
- **Solution:** Check `DATABASE_URL` environment variable, verify Postgres running

**2. "401 Unauthorized" on all endpoints**
- **Cause:** Invalid or expired JWT token
- **Solution:** Obtain new token from Keycloak, verify token audience/issuer

**3. "Idempotency not working (different sessions returned)"**
- **Cause:** `IDEMPOTENCY_ENABLED=false` or Redis unavailable
- **Solution:** Set `IDEMPOTENCY_ENABLED=true`, verify Redis running

**4. "422 Unprocessable Entity" on PUT /sessions/{id}**
- **Cause:** Invalid state transition (e.g., `completed → active`)
- **Solution:** Check current session state, refer to state machine diagram

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Check health endpoint
curl http://localhost:8088/health | jq .
```

---

## License & Contact

**Project:** goose-org-twin  
**Repository:** https://github.com/JEFH507/org-chart-goose-orchestrator  
**Phase:** 4 (Storage/Metadata + Session Persistence)  
**Version:** v0.4.0  
**Author:** Javier (JEFH507)  
**Contact:** javsfeliu@gmail.com

---

**Last Updated:** 2025-11-05  
**Next:** Phase 5 (Directory/Policy + Profiles + Simple UI)
