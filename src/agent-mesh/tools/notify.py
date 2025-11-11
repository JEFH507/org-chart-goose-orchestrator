"""
notify tool for Agent Mesh MCP Server.

Sends a notification to another agent via the Controller API.
Uses the POST /tasks/route endpoint with a notification task type.
"""

import os
import uuid
from typing import Any

import requests
from mcp.types import Tool, TextContent
from pydantic import BaseModel, Field


class NotifyParams(BaseModel):
    """Parameters for the notify tool."""

    target: str = Field(
        description="Target agent role (e.g., 'manager', 'finance', 'engineering')"
    )
    message: str = Field(description="Notification message content")
    priority: str = Field(
        default="normal",
        description="Notification priority: 'low', 'normal', or 'high'",
    )


async def notify_handler(params: NotifyParams) -> list[TextContent]:
    """
    Send a notification to another agent via the Controller API.

    This tool uses the POST /tasks/route endpoint with a notification task type.
    The notification is delivered as a task with type='notification'.

    Args:
        params: NotifyParams containing target, message, and priority

    Returns:
        List containing a TextContent with success or error message

    Environment Variables:
        CONTROLLER_URL: Controller API base URL (default: http://localhost:8088)
        MESH_JWT_TOKEN: JWT token for authentication (required)
        MESH_TIMEOUT_SECS: Request timeout in seconds (default: 30)

    Error Handling:
        - 400 Bad Request: Invalid notification format or missing fields
        - 401 Unauthorized: Invalid or expired JWT token
        - 413 Payload Too Large: Message exceeds 1MB limit
        - Timeout: Controller API slow or unresponsive
        - Connection: Controller API not running or network issues
    """
    # Get configuration from environment
    controller_url = os.getenv("CONTROLLER_URL", "http://localhost:8088")
    jwt_token = os.getenv("MESH_JWT_TOKEN")
    timeout = int(os.getenv("MESH_TIMEOUT_SECS", "30"))

    # Validate JWT token
    if not jwt_token:
        return [
            TextContent(
                type="text",
                text="❌ Configuration Error\n\n"
                "MESH_JWT_TOKEN environment variable is not set.\n\n"
                "**Action Required:**\n"
                "1. Obtain a JWT token from Keycloak\n"
                "2. Set MESH_JWT_TOKEN in your .env file or environment\n"
                "3. Retry the notification",
            )
        ]

    # Validate priority
    valid_priorities = ["low", "normal", "high"]
    if params.priority not in valid_priorities:
        return [
            TextContent(
                type="text",
                text=f"❌ Invalid Priority\n\n"
                f"Priority must be one of: {', '.join(valid_priorities)}\n"
                f"You provided: '{params.priority}'\n\n"
                f"**Valid priorities:**\n"
                f"- 'low': Non-urgent notifications\n"
                f"- 'normal': Standard notifications (default)\n"
                f"- 'high': Urgent notifications requiring immediate attention",
            )
        ]

    # Generate unique IDs
    idempotency_key = str(uuid.uuid4())
    trace_id = str(uuid.uuid4())

    # Prepare notification task (matches Controller API TaskPayload schema)
    notification_task = {
        "task_type": "notification",
        "description": params.message,
        "data": {"priority": params.priority},
    }

    # Prepare request headers
    headers = {
        "Authorization": f"Bearer {jwt_token}",
        "Content-Type": "application/json",
        "idempotency-key": idempotency_key,  # lowercase per Axum requirements
        "X-Trace-Id": trace_id,
    }

    # Prepare request payload
    payload = {"target": params.target, "task": notification_task, "context": {}}

    # Send notification via POST /tasks/route
    try:
        response = requests.post(
            f"{controller_url}/tasks/route",
            headers=headers,
            json=payload,
            timeout=timeout,
        )

        # Handle HTTP errors
        if not response.ok:
            status_code = response.status_code

            # Client errors (4xx) - don't retry
            if 400 <= status_code < 500:
                if status_code == 400:
                    error_detail = (
                        "The notification request was rejected by the Controller API.\n\n"
                        "**Possible causes:**\n"
                        "- Invalid target role\n"
                        "- Missing or invalid Idempotency-Key header\n"
                        "- Malformed JSON payload\n\n"
                        "**Action Required:**\n"
                        "1. Verify the target role exists\n"
                        "2. Check notification message format\n"
                        "3. Retry with valid parameters"
                    )
                elif status_code == 401:
                    error_detail = (
                        "Authentication failed.\n\n"
                        "**Possible causes:**\n"
                        "- Invalid or expired JWT token\n"
                        "- Token signature verification failed\n"
                        "- Missing Authorization header\n\n"
                        "**Action Required:**\n"
                        "1. Obtain a fresh JWT token from Keycloak\n"
                        "2. Update MESH_JWT_TOKEN environment variable\n"
                        "3. Retry the notification"
                    )
                elif status_code == 413:
                    error_detail = (
                        "Notification message too large (>1MB).\n\n"
                        "**Action Required:**\n"
                        "1. Reduce the message length\n"
                        "2. Split into multiple notifications if needed\n"
                        "3. Retry with shorter message"
                    )
                else:
                    error_detail = (
                        f"HTTP {status_code} Client Error.\n\n"
                        "The Controller API rejected the notification request.\n\n"
                        "**Trace ID:** {trace_id}\n"
                        "Check Controller API logs for details."
                    )

                return [
                    TextContent(
                        type="text",
                        text=f"❌ HTTP {status_code} Client Error\n\n"
                        f"{error_detail}\n\n"
                        f"**Trace ID:** {trace_id}",
                    )
                ]

            # Server errors (5xx) - unexpected
            else:
                return [
                    TextContent(
                        type="text",
                        text=f"❌ HTTP {status_code} Server Error\n\n"
                        "The Controller API encountered an internal error.\n\n"
                        "**Action Required:**\n"
                        "1. Check Controller API logs\n"
                        "2. Verify Controller API is running correctly\n"
                        "3. Retry the notification\n\n"
                        f"**Trace ID:** {trace_id}",
                    )
                ]

        # Parse successful response
        data = response.json()
        task_id = data.get("task_id", "unknown")
        status = data.get("status", "unknown")

        # Return success message
        return [
            TextContent(
                type="text",
                text=f"✅ Notification sent successfully!\n\n"
                f"**Task ID:** {task_id}\n"
                f"**Status:** {status}\n"
                f"**Target:** {params.target}\n"
                f"**Priority:** {params.priority}\n"
                f"**Trace ID:** {trace_id}\n\n"
                f"The notification has been routed to the {params.target} role.",
            )
        ]

    except requests.exceptions.Timeout:
        return [
            TextContent(
                type="text",
                text=f"❌ Request Timeout\n\n"
                f"The Controller API did not respond within {timeout} seconds.\n\n"
                "**Possible causes:**\n"
                "- Controller API is slow or overloaded\n"
                "- Network latency is high\n"
                "- Privacy Guard processing is slow\n\n"
                "**Action Required:**\n"
                "1. Increase MESH_TIMEOUT_SECS environment variable\n"
                "2. Check Controller API health and performance\n"
                "3. Retry the notification\n\n"
                f"**Trace ID:** {trace_id}",
            )
        ]

    except requests.exceptions.ConnectionError as e:
        return [
            TextContent(
                type="text",
                text=f"❌ Connection Error\n\n"
                f"Could not connect to the Controller API at {controller_url}\n\n"
                "**Possible causes:**\n"
                "- Controller API is not running\n"
                "- Incorrect CONTROLLER_URL\n"
                "- Network connectivity issues\n\n"
                "**Action Required:**\n"
                "1. Verify Controller API is running: curl {controller_url}/status\n"
                "2. Check CONTROLLER_URL environment variable\n"
                "3. Verify network connectivity\n\n"
                f"**Error details:** {str(e)}\n"
                f"**Trace ID:** {trace_id}",
            )
        ]

    except requests.exceptions.RequestException as e:
        return [
            TextContent(
                type="text",
                text=f"❌ Unexpected Error\n\n"
                f"An unexpected error occurred while sending the notification.\n\n"
                f"**Error details:** {str(e)}\n"
                f"**Trace ID:** {trace_id}\n\n"
                "**Action Required:**\n"
                "1. Check the error details above\n"
                "2. Verify Controller API is accessible\n"
                "3. Contact support if the issue persists",
            )
        ]

    except Exception as e:
        return [
            TextContent(
                type="text",
                text=f"❌ Internal Error\n\n"
                f"An internal error occurred in the notify tool.\n\n"
                f"**Error details:** {str(e)}\n"
                f"**Trace ID:** {trace_id}\n\n"
                "This is likely a bug. Please report this issue.",
            )
        ]


# Define the MCP tool
notify_tool = Tool(
    name="notify",
    description=(
        "Send a notification to another agent via the Controller API. "
        "The notification is delivered as a task with type='notification'. "
        "Supports priority levels: low, normal (default), high."
    ),
    inputSchema=NotifyParams.model_json_schema(),
)

# Attach the handler
notify_tool.call = notify_handler
