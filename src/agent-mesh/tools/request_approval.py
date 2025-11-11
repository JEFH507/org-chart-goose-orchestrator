"""
request_approval tool - Request approval for a task from a specific agent role.

This tool submits an approval request to the Controller API's POST /approvals endpoint.
The approval is routed to the specified approver role (e.g., 'manager', 'director').

Features:
- JWT authentication via MESH_JWT_TOKEN environment variable
- Idempotency key generation (UUID v4)
- Trace ID for request tracking
- User-friendly error messages with troubleshooting steps
"""

import os
import uuid
from typing import Any

import requests
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field


class RequestApprovalParams(BaseModel):
    """Parameters for the request_approval tool."""
    
    task_id: str = Field(
        description="Task ID requiring approval (UUID format from send_task response)"
    )
    approver_role: str = Field(
        description="Role of the approver (e.g., 'manager', 'director', 'finance')"
    )
    reason: str = Field(
        description="Reason for the approval request (human-readable explanation)"
    )
    decision: str = Field(
        default="pending",
        description="Decision status (default: 'pending', can be 'approved' or 'rejected' if submitting on behalf)"
    )
    comments: str = Field(
        default="",
        description="Optional comments providing additional context for the approval"
    )


async def request_approval_handler(params: RequestApprovalParams) -> list[TextContent]:
    """
    Request approval for a task from a specific agent role.
    
    This function submits an approval request to the Controller API, which routes
    the request to the appropriate approver based on their role.
    
    Args:
        params: RequestApprovalParams containing task_id, approver_role, reason, etc.
    
    Returns:
        list[TextContent]: Success message with approval ID or error message
    
    Environment Variables Required:
        CONTROLLER_URL: Base URL of the Controller API (default: http://localhost:8088)
        MESH_JWT_TOKEN: JWT token for authentication (required)
        MESH_TIMEOUT_SECS: Request timeout in seconds (default: 30)
    """
    # Get configuration from environment
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    timeout = int(os.getenv("MESH_TIMEOUT_SECS", "30"))
    
    # Validate JWT token
    if not jwt_token:
        return [TextContent(
            type="text",
            text="❌ ERROR: MESH_JWT_TOKEN environment variable not set\n\n"
                 "**Action Required:**\n"
                 "1. Obtain JWT token from Keycloak (POST /realms/dev/protocol/openid-connect/token)\n"
                 "2. Set MESH_JWT_TOKEN environment variable\n"
                 "3. Retry the approval request\n\n"
                 "Example:\n"
                 "```bash\n"
                 "export MESH_JWT_TOKEN='eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...'\n"
                 "```"
        )]
    
    # Generate idempotency key and trace ID
    idempotency_key = str(uuid.uuid4())
    trace_id = str(uuid.uuid4())
    
    # Prepare request - route approval request as a task
    url = f"{controller_url}/tasks/route"
    headers = {
        "Authorization": f"Bearer {jwt_token}",
        "Content-Type": "application/json",
        "idempotency-key": idempotency_key,  # lowercase per Axum requirements
        "X-Trace-Id": trace_id,
    }
    
    # Construct task payload matching Controller API format
    task_payload = {
        "task_type": "approval_request",
        "description": params.reason,
        "data": {
            "original_task_id": params.task_id,
            "decision": params.decision,
            "comments": params.comments,
        }
    }
    
    payload = {
        "target": params.approver_role,
        "task": task_payload,
        "context": {"request_type": "approval"},
    }
    
    try:
        # Send approval request
        response = requests.post(
            url,
            headers=headers,
            json=payload,
            timeout=timeout,
        )
        
        # Check for HTTP errors
        response.raise_for_status()
        
        # Parse response
        data = response.json()
        
        # Success response
        return [TextContent(
            type="text",
            text=f"✅ Approval requested successfully!\n\n"
                 f"**Task ID (Approval Request):** {data.get('task_id', 'N/A')}\n"
                 f"**Status:** {data.get('status', 'accepted')}\n"
                 f"**Original Task:** {params.task_id}\n"
                 f"**Approver Role:** {params.approver_role}\n"
                 f"**Trace ID:** {trace_id}\n\n"
                 f"The approval request has been routed to the {params.approver_role} role.\n"
                 f"Use fetch_status with the Task ID to check the approval decision."
        )]
    
    except requests.exceptions.HTTPError as e:
        # HTTP error response
        status_code = e.response.status_code
        
        # Try to get error details from response
        try:
            error_detail = e.response.json().get("detail", str(e))
        except Exception:
            error_detail = str(e)
        
        # User-friendly error message based on status code
        if status_code == 400:
            error_msg = (
                f"❌ HTTP 400 Bad Request\n\n"
                f"The approval request was rejected:\n{error_detail}\n\n"
                f"**Possible causes:**\n"
                f"- Invalid task_id format (must be UUID)\n"
                f"- Missing required fields (task_id, approver_role, reason)\n"
                f"- Invalid decision value (must be 'pending', 'approved', or 'rejected')\n"
                f"- Malformed JSON payload\n\n"
                f"**Trace ID:** {trace_id}"
            )
        elif status_code == 401:
            error_msg = (
                f"❌ HTTP 401 Unauthorized\n\n"
                f"Authentication failed.\n\n"
                f"**Possible causes:**\n"
                f"- Invalid or expired JWT token\n"
                f"- Token signature verification failed\n"
                f"- Missing Authorization header\n\n"
                f"**Action Required:**\n"
                f"1. Obtain a fresh JWT token from Keycloak\n"
                f"2. Update MESH_JWT_TOKEN environment variable\n"
                f"3. Retry the approval request\n\n"
                f"**Trace ID:** {trace_id}"
            )
        elif status_code == 404:
            error_msg = (
                f"❌ HTTP 404 Not Found\n\n"
                f"The task_id '{params.task_id}' was not found.\n\n"
                f"**Possible causes:**\n"
                f"- Task ID does not exist\n"
                f"- Task was deleted or expired\n"
                f"- Typo in task_id\n\n"
                f"**Action Required:**\n"
                f"1. Verify the task_id from the send_task response\n"
                f"2. Check if the task still exists (use fetch_status tool)\n\n"
                f"**Trace ID:** {trace_id}"
            )
        elif status_code == 413:
            error_msg = (
                f"❌ HTTP 413 Payload Too Large\n\n"
                f"The approval request payload exceeds the 1MB limit.\n\n"
                f"**Action Required:**\n"
                f"1. Reduce the length of the reason or comments fields\n"
                f"2. Keep approval requests concise\n\n"
                f"**Trace ID:** {trace_id}"
            )
        else:
            error_msg = (
                f"❌ HTTP {status_code} Error\n\n"
                f"The Controller API returned an error:\n{error_detail}\n\n"
                f"**Trace ID:** {trace_id}\n\n"
                f"Check the Controller API logs for more details."
            )
        
        return [TextContent(type="text", text=error_msg)]
    
    except requests.exceptions.Timeout:
        # Request timeout
        return [TextContent(
            type="text",
            text=f"❌ Request Timeout\n\n"
                 f"The approval request to {url} timed out after {timeout} seconds.\n\n"
                 f"**Possible causes:**\n"
                 f"- Controller API is slow or unresponsive\n"
                 f"- Network latency issues\n"
                 f"- Privacy Guard masking taking too long\n\n"
                 f"**Action Required:**\n"
                 f"1. Check if the Controller API is running (curl {controller_url}/status)\n"
                 f"2. Increase MESH_TIMEOUT_SECS if needed\n"
                 f"3. Retry the approval request\n\n"
                 f"**Trace ID:** {trace_id}"
        )]
    
    except requests.exceptions.ConnectionError:
        # Connection error
        return [TextContent(
            type="text",
            text=f"❌ Connection Error\n\n"
                 f"Could not connect to the Controller API at {controller_url}\n\n"
                 f"**Possible causes:**\n"
                 f"- Controller API is not running\n"
                 f"- Incorrect CONTROLLER_URL\n"
                 f"- Network connectivity issues\n\n"
                 f"**Action Required:**\n"
                 f"1. Verify Controller API is running:\n"
                 f"   ```bash\n"
                 f"   docker compose -f deploy/compose/ce.dev.yml ps controller\n"
                 f"   ```\n"
                 f"2. Check CONTROLLER_URL environment variable\n"
                 f"3. Test connectivity: curl {controller_url}/status\n\n"
                 f"**Trace ID:** {trace_id}"
        )]
    
    except requests.exceptions.RequestException as e:
        # Other request errors
        return [TextContent(
            type="text",
            text=f"❌ Request Error\n\n"
                 f"An unexpected error occurred while requesting approval:\n{str(e)}\n\n"
                 f"**Trace ID:** {trace_id}\n\n"
                 f"Please check the Controller API logs for more details."
        )]


# Tool definition for MCP server registration
request_approval_tool = Tool(
    name="request_approval",
    description=(
        "Request approval for a task from a specific agent role via the Controller API. "
        "Submit an approval request that will be routed to the appropriate approver "
        "(e.g., 'manager', 'director', 'finance'). Returns an approval ID that can be "
        "used to track the approval status. Requires JWT authentication via MESH_JWT_TOKEN."
    ),
    inputSchema=RequestApprovalParams.model_json_schema(),
)

# Attach handler to tool
request_approval_tool.call = request_approval_handler
