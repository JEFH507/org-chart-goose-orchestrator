# Phase 3 Checklist — Controller API + Agent Mesh

**Status:** 📋 READY  
**Total Tasks:** 31  
**Estimated Effort:** ~8-9 days  

---

## Workstream A: Controller API (Rust/Axum) - ~3 days

- [x] A1. OpenAPI Schema Design (~4h)
  - [x] Add utoipa + uuid dependencies to Cargo.toml
  - [x] Create src/controller/src/api/openapi.rs
  - [x] Define request/response schemas
  - [⏸️] Mount Swagger UI in main.rs _(deferred: utoipa-swagger-ui 4.0 incompatible with axum 0.7)_
  - [x] ~~Verify Swagger UI accessible at /swagger-ui~~ → OpenAPI JSON at /api-docs/openapi.json

- [x] A2. Route Implementations (~1 day)
  - [x] A2.1: POST /tasks/route (3h)
  - [x] A2.2: GET /sessions (2h)
  - [x] A2.3: POST /sessions (1h)
  - [x] A2.4: POST /approvals (2h)
  - [x] A2.5: GET /profiles/{role} (1h)

- [x] A3. Idempotency + Request Limits Middleware (~4h)
  - [x] ~~Create middleware/idempotency.rs~~ → Idempotency validation in route handler
  - [x] Validate Idempotency-Key header
  - [x] Add RequestBodyLimitLayer (1MB)
  - [⏸️] Test error responses (400, 413) → Will be tested in A5

- [x] A4. Privacy Guard Integration (~3h)
  - [x] Implement mask_json utility
  - [x] Integrate in POST /tasks/route
  - [x] Log Privacy Guard latency

- [⏸️] A5. Unit Tests (~4h) **83% COMPLETE**
  - [x] Created lib.rs and test infrastructure
  - [x] Test POST /tasks/route (6 test cases written)
  - [x] Test GET /sessions (1 test case)
  - [x] Test POST /sessions (2 test cases)
  - [x] Test POST /approvals (4 test cases)
  - [x] Test GET /profiles/{role} (4 test cases)
  - [⏸️] Fix test compilation (OpenAPI path references issue)
  - [ ] All tests pass with cargo test

- [ ] A6. Progress Tracking (~15 min) 🚨 MANDATORY CHECKPOINT
  - [ ] Update Phase-3-Agent-State.json (workstream A = COMPLETE)
  - [ ] Update Phase-3-Checklist.md (mark all A tasks [x])
  - [ ] Update docs/tests/phase3-progress.md (append Workstream A summary)
  - [ ] Commit changes to git
  - [ ] Report to user and WAIT for confirmation

**Progress:** 83% (5/6 tasks complete) - _A5: 83% done (needs compilation fix), A6 pending_

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

- [ ] B9. Progress Tracking (~15 min) 🚨 MANDATORY CHECKPOINT
  - [ ] Update Phase-3-Agent-State.json (workstream B = COMPLETE)
  - [ ] Update Phase-3-Checklist.md (mark all B tasks [x])
  - [ ] Update docs/tests/phase3-progress.md (append Workstream B summary)
  - [ ] Commit changes to git (include ADR-0024)
  - [ ] Report to user and WAIT for confirmation

**Progress:** 0% (0/9 tasks complete)

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

- [ ] C4. ADR-0025 Creation (~30 min)
  - [ ] **Create ADR-0025: Controller API v1 Design**
  - [ ] Document minimal API design decision
  - [ ] Document deferral of persistence to Phase 4

- [ ] C5. Progress Tracking (~15 min) 🚨 MANDATORY CHECKPOINT
  - [ ] Update Phase-3-Agent-State.json (status = COMPLETE)
  - [ ] Update Phase-3-Checklist.md (mark all C tasks [x])
  - [ ] Update docs/tests/phase3-progress.md (append Workstream C + completion summary)
  - [ ] Create Phase-3-Completion-Summary.md
  - [ ] Update CHANGELOG.md
  - [ ] Commit changes to git (include ADR-0025)
  - [ ] Report to user - Phase 3 COMPLETE

**Progress:** 0% (0/5 tasks complete)

---

## Overall Progress

**Total:** 16% (5/31 tasks complete)  
**Workstream A:** 83% complete (5/6 tasks)  
**Time Spent:** <1 day  
**Time Remaining:** ~8 days

---

## Progress Log Tracking

- [ ] docs/tests/phase3-progress.md created at start of Phase 3
- [ ] Progress log updated after Workstream A (Checkpoint 1)
- [ ] Progress log updated after Workstream B (Checkpoint 2)
- [ ] Progress log updated after Workstream C (Checkpoint 3 - final)
- [ ] Progress log complete with all sections filled

---

## ADRs to Create

- [ ] **ADR-0024:** Agent Mesh Python Implementation (Workstream B8)
- [ ] **ADR-0025:** Controller API v1 Design (Workstream C4)
