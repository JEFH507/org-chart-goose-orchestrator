# Phase 4 Requirements - Deferred from Phase 3

**Created:** 2025-11-04 (Phase 3)  
**Status:** Planning Document  
**Source:** Phase 3 implementation decisions and deferred items  

---

## Executive Summary

This document consolidates all requirements, fixes, and features deferred from Phase 3 to Phase 4. These items were intentionally postponed to maintain Phase 3 focus on MVP functionality while ensuring a clear roadmap for production readiness.

**Total Estimated Effort:** ~41 hours (5+ days)

**Categories:**
1. **Security & Authentication** (8 hours) 🔐 HIGH PRIORITY
2. **Session Persistence** (6 hours) 🔴 HIGH PRIORITY
3. **Privacy Guard Testing** (33 hours) 🟡 MEDIUM PRIORITY
4. **Integration Test Updates** (2 hours) 🔴 HIGH PRIORITY
5. **Schema & API Fixes** (1 hour) 🟢 LOW PRIORITY
6. **Production Hardening** (varies) 🟡 MEDIUM PRIORITY

---

## 1. Security & Authentication (8 hours) 🔐 HIGH PRIORITY

### 1.1 Regenerate Keycloak Client Secret (30 minutes)

**Priority:** HIGH 🔴  
**Reason:** Current client secret exposed in Phase 3 conversation logs/documentation

**Current State:**
- Client secret: `ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1`
- Generated during Phase 3 development
- Documented in temporary files (to be deleted before commit)
- Safe for local development, NOT safe for production

**Phase 4 Actions:**

**Option A: Regenerate in Dev Realm**
```bash
# Login to Keycloak Admin Console
# http://localhost:8080 (admin/admin)
# Navigate: Realms → dev → Clients → goose-controller → Credentials tab
# Click: "Regenerate Secret"
# Update .env.ce with new secret
# Restart controller
```

**Option B: Create Production Realm (Recommended)**
```bash
# Create new realm: 'production'
# Create new client: 'goose-controller'
# Generate client secret
# Store in Vault (already configured)
# Update production .env with Vault references
```

**Deliverables:**
- [ ] New client secret generated
- [ ] `.env.ce` updated (or Vault for production)
- [ ] Controller restarted and verified
- [ ] Old secret removed from all documentation
- [ ] ADR documenting secret rotation policy

**Estimated Time:** 30 minutes (dev realm) or 2 hours (production realm setup)

---

### 1.2 Production Keycloak Setup (4 hours)

**Priority:** HIGH 🔴  
**Reason:** Production needs separate Keycloak instance with proper configuration

**Current State:**
- Using 'dev' realm in local Keycloak
- Admin credentials: admin/admin (insecure)
- No user management
- No MFA/2FA
- No audit logging

**Phase 4 Actions:**

1. **Deploy Production Keycloak**
   - Separate instance (not localhost)
   - TLS/SSL certificates
   - Database backend (Postgres)
   - High availability (optional)

2. **Create Production Realm**
   - Realm name: 'production'
   - Token lifetimes: shorter than dev
   - Session timeouts: production values
   - Enable audit logging

3. **Configure Security Policies**
   - Password complexity requirements
   - MFA/2FA enforcement
   - Account lockout policies
   - IP allowlisting (optional)

4. **Client Configuration**
   - Create 'goose-controller' client
   - Enable PKCE (proof key for code exchange)
   - Configure redirect URIs
   - Set up audience mapper (goose-controller)

5. **User Management**
   - Create admin users
   - Create service accounts
   - Define user roles (manager, finance, engineering, etc.)
   - Import test users for staging

6. **Integration with Vault**
   - Store client secret in Vault
   - Rotate secrets automatically
   - Audit secret access

**Deliverables:**
- [ ] Production Keycloak deployed
- [ ] Production realm configured
- [ ] Client created with secure settings
- [ ] Users and roles defined
- [ ] Vault integration complete
- [ ] Documentation in docs/production/keycloak-setup.md

**Estimated Time:** 4 hours

---

### 1.3 JWT Token Management (2 hours)

**Priority:** MEDIUM 🟡  
**Reason:** Current JWT setup is basic, needs production features

**Current State:**
- Token acquisition via password grant
- No token refresh
- No token revocation
- No session management

**Phase 4 Actions:**

1. **Token Refresh Flow**
   - Implement refresh token support
   - Configure refresh token lifetimes
   - Auto-refresh before expiration

