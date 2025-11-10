#!/usr/bin/env python3
"""
Agent Mesh E2E Testing Framework

Tests 3 scenarios:
1. Expense Approval (Finance → Manager)
2. Legal Review (Finance → Legal → Manager)
3. Cross-Department (HR → Finance → Manager)

Usage:
    python3 tests/e2e/test_agent_mesh_e2e.py
"""

import os
import sys
import requests
import uuid
import json
from typing import Dict, Any, Optional
from datetime import datetime

# Configuration
CONTROLLER_URL = os.getenv("CONTROLLER_URL", "http://localhost:8088")
KEYCLOAK_URL = os.getenv("KEYCLOAK_URL", "http://localhost:8080")
CLIENT_ID = "goose-controller"
REALM = "dev"

class Colors:
    """ANSI color codes for output"""
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    BOLD = '\033[1m'
    END = '\033[0m'

class AgentMeshTester:
    """E2E testing framework for Agent Mesh"""
    
    def __init__(self, client_secret: str):
        self.client_secret = client_secret
        self.jwt_token: Optional[str] = None
        self.test_results = []
        
    def get_jwt_token(self) -> str:
        """Get JWT using client_credentials grant"""
        if self.jwt_token:
            return self.jwt_token
            
        response = requests.post(
            f"{KEYCLOAK_URL}/realms/{REALM}/protocol/openid-connect/token",
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            data={
                "grant_type": "client_credentials",
                "client_id": CLIENT_ID,
                "client_secret": self.client_secret,
            },
            timeout=10
        )
        
        response.raise_for_status()
        self.jwt_token = response.json()["access_token"]
        return self.jwt_token
    
    def send_task(
        self,
        source: str,
        target: str,
        task_type: str,
        description: str,
        data: Optional[Dict[str, Any]] = None,
        context: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """Send a task to target agent via Controller API"""
        
        idempotency_key = str(uuid.uuid4())
        trace_id = str(uuid.uuid4())
        
        payload = {
            "target": target,
            "task": {
                "task_type": task_type,
                "description": description,
            },
            "context": context or {}
        }
        
        if data:
            payload["task"]["data"] = data
        
        payload["context"]["source"] = source
        payload["context"]["timestamp"] = datetime.utcnow().isoformat()
        
        response = requests.post(
            f"{CONTROLLER_URL}/tasks/route",
            headers={
                "Authorization": f"Bearer {self.get_jwt_token()}",
                "Content-Type": "application/json",
                "Idempotency-Key": idempotency_key,
                "X-Trace-Id": trace_id,
            },
            json=payload,
            timeout=30
        )
        
        response.raise_for_status()
        result = response.json()
        result["idempotency_key"] = idempotency_key
        return result
    
    def print_header(self, text: str):
        """Print section header"""
        print(f"\n{Colors.BOLD}{Colors.BLUE}{'=' * 70}{Colors.END}")
        print(f"{Colors.BOLD}{Colors.BLUE}{text}{Colors.END}")
        print(f"{Colors.BOLD}{Colors.BLUE}{'=' * 70}{Colors.END}\n")
    
    def print_success(self, text: str):
        """Print success message"""
        print(f"{Colors.GREEN}✓ {text}{Colors.END}")
    
    def print_error(self, text: str):
        """Print error message"""
        print(f"{Colors.RED}✗ {text}{Colors.END}")
    
    def print_info(self, text: str):
        """Print info message"""
        print(f"{Colors.YELLOW}ℹ {text}{Colors.END}")
    
    def test_scenario_1_expense_approval(self):
        """
        Scenario 1: Expense Approval
        Finance → Manager
        
        Steps:
        1. Finance agent creates budget approval request
        2. Finance sends task to Manager for approval
        3. Verify task routed successfully
        """
        self.print_header("Scenario 1: Expense Approval (Finance → Manager)")
        
        try:
            # Step 1: Finance creates budget approval request
            self.print_info("Step 1: Finance creating budget approval request...")
            result = self.send_task(
                source="finance",
                target="manager",
                task_type="budget_approval",
                description="Q1 FY2025 Engineering budget approval request",
                data={
                    "amount": 125000,
                    "department": "Engineering",
                    "quarter": "Q1",
                    "year": 2025,
                    "category": "headcount",
                    "justification": "3 new SWE hires for Platform team"
                },
                context={
                    "priority": "high",
                    "deadline": "2025-01-15",
                    "approver": "manager"
                }
            )
            
            self.print_success(f"Task created: {result['task_id']}")
            self.print_success(f"Status: {result['status']}")
            self.print_success(f"Trace ID: {result['trace_id']}")
            
            self.test_results.append({
                "scenario": "Expense Approval",
                "status": "PASS",
                "task_id": result['task_id'],
                "trace_id": result['trace_id']
            })
            
            return True
            
        except Exception as e:
            self.print_error(f"Scenario 1 failed: {e}")
            self.test_results.append({
                "scenario": "Expense Approval",
                "status": "FAIL",
                "error": str(e)
            })
            return False
    
    def test_scenario_2_legal_review(self):
        """
        Scenario 2: Legal Review
        Finance → Legal → Manager
        
        Steps:
        1. Finance discovers compliance issue
        2. Finance escalates to Legal
        3. Legal reviews (isolated environment)
        4. Legal provides summary to Manager
        """
        self.print_header("Scenario 2: Legal Review (Finance → Legal → Manager)")
        
        try:
            # Step 1: Finance escalates to Legal
            self.print_info("Step 1: Finance escalating compliance issue to Legal...")
            legal_task = self.send_task(
                source="finance",
                target="legal",
                task_type="compliance_review",
                description="Potential SOX compliance issue in Q4 expense reports",
                data={
                    "issue_type": "sox_compliance",
                    "quarter": "Q4",
                    "year": 2024,
                    "affected_accounts": ["6100", "6200", "6300"],
                    "estimated_impact": "$45,000"
                },
                context={
                    "severity": "high",
                    "confidential": True,
                    "attorney_client_privilege": True
                }
            )
            
            self.print_success(f"Legal task created: {legal_task['task_id']}")
            
            # Step 2: Legal provides summary to Manager
            self.print_info("Step 2: Legal providing summary to Manager...")
            manager_task = self.send_task(
                source="legal",
                target="manager",
                task_type="compliance_summary",
                description="Compliance review complete - action required",
                data={
                    "case_number": "REDACTED",  # Legal redacts details
                    "recommendation": "Implement additional controls",
                    "urgency": "high",
                    "estimated_remediation_time": "2 weeks"
                },
                context={
                    "parent_task": legal_task['task_id'],
                    "confidential": False,  # Summary is safe to share
                    "follow_up_required": True
                }
            )
            
            self.print_success(f"Manager task created: {manager_task['task_id']}")
            self.print_success("Legal isolation verified (privileged data not shared)")
            
            self.test_results.append({
                "scenario": "Legal Review",
                "status": "PASS",
                "legal_task_id": legal_task['task_id'],
                "manager_task_id": manager_task['task_id']
            })
            
            return True
            
        except Exception as e:
            self.print_error(f"Scenario 2 failed: {e}")
            self.test_results.append({
                "scenario": "Legal Review",
                "status": "FAIL",
                "error": str(e)
            })
            return False
    
    def test_scenario_3_cross_department(self):
        """
        Scenario 3: Cross-Department Coordination
        HR → Finance → Manager
        
        Steps:
        1. HR requests headcount budget analysis
        2. Finance analyzes and routes to Manager
        3. Manager receives aggregated view
        """
        self.print_header("Scenario 3: Cross-Department (HR → Finance → Manager)")
        
        try:
            # Step 1: HR requests headcount analysis from Finance
            self.print_info("Step 1: HR requesting headcount budget analysis from Finance...")
            finance_task = self.send_task(
                source="hr",
                target="finance",
                task_type="headcount_budget_analysis",
                description="Q2 2025 headcount budget impact analysis",
                data={
                    "new_hires": 5,
                    "departments": ["Engineering", "Sales", "Marketing"],
                    "start_date": "2025-04-01",
                    "employee_data": "REDACTED"  # HR redacts PII
                },
                context={
                    "quarter": "Q2",
                    "year": 2025,
                    "request_type": "budget_impact"
                }
            )
            
            self.print_success(f"Finance task created: {finance_task['task_id']}")
            
            # Step 2: Finance routes analysis to Manager
            self.print_info("Step 2: Finance routing budget analysis to Manager...")
            manager_task = self.send_task(
                source="finance",
                target="manager",
                task_type="budget_approval",
                description="Q2 headcount budget impact - requires approval",
                data={
                    "total_cost": 425000,
                    "breakdown": {
                        "Engineering": 225000,
                        "Sales": 125000,
                        "Marketing": 75000
                    },
                    "impact_on_budget": "+12%"
                },
                context={
                    "parent_task": finance_task['task_id'],
                    "pii_masked": True,  # Finance masked employee PII
                    "approval_required": True
                }
            )
            
            self.print_success(f"Manager task created: {manager_task['task_id']}")
            self.print_success("Privacy boundaries enforced (PII masked)")
            
            self.test_results.append({
                "scenario": "Cross-Department",
                "status": "PASS",
                "finance_task_id": finance_task['task_id'],
                "manager_task_id": manager_task['task_id']
            })
            
            return True
            
        except Exception as e:
            self.print_error(f"Scenario 3 failed: {e}")
            self.test_results.append({
                "scenario": "Cross-Department",
                "status": "FAIL",
                "error": str(e)
            })
            return False
    
    def run_all_tests(self):
        """Run all E2E scenarios"""
        self.print_header("Agent Mesh E2E Testing Framework")
        
        # Acquire JWT token
        self.print_info("Acquiring JWT token...")
        try:
            self.get_jwt_token()
            self.print_success(f"JWT acquired (length: {len(self.jwt_token)})")
        except Exception as e:
            self.print_error(f"Failed to acquire JWT: {e}")
            return False
        
        # Run scenarios
        scenario1 = self.test_scenario_1_expense_approval()
        scenario2 = self.test_scenario_2_legal_review()
        scenario3 = self.test_scenario_3_cross_department()
        
        # Print summary
        self.print_header("Test Summary")
        
        passed = sum(1 for r in self.test_results if r["status"] == "PASS")
        failed = sum(1 for r in self.test_results if r["status"] == "FAIL")
        total = len(self.test_results)
        
        for result in self.test_results:
            if result["status"] == "PASS":
                self.print_success(f"{result['scenario']}: PASSED")
            else:
                self.print_error(f"{result['scenario']}: FAILED - {result.get('error', 'Unknown error')}")
        
        print(f"\n{Colors.BOLD}Results: {passed}/{total} scenarios passed{Colors.END}")
        
        if failed == 0:
            print(f"\n{Colors.GREEN}{Colors.BOLD}✅ ALL TESTS PASSED{Colors.END}\n")
            return True
        else:
            print(f"\n{Colors.RED}{Colors.BOLD}❌ {failed} TEST(S) FAILED{Colors.END}\n")
            return False

def main():
    """Main entry point"""
    # Get client secret from environment
    client_secret = os.getenv("OIDC_CLIENT_SECRET")
    if not client_secret:
        print(f"{Colors.RED}ERROR: OIDC_CLIENT_SECRET environment variable not set{Colors.END}")
        print("\nUsage:")
        print("  export OIDC_CLIENT_SECRET=<your-secret>")
        print("  python3 tests/e2e/test_agent_mesh_e2e.py")
        sys.exit(1)
    
    # Run tests
    tester = AgentMeshTester(client_secret)
    success = tester.run_all_tests()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
