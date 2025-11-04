# Phase 3 Pre-Flight Analysis — Controller API + Agent Mesh
**Date:** 2025-11-04  
**Analyst:** Goose AI Agent  
**Status:** REVIEW PENDING  

---

## Executive Summary

Phase 3 aims to deliver **multi-agent orchestration** through two core components:
1. **Controller API:** HTTP endpoints for task routing, approvals, sessions, and audit
2. **Agent Mesh MCP Extension:** Cross-agent communication tools (send_task, request_approval, notify, fetch_status)

**Recommendation:** ✅ **PROCEED with modifications**  
- All technical prerequisites MET (Phase 1.2 JWT auth, Phase 2.2 Privacy Guard)
- Dependencies available in Rust ecosystem (Axum 0.7, utoipa, rmcp SDK)
- Product requirements clear and aligned with master plan
- **Gaps identified:** Need to design Agent Mesh as Python MCP server (not Rust; follows Goose architecture pattern)

---

## 1. Product & Business Alignment

### 1.1 Product Requirements (from productdescription.md)

**Core Capabilities Needed:**
- ✅ **Orchestrated Tasks:** Route tasks to appropriate agent(s) based on role/skill
- ✅ **Cross-Agent Session Broker:** Maintain scoped context shards per agent
- ✅ **Approval Workflows:** Manager/peer approvals with audit trail
- ✅ **Audit & Observability:** Who/what/when/where tracking across agents

**Value Proposition Alignment:**
- ✅ **Hierarchical Orchestration:** Controller maps to org chart (C-suite → Department → Manager → IC)
- ✅ **Policy-Driven:** RBAC/ABAC enforcement at directory level
- ✅ **Privacy by Design:** All content flows through Privacy Guard (Phase 2.2)
- ✅ **Vendor-Neutral:** MCP-first tools; HTTP-only (no bus required for MVP)

### 1.2 Master Plan Alignment (master-technical-project-plan.md)

**Phase 3 Scope (L - Large: ~1-2 weeks):**
- ✅ Minimal OpenAPI covering /tasks/route, /sessions, /approvals, /status, /profiles (proxy), /audit/ingest
- ✅ Idempotency keys; request size limits (1MB); structured logs with traceId
- ✅ MCP extension exposing: send_task, request_approval, notify, fetch_status
- ✅ All calls go through Controller API with Bearer JWT and policy evaluation
- ✅ Integration test: cross-agent approval demo (stub OK for MVP)

**Success Criteria:**
- M2 Milestone (Week 4): Controller API + Mesh verbs functional; cross-agent approval demo; audit events emitted
- Acceptance: Mesh verbs visible in Goose tool list; passing integration tests

---

## 2. Technical Architecture Review

### 2.1 Goose v1.12 Reference Architecture

**MCP Extension Pattern (from goose-mcp/developer):**
```rust
// 1. Extension implements ServerHandler trait (rmcp SDK)
#[derive(Clone)]
pub struct DeveloperServer {
    tool_router: ToolRouter<Self>,
    // state/config
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for DeveloperServer {
    fn get_info(&self) -> ServerInfo { /* ... */ }
    fn list_prompts(...) { /* ... */ }
    fn get_prompt(...) { /* ... */ }
}

// 2. Tools defined with #[tool] macro
#[tool_router(router = tool_router)]
impl DeveloperServer {
    #[tool(name = "shell", description = "...")]
    pub async fn shell(&self, params: Parameters<ShellParams>) -> Result<CallToolResult, ErrorData> {
        // tool logic
    }
}

// 3. Entry point: MCP server binary (stdio transport)
fn main() {
    let server = DeveloperServer::new();
    // serve via stdio/SSE/HTTP
}
```

**Key Insights:**
- ✅ **Rust-based MCP servers:** Goose v1.12 uses rmcp SDK (official Rust MCP library)
- ✅ **Tool pattern:** Parameters struct + CallToolResult + ErrorData
- ✅ **Streaming notifications:** peer.notify_logging_message for real-time updates
- ✅ **State management:** Arc/RwLock for shared state across tool calls
- ❓ **Question:** Do we build Agent Mesh in Rust or Python?

**Decision Required:**  
Goose v1.12 **first-party extensions** (developer, computercontroller, memory) are **Rust-based** using `rmcp` SDK.  
However, **third-party extensions** can be Python/TypeScript/Go (MCP protocol is language-agnostic).

