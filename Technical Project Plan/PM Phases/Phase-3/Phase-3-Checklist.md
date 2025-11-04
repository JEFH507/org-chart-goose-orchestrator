# Phase 3 Checklist — Controller API + Agent Mesh

**Status:** 📋 READY  
**Total Tasks:** 28  
**Estimated Effort:** ~8-9 days  

---

## Workstream A: Controller API (Rust/Axum) - ~3 days

- [ ] A1. OpenAPI Schema Design (~4h)
  - [ ] Add utoipa + uuid dependencies to Cargo.toml
  - [ ] Create src/controller/src/api/openapi.rs
  - [ ] Define request/response schemas
  - [ ] Mount Swagger UI in main.rs
  - [ ] Verify Swagger UI accessible at /swagger-ui

- [ ] A2. Route Implementations (~1 day)
  - [ ] A2.1: POST /tasks/route (3h)
  - [ ] A2.2: GET /sessions (2h)
  - [ ] A2.3: POST /sessions (1h)
  - [ ] A2.4: POST /approvals (2h)
  - [ ] A2.5: GET /profiles/{role} (1h)

- [ ] A3. Idempotency + Request Limits Middleware (~4h)
  - [ ] Create middleware/idempotency.rs
  - [ ] Validate Idempotency-Key header
  - [ ] Add RequestBodyLimitLayer (1MB)
  - [ ] Test error responses (400, 413)

- [ ] A4. Privacy Guard Integration (~3h)
  - [ ] Implement mask_json utility
  - [ ] Integrate in POST /tasks/route
  - [ ] Log Privacy Guard latency

- [ ] A5. Unit Tests (~4h)
  - [ ] Test POST /tasks/route (6 cases)
  - [ ] Test GET /sessions
  - [ ] Test POST /approvals
  - [ ] Test GET /profiles/{role}
  - [ ] All tests pass with cargo test

**Progress:** 0% (0/5 tasks complete)

---

## Workstream B: Agent Mesh MCP (Python) - ~4-5 days

- [ ] B1. MCP Server Scaffold (~4h)
  - [ ] Create src/agent-mesh/ directory
  - [ ] Write pyproject.toml
  - [ ] Create agent_mesh_server.py
  - [ ] Install dependencies (mcp, requests, pydantic)
  - [ ] Test server starts successfully

- [ ] B2. send_task Tool (~6h)
  - [ ] Create tools/send_task.py
  - [ ] Implement retry logic (3x exponential backoff)
  - [ ] Add idempotency key generation
  - [ ] Test with Controller API

- [ ] B3. request_approval Tool (~4h)
  - [ ] Create tools/request_approval.py
  - [ ] Implement approval request
  - [ ] Test with Controller API

- [ ] B4. notify Tool (~3h)
  - [ ] Create tools/notify.py
  - [ ] Implement notification sending
  - [ ] Test with Controller API

- [ ] B5. fetch_status Tool (~3h)
  - [ ] Create tools/fetch_status.py
  - [ ] Implement status fetching
  - [ ] Test with Controller API

- [ ] B6. Configuration & Environment (~2h)
  - [ ] Create .env.example
  - [ ] Write README.md with setup instructions
  - [ ] Document Goose profiles.yaml integration

- [ ] B7. Integration Testing (~6h)
  - [ ] Write tests/test_integration.py
  - [ ] Test tool discovery
  - [ ] Test all 4 tools
  - [ ] All tests pass with pytest

- [ ] B8. Deployment & Docs (~4h)
  - [ ] Test with actual Goose instance
  - [ ] Verify tools visible in Goose
  - [ ] Update VERSION_PINS.md
  - [ ] **Create ADR-0024: Agent Mesh Python Implementation**

**Progress:** 0% (0/8 tasks complete)

---

## Workstream C: Cross-Agent Approval Demo - ~1 day

- [ ] C1. Demo Scenario Design (~2h)
  - [ ] Document scenario in docs/demos/cross-agent-approval.md
  - [ ] Define Finance → Manager flow

- [ ] C2. Implementation (~4h)
  - [ ] Set up 2 Goose instances
  - [ ] Execute Finance agent steps
  - [ ] Execute Manager agent steps
  - [ ] Verify approval workflow

- [ ] C3. Smoke Test Procedure (~2h)
  - [ ] Create docs/tests/smoke-phase3.md
  - [ ] Test Controller API health
  - [ ] Test Agent Mesh loading
  - [ ] Test cross-agent communication
  - [ ] Test audit trail
  - [ ] Test backward compatibility (Phase 1.2 + 2.2)
  - [ ] **Create ADR-0025: Controller API v1 Design**

**Progress:** 0% (0/3 tasks complete)

---

## Overall Progress

**Total:** 0% (0/28 tasks complete)  
**Time Spent:** 0 days  
**Time Remaining:** ~8-9 days

---

## ADRs to Create

- [ ] **ADR-0024:** Agent Mesh Python Implementation (Workstream B8)
- [ ] **ADR-0025:** Controller API v1 Design (Workstream C3)
