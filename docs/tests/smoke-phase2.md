# Phase 2 Smoke Tests — Privacy Guard

Manual validation checklist for Privacy Guard functionality.

**Version:** 1.0  
**Last Updated:** 2025-11-03  
**Phase:** Phase 2 - Privacy Guard

---

## Prerequisites

- Docker and Docker Compose installed
- Repository cloned: `/home/papadoc/Gooseprojects/goose-org-twin`
- Vault running with `PSEUDO_SALT` set (from Phase 1.2)
- Basic understanding of REST APIs and curl

---

## Setup

### 1. Navigate to compose directory

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
```

### 2. Verify environment configuration

```bash
# Check that PSEUDO_SALT is set
grep PSEUDO_SALT .env.ce
# Should show: PSEUDO_SALT=<value>

# If not set, retrieve from Vault (Phase 1.2)
# See: docs/tests/smoke-phase1.2.md for Vault access
```

### 3. Start services with privacy-guard profile

```bash
docker compose --profile privacy-guard up -d
```

### 4. Wait for healthchecks

```bash
# Check service status
docker compose ps

# All services should show "healthy" status
# Specifically look for:
# - vault: healthy
# - privacy-guard: healthy
# - controller: healthy (if running)
```

### 5. Check logs for startup

```bash
# Verify guard started successfully
docker compose logs privacy-guard | grep "listening"
# Should see: "Privacy Guard listening on 0.0.0.0:8089"

# Check for config loading
docker compose logs privacy-guard | grep "config"
# Should see rule count and policy loaded
```

---

## Test Suite

### Test 1: Healthcheck

**Objective:** Verify guard service is running and configured

**Command:**
```bash
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

**Pass Criteria:**
- ✅ HTTP 200 status
- ✅ All fields present
- ✅ `config_loaded: true`
- ✅ `rule_count > 0` (should be ~24 patterns)
- ✅ `mode: "Mask"` (default)

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 2: PII Detection (Scan Endpoint)

**Objective:** Detect PII without masking (dry-run mode)

**Command:**
```bash
curl -X POST http://localhost:8089/guard/scan \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789",
    "tenant_id": "test-org"
  }'
```

**Expected Response:**
```json
{
  "detections": [
    {
      "start": 8,
      "end": 16,
      "entity_type": "PERSON",
      "confidence": "MEDIUM",
      "matched_text": "John Doe"
    },
    {
      "start": 20,
      "end": 32,
      "entity_type": "PHONE",
      "confidence": "HIGH",
      "matched_text": "555-123-4567"
    },
    {
      "start": 36,
      "end": 56,
      "entity_type": "EMAIL",
      "confidence": "HIGH",
      "matched_text": "john.doe@example.com"
    },
    {
      "start": 63,
      "end": 74,
      "entity_type": "SSN",
      "confidence": "HIGH",
      "matched_text": "123-45-6789"
    }
  ]
}
```

**Pass Criteria:**
- ✅ HTTP 200 status
- ✅ 4 detections found (PERSON, PHONE, EMAIL, SSN)
- ✅ Entity types correct
- ✅ Confidence levels appropriate (PHONE/EMAIL/SSN: HIGH, PERSON: MEDIUM)
- ✅ Start/end positions accurate

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 3: Masking with Pseudonyms

**Objective:** Mask PII using PSEUDONYM strategy

**Command:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Email sent to alice.smith@example.com from 192.168.1.100",
    "tenant_id": "test-org"
  }'
```

**Expected Response:**
```json
{
  "masked_text": "Email sent to EMAIL_<8_hex_chars> from IP_ADDRESS_<8_hex_chars>",
  "redactions": {
    "EMAIL": 1,
    "IP_ADDRESS": 1
  },
  "session_id": "sess_<uuid>"
}
```

**Pass Criteria:**
- ✅ HTTP 200 status
- ✅ EMAIL masked with format `EMAIL_<hash>`
- ✅ IP_ADDRESS masked with format `IP_ADDRESS_<hash>`
- ✅ Hash is 8 hex characters
- ✅ Redaction counts correct (`EMAIL: 1, IP_ADDRESS: 1`)
- ✅ Session ID returned (UUID format)
- ✅ Original PII not in masked_text

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 4: Format-Preserving Encryption (Phone)

**Objective:** Verify FPE preserves phone number format

**Command:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Call me at 555-123-4567",
    "tenant_id": "test-org"
  }'
```

**Expected Response:**
```json
{
  "masked_text": "Call me at 555-XXX-XXXX",
  "redactions": {
    "PHONE": 1
  },
  "session_id": "sess_<uuid>"
}
```

