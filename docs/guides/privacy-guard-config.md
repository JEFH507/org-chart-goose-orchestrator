# Privacy Guard Configuration Guide

Privacy Guard is configured via two YAML files: `rules.yaml` (detection patterns) and `policy.yaml` (masking behavior).

---

## Configuration Files

**Location:** `deploy/compose/guard-config/`

- **`rules.yaml`** - PII detection patterns and entity types
- **`policy.yaml`** - Masking modes, strategies, and audit settings

---

## rules.yaml Structure

### Overview

The rules file defines what PII entities to detect and how to detect them using regular expressions.

```yaml
version: "1.0"
metadata:
  author: "Your Name"
  date: "2025-11-03"
  description: "PII detection rules"

entity_types:
  ENTITY_NAME:
    display_name: "Human-readable name"
    category: "CATEGORY"  # GOVERNMENT_ID, FINANCIAL, CONTACT, IDENTITY, NETWORK
    patterns:
      - regex: 'regex pattern'
        confidence: HIGH|MEDIUM|LOW
        description: "What this pattern matches"
        context_keywords: ["optional", "keywords"]  # For MEDIUM/LOW confidence
        luhn_check: true  # Optional, for credit cards
```

### Supported Entity Types

The baseline configuration includes 8 entity types:

1. **SSN** (Social Security Number) — Category: GOVERNMENT_ID
2. **CREDIT_CARD** — Category: FINANCIAL
3. **EMAIL** — Category: CONTACT
4. **PHONE** — Category: CONTACT
5. **PERSON** (Names) — Category: IDENTITY
6. **IP_ADDRESS** — Category: NETWORK
7. **DATE_OF_BIRTH** — Category: IDENTITY
8. **ACCOUNT_NUMBER** — Category: FINANCIAL

### Confidence Levels

- **HIGH:** Very confident matches, low false positive rate (e.g., email with `@`)
- **MEDIUM:** Moderate confidence, may need context keywords (e.g., phone with country code)
- **LOW:** Ambiguous patterns, requires context keywords to avoid false positives (e.g., generic dates)

---

## Adding a New Entity Type

### Example: Add Passport Number Detection

1. **Edit `rules.yaml`:**

```yaml
entity_types:
  # ... existing types ...

  PASSPORT:
    display_name: "Passport Number"
    category: "GOVERNMENT_ID"
    patterns:
      - regex: '\b[A-Z]{1,2}\d{6,9}\b'
        confidence: MEDIUM
        description: "Generic passport format (1-2 letters + 6-9 digits)"
        context_keywords: ["passport", "PP#", "travel document", "issued"]
        examples:
          - "A1234567"
          - "AB12345678"
      
      - regex: '\b(?:Passport|PP|Travel Document):\s*([A-Z]{1,2}\d{6,9})\b'
        confidence: HIGH
        description: "Passport number with label"
        examples:
          - "Passport: A1234567"
          - "PP: AB12345678"
```

2. **Update code:** The detection engine automatically loads new types from `rules.yaml` — no code changes needed!

3. **Add masking strategy** in `policy.yaml`:

```yaml
masking:
  per_type:
    # ... existing types ...
    
    PASSPORT:
      strategy: PSEUDONYM
      format: "{type}_{hash}"
```

4. **Test the pattern:**

```bash
curl -X POST http://localhost:8089/guard/scan \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "My passport number is A1234567",
    "tenant_id": "test-org"
  }'
```

Expected: Detection of type `PASSPORT` with confidence `MEDIUM`.

---

## Regex Best Practices

### 1. Use Word Boundaries

```yaml
# Good: Uses \b to match whole words only
regex: '\b\d{3}-\d{2}-\d{4}\b'

# Bad: May match partial numbers
regex: '\d{3}-\d{2}-\d{4}'
```

### 2. Test on Diverse Samples

Create test samples covering:
- Different formats (with/without separators)
- Edge cases (minimum/maximum lengths)
- International variations
- False positives (similar but not PII)

### 3. Start Conservative (HIGH Confidence)

```yaml
# Start with specific, high-confidence patterns
- regex: '\b(?:SSN|Social Security):\s*(\d{3}-\d{2}-\d{4})\b'
  confidence: HIGH

# Add broader patterns with context keywords if needed
- regex: '\b\d{3}-\d{2}-\d{4}\b'
  confidence: MEDIUM
  context_keywords: ["social security", "SSN", "taxpayer"]
```

### 4. Use Context Keywords for Ambiguous Patterns

For patterns that might match non-PII (like generic dates or numbers), require nearby keywords:

