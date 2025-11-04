# Phase 3 Execution Plan — Controller API + Agent Mesh

**Phase:** 3 (M2 Milestone)  
**Type:** Medium (M) - ~8-9 days  
**Priority:** HIGH (Core orchestration capability)  
**Date Created:** 2025-11-04  
**Prerequisites:** Phases 0, 1, 1.2, 2, 2.2, **2.5** complete  
**Blocks:** Phase 4 (Directory Service + Policy Engine)

---

## 🎯 Objectives

### Primary Goal
Implement core multi-agent orchestration capability:

1. **Controller API (Rust):** HTTP endpoints for task routing, session management, approval workflows
2. **Agent Mesh MCP (Python):** MCP extension for Goose with 4 tools (send_task, request_approval, notify, fetch_status)
3. **Cross-Agent Demo:** Finance agent → Manager agent approval flow

### Success Criteria (M2 Milestone)
- ✅ Controller API endpoints functional (POST /tasks/route, GET/POST /sessions, POST /approvals, GET /profiles/{role})
- ✅ OpenAPI spec published and validated (Spectral lint passes)
- ✅ Agent Mesh MCP extension loads in Goose
- ✅ All 4 MCP tools functional (send_task, request_approval, notify, fetch_status)
- ✅ Cross-agent approval demo works (Finance → Manager)
- ✅ Audit events emitted (traceId propagation)
- ✅ Integration tests pass (100%)
- ✅ **ADR-0024 created: "Agent Mesh Python Implementation"**
- ✅ **ADR-0025 created: "Controller API v1 Design"**
- ✅ No breaking changes to Phase 1.2/2.2 functionality

---

## 📋 Technical Design Summary

### Controller API (Rust/Axum)
**File:** `src/controller/src/main.rs` (extend existing)

**New Routes:**
- `POST /tasks/route` - Route task to target agent
- `GET /sessions` - List active sessions
- `POST /sessions` - Create new session
- `POST /approvals` - Submit approval decision
- `GET /profiles/{role}` - Get agent profile by role

**Middleware:**
- JWT verification (existing from Phase 1.2)
- Privacy Guard integration (existing from Phase 2.2)
- Idempotency key validation (new)
- Request size limit: 1MB (new)
- traceId extraction/generation (new)

**Dependencies to Add:**
- `utoipa = "4.0"` - OpenAPI generation
- `uuid = "1.6"` - Idempotency keys + traceId

### Agent Mesh MCP (Python)
**Location:** `src/agent-mesh/` (new directory)

**Structure:**
```
src/agent-mesh/
├── pyproject.toml           # Python package config
├── agent_mesh_server.py     # MCP server entry point
├── tools/
│   ├── send_task.py         # send_task tool
│   ├── request_approval.py  # request_approval tool
│   ├── notify.py            # notify tool
│   └── fetch_status.py      # fetch_status tool
├── client/
│   └── controller_client.py # HTTP client for Controller API
└── README.md                # Setup + usage docs
```

**Dependencies:**
- `mcp >= 1.0.0` - MCP SDK (Python)
- `requests >= 2.31.0` - HTTP client
- `pydantic >= 2.0.0` - Data validation

**MCP Tools:**
1. **send_task(target: str, task: dict, context: dict) → dict**
   - Calls `POST /tasks/route`
   - Returns `{taskId, status}`
   
2. **request_approval(task_id: str, approver_role: str, reason: str) → dict**
   - Calls `POST /approvals`
   - Returns `{approvalId, status}`

3. **notify(target: str, message: str, priority: str) → dict**
   - Calls `POST /tasks/route` (notification type)
   - Returns `{ack: true}`

4. **fetch_status(task_id: str) → dict**
   - Calls `GET /sessions/{task_id}`
   - Returns `{status, progress, result}`

### Cross-Agent Demo
**Scenario:** Finance agent requests budget approval from Manager agent

```python
# Finance Agent (Goose instance 1)
result = send_task(
    target="manager",
    task={"type": "budget_approval", "amount": 50000},
    context={"department": "Engineering", "quarter": "Q1"}
)

approval = request_approval(
    task_id=result["taskId"],
    approver_role="manager",
    reason="Budget increase for new hires"
)

# Manager Agent (Goose instance 2)
status = fetch_status(task_id=result["taskId"])
# Review and approve via Controller API
```

---

## 🔧 Workstream Breakdown

### Workstream A: Controller API (Rust/Axum) - ~3 days

