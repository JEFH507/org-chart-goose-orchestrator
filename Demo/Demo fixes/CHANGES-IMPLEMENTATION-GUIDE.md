# Privacy Guard Enhancements - Implementation Guide

**Date**: 2025-11-16  
**Changes**: Credit card pattern support + Masked text logging  
**Rebuild Required**: YES (Privacy Guard services only)

---

## Summary of Changes

### 1. Credit Card Patterns with Separators
**File**: `src/privacy-guard/src/detection.rs`  
**What**: Added 4 new credit card patterns to support hyphens and spaces  
**Before**: Only `4532015112830366` (continuous digits) ❌ `4532-1234-5678-9012`  
**After**: Both formats supported ✅

### 2. Masked Text Logging
**File**: `src/privacy-guard/src/main.rs`  
**What**: Added INFO-level logging of exact masked payload sent to LLM  
**Purpose**: See verbatim what Privacy Guard sends (e.g., `"My SSN is [SSN]"`)

---

## Step-by-Step Implementation

### STEP 1: Stop Privacy Guard Services

**Why**: Need to rebuild Rust code changes

```bash
# Stop only Privacy Guard services (NOT Goose containers)
docker stop ce_privacy_guard_finance ce_privacy_guard_manager ce_privacy_guard_legal
```

**Status**: ✅ Other services (Goose, Controller, Ollama) keep running

---

### STEP 2: Rebuild Privacy Guard Services

**Why**: Compile new Rust code with pattern changes + logging

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Rebuild all 3 Privacy Guard services
docker compose -f deploy/compose/ce.dev.yml build \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal
```

**Expected Output**:
```
[+] Building 45.2s (12/12) FINISHED
 => [privacy-guard-finance internal] load build definition
 => => transferring dockerfile: 1.23kB
 ...
 => exporting to image
 => => writing image sha256:abc123...
```

**Time**: ~30-60 seconds (Rust compilation)

---

### STEP 3: Restart Privacy Guard Services

```bash
# Start the rebuilt services
docker compose -f deploy/compose/ce.dev.yml up -d \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal
```

**Verification**:
```bash
# Check services are running
docker ps | grep privacy_guard

