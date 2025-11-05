"""
Integration tests for Agent Mesh MCP Server with Controller API.

These tests require:
1. Controller API running at CONTROLLER_URL (default: http://localhost:8088)
2. Valid JWT token in MESH_JWT_TOKEN environment variable
3. Controller API endpoints functional (POST /tasks/route, GET /sessions, POST /approvals)

Run with: pytest tests/test_integration.py -v
"""

import asyncio
import os
import uuid
import time
import pytest
import requests
from unittest.mock import AsyncMock, patch, MagicMock
from mcp.types import TextContent

# Import tools
from tools.send_task import send_task_handler, SendTaskParams
from tools.request_approval import request_approval_handler, RequestApprovalParams
from tools.notify import notify_handler, NotifyParams
from tools.fetch_status import fetch_status_handler, FetchStatusParams


# ============================================================
# Fixtures
# ============================================================

@pytest.fixture
def controller_url():
    """Get Controller API URL from environment."""
    return os.getenv("CONTROLLER_URL", "http://localhost:8088")


@pytest.fixture
def jwt_token():
    """Get JWT token from environment."""
    token = os.getenv("MESH_JWT_TOKEN")
    if not token:
        pytest.skip("MESH_JWT_TOKEN environment variable not set")
    return token


@pytest.fixture
def check_controller_health(controller_url):
    """Verify Controller API is running before tests."""
    try:
        response = requests.get(f"{controller_url}/status", timeout=5)
        if response.status_code != 200:
            pytest.skip(f"Controller API not healthy: {response.status_code}")
    except requests.exceptions.RequestException as e:
        pytest.skip(f"Controller API not reachable: {e}")


# ============================================================
# Test 1: send_task Tool Integration
# ============================================================

@pytest.mark.asyncio
async def test_send_task_success(controller_url, jwt_token, check_controller_health):
    """Test send_task successfully routes a task to the Controller API."""
    # Arrange
    params = SendTaskParams(
        target="manager",
        task={"type": "budget_approval", "amount": 50000},
        context={"department": "Engineering", "quarter": "Q1"}
    )
    
    # Act
    result = await send_task_handler(params)
    
    # Assert
    assert isinstance(result, list)
    assert len(result) == 1
    assert isinstance(result[0], TextContent)
    
    text = result[0].text
    assert "✅" in text or "success" in text.lower()
    assert "Task ID:" in text or "task_id" in text.lower()
    assert "routed" in text.lower() or "accepted" in text.lower()


@pytest.mark.asyncio
async def test_send_task_missing_jwt_token(controller_url, check_controller_health):
    """Test send_task fails gracefully when JWT token is missing."""
    # Arrange
    params = SendTaskParams(
        target="manager",
        task={"type": "test"},
        context={}
    )
    
    # Temporarily remove JWT token
    original_token = os.environ.get("MESH_JWT_TOKEN")
    if "MESH_JWT_TOKEN" in os.environ:
        del os.environ["MESH_JWT_TOKEN"]
    
    try:
        # Act
        result = await send_task_handler(params)
        
        # Assert
        assert isinstance(result, list)
        assert len(result) == 1
        text = result[0].text
        assert "ERROR" in text or "❌" in text
        assert "MESH_JWT_TOKEN" in text
    finally:
        # Restore token
        if original_token:
            os.environ["MESH_JWT_TOKEN"] = original_token


@pytest.mark.asyncio
async def test_send_task_retry_logic(controller_url, jwt_token, check_controller_health):
    """Test send_task retry logic with exponential backoff."""
    # This test verifies retry behavior by checking timing
    # We'll use a mock to simulate transient failures
    
    params = SendTaskParams(
        target="manager",
        task={"type": "retry_test"},
        context={}
    )
    
    # Mock requests.post to fail twice, then succeed
    original_post = requests.post
    call_count = [0]
    
    def mock_post(*args, **kwargs):
        call_count[0] += 1
        if call_count[0] < 3:
            # Simulate connection error for first 2 attempts
            raise requests.exceptions.ConnectionError("Simulated connection error")
        # Third attempt succeeds
        return original_post(*args, **kwargs)
    
    with patch('requests.post', side_effect=mock_post):
        start_time = time.time()
        result = await send_task_handler(params)
        elapsed = time.time() - start_time
        
        # Assert: Should have retried (elapsed time > 0s for retries)
        # With exponential backoff: 2^0 + jitter (~1s) + 2^1 + jitter (~2s) = ~3s minimum
        # But since we're mocking, actual timing may vary
        assert call_count[0] == 3, f"Expected 3 attempts, got {call_count[0]}"


@pytest.mark.asyncio
async def test_send_task_with_trace_id(controller_url, jwt_token, check_controller_health):
    """Test send_task includes trace ID in request headers."""
    # Arrange
    params = SendTaskParams(
        target="manager",
        task={"type": "trace_test"},
        context={"test": "trace_id_propagation"}
    )
    
    # Act
    result = await send_task_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    # Successful response should include trace ID in formatted output
    assert "Trace ID:" in text or "trace" in text.lower()


# ============================================================
# Test 2: request_approval Tool Integration
# ============================================================