2. **Token Revocation**
   - Enable token revocation endpoint
   - Handle revoked tokens in Controller
   - Clear revoked tokens from cache

3. **Session Management**
   - Link JWT tokens to sessions
   - Enable session tracking
   - Implement logout (end session + revoke token)

4. **Token Caching**
   - Cache valid tokens (Redis)
   - Reduce Keycloak load
   - Fast token validation

**Deliverables:**
- [ ] Refresh token flow implemented
- [ ] Token revocation working
- [ ] Session management in place
- [ ] Token caching (Redis) configured
- [ ] Tests for token lifecycle

**Estimated Time:** 2 hours

---

### 1.4 Delete Temporary JWT Documentation (15 minutes)

**Priority:** HIGH 🔴 (Before Phase 3 commit)  
**Reason:** Files contain client secret in plaintext

**Files to Delete:**
```bash
# Delete before committing Phase 3:
rm JWT-SETUP-COMPLETE.md
rm JWT-VERIFICATION-COMPLETE.md
```

**Optional - Sanitize:**
```bash
# scripts/setup-keycloak-dev-realm.sh
# Remove or replace actual client secret in output examples
# Replace with: OIDC_CLIENT_SECRET=<your-secret-here>
```

**Verification:**
```bash
# Ensure .env.ce not in git:
git status | grep .env.ce  # Should be empty
git check-ignore deploy/compose/.env.ce  # Should return "ignored"

# Search for leaked secrets:
git log --all -p | grep "ApMMxVd8b6v0Sec26FuAi8vuxpbZrAl1" | wc -l  # Should be 0
```

**Deliverables:**
- [ ] JWT-SETUP-COMPLETE.md deleted
- [ ] JWT-VERIFICATION-COMPLETE.md deleted
- [ ] .env.ce verified not in git
- [ ] No secrets in git history

**Estimated Time:** 15 minutes

**When:** Before final Phase 3 commit

---

## 2. Session Persistence (6 hours) 🔴 HIGH PRIORITY

### 2.1 Postgres Session Storage (4 hours)

**Priority:** HIGH 🔴  
**Reason:** Controller is currently stateless, fetch_status returns 501

**Current State:**
- Controller returns 501 for GET /sessions/{id}
- No session persistence
- Tasks routed but not stored
- Approvals submitted but not tracked

**Phase 4 Actions:**

1. **Database Schema**
   ```sql
   CREATE TABLE sessions (
       session_id UUID PRIMARY KEY,
       task_id UUID NOT NULL,
       target VARCHAR(100) NOT NULL,
       task_type VARCHAR(100) NOT NULL,
       task_description TEXT,
       task_data JSONB,
       context JSONB,
       status VARCHAR(50) NOT NULL,  -- 'pending', 'approved', 'rejected', 'completed'
       created_at TIMESTAMP NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
       completed_at TIMESTAMP,
       trace_id UUID,
       idempotency_key UUID,
       privacy_guard_redactions JSONB
   );

   CREATE INDEX idx_sessions_task_id ON sessions(task_id);
   CREATE INDEX idx_sessions_status ON sessions(status);
   CREATE INDEX idx_sessions_created_at ON sessions(created_at);
   ```

2. **Implement Session Store**
   - Create `src/controller/src/session_store.rs`
   - CRUD operations (create, read, update, delete)
   - Connection pooling (sqlx)
   - Transaction support

3. **Update Route Handlers**
   - POST /tasks/route: Store session on task routing
   - POST /approvals: Update session status on approval
   - GET /sessions/{id}: Return session from Postgres
   - GET /sessions: List sessions with filtering

4. **Privacy Guard Integration**
   - Store redacted data only
   - Track redaction metadata
   - Ensure no PII in database

5. **Testing**
   - Unit tests for session store
   - Integration tests for session lifecycle
   - Load tests for concurrent sessions

**Deliverables:**
- [ ] Database schema created
- [ ] Session store implementation complete
- [ ] Route handlers updated
- [ ] Tests passing (unit + integration)
- [ ] Documentation in docs/architecture/session-storage.md

**Estimated Time:** 4 hours

---

### 2.2 Implement fetch_status Tool (1 hour)

**Priority:** HIGH 🔴  
**Reason:** Tool currently returns 501, needed for workflow tracking

**Current State:**
- Tool implemented but Controller returns 501
- Tests skip fetch_status (documented limitation)

**Phase 4 Actions:**

1. **Update Controller**
   - Implement GET /sessions/{id}
   - Return session status, task details, approval status
   - Handle not found (404) vs server error (500)

