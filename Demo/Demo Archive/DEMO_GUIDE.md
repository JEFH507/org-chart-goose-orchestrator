# 🎯 goose Orchestrator - Demo Guide

Follow this steps first:
[[Container_Management_Playbook]]
[[Demo_Execution_Plan]]

---

## System Architecture Overview

### Components Explained

#### **Keycloak** - Identity & Access Management (IAM)
- **Purpose**: Issues JWT tokens for service-to-service authentication
- **Location**: http://localhost:8080
- **Configuration**:
  - Realm: `dev`
  - Client: `goose-controller`
  - Grant Type: `client_credentials` (OAuth2)
  - Token Lifetime: 10 hours (36000 seconds)
- **Usage in System**:
  - goose instances authenticate to Controller
  - Admin dashboard authenticates for CSV upload & management APIs
  - Privacy Guard proxies authenticate for configuration sync
  - Agent Mesh uses JWT for cross-agent task routing

#### **Vault** - Secrets & Cryptography Management
- **Purpose**: Stores secrets, signs profiles, manages encryption keys
- **Location**: https://localhost:8200
- **Features Used**:
  - **AppRole Auth**: Controller authenticates via role_id/secret_id (1 hr life span, not yet auto-renew)
  - VAULT_TOKEN: Design to be a fall back, current logic on Dev mode is before the AppRole Auth. (Lifespan is 32 days)
  - **Transit Engine**: Cryptographic signing of profile JSONs
  - **KV Secrets**: Stores service credentials & API keys
  - **Audit Logging**: Tracks all secret access
- **Usage in System**:
  - Profile signatures ensure integrity (detect tampering)
  - Service credentials stored securely
  - Encryption keys for sensitive data at rest
  - Automatic secret rotation capability

#### **Redis** - Caching & Session Management
- **Purpose**: Fast in-memory cache for session state & task queues
- **Location**: localhost:6379
- **Usage in System**:
  - Idempotency key tracking (prevent duplicate task execution)
  - Session state caching for goose instances
  - Task queue for Agent Mesh communication
  - Profile cache to reduce database queries
  - Real-time log buffering

#### **PostgreSQL** - Persistent Data Storage
- **Purpose**: Main database for all system data
- **UI**: pgAdmin4
- **Location**: localhost:5432
- **Database**: `orchestrator`
- **Schema**:
  - `org_users`: Organization chart (50 users)
  - `profiles`: Role-based configuration profiles (8 profiles)
  - `tasks`: Agent Mesh task persistence
  - `sessions`: goose session history
  - `audit_log`: Privacy Guard activity logs

#### **Controller** - Central Orchestration Service
- **Purpose**: Coordinates all goose instances, routes tasks, manages profiles
- **Location**: http://localhost:8088
- **Responsibilities**:
  - Profile distribution to goose instances
  - Agent Mesh task routing
  - User-to-profile assignment
  - Privacy Guard proxy coordination
  - Admin dashboard backend

#### **Privacy Guard Proxies** - PII Detection & Filtering
- **Purpose**: Real-time detection and redaction of sensitive data
- **Testing Instances**: 8 proxies (one per profile role)
- **Locations**:
  - Finance: http://localhost:8096
  - Manager: http://localhost:8097
  - Legal: http://localhost:8098
  - HR: http://localhost:8099
  - Analyst: http://localhost:8100
  - Developer: http://localhost:8101
  - Marketing: http://localhost:8102
  - Support: http://localhost:8103
- **Features**:
  - Pattern-based PII detection (regex)
  - LLM-based semantic detection
  - Configurable privacy modes: bypass/service/strict
  - Real-time audit logging

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          ADMIN INTERFACE                                │
│                     http://localhost:8088/admin                         │
│  ┌──────────────┬──────────────┬──────────────┬─────────────────────┐  │
│  │ User Mgmt    │ Profile Edit │ CSV Upload   │ Config Push         │  │
│  └──────────────┴──────────────┴──────────────┴─────────────────────┘  │
└─────────────────────────────┬───────────────────────────────────────────┘
                              │ JWT Auth
                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          KEYCLOAK (IAM)                                 │
│                     http://localhost:8080                               │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ JWT Token Issuance │ OAuth2 Client Credentials │ 10hr Lifetime   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────┬───────────────────────────────────────────┘
                              │ Tokens
                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       CONTROLLER SERVICE                                │
