# Privacy Guard + MCP Integration Testing Plan

**Target Phase:** Phase 4  
**Created:** 2025-11-04 (Phase 3)  
**Status:** Deferred from Phase 3  

---

## Executive Summary

Privacy Guard and Agent Mesh MCP infrastructure are both operational in Phase 3, but comprehensive end-to-end testing is deferred to Phase 4 when session persistence and full authentication are available.

**Phase 3 Status:**
- ✅ Privacy Guard running and healthy
- ✅ Controller integration configured
- ✅ Basic PII redaction working (regex-based)
- ⚠️ Ollama NER model not loaded (advanced detection unavailable)
- ⏸️ End-to-end MCP → Controller → Guard testing incomplete

**Recommendation:** Complete comprehensive Privacy Guard testing in Phase 4 alongside session persistence and full OIDC authentication.

---

## Issues to Address in Phase 4

### High Priority 🔴

1. **Load Ollama NER Model**
   - **Issue:** Model returns 404, advanced detection unavailable
   - **Fix:** `docker exec ce_ollama ollama pull llama3.2:latest`
   - **Impact:** Enables contextual PII detection (names, addresses, contextual entities)
   - **Estimated Time:** 10 minutes + model download time

2. **Comprehensive Integration Tests**
   - **Issue:** No end-to-end MCP → Controller → Guard tests
   - **Fix:** Implement test suite (tests/test_privacy_guard.py)
   - **Impact:** Confidence in production deployment, PII protection verified
   - **Estimated Time:** 8 hours

3. **Performance Benchmarking**
   - **Issue:** Unknown latency overhead and throughput limits
   - **Fix:** Load testing with locust (tests/load_privacy_guard.py)
   - **Impact:** Capacity planning, optimization targets, SLA definition
   - **Estimated Time:** 6 hours

### Medium Priority 🟡

4. **Error Message Sanitization**
   - **Issue:** Unknown if error messages containing PII are masked
   - **Fix:** Test error paths, ensure redaction applies to error responses
   - **Impact:** Prevent PII leakage in error messages returned to MCP tools
   - **Estimated Time:** 2 hours
   - **Test Scenario:** Validation error with user-provided PII should return masked error

5. **Metadata Preservation Verification**
   - **Issue:** Need to confirm task IDs, trace IDs intact post-redaction
   - **Fix:** Add assertions in integration tests for metadata fields
   - **Impact:** Ensure workflow tracking not broken by redaction
   - **Estimated Time:** 1 hour

6. **Guard Failure Modes**
   - **Issue:** Behavior when Guard unavailable not tested
   - **Fix:** Test with Guard down, verify failover/fallback behavior
   - **Impact:** System resilience and availability guarantees
   - **Estimated Time:** 2 hours
   - **Configuration Options:**
     - `GUARD_REQUIRED=true` → Fail requests if Guard down (high security)
     - `GUARD_REQUIRED=false` → Allow requests without redaction (high availability)

### Low Priority 🟢

7. **Custom Entity Configuration**
   - **Issue:** Domain-specific PII types (employee IDs, account numbers) not configurable
   - **Fix:** Allow custom regex patterns in `guard-config/config.yaml`
   - **Impact:** Flexibility for organization-specific PII
   - **Estimated Time:** 4 hours

8. **Horizontal Scaling**
   - **Issue:** Single Guard instance, no load balancing tested
   - **Fix:** Deploy 2+ Guard instances, test load distribution
   - **Impact:** Scalability for high-throughput scenarios (>500 req/s)
   - **Estimated Time:** 3 hours

9. **Audit Trail Analysis**
   - **Issue:** Redaction events logged but not queryable/analyzable
   - **Fix:** Build dashboard or query tool for audit events
   - **Impact:** Compliance reporting, PII exposure monitoring
   - **Estimated Time:** 6 hours

---

## Recommended Testing Approach (Phase 4)

### Test Categories

#### 1. Basic PII Redaction (4 hours)
- Send tasks with SSN, email, phone, credit card via all MCP tools
- Verify redaction in Controller logs and responses
- Confirm entity_counts in redaction events

#### 2. Advanced NER Model Testing (6 hours)
- Load Ollama model: `llama3.2:latest`
- Test contextual entity detection (names, addresses, locations)
- International phone/address formats
- Custom entity types (if configured)

#### 3. End-to-End Workflow Testing (8 hours)
- **Workflow 1:** send_task → request_approval → fetch_status (all with PII)
- **Workflow 2:** Notification chain with cascading PII
- **Workflow 3:** Error handling with PII in error messages
- Verify: PII masked at every step, metadata preserved, session data in Postgres masked

