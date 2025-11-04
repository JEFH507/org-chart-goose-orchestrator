# Agent Mesh MCP Server

MCP extension for Goose that enables multi-agent orchestration via the Controller API.

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

## Goose Integration

Add to your Goose profiles configuration (`~/.config/goose/profiles.yaml`):

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

### Start Goose with Agent Mesh

```bash
goose session start
```

The Agent Mesh tools will be available with the `agent_mesh__` prefix:

- `agent_mesh__send_task`
- `agent_mesh__request_approval`
- `agent_mesh__notify`
- `agent_mesh__fetch_status`

### Example: Send a Task

In a Goose session:

```
Use agent_mesh__send_task to route a budget approval task to the manager:
- target: "manager"
- task: {"type": "budget_approval", "amount": 50000, "purpose": "Q1 hiring"}
- context: {"department": "Engineering", "quarter": "Q1-2026"}
```

Goose will call the tool and return:

```
Task routed successfully. Task ID: task-abc123..., Status: routed
```

### Example: Check Task Status

```
Use agent_mesh__fetch_status to check the task:
- task_id: "task-abc123..."
```

Response:

```
Task Status: pending_approval
Progress: awaiting_manager_review
Result: N/A
```

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

Make sure you have created a `.env` file with a valid JWT token, or configured it in your Goose profiles.yaml.

### Connection refused to Controller API

1. Check Controller API is running: `curl http://localhost:8088/status`
2. Verify `CONTROLLER_URL` in `.env` is correct

### Tools not visible in Goose

1. Check profiles.yaml configuration
2. Verify `working_dir` path is correct
3. Check Goose logs: `goose logs`

### JWT token expired

JWT tokens from Keycloak typically expire after 5 minutes. For development:

1. Use a service account token with longer expiry
2. Or re-generate token before each session

## Architecture

```
┌─────────────────┐
│  Goose Agent 1  │
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

MIT (aligned with Goose project)

## Support

For issues or questions:
1. Check this README
2. Review Phase 3 documentation in `Technical Project Plan/PM Phases/Phase-3/`
3. Consult ADR-0024: Agent Mesh Python Implementation