# Should show 3 containers:
# ce_privacy_guard_finance
# ce_privacy_guard_manager
# ce_privacy_guard_legal
```

---

### STEP 4: Verify New Pattern Count

**Check rule count increased from 22 to 26**:

```bash
curl -s http://localhost:8093/status | jq
```

**Expected Output**:
```json
{
  "status": "healthy",
  "mode": "Mask",
  "rule_count": 26,  // <-- Was 22, now 26 (+4 new credit card patterns)
  "config_loaded": true,
  "model_enabled": false,
  "model_name": "qwen3:0.6b"
}
```

**Status**: ✅ If you see `rule_count: 26`, patterns loaded successfully

---

### STEP 5: Test Credit Card with Hyphens

**Terminal 1** - Watch live logs:
```bash
docker logs -f ce_privacy_guard_finance 2>&1 | grep --line-buffered "Masked payload"
```

**Terminal 2** - Send test message:
```bash
docker exec -it ce_goose_finance goose session
```

In Goose, type:
```
Test: Card 4532-1234-5678-9012 with hyphens
```

**Expected in Terminal 1**:
```
Masked payload: Test: Card [CREDIT_CARD] with hyphens
```

**Status**: ✅ If you see `[CREDIT_CARD]` token, hyphenated pattern working!

---

### STEP 6: Test All Credit Card Formats

**Test these formats** (one at a time):

1. **Hyphens**: `4532-1234-5678-9012` → Should detect ✅
2. **Spaces**: `4532 1234 5678 9012` → Should detect ✅
3. **No separators**: `4532015112830366` → Should detect ✅ (original pattern)
4. **Mixed**: `4532-1234 5678-9012` → Should detect ✅
5. **Invalid Luhn**: `4532-1234-5678-9999` → Should NOT detect ❌ (Luhn fails)

**Verification Command** (after each test):
```bash
docker logs ce_privacy_guard_finance | grep audit | tail -1 | jq '.entity_counts'
```

**Expected**: `{"CREDIT_CARD":1}`

---

### STEP 7: Verify Masked Text Logging

**Command**:
```bash
docker logs ce_privacy_guard_finance 2>&1 | grep "Masked payload" | tail -3
```

**Expected Output**:
```
INFO Masked payload: Test: Card [CREDIT_CARD] with hyphens
INFO Masked payload: Contact [EMAIL] for details
INFO Masked payload: My SSN is [SSN]
```

**What you see**:
- **Original message**: `My SSN is 123-45-6789`
- **Masked payload**: `My SSN is [SSN]` ← This is what LLM receives

---

## Part 3: Verify UI Detection Mode Switching

### Your Question: "Changed UI to AI mode but saw no difference"

**How to verify detection method is actually working**:

### TEST 1: Check Proxy Settings (UI vs Reality)

```bash
# Check current settings in Privacy Guard Proxy
curl -s http://localhost:8096/api/settings | jq
```

**Expected Output**:
```json
{
  "routing": "service",
  "detection": "ai",      // <-- Should match UI
  "privacy": "auto"
}
```

**Status**: ✅ If `detection` matches UI selection, settings saved correctly

---

### TEST 2: Check Privacy Guard Logs for Detection Method

**Command**:
```bash
docker logs -f ce_privacy_guard_finance 2>&1 | grep --line-buffered "detection"
```

**Send test message** (in Goose):
```
Test message with alice@company.com
```

**Expected Log**:
```
INFO Using hybrid/AI detection (balanced ~100ms or accurate ~15s)
```

**vs Rules-only**:
```
INFO Using rules-only detection (fast ~10ms)
```

**Status**: ✅ If you see "hybrid/AI detection", UI change took effect

---

### TEST 3: Check Ollama Logs (AI Mode Only)

**When AI mode enabled, Ollama should receive requests**:

```bash
# Watch Ollama Finance logs
docker logs -f ce_ollama_finance 2>&1 | grep --line-buffered "POST"
```

**Expected** (when detection=ai or hybrid):
```
[GIN] 2025/11/16 - 02:30:15 | 200 |  15.234s | POST /api/generate
```

**Not Expected** (when detection=rules):
- No Ollama activity (regex-only)

---

### TEST 4: Performance Difference (Proof of Mode Change)

**Rules Mode** (~10ms):
```bash
time docker exec ce_goose_finance goose session <<< "Test alice@company.com"
```
Output: `real    0m0.150s` (fast)

**AI/Hybrid Mode** (~15s or ~100ms):
```bash
time docker exec ce_goose_finance goose session <<< "Test alice@company.com"
```
Output: `real    0m15.234s` (slow - AI model processing)

**Status**: ✅ If AI mode is 100x slower, it's actually using Ollama

---

## Troubleshooting

### Issue: UI Changed But Logs Still Say "rules-only"

**Diagnosis**:
1. **Check proxy received change**:
   ```bash
   docker logs ce_privacy_guard_proxy_finance | grep detection_method_change
   ```
   Expected: `detection_method_change ... changed to: ai`

2. **Check Goose actually connected to proxy**:
   ```bash
   docker exec ce_goose_finance cat /root/.config/goose/config.yaml | grep OPENROUTER_HOST
   ```
   Expected: `OPENROUTER_HOST: http://privacy-guard-proxy-finance:8090`

3. **Restart Goose container** (should NOT be needed, but try):
   ```bash
   docker restart ce_goose_finance
   ```

---

### Issue: Credit Card with Hyphens NOT Detected

**Diagnosis**:
1. **Check rule count**:
   ```bash
   curl -s http://localhost:8093/status | jq .rule_count
   ```
   Expected: `26` (not 22)

2. **Check Luhn validation** (must be valid card):
   ```python3
   def luhn(n):
       digits = [int(d) for d in str(n).replace('-','').replace(' ','')]
       checksum = 0
       for i, digit in enumerate(reversed(digits)):
           if i % 2 == 1:
               digit *= 2
               if digit > 9: digit -= 9
           checksum += digit
       return checksum % 10 == 0
   
   print(luhn("4532-1234-5678-9012"))  # Must be True
   ```

3. **Use valid test card**:
   ```
   4532-0151-1283-0366  ✅ Valid Visa
   5425-2334-3010-9903  ✅ Valid Mastercard
   3782-822463-10005    ✅ Valid Amex
   ```

---

### Issue: No "Masked payload" in Logs

**Diagnosis**:
1. **Check log level**:
   ```bash
   docker exec ce_privacy_guard_finance env | grep RUST_LOG
   ```
   Expected: Empty or `RUST_LOG=info` (default)