```yaml
- regex: '\b\d{1,2}/\d{1,2}/\d{2,4}\b'
  confidence: LOW  # Generic date format
  context_keywords: ["birth", "DOB", "born", "birthday"]
  # Will only match if one of these keywords appears within ~50 characters
```

### 5. Escape Special Characters

Regex special characters must be escaped:
- `.` → `\.`
- `+` → `\+`
- `?` → `\?`
- `(` `)` → `\(` `\)`
- `[` `]` → `\[` `\]`

---

## policy.yaml Structure

### Overview

The policy file defines how detected PII should be handled.

```yaml
version: "1.0"

global:
  mode: MASK  # OFF | DETECT | MASK | STRICT
  confidence_threshold: MEDIUM  # Minimum confidence to process
  input_size_limit: 10240  # Max input size in bytes
  regex_timeout: 100  # Regex timeout in milliseconds

masking:
  default_strategy: PSEUDONYM
  
  per_type:
    ENTITY_NAME:
      strategy: PSEUDONYM | FPE | REDACT
      
      # FPE options (if strategy: FPE)
      fpe_preserve_last: 4  # For SSN, account numbers
      fpe_preserve_area_code: true  # For phone numbers
      
      # PSEUDONYM options
      format: "{type}_{hash}"  # Or "{type}_{hash}@redacted.local" for emails
      
      # REDACT options
      format: "[REDACTED_{type}]"  # Or "CARD_****_****_****_{last4}"

audit:
  log_detections: true
  log_redactions: true
  log_mapping_count: true  # Count only, not actual mappings
  log_performance: true
  log_level: info  # trace, debug, info, warn, error

session:
  ttl_minutes: 10  # How long to keep mappings in memory
  max_mappings: 10000  # Max mappings per session
  auto_flush_after_inactive: 60  # Minutes

degradation:
  missing_pseudo_salt: OFF  # Mode to use if PSEUDO_SALT not set: OFF | DETECT
  missing_rules: use_baseline  # Use hardcoded baseline rules
  slow_requests_threshold_ms: 2000  # Log warning if P95 > this
  circuit_breaker_threshold: 10  # Stop processing after N slow requests

performance:
  max_concurrent_requests: 100
  request_timeout_seconds: 5
  enable_regex_cache: true

feature_flags:
  enable_fpe: true
  enable_pseudonym: true
  enable_redaction: true
  enable_ml_detection: false  # Phase 2.2 (local NER model)
  enable_persistent_mappings: false  # Post-MVP
```

---

## Modes Explained

### OFF Mode
**Behavior:** No processing, pass-through

**Use Case:** Disable guard temporarily or for specific tenants

```bash
GUARD_MODE=OFF
# Or set in policy.yaml: mode: OFF
```

**Result:** All requests return original text unchanged.

---

### DETECT Mode
**Behavior:** Detect and log PII, but don't mask

**Use Case:** Dry-run to test patterns before enabling masking

```bash
GUARD_MODE=DETECT
```

**Example:**
```bash
curl -X POST http://localhost:8089/guard/scan \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Contact John Doe at 555-123-4567",
    "tenant_id": "test-org"
  }'
```

**Response:**
```json
{
  "detections": [
    {"start": 8, "end": 16, "type": "PERSON", "confidence": "MEDIUM"},
    {"start": 20, "end": 32, "type": "PHONE", "confidence": "HIGH"}
  ]
}
```

**Logs:** Detections logged, no masking applied.

---

### MASK Mode (Default)
**Behavior:** Detect and mask PII according to strategies

**Use Case:** Production use (mask-and-forward)

```bash
GUARD_MODE=MASK  # Default
```

**Example:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Contact John Doe at 555-123-4567",
    "tenant_id": "test-org"
  }'
```

**Response:**
```json
{
  "masked_text": "Contact PERSON_a3f7b2c8 at 555-847-9201",
  "redactions": {
    "PERSON": 1,
    "PHONE": 1
  },
  "session_id": "sess_abc123"
}
```

**Logs:** Redaction counts logged (no raw PII).

---

### STRICT Mode
**Behavior:** Block requests if ANY PII is detected

**Use Case:** Zero-tolerance environments (fail-safe)

```bash
GUARD_MODE=STRICT
```

**Example:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Contact John Doe at 555-123-4567",
    "tenant_id": "test-org"
  }'
```

**Response:**
```json
HTTP 400 Bad Request
{
  "error": "PII detected in STRICT mode: PERSON, PHONE"
}
```