**Recommendation:**  
- **Option A (Rust):** Build Agent Mesh as Rust MCP server (consistent with Goose first-party extensions)
  - **Pros:** Type safety, performance, consistent with developer/computercontroller pattern
  - **Cons:** Steeper learning curve, more boilerplate than Python
- **Option B (Python):** Build Agent Mesh as Python MCP server using `mcp` SDK
  - **Pros:** Faster prototyping, simpler HTTP client code, easier to iterate
  - **Cons:** Runtime dependency (Python 3.10+), less type safety

**USER DECISION NEEDED:** Rust or Python for Agent Mesh MCP server?

### 2.2 Controller API Architecture

**Current Implementation (from src/controller/src/main.rs):**
```rust
// Axum-based HTTP server
// Ports: 8088 (controller), 8089 (privacy-guard)
// Auth: JWT verification middleware (Phase 1.2)
// Routes:
//   GET /status → 200 (public)
//   POST /audit/ingest → 202 (JWT protected, privacy guard integrated)
//   Fallback → 501 (not implemented)
```

**Phase 3 Additions:**
```
POST /tasks/route          → 202 (JWT protected)
GET  /sessions             → 200 (JWT protected)
POST /sessions             → 201 (JWT protected)
GET  /sessions/{id}        → 200 (JWT protected)
POST /approvals            → 202 (JWT protected)
GET  /approvals/{id}       → 200 (JWT protected)
GET  /profiles/{role}      → 200 (JWT protected, proxy to directory)
```

**Dependencies (Cargo.toml):**
```toml
axum = "0.7"               # HTTP framework ✅
tokio = "1.40"              # Async runtime ✅
serde/serde_json = "1.0"    # JSON serialization ✅
tracing = "0.1"             # Logging ✅
jsonwebtoken = "9.3"        # JWT (Phase 1.2) ✅
reqwest = "0.12"            # HTTP client (for privacy guard) ✅

# Phase 3 additions:
utoipa = "4.0"              # OpenAPI generation (NEEDED)
utoipa-swagger-ui = "6.0"   # Swagger UI (optional, nice-to-have)
uuid = "1.6"                # ID generation for sessions/tasks (NEEDED)
```

**OpenAPI Generation Strategy:**
- **Approach:** Use `utoipa` derive macros (like Goose v1.12 goose-server)
- **Build step:** Generate openapi.json at compile time (build.rs)
- **Validation:** Spectral CI checks (already configured in .spectral.yaml)

---

## 3. Dependency Analysis

### 3.1 Infrastructure Dependencies (from VERSION_PINS.md)

| Component | Version | Status | Notes |
|-----------|---------|--------|-------|
| **Keycloak** | 24.0.4 | ✅ READY | OIDC provider; dev realm seeded (Phase 1.2) |
| **Vault** | 1.17.6 | ✅ READY | Secrets management; pseudo_salt path configured (Phase 1.2) |
| **Postgres** | 16.4-alpine | ✅ READY | Metadata storage; migrations stubbed (Phase 0) |
| **Ollama** | 0.12.9 | ✅ READY | Privacy Guard model (Phase 2.2) |
| **Privacy Guard** | 0.2.2 | ✅ READY | Model-enhanced PII detection (Phase 2.2) |

**Verdict:** ✅ All infrastructure dependencies operational

### 3.2 Rust Ecosystem Dependencies

| Crate | Version | Purpose | Availability |
|-------|---------|---------|--------------|
| **axum** | 0.7 | HTTP framework | ✅ Already in use |
| **utoipa** | 4.0 | OpenAPI generation | ✅ Available (crates.io) |
| **utoipa-swagger-ui** | 6.0 | Swagger UI | ✅ Optional (nice-to-have) |
| **uuid** | 1.6 | ID generation | ✅ Available (crates.io) |
| **rmcp** | 0.6.0 | MCP SDK (if Rust Agent Mesh) | ✅ Available (Goose uses this) |
| **mcp** (Python) | 1.0+ | MCP SDK (if Python Agent Mesh) | ✅ Available (PyPI) |

**Verdict:** ✅ All dependencies available; no blockers

### 3.3 MCP SDK Comparison (Rust vs Python)