**Pass Criteria:**
- ✅ HTTP 200 status
- ✅ Phone format preserved: `XXX-XXX-XXXX`
- ✅ Area code preserved: `555-`
- ✅ Last 7 digits encrypted (different from `123-4567`)
- ✅ Output is valid phone format
- ✅ Redaction count: `PHONE: 1`

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 5: Format-Preserving Encryption (SSN)

**Objective:** Verify FPE preserves SSN format and last 4 digits

**Command:**
```bash
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "SSN: 123-45-6789",
    "tenant_id": "test-org"
  }'
```

**Expected Response:**
```json
{
  "masked_text": "SSN: XXX-XX-6789",
  "redactions": {
    "SSN": 1
  },
  "session_id": "sess_<uuid>"
}
```

**Pass Criteria:**
- ✅ HTTP 200 status
- ✅ SSN format preserved: `XXX-XX-XXXX`
- ✅ Last 4 digits preserved: `6789`
- ✅ First 5 digits encrypted (different from `123-45`)
- ✅ Output is valid SSN format
- ✅ Redaction count: `SSN: 1`

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 6: Determinism (Same Input → Same Output)

**Objective:** Verify same input produces identical pseudonyms per tenant

**Commands:**
```bash
# Call 1
response1=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Email: test@example.com",
    "tenant_id": "test-org"
  }')

# Call 2 (same input, same tenant)
response2=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Email: test@example.com",
    "tenant_id": "test-org"
  }')

# Compare outputs
echo "Response 1:"
echo "$response1" | jq .masked_text

echo "Response 2:"
echo "$response2" | jq .masked_text

# Check if identical
if [ "$(echo "$response1" | jq -r .masked_text)" = "$(echo "$response2" | jq -r .masked_text)" ]; then
  echo "✅ DETERMINISTIC: Same masked_text"
else
  echo "❌ FAILED: Different masked_text"
fi
```

**Expected Output:**
```
Response 1:
"Email: EMAIL_<hash>"

Response 2:
"Email: EMAIL_<hash>"

✅ DETERMINISTIC: Same masked_text
```

**Pass Criteria:**
- ✅ Both calls return identical `masked_text`
- ✅ Pseudonyms are deterministic (same hash)
- ✅ Session IDs may differ (sessions are independent)

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 7: Tenant Isolation

**Objective:** Verify different tenants get different pseudonyms for same PII

**Commands:**
```bash
# Tenant 1
response_tenant1=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Email: shared@example.com",
    "tenant_id": "tenant-1"
  }')

# Tenant 2 (same PII, different tenant)
response_tenant2=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Email: shared@example.com",
    "tenant_id": "tenant-2"
  }')

# Compare
echo "Tenant 1:"
echo "$response_tenant1" | jq .masked_text

echo "Tenant 2:"
echo "$response_tenant2" | jq .masked_text

# Check if different
if [ "$(echo "$response_tenant1" | jq -r .masked_text)" != "$(echo "$response_tenant2" | jq -r .masked_text)" ]; then
  echo "✅ ISOLATED: Different pseudonyms per tenant"
else
  echo "❌ FAILED: Same pseudonym across tenants"
fi
```

**Expected Output:**
```
Tenant 1:
"Email: EMAIL_<hash1>"

Tenant 2:
"Email: EMAIL_<hash2>"

✅ ISOLATED: Different pseudonyms per tenant
```

**Pass Criteria:**
- ✅ Same PII produces different pseudonyms for different tenants
- ✅ Hashes are different (`<hash1>` ≠ `<hash2>`)
- ✅ Tenant isolation working correctly

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 8: Reidentification (with JWT)

**Objective:** Reverse pseudonym to original value (requires JWT authentication)

**Prerequisites:**
- Valid JWT from Keycloak (see Phase 1.2 smoke tests)
- Session ID from previous mask call

**Commands:**
```bash
# Step 1: Mask some text and capture session_id and pseudonym
mask_response=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Contact Jane Doe",
    "tenant_id": "test-org"
  }')

session_id=$(echo "$mask_response" | jq -r .session_id)
masked_text=$(echo "$mask_response" | jq -r .masked_text)

echo "Masked text: $masked_text"
echo "Session ID: $session_id"

# Extract pseudonym (e.g., PERSON_a3f7b2c8)
pseudonym=$(echo "$masked_text" | grep -oP 'PERSON_[a-f0-9]{8}')
echo "Pseudonym: $pseudonym"

# Step 2: Get JWT (from Phase 1.2 - replace with actual JWT)
JWT="<your-jwt-token-here>"

# Step 3: Reidentify
curl -X POST http://localhost:8089/guard/reidentify \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $JWT" \
  -d "{
    \"pseudonym\": \"$pseudonym\",
    \"session_id\": \"$session_id\"
  }"
```