**Use Case:** Prevent any PII from being processed (e.g., public-facing APIs).

---

## Masking Strategies

### PSEUDONYM Strategy
**Behavior:** Deterministic hash (HMAC-SHA256)

**Format:** `{TYPE}_{hash}` (e.g., `PERSON_a3f7b2c8`, `EMAIL_80779724`)

**Use For:**
- Names (PERSON)
- Emails (EMAIL)
- IP addresses (IP_ADDRESS)
- Dates of birth (DATE_OF_BIRTH)
- Account numbers (ACCOUNT_NUMBER)

**Preserves:** Nothing — produces opaque tokens

**Example:**
```yaml
per_type:
  EMAIL:
    strategy: PSEUDONYM
    format: "{type}_{hash}@redacted.local"
```

**Input:** `alice@example.com`  
**Output:** `EMAIL_80779724@redacted.local`

**Benefits:**
- Deterministic (same input → same output)
- Reversible with session state
- Works for any text

---

### FPE Strategy (Format-Preserving Encryption)
**Behavior:** Encrypts while preserving format and optionally partial values

**Use For:**
- Phone numbers (PHONE)
- Social Security Numbers (SSN)
- Credit card numbers (CREDIT_CARD, if needed)

**Preserves:**
- Original format (hyphens, parentheses)
- Optional partial values (area code, last 4 digits)

**Example (Phone):**
```yaml
per_type:
  PHONE:
    strategy: FPE
    fpe_preserve_area_code: true
```

**Input:** `555-123-4567`  
**Output:** `555-847-9201` (area code preserved, rest encrypted)

**Example (SSN):**
```yaml
per_type:
  SSN:
    strategy: FPE
    fpe_preserve_last: 4
```

**Input:** `123-45-6789`  
**Output:** `847-92-6789` (last 4 preserved)

**Benefits:**
- Human-readable (still looks like a phone/SSN)
- Useful for validation or UI display
- Deterministic

**Limitations:**
- Only works for numeric or alphanumeric patterns with consistent length
- Requires FPE key (derived from `PSEUDO_SALT`)

---

### REDACT Strategy
**Behavior:** Replace with placeholder

**Format:** `[REDACTED_{TYPE}]` or custom format

**Use For:**
- Credit cards (show last 4 digits only)
- Sensitive data that should never be exposed

**Example:**
```yaml
per_type:
  CREDIT_CARD:
    strategy: REDACT
    format: "CARD_****_****_****_{last4}"
```

**Input:** `4532015112830366`  
**Output:** `CARD_****_****_****_0366`

**Benefits:**
- Clear indication of redaction
- Optional partial preservation (last N digits)
- Not reversible

**Limitations:**
- Loses original information (not reversible)

---

## Tuning Confidence Threshold

The confidence threshold determines the minimum confidence level required for a detection to be processed.

```yaml
global:
  confidence_threshold: MEDIUM  # HIGH | MEDIUM | LOW
```

### HIGH Threshold
**Effect:** Only very confident matches are processed

**Use Case:** Minimize false positives, strict environments

**Example:** Only detect emails with `@` and SSNs with hyphens

**Tradeoff:** May miss some PII (lower recall)

---

### MEDIUM Threshold (Default)
**Effect:** Balance of precision and recall

**Use Case:** Most production scenarios

**Example:** Detect emails, SSNs with/without hyphens, phones in common formats

**Tradeoff:** Some false positives possible

---

### LOW Threshold
**Effect:** Catch more PII, including ambiguous patterns

**Use Case:** High-sensitivity environments, better safe than sorry

**Example:** Detect generic dates (if near "DOB"), two-word capitalized names (if near "contact")

**Tradeoff:** Higher false positive rate

---

## Tuning Workflow

**Step 1: Start in DETECT Mode**
```yaml
global:
  mode: DETECT
  confidence_threshold: MEDIUM
```

**Step 2: Review Logs**
```bash
docker compose logs privacy-guard | grep "Redaction event" | jq
```

Look for:
- False positives (detecting non-PII)
- False negatives (missing PII)
- Entity type distribution

**Step 3: Adjust Patterns or Threshold**

If too many false positives:
- Increase confidence threshold to HIGH
- Add context keywords to LOW/MEDIUM patterns
- Remove or refine overly broad patterns

If missing PII:
- Lower confidence threshold to MEDIUM or LOW
- Add more patterns for edge cases
- Review logs to find missed formats

**Step 4: Switch to MASK Mode**
```yaml
global:
  mode: MASK
```