#### Rust (`rmcp` 0.6.0)
```rust
// Pros:
// - Type-safe tool parameters via serde + JsonSchema
// - Integrated with Goose first-party extensions
// - Performance (compiled binary)
// - Consistent with controller implementation

// Cons:
// - More boilerplate (trait implementations, macros)
// - HTTP client code more verbose
// - Steeper learning curve

// Example tool:
#[tool(name = "send_task", description = "Route task to agent")]
pub async fn send_task(&self, params: Parameters<SendTaskParams>) -> Result<CallToolResult, ErrorData> {
    let response = self.http_client
        .post(&format!("{}/tasks/route", self.controller_url))
        .bearer_auth(&self.jwt_token)
        .json(&params.0)
        .send()
        .await?;
    // ...
}
```

#### Python (`mcp` 1.0+)
```python
# Pros:
# - Simpler syntax, faster prototyping
# - Easier HTTP client code (requests library)
# - Goose supports Python MCP servers (stdio transport)

# Cons:
# - Runtime dependency (Python 3.10+)
# - Less type safety (Pydantic helps but not compile-time)
# - Deployment: need Python + dependencies in container

# Example tool:
@app.call_tool()
async def send_task(target: str, task: dict, context: dict) -> dict:
    response = requests.post(
        f"{CONTROLLER_URL}/tasks/route",
        headers={"Authorization": f"Bearer {jwt_token}"},
        json={"target": target, "task": task, "context": context}
    )
    return {"taskId": response.json()["id"]}
```

**Recommendation:** See Section 2.1 - USER DECISION NEEDED

---

## 4. Technical Design Proposal

### 4.1 Workstream Breakdown

**Phase 3 consists of TWO parallel workstreams:**

#### Workstream A: Controller API (Rust, ~1 week)
**Objective:** Implement HTTP endpoints for orchestration

**Tasks:**
1. **A1 - OpenAPI Schema Design** (~4 hours)
   - Define utoipa schemas for Task, Session, Approval, Profile
   - Add `#[derive(utoipa::ToSchema)]` to request/response structs
   - Generate openapi.json via build.rs

2. **A2 - Route Implementations** (~1 day)
   - POST /tasks/route (stub: accept task, assign ID, return 202)
   - GET/POST /sessions (stub: create/list sessions with metadata-only)
   - POST /approvals (stub: record approval decision, return 202)
   - GET /profiles/{role} (proxy to directory service - stub for now)

3. **A3 - Idempotency & Request Limits** (~4 hours)
   - Add `Idempotency-Key` header middleware
   - Add request size limit (1MB) via tower-http
   - Add structured logging with traceId propagation

4. **A4 - Integration with Privacy Guard** (~3 hours)
   - Apply guard_client.mask_text to task/session content fields
   - Preserve redaction maps in audit events

5. **A5 - Unit Tests** (~4 hours)
   - Test each route handler
   - Test idempotency logic
   - Test JWT middleware integration

**Dependencies:** Phase 1.2 (JWT auth), Phase 2.2 (Privacy Guard)  
**Estimated Effort:** ~3 days  
**Deliverables:**
- Updated `src/controller/src/main.rs` with new routes
- New files: `src/controller/src/routes.rs`, `src/controller/src/schemas.rs`
- `docs/api/controller/openapi.yaml` updated (auto-generated)
- Unit tests in `src/controller/tests/`

#### Workstream B: Agent Mesh MCP Extension (~4-5 days)
**Objective:** Enable cross-agent communication via MCP tools

**Tasks:**
1. **B1 - MCP Server Scaffold** (~4 hours)
   - Choose language (Rust or Python) - **USER DECISION**
   - Set up project structure (Rust: Cargo.toml, Python: pyproject.toml)
   - Implement ServerHandler/get_info boilerplate

2. **B2 - Tool: send_task** (~6 hours)
   - Parameters: target (agent ID), task (dict), context (dict), policyHints (optional)
   - HTTP POST to /tasks/route
   - Bearer JWT from environment (MESH_JWT_TOKEN)
   - Retry logic: 3x with exponential backoff + jitter
   - Idempotency-Key header (UUID v4)
   - Returns: taskId

3. **B3 - Tool: request_approval** (~4 hours)
   - Parameters: sessionId, stepId, approver (role/agent ID), payload (dict)
   - HTTP POST to /approvals
   - Returns: approvalId

4. **B4 - Tool: notify** (~3 hours)
   - Parameters: target (agent ID), message (string), severity (info/warn/error)
   - HTTP POST to /tasks/route with special "notification" task type
   - Returns: ack (boolean)

5. **B5 - Tool: fetch_status** (~3 hours)
   - Parameters: taskId or sessionId
   - HTTP GET to /tasks/{id} or /sessions/{id}
   - Returns: status object (state, progress, metadata)

