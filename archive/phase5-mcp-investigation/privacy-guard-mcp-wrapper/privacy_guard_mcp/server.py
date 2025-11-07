#!/usr/bin/env python3
"""
Privacy Guard MCP Server

MCP server for Privacy Guard PII detection and masking.
Provides 4 tools: scan_pii, mask_pii, set_privacy_mode, get_privacy_status
"""

import sys
from pathlib import Path

# Add parent directory to path for tool imports
sys.path.insert(0, str(Path(__file__).parent))

from mcp.server import FastMCP

# Import tool handlers
from tools.scan_pii import scan_pii_handler
from tools.mask_pii import mask_pii_handler
from tools.set_privacy_mode import set_privacy_mode_handler
from tools.get_privacy_status import get_privacy_status_handler

# Create FastMCP server
mcp = FastMCP("privacy-guard")

# Register tools
mcp.add_tool(
    scan_pii_handler,
    name="scan_pii",
    description="Detect PII (SSN, email, phone, names, etc.) using Privacy Guard. Returns findings with positions and confidence scores."
)

mcp.add_tool(
    mask_pii_handler,
    name="mask_pii",
    description="Mask detected PII in text using specified method (fpe, pseudonym, or redact). Returns masked text and replacement details."
)

mcp.add_tool(
    set_privacy_mode_handler,
    name="set_privacy_mode",
    description="Configure Privacy Guard detection mode: off, rules_only (regex), ner_only (AI), or hybrid (both)."
)

mcp.add_tool(
    get_privacy_status_handler,
    name="get_privacy_status",
    description="Query Privacy Guard configuration and health status. Returns current mode, supported categories, and service health."
)


def main():
    """Main entry point for the Privacy Guard MCP server."""
    print("Privacy Guard MCP Server starting...", file=sys.stderr)
    print("Version: 0.1.0", file=sys.stderr)
    print("Registered 4 tools: scan_pii, mask_pii, set_privacy_mode, get_privacy_status", file=sys.stderr)
    mcp.run()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nPrivacy Guard MCP Server shutting down...", file=sys.stderr)
        sys.exit(0)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
