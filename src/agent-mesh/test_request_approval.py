#!/usr/bin/env python3
"""
Validation tests for request_approval tool (B3)

Tests tool structure, input schema, and parameter validation.
Does NOT test HTTP requests (requires running Controller API).
"""

import sys
from pathlib import Path

# Add current directory to path
sys.path.insert(0, str(Path(__file__).parent))

from tools.request_approval import (
    request_approval_tool,
    RequestApprovalParams,
)


def test_tool_definition():
    """Test 1: Tool is properly defined."""
    print("Test 1: Tool definition...")
    
    assert request_approval_tool.name == "request_approval", \
        f"Expected name 'request_approval', got '{request_approval_tool.name}'"
    
    assert "approval" in request_approval_tool.description.lower(), \
        "Tool description should mention 'approval'"
    
    assert request_approval_tool.inputSchema is not None, \
        "Tool must have input schema"
    
    print("✓ Tool name: request_approval")
    print(f"✓ Tool description: {request_approval_tool.description[:80]}...")
    print("✓ Tool has input schema")


def test_input_schema():
    """Test 2: Input schema is correctly structured."""
    print("\nTest 2: Input schema structure...")
    
    schema = request_approval_tool.inputSchema
    
    assert isinstance(schema, dict), "Input schema must be dict"
    assert "properties" in schema, "Schema must have 'properties'"
    
    properties = schema["properties"]
    
    # Required fields
    required_fields = ["task_id", "approver_role", "reason"]
    for field in required_fields:
        assert field in properties, f"Schema must have '{field}' property"
    
    # Optional fields
    optional_fields = ["decision", "comments"]
    for field in optional_fields:
        assert field in properties, f"Schema must have '{field}' property"
    
    print(f"✓ Schema has required properties: {required_fields}")
    print(f"✓ Schema has optional properties: {optional_fields}")
    
    # Check required fields list
    assert "required" in schema, "Schema must specify required fields"
    schema_required = schema["required"]
    
    for field in required_fields:
        assert field in schema_required, f"'{field}' must be in required list"
    
    print(f"✓ Schema correctly marks required fields: {schema_required}")


def test_params_validation():
    """Test 3: Pydantic params validation."""
    print("\nTest 3: Parameter validation...")
    
    # Valid params with all fields
    params = RequestApprovalParams(
        task_id="task-12345",
        approver_role="manager",
        reason="Budget increase for Q1 hiring",
        decision="pending",
        comments="Urgent request"
    )
    assert params.task_id == "task-12345"
    assert params.approver_role == "manager"
    assert params.reason == "Budget increase for Q1 hiring"
    assert params.decision == "pending"
    assert params.comments == "Urgent request"
    print("✓ Valid params with all fields accepted")
    
    # Valid params with only required fields
    params = RequestApprovalParams(
        task_id="task-67890",
        approver_role="director",
        reason="Project funding approval"
    )
    assert params.task_id == "task-67890"
    assert params.approver_role == "director"
    assert params.reason == "Project funding approval"
    assert params.decision == "pending"  # Default value
    assert params.comments == ""  # Default value
    print("✓ Valid params with only required fields accepted")
    print(f"✓ Default decision: '{params.decision}'")
    print(f"✓ Default comments: '{params.comments}'")
    
    # Test missing required field
    try:
        RequestApprovalParams(
            approver_role="manager",
            reason="Test"
        )  # Missing task_id
        assert False, "Should have raised ValidationError for missing task_id"
    except Exception as e:
        assert "task_id" in str(e).lower(), "Error should mention missing task_id"
        print("✓ Correctly rejected missing 'task_id' field")
    
    try:
        RequestApprovalParams(
            task_id="task-123",
            reason="Test"
        )  # Missing approver_role
        assert False, "Should have raised ValidationError for missing approver_role"
    except Exception as e:
        assert "approver_role" in str(e).lower(), "Error should mention missing approver_role"
        print("✓ Correctly rejected missing 'approver_role' field")
    
    try:
        RequestApprovalParams(
            task_id="task-123",
            approver_role="manager"
        )  # Missing reason
        assert False, "Should have raised ValidationError for missing reason"
    except Exception as e:
        assert "reason" in str(e).lower(), "Error should mention missing reason"
        print("✓ Correctly rejected missing 'reason' field")


def test_schema_types():
    """Test 4: Input schema field types."""
    print("\nTest 4: Schema field types...")
    
    schema = request_approval_tool.inputSchema
    properties = schema["properties"]
    
    # task_id should be string
    assert properties["task_id"]["type"] == "string", \
        "task_id must be string type"
    print("✓ task_id type: string")
    
    # approver_role should be string
    assert properties["approver_role"]["type"] == "string", \
        "approver_role must be string type"
    print("✓ approver_role type: string")
    
    # reason should be string
    assert properties["reason"]["type"] == "string", \
        "reason must be string type"
    print("✓ reason type: string")
    
    # decision should be string
    assert properties["decision"]["type"] == "string", \
        "decision must be string type"
    print("✓ decision type: string")
    
    # comments should be string
    assert properties["comments"]["type"] == "string", \
        "comments must be string type"
    print("✓ comments type: string")


def test_default_values():
    """Test 5: Default values for optional fields."""
    print("\nTest 5: Default values...")
    
    params = RequestApprovalParams(
        task_id="task-123",
        approver_role="manager",
        reason="Test approval"
    )
    
    assert params.decision == "pending", \
        "Default decision should be 'pending'"
    print("✓ Default decision: 'pending'")
    
    assert params.comments == "", \
        "Default comments should be empty string"
    print("✓ Default comments: '' (empty string)")


def main():
    """Run all validation tests."""
    print("=" * 60)
    print("Request Approval Tool Validation (B3)")
    print("=" * 60)
    
    try:
        test_tool_definition()
        test_input_schema()
        test_params_validation()
        test_schema_types()
        test_default_values()
        
        print("\n" + "=" * 60)
        print("✅ All validation tests PASSED")
        print("=" * 60)
        return 0
    
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        return 1
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
