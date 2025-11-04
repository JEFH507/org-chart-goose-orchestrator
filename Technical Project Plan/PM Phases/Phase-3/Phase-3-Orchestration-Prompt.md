# Phase 3 Orchestration Prompt

**Copy this entire prompt to a new Goose session to execute Phase 3**

---

## 🔄 Resume Prompt — Copy this block if resuming Phase 3

```markdown
You are resuming Phase 3 orchestration for goose-org-twin.

**Context:**
- Phase: 3 — Controller API + Agent Mesh (Medium - ~8-9 days)
- Repository: /home/papadoc/Gooseprojects/goose-org-twin

**Required Actions:**
1. Read state from: `Technical Project Plan/PM Phases/Phase-3/Phase-3-Agent-State.json`
2. Read last progress entry from: `docs/tests/phase3-progress.md` (if exists)
3. Re-read authoritative documents:
   - `Technical Project Plan/master-technical-project-plan.md`
   - `Technical Project Plan/PM Phases/Phase-3/Phase-3-Execution-Plan.md`
   - `Technical Project Plan/PM Phases/Phase-3/Phase-3-Checklist.md`
   - `Technical Project Plan/PM Phases/Phase-3/Phase-3-Orchestration-Prompt.md` (this file)
   - `Technical Project Plan/PM Phases/Phase-3-PRE-FLIGHT-ANALYSIS.md`
   - Relevant ADRs: `docs/adr/0007-agent-mesh-mcp.md`, `docs/adr/0010-controller-openapi-and-http-interfaces.md`

**Summarize for me:**
- Current workstream and task from state JSON (A/B/C)
- Last step completed (from progress.md or state JSON)
- Checklist progress (X/28 tasks complete)
- Milestones achieved (M1/M2/M3/M4)
- Pending questions (if any in state JSON)
- ADRs status: ADR-0024 created? ADR-0025 created?

**Then proceed with:**
- If pending_questions exist: ask them and wait for my answers
- Otherwise: continue with the next unchecked task in the checklist
- Update state JSON and progress log after each task/milestone

**Guardrails:**
- HTTP-only orchestrator; metadata-only server model
- No secrets in git; .env samples only
- Update state JSON and progress log after each milestone
- Create ADR-0024 (Workstream B8) and ADR-0025 (Workstream C3) before marking phase complete
- Before starting, check Phase-2.5/ folder for changes from dependency upgrades
```

---

## 🚀 Master Orchestration Prompt — Copy this block for new session

You are executing **Phase 3: Controller API + Agent Mesh** for the goose-org-twin project.

## 📋 Context

**Project:** goose-org-twin (Multi-agent orchestration system)  
**Repository:** git@github.com:JEFH507/org-chart-goose-orchestrator.git  
**Current Branch:** main  
**Phase:** 3 (Controller API + Agent Mesh - M2 Milestone)  
**Priority:** 🔴 HIGH (Core orchestration capability)  
**Estimated Effort:** 8-9 days (~2 weeks)

### Prerequisites (Must be Completed)
- ✅ Phase 0: Infrastructure bootstrap
- ✅ Phase 1: Basic Controller skeleton
- ✅ Phase 1.2: JWT verification middleware  
- ✅ Phase 2: Vault integration
- ✅ Phase 2.2: Privacy Guard with Ollama model
- ✅ **Phase 2.5: Dependency upgrades (Keycloak 26, Vault 1.18, Python 3.13, Rust 1.83)**
  - Note: Rust 1.91.0 was tested but deferred (requires Clone derives on structs)

**IMPORTANT:** Before starting, check `Technical Project Plan/PM Phases/Phase-2.5/` folder for any changes from dependency upgrades.

### Blocks
- ⏸️ Phase 4: Directory Service + Policy Engine (waiting for this phase)

---

## 🎯 Objectives

### Primary Goal
Implement core multi-agent orchestration capability through:

1. **Controller API (Rust/Axum):** HTTP endpoints for task routing, session management, approval workflows
2. **Agent Mesh MCP (Python):** MCP extension for Goose with 4 tools for multi-agent communication  
3. **Cross-Agent Demo:** Finance agent → Manager agent approval flow

### Success Criteria (M2 Milestone)
- ✅ Controller API: 5 routes functional (POST /tasks/route, GET/POST /sessions, POST /approvals, GET /profiles/{role})
- ✅ OpenAPI spec published and validated (Spectral lint passes)
- ✅ Swagger UI accessible at http://localhost:8088/swagger-ui
- ✅ Agent Mesh MCP: Extension loads in Goose
- ✅ Agent Mesh MCP: All 4 tools functional (send_task, request_approval, notify, fetch_status)
- ✅ Cross-agent approval demo works end-to-end (Finance → Manager)
- ✅ Audit events emitted with traceId propagation
- ✅ Integration tests pass (100%)
- ✅ Smoke tests pass (5/5)
- ✅ **ADR-0024 created: "Agent Mesh Python Implementation"**
- ✅ **ADR-0025 created: "Controller API v1 Design"**
- ✅ No breaking changes to Phase 1.2/2.2 functionality

---

## 📦 Components to Build

### Controller API (Rust/Axum)
**Location:** `src/controller/` (extend existing)

**New Dependencies:**
```toml
# Add to src/controller/Cargo.toml
utoipa = { version = "4.0", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "4.0", features = ["axum"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
tower-http = { version = "0.5", features = ["limit"] }
```

**New Routes:**
- POST /tasks/route - Route task to target agent (202 Accepted)
- GET /sessions - List active sessions (200 OK)
- POST /sessions - Create new session (201 Created)
- POST /approvals - Submit approval decision (202 Accepted)
- GET /profiles/{role} - Get agent profile (200 OK)

**Middleware:**
- Idempotency key validation (Idempotency-Key header, UUID format)
- Request size limit: 1MB (413 Payload Too Large)
- traceId extraction/generation (X-Trace-Id header)
- Privacy Guard integration (mask sensitive data in task/context)

### Agent Mesh MCP (Python)
**Location:** `src/agent-mesh/` (new directory)

**Structure:**
```
src/agent-mesh/
├── pyproject.toml
├── agent_mesh_server.py
├── tools/
│   ├── send_task.py
│   ├── request_approval.py
│   ├── notify.py
│   └── fetch_status.py
├── client/
│   └── controller_client.py
├── .env.example
└── README.md
```

**Dependencies:**
- mcp >= 1.0.0 (MCP SDK for Python)
- requests >= 2.31.0 (HTTP client)
- pydantic >= 2.0.0 (Data validation)

**Tools:**
1. send_task(target, task, context) → {taskId, status}
2. request_approval(task_id, approver_role, reason) → {approvalId, status}
3. notify(target, message, priority) → {ack: true}
4. fetch_status(task_id) → {status, progress, result}

---

## 🔧 Execution Plan (3 Workstreams)

### Workstream A: Controller API (Rust/Axum) - Days 1-3

Execute in order:

**Day 1: OpenAPI + Basic Routes (~8h)**

A1. OpenAPI Schema Design (~4h)
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/src/controller

# Add dependencies to Cargo.toml
# Create src/api/openapi.rs with utoipa structs
# Mount Swagger UI in main.rs

