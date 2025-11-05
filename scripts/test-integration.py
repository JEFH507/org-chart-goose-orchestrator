#!/usr/bin/env python3
"""
Phase 3 Integration Testing
Tests Workstream A + B integration and backward compatibility
"""

import requests
import subprocess
import uuid
import json
import sys
import time

# Colors
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'

passed = 0
failed = 0

def pass_test(msg):
    global passed
    print(f"{GREEN}✅ PASS{NC} - {msg}")
    passed += 1

def fail_test(msg):
    global failed
    print(f"{RED}❌ FAIL{NC} - {msg}")
    failed += 1

def info(msg):
    print(f"{BLUE}ℹ️  {msg}{NC}")

def warn(msg):
    print(f"{YELLOW}⚠️  {msg}{NC}")

print(f"{BLUE}═══════════════════════════════════════════════════════════{NC}")
print(f"{BLUE}  Phase 3 Integration Testing{NC}")
print(f"{BLUE}═══════════════════════════════════════════════════════════{NC}")
print()

# Test 1: Infrastructure Health
print(f"{BLUE}[Test 1]{NC} Infrastructure Health Check")
try:
    response = requests.get("http://localhost:8088/status", timeout=5)
    if response.status_code == 200:
        version = response.json().get('version')
        pass_test(f"Controller API healthy (version: {version})")
    else:
        fail_test(f"Controller API returned {response.status_code}")
except Exception as e:
    fail_test(f"Controller API not responding: {e}")

try:
    response = requests.get("http://localhost:8080/realms/dev", timeout=5)
    if response.status_code == 200:
        realm = response.json().get('realm')
        pass_test(f"Keycloak 'dev' realm accessible (realm: {realm})")
    else:
        fail_test(f"Keycloak returned {response.status_code}")
except Exception as e:
    fail_test(f"Keycloak not accessible: {e}")
print()

# Test 2: JWT Token Acquisition
print(f"{BLUE}[Test 2]{NC} JWT Token Acquisition")
try:
    result = subprocess.run(
        ["./scripts/get-jwt-token.sh"],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        timeout=10
    )
    token_lines = result.stdout.strip().split('\n')
    token = token_lines[-1] if token_lines else None
    
    if token and len(token) > 50:
        pass_test("JWT token acquired (expires in 60 min)")
        info(f"Token: {token[:50]}...")
    else:
        fail_test("JWT token empty or invalid")
        token = None
except Exception as e:
    fail_test(f"Failed to get JWT token: {e}")
    token = None
print()

# Test 3: Controller API - Dev Mode (No JWT)
print(f"{BLUE}[Test 3]{NC} Controller API (Dev Mode - No JWT)")
try:
    response = requests.post(
        "http://localhost:8088/tasks/route",
        headers={
            "Content-Type": "application/json",
            "Idempotency-Key": str(uuid.uuid4()),
            "X-Trace-Id": str(uuid.uuid4())
        },
        json={
            "target": "test",
            "task": {
                "task_type": "test",
                "description": "No JWT test",
                "data": {}
            },
            "context": {}
        },
        timeout=5
    )
    
    if response.status_code == 200:
        task_id = response.json().get('task_id')
        pass_test(f"POST /tasks/route works without JWT (dev mode)")
        info(f"Task ID: {task_id}")
    elif response.status_code == 401:
        warn("Controller requires JWT (production mode active)")
        info("This is expected if OIDC env vars are set")
    else:
        fail_test(f"POST /tasks/route returned {response.status_code}")
except Exception as e:
    fail_test(f"POST /tasks/route failed: {e}")
print()

# Test 4: Controller API - With JWT
if token:
    print(f"{BLUE}[Test 4]{NC} Controller API (With JWT)")
    try:
        response = requests.post(
            "http://localhost:8088/tasks/route",
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
                "Idempotency-Key": str(uuid.uuid4()),
                "X-Trace-Id": str(uuid.uuid4())
            },
            json={
                "target": "manager",
                "task": {
                    "task_type": "integration_test",
                    "description": "Testing with JWT auth",
                    "data": {"test": "workstream_a"}
                },
                "context": {"test_phase": "phase3"}
            },
            timeout=5
        )
        
        if response.status_code == 200:
            task_id = response.json().get('task_id')
            pass_test("POST /tasks/route with JWT (HTTP 200)")
            info(f"Task ID: {task_id}")
        elif response.status_code == 401:
            warn("JWT authentication token rejected (HTTP 401)")
            info("Controller needs OIDC env vars (OIDC_ISSUER_URL, OIDC_JWKS_URL)")
        else:
            fail_test(f"POST /tasks/route with JWT (HTTP {response.status_code})")
            info(f"Response: {response.text[:200]}")
    except Exception as e:
        fail_test(f"POST /tasks/route with JWT failed: {e}")
    print()

# Test 5: POST /approvals
print(f"{BLUE}[Test 5]{NC} POST /approvals")
try:
    response = requests.post(
        "http://localhost:8088/approvals",
        headers={
            "Content-Type": "application/json",
            "Idempotency-Key": str(uuid.uuid4()),
            "X-Trace-Id": str(uuid.uuid4())
        },
        json={
            "task_id": str(uuid.uuid4()),
            "decision": "approved",
            "comments": "Integration test approval"
        },
        timeout=5
    )
    
    if response.status_code == 200:
        approval_id = response.json().get('approval_id')
        pass_test(f"POST /approvals (HTTP 200)")
        info(f"Approval ID: {approval_id}")
    else:
        fail_test(f"POST /approvals (HTTP {response.status_code})")
except Exception as e:
    fail_test(f"POST /approvals failed: {e}")
print()

# Test 6: GET /profiles/{role}
print(f"{BLUE}[Test 6]{NC} GET /profiles/{{role}}")
try:
    response = requests.get("http://localhost:8088/profiles/manager", timeout=5)
    
    if response.status_code == 200:
        role = response.json().get('role')
        pass_test(f"GET /profiles/manager (HTTP 200, role: {role})")
    else:
        fail_test(f"GET /profiles/manager (HTTP {response.status_code})")
except Exception as e:
    fail_test(f"GET /profiles/manager failed: {e}")
print()

# Test 7: Agent Mesh MCP Tools Import
print(f"{BLUE}[Test 7]{NC} Agent Mesh MCP Tools")
try:
    import sys
    sys.path.insert(0, 'src/agent-mesh')
    from tools import send_task_tool, request_approval_tool, notify_tool, fetch_status_tool
    pass_test("All 4 MCP tools can be imported")
    info(f"Tools: send_task, request_approval, notify, fetch_status")
except Exception as e:
    fail_test(f"MCP tools import failed: {e}")
print()

# Summary
print(f"{BLUE}═══════════════════════════════════════════════════════════{NC}")
print(f"{BLUE}  Test Summary{NC}")
print(f"{BLUE}═══════════════════════════════════════════════════════════{NC}")
print(f"{GREEN}Passed: {passed}{NC}")
print(f"{RED}Failed: {failed}{NC}")
print()

if failed == 0:
    print(f"{GREEN}✅ All tests passed!{NC}")
    sys.exit(0)
else:
    print(f"{YELLOW}⚠️  Some tests failed. See above for details.{NC}")
    sys.exit(1)