**Step 5: Monitor and Iterate**
- Check P95 latency (should be < 1s)
- Review redaction counts per entity type
- Adjust strategies (PSEUDONYM vs FPE vs REDACT)

---

## Environment Variables

Set in `deploy/compose/.env.ce` or via Docker environment:

```bash
# Required (from Vault secret/pseudonymization:pseudo_salt)
PSEUDO_SALT=changeme_random_salt_here

# Optional (override policy.yaml)
GUARD_MODE=MASK  # OFF | DETECT | MASK | STRICT
GUARD_CONFIDENCE=MEDIUM  # HIGH | MEDIUM | LOW

# Service settings
GUARD_PORT=8089
GUARD_LOG_LEVEL=info  # trace, debug, info, warn, error
GUARD_CONFIG_PATH=/etc/guard-config

# Performance
GUARD_REQUEST_TIMEOUT=5  # Seconds
GUARD_MAX_CONCURRENT=100
```

### Precedence
1. Environment variables (highest)
2. policy.yaml settings
3. Hardcoded defaults (lowest)

---

## Model-Enhanced Detection (Phase 2.2+)

**New in Phase 2.2:** Privacy Guard can optionally use a local NER (Named Entity Recognition) model to improve detection accuracy.

### Overview

**Hybrid Detection:** Combines regex patterns (fast, high precision) with a local NER model (better recall for complex entities like person names).

**Key Features:**
- **Opt-in:** Model disabled by default (backward compatible with Phase 2)
- **Local-only:** Uses Ollama running in Docker (no cloud exposure)
- **Graceful fallback:** Falls back to regex-only if model unavailable
- **Configurable:** Choose model based on hardware constraints

**When to Use:**
- ✅ When accuracy is more important than latency (e.g., compliance audit logs)
- ✅ When detecting ambiguous PII (e.g., person names without titles)
- ✅ When hardware can handle 500-1000ms P50 latency (vs 16ms regex-only)
- ❌ When low latency is critical (high-volume APIs)
- ❌ When hardware is constrained (< 2GB available RAM)

---

### Configuration

#### Environment Variables (Model-Enhanced)

Add to `deploy/compose/.env.ce`:

```bash
# Enable model-enhanced detection (default: false)
GUARD_MODEL_ENABLED=true

# Ollama service URL (default: http://ollama:11434 for Docker Compose)
OLLAMA_URL=http://ollama:11434

# Model to use for NER (default: qwen3:0.6b)
OLLAMA_MODEL=qwen3:0.6b
```

**Precedence:** Environment variables override defaults

---

### Supported Models

**Recommended (Default):** `qwen3:0.6b`
- Size: 523MB
- Context: 40K tokens
- Hardware: Optimized for CPU-only, 8GB RAM systems
- Speed: ~500-700ms P50 latency (10-15s per request on CPU-only)
- Released: Nov 2024

**Post-MVP Alternatives (User-Selectable):**
- `gemma3:1b` (Google, 600MB, 8K context, Dec 2024)
- `phi4:3.8b-mini` (Microsoft, 2.3GB, 16K context, Dec 2024) - Best accuracy

**For more RAM:** `llama3.2:3b`, `qwen3:4b`, `gemma3:4b`

**See Also:** [guard-model-selection.md](./guard-model-selection.md) and ADR-0015

---

### How Hybrid Detection Works

**Step 1: Regex Detection** (always runs)
- Fast pattern matching (~16ms P50)
- High precision, good for structured PII (emails, SSNs, phones)
- Returns detections with confidence levels (HIGH/MEDIUM/LOW)

**Step 2: Model Detection** (if enabled and available)
- Sends text to local Ollama NER model (~500ms)
- Better at detecting unstructured PII (person names, organizations)
- Returns entity types: PERSON, EMAIL, PHONE, etc.

**Step 3: Merge Results**
- **Consensus (both methods detect):** Upgrade to HIGH confidence
- **Model-only detection:** Add as HIGH confidence
- **Regex-only detection:** Keep original confidence
- **Overlap detection:** Deduplicate when ranges overlap

**Example:**

**Input:** `"Contact Jane Smith at 555-123-4567"`

**Regex detects:**
- `555-123-4567` → PHONE (HIGH confidence)

**Model detects:**
- `Jane Smith` → PERSON (model confidence)

**Merged result:**
- `Jane Smith` → PERSON (HIGH confidence, model-only)
- `555-123-4567` → PHONE (HIGH confidence, consensus)

---

### Performance Characteristics

