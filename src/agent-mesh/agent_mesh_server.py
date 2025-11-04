#!/usr/bin/env python3
"""
Agent Mesh MCP Server

MCP server for multi-agent orchestration via Controller API.
Provides 4 tools: send_task, request_approval, notify, fetch_status
"""

import asyncio
import sys
from pathlib import Path

# Add parent directory to path for tool imports
sys.path.insert(0, str(Path(__file__).parent))

from mcp.server import Server
from mcp.server.stdio import stdio_server

# Import tool implementations
from tools.send_task import send_task_tool

# TODO: Import remaining tools (B3-B5)
# from tools.request_approval import request_approval_tool
# from tools.notify import notify_tool
# from tools.fetch_status import fetch_status_tool


async def main():
    """Main entry point for the Agent Mesh MCP server."""
    server = Server("agent-mesh")
    
    # Register implemented tools
    server.add_tool(send_task_tool)
    
    # TODO: Register remaining tools (B3-B5)
    # server.add_tool(request_approval_tool)
    # server.add_tool(notify_tool)
    # server.add_tool(fetch_status_tool)
    
    print("Agent Mesh MCP Server starting...", file=sys.stderr)
    print("Version: 0.1.0", file=sys.stderr)
    print("Waiting for MCP client connection via stdio...", file=sys.stderr)
    
    # Run stdio transport
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nAgent Mesh MCP Server shutting down...", file=sys.stderr)
        sys.exit(0)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
