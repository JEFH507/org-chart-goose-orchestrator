"""
Test that all tools are properly registered in the MCP server.
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from tools import send_task_tool, request_approval_tool, notify_tool, fetch_status_tool


def test_tool_exports():
    """Test that all implemented tools are exported from tools module."""
    print("Test 1: Tool exports from tools module...")
    
    # Check send_task_tool
    assert send_task_tool is not None, "send_task_tool should be exported"
    assert send_task_tool.name == "send_task"
    print("✓ send_task_tool exported correctly")
    
    # Check request_approval_tool
    assert request_approval_tool is not None, "request_approval_tool should be exported"
    assert request_approval_tool.name == "request_approval"
    print("✓ request_approval_tool exported correctly")
    
    # Check notify_tool
    assert notify_tool is not None, "notify_tool should be exported"
    assert notify_tool.name == "notify"
    print("✓ notify_tool exported correctly")
    
    # Check fetch_status_tool
    assert fetch_status_tool is not None, "fetch_status_tool should be exported"
    assert fetch_status_tool.name == "fetch_status"
    print("✓ fetch_status_tool exported correctly")
    
    print()


def test_tool_count():
    """Test that we have the expected number of tools."""
    print("Test 2: Tool count...")
    
    from tools import __all__
    
    expected_tools = ["send_task_tool", "request_approval_tool", "notify_tool", "fetch_status_tool"]
    
    for tool in expected_tools:
        assert tool in __all__, f"Tool '{tool}' should be in __all__"
    
    print(f"✓ All {len(expected_tools)} implemented tools in __all__")
    print(f"  Implemented: {expected_tools}")
    
    print()


def test_tool_handlers():
    """Test that all tools have handlers attached."""
    print("Test 3: Tool handlers...")
    
    tools = [
        ("send_task", send_task_tool),
        ("request_approval", request_approval_tool),
        ("notify", notify_tool),
        ("fetch_status", fetch_status_tool),
    ]
    
    for name, tool in tools:
        # Check if tool has 'call' attribute (MCP SDK handles this)
        assert hasattr(tool, 'call'), f"Tool '{name}' should have 'call' attribute"
        print(f"✓ {name} has handler attached")
    
    print()


def run_all_tests():
    """Run all server tool tests."""
    print("=" * 60)
    print("SERVER TOOL REGISTRATION TESTS")
    print("=" * 60)
    print()
    
    try:
        test_tool_exports()
        test_tool_count()
        test_tool_handlers()
        
        print("=" * 60)
        print("✅ All server tool tests PASSED")
        print("=" * 60)
        print()
        print("MCP Server Status:")
        print("  Tools implemented: 4/4 (100%)")
        print("  - send_task ✓")
        print("  - request_approval ✓")
        print("  - notify ✓")
        print("  - fetch_status ✓")
        return 0
    
    except AssertionError as e:
        print()
        print("=" * 60)
        print(f"❌ Test failed: {e}")
        print("=" * 60)
        return 1
    
    except Exception as e:
        print()
        print("=" * 60)
        print(f"❌ Unexpected error: {e}")
        print("=" * 60)
        return 1


if __name__ == "__main__":
    exit(run_all_tests())
