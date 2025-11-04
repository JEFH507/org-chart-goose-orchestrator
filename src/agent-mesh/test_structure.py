#!/usr/bin/env python3
"""Quick structure validation test - verifies imports work"""

import sys
from pathlib import Path

print("✓ Python version:", sys.version)
print("✓ Working directory:", Path.cwd())

# Test basic imports
try:
    import asyncio
    print("✓ asyncio module available")
except ImportError as e:
    print("✗ asyncio import failed:", e)
    sys.exit(1)

# Test project structure
try:
    import agent_mesh_server
    print("✓ agent_mesh_server module loadable")
except ImportError as e:
    print("⚠ agent_mesh_server import failed (expected - mcp not installed yet):", e)

# Check file structure
required_files = [
    "pyproject.toml",
    "agent_mesh_server.py",
    ".env.example",
    "README.md",
    "tools/__init__.py",
    "tests/__init__.py",
]

all_present = True
for file in required_files:
    path = Path(file)
    if path.exists():
        print(f"✓ {file} exists")
    else:
        print(f"✗ {file} MISSING")
        all_present = False

if all_present:
    print("\n✅ Structure validation PASSED")
    print("Next: Run ./setup.sh to install dependencies")
    sys.exit(0)
else:
    print("\n❌ Structure validation FAILED")
    sys.exit(1)