**Objective:** Implement minimal OpenAPI endpoints for task routing, sessions, approvals

---

#### A1. OpenAPI Schema Design with utoipa (~4 hours)

**File:** `src/controller/src/api/openapi.rs` (new)

**Tasks:**
1. Add `utoipa` and `uuid` dependencies to `src/controller/Cargo.toml`
```toml
[dependencies]
utoipa = { version = "4.0", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "4.0", features = ["axum"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
```

2. Create OpenAPI schema structs:
```rust
use utoipa::{OpenApi, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::route_task,
        crate::routes::list_sessions,
        crate::routes::create_session,
        crate::routes::submit_approval,
        crate::routes::get_profile,
    ),
    components(schemas(
        RouteTaskRequest,
        RouteTaskResponse,
        SessionResponse,
        ApprovalRequest,
        ApprovalResponse,
        ProfileResponse,
    )),
    tags(
        (name = "tasks", description = "Task routing endpoints"),
        (name = "sessions", description = "Session management"),
        (name = "approvals", description = "Approval workflows"),
        (name = "profiles", description = "Agent profiles"),
    )
)]
pub struct ApiDoc;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RouteTaskRequest {
    #[schema(example = "manager")]
    pub target: String,
    
    #[schema(example = json!({"type": "budget_approval", "amount": 50000}))]
    pub task: serde_json::Value,
    
    #[schema(example = json!({"department": "Engineering"}))]
    pub context: serde_json::Value,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RouteTaskResponse {
    #[schema(example = "task-123e4567-e89b-12d3-a456-426614174000")]
    pub task_id: String,
    
    #[schema(example = "routed")]
    pub status: String,
}

// Similar for other request/response types...
```

3. Mount Swagger UI in `main.rs`:
```rust
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    // ... existing routes
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
```

**Deliverables:**
- ✅ `src/controller/src/api/openapi.rs` created
- ✅ Swagger UI accessible at `http://localhost:8088/swagger-ui`
- ✅ OpenAPI spec at `http://localhost:8088/api-docs/openapi.json`

---

#### A2. Route Implementations (~1 day)

**File:** `src/controller/src/routes/tasks.rs` (new)

**Tasks:**

**A2.1. POST /tasks/route** (~3 hours)
```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/tasks/route",
    request_body = RouteTaskRequest,
    responses(
        (status = 202, description = "Task routed successfully", body = RouteTaskResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn route_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RouteTaskRequest>,
) -> Result<(StatusCode, Json<RouteTaskResponse>), AppError> {
    // 1. Extract/generate traceId
    let trace_id = headers
        .get("x-trace-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // 2. Validate idempotency key
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::MissingIdempotencyKey)?;
    
    // 3. Check request size (< 1MB)
    // (Handled by Axum middleware with DefaultBodyLimit)
    
    // 4. Privacy Guard: mask sensitive data in task/context
    let masked_task = state.guard_client.mask_json(&req.task).await?;
    
    // 5. Store task in database (TODO: Phase 4)
    let task_id = Uuid::new_v4().to_string();
    
    // 6. Emit audit event
    state.audit_client.log_event(AuditEvent {
        source: "controller",
        category: "task",
        action: "routed",
        trace_id: trace_id.clone(),
        metadata: json!({"task_id": task_id, "target": req.target}),
    }).await?;
    
    // 7. Return response
    Ok((
        StatusCode::ACCEPTED,
        Json(RouteTaskResponse {
            task_id,
            status: "routed".to_string(),
        })
    ))
}
```

**A2.2. GET /sessions** (~2 hours)
```rust
#[utoipa::path(
    get,
    path = "/sessions",
    responses(
        (status = 200, description = "List of sessions", body = Vec<SessionResponse>),
    ),
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("limit" = Option<u32>, Query, description = "Max results (default: 100)")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<SessionQueryParams>,
) -> Result<Json<Vec<SessionResponse>>, AppError> {
    // TODO: Query from database (Phase 4)
    // For now, return empty array
    Ok(Json(vec![]))
}
```

**A2.3. POST /sessions** (~1 hour)
```rust
#[utoipa::path(
    post,
    path = "/sessions",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created", body = SessionResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), AppError> {
    let session_id = Uuid::new_v4().to_string();
    
    // TODO: Store in database (Phase 4)
    
    Ok((
        StatusCode::CREATED,
        Json(SessionResponse {
            session_id,
            status: "active".to_string(),
            created_at: chrono::Utc::now(),
        })
    ))
}
```