6. **B6 - Configuration & Environment** (~2 hours)
   - Environment variables: CONTROLLER_URL, MESH_JWT_TOKEN, MESH_RETRY_COUNT, MESH_TIMEOUT_SECS
   - Default values: localhost:8088, 3 retries, 30s timeout

7. **B7 - Integration Testing** (~6 hours)
   - Test each tool against local controller instance
   - Mock JWT token (use Keycloak dev realm)
   - Verify idempotency, retry, timeout logic

8. **B8 - Deployment & Documentation** (~4 hours)
   - Dockerfile (if Python) or binary build (if Rust)
   - Docker Compose service definition
   - Usage guide (docs/guides/agent-mesh-usage.md)

**Dependencies:** Workstream A (Controller API routes)  
**Estimated Effort:** ~4-5 days  
**Deliverables:**
- New directory: `src/agent-mesh-mcp/` (Rust) or `mcp-servers/agent-mesh/` (Python)
- Binary or Python package
- Docker Compose service (optional for MVP)
- Integration tests
- Documentation

#### Workstream C: Cross-Agent Integration Demo (~1 day)
**Objective:** Prove multi-agent approval workflow end-to-end

**Tasks:**
1. **C1 - Demo Scenario Design** (~2 hours)
   - Scenario: Finance agent requests approval from Manager agent
   - Setup: 2 Goose instances with Agent Mesh extension enabled
   - Flow: Finance.send_task → Controller routes → Manager.request_approval → Finance.fetch_status

2. **C2 - Demo Implementation** (~4 hours)
   - Configure 2 Goose agents with different profiles (finance_agent, manager_agent)
   - Enable Agent Mesh extension in both
   - Run demo script (bash or Python)
   - Capture audit events

3. **C3 - Smoke Test Procedure** (~2 hours)
   - Document manual test steps (docs/tests/smoke-phase3.md)
   - Expected outputs and pass criteria
   - Troubleshooting guide

**Dependencies:** Workstreams A + B  
**Estimated Effort:** ~1 day  
**Deliverables:**
- Demo script: `scripts/demo-cross-agent-approval.sh`
- Smoke test procedure: `docs/tests/smoke-phase3.md`
- Demo video or transcript (optional)

### 4.2 Timeline

**Total Estimated Effort:** ~8-9 days (Large phase, per master plan)

```
Week 1:
  Days 1-3: Workstream A (Controller API)
  Days 4-5: Workstream B start (MCP server scaffold + send_task/request_approval)

Week 2:
  Days 6-7: Workstream B complete (notify/fetch_status + integration tests)
  Day 8: Workstream C (demo + smoke tests)
  Day 9: Buffer & documentation polish
```

**Milestones:**
- M1 (Day 3): Controller API routes implemented, OpenAPI published
- M2 (Day 7): Agent Mesh tools working, integration tests pass
- M3 (Day 8): Cross-agent approval demo successful
- M4 (Day 9): Phase 3 completion summary, PR ready

---

## 5. Gaps & Risks Analysis

### 5.1 Technical Gaps

| Gap | Severity | Mitigation |
|-----|----------|------------|
| **Agent Mesh language choice** | HIGH | **USER DECISION NEEDED:** Rust or Python? |
| **Directory/Policy service stub** | MEDIUM | Defer to Phase 4; hardcode profile responses for Phase 3 demo |
| **Session storage** | MEDIUM | Use in-memory HashMap for MVP; migrate to Postgres in Phase 7 |
| **Task queue** | LOW | Stub task routing (first-available agent); implement proper routing in Phase 6 |

### 5.2 Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **MCP SDK API changes** | LOW | HIGH | Pin `rmcp` or `mcp` versions; test against Goose v1.12 reference |
| **JWT token management in Agent Mesh** | MEDIUM | MEDIUM | Use environment variable for dev; document Vault integration for prod |
| **Cross-agent communication latency** | MEDIUM | MEDIUM | Accept 5s P50 for Phase 3; optimize in Phase 6 (model orchestration) |
| **Idempotency key collisions** | LOW | MEDIUM | Use UUID v4 (128-bit); collision probability negligible |
| **Request size limits too restrictive** | LOW | LOW | Start with 1MB; allow override via env var (MAX_REQUEST_SIZE) |
| **Privacy Guard performance impact** | MEDIUM | MEDIUM | Already measured in Phase 2.2 (P50=22.8s); acceptable for audit use case |