│                     http://localhost:8088                               │
│  ┌────────────────────┬──────────────────┬──────────────────────────┐  │
│  │ Profile Manager    │ Agent Mesh Router│ Session Manager          │  │
│  └────────────────────┴──────────────────┴──────────────────────────┘  │
└──┬──────────┬──────────┬──────────┬──────────────────────────────────┬─┘
   │          │          │          │                                  │
   │ Vault    │ Redis    │ Postgres │ Privacy Guard Config             │
   ▼          ▼          ▼          ▼                                  │
┌──────┐ ┌────────┐ ┌──────────┐ ┌──────────────────────────────────┐ │
│Vault │ │ Redis  │ │PostgreSQL│ │    Privacy Guard Proxies (8)     │ │
│:8200 │ │ :6379  │ │ :5432    │ │ Finance│Manager│Legal│HR│etc...   │ │
└──────┘ └────────┘ └──────────┘ └──────────────────────────────────┘ │
   │                      │                        │                   │
   │Profile Signatures    │Org Users, Profiles     │PII Detection      │
   │Secret Storage        │Tasks, Sessions         │Audit Logs         │
   │                      │                        │                   │
   └──────────────────────┴────────────────────────┴───────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        GOOSE INSTANCES (6)                              │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │Finance 1 │Finance 2 │Manager 1 │Manager 2 │Legal 1   │Legal 2   │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘  │
│                                                                         │
│  Each Instance:                                                         │
│  • Connects to assigned Privacy Guard Proxy                            │
│  • Receives profile config from Controller                             │
│  • Participates in Agent Mesh (task routing)                           │
│  • JWT authenticated to all services                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Data Flow Example: User Assigns Profile

```
1. Admin Dashboard
   └─> POST /admin/users/EMP001/assign {"profile": "finance"}
        │ (with JWT token)
        ▼
2. Keycloak
   └─> Validates JWT token, checks expiry
        ▼
3. Controller
   ├─> UPDATE org_users SET assigned_profile='finance' WHERE user_id=1
   │   (PostgreSQL)
   ├─> Cache invalidation for user EMP001
   │   (Redis)
   ├─> Fetch profile 'finance' from database
   ├─> Verify profile signature
   │   (Vault Transit Engine)
   └─> Return success to Admin Dashboard
        ▼
4. Next Login: goose Instance for EMP001
   ├─> GET /profiles/finance (with JWT)
   ├─> Controller returns signed profile JSON
   ├─> goose validates signature (Vault)
   ├─> goose configures itself:
   │   • Privacy mode = strict
   │   • Allowed extensions = [developer, browser]
   │   • Max token limit = 50000
   └─> Connects to Privacy Guard (Finance) on port 8096
```

---

## Demo Flow

### Part 0: Terminal Setup (3 Terminals for goose Instances)

For the demo, you'll have **3 goose terminal windows** showing different roles:

**⚠️ IMPORTANT NOTES:**
- goose containers are in the `multi-goose` docker-compose profile
- Containers fetch profiles from **DATABASE** at startup (via Controller API)
- Container names: `ce_goose_finance`, `ce_goose_manager`, `ce_goose_legal` (no `_1` suffix)
- Command is `goose session` (NOT `goose session start` - fixed in Phase 6)
- Profile changes in Admin UI require container restart to apply

#### Pre-Demo: Start goose Containers
```bash
cd deploy/compose

# Start all goose instance containers with the multi-goose profile
docker compose -f ce.dev.yml --profile multi-goose up -d

# Verify containers are running
docker compose -f ce.dev.yml ps | grep goose

# Check that profiles were fetched successfully
docker compose -f ce.dev.yml logs ce_goose_finance | grep "Profile fetched"
docker compose -f ce.dev.yml logs ce_goose_manager | grep "Profile fetched"
docker compose -f ce.dev.yml logs ce_goose_legal | grep "Profile fetched"
```

**Expected output:**
- ✅ `ce_goose_finance` - Running, healthy
- ✅ `ce_goose_manager` - Running, healthy
- ✅ `ce_goose_legal` - Running, healthy
- ✅ "Profile fetched successfully" in logs for all 3

---

#### Terminal 1: Finance User (Alice)
```bash
# Start interactive goose session for Finance role
docker exec -it ce_goose_finance goose session
```