**Expected Response (with valid JWT):**
```json
{
  "original": "Jane Doe"
}
```

**Expected Response (without JWT or invalid JWT):**
```json
{
  "error": "Unauthorized"
}
```
HTTP 401 status

**Pass Criteria:**
- ✅ With valid JWT: HTTP 200, original value returned
- ✅ Without JWT: HTTP 401 Unauthorized
- ✅ Invalid JWT: HTTP 401 Unauthorized
- ✅ Wrong session_id: HTTP 404 Not Found

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 9: Audit Logs (No PII)

**Objective:** Verify logs contain only counts and metadata, not raw PII

**Commands:**
```bash
# Run a mask request with known PII
curl -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Secret: My SSN is 987-65-4321 and email is secret@hidden.com",
    "tenant_id": "test-org"
  }'

# Check guard logs (last 30 lines)
docker compose logs privacy-guard --tail 30
```

**Expected Log Entry (structured JSON in audit target):**
```json
{
  "timestamp": "2025-11-03T...",
  "tenant_id": "test-org",
  "session_id": "sess_...",
  "mode": "Mask",
  "entity_counts": {
    "SSN": 1,
    "EMAIL": 1
  },
  "total_redactions": 2,
  "performance_ms": 45,
  "trace_id": null
}
```

**Pass Criteria:**
- ✅ Log contains `entity_counts` with counts only
- ✅ Log contains `total_redactions`
- ✅ Log contains `performance_ms`
- ✅ Log DOES NOT contain "987-65-4321" (raw SSN)
- ✅ Log DOES NOT contain "secret@hidden.com" (raw email)
- ✅ Log DOES NOT contain any pseudonym mappings
- ✅ Structured JSON format

**Validation:**
```bash
# Search for raw PII in logs (should return nothing)
docker compose logs privacy-guard | grep "987-65-4321"
docker compose logs privacy-guard | grep "secret@hidden.com"

# If empty output → PASS (no PII in logs)
```

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

### Test 10: Performance Benchmarking

**Objective:** Measure P50, P95, P99 latency and verify against targets

**Setup:**

Create benchmark script `bench_guard.sh`:
```bash
#!/bin/bash

# Create results directory
mkdir -p benchmark_results

echo "Starting Privacy Guard Performance Benchmark..."
echo "Running 100 requests..."

for i in {1..100}; do
  start=$(date +%s%N)
  curl -s -X POST http://localhost:8089/guard/mask \
    -H 'Content-Type: application/json' \
    -d '{
      "text": "Contact John Doe at 555-123-4567 or john.doe@example.com. SSN: 123-45-6789. Credit card: 4532015112830366. From IP: 192.168.1.100",
      "tenant_id": "test-org"
    }' > /dev/null
  end=$(date +%s%N)
  
  # Calculate duration in milliseconds
  duration=$(( (end - start) / 1000000 ))
  echo $duration
done | tee benchmark_results/latencies.txt | sort -n | awk '
  BEGIN {
    print "=== Privacy Guard Performance Results ==="
  }
  {
    arr[NR]=$1
    sum+=$1
  }
  END {
    print "Total requests: " NR
    print "Mean: " int(sum/NR) " ms"
    print "Min: " arr[1] " ms"
    print "Max: " arr[NR] " ms"
    print "P50 (median): " arr[int(NR*0.50)] " ms"
    print "P90: " arr[int(NR*0.90)] " ms"
    print "P95: " arr[int(NR*0.95)] " ms"
    print "P99: " arr[int(NR*0.99)] " ms"
    print ""
    print "=== Target Validation ==="
    if (arr[int(NR*0.50)] <= 500) print "✅ P50 <= 500ms: PASS"; else print "❌ P50 > 500ms: FAIL"
    if (arr[int(NR*0.95)] <= 1000) print "✅ P95 <= 1000ms: PASS"; else print "❌ P95 > 1000ms: FAIL"
    if (arr[int(NR*0.99)] <= 2000) print "✅ P99 <= 2000ms: PASS"; else print "❌ P99 > 2000ms: FAIL"
  }
'
```

**Execution:**
```bash
chmod +x bench_guard.sh
./bench_guard.sh
```