**A2.4. POST /approvals** (~2 hours)
```rust
#[utoipa::path(
    post,
    path = "/approvals",
    request_body = ApprovalRequest,
    responses(
        (status = 202, description = "Approval submitted", body = ApprovalResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn submit_approval(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ApprovalRequest>,
) -> Result<(StatusCode, Json<ApprovalResponse>), AppError> {
    let trace_id = headers
        .get("x-trace-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    let approval_id = Uuid::new_v4().to_string();
    
    // Emit audit event
    state.audit_client.log_event(AuditEvent {
        source: "controller",
        category: "approval",
        action: "submitted",
        trace_id,
        metadata: json!({
            "approval_id": approval_id,
            "task_id": req.task_id,
            "decision": req.decision,
        }),
    }).await?;
    
    Ok((
        StatusCode::ACCEPTED,
        Json(ApprovalResponse {
            approval_id,
            status: "pending".to_string(),
        })
    ))
}
```

**A2.5. GET /profiles/{role}** (~1 hour)
```rust
#[utoipa::path(
    get,
    path = "/profiles/{role}",
    responses(
        (status = 200, description = "Agent profile", body = ProfileResponse),
        (status = 404, description = "Profile not found"),
    ),
    params(
        ("role" = String, Path, description = "Agent role (e.g., 'manager', 'finance')")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Json<ProfileResponse>, AppError> {
    // TODO: Query from Directory Service (Phase 4)
    // For now, return mock profile
    Ok(Json(ProfileResponse {
        role,
        capabilities: vec!["task_routing".to_string()],
        status: "active".to_string(),
    }))
}
```

**Deliverables:**
- ✅ All 5 routes implemented
- ✅ Request validation (Idempotency-Key header, request size)
- ✅ Privacy Guard integration on task data
- ✅ Audit events emitted with traceId

---

#### A3. Idempotency + Request Limits Middleware (~4 hours)

**File:** `src/controller/src/middleware/idempotency.rs` (new)

**Tasks:**

**A3.1. Idempotency Key Validation**
```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn validate_idempotency_key(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Only check for POST/PUT/DELETE
    if !matches!(req.method(), &Method::POST | &Method::PUT | &Method::DELETE) {
        return Ok(next.run(req).await);
    }
    
    // Require Idempotency-Key header
    let headers = req.headers();
    let key = headers
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Validate UUID format
    Uuid::parse_str(key).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // TODO: Check cache for duplicate (Phase 4 with Redis)
    
    Ok(next.run(req).await)
}
```

**A3.2. Request Size Limit**
```rust
use tower_http::limit::RequestBodyLimitLayer;

// In main.rs:
let app = Router::new()
    .route("/tasks/route", post(route_task))
    // ... other routes
    .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
    .layer(middleware::from_fn(validate_idempotency_key));
```

**Deliverables:**
- ✅ Idempotency key validation middleware
- ✅ 1MB request size limit
- ✅ Proper error responses (400 for invalid key, 413 for too large)

---

#### A4. Privacy Guard Integration (~3 hours)

**File:** `src/controller/src/guard/mod.rs` (extend existing from Phase 2.2)

**Tasks:**

**A4.1. JSON Masking Utility**
```rust
impl GuardClient {
    pub async fn mask_json(&self, data: &serde_json::Value) -> Result<serde_json::Value, GuardError> {
        match data {
            Value::String(s) => {
                let masked = self.mask_text(s).await?;
                Ok(Value::String(masked))
            }
            Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (k, v) in map {
                    result.insert(k.clone(), self.mask_json(v).await?);
                }
                Ok(Value::Object(result))
            }
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr {
                    result.push(self.mask_json(item).await?);
                }
                Ok(Value::Array(result))
            }
            _ => Ok(data.clone()), // Numbers, bools, null unchanged
        }
    }
}
```

**A4.2. Integrate into route_task**
- Call `guard_client.mask_json(&req.task)` before storing
- Call `guard_client.mask_json(&req.context)` for context data
- Log Privacy Guard latency in traceId metadata

**Deliverables:**
- ✅ JSON masking utility
- ✅ Integration in POST /tasks/route
- ✅ Privacy Guard latency metrics

---

#### A5. Unit Tests (~4 hours)

**File:** `src/controller/src/routes/tasks_test.rs` (new)