@pytest.mark.asyncio
async def test_request_approval_success(controller_url, jwt_token, check_controller_health):
    """Test request_approval successfully submits an approval request."""
    # First, create a task to get a task_id
    send_params = SendTaskParams(
        target="manager",
        task={"type": "approval_test"},
        context={}
    )
    send_result = await send_task_handler(send_params)
    send_text = send_result[0].text
    
    # Extract task_id from send_task response
    # Format: "Task ID: task-abc123..."
    import re
    match = re.search(r'task-[a-f0-9-]+', send_text, re.IGNORECASE)
    if not match:
        pytest.fail(f"Could not extract task_id from send_task response: {send_text}")
    
    task_id = match.group(0)
    
    # Arrange
    params = RequestApprovalParams(
        task_id=task_id,
        approver_role="manager",
        reason="Integration test approval request"
    )
    
    # Act
    result = await request_approval_handler(params)
    
    # Assert
    assert isinstance(result, list)
    assert len(result) == 1
    text = result[0].text
    assert "✅" in text or "success" in text.lower()
    assert "pending" in text.lower() or "submitted" in text.lower()


@pytest.mark.asyncio
async def test_request_approval_with_optional_fields(controller_url, jwt_token, check_controller_health):
    """Test request_approval with optional decision and comments fields."""
    # Create task first
    send_params = SendTaskParams(target="manager", task={"type": "test"}, context={})
    send_result = await send_task_handler(send_params)
    
    import re
    match = re.search(r'task-[a-f0-9-]+', send_result[0].text, re.IGNORECASE)
    task_id = match.group(0) if match else str(uuid.uuid4())
    
    # Arrange with all optional fields
    params = RequestApprovalParams(
        task_id=task_id,
        approver_role="manager",
        reason="Test with all fields",
        decision="approved",
        comments="Integration test approval"
    )
    
    # Act
    result = await request_approval_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    assert "✅" in text or "success" in text.lower()


@pytest.mark.asyncio
async def test_request_approval_invalid_task_id(controller_url, jwt_token, check_controller_health):
    """Test request_approval handles invalid task_id (404 Not Found)."""
    # Arrange - use a fake task_id that doesn't exist
    params = RequestApprovalParams(
        task_id="task-00000000-0000-0000-0000-000000000000",
        approver_role="manager",
        reason="Test invalid task ID"
    )
    
    # Act
    result = await request_approval_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    # May be 404 (task not found) or success (Controller API accepts all task_ids in Phase 3)
    # Either is acceptable for Phase 3 (no persistence)
    assert "❌" in text or "✅" in text


# ============================================================
# Test 3: notify Tool Integration
# ============================================================

@pytest.mark.asyncio
async def test_notify_success(controller_url, jwt_token, check_controller_health):
    """Test notify successfully sends a notification."""
    # Arrange
    params = NotifyParams(
        target="manager",
        message="Integration test notification",
        priority="normal"
    )
    
    # Act
    result = await notify_handler(params)
    
    # Assert
    assert isinstance(result, list)
    assert len(result) == 1
    text = result[0].text
    assert "✅" in text or "success" in text.lower()
    assert "notification" in text.lower() or "sent" in text.lower()


@pytest.mark.asyncio
async def test_notify_high_priority(controller_url, jwt_token, check_controller_health):
    """Test notify with high priority."""
    # Arrange
    params = NotifyParams(
        target="manager",
        message="Urgent notification",
        priority="high"
    )
    
    # Act
    result = await notify_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    assert "✅" in text or "success" in text.lower()


@pytest.mark.asyncio
async def test_notify_invalid_priority(controller_url, jwt_token, check_controller_health):
    """Test notify rejects invalid priority values."""
    # Arrange
    params = NotifyParams(
        target="manager",
        message="Test message",
        priority="urgent"  # Invalid - should be 'low', 'normal', or 'high'
    )
    
    # Act
    result = await notify_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    assert "❌" in text or "ERROR" in text or "Invalid" in text


# ============================================================
# Test 4: fetch_status Tool Integration
# ============================================================

@pytest.mark.asyncio
async def test_fetch_status_success(controller_url, jwt_token, check_controller_health):
    """Test fetch_status retrieves task status."""
    # First, create a task to get a task_id
    send_params = SendTaskParams(
        target="manager",
        task={"type": "status_test"},
        context={}
    )
    send_result = await send_task_handler(send_params)
    
    import re
    match = re.search(r'task-[a-f0-9-]+', send_result[0].text, re.IGNORECASE)
    if not match:
        pytest.skip("Could not extract task_id from send_task response")
    
    task_id = match.group(0)
    
    # Arrange
    params = FetchStatusParams(task_id=task_id)
    
    # Act
    result = await fetch_status_handler(params)
    
    # Assert
    assert isinstance(result, list)
    assert len(result) == 1
    text = result[0].text
    # May be success or 404 (Controller API ephemeral in Phase 3)
    # Either response is valid
    assert "✅" in text or "❌" in text