**Profile Configuration (from database):**
- Role: `finance`
- Privacy Mode: Rules-only (fastest, <10ms)
- Privacy Guard Proxy: http://privacy-guard-proxy-finance:8090 (port 8096 external)
- Extensions: Loaded from `profiles` table (e.g., developer, agent_mesh)
- Detection Method: Pattern-based (regex)

---

#### Terminal 2: Manager User (Bob)
```bash
# Start interactive goose session for Manager role
docker exec -it ce_goose_manager goose session
```

**Profile Configuration (from database):**
- Role: `manager`
- Privacy Mode: Hybrid (balanced, <100ms typical)
- Privacy Guard Proxy: http://privacy-guard-proxy-manager:8090 (port 8097 external)
- Extensions: Loaded from `profiles` table (e.g., developer, agent_mesh)
- Detection Method: Hybrid (rules + LLM fallback)

---

#### Terminal 3: Legal User (Carol)
```bash
# Start interactive goose session for Legal role
docker exec -it ce_goose_legal goose session
```

**Profile Configuration (from database):**
- Role: `legal`
- Privacy Mode: AI-only (most thorough, ~15s)
- Privacy Guard Proxy: http://privacy-guard-proxy-legal:8090 (port 8098 external)
- Extensions: Loaded from `profiles` table (e.g., developer, agent_mesh)
- Detection Method: LLM-based semantic detection

---

#### How Profile Configuration Works

**Database-Driven Configuration:**
1. Admin edits profile in Dashboard → Saves to PostgreSQL `profiles` table
2. goose container starts → Entrypoint script runs
3. Script fetches profile → `curl http://controller:8088/profiles/{role}`
4. Controller queries database → `SELECT role, data FROM profiles WHERE role = ?`
5. Python script generates config → `~/.config/goose/config.yaml` from profile JSON
6. goose loads config → Extensions, privacy settings, policies all from database

**To Apply Profile Changes:**
```bash
# After editing profiles in Admin Dashboard, restart containers:
docker compose -f ce.dev.yml restart ce_goose_finance
docker compose -f ce.dev.yml restart ce_goose_manager
docker compose -f ce.dev.yml restart ce_goose_legal

# Verify new profiles were fetched:
docker compose -f ce.dev.yml logs ce_goose_finance | tail -20 | grep "Profile fetched"
```

**What to Show in Each Terminal:**
- Each goose instance loads with its assigned profile (from database)
- Privacy Guard Proxy connection status
- Extension availability based on profile configuration
- MCP tools (agent_mesh) if enabled in profile
- Token limits and restrictions from profile policies

---

### Part 0.5: Demo Prompts for goose Sessions

Once terminals are set up, use these **demo prompts** to show capability differences:

#### **Finance Terminal Prompts (Alice & David):**

**Prompt 1: Data Analysis with Privacy**
```
Analyze this dataset and create visualizations:
Sales: Q1=$125K, Q2=$142K, Q3=$138K, Q4=$165K
Include my email alice@company.com in the report.
```
**Expected:** Privacy Guard detects email, redacts to `[EMAIL_REDACTED]`

**Prompt 2: Financial Calculation**
```
Calculate ROI for project with:
- Initial investment: $50,000
- Monthly revenue: $8,500
- Operating costs: $3,200/month
Show break-even point.
```
**Expected:** Full access to developer extension for calculations

**Prompt 3: Cross-Agent Task (MCP Mesh)**
```
Create a task for the manager to approve budget increase of 15% 
for marketing department. Include justification based on Q4 growth.
```
**Expected:** Task routed to Manager role via Agent Mesh

---

#### **Manager Terminal Prompts (Bob & Frank):**

**Prompt 1: Team Management**
```
List all employees in Finance department from the org chart.
Show their roles and reporting structure.
```
**Expected:** Queries org_users table, shows hierarchy

**Prompt 2: Approval Workflow**
```
Review pending tasks assigned to manager role.
Show task ID, priority, and requester.
```
**Expected:** Retrieves tasks from Agent Mesh queue

**Prompt 3: Report Generation**
```
Create a summary report of Q4 performance across all departments.
Include employee count, budget utilization, and key metrics.
```
**Expected:** Aggregates data from multiple sources, filters PII