**Tests:**
1. **POST /tasks/route**
   - ✅ Valid request returns 202 Accepted
   - ✅ Missing Idempotency-Key returns 400
   - ✅ Invalid UUID in Idempotency-Key returns 400
   - ✅ Request > 1MB returns 413
   - ✅ Unauthorized (no JWT) returns 401
   - ✅ Privacy Guard called for task data

2. **GET /sessions**
   - ✅ Returns 200 OK (empty array for now)
   - ✅ Unauthorized (no JWT) returns 401

3. **POST /approvals**
   - ✅ Valid request returns 202 Accepted
   - ✅ Audit event emitted

4. **GET /profiles/{role}**
   - ✅ Returns 200 OK with mock profile
   - ✅ Valid role formats accepted

**Deliverables:**
- ✅ Unit tests for all routes
- ✅ Tests pass with `cargo test`

---

### Workstream B: Agent Mesh MCP (Python) - ~4-5 days

**Objective:** Create Python MCP server with 4 tools for Controller API interaction

---

#### B1. MCP Server Scaffold (~4 hours)

**Tasks:**

**B1.1. Project Setup**
```bash
mkdir -p src/agent-mesh/tools
cd src/agent-mesh

# Create pyproject.toml
cat > pyproject.toml <<'EOF'
[project]
name = "agent-mesh"
version = "0.1.0"
description = "MCP server for multi-agent orchestration via Controller API"
requires-python = ">=3.13"
dependencies = [
    "mcp>=1.0.0",
    "requests>=2.31.0",
    "pydantic>=2.0.0",
]

[project.scripts]
agent-mesh = "agent_mesh_server:main"

[tool.ruff]
line-length = 100
target-version = "py313"
EOF

# Install dependencies
python -m venv .venv
source .venv/bin/activate
pip install -e .
```

**B1.2. Server Entry Point**

**File:** `src/agent-mesh/agent_mesh_server.py`
```python
import asyncio
import os
from mcp.server import Server
from mcp.server.stdio import stdio_server

from tools.send_task import send_task_tool
from tools.request_approval import request_approval_tool
from tools.notify import notify_tool
from tools.fetch_status import fetch_status_tool

async def main():
    server = Server("agent-mesh")
    
    # Register tools
    server.add_tool(send_task_tool)
    server.add_tool(request_approval_tool)
    server.add_tool(notify_tool)
    server.add_tool(fetch_status_tool)
    
    # Run stdio transport
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream)

if __name__ == "__main__":
    asyncio.run(main())
```

**Deliverables:**
- ✅ Python project structure created
- ✅ Dependencies installed (mcp, requests, pydantic)
- ✅ MCP server scaffold runs (`python agent_mesh_server.py`)

---

#### B2. send_task Tool (~6 hours)

**File:** `src/agent-mesh/tools/send_task.py`

**Tasks:**

**B2.1. Tool Definition**
```python
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests
import uuid
import os
import time
import random

class SendTaskParams(BaseModel):
    target: str = Field(description="Target agent role (e.g., 'manager', 'finance')")
    task: dict = Field(description="Task payload (JSON object)")
    context: dict = Field(default_factory=dict, description="Additional context (optional)")

async def send_task_tool(params: SendTaskParams) -> list[TextContent]:
    """
    Route a task to another agent via the Controller API.
    
    Implements retry logic with exponential backoff + jitter.
    """
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    max_retries = int(os.getenv("MESH_RETRY_COUNT", "3"))
    timeout = int(os.getenv("MESH_TIMEOUT_SECS", "30"))
    
    if not jwt_token:
        return [TextContent(
            type="text",
            text="ERROR: MESH_JWT_TOKEN environment variable not set"
        )]
    
    # Generate idempotency key
    idempotency_key = str(uuid.uuid4())
    
    # Retry loop
    for attempt in range(max_retries):
        try:
            response = requests.post(
                f"{controller_url}/tasks/route",
                headers={
                    "Authorization": f"Bearer {jwt_token}",
                    "Content-Type": "application/json",
                    "Idempotency-Key": idempotency_key,
                    "X-Trace-Id": str(uuid.uuid4()),
                },
                json={
                    "target": params.target,
                    "task": params.task,
                    "context": params.context,
                },
                timeout=timeout,
            )
            
            response.raise_for_status()
            data = response.json()
            
            return [TextContent(
                type="text",
                text=f"Task routed successfully. Task ID: {data['task_id']}, Status: {data['status']}"
            )]
        
        except requests.exceptions.RequestException as e:
            if attempt < max_retries - 1:
                # Exponential backoff with jitter
                wait_time = (2 ** attempt) + random.uniform(0, 1)
                time.sleep(wait_time)
                continue
            else:
                return [TextContent(
                    type="text",
                    text=f"ERROR: Failed to route task after {max_retries} attempts: {str(e)}"
                )]

# Tool metadata
send_task_tool = Tool(
    name="send_task",
    description="Route a task to another agent via the Controller API",
    inputSchema=SendTaskParams.model_json_schema(),
)
send_task_tool.call = send_task_tool
```

