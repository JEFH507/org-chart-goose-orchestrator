"""send_task MCP Tool

Route a task to another agent via the Controller API.

Implements:
- Retry logic with exponential backoff + jitter
- Idempotency key generation
- JWT authentication
- Error handling
"""

import os
import time
import random
import uuid
from typing import Any

from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field
import requests


class SendTaskParams(BaseModel):
    """Parameters for the send_task tool."""
    
    target: str = Field(
        description="Target agent role (e.g., 'manager', 'finance', 'engineering')"
    )
    task: dict[str, Any] = Field(
        description="Task payload as JSON object (e.g., {'type': 'budget_approval', 'amount': 50000})"
    )
    context: dict[str, Any] = Field(
        default_factory=dict,
        description="Additional context (optional, e.g., {'department': 'Engineering', 'quarter': 'Q1'})"
    )


async def send_task_handler(params: SendTaskParams) -> list[TextContent]:
    """
    Route a task to another agent via the Controller API.
    
    Features:
    - Automatic retry with exponential backoff + jitter (3 attempts by default)
    - Idempotency key generation for safe retries
    - Trace ID generation for observability
    - JWT authentication via Bearer token
    
    Environment Variables:
    - CONTROLLER_URL: Controller API base URL (default: http://localhost:8088)
    - MESH_JWT_TOKEN: JWT token for authentication (required)
    - MESH_RETRY_COUNT: Number of retry attempts (default: 3)
    - MESH_TIMEOUT_SECS: Request timeout in seconds (default: 30)
    
    Args:
        params: SendTaskParams with target, task, and optional context
        
    Returns:
        list[TextContent]: Success message with task ID or error message
    """
    # Get configuration from environment
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    max_retries = int(os.getenv("MESH_RETRY_COUNT", "3"))
    timeout = int(os.getenv("MESH_TIMEOUT_SECS", "30"))
    
    # Validate JWT token
    if not jwt_token:
        return [TextContent(
            type="text",
            text="❌ ERROR: MESH_JWT_TOKEN environment variable not set.\n\n"
                 "Please configure your JWT token:\n"
                 "1. Obtain token from Keycloak (client_credentials grant)\n"
                 "2. Set MESH_JWT_TOKEN in .env or Goose profiles.yaml"
        )]
    
    # Generate idempotency key (same for all retry attempts)
    idempotency_key = str(uuid.uuid4())
    
    # Generate trace ID for request tracking
    trace_id = str(uuid.uuid4())
    
    # Retry loop with exponential backoff
    last_error = None
    for attempt in range(max_retries):
        try:
            # Make HTTP POST request to Controller API
            response = requests.post(
                f"{controller_url}/tasks/route",
                headers={
                    "Authorization": f"Bearer {jwt_token}",
                    "Content-Type": "application/json",
                    "Idempotency-Key": idempotency_key,
                    "X-Trace-Id": trace_id,
                },
                json={
                    "target": params.target,
                    "task": params.task,
                    "context": params.context,
                },
                timeout=timeout,
            )
            
            # Raise exception for HTTP errors (4xx, 5xx)
            response.raise_for_status()
            
            # Parse JSON response
            data = response.json()
            
            # Success - return task details
            return [TextContent(
                type="text",
                text=f"✅ Task routed successfully!\n\n"
                     f"**Task ID:** {data.get('task_id', 'N/A')}\n"
                     f"**Status:** {data.get('status', 'unknown')}\n"
                     f"**Target:** {params.target}\n"
                     f"**Trace ID:** {trace_id}\n\n"
                     f"Use `fetch_status` with this Task ID to check progress."
            )]
        
        except requests.exceptions.HTTPError as e:
            # HTTP error (4xx, 5xx) - don't retry on client errors
            status_code = e.response.status_code if e.response else None
            
            if status_code and 400 <= status_code < 500:
                # Client error (4xx) - don't retry
                error_detail = e.response.text if e.response else str(e)
                return [TextContent(
                    type="text",
                    text=f"❌ HTTP {status_code} Client Error\n\n"
                         f"The request was rejected by the Controller API:\n"
                         f"{error_detail}\n\n"
                         f"**Possible causes:**\n"
                         f"- Invalid JWT token (401)\n"
                         f"- Missing or invalid Idempotency-Key (400)\n"
                         f"- Invalid request payload (400)\n"
                         f"- Request too large >1MB (413)\n\n"
                         f"**Trace ID:** {trace_id}"
                )]
            
            # Server error (5xx) - retry
            last_error = e
            
        except requests.exceptions.Timeout as e:
            # Timeout error - retry
            last_error = e
            
        except requests.exceptions.ConnectionError as e:
            # Connection error - retry
            last_error = e
            
        except requests.exceptions.RequestException as e:
            # Other request errors - retry
            last_error = e
        
        except Exception as e:
            # Unexpected error - don't retry
            return [TextContent(
                type="text",
                text=f"❌ Unexpected error: {type(e).__name__}\n\n"
                     f"{str(e)}\n\n"
                     f"**Trace ID:** {trace_id}"
            )]
        
        # If we reach here, retry is needed
        if attempt < max_retries - 1:
            # Calculate wait time: 2^attempt + random jitter (0-1 seconds)
            wait_time = (2 ** attempt) + random.uniform(0, 1)
            
            # Log retry attempt (visible in Goose)
            print(
                f"⚠️  Attempt {attempt + 1}/{max_retries} failed. "
                f"Retrying in {wait_time:.1f}s... "
                f"(Error: {type(last_error).__name__})",
                flush=True
            )
            
            time.sleep(wait_time)
    
    # All retries exhausted
    return [TextContent(
        type="text",
        text=f"❌ Failed to route task after {max_retries} attempts\n\n"
             f"**Last Error:** {type(last_error).__name__}\n"
             f"**Details:** {str(last_error)}\n\n"
             f"**Troubleshooting:**\n"
             f"1. Check Controller API is running: `curl {controller_url}/status`\n"
             f"2. Verify CONTROLLER_URL is correct in .env\n"
             f"3. Check JWT token is valid and not expired\n"
             f"4. Check network connectivity to Controller\n\n"
             f"**Trace ID:** {trace_id}\n"
             f"**Idempotency Key:** {idempotency_key}"
    )]


# MCP Tool definition
send_task_tool = Tool(
    name="send_task",
    description=(
        "Route a task to another agent via the Controller API. "
        "Supports automatic retry with exponential backoff for resilience. "
        "Returns task ID for status tracking."
    ),
    inputSchema=SendTaskParams.model_json_schema(),
)

# Attach handler to tool
send_task_tool.call = send_task_handler