#### Regex-Only (GUARD_MODEL_ENABLED=false)
- **P50:** ~16ms
- **P95:** ~22ms
- **P99:** ~23ms
- **Use Case:** High-volume APIs, latency-sensitive

#### Model-Enhanced (GUARD_MODEL_ENABLED=true)
- **P50:** ~500-700ms (qwen3:0.6b)
- **P95:** ~1000ms
- **P99:** ~2000ms
- **Use Case:** Accuracy-critical, compliance audit

**Latency Breakdown (with model):**
- Regex detection: ~16ms
- Model inference (Ollama): ~450-650ms
- Result merging: ~5-10ms
- **Total:** ~500-700ms P50

**Accuracy Improvement:**
- Expected: +10-20% better recall
- Validated on Phase 2 fixtures (see Phase 2.2 smoke tests)

---

### Enabling Model-Enhanced Detection

**Step 1: Pull the Model**

First-time setup requires downloading the model:

```bash
docker compose exec ollama ollama pull qwen3:0.6b
```

**Expected output:**
```
pulling manifest
pulling 8eeb52dfb3bb... 100% ▕████████████████▏ 523 MB
pulling 966de95ca8a6... 100% ▕████████████████▏ 1.4 KB
pulling fcc5a6bec9da... 100% ▕████████████████▏ 7.7 KB
pulling a70ff7e570d9... 100% ▕████████████████▏ 6.0 KB
pulling 56bb8bd477a5... 100% ▕████████████████▏  96 B
pulling 34bb5ab01051... 100% ▕████████████████▏ 561 B
verifying sha256 digest
writing manifest
success
```

**Disk space:** ~523MB for qwen3:0.6b

---

**Step 2: Enable in Configuration**

Edit `deploy/compose/.env.ce`:

```bash
# Enable model-enhanced detection
GUARD_MODEL_ENABLED=true

# Use default model (qwen3:0.6b)
OLLAMA_URL=http://ollama:11434
OLLAMA_MODEL=qwen3:0.6b
```

---

**Step 3: Restart Privacy Guard**

```bash
docker compose restart privacy-guard
```

---

**Step 4: Verify Model Status**

Check the `/status` endpoint:

```bash
curl -s http://localhost:8089/guard/status | jq
```

**Expected output:**
```json
{
  "status": "healthy",
  "mode": "Mask",
  "rule_count": 25,
  "config_loaded": true,
  "model_enabled": true,
  "model_name": "qwen3:0.6b"
}
```

**Fields:**
- `model_enabled`: true if GUARD_MODEL_ENABLED=true
- `model_name`: Configured model (from OLLAMA_MODEL)

---

**Step 5: Test Enhanced Detection**

```bash
curl -X POST http://localhost:8089/guard/scan \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Contact Jane Smith about the proposal",
    "tenant_id": "test-org"
  }' | jq
```

**Expected (with model):**
```json
{
  "detections": [
    {
      "start": 8,
      "end": 18,
      "type": "PERSON",
      "confidence": "HIGH",
      "matched_text": "Jane Smith"
    }
  ]
}
```

**Note:** "Jane Smith" without a title (Dr., Mr., etc.) is detected by the model but might be missed by regex.

---

### Fallback Behavior

**Scenario 1: Model Disabled (GUARD_MODEL_ENABLED=false)**
- Behavior: Regex-only detection (Phase 2 baseline)
- Latency: P50 ~16ms
- No Ollama calls made

**Scenario 2: Model Enabled but Ollama Unavailable**
- Behavior: Graceful fallback to regex-only
- Latency: P50 ~16ms (no model wait)
- Warning logged: `"Ollama health check failed, using regex-only detection"`

**Scenario 3: Model Timeout (>5 seconds)**
- Behavior: Timeout after 5 seconds, return regex-only results
- Warning logged: `"Ollama request timeout, falling back to regex"`

**Result:** Service always returns results (fail-open), never blocks requests

---

### When to Use Model vs Regex-Only

#### Use Model-Enhanced (GUARD_MODEL_ENABLED=true)

✅ **Accuracy-critical scenarios:**
- Compliance audit logs (GDPR, CCPA)
- Sensitive customer data (healthcare, finance)
- Complex PII (person names, organizations)

✅ **Low-volume APIs:**
- Admin tools
- Data ingestion pipelines
- Background processing

✅ **Hardware available:**
- CPU-only: 2GB+ free RAM
- 500-1000ms P50 latency acceptable

---

#### Use Regex-Only (GUARD_MODEL_ENABLED=false)

✅ **Latency-critical scenarios:**
- High-volume APIs (>100 req/sec)
- User-facing interactive tools
- Real-time chat/messaging