**Deliverables:**
- ✅ send_task tool implemented
- ✅ Retry logic (3x exponential backoff + jitter)
- ✅ Idempotency key generation
- ✅ Environment variable configuration

---

#### B3. request_approval Tool (~4 hours)

**File:** `src/agent-mesh/tools/request_approval.py`

```python
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests
import uuid
import os

class RequestApprovalParams(BaseModel):
    task_id: str = Field(description="Task ID requiring approval")
    approver_role: str = Field(description="Role of approver (e.g., 'manager')")
    reason: str = Field(description="Reason for approval request")

async def request_approval_tool(params: RequestApprovalParams) -> list[TextContent]:
    """Request approval for a task from a specific agent role."""
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    
    if not jwt_token:
        return [TextContent(type="text", text="ERROR: MESH_JWT_TOKEN not set")]
    
    response = requests.post(
        f"{controller_url}/approvals",
        headers={
            "Authorization": f"Bearer {jwt_token}",
            "Content-Type": "application/json",
            "Idempotency-Key": str(uuid.uuid4()),
        },
        json={
            "task_id": params.task_id,
            "approver_role": params.approver_role,
            "reason": params.reason,
        },
        timeout=30,
    )
    
    response.raise_for_status()
    data = response.json()
    
    return [TextContent(
        type="text",
        text=f"Approval requested. Approval ID: {data['approval_id']}, Status: {data['status']}"
    )]

request_approval_tool = Tool(
    name="request_approval",
    description="Request approval for a task from a specific agent role",
    inputSchema=RequestApprovalParams.model_json_schema(),
)
request_approval_tool.call = request_approval_tool
```

**Deliverables:**
- ✅ request_approval tool implemented
- ✅ Idempotency key handling

---

#### B4. notify Tool (~3 hours)

**File:** `src/agent-mesh/tools/notify.py`

```python
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests
import uuid
import os

class NotifyParams(BaseModel):
    target: str = Field(description="Target agent role")
    message: str = Field(description="Notification message")
    priority: str = Field(default="normal", description="Priority: low, normal, high")

async def notify_tool(params: NotifyParams) -> list[TextContent]:
    """Send a notification to another agent."""
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    
    if not jwt_token:
        return [TextContent(type="text", text="ERROR: MESH_JWT_TOKEN not set")]
    
    # Use send_task endpoint with notification type
    response = requests.post(
        f"{controller_url}/tasks/route",
        headers={
            "Authorization": f"Bearer {jwt_token}",
            "Content-Type": "application/json",
            "Idempotency-Key": str(uuid.uuid4()),
        },
        json={
            "target": params.target,
            "task": {
                "type": "notification",
                "message": params.message,
                "priority": params.priority,
            },
            "context": {},
        },
        timeout=30,
    )
    
    response.raise_for_status()
    
    return [TextContent(type="text", text="Notification sent successfully")]

notify_tool = Tool(
    name="notify",
    description="Send a notification to another agent",
    inputSchema=NotifyParams.model_json_schema(),
)
notify_tool.call = notify_tool
```

**Deliverables:**
- ✅ notify tool implemented

---

#### B5. fetch_status Tool (~3 hours)

**File:** `src/agent-mesh/tools/fetch_status.py`

```python
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests
import os

class FetchStatusParams(BaseModel):
    task_id: str = Field(description="Task ID to check status")

async def fetch_status_tool(params: FetchStatusParams) -> list[TextContent]:
    """Fetch the status of a routed task."""
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    
    if not jwt_token:
        return [TextContent(type="text", text="ERROR: MESH_JWT_TOKEN not set")]
    
    response = requests.get(
        f"{controller_url}/sessions/{params.task_id}",
        headers={"Authorization": f"Bearer {jwt_token}"},
        timeout=30,
    )
    
    response.raise_for_status()
    data = response.json()
    
    return [TextContent(
        type="text",
        text=f"Task Status: {data.get('status', 'unknown')}\n"
             f"Progress: {data.get('progress', 'N/A')}\n"
             f"Result: {data.get('result', 'N/A')}"
    )]

fetch_status_tool = Tool(
    name="fetch_status",
    description="Fetch the status of a routed task",
    inputSchema=FetchStatusParams.model_json_schema(),
)
fetch_status_tool.call = fetch_status_tool
```

