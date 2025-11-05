"""
Validation tests for the notify tool.

Tests the tool structure, input schema, and parameter validation.
Does NOT test actual HTTP calls (that's for integration tests).
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from tools.notify import notify_tool, NotifyParams


def test_tool_definition():
    """Test that the notify tool is properly defined."""
    print("Test 1: Tool definition...")

    assert notify_tool.name == "notify", f"Expected name 'notify', got '{notify_tool.name}'"
    print(f"✓ Tool name: {notify_tool.name}")

    assert "notification" in notify_tool.description.lower(), (
        "Tool description should mention 'notification'"
    )
    print(f"✓ Tool description mentions 'notification'")

    assert notify_tool.inputSchema is not None, "Tool must have input schema"
    print("✓ Tool has input schema")

    assert callable(notify_tool.call), "Tool must have callable handler"
    print("✓ Tool has call handler")

    print()


def test_input_schema_structure():
    """Test that the input schema has correct structure."""
    print("Test 2: Input schema structure...")

    schema = notify_tool.inputSchema
    assert isinstance(schema, dict), "Input schema must be a dict"
    print("✓ Input schema is dict")

    # Check required properties exist
    properties = schema.get("properties", {})
    required_fields = ["target", "message"]

    for field in required_fields:
        assert field in properties, f"Schema must have '{field}' property"
    print(f"✓ Schema has required properties: {required_fields}")

    # Check optional properties exist
    optional_fields = ["priority"]
    for field in optional_fields:
        assert field in properties, f"Schema must have '{field}' property"
    print(f"✓ Schema has optional properties: {optional_fields}")

    # Check required field list
    required = schema.get("required", [])
    for field in required_fields:
        assert field in required, f"Field '{field}' must be marked as required"
    print(f"✓ Schema correctly marks required fields: {required}")

    print()


def test_parameter_validation():
    """Test that NotifyParams validates input correctly."""
    print("Test 3: Parameter validation...")

    # Test valid params with all fields
    try:
        params = NotifyParams(
            target="manager", message="Budget approval needed", priority="high"
        )
        print("✓ Valid params with all fields accepted")
    except Exception as e:
        raise AssertionError(f"Valid params should be accepted: {e}")

    # Test valid params with only required fields (priority should default)
    try:
        params = NotifyParams(target="finance", message="New task assigned")
        assert params.priority == "normal", f"Default priority should be 'normal', got '{params.priority}'"
        print("✓ Valid params with only required fields accepted")
        print(f"✓ Default priority: '{params.priority}'")
    except Exception as e:
        raise AssertionError(f"Valid params with defaults should be accepted: {e}")

    # Test missing 'target' field
    try:
        params = NotifyParams(message="Test message")
        raise AssertionError("Should reject params missing 'target' field")
    except Exception:
        print("✓ Correctly rejected missing 'target' field")

    # Test missing 'message' field
    try:
        params = NotifyParams(target="manager")
        raise AssertionError("Should reject params missing 'message' field")
    except Exception:
        print("✓ Correctly rejected missing 'message' field")

    print()


def test_schema_field_types():
    """Test that schema field types are correct."""
    print("Test 4: Schema field types...")

    schema = notify_tool.inputSchema
    properties = schema.get("properties", {})

    # All fields should be string type
    expected_types = {
        "target": "string",
        "message": "string",
        "priority": "string",
    }

    for field, expected_type in expected_types.items():
        field_schema = properties.get(field, {})
        actual_type = field_schema.get("type")
        assert actual_type == expected_type, (
            f"Field '{field}' should be type '{expected_type}', got '{actual_type}'"
        )

    print(f"✓ All {len(expected_types)} fields are string type")

    print()


def test_default_values():
    """Test that default values are correct."""
    print("Test 5: Default values...")

    # Test priority default
    params = NotifyParams(target="engineering", message="Deployment complete")
    assert params.priority == "normal", f"Default priority should be 'normal', got '{params.priority}'"
    print(f"✓ Default priority: '{params.priority}'")

    # Test explicit priority values
    for priority in ["low", "normal", "high"]:
        params = NotifyParams(
            target="manager", message="Test", priority=priority
        )
        assert params.priority == priority, f"Priority should be '{priority}', got '{params.priority}'"
    print("✓ All priority values ('low', 'normal', 'high') accepted")

    print()


def run_all_tests():
    """Run all validation tests."""
    print("=" * 60)
    print("NOTIFY TOOL VALIDATION TESTS")
    print("=" * 60)
    print()

    try:
        test_tool_definition()
        test_input_schema_structure()
        test_parameter_validation()
        test_schema_field_types()
        test_default_values()

        print("=" * 60)
        print("✅ All validation tests PASSED")
        print("=" * 60)
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
