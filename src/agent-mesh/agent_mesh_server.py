#!/usr/bin/env python3
"""
Agent Mesh MCP Server

MCP server for multi-agent orchestration via Controller API.
Provides 4 tools: send_task, request_approval, notify, fetch_status

Uses FastMCP for modern MCP 1.20+ compatibility.
"""

import sys
from pathlib import Path

# Add parent directory to path for tool imports
sys.path.insert(0, str(Path(__file__).parent))

from mcp.server import FastMCP

# Import tool handlers (functions, not Tool objects)
from tools.send_task import send_task_handler
from tools.request_approval import request_approval_handler  
from tools.notify import notify_handler
from tools.fetch_status import fetch_status_handler

# Create FastMCP server
mcp = FastMCP("agent-mesh")

# Register tools using add_tool method (same as privacy-guard-mcp pattern)
mcp.add_tool(
    send_task_handler,
    name="send_task",
    description=(
        "Route a task to another agent via the Controller API. "
        "Supports automatic retry with exponential backoff for resilience. "
        "Returns task ID for status tracking."
    )
)

mcp.add_tool(
    request_approval_handler,
    name="request_approval",
    description=(
        "Request approval from a manager or other authorized role. "
        "Supports timeout tracking and automatic escalation. "
        "Returns approval request ID."
    )
)

mcp.add_tool(
    notify_handler,
    name="notify",
    description=(
        "Send a notification to another agent or role. "
        "Used for status updates, alerts, and non-blocking messages. "
        "Returns notification ID."
    )
)

mcp.add_tool(
    fetch_status_handler,
    name="fetch_status",
    description=(
        "Check the status of a previously submitted task or approval request. "
        "Returns current status, progress, and any results."
    )
)


def main():
    """Main entry point for the Agent Mesh MCP server."""
    print("Agent Mesh MCP Server starting...", file=sys.stderr)
    print("Version: 0.2.0 (FastMCP)", file=sys.stderr)
    print("Registered 4 tools: send_task, request_approval, notify, fetch_status", file=sys.stderr)
    mcp.run()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nAgent Mesh MCP Server shutting down...", file=sys.stderr)
        sys.exit(0)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
