#!/usr/bin/env python3
"""
Quick validation test for send_task tool

Tests:
1. Module imports work
2. Tool structure is valid
3. Parameter validation works
"""

import sys
from pathlib import Path

print("=" * 60)
print("send_task Tool Validation Test")
print("=" * 60)

# Test 1: Import the tool module
print("\n[Test 1] Importing send_task module...")
try:
    from tools.send_task import send_task_tool, SendTaskParams
    print("✓ send_task module imported successfully")
except ImportError as e:
    print(f"✗ Import failed: {e}")
    sys.exit(1)

# Test 2: Verify tool structure
print("\n[Test 2] Validating tool structure...")
try:
    assert hasattr(send_task_tool, 'name'), "Tool missing 'name' attribute"
    assert hasattr(send_task_tool, 'description'), "Tool missing 'description' attribute"
    assert hasattr(send_task_tool, 'inputSchema'), "Tool missing 'inputSchema' attribute"
    assert hasattr(send_task_tool, 'call'), "Tool missing 'call' attribute"
    
    print(f"✓ Tool name: {send_task_tool.name}")
    print(f"✓ Tool description: {send_task_tool.description[:60]}...")
    print(f"✓ Tool has input schema")
    print(f"✓ Tool has call handler")
except AssertionError as e:
    print(f"✗ Validation failed: {e}")
    sys.exit(1)

# Test 3: Parameter validation
print("\n[Test 3] Validating parameter schema...")
try:
    # Valid params
    params1 = SendTaskParams(
        target="manager",
        task={"type": "test", "data": "value"},
        context={"dept": "IT"}
    )
    print("✓ Valid params with all fields accepted")
    
    # Valid params (minimal - context optional)
    params2 = SendTaskParams(
        target="finance",
        task={"action": "review"}
    )
    print("✓ Valid params with optional context omitted")
    
    # Check default context
    assert params2.context == {}, "Default context should be empty dict"
    print("✓ Default context is empty dict")
    
except Exception as e:
    print(f"✗ Parameter validation failed: {e}")
    sys.exit(1)

# Test 4: Invalid parameters
print("\n[Test 4] Testing invalid parameters...")
try:
    from pydantic import ValidationError
    
    # Missing required field 'target'
    try:
        invalid_params = SendTaskParams(task={"test": "data"})
        print("✗ Should have rejected missing 'target' field")
        sys.exit(1)
    except ValidationError:
        print("✓ Correctly rejected missing 'target' field")
    
    # Missing required field 'task'
    try:
        invalid_params = SendTaskParams(target="manager")
        print("✗ Should have rejected missing 'task' field")
        sys.exit(1)
    except ValidationError:
        print("✓ Correctly rejected missing 'task' field")
        
except Exception as e:
    print(f"✗ Invalid parameter test failed unexpectedly: {e}")
    sys.exit(1)

# Test 5: Input schema structure
print("\n[Test 5] Validating JSON schema...")
try:
    schema = send_task_tool.inputSchema
    
    # Check it's a dict (JSON schema format)
    assert isinstance(schema, dict), "Input schema should be a dict"
    print("✓ Input schema is dict")
    
    # Check required fields present in schema
    assert 'properties' in schema, "Schema missing 'properties'"
    assert 'target' in schema['properties'], "Schema missing 'target' property"
    assert 'task' in schema['properties'], "Schema missing 'task' property"
    print("✓ Schema has required properties")
    
    # Check required fields list
    if 'required' in schema:
        assert 'target' in schema['required'], "'target' should be required"
        assert 'task' in schema['required'], "'task' should be required"
        print("✓ Schema correctly marks required fields")
    
except Exception as e:
    print(f"✗ Schema validation failed: {e}")
    sys.exit(1)

# Summary
print("\n" + "=" * 60)
print("✅ All validation tests PASSED")
print("=" * 60)
print("\nNext steps:")
print("1. Install dependencies: ./setup.sh or ./setup.sh docker")
print("2. Configure .env with CONTROLLER_URL and MESH_JWT_TOKEN")
print("3. Test with real Controller API")
print("4. Integrate with Goose")