2. **Update fetch_status Tool**
   - Remove 501 workaround
   - Parse session response
   - Display status to user

3. **Testing**
   - Update integration tests to include fetch_status
   - Test full workflow: send → approve → fetch
   - Test error cases (invalid ID, not found)

**Deliverables:**
- [ ] Controller GET /sessions/{id} working
- [ ] fetch_status tool returns real data
- [ ] Integration tests passing (4/4 tools = 100%)

**Estimated Time:** 1 hour

---

### 2.3 Idempotency Deduplication (1 hour)

**Priority:** MEDIUM 🟡  
**Reason:** Currently duplicate requests with same key both succeed

**Current State:**
- Idempotency-Key validated for format
- No deduplication (same key can be reused)
- Phase 3 design: stateless validation only

**Phase 4 Actions:**

1. **Redis Cache for Idempotency Keys**
   - Store: `idempotency:{key}` → `task_id`
   - TTL: 24 hours (configurable)
   - Check before processing request

2. **Update Controller Logic**
   - Check Redis for existing key
   - If found: Return cached response (task_id)
   - If not found: Process request, store in Redis

3. **Testing**
   - Test duplicate requests (same key)
   - Verify identical response returned
   - Test key expiration (TTL)

**Deliverables:**
- [ ] Redis cache configured
- [ ] Idempotency deduplication working
- [ ] Tests passing
- [ ] Metrics for cache hits/misses

**Estimated Time:** 1 hour

---

## 3. Privacy Guard Testing (33 hours) 🟡 MEDIUM PRIORITY

**See:** `docs/phase4/PRIVACY-GUARD-MCP-TESTING-PLAN.md`

**Summary:**
- Basic PII redaction tests: 4 hours
- Advanced NER model testing: 6 hours
- End-to-end workflow testing: 8 hours
- Performance & load testing: 6 hours
- Edge cases & error handling: 4 hours
- Infrastructure setup: 2 hours
- Documentation: 3 hours

**High Priority Issues:**
1. Load Ollama NER model (10 min + download)
2. Comprehensive integration tests (8h)
3. Performance benchmarking (6h)

**Total Estimated Effort:** 33 hours

**Reference:** Full details in PRIVACY-GUARD-MCP-TESTING-PLAN.md

---

## 4. Integration Test Updates (2 hours) 🔴 HIGH PRIORITY

### 4.1 Update Tests to Use JWT Tokens (1.5 hours)

**Priority:** HIGH 🔴  
**Reason:** Tests currently fail with HTTP 401 after JWT enabled

**Current State:**
- Tests use: `MESH_JWT_TOKEN=dummy-token-for-testing`
- Controller now requires: Valid JWT from Keycloak
- Result: All integration tests return HTTP 401

**Phase 4 Actions:**

1. **Create JWT Token Helper**
   ```python
   # Add to tests/conftest.py or tests/helpers.py
   import os
   import requests

   def get_jwt_token():
       """Get JWT token from Keycloak for testing."""
       response = requests.post(
           "http://localhost:8080/realms/dev/protocol/openid-connect/token",
           data={
               "username": "dev-agent",
               "password": "dev-password",
               "grant_type": "password",
               "client_id": "goose-controller",
               "client_secret": os.getenv("OIDC_CLIENT_SECRET")
           }
       )
       response.raise_for_status()
       return response.json()["access_token"]
   ```

2. **Update pytest Fixtures**
   ```python
   @pytest.fixture
   def jwt_token():
       """Fixture to provide valid JWT token."""
       return get_jwt_token()

   @pytest.fixture
   def check_controller_health(controller_url, jwt_token):
       """Verify Controller API is running before tests."""
       # Use real JWT instead of dummy token
       response = requests.get(
           f"{controller_url}/status",
           headers={"Authorization": f"Bearer {jwt_token}"}
       )
       if response.status_code != 200:
           pytest.skip(f"Controller API not healthy: {response.status_code}")
   ```

3. **Update Test Files**
   - `tests/test_integration.py` - Update all test functions
   - `tests/test_tools_without_jwt.py` - Rename to `test_tools_with_jwt.py`
   - `scripts/run_integration_tests.sh` - Add token acquisition
   - `scripts/test_manual.sh` - Add token acquisition

4. **Update Environment Variables**
   - Add `OIDC_CLIENT_SECRET` to test environment
   - Document in tests/README.md
   - Add to CI/CD configuration

