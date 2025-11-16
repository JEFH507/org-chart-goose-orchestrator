# 🚨 ISSUE-1: Privacy Guard Root Cause Analysis

**Date**: 2025-11-15  
**Status**: ROOT CAUSE IDENTIFIED  
**Severity**: HIGH - Privacy Guard completely bypassed  
**User Assessment**: ✅ CORRECT - "No information flowing through Privacy Guard"  
**AI Assessment**: ❌ INCORRECT - Misread old logs as current activity

---

## Investigation Summary

### User Observation
> "All calls going directly to the LLM. Privacy Guard proxy logs show no traffic."

### Initial AI Assessment (WRONG)
> "Privacy Guard IS working - Evidence shows redactions in logs"

**AI Mistake**: Found an old log entry from a previous curl test, mistakenly thought it was from Goose CLI session.

### User Validation Results

**Test performed**: Sent PII message through Goose Finance CLI
```bash
docker exec -it ce_goose_finance goose session
# Sent: "Process employee alice@company.com with SSN 123-45-6789"
```

**Results**:
1. ✅ Goose session worked
2. ✅ Message processed by LLM
3. ❌ **Privacy Guard Service logs: EMPTY**
4. ❌ **Privacy Guard Proxy logs: EMPTY** (only startup messages, no `/v1/chat/completions`)

**Conclusion**: **Requests are NOT going through Privacy Guard at all**

---

## Root Cause Analysis

### Problem: `api_base` in config.yaml is IGNORED by Goose

**Generated config has**:
```yaml
provider: openrouter
model: anthropic/claude-3.5-sonnet
api_base: http://privacy-guard-proxy-finance:8090/v1  # ← IGNORED!
```

**Why it's ignored**:
- Goose v1.14.0 does **NOT** support `api_base` parameter in config.yaml
- This is a custom parameter we invented, not part of Goose's configuration schema

### What Goose Actually Uses

**OpenRouter provider code** (`openrouter.rs` line 53):
```rust
let host: String = config
    .get_param("OPENROUTER_HOST")  // ← This is the parameter name
    .unwrap_or_else(|_| "https://openrouter.ai".to_string());
```

**Goose looks for**:
- Environment variable: `OPENROUTER_HOST`
- Config parameter: `OPENROUTER_HOST` (via `Config::global().get_param()`)

**We provided**:
- ❌ `api_base` in config.yaml (not recognized)
- ❌ No `OPENROUTER_HOST` environment variable

**Result**: Goose uses default `https://openrouter.ai` → bypasses Privacy Guard entirely

---

## The Fix

### Option 1: Environment Variable (RECOMMENDED)

**File**: `deploy/compose/ce.dev.yml`

**Add to each Goose container**:
```yaml
  goose-finance:
    environment:
      - OPENROUTER_HOST=http://privacy-guard-proxy-finance:8090  # ← ADD THIS
      - OPENROUTER_API_KEY=${OPENROUTER_API_KEY}
      # ... other env vars ...
```

**Do for all 3 containers**:
- `goose-finance` → `http://privacy-guard-proxy-finance:8090`
- `goose-manager` → `http://privacy-guard-proxy-manager:8090`
- `goose-legal` → `http://privacy-guard-proxy-legal:8090`

### Option 2: Fix Config Generator (ALTERNATIVE)

**File**: `docker/goose/generate-goose-config.py`

**Change line 51**:
```python
# OLD (wrong):
"api_base": f"{proxy_url}/v1",

# NEW (correct):
"OPENROUTER_HOST": proxy_url,  # No /v1 suffix!
```

**Then rebuild containers**

---

## Testing the Fix

### Step 1: Add OPENROUTER_HOST to docker-compose

**Modify**: `/home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/ce.dev.yml`

**Add environment variable** to `goose-finance` service (around line 600):
```yaml
  goose-finance:
    build:
      context: ../..
      dockerfile: docker/goose/Dockerfile
    image: goose-test:0.5.3
    container_name: ce_goose_finance
    environment:
      - GOOSE_ROLE=finance
      - CONTROLLER_URL=http://controller:8088
      - KEYCLOAK_URL=http://host.docker.internal:8080
      - KEYCLOAK_REALM=dev
      - KEYCLOAK_CLIENT_ID=goose-controller
      - KEYCLOAK_CLIENT_SECRET=${OIDC_CLIENT_SECRET}
      - OPENROUTER_API_KEY=${OPENROUTER_API_KEY}
      - OPENROUTER_HOST=http://privacy-guard-proxy-finance:8090  # ← ADD THIS LINE
      - PRIVACY_GUARD_PROXY_URL=http://privacy-guard-proxy-finance:8090
      - GOOSE_PROVIDER=${GOOSE_PROVIDER:-openrouter}
      - GOOSE_MODEL=${GOOSE_MODEL:-anthropic/claude-3.5-sonnet}
```

