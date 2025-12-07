# Agent Mesh MCP Server

MCP extension for goose that enables multi-agent orchestration via the Controller API.

## Overview

The Agent Mesh MCP server provides 4 tools for cross-agent communication:

1. **send_task** - Route a task to another agent
2. **request_approval** - Request approval from a specific agent role
3. **notify** - Send a notification to another agent
4. **fetch_status** - Check the status of a routed task

## Requirements

- Python 3.13+
- Running Controller API (http://localhost:8088)
- Valid JWT token from Keycloak

## Installation

```bash
cd src/agent-mesh

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install package in development mode
pip install -e .

# Optional: Install development dependencies
pip install -e ".[dev]"
```

## Configuration

Create a `.env` file from the example:

```bash
cp .env.example .env
```

Edit `.env` and set:

```bash
CONTROLLER_URL=http://localhost:8088
MESH_JWT_TOKEN=eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...  # Obtain from Keycloak
```

### Obtaining JWT Token

```bash
# Get token from Keycloak (client_credentials grant)
TOKEN=$(curl -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "client_id=goose-controller" \
  -d "grant_type=client_credentials" \
  -d "client_secret=<your-secret>" | jq -r '.access_token')

echo "MESH_JWT_TOKEN=$TOKEN" >> .env
```

## goose Integration

Add to your goose profiles configuration (`~/.config/goose/profiles.yaml`):

```yaml
default:
  provider: openai
  processor: gpt-4
  accelerator: gpt-4o-mini
  moderator: passive
  
  extensions:
    # ... other extensions
    
    agent_mesh:
      type: mcp
      command: 
        - "python"
        - "-m"
        - "agent_mesh_server"
      working_dir: "/home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh"
      env:
        CONTROLLER_URL: "http://localhost:8088"
        MESH_JWT_TOKEN: "eyJ..."  # Your JWT token
        MESH_RETRY_COUNT: "3"
        MESH_TIMEOUT_SECS: "30"
```

**Alternative: Use .env file (recommended for security)**

```yaml
agent_mesh:
  type: mcp
  command: ["python", "-m", "agent_mesh_server"]
  working_dir: "/home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh"
  # Environment variables will be loaded from .env file in working_dir
```

## Usage

### Start goose with Agent Mesh

```bash
goose session start
```

The Agent Mesh tools will be available with the `agent_mesh__` prefix:

- `agent_mesh__send_task`
- `agent_mesh__request_approval`
- `agent_mesh__notify`
- `agent_mesh__fetch_status`

---

## Tool Reference

### 1. send_task

**Purpose:** Route a task to another agent via the Controller API.

**Parameters:**
- `target` (string, required): Target agent role (e.g., "manager", "finance", "engineering")
- `task` (object, required): Task payload as JSON object
- `context` (object, optional): Additional context metadata (default: `{}`)

**Features:**
- Automatic retry with exponential backoff (3 attempts by default)
- Idempotency key generation (UUID v4)
- Trace ID for request tracking
- Comprehensive error handling (4xx, 5xx, timeout, connection)

**Example:**

```
Use agent_mesh__send_task to route a budget approval task:
- target: "manager"
- task: {"type": "budget_approval", "amount": 50000, "purpose": "Q1 hiring"}
- context: {"department": "Engineering", "quarter": "Q1-2026"}
```

**Success Response:**

```
✅ Task routed successfully!

**Task ID:** task-abc123-xyz789
**Status:** routed
**Target:** manager
**Trace ID:** trace-def456

Use `fetch_status` with this Task ID to check progress.
```

**Error Handling:**
- 400 Bad Request → Invalid payload, missing Idempotency-Key
- 401 Unauthorized → Invalid/expired JWT token
- 413 Payload Too Large → Request exceeds 1MB limit
- 5xx Server Error → Automatic retry with backoff
- Timeout → Automatic retry
- Connection Error → Automatic retry

**Configuration:**
- `MESH_RETRY_COUNT` - Number of retry attempts (default: 3)
- `MESH_TIMEOUT_SECS` - Request timeout in seconds (default: 30)

---

### 2. request_approval

**Purpose:** Request approval for a task from a specific agent role.

**Parameters:**
- `task_id` (string, required): Task/session ID from send_task response
- `approver_role` (string, required): Role to request approval from (e.g., "manager", "director")
- `reason` (string, required): Human-readable explanation for the approval request
- `decision` (string, optional): Initial decision state (default: "pending")
- `comments` (string, optional): Additional context or notes (default: "")

**Features:**
- JWT authentication
- Idempotency key generation
- Trace ID for observability
- User-friendly error messages with troubleshooting steps

**Example:**

```
Use agent_mesh__request_approval to request manager approval:
- task_id: "task-abc123-xyz789"
- approver_role: "manager"
- reason: "Budget exceeds department threshold, requires manager approval"
- comments: "Urgent: needed for Q1 hiring cycle"
```

**Success Response:**

```
✅ Approval requested successfully!

**Approval ID:** approval-def456-ghi789
**Status:** pending
**Task ID:** task-abc123-xyz789
**Approver Role:** manager
**Trace ID:** trace-jkl012

The approval request has been routed to the manager role.
Use the Approval ID to check the status or retrieve the decision.
```

**Error Handling:**
- 400 Bad Request → Invalid task_id format, missing required fields
- 401 Unauthorized → Invalid/expired JWT token (includes refresh instructions)
- 404 Not Found → Task ID not found in system
- 413 Payload Too Large → Reason/comments too long (reduce length)
- Timeout → Controller API unresponsive (increase timeout)
- Connection Error → Controller API not running (check connectivity)

---

### 3. notify

**Purpose:** Send a notification to another agent via the Controller API.

**Parameters:**
- `target` (string, required): Target agent role (e.g., "manager", "finance", "engineering")
- `message` (string, required): Notification message content
- `priority` (string, optional): Priority level - "low", "normal", "high" (default: "normal")

**Features:**
- Priority validation (pre-request check)
- JWT authentication
- Idempotency key generation
- Trace ID for tracking
- Routes via POST /tasks/route with type='notification'

**Example:**

```
Use agent_mesh__notify to send a high-priority notification:
- target: "manager"
- message: "Q1 budget approval completed. Total approved: $150,000"
- priority: "high"
```

**Success Response:**

```
✅ Notification sent successfully!

**Task ID:** task-mno345-pqr678
**Status:** routed
**Target:** manager
**Priority:** high
**Trace ID:** trace-stu901

The notification has been routed to the manager role.
```

**Priority Levels:**
- `low` - Non-urgent notifications (FYI, status updates)
- `normal` - Standard notifications (default)
- `high` - Urgent notifications requiring immediate attention

**Error Handling:**
- Invalid Priority → User-friendly error with valid options
- 400 Bad Request → Invalid target or format
- 401 Unauthorized → Invalid/expired JWT token
- 413 Payload Too Large → Message exceeds 1MB limit
- Timeout → Controller API slow
- Connection Error → Controller API not running

---

### 4. fetch_status

**Purpose:** Retrieve the current status of a task/session from the Controller API.

**Parameters:**
- `task_id` (string, required): Task/session ID to query (from send_task or notify response)

**Features:**
- Read-only operation (GET request)
- JWT authentication
- Trace ID for tracking
- Formatted output for easy reading
- Returns full session data plus summary

**Example:**

```
Use agent_mesh__fetch_status to check task status:
- task_id: "task-abc123-xyz789"
```

**Success Response:**

```
✅ Status retrieved successfully:

- Task ID: task-abc123-xyz789
- Status: completed
- Assigned Agent: finance
- Created At: 2025-11-04T23:00:00Z
- Updated At: 2025-11-04T23:05:00Z
- Result: {"status": "approved", "amount": 50000}
- Trace ID: trace-vwx234

Full session data:
{
  "id": "task-abc123-xyz789",
  "status": "completed",
  "assigned_agent": "finance",
  "task_type": "budget_approval",
  "payload": {...},
  "created_at": "2025-11-04T23:00:00Z",
  "updated_at": "2025-11-04T23:05:00Z",
  "result": {"status": "approved", "amount": 50000}
}
```

**Status Values:**
- `pending` - Task created, not yet assigned
- `in_progress` - Task assigned and being worked on
- `completed` - Task finished successfully
- `failed` - Task failed (check result for error details)

**Error Handling:**
- 404 Not Found → Task ID not found (verify ID is correct)
- 401 Unauthorized → Invalid/expired JWT token
- 403 Forbidden → Insufficient permissions to view task
- Timeout → Controller API unresponsive
- Connection Error → Cannot connect to Controller API

---

## Workflow Examples

### Example 1: Cross-Agent Budget Approval

**Scenario:** Finance agent requests budget approval from Manager agent.

**Finance Agent (goose instance 1):**

```
1. Send budget approval task to manager:
   agent_mesh__send_task
   - target: "manager"
   - task: {"type": "budget_approval", "amount": 75000, "purpose": "Q2 hiring"}
   - context: {"department": "Engineering", "quarter": "Q2-2026"}

   Response: Task ID: task-001

2. Request approval:
   agent_mesh__request_approval
   - task_id: "task-001"
   - approver_role: "manager"
   - reason: "Exceeds department threshold of $50k"

   Response: Approval ID: approval-001

3. Check status periodically:
   agent_mesh__fetch_status
   - task_id: "task-001"

   Response: Status: pending_approval
```

**Manager Agent (goose instance 2):**

```
1. Receive notification (via notify or polling):
   agent_mesh__fetch_status
   - task_id: "task-001"

   Response: Task details with approval request

2. Review and approve/reject via Controller API
   (manual step or automated decision)
```

**Finance Agent (continue):**

```
4. Check final status:
   agent_mesh__fetch_status
   - task_id: "task-001"

   Response: Status: completed, Result: {"approved": true}
```

---

### Example 2: Notification Workflow

**Engineering Agent notifies Manager:**

```
agent_mesh__notify
- target: "manager"
- message: "Production deployment completed successfully. All services healthy."
- priority: "normal"

Response: Notification sent to manager role
```

**Manager Agent receives notification:**

```
agent_mesh__fetch_status
- task_id: "task-notification-123"

Response: Notification message and status
```

---

## Common Usage Patterns

### Pattern 1: Fire-and-Forget Task

```python
# Send task and don't wait for completion
send_task(target="analytics", task={"generate_report": "Q1-2026"})
```

### Pattern 2: Task with Status Polling

```python
# Send task and poll for completion
task_id = send_task(target="finance", task={"process_invoice": "INV-001"})

# Poll every 30 seconds
while True:
    status = fetch_status(task_id)
    if status["status"] in ["completed", "failed"]:
        break
    sleep(30)
```

### Pattern 3: Approval Workflow

```python
# Send task requiring approval
task_id = send_task(target="manager", task={"budget_request": 100000})

# Request approval
approval_id = request_approval(
    task_id=task_id,
    approver_role="director",
    reason="Budget exceeds manager authority"
)

# Wait for approval decision
status = fetch_status(task_id)
if status["result"]["approved"]:
    # Proceed with approved action
    pass
```

### Pattern 4: Broadcast Notification

```python
# Notify multiple roles about an event
for role in ["manager", "finance", "engineering"]:
    notify(
        target=role,
        message="System maintenance scheduled for 2025-11-05 02:00 UTC",
        priority="high"
    )
```

## Multi-Agent Testing (Phase 3)

**Phase 3 introduces shell scripts for testing multi-agent workflows with role-based configurations.**

### Setup

The repository provides 3 shell scripts for multi-agent testing:

1. **`scripts/get-jwt-token.sh`** - Get fresh JWT token from Keycloak
2. **`scripts/start-finance-agent.sh`** - Start Finance agent MCP server
3. **`scripts/start-manager-agent.sh`** - Start Manager agent MCP server

### Multi-Agent Workflow: Budget Approval

**Terminal 1: Start Controller API**

```bash
cd deploy/compose
docker compose -f ce.dev.yml up controller keycloak postgres vault privacy-guard
```

**Terminal 2: Start Finance Agent**

```bash
./scripts/start-finance-agent.sh
```

This will:
- Get a fresh JWT token from Keycloak
- Start Agent Mesh MCP server with ROLE=finance
- Display Finance agent instructions

**Terminal 3: Start Manager Agent**

```bash
./scripts/start-manager-agent.sh
```

This will:
- Get a fresh JWT token from Keycloak  
- Start Agent Mesh MCP server with ROLE=manager
- Display Manager agent instructions

### Finance → Manager Approval Workflow

**Step 1: Finance Agent sends budget request (Terminal 2 / goose Desktop)**

Open goose Desktop or goose CLI and connect to the Finance agent MCP server (running in Terminal 2):

```
Use agent_mesh__send_task to send a budget approval request:
- target: "manager"
- task: {"task_type": "budget_approval", "description": "Q1 hiring budget", "data": {"amount": 50000, "department": "Engineering"}}
- context: {"quarter": "Q1-2026", "submitted_by": "finance"}
```

**Expected Output:**
```
✅ Task routed successfully!
**Task ID:** task-abc123-xyz789
**Status:** accepted
**Target:** manager
```

Copy the Task ID for the next steps.

**Step 2: Manager Agent checks for pending tasks (Terminal 3 / goose Desktop)**

Open another goose Desktop/CLI instance and connect to the Manager agent MCP server (running in Terminal 3):

```
Use agent_mesh__fetch_status to check the task:
- task_id: "task-abc123-xyz789"
```

**Expected Output (Phase 3):**
```
❌ HTTP 501 Not Implemented
This endpoint requires session persistence (deferred to Phase 4).
```

**Note:** This is expected in Phase 3 - the Controller API doesn't persist sessions yet. 

**Workaround:** Manager approves via direct curl request (simulating approval):

```bash
# Get JWT token
TOKEN=$(./scripts/get-jwt-token.sh 2>/dev/null)

# Submit approval
curl -X POST http://localhost:8088/approvals \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -H "X-Trace-Id: $(uuidgen)" \
  -d '{"task_id":"task-abc123-xyz789","decision":"approved","comments":"Budget approved for Q1 hiring"}'
```

**Expected Output:**
```json
{"approval_id":"approval-def456-ghi789","status":"approved"}
```

**Step 3: Finance Agent sends thank-you notification (Terminal 2 / goose Desktop)**

Back in Finance agent:

```
Use agent_mesh__notify to send a notification:
- target: "manager"
- message: "Thank you for approving the Q1 hiring budget!"
- priority: "normal"
```

**Expected Output:**
```
✅ Notification sent successfully!
**Task ID:** task-mno345-pqr678
**Status:** accepted
**Target:** manager
```

### Verifying Audit Trail

Check Controller API logs for the complete audit trail:

```bash
docker logs ce_controller | grep -E "task-abc123|approval-def456|task-mno345"
```

**Expected Log Entries:**
- POST /tasks/route - Finance sends budget request
- POST /approvals - Manager approves budget
- POST /tasks/route - Finance sends thank-you notification

### Known Limitations (Phase 3)

| Issue | Status | Workaround | Phase 4 Fix |
|-------|--------|-----------|-------------|
| fetch_status returns 501 | ⏸️ Expected | Use curl directly to check Controller logs | Add Postgres session storage (6h) |
| JWT tokens expire in 60 min | ⏸️ Acceptable | Run `./scripts/get-jwt-token.sh` to refresh | Automated refresh (2h) |
| No session persistence | ⏸️ By design | Verify via Controller API logs | Add session storage (6h) |

### Troubleshooting Multi-Agent Setup

**Problem: "Keycloak is not running"**

```bash
# Start Keycloak
cd deploy/compose
docker compose -f ce.dev.yml up keycloak -d

# Wait for healthy status
docker compose -f ce.dev.yml ps keycloak
```

**Problem: "Failed to get JWT token"**

```bash
# Check Keycloak dev realm exists
curl http://localhost:8080/realms/dev | jq -r '.realm'
# Should output: dev

# Check client secret is set in .env.ce
grep OIDC_CLIENT_SECRET deploy/compose/.env.ce
```

**Problem: "Controller API not running"**

```bash
# Start Controller API
cd deploy/compose
docker compose -f ce.dev.yml up controller -d

# Verify health
curl http://localhost:8088/status
# Should return: {"status":"ok","version":"0.1.0"}
```

**Problem: "MCP server won't start"**

```bash
# Check Python virtual environment
cd src/agent-mesh
ls -la .venv/  # Should exist

# If missing, create it
python3 -m venv .venv
source .venv/bin/activate
pip install -e .
```

---

## Testing

### Manual Testing (Standalone)

```bash
# Test server starts
python agent_mesh_server.py
# Should output: "Agent Mesh MCP Server starting..."
# Press Ctrl+C to stop
```

### Integration Tests (with pytest)

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest tests/
```

## Development

### Project Structure

```
src/agent-mesh/
├── pyproject.toml           # Python package configuration
├── agent_mesh_server.py     # MCP server entry point
├── .env.example             # Environment variable template
├── README.md                # This file
├── tools/
│   ├── __init__.py          # Tools package
│   ├── send_task.py         # send_task tool implementation
│   ├── request_approval.py  # request_approval tool implementation
│   ├── notify.py            # notify tool implementation
│   └── fetch_status.py      # fetch_status tool implementation
└── tests/
    ├── __init__.py
    └── test_integration.py  # Integration tests
```

### Adding a New Tool

1. Create `tools/my_tool.py`:

```python
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field

class MyToolParams(BaseModel):
    param1: str = Field(description="Description")

async def my_tool_handler(params: MyToolParams) -> list[TextContent]:
    # Implementation
    return [TextContent(type="text", text="Result")]

my_tool = Tool(
    name="my_tool",
    description="Tool description",
    inputSchema=MyToolParams.model_json_schema(),
)
my_tool.call = my_tool_handler
```

2. Register in `agent_mesh_server.py`:

```python
from tools.my_tool import my_tool

async def main():
    server = Server("agent-mesh")
    server.add_tool(my_tool)
    # ...
```

## Troubleshooting

### "MESH_JWT_TOKEN not set" error

Make sure you have created a `.env` file with a valid JWT token, or configured it in your goose profiles.yaml.

### Connection refused to Controller API

1. Check Controller API is running: `curl http://localhost:8088/status`
2. Verify `CONTROLLER_URL` in `.env` is correct

### Tools not visible in goose

1. Check profiles.yaml configuration
2. Verify `working_dir` path is correct
3. Check goose logs: `goose logs`

### JWT token expired

JWT tokens from Keycloak typically expire after 5 minutes. For development:

1. Use a service account token with longer expiry
2. Or re-generate token before each session

## Architecture

```
┌─────────────────┐
│  goose Agent 1  │
│   (Finance)     │
└────────┬────────┘
         │ MCP stdio
         ├─ agent_mesh__send_task
         ├─ agent_mesh__request_approval
         ├─ agent_mesh__notify
         └─ agent_mesh__fetch_status
         │
         v
┌─────────────────────┐      HTTP/REST      ┌──────────────────┐
│ Agent Mesh MCP      ├────────────────────>│  Controller API  │
│ (Python Server)     │                      │  (Rust/Axum)     │
└─────────────────────┘                      └────────┬─────────┘
                                                      │
                                                      v
                                              ┌───────────────┐
                                              │   Keycloak    │
                                              │ (JWT/AuthN)   │
                                              └───────────────┘
                                                      │
                                                      v
                                              ┌───────────────┐
                                              │ Privacy Guard │
                                              │  (Masking)    │
                                              └───────────────┘
```

## Version History

- **0.1.0** (2025-11-04) - Initial scaffold with 4 tools (Phase 3)

## License

MIT (aligned with goose project)

## Support

For issues or questions:
1. Check this README
2. Review Phase 3 documentation in `Technical Project Plan/PM Phases/Phase-3/`
3. Consult ADR-0024: Agent Mesh Python Implementation