✅ **Structured PII only:**
- Emails, phones, SSNs (high precision regex)
- No person names or organizations needed

✅ **Resource-constrained:**
- < 2GB available RAM
- CPU-only with limited cores
- P50 < 100ms required

---

### Troubleshooting

#### Issue: Model status shows `model_enabled: false` but env var is true

**Symptom:** `/status` endpoint shows `"model_enabled": false` even though `GUARD_MODEL_ENABLED=true`

**Solution:**
1. Check env var is set: `docker compose exec privacy-guard env | grep GUARD_MODEL`
2. Verify `.env.ce` file has correct value (no quotes needed)
3. Restart with clean env: `docker compose down && docker compose up -d`
4. Check logs for parsing errors: `docker compose logs privacy-guard | grep -i model`

---

#### Issue: Model timeout or slow responses

**Symptom:** P95 > 2000ms, timeout warnings in logs

**Solution:**
1. **Use smaller model:** Switch to `tinyllama:1.1b` or keep `qwen3:0.6b`
2. **Check CPU load:** `docker stats` (should be < 80% CPU)
3. **Reduce concurrent requests:** Lower `GUARD_MAX_CONCURRENT` to 50
4. **Disable model for high-volume endpoints:** Use `GUARD_MODEL_ENABLED=false`

---

#### Issue: Model returns incorrect entity types

**Symptom:** Model detects "ORGANIZATION" as PII (false positive)

**Solution:**
1. **Filter unmapped types:** Only PERSON, EMAIL, PHONE, SSN, etc. are mapped (see `map_ner_type()`)
2. **Tune confidence threshold:** Increase to `HIGH` to rely more on regex
3. **Review logs:** Check what the model is detecting: `docker compose logs privacy-guard | grep NER`

---

#### Issue: Ollama not responding or connection refused

**Symptom:** `"Ollama health check failed"` in logs

**Solution:**
1. **Verify Ollama is running:** `docker compose ps ollama` (should be "healthy")
2. **Check network connectivity:** `docker compose exec privacy-guard ping ollama`
3. **Verify model is pulled:** `docker compose exec ollama ollama list` (should show qwen3:0.6b)
4. **Check Ollama logs:** `docker compose logs ollama` (look for errors)
5. **Restart Ollama:** `docker compose restart ollama`

---

#### Issue: High memory usage after enabling model

**Symptom:** System RAM exhausted, OOM errors

**Solution:**
1. **Check model size:** `qwen3:0.6b` = 523MB, `llama3.2:3b` = 3GB
2. **Use smaller model:** Switch to `tinyllama:1.1b` (637MB)
3. **Monitor RAM:** `docker stats` (Ollama should be < 1GB for qwen3:0.6b)
4. **Disable model:** Set `GUARD_MODEL_ENABLED=false` if RAM < 2GB available

---

### Performance Tuning

#### Optimize for Low Latency (Hybrid Mode)

```bash
# Use smallest model
OLLAMA_MODEL=tinyllama:1.1b  # 637MB, faster

# Reduce timeout
GUARD_REQUEST_TIMEOUT=3  # Fail faster

# Lower concurrent requests
GUARD_MAX_CONCURRENT=50  # Reduce CPU contention
```

**Expected:** P50 ~300-400ms (vs 500-700ms with qwen3:0.6b)

---

#### Optimize for Accuracy

```bash
# Use larger model
OLLAMA_MODEL=llama3.2:3b  # 3GB, better NER

# Increase timeout
GUARD_REQUEST_TIMEOUT=10  # Allow more time

# Lower confidence threshold
GUARD_CONFIDENCE=MEDIUM  # Catch more detections
```

**Expected:** P50 ~800-1200ms, +5-10% better recall

---

#### Selective Model Usage (Future)

**Pattern:** Use model only for specific entity types

```yaml
# policy.yaml (future enhancement)
model:
  enabled_for_types: ["PERSON", "ORGANIZATION"]  # Only use model for these
  fallback_for_types: ["EMAIL", "PHONE", "SSN"]  # Regex-only for these
```

**Note:** Not yet implemented in Phase 2.2 (whole-request model calls only)

---

### Model Selection Decision Matrix

| Model | Size | Speed (P50) | Accuracy | Use Case |
|-------|------|-------------|----------|----------|
| **qwen3:0.6b** ✅ | 523MB | ~600ms | Good | **Default** (balanced) |
| gemma3:1b | 600MB | ~700ms | Good | Alternative (Google, Dec 2024) |
| phi4:3.8b-mini | 2.3GB | ~1100ms | Best | Quality mode (requires more RAM) |
| llama3.2:3b | 3GB | ~1100ms | Better | For 16GB+ RAM systems |