# Test
cargo build
curl http://localhost:8088/swagger-ui  # Should show Swagger UI
curl http://localhost:8088/api-docs/openapi.json  # Should return OpenAPI spec
```

A2.1. POST /tasks/route (~3h)
- Implement in `src/routes/tasks.rs`
- Extract/generate traceId
- Validate Idempotency-Key header
- Call Privacy Guard mask_json on task/context
- Emit audit event
- Return 202 Accepted with taskId

A2.2. GET /sessions (~1h)
- Implement in `src/routes/sessions.rs`
- For now, return empty array (Phase 4 will add DB)
- Return 200 OK

**Day 2: More Routes + Middleware (~8h)**

A2.3. POST /sessions (~1h)
- Generate session_id
- Return 201 Created

A2.4. POST /approvals (~2h)
- Accept task_id, decision, comments
- Emit audit event
- Return 202 Accepted with approvalId

A2.5. GET /profiles/{role} (~1h)
- Return mock profile (Phase 4 will query Directory Service)
- Return 200 OK

A3. Idempotency + Request Limits Middleware (~4h)
- Create `src/middleware/idempotency.rs`
- Validate Idempotency-Key header (must be valid UUID)
- Add RequestBodyLimitLayer (1MB limit)
- Test error responses (400 Bad Request, 413 Payload Too Large)

**Day 3: Privacy Guard + Tests (~8h)**

A4. Privacy Guard Integration (~3h)
- Implement `GuardClient::mask_json` utility
- Recursively mask strings in JSON objects/arrays
- Integrate in POST /tasks/route
- Log Privacy Guard latency in audit metadata

A5. Unit Tests (~4h)
- Create `src/routes/tasks_test.rs`
- Test POST /tasks/route (valid request, missing idempotency, invalid UUID, >1MB, no JWT)
- Test GET /sessions, POST /approvals, GET /profiles/{role}
- Run `cargo test` - all tests must pass

**Milestone M1 (Day 3):** Controller API functional, unit tests pass

---

### Workstream B: Agent Mesh MCP (Python) - Days 4-8

Execute in order:

**Day 4: Scaffold + send_task (~8h)**

B1. MCP Server Scaffold (~4h)
```bash
mkdir -p /home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh/tools
cd /home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh

# Create pyproject.toml
cat > pyproject.toml <<'EOF'
[project]
name = "agent-mesh"
version = "0.1.0"
description = "MCP server for multi-agent orchestration"
requires-python = ">=3.13"
dependencies = [
    "mcp>=1.0.0",
    "requests>=2.31.0",
    "pydantic>=2.0.0",
]

[project.scripts]
agent-mesh = "agent_mesh_server:main"
EOF

# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate
pip install -e .

# Create agent_mesh_server.py (MCP server entry point)
# Register 4 tools, run stdio_server transport
# Test: python agent_mesh_server.py (should start without errors)
```

B2. send_task Tool (~6h)
- Create `tools/send_task.py`
- Implement SendTaskParams (Pydantic model)
- Implement retry logic (3x exponential backoff + jitter)
- Generate UUID for Idempotency-Key
- Call Controller POST /tasks/route
- Test with curl to Controller API

**Day 5: request_approval + notify (~7h)**

B3. request_approval Tool (~4h)
- Create `tools/request_approval.py`
- Implement RequestApprovalParams
- Call Controller POST /approvals
- Return approvalId

B4. notify Tool (~3h)
- Create `tools/notify.py`
- Implement NotifyParams
- Use POST /tasks/route with task.type="notification"
- Return ack

**Day 6: fetch_status + config (~5h)**

B5. fetch_status Tool (~3h)
- Create `tools/fetch_status.py`
- Call Controller GET /sessions/{task_id}
- Return status, progress, result

B6. Configuration & Environment (~2h)
- Create `.env.example` (CONTROLLER_URL, MESH_JWT_TOKEN, retry settings)
- Create `README.md` with setup instructions
- Document Goose profiles.yaml integration

**Milestone M2 (Day 6):** All 4 MCP tools implemented

**Day 7: Integration Tests (~6h)**

B7. Integration Testing
- Create `tests/test_integration.py`
- Test 1: MCP server starts, tools list shows 4 tools
- Test 2: send_task calls Controller (202 Accepted)
- Test 3: request_approval calls Controller (202 Accepted)
- Test 4: notify calls Controller
- Test 5: fetch_status calls Controller (200 OK)
- Run `pytest` - all tests must pass

**Day 8: Deployment + ADR-0024 (~4h)**

B8. Deployment & Docs
- Test with actual Goose instance
- Add to `~/.config/goose/profiles.yaml`:
```yaml
extensions:
  agent_mesh:
    type: mcp
    command: ["python", "-m", "agent_mesh_server"]
    working_dir: "/home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh"
    env:
      CONTROLLER_URL: "http://localhost:8088"
      MESH_JWT_TOKEN: "eyJ..."  # From Keycloak
```
- Verify tools visible: `goose tools list | grep agent_mesh`
- Update VERSION_PINS.md with Agent Mesh version

**CRITICAL: Create ADR-0024** (see template below)

**Milestone M3 (Day 8):** Agent Mesh integration tests pass

---

### Workstream C: Cross-Agent Approval Demo - Day 9

Execute in order:

**C1. Demo Scenario Design (~2h)**
- Create `docs/demos/cross-agent-approval.md`
- Document Finance → Manager approval flow
- Define setup (2 Goose instances + Controller API)

**C2. Implementation (~4h)**

Terminal setup:
```bash
# Terminal 1: Controller API
cd src/controller
cargo run --release

# Terminal 2: Finance Agent (Goose instance 1)
goose session start --profile finance-agent

# Terminal 3: Manager Agent (Goose instance 2)
goose session start --profile manager-agent
```

Finance Agent steps:
```
Use agent_mesh__send_task:
- target: "manager"
- task: {"type": "budget_approval", "amount": 50000, "purpose": "Q1 new hires"}
- context: {"department": "Engineering", "quarter": "Q1-2026"}

Expected: Task routed successfully. Task ID: task-abc123...
```

Manager Agent approval (via curl):
```bash
TOKEN=$(curl -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "client_id=goose-controller" \
  -d "grant_type=client_credentials" \
  -d "client_secret=<secret>" | jq -r '.access_token')

curl -X POST http://localhost:8088/approvals \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{
    "task_id": "task-abc123...",
    "decision": "approved",
    "comments": "Approved for Q1 hiring plan"
  }'
```

Finance Agent check status:
```
Use agent_mesh__fetch_status:
- task_id: "task-abc123..."

Expected: Task Status: approved, Progress: 100%
```

**C3. Smoke Test Procedure + ADR-0025 (~2h)**

Create `docs/tests/smoke-phase3.md`:

Test 1: Controller API Health
- GET /status returns 200 OK
- Swagger UI accessible at /swagger-ui

Test 2: Agent Mesh Loading
- Extension loads in Goose
- All 4 tools visible: `goose tools list | grep agent_mesh`

Test 3: Cross-Agent Communication
- send_task from Finance → Manager succeeds (202 Accepted)
- fetch_status returns task details (200 OK)
- Approval workflow completes

Test 4: Audit Trail
- Audit events emitted for task routing
- Audit events emitted for approvals
- traceId consistent across events

Test 5: Backward Compatibility
- Phase 1.2 JWT auth still works (GET /status, POST /audit/ingest)
- Phase 2.2 Privacy Guard still functional (POST /guard/mask)

**CRITICAL: Create ADR-0025** (see template below)

**Milestone M4 (Day 9):** Cross-agent demo works, smoke tests pass (5/5), ADRs created

---

## 📝 ADR Templates (MANDATORY)

### ADR-0024: Agent Mesh Python Implementation

**Create:** `docs/adr/0024-agent-mesh-python-implementation.md`

```markdown
# ADR-0024: Agent Mesh Python Implementation

**Date:** [Completion date]  
**Status:** Accepted  
**Context:** Phase 3 (Controller API + Agent Mesh)  
**Deciders:** Engineering Team

## Context

Phase 3 requires an MCP extension for Goose that enables multi-agent orchestration via the Controller API. The extension must implement 4 tools: send_task, request_approval, notify, fetch_status.

### Language Choice Decision

