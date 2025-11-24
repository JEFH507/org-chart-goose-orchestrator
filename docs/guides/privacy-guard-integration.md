# Privacy Guard Integration Guide

**Version:** 1.1  
**Last Updated:** 2025-11-04 (Phase 2.2 - Model-Enhanced Detection)  
**Status:** Production Ready

---

## Overview

This guide shows how to integrate with the Privacy Guard service for PII detection and masking. Privacy Guard provides HTTP endpoints for scanning text, masking PII with deterministic pseudonyms or format-preserving encryption, and reidentifying masked data.

**Key Features:**
- 8 entity types detected (SSN, CREDIT_CARD, EMAIL, PHONE, PERSON, IP_ADDRESS, DATE_OF_BIRTH, ACCOUNT_NUMBER)
- 3 masking strategies (PSEUDONYM, FPE, REDACT)
- 4 operational modes (OFF, DETECT, MASK, STRICT)
- Deterministic output (same input → same output)
- Session-based state management
- Structured audit logging (no PII in logs)

**Related Documentation:**
- Configuration: [`privacy-guard-config.md`](./privacy-guard-config.md)
- Architecture: [`docs/architecture/mvp.md`](docs/product/mvp.md)
- ADR-0021: [Privacy Guard Rust Implementation](../adr/0021-privacy-guard-rust-implementation.md)
- ADR-0022: [PII Detection Rules and FPE](../adr/0022-pii-detection-rules-and-fpe.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [API Endpoints](#api-endpoints)
   - [GET /status](#get-status)
   - [POST /guard/scan](#post-guardscan)
   - [POST /guard/mask](#post-guardmask)
   - [POST /guard/reidentify](#post-guardreidentify)
   - [POST /internal/flush-session](#post-internalflush-session)
3. [Integration Patterns](#integration-patterns)
   - [Controller Integration](#controller-integration)
   - [Agent-Side Integration (Future)](#agent-side-integration-future)
4. [Error Handling](#error-handling)
5. [Performance Considerations](#performance-considerations)
6. [Security Best Practices](#security-best-practices)
7. [Testing](#testing)
8. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Start the Service

```bash
# Start privacy-guard with Docker Compose
docker compose -f deploy/compose/ce.dev.yml --profile privacy-guard up -d

# Verify service is healthy
curl http://localhost:8089/status
```

**Expected Response:**
```json
{
  "status": "healthy",
  "mode": "Mask",
  "rule_count": 24,
  "config_loaded": true
}
```

### Basic Usage Example

```bash
# Scan text for PII (detection only)
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact John Doe at john.doe@example.com or 555-123-4567",
    "tenant_id": "org1"
  }'

# Mask text (full PII replacement)
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact John Doe at john.doe@example.com or 555-123-4567",
    "tenant_id": "org1"
  }'
```

---

## API Endpoints

### Base URL
- **Development:** `http://localhost:8089`
- **Production:** `http://privacy-guard:8089` (Docker Compose network)

### Authentication
- `/status`, `/guard/scan`, `/guard/mask`: No authentication required
- `/guard/reidentify`: JWT authentication required (Bearer token)
- `/internal/flush-session`: Internal only (not exposed externally)

---

### GET /status

**Purpose:** Healthcheck endpoint for monitoring and service discovery.

**Request:**
```bash
curl http://localhost:8089/status
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "mode": "Mask",
  "rule_count": 24,
  "config_loaded": true,
  "model_enabled": false,
  "model_name": "qwen3:0.6b"
}
```

**Response Fields:**
- `status`: Service health (`"healthy"` or `"degraded"`)
- `mode`: Current guard mode (`"Off"`, `"Detect"`, `"Mask"`, `"Strict"`)
- `rule_count`: Number of detection rules loaded
- `config_loaded`: Whether configuration files loaded successfully
- `model_enabled`: **(Phase 2.2+)** Whether model-enhanced detection is enabled (boolean)
- `model_name`: **(Phase 2.2+)** Configured NER model name (string, e.g., "qwen3:0.6b")

**Phase 2.2 Model Status:**
- `model_enabled: true` → Hybrid detection (regex + NER model)
- `model_enabled: false` → Regex-only detection (Phase 2 baseline)
- Model configuration: See [`privacy-guard-config.md`](./privacy-guard-config.md#model-enhanced-detection-phase-22)

**Use Cases:**
- Docker healthcheck
- Service readiness probes
- Configuration validation
- Model status monitoring (Phase 2.2+)

---

### POST /guard/scan

**Purpose:** Detect PII entities without masking (read-only operation).

**Request:**
```bash
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Email me at alice@example.com or call 555-867-5309",
    "tenant_id": "org1"
  }'
```

**Request Body:**
```json
{
  "text": "string (required)",
  "tenant_id": "string (required)"
}
```

**Response (200 OK):**
```json
{
  "detections": [
    {
      "start": 12,
      "end": 30,
      "entity_type": "EMAIL",
      "confidence": "HIGH",
      "matched_text": "alice@example.com"
    },
    {
      "start": 39,
      "end": 51,
      "entity_type": "PHONE",
      "confidence": "HIGH",
      "matched_text": "555-867-5309"
    }
  ]
}
```

**Response Fields:**
- `detections`: Array of detected PII entities
  - `start`: Character offset where entity begins (0-indexed)
  - `end`: Character offset where entity ends (exclusive)
  - `entity_type`: One of 8 types (SSN, EMAIL, PHONE, etc.)
  - `confidence`: Detection confidence (`"HIGH"`, `"MEDIUM"`, `"LOW"`)
  - `matched_text`: Original text that matched (for debugging only, not logged)

**Use Cases:**
- Validate text before submission
- Audit PII presence
- Tune detection rules (DETECT mode)
- Pre-flight check before STRICT mode

**Example: Detect Before Strict Mode**
```bash
# First, scan to see what would be detected
SCAN_RESULT=$(curl -s -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{"text": "My SSN is 123-45-6789", "tenant_id": "org1"}')

echo $SCAN_RESULT | jq '.detections | length'
# Output: 1 (one SSN detected)

# If detections exist, decide: mask or reject
```

---

### POST /guard/mask

**Purpose:** Detect and mask PII entities with pseudonyms or FPE.

**Request:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact Alice (SSN: 123-45-6789) at alice@example.com or 555-867-5309",
    "tenant_id": "org1",
    "session_id": "sess_abc123"
  }'
```

**Request Body:**
```json
{
  "text": "string (required)",
  "tenant_id": "string (required)",
  "session_id": "string (optional, auto-generated if omitted)"
}
```

**Response (200 OK):**
```json
{
  "masked_text": "Contact Alice (SSN: XXX-XX-6789) at EMAIL_80779724a9b108fc or 555-482-7193",
  "redactions": {
    "SSN": 1,
    "EMAIL": 1,
    "PHONE": 1
  },
  "session_id": "sess_abc123"
}
```

**Response Fields:**
- `masked_text`: Text with PII replaced by pseudonyms or FPE
- `redactions`: Count of each entity type masked
- `session_id`: Session identifier for reidentification

**Masking Strategies (by entity type):**
- **SSN:** FPE (preserves last 4 digits) → `XXX-XX-6789`
- **PHONE:** FPE (preserves area code) → `555-482-7193`
- **EMAIL:** PSEUDONYM → `EMAIL_80779724a9b108fc`
- **PERSON:** PSEUDONYM → `PERSON_a3f7b2c8e1d4f9a2`
- **CREDIT_CARD:** REDACT → `CARD_****_****_****_1234`
- **IP_ADDRESS:** PSEUDONYM → `IP_f9a2b3c8d1e4f7a3`
- **DATE_OF_BIRTH:** PSEUDONYM → `DOB_c8d1e4f7a3b2f9a2`
- **ACCOUNT_NUMBER:** PSEUDONYM → `ACCOUNT_d1e4f7a3b2f9a2c8`

**Determinism:**
Same input text produces the same masked output:

```bash
# Call 1
curl -X POST http://localhost:8089/guard/mask \
  -d '{"text": "alice@example.com", "tenant_id": "org1"}' | jq .masked_text
# Output: "EMAIL_80779724a9b108fc"

# Call 2 (same input)
curl -X POST http://localhost:8089/guard/mask \
  -d '{"text": "alice@example.com", "tenant_id": "org1"}' | jq .masked_text
# Output: "EMAIL_80779724a9b108fc" (same pseudonym)
```

**Use Cases:**
- Mask PII in audit logs
- Mask PII before external API calls
- Mask PII in agent messages
- Store masked data for compliance

**Example: Mask Before Logging**
```bash
# Original event
EVENT='{"user": "Alice", "email": "alice@example.com", "message": "Login successful"}'

# Mask PII in message field
MASKED=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d "{\"text\": \"$(echo $EVENT | jq -r .message)\", \"tenant_id\": \"org1\"}" | jq -r .masked_text)

# Create safe event
SAFE_EVENT=$(echo $EVENT | jq --arg masked "$MASKED" '.message = $masked')

# Log safe event (no PII)
echo $SAFE_EVENT
# Output: {"user":"Alice","email":"alice@example.com","message":"Login successful"}
```

---

### POST /guard/reidentify

**Purpose:** Reverse a pseudonym to original text (restricted operation).

**Request:**
```bash
curl -X POST http://localhost:8089/guard/reidentify \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <JWT_TOKEN>" \
  -d '{
    "pseudonym": "EMAIL_80779724a9b108fc",
    "session_id": "sess_abc123"
  }'
```

**Request Headers:**
- `Authorization`: `Bearer <JWT_TOKEN>` (required)

**Request Body:**
```json
{
  "pseudonym": "string (required, e.g., EMAIL_80779724a9b108fc)",
  "session_id": "string (required)"
}
```

**Response (200 OK):**
```json
{
  "original": "alice@example.com"
}
```

**Response (401 Unauthorized):**
```json
{
  "error": "Unauthorized: valid JWT required"
}
```

**Response (404 Not Found):**
```json
{
  "error": "Pseudonym not found in session"
}
```

**Security Considerations:**
- **JWT validation:** Token must be signed by trusted issuer
- **Audit logging:** All reidentify requests logged with trace_id
- **Session scope:** Pseudonyms only available within session TTL (default: 10 minutes)
- **No persistent mappings:** Mappings cleared after session expires

**Use Cases:**
- Investigate audit events (with authorization)
- Customer support (with user consent)
- Compliance audits (restricted access)

**Example: Reidentify with JWT**
```bash
# Get JWT token (from your auth service)
JWT_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# Reidentify pseudonym
curl -X POST http://localhost:8089/guard/reidentify \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "pseudonym": "EMAIL_80779724a9b108fc",
    "session_id": "sess_abc123"
  }' | jq .original
# Output: "alice@example.com"
```

---

### POST /internal/flush-session

**Purpose:** Clear session state (for testing or manual cleanup).

**Request:**
```bash
curl -X POST http://localhost:8089/internal/flush-session \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "sess_abc123"
  }'
```

**Request Body:**
```json
{
  "session_id": "string (required)"
}
```

**Response (200 OK):**
```json
{
  "status": "flushed",
  "session_id": "sess_abc123"
}
```

**Use Cases:**
- Testing (reset state between tests)
- Manual cleanup (remove expired sessions)
- Not exposed externally (internal only)

**⚠️ Warning:** This endpoint does not require authentication. Do not expose externally.

---

## Integration Patterns

### Controller Integration

The controller service can optionally call privacy-guard to mask PII before storing audit events.

**Configuration (`deploy/compose/ce.dev.yml`):**
```yaml
controller:
  environment:
    - GUARD_ENABLED=true
    - GUARD_URL=http://privacy-guard:8089
```

**Code Example (`src/controller/src/main.rs`):**
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct MaskRequest {
    text: String,
    tenant_id: String,
}

#[derive(Deserialize)]
struct MaskResponse {
    masked_text: String,
    redactions: HashMap<String, usize>,
}

async fn audit_ingest(
    State(state): State<Arc<AppState>>,
    Json(event): Json<AuditEvent>,
) -> Result<Json<AuditResponse>, AppError> {
    // Mask content field if present and guard enabled
    let content = if let Some(text) = event.content {
        if state.guard_enabled {
            let masked = state.guard_client.mask_text(&text, &event.tenant_id).await?;
            info!(
                redactions = ?masked.redactions,
                "PII masked in audit event"
            );
            Some(masked.masked_text)
        } else {
            Some(text)
        }
    } else {
        None
    };

    // Store event with masked content
    let stored_event = AuditEvent {
        content,
        ..event
    };

    Ok(Json(AuditResponse {
        stored: stored_event,
    }))
}
```

**Guard Client (`src/controller/src/guard_client.rs`):**
```rust
use reqwest::Client;
use std::time::Duration;

pub struct GuardClient {
    client: Client,
    base_url: String,
    enabled: bool,
}

impl GuardClient {
    pub fn new(base_url: String, enabled: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            enabled,
        }
    }

    pub async fn mask_text(
        &self,
        text: &str,
        tenant_id: &str,
    ) -> Result<MaskResponse, GuardError> {
        if !self.enabled {
            return Ok(MaskResponse {
                masked_text: text.to_string(),
                redactions: HashMap::new(),
            });
        }

        let url = format!("{}/guard/mask", self.base_url);
        let req = MaskRequest {
            text: text.to_string(),
            tenant_id: tenant_id.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| GuardError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(GuardError::ApiError(resp.status().as_u16()));
        }

        resp.json::<MaskResponse>()
            .await
            .map_err(|e| GuardError::ParseError(e.to_string()))
    }
}
```

**Error Handling (Fail-Open Mode):**
```rust
// If guard fails, log error but don't block the request
match state.guard_client.mask_text(&text, &tenant_id).await {
    Ok(masked) => {
        info!("PII masked successfully");
        Some(masked.masked_text)
    }
    Err(e) => {
        warn!(error = %e, "Guard unavailable, storing original text");
        Some(text) // Fail-open: store original if guard fails
    }
}
```

**Testing:**
```bash
# Test with guard enabled
GUARD_ENABLED=true cargo test --test controller_guard_tests

# Test with guard disabled
GUARD_ENABLED=false cargo test --test controller_guard_tests
```

---

### Agent-Side Integration (Future)

**Phase 3+ Pattern:** Agent-side wrapper for LLM provider calls.

**Conceptual Flow:**
1. Agent prepares to call LLM provider (OpenAI, Anthropic, etc.)
2. Wrapper intercepts request, extracts user message
3. Wrapper calls `/guard/mask` to mask PII
4. Wrapper sends masked message to LLM provider
5. LLM response returned to agent
6. (Optional) Wrapper calls `/guard/reidentify` if needed

**Example Wrapper (Conceptual):**
```python
import requests
import openai

class PrivacyGuardWrapper:
    def __init__(self, guard_url="http://localhost:8089", tenant_id="default"):
        self.guard_url = guard_url
        self.tenant_id = tenant_id

    def mask(self, text):
        """Mask PII in text before sending to LLM."""
        resp = requests.post(
            f"{self.guard_url}/guard/mask",
            json={"text": text, "tenant_id": self.tenant_id}
        )
        resp.raise_for_status()
        return resp.json()

    def chat_completion(self, messages, **kwargs):
        """OpenAI chat completion with PII masking."""
        # Mask user message
        user_msg = messages[-1]["content"]
        masked_result = self.mask(user_msg)
        
        # Replace user message with masked version
        masked_messages = messages[:-1] + [{
            "role": "user",
            "content": masked_result["masked_text"]
        }]
        
        # Call OpenAI with masked message
        response = openai.ChatCompletion.create(
            messages=masked_messages,
            **kwargs
        )
        
        return response

# Usage
guard = PrivacyGuardWrapper()
response = guard.chat_completion([
    {"role": "user", "content": "My email is alice@example.com"}
])
# LLM sees: "My email is EMAIL_80779724a9b108fc"
```

**Future Enhancements (Post-MVP):**
- Automatic reidentification in LLM responses
- Multi-turn conversation state management
- Provider-specific wrappers (OpenAI, Anthropic, Gemini)
- Streaming support with incremental masking

---

## Error Handling

### HTTP Status Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | Success | Process response normally |
| 400 | Bad Request | Fix request body (missing fields, invalid JSON) |
| 401 | Unauthorized | Provide valid JWT token (reidentify only) |
| 404 | Not Found | Pseudonym not in session or session expired |
| 500 | Internal Error | Retry with exponential backoff |
| 503 | Service Unavailable | Service starting or overloaded, retry |

### Error Response Format

```json
{
  "error": "Error message describing what went wrong"
}
```

### Example Error Responses

**400 Bad Request:**
```json
{
  "error": "Missing required field: tenant_id"
}
```

**401 Unauthorized:**
```json
{
  "error": "Unauthorized: valid JWT required"
}
```

**404 Not Found:**
```json
{
  "error": "Pseudonym EMAIL_abc123 not found in session sess_xyz789"
}
```

**500 Internal Server Error:**
```json
{
  "error": "Detection engine error: regex timeout exceeded"
}
```

### Client-Side Error Handling

**Retry with Exponential Backoff:**
```python
import time
import requests
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry

def create_guard_client():
    """Create HTTP client with retry logic."""
    session = requests.Session()
    
    retry = Retry(
        total=3,
        backoff_factor=0.5,  # 0.5s, 1s, 2s
        status_forcelist=[500, 502, 503, 504],
        allowed_methods=["POST", "GET"]
    )
    
    adapter = HTTPAdapter(max_retries=retry)
    session.mount("http://", adapter)
    session.mount("https://", adapter)
    
    return session

# Usage
client = create_guard_client()
response = client.post(
    "http://localhost:8089/guard/mask",
    json={"text": "...", "tenant_id": "org1"},
    timeout=5
)
```

**Graceful Degradation (Fail-Open):**
```python
def safe_mask(text, tenant_id):
    """Mask text with fallback to original on error."""
    try:
        resp = requests.post(
            "http://localhost:8089/guard/mask",
            json={"text": text, "tenant_id": tenant_id},
            timeout=5
        )
        resp.raise_for_status()
        return resp.json()["masked_text"]
    except Exception as e:
        logger.warning(f"Guard unavailable, using original text: {e}")
        return text  # Fail-open: return original if guard fails
```

**Strict Mode (Fail-Closed):**
```python
def strict_mask(text, tenant_id):
    """Mask text or reject request on error."""
    try:
        resp = requests.post(
            "http://localhost:8089/guard/mask",
            json={"text": text, "tenant_id": tenant_id},
            timeout=5
        )
        resp.raise_for_status()
        return resp.json()["masked_text"]
    except Exception as e:
        logger.error(f"Guard unavailable, rejecting request: {e}")
        raise ValueError("Privacy guard unavailable") from e
```

---

## Performance Considerations

### Detection Modes Comparison (Phase 2.2+)

**Regex-Only Mode** (GUARD_MODEL_ENABLED=false)
- **P50:** ~16ms
- **P95:** ~22ms
- **P99:** ~23ms
- **Use Case:** High-volume APIs, latency-critical applications
- **Accuracy:** High precision, may miss ambiguous PII (e.g., person names without titles)

**Model-Enhanced Mode** (GUARD_MODEL_ENABLED=true)
- **P50:** ~500-700ms (qwen3:0.6b model)
- **P95:** ~1000ms
- **P99:** ~2000ms
- **Use Case:** Compliance-critical, accuracy-first applications
- **Accuracy:** +10-20% better recall (especially for person names, organizations)

**Latency Breakdown (Model-Enhanced):**
- Regex detection: ~16ms
- Model inference (Ollama NER): ~450-650ms
- Result merging: ~5-10ms
- **Total:** ~500-700ms

**Tradeoff Decision:**
- **Choose Regex-Only:** When latency < 100ms is critical (user-facing, real-time)
- **Choose Model-Enhanced:** When accuracy is more important than latency (audit logs, compliance)

**Configuration:** See [`privacy-guard-config.md`](./privacy-guard-config.md#model-enhanced-detection-phase-22) for enabling model-enhanced mode.

---

### Latency Targets

**Target Performance (from ADR-0021):**
- P50: ≤ 500ms (regex-only), ≤ 700ms (with model)
- P95: ≤ 1000ms
- P99: ≤ 2000ms

**Measured Performance (Phase 2 - Regex-Only):**
- Typical request: 2-10ms for short text (< 1KB)
- Complex text: 10-50ms for long text (1-10KB)
- P50: 16ms, P95: 22ms, P99: 23ms

**Phase 2.2 (Model-Enhanced):**
- To be measured in smoke tests (Task C2)

### Request Size Limits

**Default Limits (configurable in policy.yaml):**
- Max input size: 10KB per request
- Max regex timeout: 100ms per pattern
- Max concurrent requests: 100

**Recommendations:**
- **Short texts (< 1KB):** Use `/guard/mask` directly
- **Long texts (1-10KB):** Consider batching or chunking
- **Very long texts (> 10KB):** Split into smaller requests

### Batching Strategy

**Don't batch multiple unrelated texts:**
```bash
# ❌ Bad: Multiple unrelated texts in one request
curl -X POST http://localhost:8089/guard/mask \
  -d '{"text": "Email 1: alice@example.com\nEmail 2: bob@example.com", ...}'
```

**Do send separate requests:**
```bash
# ✅ Good: Separate requests for separate contexts
for email in alice@example.com bob@example.com; do
  curl -X POST http://localhost:8089/guard/mask \
    -d "{\"text\": \"$email\", \"tenant_id\": \"org1\"}"
done
```

**Reason:** Separate requests ensure proper session isolation and audit trails.

### Caching Considerations

**Client-Side Caching:**
Privacy Guard is deterministic, so clients can cache results:

```python
from functools import lru_cache

@lru_cache(maxsize=1000)
def mask_cached(text, tenant_id):
    """Cache masked results for identical inputs."""
    resp = requests.post(
        "http://localhost:8089/guard/mask",
        json={"text": text, "tenant_id": tenant_id}
    )
    return resp.json()["masked_text"]

# Same input returns cached result
result1 = mask_cached("alice@example.com", "org1")  # Network call
result2 = mask_cached("alice@example.com", "org1")  # From cache
```

**⚠️ Cache Invalidation:** Clear cache when rules or policy change.

---

## Security Best Practices

### 1. Never Log Raw PII

Privacy Guard follows strict no-PII logging:

```rust
// ✅ Good: Log counts only
info!(
    entity_counts = ?redactions,
    total = redactions.values().sum::<usize>(),
    "Masked PII in request"
);

// ❌ Bad: NEVER log original or masked text
error!("Failed to mask: {}", original_text);  // ❌ PII leak
error!("Masked result: {}", masked_text);     // ❌ Pseudonym leak
```

### 2. Protect PSEUDO_SALT

**PSEUDO_SALT is the master key for pseudonymization:**
- Store in Vault or secret manager (not in code or config)
- Rotate periodically (e.g., quarterly)
- Never log or expose in errors
- Use different salts per environment

**Vault Integration (Phase 1.2):**
```bash
# Set PSEUDO_SALT in Vault
vault kv put secret/guard pseudo_salt="$(openssl rand -base64 32)"

# Controller reads from Vault and passes to guard via environment
PSEUDO_SALT=$(vault kv get -field=pseudo_salt secret/guard)
```

### 3. Restrict Reidentification

**JWT Claims Required:**
```json
{
  "sub": "user@example.com",
  "aud": "privacy-guard",
  "scope": "reidentify",
  "exp": 1699999999
}
```

**Audit All Reidentify Requests:**
```rust
info!(
    user = jwt.sub,
    pseudonym = request.pseudonym,
    session_id = request.session_id,
    trace_id = trace_id,
    "Reidentification request"
);
```

### 4. Session TTL Management

**Default Session TTL:** 10 minutes (configurable)

**Automatic Cleanup:**
- Sessions expire after TTL
- Mappings deleted on expiration
- No persistent storage

**Manual Flush (if needed):**
```bash
# Flush session immediately after use
curl -X POST http://localhost:8089/internal/flush-session \
  -d '{"session_id": "sess_abc123"}'
```

### 5. Network Isolation

**Production Deployment:**
- Privacy guard should NOT be exposed to public internet
- Access restricted to:
  - Controller service (Docker Compose network)
  - Future agent wrappers (internal network only)
  - Admin access via VPN or bastion host

**Docker Compose Network:**
```yaml
services:
  privacy-guard:
    networks:
      - internal  # Not exposed to external
    # No 'ports:' mapping in production (only healthcheck)
```

---

## Testing

### Unit Tests

**Run Rust unit tests:**
```bash
cd src/privacy-guard
cargo test
```

**Example Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_masking() {
        let text = "Email: alice@example.com";
        let tenant_id = "org1";

        let result1 = mask_text(text, tenant_id).unwrap();
        let result2 = mask_text(text, tenant_id).unwrap();

        assert_eq!(result1.masked_text, result2.masked_text);
        assert_eq!(result1.redactions["EMAIL"], 1);
    }
}
```

### Integration Tests

**Run integration test script:**
```bash
# Start services
docker compose -f deploy/compose/ce.dev.yml --profile privacy-guard up -d

# Run tests
bash tests/integration/test_controller_guard.sh
```

**Test Scenarios:**
1. Health checks (guard and controller)
2. Ingest without content field (no guard call)
3. Ingest with PII content (guard masks it)
4. Determinism (same email → same pseudonym)
5. Error handling (guard unavailable)

### Manual Testing with curl

**Test Detection:**
```bash
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "SSN: 123-45-6789, Email: alice@example.com, Phone: 555-867-5309",
    "tenant_id": "test"
  }' | jq
```

**Test Masking:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact Alice at alice@example.com or 555-867-5309",
    "tenant_id": "test"
  }' | jq
```

**Test Determinism:**
```bash
# Call 1
RESULT1=$(curl -s -X POST http://localhost:8089/guard/mask \
  -d '{"text": "alice@example.com", "tenant_id": "test"}' | jq -r .masked_text)

# Call 2
RESULT2=$(curl -s -X POST http://localhost:8089/guard/mask \
  -d '{"text": "alice@example.com", "tenant_id": "test"}' | jq -r .masked_text)

# Compare
if [ "$RESULT1" = "$RESULT2" ]; then
  echo "✅ Determinism verified"
else
  echo "❌ Results differ: $RESULT1 vs $RESULT2"
fi
```

---

## Troubleshooting

### Service Won't Start

**Symptom:** Container exits immediately or fails healthcheck

**Debugging:**
```bash
# Check container logs
docker logs privacy-guard

# Common issues:
# 1. Missing PSEUDO_SALT
# 2. Invalid rules.yaml syntax
# 3. Port 8089 already in use
# 4. Vault dependency not healthy
```

**Fix:**
```bash
# Verify environment variables
docker compose -f deploy/compose/ce.dev.yml config | grep -A5 privacy-guard

# Verify Vault is healthy
docker compose -f deploy/compose/ce.dev.yml ps vault

# Restart with fresh logs
docker compose -f deploy/compose/ce.dev.yml restart privacy-guard
docker logs -f privacy-guard
```

### Detection Not Working

**Symptom:** `/guard/scan` returns empty detections array

**Debugging:**
```bash
# Check rule count in status
curl http://localhost:8089/status | jq .rule_count
# Should be 24 (8 entity types)

# Check if pattern exists in rules.yaml
grep -A5 "entity_type: EMAIL" deploy/compose/guard-config/rules.yaml

# Test pattern manually
echo "alice@example.com" | grep -E '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
```

**Fix:**
```bash
# Reload configuration (restart service)
docker compose -f deploy/compose/ce.dev.yml restart privacy-guard

# Or fix rules.yaml and rebuild
# Edit deploy/compose/guard-config/rules.yaml
docker compose -f deploy/compose/ce.dev.yml up -d --build privacy-guard
```

### Masking Not Deterministic

**Symptom:** Same input produces different masked output

**Debugging:**
```bash
# Check if PSEUDO_SALT is set
docker compose -f deploy/compose/ce.dev.yml exec privacy-guard env | grep PSEUDO_SALT

# Check if tenant_id is consistent
# (different tenant_id = different pseudonyms)
```

**Fix:**
```bash
# Set PSEUDO_SALT in .env.ce
echo "PSEUDO_SALT=$(openssl rand -base64 32)" >> deploy/compose/.env.ce

# Restart service
docker compose -f deploy/compose/ce.dev.yml restart privacy-guard
```

### High Latency

**Symptom:** Requests take > 2 seconds

**Debugging:**
```bash
# Check request size
curl -X POST http://localhost:8089/guard/mask \
  -d '{"text": "...", "tenant_id": "test"}' \
  -w "\nTime: %{time_total}s\n"

# Check concurrent requests
docker stats privacy-guard
```

**Fix:**
```bash
# Reduce input size (< 10KB recommended)
# Increase timeout in policy.yaml
# Scale horizontally (multiple guard instances)
```

### Controller Integration Not Working

**Symptom:** Controller doesn't call guard even when enabled

**Debugging:**
```bash
# Check controller environment
docker compose -f deploy/compose/ce.dev.yml exec controller env | grep GUARD

# Check controller logs
docker logs controller | grep -i guard

# Test guard directly
curl -X POST http://localhost:8089/guard/mask \
  -d '{"text": "test", "tenant_id": "org1"}'
```

**Fix:**
```bash
# Enable guard in controller
export GUARD_ENABLED=true
export GUARD_URL=http://privacy-guard:8089

# Restart controller
docker compose -f deploy/compose/ce.dev.yml restart controller
```

---

## Next Steps

1. **Configuration:** Read [`privacy-guard-config.md`](./privacy-guard-config.md) to customize rules and policy
2. **Testing:** Run smoke tests from [`docs/tests/smoke-phase2.md`](../tests/smoke-phase2.md) (when available)
3. **Production:** Review security checklist and deploy with proper secret management
4. **Monitoring:** Set up metrics and alerts for performance and error rates

---

**Questions or Issues?**
- ADR-0021: [Privacy Guard Rust Implementation](../adr/0021-privacy-guard-rust-implementation.md)
- ADR-0022: [PII Detection Rules and FPE](../adr/0022-pii-detection-rules-and-fpe.md)
- Configuration Guide: [`privacy-guard-config.md`](./privacy-guard-config.md)
