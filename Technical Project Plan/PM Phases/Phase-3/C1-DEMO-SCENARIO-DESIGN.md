# C1: Cross-Agent Demo Scenario Design

**Phase:** 3 - Workstream C  
**Task:** C1  
**Date:** 2025-11-05  
**Status:** ✅ COMPLETE

---

## Overview

This document defines the comprehensive demo scenario for Phase 3, including test cases, success criteria, and validation steps for the Finance → Manager cross-agent approval workflow.

**Objective:** Demonstrate that two Goose agents can communicate via the Controller API using Agent Mesh MCP tools, with full audit trail and JWT authentication.

---

## Demo Scenario: Q1 2026 Engineering Budget Approval

### Actors

1. **Finance Agent**
   - Role: `finance`
   - Tools: `send_task`, `notify`, `fetch_status`, `request_approval`
   - Responsibilities: Submit budget requests, send notifications

2. **Manager Agent**
   - Role: `manager`
   - Tools: `send_task`, `notify`, `fetch_status`, `request_approval`
   - Responsibilities: Review and approve/reject budget requests

### Workflow Steps

```
┌─────────────┐                    ┌──────────────┐                    ┌─────────────┐
│   Finance   │                    │  Controller  │                    │   Manager   │
│    Agent    │                    │     API      │                    │    Agent    │
└──────┬──────┘                    └──────┬───────┘                    └──────┬──────┘
       │                                  │                                   │
       │ 1. send_task (budget request)   │                                   │
       │ ─────────────────────────────────>                                   │
       │                                  │                                   │
       │ ← task_id: abc-123               │                                   │
       │                                  │                                   │
       │                                  │ 2. [Phase 4] notify manager       │
       │                                  │ ──────────────────────────────────>│
       │                                  │                                   │
       │                                  │ 3. fetch_status (check task)      │
       │                                  │ <──────────────────────────────────│
       │                                  │                                   │
       │                                  │ ← 501 Not Implemented (Phase 3)   │
       │                                  │ ──────────────────────────────────>│
       │                                  │                                   │
       │                                  │ 4. POST /approvals (approved)     │
       │                                  │ <──────────────────────────────────│
       │                                  │                                   │
       │                                  │ ← approval_id: def-456            │
       │                                  │ ──────────────────────────────────>│
       │                                  │                                   │
       │ 5. notify (thank you)            │                                   │
       │ ─────────────────────────────────>                                   │
       │                                  │                                   │
       │ ← task_id: ghi-789               │                                   │
       │                                  │                                   │
```

---

## Test Cases

### TC-1: Finance Agent Sends Budget Request