Two options considered:
1. **Rust (rmcp SDK):** Aligns with Goose's native language, compile-time safety
2. **Python (mcp SDK):** Faster prototyping, simpler HTTP client, easier iteration

### MCP Protocol Details

MCP (Model Context Protocol) is language-agnostic:
- JSON-RPC over stdio/SSE/HTTP transport
- Goose v1.12 supports both Rust and Python MCP servers
- No integration concerns (protocol is the contract, not the language)

## Decision

We will implement the Agent Mesh MCP server in **Python** using the `mcp` SDK (not Rust with `rmcp`).

## Rationale

### Why Python for Phase 3 MVP?

1. **Faster Prototyping:** Python's dynamic typing and simpler syntax accelerate development
2. **Simpler HTTP Client:** `requests` library is more straightforward than Rust's `reqwest` (no async complexity)
3. **Easier Iteration:** No compilation step, faster feedback loop during development
4. **Lower Barrier:** Team can iterate on tools without Rust expertise

### Why NOT Rust for Phase 3 MVP?

1. **Complexity:** Async Rust + error handling adds 2-3 days to timeline
2. **Premature Optimization:** I/O-bound HTTP calls (not CPU-bound) - performance difference negligible (15ms vs 55ms for HTTP call)
3. **Integration:** MCP protocol is language-agnostic - Goose doesn't care about implementation language

### Migration Path to Rust (Post-Phase 3)

If needed later:
- Rewrite each tool in Rust using `rmcp` SDK
- Use same JSON-RPC contract (no protocol changes)
- Estimated effort: 2-3 days (tools are simple HTTP wrappers)
- Can do incrementally (migrate one tool at a time)

## Consequences

### Positive

- ✅ Phase 3 delivered 2-3 days faster (8-9 days vs 11-12 days with Rust)
- ✅ Team can iterate on tools quickly
- ✅ No Rust async learning curve for MCP extension development
- ✅ Same MCP protocol contract (no lock-in to Python)

### Negative

- ❌ Runtime dependency on Python 3.13 (not compiled binary)
- ❌ Slightly slower startup (Python interpreter load time ~200ms)
- ❌ Potential migration effort if Rust becomes requirement (2-3 days)

### Neutral

- ⚪ Performance: HTTP I/O-bound calls dominate (15ms Rust vs 55ms Python for HTTP call is negligible vs 2-5s Controller processing)

## Mitigations

### Performance Concerns

- Monitor P50 latency for `agent_mesh__send_task` (target: < 5s)
- If performance becomes issue, migrate to Rust incrementally

### Python Dependency Management

- Use Docker image `python:3.13-slim` for deployment (Phase 2.5 validated)
- Pin dependencies in `pyproject.toml` (mcp~=1.0.0, requests~=2.31.0)

### Migration Preparation

- Keep tool logic simple (thin HTTP wrappers)
- Avoid Python-specific features (makes Rust migration easier)
- Document API contract (JSON-RPC method names, parameters)

## Alternatives Considered

### Alternative 1: Rust + rmcp SDK
- ✅ **Pro:** Native language alignment with Goose
- ❌ **Con:** +2-3 days development time (async complexity)
- ❌ **Rejected:** Premature optimization, HTTP I/O-bound workload

### Alternative 2: TypeScript + MCP SDK
- ✅ **Pro:** Modern language, good ecosystem
- ❌ **Con:** Another runtime dependency (Node.js)
- ❌ **Rejected:** Python simpler for HTTP client, better team familiarity

## Implementation

### Phase 3 (Current)
- Python 3.13 with `mcp`, `requests`, `pydantic`
- 4 tools: send_task, request_approval, notify, fetch_status
- Retry logic: 3x exponential backoff + jitter
- Environment variables: CONTROLLER_URL, MESH_JWT_TOKEN

### Post-Phase 3 (If Migration Needed)
- Rewrite in Rust using `rmcp` SDK
- Estimated 2-3 days (one tool per day)
- Same MCP protocol contract (no changes to Goose integration)

## References

- **MCP Protocol:** https://modelcontextprotocol.io/
- **mcp Python SDK:** https://pypi.org/project/mcp/
- **rmcp Rust SDK:** https://docs.rs/rmcp/
- **Goose MCP Reference:** goose-versions-references/gooseV1.12.00/crates/goose-mcp/src/developer/rmcp_developer.rs
- **Phase 3 Pre-Flight Analysis:** Technical Project Plan/PM Phases/Phase-3-PRE-FLIGHT-ANALYSIS.md (Section 2.3)

---

**Approved by:** Engineering Team  
**Implementation:** Phase 3 (Workstream B, Days 4-8)
```

---

### ADR-0025: Controller API v1 Design

**Create:** `docs/adr/0025-controller-api-v1-design.md`

```markdown
# ADR-0025: Controller API v1 Design

**Date:** [Completion date]  
**Status:** Accepted  
**Context:** Phase 3 (Controller API + Agent Mesh)  
**Deciders:** Engineering Team, Product Team

## Context

Phase 3 requires a Controller API to enable multi-agent orchestration. The API serves as the central routing hub for:
- Task routing between agents (Finance → Manager)
- Session management (track task state)
- Approval workflows (request/submit approvals)
- Agent discovery (query profiles by role)

### Design Constraints

1. **Timeline:** Must deliver in Days 1-3 (limited scope for MVP)
2. **Dependencies:** No database yet (Phase 4 adds Postgres persistence)
3. **Integration:** Must work with existing JWT auth (Phase 1.2) and Privacy Guard (Phase 2.2)
4. **Validation:** Must support Agent Mesh MCP testing (Phase 3 Workstream B depends on API)

## Decision

We will implement a **minimal OpenAPI with 5 routes**, deferring persistence to Phase 4.

### API Design

**Routes:**
1. POST /tasks/route - Route task to target agent (202 Accepted)
2. GET /sessions - List active sessions (200 OK, empty array for Phase 3)
3. POST /sessions - Create new session (201 Created, ephemeral for Phase 3)
4. POST /approvals - Submit approval decision (202 Accepted, ephemeral for Phase 3)
5. GET /profiles/{role} - Get agent profile (200 OK, mock data for Phase 3)

**Middleware:**
- JWT verification (existing from Phase 1.2)
- Privacy Guard integration (existing from Phase 2.2)
- Idempotency key validation (new - Idempotency-Key header, UUID format)
- Request size limit: 1MB (new - 413 Payload Too Large)
- traceId extraction/generation (new - X-Trace-Id header)

**OpenAPI Generation:**
- Use `utoipa` 4.0 crate (derives OpenAPI from Rust structs)
- Mount Swagger UI at /swagger-ui
- Publish spec at /api-docs/openapi.json

## Rationale

### Why Minimal (5 Routes)?

1. **Unblock Agent Mesh Development:** Workstream B (Agent Mesh) depends on these routes
2. **Validate API Shape:** Test API contract with real MCP tools before committing to full design
3. **Defer Complexity:** Persistence (database), advanced routing, policy enforcement → Phase 4

### Why Stateless for Phase 3?

1. **No Database Yet:** Postgres is infrastructure (Phase 0), but schema design is Phase 4
2. **MVP Focus:** Prove multi-agent communication works (send_task, fetch_status) before adding state
3. **Simplicity:** Ephemeral task storage (in-memory HashMap) sufficient for cross-agent demo

### Why utoipa for OpenAPI?

1. **Type Safety:** OpenAPI spec derived from Rust structs (compile-time validation)
2. **Goose v1.12 Pattern:** Goose server uses `utoipa` (proven approach)
3. **Swagger UI:** Built-in UI for API exploration (helpful for Agent Mesh integration)

## Consequences

### Positive

