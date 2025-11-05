# Phase 3 Testing Strategy

**Document Purpose:** Clarify the multi-layer testing approach for Agent Mesh MCP tools

---

## Testing Layers

### Layer 1: Validation Tests (B2-B5) ✅ CURRENT APPROACH

**Purpose:** Verify tool structure and MCP compatibility WITHOUT running Controller API

**What We Test:**
- ✅ Tool definition (name, description, input schema)
- ✅ Pydantic parameter models (required fields, optional fields, defaults)
- ✅ JSON schema generation (types, properties, required list)
- ✅ MCP registration (tool imports, server.add_tool() works)
- ✅ Docker build (Python 3.13-slim, dependencies install)

**What We DON'T Test:**
- ❌ HTTP requests to Controller API
- ❌ JWT authentication flow
- ❌ Network connectivity
- ❌ Error handling with real HTTP responses
- ❌ Privacy Guard integration
- ❌ Audit event emission

**Why This Approach:**
- **Fast feedback** - No need to start Controller API
- **Isolated** - Tests only the tool code, not the full stack
- **Development-friendly** - Can iterate on tool structure quickly
- **Docker-based** - Consistent environment, no local Python setup needed

**Example Tests:**
```python
# test_send_task.py
assert send_task_tool.name == "send_task"
assert "target" in send_task_tool.inputSchema["properties"]
assert "target" in send_task_tool.inputSchema["required"]

# Validates structure, NOT HTTP behavior
```

---

### Layer 2: Integration Tests (B7) 🔜 UPCOMING

**Purpose:** Test tools WITH running Controller API, verify HTTP communication

**What We Test:**
- ✅ MCP server starts successfully
- ✅ Tools visible via MCP protocol
- ✅ send_task → POST /tasks/route (202 Accepted response)
- ✅ request_approval → POST /approvals (202 Accepted response)
- ✅ notify → POST /tasks/route (notification type)
- ✅ fetch_status → GET /sessions/{task_id} (200 OK response)
- ✅ JWT authentication (valid token → success, invalid → 401)
- ✅ Idempotency key handling
- ✅ Error handling (4xx, 5xx, timeout, connection errors)
- ✅ Privacy Guard integration (task data masked)
- ✅ Audit events emitted (traceId propagation)

**Prerequisites:**
- Controller API running (`cargo run` in src/controller/)
- JWT token from Keycloak (MESH_JWT_TOKEN env var)
- Controller configured with Privacy Guard, Keycloak, etc.

**Test Flow:**
```python
# tests/test_integration.py (B7)

import pytest
from mcp.client import Client
import requests

@pytest.fixture
def controller_running():
    """Ensure Controller API is running on localhost:8088"""
    response = requests.get("http://localhost:8088/status")
    assert response.status_code == 200
    yield

@pytest.mark.asyncio
async def test_send_task_integration(controller_running):
    """Test send_task tool with running Controller API"""
    client = Client("agent-mesh")
    
    # Call send_task tool via MCP
    result = await client.call_tool("send_task", {
        "target": "manager",
        "task": {"type": "test", "data": "integration test"},
        "context": {}
    })
    
    # Verify response
    assert "Task routed successfully" in result.text
    assert "task-" in result.text  # Task ID in response
    
    # Verify Controller API received the request
    # (check audit events, etc.)
```

---

### Layer 3: End-to-End Tests (Workstream C) 🔜 FINAL STAGE

**Purpose:** Test full cross-agent workflow with real Goose instances

**What We Test:**
- ✅ Finance Goose → send_task to Manager Goose
- ✅ Manager Goose → fetch_status to check task
- ✅ Manager Goose → submit approval via Controller
- ✅ Finance Goose → fetch_status to see approval
- ✅ Audit trail complete (all events logged with traceId)
- ✅ Privacy Guard masked sensitive data
- ✅ Backward compatibility (Phase 1.2 JWT, Phase 2.2 Privacy Guard still work)

**Prerequisites:**
- 2 Goose instances running (Finance, Manager)
- Controller API running with full stack (Keycloak, Vault, Privacy Guard)
- Agent Mesh MCP extension loaded in both Goose instances

**Test Flow:**
```bash
# Terminal 1: Controller API
cd src/controller && cargo run

# Terminal 2: Finance Goose
goose session start --profile finance-agent

# Terminal 3: Manager Goose
goose session start --profile manager-agent

# Finance Goose:
> Use agent_mesh__send_task to request budget approval from manager

# Manager Goose:
> Use agent_mesh__fetch_status to check the task
> Approve via Controller API (curl or agent tool)

# Finance Goose:
> Use agent_mesh__fetch_status to see approval
```

---

## Why This Multi-Layer Approach?

### Development Efficiency
- **Layer 1** (Validation) → Fast iteration, no dependencies
- **Layer 2** (Integration) → Catch HTTP/auth issues early
- **Layer 3** (E2E) → Validate real-world usage

### Risk Mitigation
- **Layer 1** catches 80% of bugs (structure, schema, params)
- **Layer 2** catches HTTP/auth/error handling bugs (15%)
- **Layer 3** catches workflow/integration bugs (5%)

### Time Optimization
- **Layer 1**: ~5 minutes per tool (automated, Docker-based)
- **Layer 2**: ~30 minutes for all 4 tools (manual Controller startup)
- **Layer 3**: ~2 hours (full stack, 2 Goose instances, workflow execution)

**Total Time Saved:** ~1.5 hours by NOT testing with Controller in B2-B5

---

## Current Status (Phase 3)

### ✅ Completed
- **A1-A6**: Controller API with 21 unit tests (Layer 1 equivalent for Rust)
- **B1**: Agent Mesh scaffold
- **B2**: send_task tool + Layer 1 validation tests
- **B3**: request_approval tool + Layer 1 validation tests

### 🏗️ In Progress
- **B4**: notify tool (Layer 1 validation)
- **B5**: fetch_status tool (Layer 1 validation)

### ⏳ Upcoming
- **B6**: Configuration docs update (4 tools documented)
- **B7**: Integration tests (Layer 2 - WITH Controller API) ← **TESTS WITH CONTROLLER**
- **B8**: Deployment + ADR-0024
- **B9**: Workstream B checkpoint

### 🔜 Final Stage
- **C1-C5**: Cross-agent demo (Layer 3 - E2E with Goose instances)

---

## Key Takeaways

1. **Validation tests (B2-B5)** are NOT a replacement for Controller API tests
2. **Integration tests (B7)** will test WITH running Controller API
3. **E2E tests (C1-C5)** will test the full workflow with 2 Goose instances
4. This multi-layer approach is **faster** and **more efficient** than testing with Controller API during tool development
5. All HTTP/auth/error handling will be thoroughly tested in B7 and C1-C5

---

**Created:** 2025-11-04  
**Last Updated:** 2025-11-04  
**Status:** CURRENT APPROACH