Repeat for `goose-manager` and `goose-legal` with their respective proxy URLs.

### Step 2: Restart Goose Containers

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Restart only Goose containers (no rebuild needed)
docker compose -f ce.dev.yml --profile multi-goose restart \
  goose-finance goose-manager goose-legal

# Wait for restart
sleep 10
```

### Step 3: Verify OPENROUTER_HOST is Set

```bash
# Check environment variable is set
docker exec ce_goose_finance env | grep OPENROUTER_HOST

# Expected output:
# OPENROUTER_HOST=http://privacy-guard-proxy-finance:8090
```

### Step 4: Test Privacy Guard with PII

```bash
# Start Goose session
docker exec -it ce_goose_finance goose session

# Send message with PII
"Process employee data: alice@company.com, SSN 123-45-6789, Phone 555-1234"

# Exit session
exit
```

### Step 5: Check Privacy Guard Logs

```bash
# Check Privacy Guard Service logs
docker logs ce_privacy_guard_finance 2>&1 | grep -E "(audit|entity_counts|redaction)"

# Expected output (should NOT be empty):
# {"timestamp":"...", "entity_counts":{"EMAIL":1,"SSN":1,"PHONE":1}, "total_redactions":3}

# Check Privacy Guard Proxy logs
docker logs ce_privacy_guard_proxy_finance 2>&1 | tail -20

# Expected output (should show /v1/chat/completions requests):
# POST /v1/chat/completions
# Forwarding to Privacy Guard Service...
```

---

## Expected Results After Fix

### ✅ Success Indicators

1. **Privacy Guard Service logs show**:
   ```json
   {
     "timestamp": "2025-11-15T...",
     "entity_counts": {"EMAIL": 1, "SSN": 1, "PHONE": 1},
     "total_redactions": 3,
     "performance_ms": 8
   }
   ```

2. **Privacy Guard Proxy logs show**:
   ```
   POST /v1/chat/completions - 200 OK
   Masked request forwarded to OpenRouter
   ```

3. **Goose still works normally** - No breaking changes

4. **LLM receives masked data** - `[EMAIL]`, `[SSN]`, `[PHONE]` instead of actual PII

---

## Why This Happened

### Design Assumption (WRONG)

**We assumed**: Goose supports `api_base` config parameter like OpenAI client libraries

**Reality**: Goose uses provider-specific parameters:
- OpenRouter: `OPENROUTER_HOST`
- OpenAI: `OPENAI_HOST`
- Anthropic: `ANTHROPIC_HOST`

### Documentation Gap

**Goose config docs** (config-files.md) does NOT mention `api_base` as supported parameter.

**We should have checked**:
1. Goose provider source code (`openrouter.rs`)
2. Goose configuration schema
3. Environment variable documentation

### Testing Gap

**We tested**:
- ✅ Config generation (file created correctly)
- ✅ Goose startup (no errors)

**We didn't test**:
- ❌ Privacy Guard logs (would have caught the bypass immediately)
- ❌ Network traffic inspection (would have shown direct OpenRouter calls)

---

## Lessons Learned

1. **Never assume configuration parameters** - Always check source code
2. **Test the actual data flow** - Not just "does it work"
3. **Check logs at every layer** - Privacy Guard, Proxy, LLM
4. **User observations are valuable** - "No logs" is a red flag

---

## Fix Priority

**Priority**: 🔥 **CRITICAL**

**Why**:
- Privacy Guard is a core feature for demo
- Currently providing NO protection at all
- PII is being sent to OpenRouter unredacted
- This defeats the entire purpose of the Privacy Guard architecture

**Recommendation**: **Fix IMMEDIATELY before any further testing**

---

## Status: Awaiting User Decision

**Next Steps**:
1. User reviews this analysis
2. User confirms fix approach (env var vs config generator)
3. User makes the modification
4. User tests the fix
5. User reports results

**AI Role**: Guide user through fix implementation step-by-step

---

**Analysis Complete** ✅  
**Root Cause**: Confirmed - `api_base` not supported by Goose  
**Fix**: Add `OPENROUTER_HOST` environment variable  
**Testing**: Ready to proceed when user authorizes
