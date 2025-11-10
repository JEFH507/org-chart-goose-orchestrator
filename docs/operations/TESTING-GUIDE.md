# Comprehensive Testing Guide

**Version:** 1.0.0  
**Last Updated:** 2025-11-10  
**Purpose:** Complete reference for running all tests after system startup

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Test Categories](#test-categories)
3. [Quick Test Suite](#quick-test-suite)
4. [Detailed Test Procedures](#detailed-test-procedures)
5. [Test Scripts Reference](#test-scripts-reference)
6. [Expected Results](#expected-results)
7. [Troubleshooting Failed Tests](#troubleshooting-failed-tests)

---

## Prerequisites

### Services Must Be Running

✅ All services healthy (check with `docker ps`):
- ce_controller (8088)
- ce_privacy_guard (8089)
- ce_keycloak (8080)
- ce_vault (8200/8201) - **UNSEALED**
- ce_postgres (5432)
- ce_redis (6379)
- ce_ollama (11434)

### Database Migrations Complete

✅ All 8 tables exist:
```bash
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"
```

### JWT Token Available

✅ Obtain JWT token:
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
export JWT=$(./scripts/get-jwt-token.sh)
echo $JWT  # Should show long token string
```

---

## Test Categories

### Category 1: Infrastructure Tests
- Service health checks
- Database connectivity
- Vault status
- Redis connectivity
- Ollama model availability

### Category 2: Privacy Guard Tests
- PII scanning (regex rules)
- NER model inference
- Masking/unmasking
- Audit log generation

### Category 3: Vault Production Tests
- Transit engine signing
- AppRole authentication
- Profile tamper detection
- Token renewal

### Category 4: Controller API Tests
- Profile loading
- JWT authentication
- Admin endpoints
- Audit logging

### Category 5: Integration Tests
- Privacy Guard + Controller
- Vault + Controller (profile signing)
- Full E2E workflow

---

## Quick Test Suite

Run all tests in sequence:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# 1. Infrastructure tests (1 minute)
echo "=== INFRASTRUCTURE TESTS ==="
curl -s http://localhost:8088/status | jq
curl -s http://localhost:8089/status | jq
docker exec ce_vault vault status | grep "Sealed"
docker exec ce_redis redis-cli ping
docker exec ce_ollama ollama list

# 2. Get JWT token
echo "=== ACQUIRING JWT TOKEN ==="
export JWT=$(./scripts/get-jwt-token.sh)
echo "JWT acquired: ${JWT:0:50}..."

# 3. Privacy Guard integration test (2 minutes)
echo "=== PRIVACY GUARD TEST ==="
./scripts/test-finance-pii-jwt.sh

# 4. Vault production test (2 minutes)
echo "=== VAULT PRODUCTION TEST ==="
./scripts/test-vault-production.sh

# 5. Controller API tests
echo "=== CONTROLLER API TESTS ==="
# Health check
curl -s http://localhost:8088/status | jq

# Profile loading (if finance profile exists)
curl -s -H "Authorization: Bearer $JWT" \
  http://localhost:8088/profiles/finance | jq '.role, .display_name'

# Audit log test
curl -s -X POST http://localhost:8088/audit/ingest \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "test.event",
    "user_id": "test-user",
    "resource": "test-resource",
    "action": "test-action",
    "status": "success"
  }' | jq

echo "=== ALL TESTS COMPLETE ==="
```

**Expected Time:** ~5 minutes

---

## Detailed Test Procedures

### Test 1: Service Health Checks

**Purpose:** Verify all services are running and healthy

**Commands:**

```bash
# Controller
curl -s http://localhost:8088/status | jq
# Expected: {"status": "ok", "version": "0.1.0"}

# Privacy Guard
curl -s http://localhost:8089/status | jq
# Expected: {"status": "healthy", "mode": "Mask", "rule_count": 22, ...}

# Keycloak
curl -s http://localhost:8080/health
# Expected: HTML response (200 OK)

# Vault
docker exec ce_vault vault status
# Expected: Sealed = false, Initialized = true

# Postgres
docker exec ce_postgres psql -U postgres -c "SELECT version();"
# Expected: PostgreSQL 17.2

# Redis
docker exec ce_redis redis-cli ping
# Expected: PONG

# Ollama
docker exec ce_ollama ollama list
# Expected: qwen3:0.6b listed
```

**Pass Criteria:** All commands return expected output, no errors

---

### Test 2: JWT Token Acquisition

**Purpose:** Verify Keycloak authentication works

**Command:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/get-jwt-token.sh
```

**Expected Output:**

```
Getting JWT token from Keycloak...
eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICI...
```

**Pass Criteria:** 
- Token starts with `eyJ`
- Length > 500 characters
- No errors

**Using the Token:**

```bash
# Save to environment variable
export JWT=$(./scripts/get-jwt-token.sh)

# Test with Controller API
curl -H "Authorization: Bearer $JWT" http://localhost:8088/profiles/finance
```

---

### Test 3: Privacy Guard PII Detection

**Purpose:** Verify Privacy Guard can detect and mask PII

**Script:** `./scripts/test-finance-pii-jwt.sh`

**What it tests:**

1. **JWT Authentication** - Acquire token from Keycloak
2. **Profile Loading** - GET /profiles/finance
3. **PII Scanning** - Detect SSN, email, phone, employee ID
4. **PII Masking** - Replace PII with pseudonyms
5. **Audit Logging** - POST /audit/ingest
6. **NER Detection** - Ollama model inference
7. **Deterministic Pseudonymization** - Same input → same token
8. **Unmask Verification** - Tokens can be unmasked

**Run Test:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/test-finance-pii-jwt.sh
```

**Expected Results:**

```
Running Finance Profile PII Test with JWT Authentication
============================================================

Test 1: JWT Token Acquisition ✓
Test 2: Profile Loading (GET /profiles/finance) ✓
Test 3a: Privacy Guard /scan - Detect PII ✓
Test 3b: Verify PII Categories Detected ✓
Test 4: Privacy Guard /mask - Mask PII ✓
Test 5: Privacy Guard /unmask - Restore PII ✓
Test 6: Controller Audit Log Ingestion ✓
Test 7: Deterministic Pseudonymization ✓

============================================================
FINANCE PII TEST: 8/8 PASSED ✅
```

**Pass Criteria:** 8/8 tests passing

---

### Test 4: Vault Production Tests

**Purpose:** Verify Vault Transit signing, AppRole auth, tamper detection

**Script:** `./scripts/test-vault-production.sh`

**What it tests:**

1. **AppRole Authentication** - Controller authenticates with Vault
2. **Transit Signing** - Sign profile with HMAC
3. **Signature Verification** - Verify signed profile
4. **Tamper Detection** - Reject modified profile
5. **Profile Loading** - Load signed profile from DB
6. **Signature in Database** - Verify signature persisted
7. **End-to-End Workflow** - Create → Sign → Verify → Load

**Run Test:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/test-vault-production.sh
```

**Expected Results:**

```
Running Vault Production Tests (Phase 6A)
============================================

Test 1: AppRole Authentication ✓
Test 2: Transit Engine Key Creation ✓
Test 3: Profile Signing (HMAC) ✓
Test 4: Signature Verification ✓
Test 5: Tamper Detection ✓
Test 6: Profile Load from Database ✓
Test 7: Signature Persistence ✓

============================================
VAULT PRODUCTION TEST: 7/7 PASSED ✅
```

**Pass Criteria:** 7/7 tests passing

---

### Test 5: Controller API Endpoints

**Purpose:** Verify all Controller routes work

**Prerequisites:** JWT token in `$JWT` variable

**Tests:**

**5a. Health Check**

```bash
curl -s http://localhost:8088/status | jq
# Expected: {"status": "ok", "version": "0.1.0"}
```

**5b. Profile Loading (Finance)**

```bash
curl -s -H "Authorization: Bearer $JWT" \
  http://localhost:8088/profiles/finance | jq '.role, .display_name'

# Expected:
# "finance"
# "Finance Team Agent"
```

**5c. OpenAPI Spec**

```bash
curl -s http://localhost:8088/api-docs/openapi.json | jq '.info'

# Expected:
# {
#   "title": "Goose Orchestrator Controller API",
#   "version": "0.5.0",
#   ...
# }
```

**5d. Audit Log Ingestion**

```bash
curl -s -X POST http://localhost:8088/audit/ingest \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "test.event",
    "user_id": "test-user",
    "resource": "test-resource",
    "action": "test-action",
    "status": "success",
    "metadata": {}
  }' | jq

# Expected: 202 Accepted
```

**5e. Session Creation**

```bash
curl -s -X POST http://localhost:8088/sessions \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "test-user",
    "metadata": {"test": true}
  }' | jq

# Expected: 201 Created with session_id
```

**Pass Criteria:** All endpoints return expected HTTP status and JSON

---

### Test 6: Session Lifecycle Testing (Phase 6 A.3)

**Purpose:** Verify Session Lifecycle FSM state transitions and persistence

**Script:** `./tests/integration/test_session_lifecycle_comprehensive.sh`

**What it tests:**

1. **Create Session → PENDING** - Session created with initial state
2. **Activate → ACTIVE** - PENDING → ACTIVE transition
3. **Pause → PAUSED** - ACTIVE → PAUSED transition + timestamp
4. **Resume → ACTIVE** - PAUSED → ACTIVE transition + clear timestamp
5. **Complete → COMPLETED** - ACTIVE → COMPLETED + timestamp + terminal protection
6. **Persistence** - Session survives controller restart
7. **Concurrent Sessions** - Multiple sessions for same user
8. **Timeout Simulation** - Expiration testing

**Run Test:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./tests/integration/test_session_lifecycle_comprehensive.sh
```

**Expected Results:**

```
=== Acquiring JWT token ===
JWT token acquired: eyJhbGci...

=== Test 1: Create session → PENDING state ===
✓ PASS: Session created with ID
✓ PASS: Session status is pending

=== Test 2: Start task → ACTIVE state ===
✓ PASS: Session transitioned to active

=== Test 3: Pause session → PAUSED state ===
✓ PASS: Session transitioned to paused
✓ PASS: paused_at timestamp set in database

=== Test 4: Resume session → ACTIVE state ===
✓ PASS: Session resumed to active
✓ PASS: paused_at timestamp cleared after resume

=== Test 5: Complete session → COMPLETED state ===
✓ PASS: Session transitioned to completed
✓ PASS: completed_at timestamp set in database
✓ PASS: Terminal state (completed) cannot transition

=== Test 6: Session persistence across Controller restart ===
Restarting controller...
✓ PASS: Session status persisted
✓ PASS: Session role persisted

=== Test 7: Concurrent sessions for same user ===
✓ PASS: Session 1 is active
✓ PASS: Session 2 is active
✓ PASS: Concurrent sessions have unique IDs

=== Test 8: Session timeout (simulated with expire event) ===
✓ PASS: Session is active before timeout
✓ PASS: Expiration test session created

=== TEST SUMMARY ===
PASSED: 17
FAILED: 0
✓ ALL TESTS PASSED
```

**Pass Criteria:** 17/17 tests passing

**Session State Diagram:**

```
PENDING → ACTIVE → PAUSED → ACTIVE → COMPLETED (terminal)
          │         ↓
          ├─→ FAILED (terminal)
          └─→ EXPIRED (terminal)
```

**FSM Events:**
- `activate` - PENDING → ACTIVE
- `pause` - ACTIVE → PAUSED
- `resume` - PAUSED → ACTIVE
- `complete` - ACTIVE → COMPLETED
- `fail` - ACTIVE → FAILED

**API Endpoint:**
```bash
# Trigger lifecycle event
curl -X PUT http://localhost:8088/sessions/{id}/events \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "pause"}'
```

**Database Columns:**
- `status` - Current state (pending, active, paused, completed, failed, expired)
- `fsm_metadata` - FSM-specific metadata (JSONB)
- `last_transition_at` - Timestamp of last state change
- `paused_at` - Timestamp when paused (NULL if not paused)
- `completed_at` - Timestamp when completed (NULL if not completed)
- `failed_at` - Timestamp when failed (NULL if not failed)

**References:**
- **State Diagram:** `/docs/architecture/session-lifecycle.md`
- **Implementation:** `src/lifecycle/session_lifecycle.rs`
- **Routes:** `src/controller/src/routes/sessions.rs`
- **Migration:** `db/migrations/metadata-only/0007_update_sessions_for_lifecycle.sql`

---

### Test 7: Database Verification

**Purpose:** Verify database schema and data integrity

**Tests:**

**7a. Table Count**

```bash
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt" | grep -c "public |"
# Expected: 8 tables
```

**6b. Profiles Table**

```bash
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT role, display_name FROM profiles WHERE role = 'finance';"

# Expected:
#   role   |    display_name
# ---------+--------------------
#  finance | Finance Team Agent
```

**6c. Privacy Audit Logs**

```bash
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT COUNT(*) FROM privacy_audit_logs;"

# Expected: (some number, depending on tests run)
```

**6d. Audit Events**

```bash
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT COUNT(*) FROM audit_events;"

# Expected: (some number, including test events)
```

**Pass Criteria:** All queries return data without errors

---

## Test Scripts Reference

### Available Test Scripts

```
scripts/
├── get-jwt-token.sh              # Acquire JWT from Keycloak
├── vault-unseal.sh               # Interactive Vault unseal
├── test-finance-pii-jwt.sh       # Privacy Guard integration (8 tests)
├── test-vault-production.sh      # Vault production features (7 tests)
└── test-legal-local.sh           # Legal profile (SKIP - no profile in DB)
```

### Test Script Details

**get-jwt-token.sh**

```bash
# Usage
./scripts/get-jwt-token.sh

# Output: JWT token string
# Uses: OIDC password grant (admin/admin)
# Client: goose-controller
# Realm: dev
```

**test-finance-pii-jwt.sh**

```bash
# Usage
./scripts/test-finance-pii-jwt.sh

# Tests: 8 (JWT, profile, scan, mask, unmask, audit, pseudonym, NER)
# Duration: ~2 minutes
# Dependencies: Keycloak, Controller, Privacy Guard, Postgres
```

**test-vault-production.sh**

```bash
# Usage
./scripts/test-vault-production.sh

# Tests: 7 (AppRole, Transit, sign, verify, tamper, load, persist)
# Duration: ~2 minutes
# Dependencies: Vault (unsealed), Controller, Postgres
```

---

## Expected Results

### Full Test Suite Success

When all tests pass, you should see:

```
=== INFRASTRUCTURE TESTS ===
✅ Controller: OK
✅ Privacy Guard: Healthy
✅ Vault: Unsealed
✅ Redis: PONG
✅ Ollama: qwen3:0.6b loaded

=== JWT TOKEN ===
✅ Token acquired: eyJhbGci...

=== PRIVACY GUARD TEST ===
✅ FINANCE PII TEST: 8/8 PASSED

=== VAULT PRODUCTION TEST ===
✅ VAULT PRODUCTION TEST: 7/7 PASSED

=== CONTROLLER API TESTS ===
✅ Health check: OK
✅ Profile loading: Finance Team Agent
✅ Audit log: Accepted

=== ALL TESTS COMPLETE ===
Total: 18/18 PASSED ✅
```

---

## Troubleshooting Failed Tests

### Test Failure: JWT Token Acquisition

**Error:** `curl: (7) Failed to connect to localhost:8080`

**Cause:** Keycloak not running

**Fix:**
```bash
docker ps | grep keycloak
# If not running:
docker compose -f deploy/compose/ce.dev.yml up -d keycloak
```

---

### Test Failure: Privacy Guard /scan

**Error:** `Connection refused (port 8089)`

**Cause:** Privacy Guard not running

**Fix:**
```bash
docker compose -f deploy/compose/ce.dev.yml \
  --profile ollama --profile privacy-guard up -d privacy-guard
```

---

### Test Failure: Profile Not Found (403/404)

**Error:** `GET /profiles/finance` → 403 Forbidden or 404 Not Found

**Cause 1:** Vault token expired (Controller can't access Vault)

**Fix:**
```bash
docker compose -f deploy/compose/ce.dev.yml --profile controller restart controller
sleep 10
```

**Cause 2:** Profile not loaded into database

**Fix:**
```bash
# Load profile via Admin API (requires profile loading script - to be created)
# Or check if profile exists:
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT role FROM profiles WHERE role = 'finance';"
```

---

### Test Failure: Vault Signing

**Error:** `Vault Transit signing failed`

**Cause:** Vault is sealed

**Fix:**
```bash
docker exec ce_vault vault status | grep Sealed
# If Sealed = true:
./scripts/vault-unseal.sh
```

---

### Test Failure: Audit Log 500 Error

**Error:** `POST /audit/ingest` → 500 Internal Server Error

**Cause:** Table `audit_events` doesn't exist

**Fix:**
```bash
# Run migration
docker exec -i ce_postgres psql -U postgres -d orchestrator \
  < deploy/migrations/001_create_schema.sql
```

---

### Test Failure: NER Model Not Found

**Error:** Privacy Guard logs show "model not found"

**Cause:** Ollama doesn't have qwen3:0.6b model

**Fix:**
```bash
# Pull model
docker exec ce_ollama ollama pull qwen3:0.6b

# Wait for download (522 MB)
# Verify
docker exec ce_ollama ollama list
```

---

## Test Coverage Summary

### By Component

| Component | Tests | Coverage |
|-----------|-------|----------|
| Controller API | 5 | Routes, middleware, auth |
| Privacy Guard | 8 | Scan, mask, unmask, NER, audit |
| Vault | 7 | Transit, AppRole, tamper detection |
| Keycloak | 1 | JWT acquisition |
| Database | 4 | Schema, tables, queries |
| **Total** | **25** | **Full stack** |

### By Test Type

| Type | Count | Examples |
|------|-------|----------|
| Unit | 0 | (Rust tests, not in scripts) |
| Integration | 15 | Privacy Guard + Controller, Vault + Controller |
| E2E | 5 | Full workflow (JWT → API → DB → Vault) |
| Smoke | 5 | Health checks, connectivity |

---

## Next Steps

After successful testing:

1. **Load Profiles:** See `/docs/operations/PROFILE-LOADING-GUIDE.md` (to be created)
2. **Phase 6 Planning:** Review test results with stakeholder
3. **Multi-Goose Testing:** Set up Docker Goose containers
4. **Agent Mesh E2E:** Test cross-agent communication

---

**Document Version:** 1.0.0  
**Maintained By:** Goose Orchestrator Agent  
**Last Reviewed:** 2025-11-10