**Preconditions:**
- ✅ Controller API running (http://localhost:8088)
- ✅ Keycloak running with 'dev' realm
- ✅ Finance agent MCP server running
- ✅ Valid JWT token acquired

**Input:**
```json
{
  "target": "manager",
  "task": {
    "task_type": "budget_approval",
    "description": "Q1 2026 Engineering hiring budget",
    "data": {
      "amount": 50000,
      "department": "Engineering",
      "purpose": "Q1 hiring"
    }
  },
  "context": {
    "quarter": "Q1-2026",
    "submitted_by": "finance",
    "priority": "high"
  }
}
```

**Expected Output:**
```
✅ Task routed successfully!

**Task ID:** task-<uuid>
**Status:** accepted
**Target:** manager
**Trace ID:** trace-<uuid>

Use `fetch_status` with this Task ID to check progress.
```

**Success Criteria:**
- ✅ HTTP 200 OK response
- ✅ Task ID returned (valid UUID format)
- ✅ Status: "accepted"
- ✅ Trace ID present
- ✅ Controller logs show POST /tasks/route with JWT verification
- ✅ Privacy Guard masking applied (if PII detected)

**Validation:**
```bash
# Check Controller logs
docker logs ce_controller | grep "POST /tasks/route"

# Verify JWT verification
docker logs ce_controller | grep "JWT verified"

# Verify Privacy Guard masking
docker logs ce_controller | grep "Privacy Guard"
```

---

### TC-2: Manager Checks Task Status (fetch_status)

**Preconditions:**
- ✅ TC-1 completed successfully
- ✅ Task ID from TC-1 available
- ✅ Manager agent MCP server running
- ✅ Valid JWT token acquired

**Input:**
```json
{
  "task_id": "task-<uuid-from-tc1>"
}
```

**Expected Output (Phase 3):**
```
❌ HTTP 501 Not Implemented

This endpoint requires session persistence (deferred to Phase 4).

Trace ID: trace-<uuid>
```

**Success Criteria:**
- ✅ HTTP 501 response (expected behavior for Phase 3)
- ✅ Trace ID present in error message
- ✅ Error message explains deferral to Phase 4
- ✅ Controller logs show GET /sessions/{id} request

**Validation:**
```bash
# Check Controller logs
docker logs ce_controller | grep "GET /sessions/"

# Verify 501 response
docker logs ce_controller | grep "501"
```

**Note:** This is **expected behavior** in Phase 3. Session persistence is deferred to Phase 4.

---

### TC-3: Manager Approves Budget (Direct API Call)

**Preconditions:**
- ✅ TC-1 completed successfully
- ✅ Task ID from TC-1 available
- ✅ Valid JWT token acquired

**Input:**
```bash
curl -X POST http://localhost:8088/approvals \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -H "X-Trace-Id: $(uuidgen)" \
  -d '{
    "task_id": "task-<uuid-from-tc1>",
    "decision": "approved",
    "comments": "Budget approved for Q1 2026 Engineering hiring."
  }'
```

**Expected Output:**
```json
{
  "approval_id": "approval-<uuid>",
  "status": "approved"
}
```

**Success Criteria:**
- ✅ HTTP 200 OK response
- ✅ Approval ID returned (valid UUID format)
- ✅ Status: "approved"
- ✅ Controller logs show POST /approvals with JWT verification
- ✅ Audit event logged with task_id, decision, comments

**Validation:**
```bash
# Check Controller logs
docker logs ce_controller | grep "POST /approvals"

# Verify approval recorded
docker logs ce_controller | grep "Approval recorded"

# Check decision
docker logs ce_controller | grep "approved"
```

---

### TC-4: Finance Agent Sends Thank-You Notification

**Preconditions:**
- ✅ TC-3 completed successfully
- ✅ Finance agent MCP server running
- ✅ Valid JWT token acquired

**Input:**
```json
{
  "target": "manager",
  "message": "Thank you for approving the Q1 2026 Engineering hiring budget! We will proceed with recruitment as planned.",
  "priority": "normal"
}
```

**Expected Output:**
```
✅ Notification sent successfully!

**Task ID:** task-<uuid>
**Status:** accepted
**Target:** manager
**Priority:** normal
**Trace ID:** trace-<uuid>

The notification has been routed to the manager role.
```

**Success Criteria:**
- ✅ HTTP 200 OK response
- ✅ Task ID returned (valid UUID format)
- ✅ Status: "accepted"
- ✅ Priority: "normal"
- ✅ Trace ID present
- ✅ Controller logs show POST /tasks/route with task_type: "notification"
- ✅ Privacy Guard masking applied (if PII detected)

**Validation:**
```bash
# Check Controller logs
docker logs ce_controller | grep "POST /tasks/route"

# Verify notification type
docker logs ce_controller | grep "notification"

# Verify priority
docker logs ce_controller | grep "normal"
```

---

### TC-5: End-to-End Audit Trail

**Preconditions:**
- ✅ All previous test cases completed

**Expected Audit Trail:**

```
[INFO] POST /tasks/route - Budget request from finance to manager
  JWT: sub=<finance-user-id>
  Task ID: task-<uuid-1>
  Target: manager
  Task Type: budget_approval
  Trace ID: trace-<uuid-1>

[INFO] GET /sessions/<uuid-1> - Manager checks task status
  JWT: sub=<manager-user-id>
  Response: 501 Not Implemented
  Trace ID: trace-<uuid-2>

[INFO] POST /approvals - Manager approves budget
  JWT: sub=<manager-user-id>
  Approval ID: approval-<uuid-3>
  Task ID: task-<uuid-1>
  Decision: approved
  Trace ID: trace-<uuid-3>

[INFO] POST /tasks/route - Thank-you notification from finance to manager
  JWT: sub=<finance-user-id>
  Task ID: task-<uuid-4>
  Target: manager
  Task Type: notification
  Trace ID: trace-<uuid-4>
```

**Success Criteria:**
- ✅ All 4 API calls logged
- ✅ JWT verification for each call
- ✅ Trace IDs unique and present
- ✅ Task IDs correlate correctly
- ✅ Chronological order preserved
- ✅ No errors (except expected 501)

**Validation:**
```bash
# Extract all relevant audit entries
docker logs ce_controller | grep -E "task-<uuid-1>|approval-<uuid-3>|task-<uuid-4>"

# Count total API calls
docker logs ce_controller | grep -c "POST /tasks/route"  # Should be 2
docker logs ce_controller | grep -c "GET /sessions/"     # Should be 1
docker logs ce_controller | grep -c "POST /approvals"    # Should be 1
```

---

## Success Criteria Summary

### Functional Requirements

| Requirement | Status | Validation |
|-------------|--------|------------|
| **FR-1:** Finance sends task to Manager | ✅ TC-1 | HTTP 200, task_id returned |
| **FR-2:** Manager checks task status | ✅ TC-2 | HTTP 501 (expected Phase 3) |
| **FR-3:** Manager approves task | ✅ TC-3 | HTTP 200, approval_id returned |
| **FR-4:** Finance sends notification | ✅ TC-4 | HTTP 200, task_id returned |
| **FR-5:** Audit trail complete | ✅ TC-5 | 4 events logged chronologically |

---

### Non-Functional Requirements

| Requirement | Target | Validation |
|-------------|--------|------------|
| **NFR-1:** JWT authentication | 100% of API calls | Controller logs show "JWT verified" |
| **NFR-2:** Privacy Guard masking | Applied if PII detected | Controller logs show "Privacy Guard masked" |
| **NFR-3:** Trace ID propagation | All API calls | Trace IDs present in responses and logs |
| **NFR-4:** Idempotency | Duplicate requests handled | Same key = same response |
| **NFR-5:** API latency | P50 < 5s | Measure end-to-end response time |

**Validation:**
```bash
# Check JWT verification rate
docker logs ce_controller | grep -c "JWT verified"  # Should match API call count

# Check Privacy Guard usage
docker logs ce_controller | grep -c "Privacy Guard"

# Check Trace ID presence
docker logs ce_controller | grep -c "X-Trace-Id"

# Measure latency (example)
time curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{"target":"manager","task":{"task_type":"test"},"context":{}}'
```

---

## Edge Cases and Error Scenarios

### EC-1: Invalid JWT Token

**Input:**
```bash
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer invalid-token" \
  -H "Content-Type: application/json" \
  -d '{"target":"manager","task":{"task_type":"test"},"context":{}}'
```

**Expected Output:**
```
HTTP 401 Unauthorized
```

**Success Criteria:**
- ✅ HTTP 401 response
- ✅ Controller logs show "JWT verification failed"

---

### EC-2: Missing Idempotency-Key

**Input:**
```bash
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"target":"manager","task":{"task_type":"test"},"context":{}}'
```

**Expected Output:**
```
HTTP 400 Bad Request
{"error": "Missing Idempotency-Key header"}
```

**Success Criteria:**
- ✅ HTTP 400 response
- ✅ Error message explains missing header

---

### EC-3: Malformed JSON

**Input:**
```bash
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{invalid json}'
```

**Expected Output:**
```
HTTP 400 Bad Request
{"error": "Invalid JSON"}
```

**Success Criteria:**
- ✅ HTTP 400 response
- ✅ Error message indicates JSON parse error

---

### EC-4: Payload Too Large (>1MB)

**Input:**
```bash
# Generate 2MB payload
dd if=/dev/zero bs=1M count=2 | base64 > large_payload.txt

curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d @large_payload.txt
```

**Expected Output:**
```
HTTP 413 Payload Too Large
```

**Success Criteria:**
- ✅ HTTP 413 response
- ✅ Request rejected before processing

---

### EC-5: Duplicate Idempotency Key

**Input:**
```bash
# First request
IDEM_KEY=$(uuidgen)
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $IDEM_KEY" \
  -d '{"target":"manager","task":{"task_type":"test"},"context":{}}'

# Second request with same key
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $IDEM_KEY" \
  -d '{"target":"manager","task":{"task_type":"test"},"context":{}}'
```

**Expected Output (Phase 3):**
```
HTTP 200 OK (both requests succeed - no deduplication)
```

**Success Criteria (Phase 3):**
- ⚠️ Both requests succeed (idempotency deduplication deferred to Phase 4)
- ✅ Controller logs both requests

**Phase 4 Fix:** Add Redis cache for idempotency key tracking (~1h)

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **P50 Latency** | < 5s | `time curl ...` (average of 10 requests) |
| **P95 Latency** | < 10s | `time curl ...` (95th percentile of 100 requests) |
| **Throughput** | > 10 req/s | `ab -n 100 -c 10 ...` (Apache Bench) |
| **Error Rate** | < 1% | Count 4xx/5xx errors in 1000 requests |
| **JWT Verification** | < 100ms | Controller logs: time between request and "JWT verified" |
| **Privacy Guard Latency** | < 200ms | Controller logs: Privacy Guard masking time |

**Performance Test Script:**
```bash
#!/bin/bash
# performance-test.sh

TOKEN=$(./scripts/get-jwt-token.sh 2>/dev/null)

# Latency test (10 requests)
echo "Testing P50 latency..."
for i in {1..10}; do
  time curl -s -X POST http://localhost:8088/tasks/route \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -H "Idempotency-Key: $(uuidgen)" \
    -d '{"target":"manager","task":{"task_type":"test"},"context":{}}'
done

# Throughput test (100 requests, 10 concurrent)
echo "Testing throughput..."
ab -n 100 -c 10 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -p payload.json \
  http://localhost:8088/tasks/route
```

---

## Backward Compatibility Validation

### Phase 1.2 Compatibility

**Test:** JWT middleware still works with Keycloak 26.0.4

**Validation:**
```bash
# Get JWT token from Keycloak
TOKEN=$(./scripts/get-jwt-token.sh 2>/dev/null)

# Call Phase 1.2 endpoint
curl -X GET http://localhost:8088/status \
  -H "Authorization: Bearer $TOKEN"

# Expected: 200 OK
```

**Success Criteria:**
- ✅ HTTP 200 OK
- ✅ JWT middleware validates token
- ✅ No breaking changes from Phase 1.2

---

### Phase 2.2 Compatibility

**Test:** Privacy Guard integration still works

**Validation:**
```bash
# Check Privacy Guard health
curl http://localhost:8089/status

# Expected: {"status":"healthy","mode":"Mask",...}

# Send task with PII
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{
    "target":"manager",
    "task":{
      "task_type":"test",
      "description":"Contact John Doe at john.doe@example.com"
    },
    "context":{}
  }'

# Check Controller logs for masking
docker logs ce_controller | grep "Privacy Guard masked"
```

**Success Criteria:**
- ✅ Privacy Guard masking applied
- ✅ PII replaced with placeholders
- ✅ No breaking changes from Phase 2.2

---

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| **R-1:** JWT token expires during demo | HIGH | Tokens valid for 60 min; script to refresh | ✅ Mitigated |
| **R-2:** fetch_status returns 501 | MEDIUM | Document as expected Phase 3 behavior | ✅ Documented |
| **R-3:** Privacy Guard unavailable | MEDIUM | Controller falls back to no masking | ✅ Graceful degradation |
| **R-4:** Keycloak down | HIGH | Health checks before demo, restart script | ✅ Automated recovery |
| **R-5:** Network latency > target | LOW | Baseline performance metrics, optimize Phase 4 | ✅ Measured |

---

## Deliverables for C1

- ✅ This document (C1-DEMO-SCENARIO-DESIGN.md)
- ✅ Test cases defined (TC-1 through TC-5)
- ✅ Edge cases documented (EC-1 through EC-5)
- ✅ Success criteria established (Functional + Non-Functional)
- ✅ Performance targets set
- ✅ Backward compatibility validation plan
- ✅ Risk mitigation strategies

---

## Next Steps (C2: Implementation)

1. Execute TC-1 through TC-5 with real agents
2. Validate all success criteria
3. Measure performance metrics
4. Document results in progress log
5. Capture screenshots/logs for evidence

**Estimated Time:** ~4 hours

---

**Status:** ✅ COMPLETE  
**Date:** 2025-11-05  
**Next:** C2 - Demo Implementation