**Deliverables:**
- ✅ fetch_status tool implemented

---

#### B6. Configuration & Environment (~2 hours)

**File:** `src/agent-mesh/.env.example`

```bash
# Controller API Configuration
CONTROLLER_URL=http://localhost:8088

# JWT Token (obtain from Keycloak)
MESH_JWT_TOKEN=eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...

# Retry Configuration
MESH_RETRY_COUNT=3
MESH_TIMEOUT_SECS=30
```

**File:** `src/agent-mesh/README.md`

```markdown
# Agent Mesh MCP Server

MCP extension for Goose that enables multi-agent orchestration via the Controller API.

## Installation

```bash
cd src/agent-mesh
python -m venv .venv
source .venv/bin/activate
pip install -e .
```

## Configuration

Create `.env` file:
```bash
cp .env.example .env
# Edit .env with your CONTROLLER_URL and MESH_JWT_TOKEN
```

## Goose Integration

Add to `~/.config/goose/profiles.yaml`:

```yaml
extensions:
  agent_mesh:
    type: mcp
    command: ["python", "-m", "agent_mesh_server"]
    working_dir: "/home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh"
    env:
      CONTROLLER_URL: "http://localhost:8088"
      MESH_JWT_TOKEN: "eyJ..."
```

## Tools

1. **send_task** - Route task to another agent
2. **request_approval** - Request approval from specific role
3. **notify** - Send notification to another agent
4. **fetch_status** - Check status of routed task

## Usage

```bash
# In Goose session
Agent Mesh tools available:
- agent_mesh__send_task
- agent_mesh__request_approval
- agent_mesh__notify
- agent_mesh__fetch_status
```
```

**Deliverables:**
- ✅ Environment variable configuration
- ✅ README with setup instructions
- ✅ Goose profiles.yaml integration guide

---

#### B7. Integration Testing (~6 hours)

**File:** `src/agent-mesh/tests/test_integration.py`

**Tests:**
1. **Tool Discovery**
   - ✅ MCP server starts successfully
   - ✅ All 4 tools listed in capabilities

2. **send_task**
   - ✅ Valid task routes successfully (202 Accepted)
   - ✅ Returns task_id in response
   - ✅ Missing JWT returns error
   - ✅ Retry logic works on transient failures

3. **request_approval**
   - ✅ Approval request submitted (202 Accepted)
   - ✅ Returns approval_id

4. **notify**
   - ✅ Notification sent successfully

5. **fetch_status**
   - ✅ Returns task status (200 OK)

**Deliverables:**
- ✅ Integration tests pass (pytest)

---

#### B8. Deployment & Docs (~4 hours)

**Tasks:**
1. Document Goose extension loading
2. Test with actual Goose instance
3. Verify tools visible in Goose tool list
4. Update VERSION_PINS.md with Agent Mesh version

**Deliverables:**
- ✅ Agent Mesh loadable in Goose
- ✅ All tools functional from Goose CLI
- ✅ Documentation complete

---

### Workstream C: Cross-Agent Approval Demo (~1 day)

**Objective:** Demonstrate multi-agent orchestration with approval workflow

---

#### C1. Demo Scenario Design (~2 hours)

**Scenario:** Finance Agent requests budget approval from Manager Agent

**Actors:**
- **Finance Agent** (Goose instance 1)
- **Manager Agent** (Goose instance 2)
- **Controller API** (central orchestrator)

**Flow:**
1. Finance creates budget request task
2. Finance sends task to Manager via `send_task`
3. Manager receives notification (via Controller polling)
4. Manager reviews task via `fetch_status`
5. Manager approves/rejects via Controller API
6. Finance checks approval status

**Deliverables:**
- ✅ Scenario documented in `docs/demos/cross-agent-approval.md`

---

#### C2. Implementation (~4 hours)

**File:** `docs/demos/cross-agent-approval.md`

```markdown
# Cross-Agent Approval Demo

## Setup

### Terminal 1: Start Controller API
```bash
cd src/controller
cargo run --release
# Listening on http://localhost:8088
```

### Terminal 2: Finance Agent (Goose instance 1)
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
goose session start --profile finance-agent
```

