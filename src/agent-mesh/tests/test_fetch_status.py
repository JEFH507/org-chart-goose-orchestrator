"""
Validation tests for the fetch_status tool.

Tests tool definition, schema structure, parameter validation, and handler presence.
"""

import sys
from pathlib import Path

# Add parent directory to path to import tools module
sys.path.insert(0, str(Path(__file__).parent.parent))

from tools.fetch_status import fetch_status_tool, FetchStatusParams


def test_tool_definition():
    """Test that fetch_status_tool is properly defined."""
    assert fetch_status_tool.name == "fetch_status"
    assert fetch_status_tool.description is not None
    assert len(fetch_status_tool.description) > 0
    assert "status" in fetch_status_tool.description.lower()
    assert "task" in fetch_status_tool.description.lower()
    assert fetch_status_tool.inputSchema is not None
    assert hasattr(fetch_status_tool, 'call')
    print("✓ Tool definition test passed")


def test_input_schema_structure():
    """Test that the input schema has the correct structure."""
    schema = fetch_status_tool.inputSchema
    
    # Check schema has properties
    assert "properties" in schema
    properties = schema["properties"]
    
    # Check required field is present
    assert "task_id" in properties
    
    # Check required fields list
    assert "required" in schema
    assert "task_id" in schema["required"]
    
    # task_id should be the only required field
    assert len(schema["required"]) == 1
    
    print("✓ Input schema structure test passed")


def test_parameter_validation():
    """Test that FetchStatusParams validates correctly."""
    # Valid params with task_id
    valid_params = FetchStatusParams(task_id="550e8400-e29b-41d4-a716-446655440000")
    assert valid_params.task_id == "550e8400-e29b-41d4-a716-446655440000"
    
    # Test with different task ID format
    valid_params2 = FetchStatusParams(task_id="task-123")
    assert valid_params2.task_id == "task-123"
    
    # Test that task_id is required
    try:
        invalid_params = FetchStatusParams()
        assert False, "Should have raised validation error for missing task_id"
    except Exception as e:
        # Pydantic validation error expected
        assert "task_id" in str(e).lower()
    
    print("✓ Parameter validation test passed")


def test_schema_field_types():
    """Test that schema fields have correct types."""
    schema = fetch_status_tool.inputSchema
    properties = schema["properties"]
    
    # task_id should be a string
    assert properties["task_id"]["type"] == "string"
    
    # Check description is present
    assert "description" in properties["task_id"]
    assert len(properties["task_id"]["description"]) > 0
    
    print("✓ Schema field types test passed")


def test_no_optional_parameters():
    """Test that there are no optional parameters (only task_id required)."""
    schema = fetch_status_tool.inputSchema
    properties = schema["properties"]
    required_fields = schema["required"]
    
    # Should have exactly 1 property (task_id)
    assert len(properties) == 1
    
    # Should have exactly 1 required field
    assert len(required_fields) == 1
    assert required_fields[0] == "task_id"
    
    print("✓ No optional parameters test passed")


if __name__ == "__main__":
    test_tool_definition()
    test_input_schema_structure()
    test_parameter_validation()
    test_schema_field_types()
    test_no_optional_parameters()
    print("\n✅ All fetch_status validation tests passed!")