**Expected Output:**
```
=== Privacy Guard Performance Results ===
Total requests: 100
Mean: 85 ms
Min: 42 ms
Max: 1234 ms
P50 (median): 78 ms
P90: 145 ms
P95: 256 ms
P99: 892 ms

=== Target Validation ===
✅ P50 <= 500ms: PASS
✅ P95 <= 1000ms: PASS
✅ P99 <= 2000ms: PASS
```

**Pass Criteria:**
- ✅ **P50 ≤ 500ms** (PRIMARY TARGET)
- ✅ **P95 ≤ 1000ms** (PRIMARY TARGET)
- ✅ **P99 ≤ 2000ms** (PRIMARY TARGET)
- ✅ No request failures (all 100 requests succeed)
- ✅ Results saved to `benchmark_results/latencies.txt`

**Note:** Performance may vary based on:
- System load
- Docker overhead
- Text complexity (number of PII entities)
- FPE encryption operations

**Result:** [ ] PASS [ ] FAIL

**P50:** _____ ms  
**P95:** _____ ms  
**P99:** _____ ms

**Notes:**
_____________________

---

### Test 11: Controller Integration (Optional)

**Objective:** Verify controller calls guard when GUARD_ENABLED=true

**Prerequisites:**
- Controller service running
- JWT token from Phase 1.2

**Setup:**
```bash
# Enable guard in controller
echo "GUARD_ENABLED=true" >> .env.ce
echo "GUARD_URL=http://privacy-guard:8089" >> .env.ce

# Restart controller to pick up new env vars
docker compose restart controller

# Wait for controller to be healthy
docker compose ps controller
```

**Command:**
```bash
# Get JWT (from Phase 1.2 procedure)
JWT="<your-jwt-token>"

# Send audit event with PII content
curl -X POST http://localhost:8088/audit/ingest \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $JWT" \
  -d '{
    "tenant_id": "test-org",
    "event_type": "task_completed",
    "content": "User alice@example.com completed task at 555-123-4567"
  }'
```

**Expected Response:**
```json
{
  "status": "ingested",
  "event_id": "<uuid>"
}
```

**Validation:**
```bash
# Check controller logs for guard integration
docker compose logs controller --tail 50 | grep -i "guard"

# Should see:
# - "Privacy Guard: ENABLED"
# - "Masked audit content" with redaction counts

# Check for redaction summary
docker compose logs controller | grep "redactions"
# Should show: {"EMAIL": 1, "PHONE": 1}
```

**Pass Criteria:**
- ✅ Controller calls guard service
- ✅ Audit event ingested successfully
- ✅ Controller logs show "Masked audit content"
- ✅ Redaction counts logged: `EMAIL: 1, PHONE: 1`
- ✅ No raw PII in controller logs
- ✅ Guard service logs show request from controller

**Graceful Degradation Test:**
```bash
# Stop guard service
docker compose stop privacy-guard

# Send another audit event
curl -X POST http://localhost:8088/audit/ingest \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $JWT" \
  -d '{
    "tenant_id": "test-org",
    "event_type": "task_started",
    "content": "Test content"
  }'

# Check controller logs
docker compose logs controller --tail 20

# Should see warning: "Guard call failed" or "Guard returned error, failing open"
# Event should still be ingested (fail-open behavior)

# Restart guard
docker compose start privacy-guard
```

**Pass Criteria (Graceful Degradation):**
- ✅ Controller logs warning when guard unavailable
- ✅ Audit event still ingested (fail-open)
- ✅ No controller crash or error

**Result:** [ ] PASS [ ] FAIL [ ] SKIPPED (if controller integration disabled)

**Notes:**
_____________________

---

### Test 12: Flush Session State

**Objective:** Verify session state can be cleared

**Commands:**
```bash
# Step 1: Create mappings in a session
mask1=$(curl -s -X POST http://localhost:8089/guard/mask \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Email: test@example.com",
    "tenant_id": "test-org",
    "session_id": "test-session-123"
  }')

session_id=$(echo "$mask1" | jq -r .session_id)
echo "Session ID: $session_id"
echo "Masked text: $(echo "$mask1" | jq -r .masked_text)"

# Step 2: Flush the session
curl -X POST http://localhost:8089/internal/flush-session \
  -H 'Content-Type: application/json' \
  -d "{
    \"session_id\": \"$session_id\"
  }"

# Step 3: Try to reidentify (should fail - mapping cleared)
# This would require JWT, but the mapping should be gone
echo "Session flushed. Mappings cleared."
```

**Pass Criteria:**
- ✅ Flush endpoint returns success (200 or 204)
- ✅ Subsequent reidentify calls fail (404 Not Found)
- ✅ New mask calls create fresh session
- ✅ No errors in logs