---

#### **Legal Terminal Prompts (Carol & Grace):**

**Prompt 1: Document Analysis with Strict Privacy**
```
Review this contract excerpt:
"Party A (John Smith, SSN 123-45-6789) agrees to terms..."
Identify privacy concerns.
```
**Expected:** Privacy Guard detects SSN, redacts to `[SSN_REDACTED]`

**Prompt 2: Compliance Check**
```
Check if our data handling practices comply with GDPR Article 32.
Review current privacy settings and audit logs.
```
**Expected:** Queries Privacy Guard audit logs, shows compliance status

**Prompt 3: Cross-Role Coordination**
```
Request financial data from Finance team for audit purposes.
Include case ID: AUDIT-2025-001
```
**Expected:** Creates task routed to Finance role, logs audit trail

---

### Part 0.6: MCP Mesh Communication Test

**Demonstrate Agent Mesh (MCP) cross-role task routing:**

#### Step 1: Finance Creates Task for Manager
In **Finance Terminal (Alice)**:
```
Create a task for the manager:
Task ID: BUDGET-APPROVAL-Q1-2025
Priority: High
Description: Approve 12% budget increase for IT infrastructure upgrade.
Justification: Growing security requirements and team expansion.
Amount: $75,000
```

#### Step 2: Verify Task in Controller
In **separate terminal**:
```bash
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" \
  | jq -r '.access_token')

curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/tasks/BUDGET-APPROVAL-Q1-2025 | jq '.'
```

**Expected output:**
```json
{
  "task_id": "BUDGET-APPROVAL-Q1-2025",
  "task_type": "approval",
  "source_role": "finance",
  "target_role": "manager",
  "priority": "high",
  "status": "pending",
  "content": {
    "description": "Approve 12% budget increase for IT infrastructure upgrade",
    "amount": 75000,
    "justification": "Growing security requirements and team expansion"
  },
  "created_at": "2025-01-11T19:30:00Z",
  "created_by": "alice@company.com"
}
```

#### Step 3: Manager Retrieves Task
In **Manager Terminal (Bob)**:
```
Show me all pending approval tasks assigned to my role.
```

**Expected:** Bob sees the budget approval task created by Alice

#### Step 4: Manager Responds to Task
In **Manager Terminal (Bob)**:
```
Approve task BUDGET-APPROVAL-Q1-2025 with comment:
"Approved. Please coordinate with procurement for vendor selection."
```

#### Step 5: Verify Task Update
```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/tasks/BUDGET-APPROVAL-Q1-2025 | jq '.'
```

**Expected output:**
```json
{
  "task_id": "BUDGET-APPROVAL-Q1-2025",
  "status": "approved",
  "approved_by": "bob@company.com",
  "approved_at": "2025-01-11T19:35:00Z",
  "comments": "Approved. Please coordinate with procurement for vendor selection."
}
```

#### Step 6: Legal Audits the Transaction
In **Legal Terminal (Carol)**:
```
Show audit trail for task BUDGET-APPROVAL-Q1-2025.
Include all participants and timestamps.
```

**Expected:** Carol sees full audit log:
- Task created by Alice (Finance) at 19:30
- Task routed to Manager role
- Task approved by Bob (Manager) at 19:35
- All actions logged with JWT authentication details

**What This Demonstrates:**
- ✅ Cross-role task routing (Finance → Manager → Legal)
- ✅ Task persistence in database
- ✅ Idempotency (duplicate requests handled)
- ✅ Full audit trail for compliance
- ✅ JWT-authenticated communication
- ✅ Role-based access control (only authorized roles see tasks)

---

### Part 1: Admin Dashboard Tour

#### Open Admin Dashboard
```
http://localhost:8088/admin
```