- ✅ Phase 3 delivered on time (8-9 days)
- ✅ Agent Mesh development unblocked (API available by Day 3)
- ✅ API shape validated before Phase 4 (easy to iterate)
- ✅ Swagger UI aids debugging (visualize request/response schemas)

### Negative

- ❌ No persistence in Phase 3 (task/session state lost on restart)
- ❌ Limited functionality (GET /sessions returns empty array)
- ❌ Mock data for GET /profiles/{role} (real Directory Service in Phase 4)

### Mitigations

#### Ephemeral State

- Use in-memory HashMap for task storage (sufficient for cross-agent demo)
- Document limitation in README.md
- Phase 4 will add Postgres persistence (migration straightforward)

#### Mock Profile Data

- GET /profiles/{role} returns static mock (e.g., {"role": "manager", "capabilities": ["task_routing"]})
- Phase 4 will query real Directory Service
- API contract stays same (only implementation changes)

## Alternatives Considered

### Alternative 1: Full API with Persistence (8 routes + database)
- ✅ **Pro:** Production-ready from Phase 3
- ❌ **Con:** +3-4 days for database schema design/migration
- ❌ **Rejected:** Delays Agent Mesh development, over-engineering for MVP

### Alternative 2: gRPC Instead of HTTP/REST
- ✅ **Pro:** Type-safe, faster serialization
- ❌ **Con:** More complex (proto files, code generation), harder to debug
- ❌ **Rejected:** HTTP/REST simpler for Agent Mesh HTTP client

### Alternative 3: GraphQL for Flexible Querying
- ✅ **Pro:** Flexible queries, single endpoint
- ❌ **Con:** Overkill for 5 simple routes, steep learning curve
- ❌ **Rejected:** REST sufficient for Phase 3 scope

## Implementation

### Phase 3 (Current - Minimal API)

**Routes:**
- POST /tasks/route → Store in memory, emit audit event, return taskId
- GET /sessions → Return empty Vec (no persistence)
- POST /sessions → Generate session_id, return 201 Created (ephemeral)
- POST /approvals → Emit audit event, return approvalId (ephemeral)
- GET /profiles/{role} → Return mock ProfileResponse

**Middleware:**
- Idempotency key: Validate UUID format, return 400 if missing/invalid
- Request limit: 1MB via RequestBodyLimitLayer
- traceId: Extract from X-Trace-Id header, generate UUID if missing

**OpenAPI:**
- Swagger UI at http://localhost:8088/swagger-ui
- Spec at http://localhost:8088/api-docs/openapi.json

### Phase 4 (Persistence + Full Functionality)

**Database Schema:**
- tasks table: task_id (PK), target, payload, status, created_at
- sessions table: session_id (PK), task_id (FK), state, updated_at
- approvals table: approval_id (PK), task_id (FK), decision, approver_role

**New Routes:**
- GET /tasks/{id} - Get task by ID
- PATCH /tasks/{id} - Update task status
- GET /approvals/{id} - Get approval details

**Directory Service Integration:**
- GET /profiles/{role} queries real Directory Service (LDAP/AD)
- Returns capabilities, contact info, org hierarchy

## References

- **Axum Framework:** https://docs.rs/axum/latest/axum/
- **utoipa OpenAPI:** https://docs.rs/utoipa/latest/utoipa/
- **Goose Server Reference:** goose-versions-references/gooseV1.12.00/crates/goose-server/src/lib.rs
- **Controller Stub:** src/controller/src/main.rs
- **OpenAPI Stub:** docs/api/controller/openapi.yaml
- **Phase 3 Pre-Flight Analysis:** Technical Project Plan/PM Phases/Phase-3-PRE-FLIGHT-ANALYSIS.md

---

**Approved by:** Engineering Team, Product Team  
**Implementation:** Phase 3 (Workstream A, Days 1-3)
```

---

## 📊 Timeline Summary

**Total Effort:** ~8-9 days (2 weeks)

```
Day 1:   Workstream A - OpenAPI + POST /tasks/route + GET /sessions
Day 2:   Workstream A - POST /sessions + POST /approvals + GET /profiles + Middleware
Day 3:   Workstream A - Privacy Guard integration + Unit tests ← Milestone M1

Day 4:   Workstream B - MCP scaffold + send_task tool
Day 5:   Workstream B - request_approval + notify tools
Day 6:   Workstream B - fetch_status + configuration ← Milestone M2
Day 7:   Workstream B - Integration tests ← Milestone M3
Day 8:   Workstream B - Deployment + docs + ADR-0024

Day 9:   Workstream C - Demo scenario + implementation + smoke tests + ADR-0025 ← Milestone M4
```

---

## 🚦 Progress Tracking (MANDATORY)

### Update After EVERY Task Completion

**Files to Update:**
1. **Phase-3-Agent-State.json** - Increment task counts, update component status
2. **Phase-3-Checklist.md** - Mark task complete with `[x]`
3. **docs/tests/phase3-progress.md** - Append brief task entry (optional per task, mandatory per workstream)

**State File Update Commands:**
```bash
cd "Technical Project Plan/PM Phases/Phase-3"

# Example: After completing A1 (OpenAPI schema)
jq '.workstreams.A.tasks_completed += 1 | 
    .progress.completed_tasks += 1 | 
    .progress.percentage = ((.progress.completed_tasks / .progress.total_tasks) * 100 | round) |
    .components.controller_api.openapi_spec = true' \
  Phase-3-Agent-State.json > tmp.json && mv tmp.json Phase-3-Agent-State.json
```

**Manual Alternative (if jq fails):**
1. Open `Phase-3-Agent-State.json` in text editor
2. Update `workstreams.A.tasks_completed` (increment by 1)
3. Update `progress.completed_tasks` (increment by 1)
4. Recalculate `progress.percentage` = (completed_tasks / total_tasks * 100)
5. Update relevant `components` fields
6. Save file

**Checklist Update:**
```bash
# Open Phase-3-Checklist.md in editor
# Change: - [ ] A1. OpenAPI Schema Design
# To:     - [x] A1. OpenAPI Schema Design
# Save file
```

### Progress Log Structure

**File:** `docs/tests/phase3-progress.md`

**Create at start of Phase 3 if not exists:**
```markdown
# Phase 3 Progress Log — Controller API + Agent Mesh

**Phase:** 3  
**Status:** IN_PROGRESS  
**Start Date:** [YYYY-MM-DD]  
**End Date:** [TBD]  
**Branch:** feature/phase-3-controller-agent-mesh

---

## Timeline

[Entries added chronologically below]

---

## Issues Encountered & Resolutions

[Issues added as encountered]

---

## Git History

[Commits logged here]

---

## Deliverables Tracking

[Files created/modified logged here]

---

**End of Progress Log**
```

**After completing each task (optional, brief):**
```markdown
**HH:MM - HH:MM** - Task A1: OpenAPI Schema Design
- Added utoipa 4.0, uuid 1.6 to Cargo.toml
- Created src/controller/src/api/openapi.rs
- Mounted Swagger UI at /swagger-ui
- **Files:** src/controller/Cargo.toml, src/api/openapi.rs, src/main.rs
```

### Update After EVERY Milestone Achievement

**When completing M1, M2, M3, or M4:**
```bash
cd "Technical Project Plan/PM Phases/Phase-3"

# Example: After completing M1 (Controller API functional)
jq '.milestones.M1.achieved = true | 
    .milestones.M1.date = now' \
  Phase-3-Agent-State.json > tmp.json && mv tmp.json Phase-3-Agent-State.json
```

---

## ⚠️ Important Notes

### Pre-Execution Check

**CRITICAL:** Before starting Phase 3, check if Phase 2.5 dependency upgrades changed anything:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
git log --oneline -10 | grep "phase-2.5"
git diff <phase-2.5-commit> HEAD -- src/controller/Cargo.toml
# Review any changes to dependencies, config files
```