#### 4. Performance & Load Testing (6 hours)
- **Baseline:** Controller latency without Guard
- **With Guard, no PII:** Scan overhead measurement
- **With Guard, 50% PII:** Mixed workload
- **With Guard, 100% PII:** Worst-case overhead
- **Target:** p99 latency <5s, throughput >100 req/s

#### 5. Edge Cases & Error Handling (4 hours)
- Privacy Guard unavailable (failover behavior)
- Malformed PII patterns (edge-case formats)
- Very large payloads (900KB near 1MB limit)
- Ollama model failure (fallback to regex)
- Concurrent redaction requests (queue handling)

---

## Performance Targets

| Metric                     | Target (Phase 4)        | Notes                               |
|----------------------------|-------------------------|-------------------------------------|
| p50 Latency (with PII)     | <150ms                  | Includes Guard scan + mask          |
| p95 Latency (with PII)     | <300ms                  | Acceptable for interactive use      |
| p99 Latency (with PII)     | <5s                     | MVP target, optimize in Phase 5     |
| Throughput (all PII)       | >100 req/s              | Single Guard instance               |
| Throughput (50% PII)       | >300 req/s              | Mixed workload                      |
| Redaction Accuracy         | >99% recall             | No false negatives (PII missed)     |
| False Positive Rate        | <1%                     | Minimal over-redaction              |
| Guard Availability         | 99.9%                   | Graceful degradation if down        |

---

## Success Criteria for Phase 4

### Functional Requirements
- ✅ All PII types detected and masked in MCP tool interactions
- ✅ End-to-end workflows complete with PII redaction
- ✅ Error messages sanitized (no PII leakage)
- ✅ Session data in Postgres contains only masked PII
- ✅ Audit trail complete and queryable

### Performance Requirements
- ✅ p99 latency <5s with full PII redaction
- ✅ Throughput >100 req/s (all requests with PII)
- ✅ No request timeouts under concurrent load
- ✅ Privacy Guard scales to 2+ instances

### Operational Requirements
- ✅ Ollama NER model loaded and functional
- ✅ Graceful degradation when Guard unavailable
- ✅ Health checks passing for all services
- ✅ Documentation complete (setup, testing, troubleshooting)

---

## Estimated Effort (Phase 4)

| Task Category                  | Estimated Hours | Priority |
|--------------------------------|-----------------|----------|
| Basic PII Redaction Tests      | 4               | High     |
| Advanced NER Model Testing     | 6               | High     |
| End-to-End Workflow Testing    | 8               | High     |
| Performance & Load Testing     | 6               | High     |
| Edge Cases & Error Handling    | 4               | Medium   |
| Infrastructure Setup           | 2               | High     |
| Documentation                  | 3               | Medium   |
| **Total**                      | **33 hours**    | -        |

**Recommended Allocation:**
- Sprint 1 (High Priority): Basic tests, NER setup, E2E workflows (18h)
- Sprint 2 (Medium Priority): Performance, edge cases, docs (15h)

---

## Current Evidence (Phase 3)

### Privacy Guard Logs

```
[2025-11-04T19:18:34.469259Z] [INFO] Received scan request (tenant_id=test-tenant, text_length=60)
[2025-11-04T19:18:34.472005Z] [WARN] Ollama returned error status: 404 Not Found
[2025-11-04T19:18:49.019569Z] [INFO] Redaction event: {
  "timestamp":"2025-11-04T19:18:49.019557854+00:00",
  "tenant_id":"test-tenant",
  "session_id":"test-session",
  "mode":"MASK",
  "entity_counts":{"SSN":1},
  "total_redactions":1,
  "performance_ms":4
}
```

**Observations:**
- ✅ Guard receives and processes requests
- ⚠️ Ollama model unavailable (404) - falls back to regex
- ✅ Redaction events logged with metrics (4ms latency)
- ✅ Entity counts tracked

### Controller Logs

```json
{
  "timestamp": "2025-11-04T22:47:43.633735Z",
  "level": "INFO",
  "fields": {"message": "privacy guard integration enabled"},
  "target": "goose_controller"
}
```

**Observations:**
- ✅ Controller detects Privacy Guard (GUARD_ENABLED=true)
- ✅ Integration wired and functional

---

## References

- **Phase 3 Security Status:** `src/agent-mesh/SECURITY-INTEGRATION-STATUS.md`
- **Privacy Guard Source:** `src/privacy-guard/`
- **Controller Guard Client:** `src/controller/src/guard_client.rs`
- **Agent Mesh Tools:** `src/agent-mesh/tools/`
- **Docker Compose Config:** `deploy/compose/ce.dev.yml`

---

**Document Owner:** Phase 3 Agent (goose-org-twin)  
**Last Updated:** 2025-11-04 22:00 UTC  
**Status:** APPROVED for Phase 4 Implementation