**Result:** [ ] PASS [ ] FAIL

**Notes:**
_____________________

---

## Cleanup

After completing all tests:

```bash
# Stop services
docker compose --profile privacy-guard down

# Remove benchmark results (optional)
rm -rf benchmark_results/

# Reset .env.ce if modified
# (remove GUARD_ENABLED=true if added for Test 11)
```

---

## Test Summary

**Total Tests:** 12  
**Required for Sign-Off:** 10 (Tests 1-10)  
**Optional:** Test 11 (controller integration), Test 12 (flush session)

### Results Table

| Test | Description | Status | Notes |
|------|-------------|--------|-------|
| 1 | Healthcheck | [ ] PASS [ ] FAIL | |
| 2 | PII Detection (Scan) | [ ] PASS [ ] FAIL | |
| 3 | Masking with Pseudonyms | [ ] PASS [ ] FAIL | |
| 4 | FPE (Phone) | [ ] PASS [ ] FAIL | |
| 5 | FPE (SSN) | [ ] PASS [ ] FAIL | |
| 6 | Determinism | [ ] PASS [ ] FAIL | |
| 7 | Tenant Isolation | [ ] PASS [ ] FAIL | |
| 8 | Reidentification (JWT) | [ ] PASS [ ] FAIL | |
| 9 | Audit Logs (No PII) | [ ] PASS [ ] FAIL | |
| 10 | Performance Benchmarking | [ ] PASS [ ] FAIL | |
| 11 | Controller Integration | [ ] PASS [ ] FAIL [ ] SKIP | |
| 12 | Flush Session State | [ ] PASS [ ] FAIL [ ] SKIP | |

### Performance Results

- **P50 Latency:** _____ ms (target: ≤ 500ms)
- **P95 Latency:** _____ ms (target: ≤ 1000ms)
- **P99 Latency:** _____ ms (target: ≤ 2000ms)

### Acceptance Criteria

**Phase 2 is ready for sign-off if:**

- ✅ All functional tests (1-9) PASS
- ✅ Performance test (10) PASS with targets met
- ✅ No raw PII found in logs (Test 9)
- ✅ All entity types detected correctly
- ✅ Determinism verified
- ✅ Tenant isolation verified
- ✅ FPE format preservation working

**Optional (nice-to-have):**
- ✅ Controller integration working (Test 11)
- ✅ Session management working (Test 12)

---

## Troubleshooting

### Service won't start

```bash
# Check logs
docker compose logs privacy-guard

# Common issues:
# - PSEUDO_SALT not set → set in .env.ce
# - Config files missing → check deploy/compose/guard-config/
# - Port 8089 in use → stop conflicting service
```

### Detection not working

```bash
# Verify rules loaded
curl http://localhost:8089/status | jq .rule_count
# Should be > 0

# Check guard logs
docker compose logs privacy-guard | grep "rule"

# Test with known PII patterns
curl -X POST http://localhost:8089/guard/scan \
  -H 'Content-Type: application/json' \
  -d '{"text": "SSN: 123-45-6789", "tenant_id": "test"}'
```

### Performance issues

```bash
# Check system resources
docker stats privacy-guard

# Reduce concurrency if needed
# Check guard logs for slow requests
docker compose logs privacy-guard | grep "performance_ms"
```

### Reidentification fails

```bash
# Check JWT validity
# Ensure session_id matches mask call
# Verify pseudonym format: ENTITYTYPE_<hash>
```

---

## Sign-Off

**Test Execution Date:** __________  
**Tester Name:** __________  
**Tester Role:** __________

**Results:**
- [ ] All required tests PASSED (Tests 1-10)
- [ ] Performance targets MET (P50 ≤ 500ms, P95 ≤ 1s, P99 ≤ 2s)
- [ ] No PII in logs VERIFIED
- [ ] Phase 2 ready for completion

**Signature:** __________________  
**Date:** __________

**Notes/Issues:**
_____________________
_____________________
_____________________

---

## References

- **Integration Guide:** `docs/guides/privacy-guard-integration.md`
- **Configuration Guide:** `docs/guides/privacy-guard-config.md`
- **ADR-0021:** Privacy Guard Rust Implementation
- **ADR-0022:** PII Detection Rules and FPE
- **Phase 2 Execution Plan:** `Technical Project Plan/PM Phases/Phase-2/Phase-2-Execution-Plan.md`
- **Phase 1.2 Smoke Tests:** `docs/tests/smoke-phase1.2.md` (for JWT/Vault access)

---

**End of Smoke Test Procedure**