### 5.3 Dependencies on Future Phases

| Phase | Dependency | Impact on Phase 3 |
|-------|------------|-------------------|
| **Phase 4 (Directory/Policy)** | Profile service, policy evaluation | **STUB REQUIRED:** Hardcode role profiles for demo |
| **Phase 5 (Audit)** | Postgres audit index | **STUB OK:** Log to JSON for now, migrate later |
| **Phase 6 (Model Orchestration)** | Task routing by skill/load | **STUB OK:** First-available routing for demo |
| **Phase 7 (Storage/Metadata)** | Session persistence | **IN-MEMORY OK:** Use HashMap, acceptable for demo |

**Verdict:** ✅ All dependencies can be stubbed for Phase 3 MVP

---

## 6. Product-Technical Alignment Matrix

| Product Requirement | Technical Implementation | Phase 3 Deliverable | Gap? |
|---------------------|-------------------------|---------------------|------|
| **Orchestrated Tasks** | POST /tasks/route | Stub route + send_task tool | ✅ ALIGNED |
| **Cross-Agent Communication** | Agent Mesh MCP tools | send_task, request_approval, notify, fetch_status | ✅ ALIGNED |
| **Approval Workflows** | POST /approvals | Stub route + request_approval tool | ✅ ALIGNED |
| **Session Management** | GET/POST /sessions | Stub routes with in-memory storage | ⚠️ PARTIAL (Postgres in Phase 7) |
| **Audit Trail** | Audit events via /audit/ingest | Already implemented (Phase 1.2) | ✅ ALIGNED |
| **Policy Enforcement** | GET /profiles/{role} + policy eval | Stub proxy to directory (Phase 4) | ⚠️ PARTIAL (Policy in Phase 4) |
| **Privacy by Design** | Privacy Guard integration | guard_client.mask_text in routes | ✅ ALIGNED (Phase 2.2) |

**Verdict:** ✅ 5/7 full alignment, 2/7 partial (acceptable for MVP with stubs)

---

## 7. Goose Architecture Compatibility

### 7.1 MCP Extension Integration

**Goose v1.12 Extension Loading:**
```yaml
# ~/.config/goose/profiles.yaml
extensions:
  agent_mesh:
    type: mcp
    command: ["path/to/agent-mesh-server"]  # stdio transport
    env:
      CONTROLLER_URL: "http://localhost:8088"
      MESH_JWT_TOKEN: "eyJ..."
      MESH_RETRY_COUNT: "3"
      MESH_TIMEOUT_SECS: "30"
```

**Tool Discovery:**
- Goose loads MCP server via stdio transport
- ExtensionManager calls `list_tools` → gets 4 tools (send_task, request_approval, notify, fetch_status)
- Tools appear in LLM context with prefix: `agent_mesh__send_task`, etc.

**Tool Invocation:**
- Agent calls `agent_mesh__send_task(target="manager_agent", task={...})`
- MCP server makes HTTP POST to Controller API
- Returns `{taskId: "uuid-..."}` to agent

**Verdict:** ✅ Goose architecture fully supports our design

### 7.2 Controller API as Axum Service

**Deployment Pattern:**
- **Dev:** `cargo run --bin goose-controller` (like `goosed`)
- **Prod:** Docker Compose service (like privacy-guard)
- **Ports:** 8088 (controller), 8089 (privacy-guard), 8080 (keycloak), 8200 (vault)

**Service Discovery:**
- Agents configure Controller URL via environment (`CONTROLLER_URL=http://localhost:8088`)
- Docker Compose: use service names (`controller:8088` in network)

**Verdict:** ✅ Consistent with existing architecture

---

## 8. User Decision Points

### 8.1 CRITICAL DECISION: Agent Mesh Language

**Question:** Should Agent Mesh MCP server be implemented in **Rust** or **Python**?

**Option A: Rust**
- ✅ Pros: Type safety, performance, consistent with Goose first-party extensions
- ❌ Cons: More boilerplate, steeper learning curve
- **Recommendation:** Choose if prioritizing long-term maintainability and consistency

**Option B: Python**
- ✅ Pros: Faster prototyping, simpler HTTP client, easier iteration
- ❌ Cons: Runtime dependency, less type safety
- **Recommendation:** Choose if prioritizing rapid MVP delivery

**USER INPUT REQUIRED:** Which language do you prefer for Agent Mesh?

### 8.2 Optional Decisions (Can Defer)