### Terminal 3: Manager Agent (Goose instance 2)
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
goose session start --profile manager-agent
```

## Execution

### Step 1: Finance Agent - Create Budget Request

In Goose (Finance Agent):
```
Use agent_mesh__send_task to request budget approval:
- target: "manager"
- task: {"type": "budget_approval", "amount": 50000, "purpose": "Q1 new hires"}
- context: {"department": "Engineering", "quarter": "Q1-2026"}
```

Expected output:
```
Task routed successfully. Task ID: task-abc123..., Status: routed
```

### Step 2: Manager Agent - Check Pending Tasks

In Goose (Manager Agent):
```
Use agent_mesh__fetch_status to check task:
- task_id: "task-abc123..."
```

Expected output:
```
Task Status: pending_approval
Progress: awaiting_manager_review
Result: N/A
```

### Step 3: Manager Agent - Approve Request

Via curl (simulating Manager's decision):
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

### Step 4: Finance Agent - Check Approval Status

In Goose (Finance Agent):
```
Use agent_mesh__fetch_status again:
- task_id: "task-abc123..."
```

Expected output:
```
Task Status: approved
Progress: 100%
Result: {"decision": "approved", "approver": "manager", "comments": "..."}
```

## Validation

✅ Task routed from Finance to Manager  
✅ Manager received task notification  
✅ Manager approved task via Controller API  
✅ Finance retrieved approval status  
✅ Audit events emitted for all steps  
✅ traceId propagated through workflow
```

**Deliverables:**
- ✅ Demo script documented
- ✅ All steps executable

---

#### C3. Smoke Test Procedure (~2 hours)

**File:** `docs/tests/smoke-phase3.md`

**Tests:**
1. **Controller API Health**
   - ✅ GET /status returns 200 OK
   - ✅ Swagger UI accessible at /swagger-ui

2. **Agent Mesh Loading**
   - ✅ Extension loads in Goose
   - ✅ All 4 tools visible (`goose tools list | grep agent_mesh`)

3. **Cross-Agent Communication**
   - ✅ send_task from Finance → Manager succeeds
   - ✅ fetch_status returns task details
   - ✅ Approval workflow completes

4. **Audit Trail**
   - ✅ Audit events emitted for task routing
   - ✅ Audit events emitted for approvals
   - ✅ traceId consistent across events

5. **Backward Compatibility**
   - ✅ Phase 1.2 JWT auth still works
   - ✅ Phase 2.2 Privacy Guard still functional

**Deliverables:**
- ✅ Smoke test document created
- ✅ All tests pass (5/5)

---

## 📊 Timeline

**Total Estimated Effort:** ~8-9 days

```
Day 1-3:   Workstream A (Controller API)
  Day 1:   A1 (OpenAPI schema) + A2.1-A2.2 (route_task, list_sessions)
  Day 2:   A2.3-A2.5 (create_session, approvals, profiles) + A3 (middleware)
  Day 3:   A4 (Privacy Guard) + A5 (unit tests)

Day 4-8:   Workstream B (Agent Mesh MCP)
  Day 4:   B1 (scaffold) + B2 (send_task)
  Day 5:   B3 (request_approval) + B4 (notify)
  Day 6:   B5 (fetch_status) + B6 (config)
  Day 7:   B7 (integration tests)
  Day 8:   B8 (deployment + docs)

Day 9:     Workstream C (Cross-Agent Demo)
  Day 9:   C1 (scenario) + C2 (implementation) + C3 (smoke tests)
```

---

## 🎯 Milestones

**M1 (Day 3):** Controller API functional, unit tests pass  
**M2 (Day 6):** All 4 MCP tools implemented  
**M3 (Day 8):** Agent Mesh integration tests pass  
**M4 (Day 9):** Cross-agent demo works, smoke tests pass, **ADRs created**, Phase 3 complete

---

## ⚠️ Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **MCP SDK API changes** | LOW | HIGH | Pin to mcp~=1.0.0, test with Goose v1.12 |
| **Rust compatibility** | LOW | LOW | Using Rust 1.83.0 (validated in Phase 2.5) |
| **Privacy Guard latency** | MEDIUM | MEDIUM | Cache guard_client, measure P50 < 5s |
| **Goose extension loading fails** | LOW | HIGH | Test profiles.yaml format, validate command array |
| **JWT token expiration during demo** | MEDIUM | LOW | Use long-lived service account token |