#### Quick Links Overview
Show the navigation bar with links to:
- **Keycloak Admin** - Identity & Access Management
- **Vault Dashboard** - Secrets Management (https://localhost:8200/ui/vault/dashboard)
- **API Docs** - OpenAPI Specification
- **Privacy Guard Control Panels** - Finance, Manager, Legal instances

---

### Part 2: CSV Organization Chart Upload

#### Option A: Browser Upload (Recommended)

**Step 1: Get JWT Token**
```bash
./get_admin_token.sh
```

Copy the token displayed.

**Step 2: Set Token in Browser**
1. Open browser Dev Tools (F12)
2. Go to Console tab
3. Paste the localStorage command shown in script output
4. Refresh the page

**Step 3: Upload CSV**
1. Click "Select CSV File" or drag-and-drop
2. Choose `test_data/demo_org_chart.csv`
3. Wait for success message: "✅ Successfully imported! Created: 0, Updated: 50"

#### Option B: Command Line Upload
```bash
./admin_upload_csv.sh test_data/demo_org_chart.csv
```

**What to Show:**
- 50 users loaded into organizational hierarchy
- 3 departments: Finance, Legal, Operations
- 3 role types: finance, manager, legal

---

### Part 3: User Management

After CSV upload, the User Management section shows:
- **50 users** from the org chart
- Employee ID, Name, Email, Department, Role
- Profile assignment dropdown for each user

**Demo Actions:**
1. **Show the full user list** scrolling through all 50 users
2. **Assign profiles** to users:
   - Select "finance" profile for Finance department users
   - Select "manager" profile for managers
   - Select "legal" profile for Legal department users
3. **Explain auto-config**: When a profile is assigned, their goose instance will auto-configure on next login

---

### Part 4: Profile Management

**View Existing Profiles:**
1. Select "Finance" from dropdown
2. Show the JSON profile document with:
   - Privacy guard settings
   - Extensions configuration
   - Policies and rules
   - goose hints

**Edit a Profile:**
1. Modify settings in the editor (e.g., change `privacy_mode` to `"strict"`)
2. Click "Save Profile Changes"
3. Show success message

**Create New Profile:**
1. Enter name: "executive"
2. Click "Create New Profile"
3. Edit the default template
4. Save changes

**Download/Upload:**
- **Download**: Export profile as JSON file
- **Upload**: Import modified profile from file

---

### Part 5: JWT & Vault Demonstration

#### Show JWT Token Usage

**Terminal Demo:**
```bash
# Get a token (valid 10 hours)
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" \
  | jq -r '.access_token')

# Use token for authenticated API call
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/tasks | jq '.'
```

**Explain:**
- JWT tokens secure service-to-service communication
- goose instances use JWT when calling Controller APIs
- Agent Mesh communication protected by JWT
- Tokens auto-rotate (request new before expiry)

#### Show Vault Integration

**Open Vault UI:**
```
https://localhost:8200/ui/vault/dashboard
```

**Login with root token** (from unseal script or .env.ce)

**Demo Points:**
- **AppRole Auth**: Controller authenticates via AppRole
- **Profile Signatures**: Profiles cryptographically signed
- **Transit Engine**: Signature verification for profile integrity
- **Audit Logs**: All secret access logged

---

### Part 6: Privacy Guard Control Panels

**Open Control Panels** (one for each role):
- Finance: http://localhost:8096/ui
- Manager: http://localhost:8097/ui
- Legal: http://localhost:8098/ui

**Demo Privacy Settings:**
1. **Privacy Mode**: Show bypass/service/strict modes
2. **Detection Method**: Pattern/LLM/Hybrid detection
3. **Real-time Filtering**: Show how PII gets redacted
4. **Audit Logs**: Review detected sensitive data

**Show Role-Based Differences:**
- Finance: Strict mode for financial data
- Manager: Balanced mode for operational data
- Legal: Service mode for confidential documents

---

### Part 7: Agent Mesh Communication

**Show Task Persistence:**
```bash
# Create a task through Agent Mesh
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8" \
  | jq -r '.access_token')

curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{
    "target": "manager",
    "task": {
      "task_id": "demo-task-001",
      "task_type": "approval",
      "priority": "high",
      "content": {
        "description": "Review Q4 budget proposal"
      }
    }
  }' | jq '.'

# Retrieve task status
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/tasks/demo-task-001 | jq '.'
```

**Explain:**
- Tasks routed to specific roles (finance, manager, legal)
- Task persistence in database (survives restarts)
- Idempotency keys prevent duplicate execution
- Cross-agent communication with full audit trail

---

### Part 8: Configuration Push

**Demo Config Distribution:**
1. Modify a profile (e.g., Finance)
2. Click "Push Configs to All Instances"
3. Show success: "✅ Configs pushed successfully: 3 instances updated"

**Explain What Happens:**
- Updated profiles pushed to all goose instances
- Privacy Guard proxies updated with new settings
- Users get latest configuration on next session
- No manual restart required

---

### Part 9: Live System Logs

**Show the Live Logs section** at bottom of dashboard:
- Real-time log streaming
- Auto-scrolls to latest entries
- Shows:
  - Controller startup
  - Privacy Guard connections
  - User profile assignments
  - Config push operations

---

### Part 10: System Logs Demonstration

**Show different log streams to demonstrate system monitoring:**

#### Controller Logs (Main Orchestration)
```bash
docker compose -f ce.dev.yml logs -f --tail=50 controller
```

**What to highlight:**
- JWT authentication events
- Profile fetch requests
- Agent Mesh task routing
- Database queries
- Error handling

**Sample log entries to point out:**
```
controller  | INFO: Profile 'finance' loaded from database
controller  | INFO: JWT token validated for client 'goose-controller'
controller  | INFO: Task 'BUDGET-APPROVAL-Q1-2025' routed to role 'manager'
controller  | INFO: Profile assigned: user=EMP001, profile=finance
controller  | INFO: Config push initiated for 3 instances
```

---

#### Privacy Guard Logs (Finance Instance)
```bash
docker compose -f ce.dev.yml logs -f --tail=30 privacy_finance
```

**What to highlight:**
- PII detection events
- Redaction actions
- Privacy mode changes
- Audit log entries

**Sample log entries to point out:**
```
privacy_finance  | WARN: PII detected - EMAIL in message (redacted)
privacy_finance  | INFO: Privacy mode set to 'strict' for session abc123
privacy_finance  | INFO: Pattern match: SSN detected at position 45
privacy_finance  | INFO: Audit log written: user=alice, action=redact, type=email
```

---

#### Keycloak Logs (Authentication)
```bash
docker compose -f ce.dev.yml logs -f --tail=30 keycloak
```

**What to highlight:**
- Token issuance
- Client authentication
- Token expiration warnings

**Sample log entries to point out:**
```
keycloak  | INFO: Token issued for client 'goose-controller', expires in 36000s
keycloak  | INFO: Client authenticated: goose-controller (client_credentials)
keycloak  | WARN: Token refresh requested 30 seconds before expiry
```

---

#### Vault Logs (Secrets & Signing)
```bash
docker compose -f ce.dev.yml logs -f --tail=30 vault
```

**What to highlight:**
- AppRole authentication
- Transit engine signing operations
- Audit log writes

**Sample log entries to point out:**
```
vault  | INFO: AppRole login successful: role_id=controller-role
vault  | INFO: Transit sign operation: key=profile-signing-key
vault  | INFO: Secret read: path=kv/goose/controller/db_credentials
vault  | INFO: Audit log entry written to stdout
```

---

#### PostgreSQL Logs (Database Activity)
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
docker compose -f ce.dev.yml logs -f --tail=30 postgres
```

**What to highlight:**
- Connection events
- Query execution
- Migration applications

**Sample log entries to point out:**
```
postgres  | LOG: connection received: host=controller port=41234
postgres  | LOG: execute SELECT: SELECT role, data FROM profiles WHERE role = 'finance'
postgres  | LOG: execute UPDATE: UPDATE org_users SET assigned_profile = 'finance'
postgres  | LOG: checkpoint complete: wrote 42 buffers
```

---

#### Redis Logs (Cache Activity)
```bash
docker compose -f ce.dev.yml logs -f --tail=20 redis
```

**What to highlight:**
- Cache hits/misses
- Idempotency key storage
- Session data caching

**Sample log entries to point out:**
```
redis  | INFO: DB 0: 42 keys (12 volatile) in 64 slots
redis  | INFO: SET idempotency:abc-123-def-456 EX 3600
redis  | INFO: GET profile:cache:finance (hit)
redis  | INFO: EXPIRE session:user-001 3600
```

---

#### Combined View (All Services)
```bash
docker compose -f ce.dev.yml logs -f --tail=20
```

**Use this to show the full system activity during key demo moments:**
- CSV upload (shows database inserts, cache updates)
- Profile assignment (shows controller, database, cache interaction)
- Task routing (shows agent mesh communication across services)
- Privacy filtering (shows Privacy Guard detecting and redacting PII)

---

### Part 11: Live Demo Log Checkpoints

**Timeline of what logs to show during each demo section:**

| Demo Section | Command | What to Point Out |
|--------------|---------|-------------------|
| CSV Upload | `docker compose logs -f controller postgres` | 50 INSERT statements, transaction commit |
| Profile Assignment | `docker compose logs -f controller redis` | UPDATE org_users, cache invalidation |
| goose Session Start | `docker compose logs -f controller privacy_finance` | Profile fetch, Privacy Guard connection |
| MCP Mesh Task | `docker compose logs -f controller redis` | Task INSERT, idempotency key SET, routing logic |
| Privacy Filtering | `docker compose logs -f privacy_finance` | PII detection, redaction, audit logging |
| JWT Token Refresh | `docker compose logs -f keycloak controller` | Token issuance, validation, expiry check |
| Vault Signing | `docker compose logs -f vault controller` | Transit sign, profile verification |

---

## Key Demo Talking Points

### 1. **Enterprise-Ready Security**
- JWT tokens for service authentication
- Vault for secrets management  
- Profile signature verification
- Role-based access control (RBAC)

### 2. **Privacy-First Design**
- Real-time PII detection & redaction
- Configurable privacy modes per role
- Audit logging of all sensitive data access
- Compliance-ready (GDPR, HIPAA, SOC2)

### 3. **Scalable Architecture**
- Database-backed persistence
- Idempotency for reliable task execution
- Multi-role support (finance, manager, legal)
- Horizontal scaling ready

### 4. **Admin Productivity**
- CSV bulk import (upload 50+ users instantly)
- Visual profile editor with JSON validation
- One-click config distribution
- Real-time monitoring dashboard

### 5. **Developer Experience**
- OpenAPI documentation
- RESTful API design
- Event-driven architecture
- Extensible plugin system

---

## Troubleshooting

### Vault is Sealed
```bash
./scripts/unseal_vault.sh
```
Use 3 unseal keys from `.env.ce` file.

### CSV Upload Fails (401 Unauthorized)
Get a fresh JWT token:
```bash
./get_admin_token.sh
```
Copy localStorage command to browser console.

### Privacy Guard Not Responding
Restart Privacy Guard containers:
```bash
docker compose -f ce.dev.yml restart $(docker ps --filter name=privacy --format "{{.Names}}")
```

### Database Connection Issues
Check PostgreSQL is running:
```bash
docker compose -f ce.dev.yml logs postgres
```

### Token Expired
Tokens last 10 hours. Get a new one:
```bash
./get_admin_token.sh
```

---

## Demo Reset (Start Fresh)

```bash
# Stop everything
docker compose -f ce.dev.yml down

# Remove volumes (WARNING: Deletes all data!)
docker volume rm compose_postgres_data compose_vault_raft

# Start fresh
docker compose -f ce.dev.yml up -d

# Unseal Vault
./scripts/unseal_vault.sh

# Re-upload org chart
./admin_upload_csv.sh test_data/demo_org_chart.csv
```

---

## URLs Quick Reference

| Service | URL | Purpose |
|---------|-----|---------|
| Admin Dashboard | http://localhost:8088/admin | Main demo interface |
| Keycloak | http://localhost:8080 | Identity & auth |
| Vault | https://localhost:8200 | Secrets management |
| API Docs | http://localhost:8088/docs | OpenAPI spec |
| Privacy Guard (Finance) | http://localhost:8096/ui | Finance privacy controls |
| Privacy Guard (Manager) | http://localhost:8097/ui | Manager privacy controls |
| Privacy Guard (Legal) | http://localhost:8098/ui | Legal privacy controls |
| Controller API | http://localhost:8088 | REST API base |

---

## Next Steps After Demo

1. **Phase 6 Completion:**
   - ✅ Admin Dashboard (complete)
   - ✅ CSV Upload (complete)
   - ✅ User Management (complete)
   - ✅ Profile Management (complete)
   - ⏳ Demo Validation (run `test_data/demo_validation.sh`)

2. **Production Readiness:**
   - Add Keycloak user sync
   - Implement config push mechanism
   - Add real-time log streaming
   - Deploy to staging environment

3. **Future Enhancements:**
   - LDAP/AD integration
   - SAML SSO support
   - Multi-tenancy
   - Advanced analytics dashboard

---

**🎉 You're ready to demo!**