2. **Check service is masking** (not bypassed):
   ```bash
   curl -s http://localhost:8096/api/settings | jq .privacy
   ```
   Expected: `"auto"` or `"strict"` (NOT "service-bypass")

3. **Force log output**:
   ```bash
   # Restart with explicit log level
   docker stop ce_privacy_guard_finance
   docker run -d --name ce_privacy_guard_finance \
     -e RUST_LOG=info \
     $(docker inspect ce_privacy_guard_finance -f '{{range .Config.Env}}{{println .}}{{end}}' | grep -v RUST_LOG) \
     ...
   ```

---

## Verification Checklist

After completing all steps, verify:

- [ ] **Rule count = 26** (was 22)
- [ ] **Credit card with hyphens detected**: `4532-1234-5678-9012` → `[CREDIT_CARD]`
- [ ] **Credit card with spaces detected**: `4532 1234 5678 9012` → `[CREDIT_CARD]`
- [ ] **Masked payload logged**: See `INFO Masked payload: ...` in logs
- [ ] **UI detection mode changes work**: Switch rules→AI, see "hybrid/AI detection" in logs
- [ ] **Ollama called in AI mode**: See POST requests in Ollama logs
- [ ] **Performance difference**: AI mode ~100x slower than rules mode

---

## Quick Test Script

Save as `test-privacy-guard-enhancements.sh`:

```bash
#!/bin/bash
set -e

echo "=== Privacy Guard Enhancement Tests ==="

# Test 1: Rule count
echo -e "\n1. Checking rule count (expect 26)..."
RULES=$(curl -s http://localhost:8093/status | jq -r .rule_count)
if [ "$RULES" -eq 26 ]; then
  echo "   ✅ Rule count: $RULES"
else
  echo "   ❌ Rule count: $RULES (expected 26)"
fi

# Test 2: Proxy settings
echo -e "\n2. Checking proxy settings..."
DETECTION=$(curl -s http://localhost:8096/api/settings | jq -r .detection)
echo "   Current detection method: $DETECTION"

# Test 3: Live log monitoring
echo -e "\n3. Monitoring masked payload logs (send test message now)..."
echo "   Run in Goose: Test card 4532-1234-5678-9012"
timeout 30 docker logs -f ce_privacy_guard_finance 2>&1 | grep --line-buffered "Masked payload" | head -1 || echo "   (timeout - no logs in 30s)"

# Test 4: Audit log verification
echo -e "\n4. Checking last audit event..."
docker logs ce_privacy_guard_finance | grep audit | tail -1 | jq -C '.entity_counts'

echo -e "\n=== Tests Complete ==="
```

Run it:
```bash
chmod +x test-privacy-guard-enhancements.sh
./test-privacy-guard-enhancements.sh
```

---

## Rollback (If Needed)

If changes cause issues:

```bash
# Revert code changes
cd /home/papadoc/Gooseprojects/goose-org-twin
git checkout src/privacy-guard/src/detection.rs
git checkout src/privacy-guard/src/main.rs

# Rebuild with old code
docker compose -f deploy/compose/ce.dev.yml build \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal

# Restart services
docker compose -f deploy/compose/ce.dev.yml up -d \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal

# Verify rule count back to 22
curl -s http://localhost:8093/status | jq .rule_count
# Expected: 22
```

---

## Summary: What Changed & What Needs Rebuild

| Component | Changed? | Rebuild Needed? | Restart Needed? |
|-----------|----------|-----------------|-----------------|
| **Privacy Guard Service** | ✅ Yes (detection.rs, main.rs) | ✅ YES | ✅ YES |
| **Privacy Guard Proxy** | ❌ No | ❌ NO | ❌ NO |
| **Goose Containers** | ❌ No | ❌ NO | ❌ NO |
| **Controller** | ❌ No | ❌ NO | ❌ NO |
| **Ollama** | ❌ No | ❌ NO | ❌ NO |

**Total Downtime**: ~2 minutes (only Privacy Guard services)
**Goose Sessions**: Continue working (no restart needed)

---

## References

- **Pattern Reference**: `Demo/Privacy-Guard-Pattern-Reference.md`
- **Source Files**:
  - `src/privacy-guard/src/detection.rs` (lines 161-205 - credit card patterns)
  - `src/privacy-guard/src/main.rs` (line ~320 - masked payload logging)
- **Docker Compose**: `deploy/compose/ce.dev.yml`