**Recommendation:** Start with `qwen3:0.6b` (default), adjust based on performance and accuracy needs.

**See Also:** [guard-model-selection.md](./guard-model-selection.md)

---

### Security Considerations (Model-Enhanced)

**No PII Sent to Cloud:**
- All model inference is local (Ollama in Docker)
- No external API calls
- No data leaves the host machine

**Model Artifact Security:**
- Models stored in Docker volumes (not committed to git)
- Downloaded on first use (explicit consent)
- Verify checksums: `ollama pull` validates SHA256

**Audit Logging:**
- Model status logged at startup
- No raw PII in logs (counts only, same as regex-only)
- Model name recorded in audit metadata

---

## Testing Your Configuration

### 1. Validate YAML Syntax
```bash
python3 -c "import yaml; yaml.safe_load(open('deploy/compose/guard-config/rules.yaml'))"
python3 -c "import yaml; yaml.safe_load(open('deploy/compose/guard-config/policy.yaml'))"
```

Expected: No errors

---

### 2. Restart Guard Service
```bash
docker compose restart privacy-guard
```

---

### 3. Check Logs for Config Load
```bash
docker compose logs privacy-guard | grep -i "config"
```

Expected:
```
INFO privacy_guard: Loaded 24 detection patterns from rules.yaml
INFO privacy_guard: Policy mode: MASK, confidence: MEDIUM
INFO privacy_guard: Config validation: OK
```

---

### 4. Test Detection (DETECT Mode)
```bash
curl -X POST http://localhost:8089/guard/scan \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Test SSN: 123-45-6789 and email test@example.com",
    "tenant_id": "test-org"
  }' | jq
```

Expected:
```json
{
  "detections": [
    {"type": "SSN", "confidence": "HIGH", "matched_text": "123-45-6789"},
    {"type": "EMAIL", "confidence": "HIGH", "matched_text": "test@example.com"}
  ]
}
```

---

### 5. Test Masking (MASK Mode)
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Test SSN: 123-45-6789 and email test@example.com",
    "tenant_id": "test-org"
  }' | jq
```

Expected:
```json
{
  "masked_text": "Test SSN: XXX-XX-6789 and email EMAIL_a1b2c3d4@redacted.local",
  "redactions": {
    "SSN": 1,
    "EMAIL": 1
  },
  "session_id": "sess_..."
}
```

---

### 6. Verify Determinism
```bash
# Call twice with same input
response1=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{"text": "Email: alice@example.com", "tenant_id": "test-org"}')

response2=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{"text": "Email: alice@example.com", "tenant_id": "test-org"}')

# Compare masked_text
echo "$response1" | jq -r .masked_text
echo "$response2" | jq -r .masked_text
# Should be identical
```

---

### 7. Check Performance
```bash
# Benchmark with 100 requests
for i in {1..100}; do
  start=$(date +%s%3N)
  curl -s -X POST http://localhost:8089/guard/mask \
    -H 'Content-Type: application/json' \
    -d '{"text": "Contact John at 555-123-4567", "tenant_id": "test"}' > /dev/null
  end=$(date +%s%3N)
  echo $((end - start))
done | sort -n | awk 'END{print "P50:", $(NR/2), "P95:", $(NR*0.95), "P99:", $(NR*0.99)}'
```

Expected: P50 < 500ms, P95 < 1000ms

---

## Common Configuration Patterns

### Pattern 1: High-Security Environment
```yaml
global:
  mode: STRICT  # Block on any PII
  confidence_threshold: MEDIUM

masking:
  per_type:
    SSN: {strategy: REDACT}
    CREDIT_CARD: {strategy: REDACT}
    PERSON: {strategy: REDACT}
    EMAIL: {strategy: REDACT}
    # Redact everything, no preservation
```

---

### Pattern 2: Development/Testing Environment
```yaml
global:
  mode: DETECT  # Dry-run only
  confidence_threshold: LOW  # Catch everything

audit:
  log_detections: true
  log_level: debug
```

---

### Pattern 3: Balanced Production
```yaml
global:
  mode: MASK
  confidence_threshold: MEDIUM

