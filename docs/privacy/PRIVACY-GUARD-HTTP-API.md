# Privacy Guard HTTP API Guide

**Version:** v0.5.0  
**Service Port:** 8089  
**Protocol:** HTTP/1.1 (REST)  
**Authentication:** JWT (for `/guard/reidentify` only)

---

## Table of Contents

1. [Overview](#overview)
2. [Why HTTP API Instead of MCP?](#why-http-api-instead-of-mcp)
3. [Quick Start](#quick-start)
4. [API Reference](#api-reference)
5. [Integration Patterns](#integration-patterns)
6. [Configuration](#configuration)
7. [Performance](#performance)
8. [Troubleshooting](#troubleshooting)
9. [Security Considerations](#security-considerations)

---

## Overview

Privacy Guard is a **standalone HTTP service** that provides PII (Personally Identifiable Information) detection and masking for enterprise applications. It runs as a Docker container and exposes a REST API for:

- **PII Detection**: Scan text for sensitive data (SSN, Email, Phone, Credit Card, etc.)
- **PII Masking**: Replace detected PII with deterministic pseudonyms
- **Re-identification**: Restore original values from pseudonyms (admin-only)
- **Session Management**: Track masked tokens per user session
- **Audit Logging**: Record redaction events for compliance

### Architecture

```
┌─────────────────────┐
│  Your Application   │
│  (Goose, Admin UI)  │
└──────────┬──────────┘
           │ HTTP
           │
┌──────────▼──────────────────┐
│  Privacy Guard Service      │
│  http://localhost:8089       │
│                              │
│  Endpoints:                  │
│  - GET  /status              │
│  - POST /guard/scan          │
│  - POST /guard/mask          │
│  - POST /guard/reidentify    │
│  - POST /internal/flush      │
└──────────┬───────────────────┘
           │
┌──────────▼──────────┐
│  PostgreSQL         │
│  (Audit Logs)       │
└─────────────────────┘
```

---

## Why HTTP API Instead of MCP?

### The MCP Limitation Discovery

During Phase 5 development, we investigated using Privacy Guard as an **MCP (Model Context Protocol) extension** for Goose Desktop. We discovered a critical architectural limitation:

**Problem:**
```
User types "My SSN is 123-45-6789"
    ↓
Goose sends to LLM (OpenRouter) ← ⚠️ PII LEAKED
    ↓
LLM decides to call Privacy Guard MCP tool
    ↓
Privacy Guard masks PII
    ↓
Too late - LLM already saw raw PII ❌
```

**Root Cause:** MCP tools are **called BY the LLM**, not BEFORE the LLM sees user input. This means:
- The LLM receives the raw prompt containing PII first
- The LLM then decides (via tool calling) to invoke Privacy Guard
- By the time Privacy Guard runs, the PII has already been sent to the cloud provider

**Impact:** This violates enterprise privacy requirements where PII must **never** reach external LLM APIs.

### HTTP API Solution

The HTTP API enables **interception BEFORE LLM access**:

```
User Input
    ↓
[Privacy Guard HTTP API] ← INTERCEPTS HERE
    ↓ (scans → masks → forwards)
OpenRouter API (only sees masked text: "My SSN is SSN_a1b2c3d4")
    ↓
LLM processes masked version ✅
```

**Implementation Approaches:**
1. **Proxy Server** - Intercept HTTP requests to OpenRouter (see `docs/decisions/privacy-guard-llm-integration-options.md`)
2. **UI Integration** - Call Privacy Guard before submitting prompts (Goose Desktop fork)
3. **Direct Integration** - Application-level masking (current approach for Admin UI)

**Decision:** For v0.5.0 MVP, we use **HTTP API with direct integration** in the Controller. Future versions may add proxy or UI-level interception.

**Reference:** Full analysis in `docs/decisions/privacy-guard-llm-integration-options.md`

---

## Quick Start

### 1. Start Privacy Guard Service

```bash
# Using Docker Compose (recommended)
cd deploy/compose
docker-compose up -d privacy-guard

# Verify service is running
curl http://localhost:8089/status
```

**Expected Response:**
```json
{
  "status": "healthy",
  "mode": "Hybrid",
  "rule_count": 10,
  "config_loaded": true,
  "model_enabled": true,
  "model_name": "qwen3:0.6b"
}
```

### 2. Scan Text for PII

```bash
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "My SSN is 123-45-6789 and email is john@example.com",
    "tenant_id": "test-org"
  }'
```

**Response:**
```json
{
  "detections": [
    {
      "start": 10,
      "end": 21,
      "entity_type": "SSN",
      "confidence": "High",
      "matched_text": "123-45-6789"
    },
    {
      "start": 35,
      "end": 52,
      "entity_type": "EMAIL",
      "confidence": "High",
      "matched_text": "john@example.com"
    }
  ]
}
```

### 3. Mask PII

```bash
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "My SSN is 123-45-6789 and email is john@example.com",
    "tenant_id": "test-org",
    "mode": "hybrid"
  }'
```

**Response:**
```json
{
  "masked_text": "My SSN is SSN_a1b2c3d4 and email is EMAIL_x9y8z7w6",
  "redactions": {
    "SSN": 1,
    "EMAIL": 1
  },
  "session_id": "sess_0550d493-0a58-428a-b9b7-7b346c0369d8"
}
```

**Key Points:**
- Pseudonyms are **deterministic**: Same input → same token (within session)
- Session ID tracks the mapping between tokens and original values
- Redactions count shows how many of each PII type were masked

---

## API Reference

### GET /status

**Description:** Health check and service configuration

**Authentication:** None

**Request:**
```bash
curl http://localhost:8089/status
```

**Response:**
```json
{
  "status": "healthy",
  "mode": "Hybrid",
  "rule_count": 10,
  "config_loaded": true,
  "model_enabled": true,
  "model_name": "qwen3:0.6b"
}
```

**Fields:**
- `status`: Service health (`"healthy"` or `"degraded"`)
- `mode`: Detection mode (`"RulesOnly"`, `"NerOnly"`, `"Hybrid"`, `"Off"`)
- `rule_count`: Number of loaded regex patterns (10 for default rules)
- `config_loaded`: Whether configuration loaded successfully
- `model_enabled`: Whether Ollama NER is available
- `model_name`: Ollama model name (e.g., `qwen3:0.6b`)

---

### POST /guard/scan

**Description:** Detect PII in text without masking

**Authentication:** None

**Request Body:**
```json
{
  "text": "Contact john@example.com or call 555-1234",
  "tenant_id": "my-org"
}
```

**Request Fields:**
- `text` (string, required): Text to scan for PII
- `tenant_id` (string, optional): Organization identifier for audit logs

**Response:**
```json
{
  "detections": [
    {
      "start": 8,
      "end": 25,
      "entity_type": "EMAIL",
      "confidence": "High",
      "matched_text": "john@example.com"
    },
    {
      "start": 37,
      "end": 45,
      "entity_type": "PHONE",
      "confidence": "High",
      "matched_text": "555-1234"
    }
  ]
}
```

**Response Fields:**
- `detections`: Array of detected PII items
  - `start`: Character offset where PII starts (0-indexed)
  - `end`: Character offset where PII ends
  - `entity_type`: PII category (`SSN`, `EMAIL`, `PHONE`, `CREDIT_CARD`, `EMPLOYEE_ID`, `IP_ADDRESS`, `PERSON`, `ORG`, `LOCATION`)
  - `confidence`: Detection confidence (`High`, `Medium`, `Low`)
  - `matched_text`: The actual PII value found

**Example (cURL):**
```bash
curl -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Process employee SSN 123-45-6789",
    "tenant_id": "finance-dept"
  }'
```

---

### POST /guard/mask

**Description:** Mask PII in text with pseudonyms

**Authentication:** None

**Request Body:**
```json
{
  "text": "My SSN is 123-45-6789 and email is john@example.com",
  "tenant_id": "my-org",
  "session_id": "sess_abc123",
  "mode": "hybrid"
}
```

**Request Fields:**
- `text` (string, required): Text to mask
- `tenant_id` (string, required): Organization identifier
- `session_id` (string, optional): Reuse existing session or create new one
- `mode` (string, optional): Detection mode override (`"rules"`, `"ner"`, `"hybrid"`)

**Response:**
```json
{
  "masked_text": "My SSN is SSN_a1b2c3d4 and email is EMAIL_x9y8z7w6",
  "redactions": {
    "SSN": 1,
    "EMAIL": 1
  },
  "session_id": "sess_0550d493-0a58-428a-b9b7-7b346c0369d8"
}
```

**Response Fields:**
- `masked_text`: Text with PII replaced by pseudonyms
- `redactions`: Count of masked items per PII category
- `session_id`: Session ID for re-identification (store this for later unmask)

**Pseudonym Format:**
- `SSN_<8_hex_chars>`: Social Security Number
- `EMAIL_<16_hex_chars>`: Email address
- `PHONE_<8_hex_chars>`: Phone number
- `CC_<8_hex_chars>`: Credit card number
- `EMPID_<8_hex_chars>`: Employee ID
- `IP_<8_hex_chars>`: IP address
- `PERSON_<8_hex_chars>`: Person name
- `ORG_<8_hex_chars>`: Organization name

**Session Behavior:**
- Same `session_id` + same PII value → **same pseudonym** (deterministic)
- Different `session_id` → **different pseudonym** (prevents cross-session correlation)
- Session stored in-memory (Redis backend planned for production)

**Example (cURL):**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact John Smith at john@acme.com or SSN 123-45-6789",
    "tenant_id": "hr-dept",
    "mode": "hybrid"
  }'
```

---

### POST /guard/reidentify

**Description:** Restore original value from pseudonym (admin-only)

**Authentication:** **JWT required** (Bearer token in Authorization header)

**Request Body:**
```json
{
  "pseudonym": "SSN_a1b2c3d4",
  "session_id": "sess_0550d493-0a58-428a-b9b7-7b346c0369d8"
}
```

**Request Fields:**
- `pseudonym` (string, required): The masked token (e.g., `SSN_a1b2c3d4`)
- `session_id` (string, required): Session ID from original masking

**Request Headers:**
- `Authorization: Bearer <JWT_TOKEN>` (required)

**Response:**
```json
{
  "original": "123-45-6789"
}
```

**Error Responses:**
- `401 Unauthorized`: Missing or invalid JWT token
- `404 Not Found`: Session ID not found or pseudonym not in session

**Example (cURL):**
```bash
# Get JWT token first (from Keycloak)
TOKEN=$(curl -X POST http://localhost:8081/realms/goose/protocol/openid-connect/token \
  -d "client_id=controller-api" \
  -d "client_secret=your-secret" \
  -d "grant_type=client_credentials" \
  | jq -r '.access_token')

# Reidentify pseudonym
curl -X POST http://localhost:8089/guard/reidentify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "pseudonym": "SSN_a1b2c3d4",
    "session_id": "sess_0550d493-0a58-428a-b9b7-7b346c0369d8"
  }'
```

**Security Note:** This endpoint is **highly restricted** and should only be accessible to:
- Compliance officers
- Legal team (for attorney-client privilege cases)
- Authorized administrators with explicit need-to-know

---

### POST /internal/flush-session

**Description:** Delete session and all token mappings (cleanup)

**Authentication:** None (internal endpoint)

**Request Body:**
```json
{
  "session_id": "sess_0550d493-0a58-428a-b9b7-7b346c0369d8"
}
```

**Response:**
```json
{
  "status": "flushed"
}
```

**Use Cases:**
- User logs out → flush their session
- Session expired → cleanup old tokens
- Testing → reset state between tests

**Example (cURL):**
```bash
curl -X POST http://localhost:8089/internal/flush-session \
  -H "Content-Type: application/json" \
  -d '{"session_id": "sess_abc123"}'
```

---

## Integration Patterns

### Pattern 1: Direct Integration (Controller API)

**Use Case:** Backend service needs to mask user input before storing or processing

```rust
// Example: Controller API masking user profile data
use reqwest::Client;
use serde_json::json;

async fn mask_user_data(text: &str, tenant_id: &str) -> Result<String, Error> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8089/guard/mask")
        .json(&json!({
            "text": text,
            "tenant_id": tenant_id,
            "mode": "hybrid"
        }))
        .send()
        .await?;
    
    let masked: MaskResponse = response.json().await?;
    Ok(masked.masked_text)
}
```

**Integration Points:**
- User profile creation (mask PII in bio fields)
- Audit log submission (mask sensitive data before logging)
- Session exports (mask PII before download)

---

### Pattern 2: Proxy Interception (Goose Desktop)

**Use Case:** Intercept LLM API requests to mask prompts BEFORE sending to cloud

```typescript
// Example: Privacy Guard Proxy Server (Option 1 from decision doc)
import express from 'express';

const app = express();

app.post('/api/v1/chat/completions', async (req, res) => {
  const { messages } = req.body;
  const userMsg = messages[messages.length - 1].content;
  
  // Step 1: Mask PII
  const masked = await fetch('http://localhost:8089/guard/mask', {
    method: 'POST',
    body: JSON.stringify({
      text: userMsg,
      tenant_id: getUserTenantId(),
      mode: 'hybrid'
    })
  }).then(r => r.json());
  
  // Step 2: Forward masked text to OpenRouter
  messages[messages.length - 1].content = masked.masked_text;
  const llmResp = await fetch('https://openrouter.ai/api/v1/chat/completions', {
    method: 'POST',
    headers: req.headers,
    body: JSON.stringify({ ...req.body, messages })
  });
  
  // Step 3: Return response (optionally unmask)
  res.json(await llmResp.json());
});

app.listen(8090); // Goose Desktop configured to use localhost:8090
```

**Configuration:**
```yaml
# ~/.config/goose/config.yaml
GOOSE_PROVIDER__OPENROUTER_BASE_URL: http://localhost:8090/api/v1
PRIVACY_GUARD_ENABLED: true
```

**Pros:**
- No Goose Desktop code changes
- Transparent to user
- LLM never sees raw PII ✅

**Cons:**
- Adds 50-200ms latency
- Requires separate proxy service
- Must handle multiple LLM providers

---

### Pattern 3: UI-Level Masking (Goose Desktop Fork)

**Use Case:** Mask PII in chat input component before submitting to backend

```typescript
// Example: ChatInput.tsx component
import { usePrivacyGuard } from '../lib/privacy-guard';

export function ChatInput() {
  const { scan, mask, enabled } = usePrivacyGuard();
  
  async function handleSubmit(message: string) {
    if (enabled) {
      // Scan for PII
      const scanResult = await scan(message);
      
      if (scanResult.detections.length > 0) {
        // Show notification
        toast.warning(`🔒 Detected ${scanResult.detections.length} PII items. Masking...`);
        
        // Mask PII
        const masked = await mask(message);
        message = masked.masked_text;
      }
    }
    
    // Send to Goose backend (masked if PII found)
    await goose.sendMessage(message);
  }
  
  return <input onSubmit={handleSubmit} />;
}
```

**Pros:**
- User sees notification when PII is detected
- Can show masked vs. original in UI
- No proxy service needed

**Cons:**
- Requires Goose Desktop fork
- Maintenance burden for upstream merges

---

### Pattern 4: Batch Processing

**Use Case:** Scan/mask large datasets (audit logs, session exports)

```python
# Example: Batch PII masking for audit logs
import requests
import json

def mask_audit_logs(logs: list[str], tenant_id: str) -> list[str]:
    """Mask PII in batch of audit logs."""
    masked_logs = []
    
    for log in logs:
        response = requests.post(
            "http://localhost:8089/guard/mask",
            json={
                "text": log,
                "tenant_id": tenant_id,
                "mode": "hybrid"
            },
            timeout=5
        )
        result = response.json()
        masked_logs.append(result["masked_text"])
    
    return masked_logs

# Usage
logs = [
    "User john@acme.com logged in from 192.168.1.1",
    "SSN 123-45-6789 updated by admin",
    "Credit card ****1234 charged $99.99"
]
masked = mask_audit_logs(logs, tenant_id="finance-dept")
print(masked)
# Output:
# [
#   "User EMAIL_x9y8z7w6 logged in from IP_a1b2c3d4",
#   "SSN SSN_d4e5f6g7 updated by admin",
#   "Credit card CC_h8i9j0k1 charged $99.99"
# ]
```

**Performance Tip:** Use async/await for parallel processing:
```python
import asyncio
import aiohttp

async def mask_batch_async(logs: list[str], tenant_id: str) -> list[str]:
    async with aiohttp.ClientSession() as session:
        tasks = [
            mask_single(session, log, tenant_id)
            for log in logs
        ]
        return await asyncio.gather(*tasks)

async def mask_single(session, text, tenant_id):
    async with session.post(
        "http://localhost:8089/guard/mask",
        json={"text": text, "tenant_id": tenant_id}
    ) as resp:
        result = await resp.json()
        return result["masked_text"]
```

---

## Configuration

### Environment Variables

Privacy Guard reads configuration from environment variables (set in `docker-compose.yml` or `.env` file):

```bash
# Detection Mode
# Options: RulesOnly, NerOnly, Hybrid, Off
# Default: Hybrid (regex + Ollama NER)
GUARD_MODE=Hybrid

# Service Port
# Default: 8089
GUARD_PORT=8089

# Pseudonym Salt (required for masking)
# Must be set for /guard/mask to work
# Generate with: openssl rand -base64 32
PSEUDO_SALT=your-secret-salt-here

# Ollama Configuration (for NER mode)
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL=qwen3:0.6b
OLLAMA_ENABLED=true

# Logging Level
# Options: ERROR, WARN, INFO, DEBUG, TRACE
RUST_LOG=info
```

### Docker Compose Example

```yaml
# deploy/compose/docker-compose.yml
services:
  privacy-guard:
    build:
      context: ../../src/privacy-guard
      dockerfile: Dockerfile
    container_name: privacy-guard
    ports:
      - "8089:8089"
    environment:
      GUARD_PORT: 8089
      GUARD_MODE: Hybrid
      PSEUDO_SALT: ${PSEUDO_SALT}  # From .env file
      OLLAMA_URL: http://ollama:11434
      OLLAMA_MODEL: qwen3:0.6b
      OLLAMA_ENABLED: true
      RUST_LOG: info
    depends_on:
      - ollama
    networks:
      - goose-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8089/status"]
      interval: 10s
      timeout: 5s
      retries: 3
```

### Detection Modes

| Mode | Description | Performance | Accuracy | Use Case |
|------|-------------|-------------|----------|----------|
| **RulesOnly** | Regex patterns only | **Fastest** (10ms P50) | Good for structured PII | High-throughput, known formats (SSN, Email) |
| **NerOnly** | Ollama NER model only | Slower (~500ms P50) | Best for names, orgs | Unstructured text, legal compliance |
| **Hybrid** | Regex first, NER for gaps | Balanced (50-200ms P50) | **Recommended** | General-purpose, production use |
| **Off** | Passthrough (no detection) | 0ms overhead | N/A | Development, testing |

**Recommendation:** Use **Hybrid** mode for production (best balance of speed and accuracy).

---

## Performance

### Benchmarks (from Phase 5 Testing)

**Test Environment:**
- Docker Compose on Ubuntu Linux
- 6-core CPU, 16GB RAM
- Privacy Guard + Ollama containers

**Results (H7 Performance Test):**
```
Endpoint: /guard/scan
Method:   1000 requests (Hybrid mode)
P50:      10ms
P95:      18ms
P99:      25ms
Max:      30ms
Target:   <500ms
Result:   50x faster than target ✅
```

**Latency Breakdown:**
- Regex detection: ~2ms
- Ollama NER (if needed): ~8-15ms
- HTTP overhead: ~3ms
- **Total**: 10-20ms typical

**Throughput:**
- **Hybrid mode**: ~50-100 requests/sec
- **RulesOnly mode**: ~200-500 requests/sec
- **NerOnly mode**: ~5-10 requests/sec

### Performance Tuning

**1. Use RulesOnly Mode for High Throughput**
```bash
GUARD_MODE=RulesOnly  # 10x faster than Hybrid
```

**2. Batch Requests (Async)**
```python
# Instead of:
for text in texts:
    mask(text)  # Serial: 100 texts × 20ms = 2000ms

# Do:
await asyncio.gather(*[mask(t) for t in texts])  # Parallel: ~200ms
```

**3. Cache Scan Results (Application-Level)**
```python
from functools import lru_cache

@lru_cache(maxsize=1000)
def scan_cached(text: str) -> dict:
    """Cache scan results for identical text."""
    return requests.post("http://localhost:8089/guard/scan", json={"text": text}).json()
```

**4. Connection Pooling**
```python
import requests

# Create session (reuses TCP connections)
session = requests.Session()
session.mount('http://', requests.adapters.HTTPAdapter(pool_connections=10, pool_maxsize=20))

# Use session instead of requests directly
response = session.post("http://localhost:8089/guard/mask", json={"text": "..."})
```

---

## Troubleshooting

### Service Not Responding

**Symptom:**
```bash
curl http://localhost:8089/status
curl: (7) Failed to connect to localhost port 8089: Connection refused
```

**Diagnosis:**
```bash
# Check if container is running
docker ps | grep privacy-guard

# Check container logs
docker logs privacy-guard

# Check health
docker inspect privacy-guard | jq '.[0].State.Health'
```

**Solutions:**
1. **Start service:** `docker-compose up -d privacy-guard`
2. **Check port binding:** Ensure port 8089 is not in use
3. **Check firewall:** `sudo ufw allow 8089` (if using UFW)

---

### PSEUDO_SALT Not Set

**Symptom:**
```json
{
  "error": "PSEUDO_SALT not configured, masking unavailable"
}
```

**Solution:**
```bash
# Generate salt
openssl rand -base64 32

# Add to .env file
echo "PSEUDO_SALT=your-generated-salt-here" >> deploy/compose/.env

# Restart service
docker-compose restart privacy-guard
```

**Important:** Store `PSEUDO_SALT` securely (Vault, AWS Secrets Manager, etc.). Losing the salt means you cannot reidentify masked data.

---

### Ollama Not Available

**Symptom:**
```json
{
  "status": "healthy",
  "mode": "Hybrid",
  "model_enabled": false,
  "model_name": "qwen3:0.6b"
}
```

**Diagnosis:**
```bash
# Check Ollama container
docker ps | grep ollama

# Test Ollama directly
curl http://localhost:11434/api/tags

# Check Privacy Guard logs
docker logs privacy-guard | grep -i ollama
```

**Solutions:**
1. **Start Ollama:** `docker-compose up -d ollama`
2. **Pull model:** `docker exec ollama ollama pull qwen3:0.6b`
3. **Verify connectivity:** `curl http://ollama:11434/api/tags` (from inside privacy-guard container)

**Fallback Behavior:** If Ollama is unavailable, Privacy Guard automatically falls back to **RulesOnly mode** (regex patterns only).

---

### High Latency (>500ms)

**Diagnosis:**
```bash
# Run performance test
curl -w "Time: %{time_total}s\n" -X POST http://localhost:8089/guard/scan \
  -H "Content-Type: application/json" \
  -d '{"text":"Test","tenant_id":"test"}'
```

**Common Causes:**
1. **NerOnly mode with large text:** Switch to `Hybrid` or `RulesOnly`
2. **Ollama cold start:** First request takes ~2-5s (warm up with test request)
3. **Network latency:** Ensure Privacy Guard and Ollama are on same Docker network
4. **CPU throttling:** Check container resource limits

**Solutions:**
```bash
# Warm up Ollama
curl -X POST http://localhost:11434/api/generate \
  -d '{"model":"qwen3:0.6b","prompt":"test","stream":false}'

# Increase container resources (docker-compose.yml)
services:
  privacy-guard:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
```

---

### Session Not Found (404)

**Symptom:**
```bash
curl -X POST http://localhost:8089/guard/reidentify \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"pseudonym":"SSN_abc123","session_id":"sess_old"}'

# Response:
{"error": "Not found"}
```

**Causes:**
1. Session expired (Privacy Guard restarted → in-memory sessions lost)
2. Wrong session_id (typo or using different session)
3. Session flushed (explicit cleanup)

**Solutions:**
1. **Check session exists:** Sessions are in-memory, lost on restart
2. **Persist sessions:** Future versions will use Redis for persistence
3. **Regenerate mask:** If session lost, re-mask the original text to get new session

---

### JWT Validation Failing

**Symptom:**
```bash
curl -X POST http://localhost:8089/guard/reidentify \
  -H "Authorization: Bearer invalid-token" \
  -d '{"pseudonym":"SSN_abc","session_id":"sess_123"}'

# Response:
{"error": "Unauthorized"}
```

**Diagnosis:**
```bash
# Decode JWT (check expiration, issuer)
echo $TOKEN | cut -d. -f2 | base64 -d | jq .

# Check Keycloak is running
curl http://localhost:8081/realms/goose/.well-known/openid-configuration
```

**Solutions:**
1. **Get fresh token:** Tokens expire (default: 5 minutes)
2. **Check issuer:** JWT must be from `http://localhost:8081/realms/goose`
3. **Verify client:** Ensure `controller-api` client exists in Keycloak

---

## Security Considerations

### 1. Transport Security

**Production Deployment:**
```yaml
# Use HTTPS in production
services:
  privacy-guard:
    environment:
      GUARD_TLS_CERT: /certs/privacy-guard.crt
      GUARD_TLS_KEY: /certs/privacy-guard.key
    volumes:
      - ./certs:/certs:ro
```

**Nginx Reverse Proxy:**
```nginx
# /etc/nginx/sites-enabled/privacy-guard
server {
    listen 443 ssl;
    server_name privacy-guard.example.com;
    
    ssl_certificate /etc/letsencrypt/live/privacy-guard.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/privacy-guard.example.com/privkey.pem;
    
    location / {
        proxy_pass http://localhost:8089;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 2. Network Isolation

**Docker Network Security:**
```yaml
# Restrict Privacy Guard to internal network only
services:
  privacy-guard:
    networks:
      - internal  # No external access
    # Remove ports: 8089:8089 (no host exposure)
  
  nginx:
    ports:
      - "443:443"  # Only nginx exposed
    networks:
      - internal
      - external
```

### 3. PSEUDO_SALT Management

**Best Practices:**
- **Rotate salt quarterly** (requires re-masking all data)
- **Store in Vault:** `vault kv put secret/privacy-guard/salt value=<base64>`
- **Never commit to git:** Add to `.gitignore` and `.gooseignore`
- **Use per-tenant salts:** Different salt for each organization (prevents cross-tenant correlation)

**Vault Integration:**
```bash
# Fetch salt from Vault
export PSEUDO_SALT=$(vault kv get -field=value secret/privacy-guard/salt)

# Or in docker-compose.yml
services:
  privacy-guard:
    environment:
      PSEUDO_SALT: ${VAULT_PRIVACY_SALT}  # Injected by Vault agent
```

### 4. Audit Logging

**What is Logged:**
- Session ID
- Tenant ID
- Redaction count (per category)
- Detection mode
- Timestamp

**What is NOT Logged:**
- Original PII values ❌
- Masked text ❌
- Pseudonyms ❌
- User prompts/responses ❌

**Database Storage:**
```sql
-- privacy_audit_logs table
CREATE TABLE privacy_audit_logs (
  id SERIAL PRIMARY KEY,
  session_id VARCHAR(100),
  tenant_id VARCHAR(100),
  redaction_count INTEGER,
  categories TEXT[],  -- Array of PII types (SSN, EMAIL, etc.)
  mode VARCHAR(20),   -- Detection mode (Hybrid, RulesOnly, etc.)
  timestamp TIMESTAMP DEFAULT NOW()
);
```

**Access Control:**
```sql
-- Only compliance officers can query audit logs
GRANT SELECT ON privacy_audit_logs TO compliance_officer_role;
REVOKE ALL ON privacy_audit_logs FROM public;
```

### 5. Rate Limiting

**Prevent Abuse:**
```bash
# Nginx rate limiting (100 req/min per IP)
http {
    limit_req_zone $binary_remote_addr zone=privacy:10m rate=100r/m;
    
    server {
        location /guard/ {
            limit_req zone=privacy burst=20 nodelay;
            proxy_pass http://privacy-guard:8089;
        }
    }
}
```

**Application-Level (Future):**
```rust
// Rate limit per tenant_id (1000 req/hour)
use tower::limit::RateLimitLayer;

let app = Router::new()
    .route("/guard/mask", post(mask_handler))
    .layer(RateLimitLayer::new(1000, Duration::from_secs(3600)));
```

---

## Additional Resources

- **Decision Document:** `docs/decisions/privacy-guard-llm-integration-options.md`
- **MCP Extension Guide:** `docs/privacy/PRIVACY-GUARD-MCP.md` (alternative approach)
- **Admin Guide:** `docs/admin/ADMIN-GUIDE.md` (operational procedures)
- **OpenAPI Spec:** `docs/api/openapi-v0.5.0.yaml` (machine-readable API)
- **Test Results:** `docs/tests/phase5-test-results.md` (performance benchmarks)
- **Integration Tests:** `tests/integration/test_finance_pii_jwt.sh`

---

## Support

- **GitHub Issues:** https://github.com/JEFH507/org-chart-goose-orchestrator/issues
- **Technical Contact:** Javier (132608441+JEFH507@users.noreply.github.com)
- **License:** Apache-2.0

---

**Last Updated:** 2025-11-07  
**Version:** v0.5.0 (Grant Application Ready)