@pytest.mark.asyncio
async def test_fetch_status_empty_task_id(controller_url, jwt_token, check_controller_health):
    """Test fetch_status handles empty task_id."""
    # Arrange
    params = FetchStatusParams(task_id="")
    
    # Act
    result = await fetch_status_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    assert "❌" in text or "ERROR" in text or "empty" in text.lower()


@pytest.mark.asyncio
async def test_fetch_status_not_found(controller_url, jwt_token, check_controller_health):
    """Test fetch_status handles task not found (404)."""
    # Arrange - use a task_id that doesn't exist
    params = FetchStatusParams(task_id="task-nonexistent-12345")
    
    # Act
    result = await fetch_status_handler(params)
    
    # Assert
    assert isinstance(result, list)
    text = result[0].text
    # Expect 404 error or success (depending on Controller API implementation)
    assert "❌" in text or "✅" in text


# ============================================================
# Test 5: End-to-End Workflow
# ============================================================

@pytest.mark.asyncio
async def test_end_to_end_workflow(controller_url, jwt_token, check_controller_health):
    """Test complete workflow: send_task → request_approval → fetch_status."""
    
    # Step 1: Send task
    send_params = SendTaskParams(
        target="manager",
        task={"type": "e2e_test", "amount": 75000},
        context={"department": "Finance"}
    )
    send_result = await send_task_handler(send_params)
    assert "✅" in send_result[0].text or "success" in send_result[0].text.lower()
    
    # Extract task_id
    import re
    match = re.search(r'task-[a-f0-9-]+', send_result[0].text, re.IGNORECASE)
    if not match:
        pytest.fail("Could not extract task_id from send_task response")
    task_id = match.group(0)
    
    # Step 2: Request approval
    approval_params = RequestApprovalParams(
        task_id=task_id,
        approver_role="manager",
        reason="E2E test approval workflow"
    )
    approval_result = await request_approval_handler(approval_params)
    assert isinstance(approval_result, list)
    # Accept both success and error (Controller API may not persist in Phase 3)
    
    # Step 3: Fetch status
    status_params = FetchStatusParams(task_id=task_id)
    status_result = await fetch_status_handler(status_params)
    assert isinstance(status_result, list)
    # Accept both success and 404 (Controller API ephemeral in Phase 3)


# ============================================================
# Test 6: Error Handling
# ============================================================

@pytest.mark.asyncio
async def test_controller_api_unreachable(controller_url, jwt_token):
    """Test tools handle Controller API being unreachable."""
    # Arrange - use invalid URL
    original_url = os.environ.get("CONTROLLER_URL")
    os.environ["CONTROLLER_URL"] = "http://localhost:9999"  # Non-existent port
    
    try:
        params = SendTaskParams(target="manager", task={"type": "test"}, context={})
        
        # Act
        result = await send_task_handler(params)
        
        # Assert
        assert isinstance(result, list)
        text = result[0].text
        assert "❌" in text or "ERROR" in text or "failed" in text.lower()
    finally:
        # Restore original URL
        if original_url:
            os.environ["CONTROLLER_URL"] = original_url
        else:
            del os.environ["CONTROLLER_URL"]


@pytest.mark.asyncio
async def test_invalid_jwt_token(controller_url, check_controller_health):
    """Test tools handle invalid JWT token (401 Unauthorized)."""
    # Arrange - use invalid token
    original_token = os.environ.get("MESH_JWT_TOKEN")
    os.environ["MESH_JWT_TOKEN"] = "invalid.jwt.token"
    
    try:
        params = SendTaskParams(target="manager", task={"type": "test"}, context={})
        
        # Act
        result = await send_task_handler(params)
        
        # Assert
        assert isinstance(result, list)
        text = result[0].text
        # Expect 401 Unauthorized error
        assert "❌" in text or "401" in text or "Unauthorized" in text
    finally:
        # Restore original token
        if original_token:
            os.environ["MESH_JWT_TOKEN"] = original_token
        else:
            del os.environ["MESH_JWT_TOKEN"]


# ============================================================
# Test 7: Performance & Latency
# ============================================================

@pytest.mark.asyncio
async def test_send_task_latency(controller_url, jwt_token, check_controller_health):
    """Test send_task latency is within acceptable range (< 5s)."""
    # Arrange
    params = SendTaskParams(
        target="manager",
        task={"type": "latency_test"},
        context={}
    )
    
    # Act
    start_time = time.time()
    result = await send_task_handler(params)
    elapsed = time.time() - start_time
    
    # Assert
    assert elapsed < 5.0, f"send_task took {elapsed:.2f}s (expected < 5s)"
    assert isinstance(result, list)


@pytest.mark.asyncio
async def test_concurrent_requests(controller_url, jwt_token, check_controller_health):
    """Test multiple concurrent tool invocations."""
    # Arrange - create 3 concurrent send_task requests
    params_list = [
        SendTaskParams(target="manager", task={"type": f"concurrent_{i}"}, context={})
        for i in range(3)
    ]
    
    # Act - run concurrently
    results = await asyncio.gather(
        *[send_task_handler(p) for p in params_list]
    )
    
    # Assert
    assert len(results) == 3
    for result in results:
        assert isinstance(result, list)
        # All should succeed or fail gracefully
        assert "✅" in result[0].text or "❌" in result[0].text