### Rollback Strategy

If ANY acceptance criteria fails:
```bash
git checkout main
git branch -D feature/phase-3-controller-agent-mesh
docker compose -f deploy/compose/ce.dev.yml restart
```

### Critical Files to Create/Update

1. **Controller API:**
   - src/controller/Cargo.toml (add utoipa, uuid, tower-http)
   - src/controller/src/api/openapi.rs (new)
   - src/controller/src/routes/tasks.rs (new)
   - src/controller/src/routes/sessions.rs (new)
   - src/controller/src/middleware/idempotency.rs (new)

2. **Agent Mesh:**
   - src/agent-mesh/pyproject.toml (new)
   - src/agent-mesh/agent_mesh_server.py (new)
   - src/agent-mesh/tools/*.py (4 new files)
   - src/agent-mesh/README.md (new)

3. **Documentation:**
   - docs/adr/0024-agent-mesh-python-implementation.md ← MANDATORY
   - docs/adr/0025-controller-api-v1-design.md ← MANDATORY
   - docs/demos/cross-agent-approval.md (new)
   - docs/tests/smoke-phase3.md (new)
   - VERSION_PINS.md (update with Agent Mesh version)

4. **Goose Integration:**
   - ~/.config/goose/profiles.yaml (add agent_mesh extension)

---

## 📝 Git Workflow

### Branch Strategy
```bash
git checkout main
git pull origin main
git checkout -b feature/phase-3-controller-agent-mesh
```

### Commit Messages (Conventional Commits)

**Workstream A (Controller API):**
```bash
git add src/controller/
git commit -m "feat(controller): add OpenAPI schema with utoipa

- Add utoipa, uuid dependencies to Cargo.toml
- Create src/api/openapi.rs with request/response schemas
- Mount Swagger UI at /swagger-ui
- OpenAPI spec at /api-docs/openapi.json

Part of Phase 3 Workstream A (Day 1)"

git add src/controller/src/routes/
git commit -m "feat(controller): implement task routing and session routes

- POST /tasks/route: Route task to target agent (202 Accepted)
- GET /sessions: List sessions (200 OK, empty for Phase 3)
- POST /sessions: Create session (201 Created)
- POST /approvals: Submit approval (202 Accepted)
- GET /profiles/{role}: Get profile (200 OK, mock)

Includes idempotency validation and Privacy Guard integration.

Part of Phase 3 Workstream A (Days 1-3)"
```

**Workstream B (Agent Mesh):**
```bash
git add src/agent-mesh/
git commit -m "feat(agent-mesh): implement MCP server with 4 tools

- send_task: Route task via Controller API (retry logic)
- request_approval: Request approval from role
- notify: Send notification to target
- fetch_status: Get task status

Uses Python 3.13 + mcp SDK + requests.

Part of Phase 3 Workstream B (Days 4-8)"
```

**Workstream C (Demo + ADRs):**
```bash
git add docs/demos/cross-agent-approval.md docs/tests/smoke-phase3.md
git commit -m "docs(demo): add cross-agent approval demo and smoke tests

- Finance → Manager approval flow
- 5 smoke tests (API health, MCP loading, cross-agent, audit, compat)

Part of Phase 3 Workstream C (Day 9)"

git add docs/adr/0024-agent-mesh-python-implementation.md
git commit -m "docs(adr): add ADR-0024 Agent Mesh Python implementation

Documents decision to use Python (not Rust) for Phase 3 MVP.
Migration path to Rust documented (2-3 days effort if needed).

Part of Phase 3 (Workstream B8)"

git add docs/adr/0025-controller-api-v1-design.md
git commit -m "docs(adr): add ADR-0025 Controller API v1 design

Documents minimal 5-route API for Phase 3.
Defers persistence to Phase 4.

Part of Phase 3 (Workstream C3)"
```

### Merge to Main
```bash
# After all acceptance criteria pass
git checkout main
git merge --squash feature/phase-3-controller-agent-mesh
git commit -m "feat(phase-3): controller API + agent mesh [COMPLETE]

Summary:
- Controller API: 5 routes (tasks, sessions, approvals, profiles)
- OpenAPI spec with Swagger UI
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status)
- Cross-agent demo: Finance → Manager approval workflow
- Integration tests: 100% pass
- Smoke tests: 5/5 pass
- ADR-0024: Agent Mesh Python implementation
- ADR-0025: Controller API v1 design

Phase 3 (M2 milestone) complete. Unblocks Phase 4 (Directory + Policy)."

git push origin main
```

---

## 🚨 MANDATORY CHECKPOINTS

### ⚠️ Checkpoint 1: After Workstream A (Day 3 - Milestone M1)

**🛑 STOP HERE. Do not proceed to Workstream B until user confirms.**

**Before proceeding, complete ALL steps below:**

#### Step 1: Update State Files (~5 min)

```bash
cd "Technical Project Plan/PM Phases/Phase-3"

# Update state JSON - Workstream A complete
jq '.workstreams.A.status = "COMPLETE" |
    .workstreams.A.checkpoint_complete = true |
    .current_workstream = "B" |
    .milestones.M1.achieved = true |
    .milestones.M1.date = now |
    .pending_user_confirmation = true |
    .checkpoint_reason = "Workstream A complete - awaiting confirmation to proceed to B"' \
  Phase-3-Agent-State.json > tmp.json && mv tmp.json Phase-3-Agent-State.json
```

**Manual alternative:** Edit Phase-3-Agent-State.json and set:
- `workstreams.A.status = "COMPLETE"`
- `workstreams.A.checkpoint_complete = true`
- `current_workstream = "B"`
- `milestones.M1.achieved = true` and `milestones.M1.date = [current timestamp]`
- `pending_user_confirmation = true`

#### Step 2: Update Checklist (~2 min)

Open `Phase-3-Checklist.md` and ensure all Workstream A tasks marked `[x]`:
- [x] A1, A2.1-A2.5, A3, A4, A5, A6 all complete
- Update progress: 6/31 tasks = 19%

#### Step 3: Update Progress Log (~10 min)

Append to `docs/tests/phase3-progress.md`:

```markdown
### [YYYY-MM-DD] - Workstream A: Controller API (COMPLETE)

**Duration:** Day 1-3  
**Status:** ✅ COMPLETE  

#### Tasks Completed:
- [x] A1: OpenAPI Schema Design (~4h)
- [x] A2.1-A2.5: Route Implementations (~1 day)
- [x] A3: Idempotency + Request Limits Middleware (~4h)
- [x] A4: Privacy Guard Integration (~3h)
- [x] A5: Unit Tests (~4h)
- [x] A6: Progress Tracking (~15 min)

#### Deliverables:
- ✅ src/controller/Cargo.toml (added utoipa, uuid, tower-http)
- ✅ src/controller/src/api/openapi.rs (OpenAPI schema)
- ✅ src/controller/src/routes/tasks.rs (POST /tasks/route)
- ✅ src/controller/src/routes/sessions.rs (GET /sessions, POST /sessions)
- ✅ src/controller/src/routes/approvals.rs (POST /approvals)
- ✅ src/controller/src/routes/profiles.rs (GET /profiles/{role})
- ✅ src/controller/src/middleware/idempotency.rs (validation)
- ✅ src/controller/src/routes/tasks_test.rs (unit tests)

#### Issues Encountered:
[List any issues encountered during Workstream A, or write "None"]

#### Performance Metrics:
- OpenAPI spec size: [X KB]
- Unit tests: ALL PASS (cargo test)
- Build time: [X seconds]

#### Git Commits:
- [sha]: feat(controller): add OpenAPI schema with utoipa
- [sha]: feat(controller): implement task routing and session routes
- [sha]: feat(controller): add idempotency middleware
- [sha]: feat(controller): integrate Privacy Guard mask_json
- [sha]: test(controller): add unit tests for routes

**Milestone M1 Achieved:** ✅ Controller API functional, unit tests pass

**Next:** Workstream B (Agent Mesh MCP)

---
```

#### Step 4: Commit Progress (~3 min)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

git add "Technical Project Plan/PM Phases/Phase-3/" docs/tests/phase3-progress.md
git commit -m "docs(phase-3): workstream A complete - controller API functional

Milestone M1 achieved:
- All 5 routes implemented (tasks, sessions, approvals, profiles)
- OpenAPI spec published at /api-docs/openapi.json
- Swagger UI accessible at /swagger-ui
- Idempotency middleware functional
- Privacy Guard integration working
- Unit tests: ALL PASS (cargo test)

Progress:
- 6/31 tasks complete (19%)
- State file updated
- Progress log updated
- Checklist updated

Awaiting user confirmation to proceed to Workstream B.

Refs: #phase3 #milestone-m1 #checkpoint"
```

#### Step 5: Report to User & WAIT (~2 min)

**Copy this message to user:**

```
🎉 Workstream A (Controller API) is COMPLETE ✅

**Summary:**
- ✅ 5 routes implemented: POST /tasks/route, GET/POST /sessions, POST /approvals, GET /profiles/{role}
- ✅ OpenAPI spec published at /api-docs/openapi.json
- ✅ Swagger UI accessible at /swagger-ui
- ✅ Idempotency middleware functional (UUID validation)
- ✅ Request size limit enforced (1MB)
- ✅ Privacy Guard integration working (mask_json utility)
- ✅ Unit tests: ALL PASS (cargo test)

**Files Updated:**
- Phase-3-Agent-State.json (workstream A status = COMPLETE)
- Phase-3-Checklist.md (6/31 tasks = 19% complete)
- docs/tests/phase3-progress.md (Workstream A summary appended)

**Git:**
- Commit: [sha] docs(phase-3): workstream A complete

**Milestone M1 Achieved:** ✅ Controller API functional

**Next: Workstream B (Agent Mesh MCP) - Days 4-8**
- B1: MCP Server Scaffold
- B2: send_task Tool
- B3: request_approval Tool
- B4: notify Tool
- B5: fetch_status Tool
- B6: Configuration
- B7: Integration Tests
- B8: Deployment + ADR-0024

---

**⏸️ WAITING FOR YOUR CONFIRMATION**

Type **"proceed"** to continue to Workstream B  
Type **"review"** to inspect files first  
Type **"pause"** to stop and save progress
```

**🛑 DO NOT PROCEED until user responds with "proceed", "review", or other instruction.**

---

### ⚠️ Checkpoint 2: After Workstream B (Day 8 - Milestone M3)

**🛑 STOP HERE. Do not proceed to Workstream C until user confirms.**

**Before proceeding, complete ALL steps below:**

#### Step 1: Update State Files (~5 min)

```bash
cd "Technical Project Plan/PM Phases/Phase-3"

# Update state JSON - Workstream B complete
jq '.workstreams.B.status = "COMPLETE" |
    .workstreams.B.checkpoint_complete = true |
    .current_workstream = "C" |
    .milestones.M2.achieved = true |
    .milestones.M2.date = now |
    .milestones.M3.achieved = true |
    .milestones.M3.date = now |
    .components.agent_mesh.tools_implemented = 4 |
    .components.agent_mesh.integration_tests_pass = true |
    .components.agent_mesh.goose_loadable = true |
    .adrs_to_create[0].created = true |
    .pending_user_confirmation = true |
    .checkpoint_reason = "Workstream B complete - awaiting confirmation to proceed to C"' \
  Phase-3-Agent-State.json > tmp.json && mv tmp.json Phase-3-Agent-State.json
```

#### Step 2: Update Checklist (~2 min)

Open `Phase-3-Checklist.md` and ensure all Workstream B tasks marked `[x]`:
- [x] B1-B9 all complete (including ADR-0024)
- Update progress: 15/31 tasks = 48%

#### Step 3: Update Progress Log (~15 min)

Append to `docs/tests/phase3-progress.md`:

```markdown
### [YYYY-MM-DD] - Workstream B: Agent Mesh MCP (COMPLETE)

**Duration:** Day 4-8  
**Status:** ✅ COMPLETE  

#### Tasks Completed:
- [x] B1: MCP Server Scaffold (~4h)
- [x] B2: send_task Tool (~6h)
- [x] B3: request_approval Tool (~4h)
- [x] B4: notify Tool (~3h)
- [x] B5: fetch_status Tool (~3h)
- [x] B6: Configuration & Environment (~2h)
- [x] B7: Integration Testing (~6h)
- [x] B8: Deployment & Docs (~4h)
- [x] B9: Progress Tracking (~15 min)

#### Deliverables:
- ✅ src/agent-mesh/pyproject.toml
- ✅ src/agent-mesh/agent_mesh_server.py (MCP entry point)
- ✅ src/agent-mesh/tools/send_task.py (retry logic with exponential backoff)
- ✅ src/agent-mesh/tools/request_approval.py
- ✅ src/agent-mesh/tools/notify.py
- ✅ src/agent-mesh/tools/fetch_status.py
- ✅ src/agent-mesh/.env.example (CONTROLLER_URL, MESH_JWT_TOKEN)
- ✅ src/agent-mesh/README.md (setup instructions)
- ✅ src/agent-mesh/tests/test_integration.py
- ✅ docs/adr/0024-agent-mesh-python-implementation.md ← CRITICAL ADR

#### Issues Encountered:
[List any issues encountered during Workstream B, or write "None"]

#### Test Results:
- Integration tests: [X/X PASS] (pytest)
- Tools visible in Goose: ✅ [send_task, request_approval, notify, fetch_status]
- MCP server startup: ✅ SUCCESS
- Extension loading: ✅ profiles.yaml configured

#### Git Commits:
- [sha]: feat(agent-mesh): implement MCP server with 4 tools
- [sha]: test(agent-mesh): add integration tests
- [sha]: docs(agent-mesh): add README and configuration
- [sha]: docs(adr): add ADR-0024 Agent Mesh Python implementation

**Milestone M2 Achieved:** ✅ All 4 MCP tools implemented  
**Milestone M3 Achieved:** ✅ Agent Mesh integration tests pass

**Next:** Workstream C (Cross-Agent Approval Demo)

---
```

#### Step 4: Commit Progress (~3 min)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

git add "Technical Project Plan/PM Phases/Phase-3/" docs/tests/phase3-progress.md docs/adr/0024-*.md
git commit -m "docs(phase-3): workstream B complete - agent mesh MCP functional

Milestone M2 & M3 achieved:
- All 4 tools implemented (send_task, request_approval, notify, fetch_status)
- MCP server starts successfully
- Extension loadable in Goose (profiles.yaml configured)
- Tools visible: goose tools list | grep agent_mesh ✅
- Integration tests: ALL PASS (pytest)
- ADR-0024 created: Agent Mesh Python Implementation ✅

Progress:
- 15/31 tasks complete (48%)
- State file updated
- Progress log updated
- Checklist updated

Awaiting user confirmation to proceed to Workstream C.

Refs: #phase3 #milestone-m2 #milestone-m3 #checkpoint"
```

#### Step 5: Report to User & WAIT (~2 min)

**Copy this message to user:**

```
🎉 Workstream B (Agent Mesh MCP) is COMPLETE ✅

**Summary:**
- ✅ 4 tools implemented: send_task, request_approval, notify, fetch_status
- ✅ MCP server starts successfully (stdio transport)
- ✅ Extension loads in Goose (profiles.yaml configured)
- ✅ Tools visible: `goose tools list | grep agent_mesh` shows all 4 ✅
- ✅ Integration tests: ALL PASS (pytest)
- ✅ Retry logic: 3x exponential backoff + jitter
- ✅ ADR-0024 created: Agent Mesh Python Implementation ✅

**Files Updated:**
- Phase-3-Agent-State.json (workstream B status = COMPLETE)
- Phase-3-Checklist.md (15/31 tasks = 48% complete)
- docs/tests/phase3-progress.md (Workstream B summary appended)

**Git:**
- Commit: [sha] docs(phase-3): workstream B complete

**Milestones Achieved:**
- ✅ M2: All 4 MCP tools implemented
- ✅ M3: Agent Mesh integration tests pass

**Next: Workstream C (Cross-Agent Approval Demo) - Day 9**
- C1: Demo Scenario Design
- C2: Implementation (Finance → Manager workflow)
- C3: Smoke Test Procedure
- C4: ADR-0025 Creation
- C5: Progress Tracking

---

**⏸️ WAITING FOR YOUR CONFIRMATION**

Type **"proceed"** to continue to Workstream C  
Type **"review"** to inspect files first  
Type **"pause"** to stop and save progress
```

**🛑 DO NOT PROCEED until user responds with "proceed", "review", or other instruction.**

---

### ⚠️ Checkpoint 3: After Workstream C (Day 9 - Milestone M4)

**🛑 STOP HERE. Phase 3 complete. Wait for user review before marking COMPLETE.**

**Before marking phase complete, complete ALL steps below:**

#### Step 1: Update State Files (~5 min)

```bash
cd "Technical Project Plan/PM Phases/Phase-3"

# Update state JSON - Phase 3 COMPLETE
jq '.workstreams.C.status = "COMPLETE" |
    .workstreams.C.checkpoint_complete = true |
    .status = "COMPLETE" |
    .end_date = (now | strftime("%Y-%m-%d")) |
    .current_workstream = null |
    .milestones.M4.achieved = true |
    .milestones.M4.date = now |
    .components.demo.scenario_defined = true |
    .components.demo.implementation_complete = true |
    .components.demo.smoke_tests_pass = true |
    .integration_results.controller_health = true |
    .integration_results.agent_mesh_tools_visible = true |
    .integration_results.cross_agent_communication = true |
    .integration_results.audit_trail = true |
    .integration_results.backward_compatibility_phase_1_2 = true |
    .integration_results.backward_compatibility_phase_2_2 = true |
    .adrs_to_create[1].created = true |
    .pending_user_confirmation = false |
    .progress_log_created = true' \
  Phase-3-Agent-State.json > tmp.json && mv tmp.json Phase-3-Agent-State.json
```

#### Step 2: Update Checklist (~2 min)

Open `Phase-3-Checklist.md` and ensure all tasks marked `[x]`:
- [x] All Workstream C tasks complete (C1-C5)
- Update progress: 31/31 tasks = 100% ✅

#### Step 3: Update Progress Log (FINAL ENTRY) (~15 min)

Append to `docs/tests/phase3-progress.md`:

```markdown
### [YYYY-MM-DD] - Workstream C: Cross-Agent Approval Demo (COMPLETE)

**Duration:** Day 9  
**Status:** ✅ COMPLETE  

#### Tasks Completed:
- [x] C1: Demo Scenario Design (~2h)
- [x] C2: Implementation (~4h)
- [x] C3: Smoke Test Procedure (~2h)
- [x] C4: ADR-0025 Creation
- [x] C5: Progress Tracking (~15 min)

#### Deliverables:
- ✅ docs/demos/cross-agent-approval.md (Finance → Manager workflow)
- ✅ docs/tests/smoke-phase3.md (5 smoke tests)
- ✅ docs/adr/0025-controller-api-v1-design.md ← CRITICAL ADR

#### Smoke Test Results:
- Test 1: Controller API Health ✅ PASS
- Test 2: Agent Mesh Loading ✅ PASS
- Test 3: Cross-Agent Communication ✅ PASS
- Test 4: Audit Trail ✅ PASS
- Test 5: Backward Compatibility ✅ PASS

**Overall:** 5/5 PASS ✅

#### Cross-Agent Demo Results:
- Finance → Manager approval flow: ✅ SUCCESS
- Task routed: task-[uuid]
- Approval submitted: approval-[uuid]
- Status retrieved: approved ✅
- traceId propagation: ✅ WORKING

#### Git Commits:
- [sha]: docs(demo): add cross-agent approval demo and smoke tests
- [sha]: docs(adr): add ADR-0025 Controller API v1 design
- [sha]: docs(phase-3): workstream C complete

**Milestone M4 Achieved:** ✅ Cross-agent demo works, smoke tests pass, ADRs created

---

## Phase 3 COMPLETION SUMMARY

**Status:** ✅ COMPLETE  
**Duration:** [X days]  
**Total Tasks:** 31/31 (100%)  
**Milestones:** 4/4 (100%)  

### All Deliverables Complete:
- ✅ Controller API (5 routes functional)
- ✅ OpenAPI spec (published at /api-docs/openapi.json, validated)
- ✅ Swagger UI accessible (http://localhost:8088/swagger-ui)
- ✅ Agent Mesh MCP (4 tools functional in Goose)
- ✅ Cross-agent approval demo working (Finance → Manager)
- ✅ docs/demos/cross-agent-approval.md
- ✅ docs/tests/smoke-phase3.md
- ✅ docs/tests/phase3-progress.md ← THIS FILE
- ✅ ADR-0024: Agent Mesh Python Implementation
- ✅ ADR-0025: Controller API v1 Design
- ✅ VERSION_PINS.md updated (Agent Mesh 0.1.0)
- ✅ CHANGELOG.md updated (Phase 3 entry)

### Test Results Summary:
- ✅ Unit tests (Controller): ALL PASS (cargo test)
- ✅ Integration tests (Agent Mesh): ALL PASS (pytest)
- ✅ Smoke tests: 5/5 PASS
- ✅ Backward compatibility: Phase 1.2 (JWT auth) ✅ + Phase 2.2 (Privacy Guard) ✅

### Performance Metrics:
- agent_mesh__send_task P50: [X ms] (target: <5s) ✅
- Controller API response time: [X ms]
- OpenAPI spec validation: PASS ✅

### Issues Resolved:
[Summary of major issues resolved across all workstreams, or write "No critical issues"]

### Lessons Learned:
[What went well, what could be improved for Phase 4]

---

**Phase 3 COMPLETE. Ready for Phase 4 (Directory Service + Policy Engine).** ✅
```

#### Step 4: Create Completion Summary (~30 min)

Create `Technical Project Plan/PM Phases/Phase-3/Phase-3-Completion-Summary.md` with:
- Executive summary
- Workstream-by-workstream achievements
- Technical decisions made
- ADRs created
- Git commit history
- Phase 4 readiness assessment

#### Step 5: Update CHANGELOG.md (~15 min)

[See CHECKPOINT-ADDITIONS.md for full CHANGELOG entry template]

#### Step 6: Final Git Commit & Merge (~10 min)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Final commit on feature branch
git add .
git commit -m "feat(phase-3): controller API + agent mesh [COMPLETE]

Summary:
- Controller API: 5 routes (tasks, sessions, approvals, profiles)
- OpenAPI spec with Swagger UI
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status)
- Cross-agent demo: Finance → Manager approval workflow
- Integration tests: 100% pass
- Smoke tests: 5/5 pass
- ADR-0024: Agent Mesh Python implementation
- ADR-0025: Controller API v1 design

All milestones achieved (M1, M2, M3, M4).
Phase 3 complete. Unblocks Phase 4 (Directory + Policy).

Refs: #phase3 #complete"

# Merge to main
git checkout main
git merge --squash feature/phase-3-controller-agent-mesh
git commit -m "feat(phase-3): controller API + agent mesh [COMPLETE]

Summary:
- Controller API: 5 routes (tasks, sessions, approvals, profiles)
- OpenAPI spec with Swagger UI (utoipa 4.0)
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status)
- Cross-agent demo: Finance → Manager approval workflow
- Unit tests: 100% pass (cargo test)
- Integration tests: 100% pass (pytest)
- Smoke tests: 5/5 pass
- ADR-0024: Agent Mesh Python implementation
- ADR-0025: Controller API v1 design

Phase 3 (M2 milestone) complete. Unblocks Phase 4 (Directory + Policy).

Refs: #phase3 #milestone-m1 #milestone-m2 #milestone-m3 #milestone-m4"

git push origin main
```

#### Step 7: Report to User & WAIT (~5 min)

**Copy this message to user:**

```
🎉🎉🎉 Phase 3 (Controller API + Agent Mesh) is COMPLETE ✅

**Summary:**
✅ **All 3 workstreams complete** (A, B, C)  
✅ **All 4 milestones achieved** (M1, M2, M3, M4)  
✅ **All 31 tasks complete** (100%)  
✅ **All deliverables created**

**Controller API:**
- ✅ 5 routes functional (tasks, sessions, approvals, profiles)
- ✅ OpenAPI spec published (/api-docs/openapi.json)
- ✅ Swagger UI accessible (/swagger-ui)
- ✅ Idempotency middleware + request size limits working
- ✅ Privacy Guard integration (mask_json utility)
- ✅ Unit tests: ALL PASS (cargo test)

**Agent Mesh MCP:**
- ✅ 4 tools functional (send_task, request_approval, notify, fetch_status)
- ✅ MCP server starts successfully
- ✅ Extension loads in Goose
- ✅ Integration tests: ALL PASS (pytest)
- ✅ Retry logic with exponential backoff

**Cross-Agent Demo:**
- ✅ Finance → Manager approval workflow: SUCCESS
- ✅ Smoke tests: 5/5 PASS
- ✅ Audit trail validated (traceId propagation working)

**Documentation:**
- ✅ ADR-0024 created (Agent Mesh Python)
- ✅ ADR-0025 created (Controller API v1)
- ✅ Progress log complete (docs/tests/phase3-progress.md)
- ✅ Completion summary created
- ✅ CHANGELOG.md updated
- ✅ VERSION_PINS.md updated

**Backward Compatibility:**
- ✅ Phase 1.2 (JWT auth) still works
- ✅ Phase 2.2 (Privacy Guard) still works

**Git Status:**
- ✅ All commits merged to main
- ✅ Pushed to GitHub
- ✅ Feature branch deleted

---

**Phase 3 COMPLETE. Ready for Phase 4 (Directory Service + Policy Engine).** ✅

**Would you like me to:**
1. Review Phase 3 completion summary
2. Begin Phase 4 preparation
3. Other action?
```

**🛑 DO NOT PROCEED TO PHASE 4 until user confirms.**

---

## ✅ Completion Checklist

Before marking Phase 3 complete:

### Workstream A: Controller API
- [ ] All 5 routes implemented (tasks, sessions, approvals, profiles)
- [ ] OpenAPI spec published (/api-docs/openapi.json)
- [ ] Swagger UI accessible (/swagger-ui)
- [ ] Idempotency middleware functional
- [ ] Request size limit enforced (1MB)
- [ ] Privacy Guard integration working
- [ ] Unit tests pass (cargo test)

### Workstream B: Agent Mesh
- [ ] All 4 tools implemented (send_task, request_approval, notify, fetch_status)
- [ ] MCP server starts successfully
- [ ] Extension loads in Goose (profiles.yaml configured)
- [ ] Tools visible in Goose (goose tools list | grep agent_mesh)
- [ ] Integration tests pass (pytest)
- [ ] **ADR-0024 created and committed**

### Workstream C: Cross-Agent Demo
- [ ] Demo scenario documented (docs/demos/cross-agent-approval.md)
- [ ] Finance → Manager workflow works end-to-end
- [ ] Smoke tests documented (docs/tests/smoke-phase3.md)
- [ ] All 5 smoke tests pass
- [ ] **ADR-0025 created and committed**

### Documentation
- [ ] VERSION_PINS.md updated (Agent Mesh version)
- [ ] CHANGELOG.md updated (Phase 3 features)
- [ ] docs/tests/phase3-progress.md created and complete
- [ ] Both ADRs created (0024, 0025)

### Git
- [ ] All commits follow conventional format
- [ ] Squash merge to main
- [ ] Feature branch deleted

### Final Validation
- [ ] Phase 1.2 still works (JWT auth)
- [ ] Phase 2.2 still works (Privacy Guard)
- [ ] Phase-3-Agent-State.json status="COMPLETE"
- [ ] Phase-3-Checklist.md 28/28 tasks complete
- [ ] Create Phase-3-Completion-Summary.md

---

## 📚 Reference Documents

### Execution Artifacts
- **Full Details:** `Technical Project Plan/PM Phases/Phase-3/Phase-3-Execution-Plan.md`
- **Checklist:** `Technical Project Plan/PM Phases/Phase-3/Phase-3-Checklist.md`
- **State Tracking:** `Technical Project Plan/PM Phases/Phase-3/Phase-3-Agent-State.json`

### Analysis & Design
- **Pre-Flight Analysis:** `Technical Project Plan/PM Phases/Phase-3-PRE-FLIGHT-ANALYSIS.md` (30 pages)
- **MCP Architecture:** `goose-versions-references/how-goose-works-docs/docs/goose-v1.12.00-technical-architecture-report.md`
- **Goose MCP Reference:** `goose-versions-references/gooseV1.12.00/crates/goose-mcp/src/developer/rmcp_developer.rs`

### API Specifications
- **Controller Stub:** `src/controller/src/main.rs`
- **OpenAPI Stub:** `docs/api/controller/openapi.yaml`
- **ADR-0007:** Agent Mesh MCP (`docs/adr/0007-agent-mesh-mcp.md`)
- **ADR-0010:** Controller OpenAPI (`docs/adr/0010-controller-openapi-and-http-interfaces.md`)

### External Documentation
- **MCP Protocol:** https://modelcontextprotocol.io/
- **Axum Framework:** https://docs.rs/axum/latest/axum/
- **utoipa OpenAPI:** https://docs.rs/utoipa/latest/utoipa/
- **Python mcp SDK:** https://pypi.org/project/mcp/
- **Rust rmcp SDK:** https://docs.rs/rmcp/

---

## 🎯 Success Criteria (Final Check)

At the end of Phase 3, confirm:

- ✅ **Controller API:** All 5 routes functional, OpenAPI spec validated
- ✅ **Agent Mesh:** All 4 tools functional in Goose, integration tests pass
- ✅ **Demo:** Finance → Manager approval works end-to-end
- ✅ **Testing:** Unit tests pass (cargo test), integration tests pass (pytest), smoke tests pass (5/5)
- ✅ **Audit:** Events emitted with traceId propagation
- ✅ **Compatibility:** Phase 1.2 + Phase 2.2 still functional
- ✅ **Documentation:** ADR-0024 + ADR-0025 created, VERSION_PINS updated
- ✅ **Performance:** agent_mesh__send_task P50 < 5s

**If all ✅, Phase 3 is COMPLETE.** Proceed to Phase 4 (Directory Service + Policy Engine).

---

**Orchestrated by:** Goose AI Agent  
**Date:** 2025-11-04  
**Execution Time:** ~8-9 days  
**Next Phase:** Phase 4 (after Phase 3 complete)
