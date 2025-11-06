#!/usr/bin/env python3
"""
Privacy Guard MCP Server

MCP server for Privacy Guard PII detection and masking.
Provides 4 tools: scan_pii, mask_pii, set_privacy_mode, get_privacy_status
"""

import asyncio
import sys
from pathlib import Path

# Add parent directory to path for tool imports
sys.path.insert(0, str(Path(__file__).parent))

from mcp.server import Server
from mcp.server.stdio import stdio_server

# Import tool implementations
from tools.scan_pii import scan_pii_tool
from tools.mask_pii import mask_pii_tool
from tools.set_privacy_mode import set_privacy_mode_tool
from tools.get_privacy_status import get_privacy_status_tool


async def main():
    """Main entry point for the Privacy Guard MCP server."""
    server = Server("privacy-guard")
    
    # Register implemented tools
    server.add_tool(scan_pii_tool)
    server.add_tool(mask_pii_tool)
    server.add_tool(set_privacy_mode_tool)
    server.add_tool(get_privacy_status_tool)
    
    print("Privacy Guard MCP Server starting...", file=sys.stderr)
    print("Version: 0.1.0", file=sys.stderr)
    print("Waiting for MCP client connection via stdio...", file=sys.stderr)
    
    # Run stdio transport
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nPrivacy Guard MCP Server shutting down...", file=sys.stderr)
        sys.exit(0)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