masking:
  per_type:
    SSN: {strategy: FPE, fpe_preserve_last: 4}
    PHONE: {strategy: FPE, fpe_preserve_area_code: true}
    EMAIL: {strategy: PSEUDONYM, format: "{type}_{hash}@redacted.local"}
    PERSON: {strategy: PSEUDONYM}
    CREDIT_CARD: {strategy: REDACT, format: "CARD_****_****_****_{last4}"}
```

---

### Pattern 4: Minimal Masking (Email Only)
```yaml
global:
  mode: MASK
  confidence_threshold: HIGH  # Only very confident matches

masking:
  per_type:
    EMAIL: {strategy: PSEUDONYM}
    # All others: no masking (not in per_type)
```

---

## Troubleshooting

### Issue: Rules not loading
**Symptom:** Log shows "Using default rules" or "Config validation failed"

**Solution:**
1. Check YAML syntax: `python3 -c "import yaml; yaml.safe_load(open('rules.yaml'))"`
2. Verify file is mounted: `docker compose exec privacy-guard ls -la /etc/guard-config/`
3. Check file permissions: `chmod 644 deploy/compose/guard-config/*.yaml`

---

### Issue: Pattern not detecting expected PII
**Symptom:** `/guard/scan` returns empty detections

**Solution:**
1. Test regex in isolation: `echo "test@example.com" | grep -oP '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'`
2. Check confidence threshold: Lower to `LOW` temporarily
3. Add context keywords if pattern is ambiguous
4. Review logs for regex errors

---

### Issue: Too many false positives
**Symptom:** Generic words or dates flagged as PII

**Solution:**
1. Increase confidence threshold to `HIGH`
2. Add context keywords to `LOW`/`MEDIUM` patterns
3. Remove overly broad patterns
4. Use DETECT mode to review before masking

---

### Issue: FPE format not preserved
**Symptom:** Phone/SSN output format is wrong

**Solution:**
1. Verify input format matches expected: `555-123-4567` (hyphens)
2. Check FPE key is set: `echo $PSEUDO_SALT`
3. Review logs for FPE errors
4. Fallback to PSEUDONYM if FPE fails

---

## Security Considerations

### Never Log Raw PII
All audit logs contain **counts only**, no raw text or pseudonyms.

**Correct:**
```json
{"entity_counts": {"EMAIL": 1, "PHONE": 2}, "total_redactions": 3}
```

**Incorrect (never do this):**
```json
{"detected": ["alice@example.com", "555-123-4567"]}  // NEVER!
```

---

### PSEUDO_SALT Security
- Store in Vault (`secret/pseudonymization:pseudo_salt`)
- Rotate periodically (invalidates old mappings)
- Never commit to git
- Unique per tenant in multi-tenant setups (Phase 3+)

---

### Reidentification Controls
- Requires JWT authentication (from OIDC)
- Limited to session-scoped mappings (in-memory, TTL)
- Audit all reidentification requests
- Consider RBAC for reidentify endpoint (Phase 3+)

---

## References

- **ADR-0021:** Privacy Guard Rust Implementation
- **ADR-0022:** PII Detection Rules and FPE
- **Integration Guide:** [privacy-guard-integration.md](./privacy-guard-integration.md)
- **Smoke Tests:** [smoke-phase2.md](../tests/smoke-phase2.md)
- **Architecture:** [mvp.md](../architecture/mvp.md)

---

## Appendix: Complete Example Configuration

**`deploy/compose/guard-config/rules.yaml`:**
```yaml
version: "1.0"
entity_types:
  EMAIL:
    patterns:
      - regex: '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
        confidence: HIGH
  
  PHONE:
    patterns:
      - regex: '\b\d{3}-\d{3}-\d{4}\b'
        confidence: HIGH
      - regex: '\(\d{3}\)\s*\d{3}-\d{4}'
        confidence: HIGH
  
  # ... (see deployed rules.yaml for full config)
```

**`deploy/compose/guard-config/policy.yaml`:**
```yaml
version: "1.0"
global:
  mode: MASK
  confidence_threshold: MEDIUM

masking:
  per_type:
    EMAIL: {strategy: PSEUDONYM, format: "{type}_{hash}@redacted.local"}
    PHONE: {strategy: FPE, fpe_preserve_area_code: true}
    SSN: {strategy: FPE, fpe_preserve_last: 4}
    CREDIT_CARD: {strategy: REDACT, format: "CARD_****_****_****_{last4}"}

audit:
  log_detections: true
  log_redactions: true
  log_performance: true
```

---

**Last Updated:** 2025-11-04 (Phase 2.2 - Model-Enhanced Detection)  
**Author:** Phase 2 Team, Phase 2.2 Team  
**Version:** 1.1
