"""
fetch_status tool for Agent Mesh MCP server.

Retrieves the current status of a task/session from the Controller API.
"""

import os
import uuid
from typing import Any
import requests
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field


class FetchStatusParams(BaseModel):
    """Parameters for the fetch_status tool."""
    task_id: str = Field(description="The task/session ID to query for status")


async def fetch_status_handler(params: FetchStatusParams) -> list[TextContent]:
    """
    Handler for fetch_status tool.
    
    Calls GET /sessions/{task_id} on the Controller API to retrieve
    the current status, assigned agent, and completion details.
    
    Args:
        params: FetchStatusParams containing task_id
        
    Returns:
        List containing a TextContent with status information or error message
        
    Raises:
        No exceptions - all errors are caught and returned as TextContent
    """
    # Get configuration from environment
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:3000")
    jwt_token = os.getenv("MESH_JWT_TOKEN", "")
    timeout_secs = int(os.getenv("MESH_TIMEOUT_SECS", "30"))
    
    # Validate JWT token is present
    if not jwt_token:
        return [
            TextContent(
                type="text",
                text="Error: MESH_JWT_TOKEN environment variable is not set. "
                     "Cannot authenticate with Controller API."
            )
        ]
    
    # Validate task_id is not empty
    if not params.task_id or not params.task_id.strip():
        return [
            TextContent(
                type="text",
                text="Error: task_id parameter is required and cannot be empty."
            )
        ]
    
    # Generate trace ID for observability
    trace_id = str(uuid.uuid4())
    
    # Prepare request
    url = f"{controller_url}/tasks/{params.task_id}"
    headers = {
        "Authorization": f"Bearer {jwt_token}",
        "X-Trace-ID": trace_id,
        "Content-Type": "application/json"
    }
    
    try:
        # Make GET request to Controller API
        response = requests.get(
            url,
            headers=headers,
            timeout=timeout_secs
        )
        
        # Handle different HTTP status codes
        if response.status_code == 200:
            # Success - parse and return session data
            session_data = response.json()
            
            # Extract fields from SessionResponse
            # API contract (Phase 4 - database-backed):
            # {
            #   "session_id": "uuid",
            #   "agent_role": "finance",
            #   "state": "active",  // pending/active/completed/failed/expired
            #   "metadata": {...}   // optional JSON
            # }
            session_id = session_data.get('session_id', 'unknown')
            agent_role = session_data.get('agent_role', 'unknown')
            state = session_data.get('state', 'unknown')
            metadata = session_data.get('metadata', {})
            
            # Format metadata for display (limit size)
            metadata_str = str(metadata) if metadata else '{}'
            if len(metadata_str) > 200:
                metadata_str = metadata_str[:200] + '...'
            
            # Format the response for readability
            status_text = f"""✅ Status retrieved successfully:
- Session ID: {session_id}
- Agent Role: {agent_role}
- Current State: {state}
- Metadata: {metadata_str}
- Trace ID: {trace_id}

State meanings:
- pending: Session created, not yet active
- active: Session in progress
- completed: Session finished successfully
- failed: Session encountered an error
- expired: Session exceeded retention period

Use this information to track task progress and coordinate with other agents."""
            
            return [TextContent(type="text", text=status_text)]
            
        elif response.status_code == 404:
            # Task/session not found
            return [
                TextContent(
                    type="text",
                    text=f"Error: Task/session '{params.task_id}' not found in Controller API. "
                         f"Please verify the task ID is correct.\n"
                         f"Trace ID: {trace_id}"
                )
            ]
            
        elif response.status_code == 401:
            # Authentication failed
            return [
                TextContent(
                    type="text",
                    text=f"Error: Authentication failed (401). "
                         f"The JWT token may be invalid or expired. "
                         f"Please check MESH_JWT_TOKEN configuration.\n"
                         f"Trace ID: {trace_id}"
                )
            ]
            
        elif response.status_code == 403:
            # Forbidden - insufficient permissions
            return [
                TextContent(
                    type="text",
                    text=f"Error: Access forbidden (403). "
                         f"The authenticated agent does not have permission to view this task.\n"
                         f"Trace ID: {trace_id}"
                )
            ]
            
        else:
            # Other HTTP errors
            error_detail = ""
            try:
                error_data = response.json()
                error_detail = error_data.get("error", str(error_data))
            except Exception:
                error_detail = response.text or "No error details provided"
            
            return [
                TextContent(
                    type="text",
                    text=f"Error: Controller API returned status {response.status_code}\n"
                         f"Details: {error_detail}\n"
                         f"Trace ID: {trace_id}"
                )
            ]
    
    except requests.exceptions.Timeout:
        return [
            TextContent(
                type="text",
                text=f"Error: Request timed out after {timeout_secs} seconds. "
                     f"The Controller API at {controller_url} may be unresponsive. "
                     f"Consider increasing MESH_TIMEOUT_SECS or checking API availability.\n"
                     f"Trace ID: {trace_id}"
            )
        ]
    
    except requests.exceptions.ConnectionError as e:
        return [
            TextContent(
                type="text",
                text=f"Error: Could not connect to Controller API at {controller_url}. "
                     f"Please verify CONTROLLER_URL is correct and the API is running.\n"
                     f"Connection error: {str(e)}\n"
                     f"Trace ID: {trace_id}"
            )
        ]
    
    except Exception as e:
        return [
            TextContent(
                type="text",
                text=f"Error: Unexpected error occurred: {str(e)}\n"
                     f"Trace ID: {trace_id}"
            )
        ]


# Define the tool
fetch_status_tool = Tool(
    name="fetch_status",
    description=(
        "Retrieve the current status of a session from the Controller API (database-backed). "
        "Returns real-time session data including state, agent role, and metadata. "
        "Use this to track task progress and coordinate with other agents.\n\n"
        "Parameters:\n"
        "- task_id: The unique session ID to query (UUID format, required)\n\n"
        "Returns status information including:\n"
        "- Session ID: Unique identifier\n"
        "- Agent Role: Agent responsible for this session (e.g., 'finance', 'manager')\n"
        "- Current State: One of pending/active/completed/failed/expired\n"
        "- Metadata: Custom session context (JSON object)\n\n"
        "State lifecycle:\n"
        "- pending: Session created, not yet started\n"
        "- active: Session in progress\n"
        "- completed: Session finished successfully\n"
        "- failed: Session encountered an error\n"
        "- expired: Session exceeded retention period (7 days default)\n\n"
        "Example: Checking status of session '550e8400-e29b-41d4-a716-446655440000'"
    ),
    inputSchema=FetchStatusParams.model_json_schema()
)

# Attach the handler to the tool
fetch_status_tool.call = fetch_status_handler