5. **Testing**
   - Run all integration tests
   - Verify 100% pass rate (3/3 working tools)
   - fetch_status still returns 501 until session persistence added

**Deliverables:**
- [ ] JWT token helper implemented
- [ ] All test files updated
- [ ] Test scripts updated
- [ ] Tests passing with real JWT tokens
- [ ] Documentation updated

**Estimated Time:** 1.5 hours

---

### 4.2 Update Test Documentation (30 minutes)

**Priority:** MEDIUM 🟡  
**Reason:** Test setup instructions need JWT token setup

**Phase 4 Actions:**

1. **Update tests/README.md**
   - Add Keycloak setup instructions
   - Document OIDC_CLIENT_SECRET requirement
   - Add troubleshooting section (401 errors)

2. **Update Integration Test Summary**
   - Update `src/agent-mesh/B7-INTEGRATION-TEST-SUMMARY.md`
   - Change "dummy token" to "real JWT token"
   - Add token acquisition examples

3. **Create Test Setup Script**
   ```bash
   # scripts/setup-test-env.sh
   # - Check Keycloak running
   # - Check 'dev' realm exists
   # - Check 'dev-agent' user exists
   # - Get test JWT token
   # - Verify Controller API accessible
   ```

**Deliverables:**
- [ ] tests/README.md updated
- [ ] B7-INTEGRATION-TEST-SUMMARY.md updated
- [ ] setup-test-env.sh created
- [ ] CI/CD documentation updated

**Estimated Time:** 30 minutes

---

## 5. Schema & API Fixes (1 hour) 🟢 LOW PRIORITY

### 5.1 Fix notify Tool Schema (DONE ✅)

**Status:** COMPLETED in Phase 3

**Resolution:**
- Changed from `{type, message, priority}` to `{task_type, description, data}`
- Test result: HTTP 200 (was HTTP 422)
- No Phase 4 action needed

---

### 5.2 Verify All Tool Schemas Match Controller (30 minutes)

**Priority:** LOW 🟢  
**Reason:** Ensure no other schema mismatches

**Phase 4 Actions:**

1. **Schema Audit**
   - Review send_task payload schema
   - Review request_approval payload schema
   - Review notify payload schema (already fixed)
   - Compare with Controller OpenAPI spec

2. **Automated Schema Validation**
   - Create schema validation test
   - Compare tool payloads with OpenAPI schema
   - Fail if mismatch detected

3. **Documentation**
   - Update tool documentation with exact schemas
   - Add schema examples to README.md

**Deliverables:**
- [ ] Schema audit complete
- [ ] Schema validation test added
- [ ] Documentation updated

**Estimated Time:** 30 minutes

---

### 5.3 API Error Message Consistency (30 minutes)

**Priority:** LOW 🟢  
**Reason:** Ensure error messages are consistent and helpful

**Phase 4 Actions:**

1. **Error Message Audit**
   - Review all Controller error responses
   - Check for PII in error messages
   - Ensure consistent format

2. **Error Response Schema**
   ```json
   {
     "error": {
       "code": "INVALID_TARGET",
       "message": "Target role 'invalid-role' not found",
       "trace_id": "abc123",
       "timestamp": "2025-11-04T23:00:00Z"
     }
   }
   ```

3. **Update Controller**
   - Standardize error responses
   - Add error codes (for client handling)
   - Ensure Privacy Guard masks PII in errors

**Deliverables:**
- [ ] Error message audit complete
- [ ] Error response schema defined
- [ ] Controller updated with consistent errors
- [ ] Tests for error responses

**Estimated Time:** 30 minutes

---

## 6. Production Hardening (Varies) 🟡 MEDIUM PRIORITY

### 6.1 Observability & Monitoring (4 hours)

**Priority:** MEDIUM 🟡  
**Reason:** Production needs metrics, logging, tracing

**Phase 4 Actions:**

1. **Metrics (Prometheus)**
   - Request rates (req/s by endpoint)
   - Response times (p50, p95, p99)
   - Error rates (4xx, 5xx by endpoint)
   - Privacy Guard latency
   - Session store latency

2. **Distributed Tracing (Jaeger)**
   - Trace full request flow
   - Controller → Privacy Guard → Postgres
   - Correlate with trace_id

3. **Structured Logging**
   - Already implemented (JSON logs)
   - Add log aggregation (Loki, ELK)
   - Create dashboards (Grafana)

4. **Alerting**
   - High error rate (>5%)
   - Slow response time (p99 >5s)
   - Privacy Guard failures
   - Keycloak unavailable