---

## 📝 Acceptance Criteria

### Must Pass
- ✅ Controller API: All 5 routes functional
- ✅ OpenAPI spec validates with Spectral
- ✅ Agent Mesh: All 4 tools functional in Goose
- ✅ Cross-agent demo: Finance → Manager approval works end-to-end
- ✅ Integration tests: 100% pass
- ✅ Smoke tests: 5/5 pass
- ✅ Audit events emitted with traceId propagation
- ✅ **ADR-0024 created: "Agent Mesh Python Implementation"**
- ✅ **ADR-0025 created: "Controller API v1 Design"**
- ✅ No breaking changes to Phase 1.2/2.2

### Nice to Have
- ⭐ Performance: P50 latency < 5s for agent_mesh__send_task
- ⭐ Documentation: Architecture diagrams for Controller + Agent Mesh interaction
- ⭐ Error handling: Retry exhausted → user-friendly error messages

---

## 🔗 Dependencies

### Upstream (Completed)
- ✅ Phase 0: Infrastructure bootstrap
- ✅ Phase 1.2: JWT verification middleware (RS256, JWKS)
- ✅ Phase 2.2: Privacy Guard integration (Vault, Ollama)
- ✅ **Phase 2.5: Dependency upgrades (Keycloak 26, Vault 1.18, Python 3.13, Rust 1.83)**
  - Note: Rust 1.91.0 was tested but deferred (requires Clone derives on structs)

### Downstream (Blocked Until Complete)
- ⏸️ Phase 4: Directory Service + Policy Engine
  - **Requirement:** Controller API stable (task routing, sessions)
  - **Requirement:** Agent Mesh MCP proven (send_task, fetch_status)

---

## 📚 Reference Documents

### ADRs to Create
- **ADR-0024: Agent Mesh Python Implementation** (create in Workstream B8)
  - Decision: Use Python + mcp SDK (not Rust + rmcp)
  - Rationale: Faster MVP, easier HTTP client, migration straightforward
  - Consequences: ~2-3 day migration to Rust post-Phase 3 if needed

- **ADR-0025: Controller API v1 Design** (create in Workstream A5)
  - Decision: Minimal OpenAPI (5 routes), defer persistence to Phase 4
  - Rationale: Unblock Agent Mesh development, validate API shape
  - Consequences: Stateless for Phase 3 (ephemeral task storage)

### External Documentation
- **MCP Protocol:** https://modelcontextprotocol.io/
- **Goose Extension Loading:** goose-versions-references/how-goose-works-docs/docs/extensions.md
- **Axum Framework:** https://docs.rs/axum/latest/axum/
- **utoipa OpenAPI:** https://docs.rs/utoipa/latest/utoipa/

### Internal Documentation
- **Phase 3 Pre-Flight Analysis:** `Technical Project Plan/PM Phases/Phase-3-PRE-FLIGHT-ANALYSIS.md`
- **Goose v1.12 MCP Reference:** `goose-versions-references/gooseV1.12.00/crates/goose-mcp/src/developer/rmcp_developer.rs`
- **Controller Stub:** `src/controller/src/main.rs`
- **OpenAPI Stub:** `docs/api/controller/openapi.yaml`

---

## 🚀 Execution Workflow

### 1. Pre-Execution Checklist
- [ ] Phase 2.5 complete and merged to main
- [ ] Review Phase-2.5/ folder for any changes from upgrades
- [ ] Git working directory clean
- [ ] Rust 1.83.0 Docker image available (already validated in Phase 2.5)
- [ ] Python 3.13.9 Docker image available (`docker pull python:3.13-slim`)

### 2. Execute Workstreams
- [ ] Workstream A: Controller API (Days 1-3)
- [ ] Workstream B: Agent Mesh MCP (Days 4-8)
- [ ] Workstream C: Cross-Agent Demo (Day 9)

### 3. Post-Execution
- [ ] Create Phase-3-Completion-Summary.md
- [ ] Create ADR-0024 (Agent Mesh Python)
- [ ] Create ADR-0025 (Controller API v1)
- [ ] Commit changes (conventional commits)
- [ ] Update Phase-3-Agent-State.json (final state)
- [ ] Merge to main

---

**Prepared by:** Goose AI Agent  
**Date:** 2025-11-04  
**Status:** READY FOR EXECUTION (after Phase 2.5 complete)  
**Next Action:** User review & approval, then execute after Phase 2.5
