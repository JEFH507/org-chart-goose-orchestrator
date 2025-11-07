"""
Entry point for running privacy_guard_mcp as a module.

Allows execution via: python -m privacy_guard_mcp
"""

import sys
from .server import main

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nPrivacy Guard MCP Server shutting down...", file=sys.stderr)
        sys.exit(0)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