1. **Swagger UI:** Include utoipa-swagger-ui for interactive API docs?
   - **Default:** Yes (nice-to-have, minimal effort)

2. **Request Size Limit:** 1MB default OK or adjust?
   - **Default:** 1MB (sufficient for task payloads)

3. **Session Storage:** In-memory HashMap or Postgres from start?
   - **Default:** HashMap (migrate to Postgres in Phase 7)

4. **Retry Policy:** 3 retries with exponential backoff OK?
   - **Default:** Yes (industry standard)

---

## 9. Recommended Next Steps

### 9.1 Immediate Actions

1. **USER DECISION:** Choose Agent Mesh language (Rust or Python)
2. **Review & Approve:** This pre-flight analysis document
3. **Create Artifacts:**
   - Phase-3-Execution-Plan.md (3 workstreams: A, B, C)
   - Phase-3-Checklist.md (tasks with time estimates)
   - Phase-3-Agent-State.json (tracking template)
   - Phase-3-Orchestration-Prompt.md (copy-paste prompt for new session)

### 9.2 Artifact Structure (To Be Created)

```
Technical Project Plan/PM Phases/Phase-3/
├── Phase-3-Execution-Plan.md       # Workstreams A/B/C breakdown
├── Phase-3-Checklist.md            # Task checklist with % completion
├── Phase-3-Agent-State.json        # Real-time state tracking
├── Phase-3-Orchestration-Prompt.md # Master prompt for execution session
└── Phase-3-Completion-Summary.md   # (Created at end of phase)
```

### 9.3 Execution Workflow (Same as Phase 2.2)

1. **Create artifacts** (this document + execution plan + checklist + state JSON)
2. **User reviews** artifacts and approves language choice
3. **Copy Phase-3-Orchestration-Prompt.md** to new Goose session
4. **Execute Phase 3** following checklist:
   - Workstream A → Workstream B → Workstream C
   - Update state JSON after each task
   - Append progress log entries
   - Commit frequently with conventional commits
5. **Complete Phase 3:**
   - Run smoke tests (C3)
   - Create completion summary
   - Merge PR to main
   - Tag release

---

## 10. Final Recommendation

### 10.1 GO/NO-GO Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| **Product Alignment** | ✅ GO | Clear value proposition; aligns with master plan |
| **Technical Feasibility** | ✅ GO | All dependencies available; no blockers |
| **Prerequisites Complete** | ✅ GO | Phase 1.2 (JWT) + Phase 2.2 (Privacy Guard) done |
| **Risks Manageable** | ✅ GO | All risks have mitigations; no showstoppers |
| **Scope Reasonable** | ✅ GO | 8-9 days estimated (Large phase, per plan) |
| **User Decision Needed** | ⚠️ PENDING | Language choice for Agent Mesh (Rust vs Python) |

**VERDICT:** ✅ **RECOMMEND GO** (pending Agent Mesh language decision)

### 10.2 Success Criteria (From Master Plan)

**M2 Milestone (Week 4):**
- ✅ Controller API + Mesh verbs functional
- ✅ Cross-agent approval demo working
- ✅ Audit events emitted

**Acceptance Criteria:**
- ✅ Mesh verbs visible in Goose tool list
- ✅ Passing integration tests
- ✅ OpenAPI spec published and Spectral checks pass

---

## Appendix A: Reference Documents

### A.1 Master Plan
- **Path:** `Technical Project Plan/master-technical-project-plan.md`
- **Relevant Sections:** Phase 3 WBS, Timeline (Weeks 3–5), Success Criteria

### A.2 ADRs
- **ADR-0007:** Agent Mesh MCP (MCP verbs, Controller-only routing, no P2P)
- **ADR-0010:** Controller OpenAPI (idempotency, size limits, structured logs)

### A.3 Component Plans
- **Controller API:** `Technical Project Plan/components/controller-api/requirements.md`
- **Agent Mesh:** `Technical Project Plan/components/agent-mesh-mcp/requirements.md`

### A.4 Goose Reference
- **Architecture Report:** `goose-versions-references/how-goose-works-docs/docs/goose-v1.12.00-technical-architecture-report.md`
- **MCP Example:** `goose-versions-references/gooseV1.12.00/crates/goose-mcp/src/developer/rmcp_developer.rs`

---

**Prepared by:** Goose AI Agent  
**Date:** 2025-11-04  
**Review Required:** User approval on Agent Mesh language choice  
**Next Action:** Create Phase 3 execution artifacts upon approval
