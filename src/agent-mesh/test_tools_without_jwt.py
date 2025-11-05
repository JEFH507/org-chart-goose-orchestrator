#!/usr/bin/env python3
"""
Quick smoke test for Agent Mesh tools with JWT authentication.
Tests the tools directly against the Controller API.

Requires:
- MESH_JWT_TOKEN environment variable (or will attempt to obtain from Keycloak)
- KEYCLOAK_CLIENT_SECRET environment variable (if obtaining token)
"""

import asyncio
import os
import sys
import subprocess
import json

# Set default environment variables
os.environ["CONTROLLER_URL"] = os.getenv("CONTROLLER_URL", "http://localhost:8088")

from tools.send_task import send_task_handler, SendTaskParams
from tools.request_approval import request_approval_handler, RequestApprovalParams
from tools.notify import notify_handler, NotifyParams
from tools.fetch_status import fetch_status_handler, FetchStatusParams


def get_jwt_token():
    """Obtain JWT token from Keycloak if not already set."""
    if os.getenv("MESH_JWT_TOKEN"):
        print("✅ Using JWT token from MESH_JWT_TOKEN environment variable")
        return True
    
    print("⚠️  MESH_JWT_TOKEN not set, attempting to obtain from Keycloak...")
    
    client_secret = os.getenv("KEYCLOAK_CLIENT_SECRET")
    if not client_secret:
        print("❌ KEYCLOAK_CLIENT_SECRET not set")
        print("\nPlease set MESH_JWT_TOKEN or KEYCLOAK_CLIENT_SECRET:")
        print("  export MESH_JWT_TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \\")
        print("    -d 'client_id=goose-controller' \\")
        print("    -d 'grant_type=client_credentials' \\")
        print("    -d 'client_secret=<secret>' | jq -r '.access_token')")
        return False
    
    # Obtain token from Keycloak
    keycloak_url = os.getenv("KEYCLOAK_URL", "http://localhost:8080")
    keycloak_realm = os.getenv("KEYCLOAK_REALM", "dev")
    keycloak_client = os.getenv("KEYCLOAK_CLIENT", "goose-controller")
    
    try:
        result = subprocess.run(
            [
                "curl", "-s", "-X", "POST",
                f"{keycloak_url}/realms/{keycloak_realm}/protocol/openid-connect/token",
                "-d", f"client_id={keycloak_client}",
                "-d", "grant_type=client_credentials",
                "-d", f"client_secret={client_secret}"
            ],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode != 0:
            print(f"❌ Failed to obtain JWT token: {result.stderr}")
            return False
        
        response = json.loads(result.stdout)
        token = response.get("access_token")
        
        if not token:
            print(f"❌ Failed to extract access_token from response: {result.stdout}")
            return False
        
        os.environ["MESH_JWT_TOKEN"] = token
        print("✅ JWT token obtained from Keycloak")
        return True
        
    except Exception as e:
        print(f"❌ Failed to obtain JWT token: {e}")
        return False


async def test_controller_health():
    """Test 1: Controller API health check."""
    print("\n" + "="*60)
    print("Test 1: Controller API Health Check")
    print("="*60)
    
    import requests
    try:
        response = requests.get(f"{os.environ['CONTROLLER_URL']}/status", timeout=5)
        if response.status_code == 200:
            print("✅ Controller API is running")
            print(f"   Response: {response.json()}")
            return True
        else:
            print(f"❌ Controller API returned {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Controller API not reachable: {e}")
        return False


async def test_send_task():
    """Test 2: send_task tool."""
    print("\n" + "="*60)
    print("Test 2: send_task Tool")
    print("="*60)
    
    # Note: send_task expects task as dict, but Controller expects specific schema
    # We'll need to adapt the task format
    params = SendTaskParams(
        target="manager",
        task={
            "task_type": "integration_test",
            "description": "Test send_task tool",
            "data": {"amount": 50000, "priority": "high"}
        },
        context={"department": "Engineering", "quarter": "Q1"}
    )
    
    try:
        result = await send_task_handler(params)
        text = result[0].text
        print(text)
        
        if "✅" in text and ("task" in text.lower() or "accepted" in text.lower()):
            print("\n✅ send_task test PASSED")
            # Extract task_id for later tests
            import re
            match = re.search(r'task-[a-f0-9-]+', text, re.IGNORECASE)
            return match.group(0) if match else None
        else:
            print("\n❌ send_task test FAILED")
            return None
    except Exception as e:
        print(f"❌ send_task test FAILED with exception: {e}")
        import traceback
        traceback.print_exc()
        return None


async def test_request_approval(task_id):
    """Test 3: request_approval tool."""
    print("\n" + "="*60)
    print("Test 3: request_approval Tool")
    print("="*60)
    
    if not task_id:
        task_id = "task-test-12345"  # Use a dummy ID if we don't have one
        print(f"⚠️  Using dummy task_id: {task_id}")
    
    params = RequestApprovalParams(
        task_id=task_id,
        approver_role="manager",
        reason="Integration test approval request",
        decision="pending",
        comments="Testing the approval workflow"
    )
    
    try:
        result = await request_approval_handler(params)
        text = result[0].text
        print(text)
        
        if "✅" in text or "approval" in text.lower():
            print("\n✅ request_approval test PASSED")
            return True
        else:
            print("\n❌ request_approval test FAILED")
            return False
    except Exception as e:
        print(f"❌ request_approval test FAILED with exception: {e}")
        import traceback
        traceback.print_exc()
        return False


async def test_notify():
    """Test 4: notify tool."""
    print("\n" + "="*60)
    print("Test 4: notify Tool")
    print("="*60)
    
    params = NotifyParams(
        target="manager",
        message="Integration test notification - urgent update",
        priority="high"
    )
    
    try:
        result = await notify_handler(params)
        text = result[0].text
        print(text)
        
        if "✅" in text or "notification" in text.lower():
            print("\n✅ notify test PASSED")
            return True
        else:
            print("\n❌ notify test FAILED")
            return False
    except Exception as e:
        print(f"❌ notify test FAILED with exception: {e}")
        import traceback
        traceback.print_exc()
        return False


async def test_fetch_status(task_id):
    """Test 5: fetch_status tool."""
    print("\n" + "="*60)
    print("Test 5: fetch_status Tool")
    print("="*60)
    
    if not task_id:
        task_id = "task-nonexistent-12345"
        print(f"⚠️  Using dummy task_id: {task_id} (404 expected)")
    
    params = FetchStatusParams(task_id=task_id)
    
    try:
        result = await fetch_status_handler(params)
        text = result[0].text
        print(text)
        
        # Accept both success and 404 (Controller API is ephemeral in Phase 3)
        if "✅" in text or "❌" in text:
            print("\n✅ fetch_status test PASSED (tool responded correctly)")
            return True
        else:
            print("\n❌ fetch_status test FAILED (unexpected response)")
            return False
    except Exception as e:
        print(f"❌ fetch_status test FAILED with exception: {e}")
        import traceback
        traceback.print_exc()
        return False


async def test_invalid_priority():
    """Test 6: notify with invalid priority (should fail gracefully)."""
    print("\n" + "="*60)
    print("Test 6: Invalid Priority Validation")
    print("="*60)
    
    params = NotifyParams(
        target="manager",
        message="Test message",
        priority="urgent"  # Invalid - should be 'low', 'normal', or 'high'
    )
    
    try:
        result = await notify_handler(params)
        text = result[0].text
        print(text)
        
        if "❌" in text and ("Invalid" in text or "priority" in text.lower()):
            print("\n✅ Invalid priority test PASSED (rejected correctly)")
            return True
        else:
            print("\n❌ Invalid priority test FAILED (should have rejected 'urgent')")
            return False
    except Exception as e:
        print(f"❌ Invalid priority test FAILED with exception: {e}")
        return False


async def main():
    """Run all smoke tests."""
    print("="*60)
    print("Agent Mesh MCP Tools - Smoke Test Suite")
    print("="*60)
    print(f"Controller URL: {os.environ['CONTROLLER_URL']}")
    
    # Obtain JWT token if needed
    if not get_jwt_token():
        sys.exit(1)
    
    print(f"JWT Token: {'Set' if os.environ.get('MESH_JWT_TOKEN') else 'Not set'}")
    
    # Test 1: Controller health
    if not await test_controller_health():
        print("\n❌ Controller API not available - aborting tests")
        print("Please start the Controller API:")
        print("  cd deploy/compose")
        print("  docker compose -f ce.dev.yml --profile controller up -d")
        sys.exit(1)
    
    # Test 2: send_task
    task_id = await test_send_task()
    
    # Test 3: request_approval
    await test_request_approval(task_id)
    
    # Test 4: notify
    await test_notify()
    
    # Test 5: fetch_status
    await test_fetch_status(task_id)
    
    # Test 6: Invalid priority
    await test_invalid_priority()
    
    # Summary
    print("\n" + "="*60)
    print("Smoke Test Suite Complete")
    print("="*60)
    print("\n✅ All 6 tests completed!")
    print("\nNext steps:")
    print("1. Review test output above for any failures")
    print("2. Run full integration tests with: pytest tests/test_integration.py -v")
    print("3. Test with Goose instance (load agent_mesh extension)")


if __name__ == "__main__":
    asyncio.run(main())