**Deliverables:**
- [ ] Prometheus metrics exported
- [ ] Grafana dashboards created
- [ ] Jaeger tracing configured
- [ ] Alerts defined and tested

**Estimated Time:** 4 hours

---

### 6.2 Rate Limiting & Throttling (2 hours)

**Priority:** MEDIUM 🟡  
**Reason:** Protect against abuse and DoS

**Phase 4 Actions:**

1. **Rate Limiting (Redis)**
   - Per-user rate limits (e.g., 100 req/min)
   - Per-IP rate limits (e.g., 1000 req/min)
   - Global rate limits (e.g., 10000 req/min)

2. **Throttling**
   - Gradual backoff (429 responses)
   - Retry-After headers
   - Queue overflow handling

3. **Configuration**
   - Environment variables for limits
   - Different limits per environment (dev/staging/prod)

**Deliverables:**
- [ ] Rate limiting implemented (Redis)
- [ ] Throttling logic in place
- [ ] Tests for rate limit enforcement
- [ ] Documentation

**Estimated Time:** 2 hours

---

### 6.3 Deployment Automation (4 hours)

**Priority:** MEDIUM 🟡  
**Reason:** Production needs CI/CD, infrastructure as code

**Phase 4 Actions:**

1. **CI/CD Pipeline**
   - GitHub Actions (or equivalent)
   - Automated testing (unit + integration)
   - Docker image builds
   - Automated deployment

2. **Infrastructure as Code**
   - Terraform or Helm charts
   - Define all resources (DB, Keycloak, Controller, Privacy Guard)
   - Environment-specific configs

3. **Deployment Strategy**
   - Blue-green deployment
   - Rolling updates
   - Health checks before cutover
   - Rollback procedures

4. **Secrets Management**
   - Vault integration
   - Secret rotation automation
   - No secrets in code or env files

**Deliverables:**
- [ ] CI/CD pipeline working
- [ ] IaC templates created
- [ ] Deployment runbook documented
- [ ] Secrets managed via Vault

**Estimated Time:** 4 hours

---

## Summary by Priority

### HIGH PRIORITY 🔴 (Before Production)

| Item                              | Effort  | Phase 3 Blocker? |
|-----------------------------------|---------|------------------|
| Delete JWT documentation files    | 15 min  | Yes (before commit) |
| Regenerate client secret          | 30 min  | No               |
| Production Keycloak setup         | 4h      | No               |
| Postgres session storage          | 4h      | No               |
| Implement fetch_status tool       | 1h      | No               |
| Update integration tests (JWT)    | 1.5h    | Yes (B8)         |
| **Total**                         | **11.25h** | -            |

### MEDIUM PRIORITY 🟡 (Before Production)

| Item                              | Effort  |
|-----------------------------------|---------|
| JWT token management              | 2h      |
| Idempotency deduplication         | 1h      |
| Privacy Guard testing             | 33h     |
| Update test documentation         | 30 min  |
| Observability & monitoring        | 4h      |
| Rate limiting & throttling        | 2h      |
| Deployment automation             | 4h      |
| **Total**                         | **46.5h** |

### LOW PRIORITY 🟢 (Nice to Have)

| Item                              | Effort  |
|-----------------------------------|---------|
| Verify tool schemas               | 30 min  |
| API error message consistency     | 30 min  |
| **Total**                         | **1h**  |

### GRAND TOTAL: ~59 hours (7-8 days)

---

## References

### Phase 3 Documents
- Progress log: `docs/tests/phase3-progress.md`
- Agent state: `Technical Project Plan/PM Phases/Phase-3/Phase-3-Agent-State.json`
- Checklist: `Technical Project Plan/PM Phases/Phase-3/Phase-3-Checklist.md`
- Integration test summary: `src/agent-mesh/B7-INTEGRATION-TEST-SUMMARY.md`
- Security status: `src/agent-mesh/SECURITY-INTEGRATION-STATUS.md`

### Phase 4 Documents
- Privacy Guard testing: `docs/phase4/PRIVACY-GUARD-MCP-TESTING-PLAN.md`
- This document: `docs/phase4/PHASE-4-REQUIREMENTS.md`

### Source Code
- Controller: `src/controller/`
- Agent Mesh: `src/agent-mesh/`
- Privacy Guard: `src/privacy-guard/`

---

**Document Owner:** Phase 3 Agent (goose-org-twin)  
**Last Updated:** 2025-11-04 23:55 UTC  
**Status:** APPROVED for Phase 4 Planning
