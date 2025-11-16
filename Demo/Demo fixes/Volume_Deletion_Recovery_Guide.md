# 🔄 VOLUME DELETION & FULL RECOVERY GUIDE

**Version:** 1.0  
**Date:** 2025-11-12  
**Phase:** 6 (Backend Integration & MVP Demo)  
**Purpose:** Complete recovery procedure after `docker compose down -v` (volume deletion)

---

## ⚠️ WARNING: DESTRUCTIVE OPERATION

**This guide covers recovery AFTER you've run:**
```bash
docker compose -f ce.dev.yml --profile controller --profile multi-goose \
  --profile redis down -v
```

**The `-v` flag deletes ALL volumes, resulting in:**
- ❌ All 50 users deleted (CSV must be re-uploaded)
- ❌ All Vault secrets deleted (NEW unseal keys generated)
- ❌ All Keycloak config deleted (realm/client must be recreated)
- ❌ All Ollama models deleted (~6GB must be re-downloaded)
- ❌ All profile signatures deleted (must re-sign)
- ❌ All tasks/sessions deleted (fresh database)

**Recovery Time:** 30-45 minutes (including downloads)

---

## Table of Contents
1. [What Gets Deleted](#1-what-gets-deleted)
2. [Volume Auto-Creation Behavior](#2-volume-auto-creation-behavior)
3. [Full Recovery Procedure](#3-full-recovery-procedure)
4. [Vault Re-initialization (Critical)](#4-vault-re-initialization-critical)
5. [Keycloak Reconfiguration](#5-keycloak-reconfiguration)
6. [Verification & Testing](#6-verification--testing)
7. [Recovery Checklist](#7-recovery-checklist)

---

## 1. What Gets Deleted

### Volume Deletion Impact Matrix

| Volume Name | Data Lost | Auto-Recovery | Manual Steps | Impact Level |
|-------------|-----------|---------------|--------------|--------------|
| `postgres_data` | Users, profiles, tasks, sessions | ✅ Schema auto-created | ❌ Re-upload CSV, re-sign profiles | **CRITICAL** |
| `vault_raft` | Secrets, unseal keys, signatures | ❌ Must re-init | ✅ New keys, new token, re-enable Transit | **CRITICAL** |
| `keycloak_data` | Realm, clients, users | ⚠️ Default realm only | ✅ Recreate dev realm, client, secret | **MAJOR** |
| `vault_logs` | Audit logs | ✅ Empty logs created | N/A | Minor |
| `redis_data` | Cache, idempotency keys | ✅ Empty cache created | N/A | Minor |
| `ollama_finance` | qwen3:0.6b model | ✅ Empty volume created | ❌ Re-download 522MB | **MAJOR** |
| `ollama_manager` | qwen3:0.6b model | ✅ Empty volume created | ❌ Re-download 522MB | **MAJOR** |
| `ollama_legal` | qwen3:0.6b model | ✅ Empty volume created | ❌ Re-download 522MB | **MAJOR** |
| `goose_*_workspace` | Session files | ✅ Empty workspace created | N/A | Minor |

**Total Download Required:** ~1.6GB (3× Ollama models)  
**Total Configuration Time:** ~20-30 minutes  
**Risk Level:** HIGH - New Vault keys must be saved or system is unrecoverable

---

## 2. Volume Auto-Creation Behavior

### What Happens on First `docker compose up` After Volume Deletion

**Automatic (No Intervention Needed):**
- ✅ Docker creates all 13 volumes as empty directories/filesystems
- ✅ PostgreSQL initializes empty database cluster
- ✅ Controller runs migrations (creates tables, seeds 8 default profiles)
- ✅ Redis starts with empty cache
- ✅ Ollama starts with empty model storage
- ✅ Keycloak creates default `master` realm

**Manual Intervention Required:**
- ❌ Vault: Must run `vault operator init` to generate NEW unseal keys
- ❌ Vault: Must unseal with 3 of 5 NEW keys (old keys won't work)
- ❌ Vault: Must re-enable Transit engine and create signing key
- ❌ Vault: Must create new Controller token with proper policy
- ❌ Keycloak: Must create `dev` realm (or import from backup)
- ❌ Keycloak: Must create `goose-controller` client
- ❌ Keycloak: Must generate new client secret
- ❌ Ollama: Must pull qwen3:0.6b model into each instance
- ❌ PostgreSQL: Must re-upload 50 users from CSV
- ❌ PostgreSQL: Must re-sign all 8 profiles

---

## 3. Full Recovery Procedure

### Prerequisites

```bash
# Navigate to compose directory
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Verify volumes are deleted (should return empty or error)
docker volume ls | grep compose_

# Verify no containers running
docker compose -f ce.dev.yml ps
# Should show: no containers
```

---

### Step 1: Start Infrastructure Layer

```bash
# Start postgres, keycloak, vault, redis
docker compose -f ce.dev.yml up -d postgres keycloak vault redis

# Wait for health checks (empty volumes take longer on first start)
echo "Waiting for infrastructure (45s)..."
sleep 45

# Verify all healthy
docker compose -f ce.dev.yml ps postgres keycloak vault redis

# Expected output:
# ce_postgres    Up 45s (healthy)
# ce_keycloak    Up 45s (healthy)
# ce_vault       Up 45s (healthy)
# ce_redis       Up 45s (healthy)
```

**What Happened:**
- PostgreSQL: Empty database cluster created, ready for migrations
- Keycloak: Default `master` realm created
- Vault: Started in sealed state with EMPTY storage (no keys yet)
- Redis: Empty cache ready

---

### Step 2: Verify PostgreSQL Migrations Ran

```bash
# Check if database "orchestrator" exists
docker exec ce_postgres psql -U postgres -c "\l" | grep orchestrator

# Expected: "orchestrator" database listed

# Check tables created
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"

# Expected tables:
# - _sqlx_migrations
# - sessions
# - org_users
# - profiles
# - tasks

# Check default profiles seeded (migrations 0003-0006)
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role FROM profiles ORDER BY role;"

# Expected: 8 profiles (analyst, developer, finance, hr, legal, manager, marketing, support)

# Check users table (should be empty)
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM org_users;"

# Expected: 0 (CSV not uploaded yet)
```

**Result:**
- ✅ Database schema fully created
- ✅ 8 default profiles auto-seeded
- ❌ Profiles NOT signed yet (Vault not initialized)
- ❌ Users table empty (CSV upload needed)

---

## 4. Vault Re-initialization (Critical)

### Step 3: Initialize New Vault (Generates NEW Keys)

**⚠️ CRITICAL:** This generates COMPLETELY NEW unseal keys and root token.  
Your old keys from `.env.ce` will NOT work anymore!

```bash
# Initialize Vault (interactive)
docker exec -it ce_vault vault operator init

# OUTPUT (EXAMPLE - YOUR KEYS WILL BE DIFFERENT):
# Unseal Key 1: ABC123...
# Unseal Key 2: DEF456...
# Unseal Key 3: GHI789...
# Unseal Key 4: JKL012...
# Unseal Key 5: MNO345...
# 
# Initial Root Token: hvs.CAESI...
```

**⚠️ ACTION REQUIRED: SAVE THESE KEYS IMMEDIATELY!**

**Option 1: Save to .env.ce (Recommended for Dev, not for Production)**
```bash
# Open .env.ce in editor
nano /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce

# Update these lines with NEW keys:
VAULT_UNSEAL_KEY_1=<paste-new-key-1>
VAULT_UNSEAL_KEY_2=<paste-new-key-2>
VAULT_UNSEAL_KEY_3=<paste-new-key-3>
VAULT_UNSEAL_KEY_4=<paste-new-key-4>
VAULT_UNSEAL_KEY_5=<paste-new-key-5>

# Save and exit (Ctrl+O, Enter, Ctrl+X)
```

**Option 2: Save to Secure Location (Production)**
- Store in password manager (1Password, LastPass, etc.)
- Store in encrypted file
- **NEVER commit to Git!**

---

### Step 4: Unseal Vault with NEW Keys

**You need 3 of the 5 keys to unseal:**

```bash
# First unseal (requires key 1)
docker exec -it ce_vault vault operator unseal
# Paste: <new-key-1>
# Press Enter

# Output:
# Sealed: true
# Unseal Progress: 1/3

# Second unseal (requires key 2)
docker exec -it ce_vault vault operator unseal
# Paste: <new-key-2>
# Press Enter

# Output:
# Sealed: true
# Unseal Progress: 2/3

# Third unseal (requires key 3)
docker exec -it ce_vault vault operator unseal
# Paste: <new-key-3>
# Press Enter

# Output:
# Sealed: false  ← SUCCESS!
# Cluster Name: vault-cluster-...
```

**Verify Vault Unsealed:**
```bash
docker exec ce_vault vault status

# Expected output:
# Sealed: false
# Cluster Name: vault-cluster-...
# HA Enabled: true
# HA Mode: active
```

**If Sealed: true** - Repeat unseal process with 3 keys.

---

### Step 5: Configure Vault Transit Engine

**Login with Root Token:**

```bash
# Use the root token from Step 3 initialization
docker exec -it ce_vault vault login

# Paste: <new-root-token>
# Press Enter

# Expected:
# Success! You are now authenticated.
```

**Enable Transit Secrets Engine:**

```bash
# Enable Transit engine (for HMAC signing)
docker exec ce_vault vault secrets enable transit

# Expected:
# Success! Enabled the transit secrets engine at: transit/

# Create profile-signing key
docker exec ce_vault vault write -f transit/keys/profile-signing

# Expected:
# Success! Data written to: transit/keys/profile-signing

# Verify key created
docker exec ce_vault vault read transit/keys/profile-signing

# Expected output shows:
# deletion_allowed: false
# exportable: false
# type: aes256-gcm96
```

---

### Step 6: Create Controller Policy & Token

**Create Policy File:**

```bash
# Create controller policy with Transit permissions
docker exec ce_vault vault policy write controller-policy - <<EOF
path "transit/keys/profile-signing" {
  capabilities = ["read"]
}
path "transit/sign/profile-signing" {
  capabilities = ["create", "update"]
}
path "transit/verify/profile-signing" {
  capabilities = ["create", "update"]
}
EOF

# Expected:
# Success! Uploaded policy: controller-policy
```

**Generate Token for Controller:**

```bash
# Generate token with 32-day TTL
NEW_VAULT_TOKEN=$(docker exec ce_vault vault token create \
  -policy=controller-policy \
  -ttl=768h \
  -format=json | jq -r '.auth.client_token')

# Display new token
echo "New Controller Vault Token:"
echo "$NEW_VAULT_TOKEN"

# Should output: hvs.CAESIABC... (starts with hvs.)
```

**Update .env.ce with New Token:**

```bash
# Open .env.ce
nano /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce

# Find and update this line:
VAULT_TOKEN=<paste-new-token-here>

# Save and exit (Ctrl+O, Enter, Ctrl+X)
```

**Verify Token Works:**

```bash
# Test token can access Transit
docker exec ce_vault sh -c "VAULT_TOKEN=$NEW_VAULT_TOKEN vault read transit/keys/profile-signing"

# Expected: Shows key details (not "permission denied")
```

---

### Step 7: Start Ollama Instances & Download Models

**Start Ollama Containers:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Start all 3 Ollama instances
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  ollama-finance ollama-manager ollama-legal

# Wait for containers to start
sleep 10

# Verify running
docker compose -f ce.dev.yml ps | grep ollama

# Expected: 3 containers running (healthy)
```

**Pull qwen3:0.6b Model (3× downloads in parallel):**

```bash
# Pull into all 3 instances simultaneously
docker exec ce_ollama_finance ollama pull qwen3:0.6b &
docker exec ce_ollama_manager ollama pull qwen3:0.6b &
docker exec ce_ollama_legal ollama pull qwen3:0.6b &

# Wait for all downloads to complete (3-5 minutes)
wait

echo "All models pulled!"

# Verify models loaded
docker exec ce_ollama_finance ollama list
docker exec ce_ollama_manager ollama list
docker exec ce_ollama_legal ollama list

# Each should show:
# NAME          ID              SIZE      MODIFIED
# qwen3:0.6b    7df6b6e09427    522 MB    ...
```

**Total Download:** ~1.6GB (3× 522MB, may share layers)

---

### Step 8: Start Controller (Migrations Already Ran)

```bash
# Start Controller (will use new VAULT_TOKEN from .env.ce)
docker compose -f ce.dev.yml --profile controller up -d controller

# Wait for health check
echo "Waiting for Controller (20s)..."
sleep 20

# Verify healthy
curl -s http://localhost:8088/status | jq '.'

# Expected:
# {
#   "status": "ok",
#   "version": "0.1.0"
# }

# Check Controller logs for Vault authentication
docker logs ce_controller | tail -30 | grep -i vault

# Expected to see:
# "Vault token authentication successful" OR
# "Vault AppRole authentication successful"
```

**If Vault errors appear:**
- Check VAULT_TOKEN in .env.ce matches token from Step 6
- Restart Controller: `docker compose -f ce.dev.yml --profile controller restart controller`

---

### Step 9: Sign Default Profiles (Auto-Seeded by Migrations)

```bash
# Navigate to project root
cd /home/papadoc/Gooseprojects/goose-org-twin

# Run profile signing script
./scripts/sign-all-profiles.sh

# Expected output:
# Signing Summary
# =========================================
# Successfully signed:  8
# Already signed:       0
# Failed:               0
# Total profiles:       8
```

**Verify Signatures in Database:**

```bash
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role, (data->'signature'->>'signature') IS NOT NULL AS has_signature FROM profiles ORDER BY role;"

# Expected: All 8 profiles show has_signature = t
#   role      | has_signature
# ------------+---------------
#  analyst    | t
#  developer  | t
#  finance    | t
#  hr         | t
#  legal      | t
#  manager    | t
#  marketing  | t
#  support    | t
```

---

### Step 10: Start Privacy Guard Services

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Start all 3 Privacy Guard Services
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal

# Wait for health checks
echo "Waiting for Privacy Services (25s)..."
sleep 25

# Verify all healthy
docker compose -f ce.dev.yml ps | grep privacy-guard | grep -v proxy

# Expected: 3 containers, all showing "healthy"
# ce_privacy_guard_finance   Up (healthy)   8093->8089
# ce_privacy_guard_manager   Up (healthy)   8094->8089
# ce_privacy_guard_legal     Up (healthy)   8095->8089
```

---

### Step 11: Start Privacy Guard Proxies

```bash
# Start all 3 Privacy Guard Proxies
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  privacy-guard-proxy-finance privacy-guard-proxy-manager privacy-guard-proxy-legal

# Wait for health checks
echo "Waiting for Proxies (20s)..."
sleep 20

# Verify all healthy
docker compose -f ce.dev.yml ps | grep proxy

# Expected: 3 containers, all showing "healthy"
# ce_privacy_guard_proxy_finance   Up (healthy)   8096->8090
# ce_privacy_guard_proxy_manager   Up (healthy)   8097->8090
# ce_privacy_guard_proxy_legal     Up (healthy)   8098->8090
```

---

### Step 12: Rebuild & Start Goose Instances

```bash
# Rebuild Goose images (ensure latest code)
docker compose -f ce.dev.yml --profile multi-goose --profile controller build --no-cache \
  goose-finance goose-manager goose-legal

# Expected: 3-5 minutes build time

# Remove old containers (if any exist)
docker rm -f ce_goose_finance ce_goose_manager ce_goose_legal 2>/dev/null || true

# Start all 3 Goose instances
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  goose-finance goose-manager goose-legal

# Wait for profile fetch
echo "Waiting for Goose instances (15s)..."
sleep 15

# Verify running
docker compose -f ce.dev.yml ps goose-finance goose-manager goose-legal

# Check profile fetch logs
echo "=== Finance ==="
docker logs ce_goose_finance 2>&1 | grep "Profile fetched"

echo "=== Manager ==="
docker logs ce_goose_manager 2>&1 | grep "Profile fetched"

echo "=== Legal ==="
docker logs ce_goose_legal 2>&1 | grep "Profile fetched"

# Expected: Each shows "✓ Profile fetched successfully"
```

---

### Step 13: Re-upload Organization Chart (50 Users)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Upload CSV with 50 test users
./admin_upload_csv.sh test_data/demo_org_chart.csv

# Expected output:
# ✅ Successfully imported! Created: 50, Updated: 0

# Verify users in database
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM org_users;"

# Expected: 50
```

---

## 5. Keycloak Reconfiguration

### Step 14: Recreate `dev` Realm

**After volume deletion, Keycloak only has the default `master` realm.**

**Option 1: Manual Creation via UI**

```bash
# Open Keycloak Admin Console
xdg-open http://localhost:8080

# Login:
# Username: admin
# Password: admin

# Steps:
# 1. Click "Master" dropdown (top-left)
# 2. Click "Create realm"
# 3. Realm name: dev
# 4. Click "Create"
```

**Option 2: Import from Backup (If You Have One)**

```bash
# If you have a realm export file:
docker cp dev-realm-export.json ce_keycloak:/tmp/

docker exec ce_keycloak /opt/keycloak/bin/kc.sh import \
  --file /tmp/dev-realm-export.json

# Restart Keycloak
docker compose -f ce.dev.yml restart keycloak
```

---

### Step 15: Create `goose-controller` Client

**In Keycloak Admin Console (http://localhost:8080):**

**Steps:**
1. Select `dev` realm (dropdown top-left)
2. Click "Clients" (left menu)
3. Click "Create client"
4. **General Settings:**
   - Client type: OpenID Connect
   - Client ID: `goose-controller`
   - Click "Next"
5. **Capability config:**
   - ✅ Client authentication: ON
   - ✅ Service accounts roles: ON
   - ❌ Standard flow: OFF
   - ❌ Direct access grants: OFF
   - Click "Next"
6. **Login settings:**
   - Leave defaults
   - Click "Save"

---

### Step 16: Generate New Client Secret

**In Keycloak Admin Console:**

**Steps:**
1. Navigate to: `dev` realm → Clients → `goose-controller`
2. Click "Credentials" tab
3. Copy the "Client secret" value (auto-generated)
4. Save this secret!

**Example secret:** `elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8`

---

### Step 17: Update .env.ce with New Client Secret

```bash
# Open .env.ce
nano /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce

# Find and update this line:
OIDC_CLIENT_SECRET=<paste-new-client-secret-here>

# Example:
OIDC_CLIENT_SECRET=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8

# Save and exit (Ctrl+O, Enter, Ctrl+X)
```

**Restart Controller to Use New Secret:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

docker compose -f ce.dev.yml --profile controller restart controller

# Wait for healthy
sleep 20

# Verify Controller can authenticate with Keycloak
docker logs ce_controller | tail -50 | grep -i "jwt\|oidc\|keycloak"

# Should NOT see authentication errors
```

**Restart Goose Instances to Use New Secret:**

```bash
docker compose -f ce.dev.yml --profile multi-goose --profile controller restart \
  goose-finance goose-manager goose-legal

# Wait for profile fetch
sleep 20

# Verify profile fetch successful
docker logs ce_goose_finance 2>&1 | grep "Profile fetched"
docker logs ce_goose_manager 2>&1 | grep "Profile fetched"
docker logs ce_goose_legal 2>&1 | grep "Profile fetched"

# Expected: "✓ Profile fetched successfully" for each
```

---

## 6. Verification & Testing

### Step 18: System Health Check

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Check all containers running
docker compose -f ce.dev.yml ps

# Expected: 17 containers total
# - 4 infrastructure (postgres, keycloak, vault, redis)
# - 3 Ollama
# - 1 Controller
# - 3 Privacy Guard Services
# - 3 Privacy Guard Proxies
# - 3 Goose instances

# Verify critical endpoints
curl -s http://localhost:8088/status | jq '.'  # Controller
curl -s http://localhost:8096/api/status | jq '.'  # Finance Proxy
curl -s http://localhost:8097/api/status | jq '.'  # Manager Proxy
curl -s http://localhost:8098/api/status | jq '.'  # Legal Proxy

# All should return: {"status": "ok"} or similar
```

---

### Step 19: Test Admin Dashboard Access

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Generate Admin JWT token
./get_admin_token.sh

# Copy the localStorage command from output
# Example: localStorage.setItem('admin_token', 'eyJhbGc...');

# Open Admin Dashboard
xdg-open http://localhost:8088/admin

# In browser:
# 1. Press F12 (Developer Console)
# 2. Click "Console" tab
# 3. Paste localStorage command
# 4. Press Enter
# 5. Refresh page (F5)

# Verify:
# - User Management section shows 50 users
# - Profile Management section shows 8 profiles
# - CSV upload works
```

---

### Step 20: Test Goose Session

```bash
# Test Finance Goose session
docker exec -it ce_goose_finance goose session

# In Goose prompt:
# > "Hello, can you confirm my role and available tools?"

# Expected response should show:
# - Role: finance (from profile)
# - Tools: agentmesh__* tools available
# - Privacy Guard integration working

# Exit: Ctrl+D or type "exit"
```

**Test Profile Fetch:**
```bash
# Check if profile loaded correctly
docker logs ce_goose_finance 2>&1 | grep -A 10 "Profile fetched"

# Should show:
# ✓ Profile fetched successfully
# Role: finance
# Display name: Finance Agent
```

---

### Step 21: Test Agent Mesh (Optional)

**Via Goose Desktop (Recommended):**
- Agent Mesh MCP tools work perfectly in Goose Desktop (no Docker stdio issues)
- Test `agentmesh__send_task`, `agentmesh__notify`, etc.

**Via API (Proof Backend Works):**

```bash
# Get JWT token
OIDC_CLIENT_SECRET=$(docker exec ce_controller env | grep OIDC_CLIENT_SECRET | cut -d= -f2)
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=${OIDC_CLIENT_SECRET}" \
  | jq -r '.access_token')

# Send test task
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{
    "target": "manager",
    "task": {
      "task_type": "test",
      "description": "Recovery test task",
      "data": {"test": true}
    }
  }' | jq '.'

# Expected: Task created with task_id

# Verify in database
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT task_id, target, task_type, status FROM tasks ORDER BY created_at DESC LIMIT 5;"

# Should show the test task
```

---

## 7. Recovery Checklist

### ✅ Post-Recovery Verification

**Infrastructure:**
- [ ] Postgres running and healthy
- [ ] Keycloak running and healthy
- [ ] Vault running, unsealed, Transit enabled
- [ ] Redis running and healthy

**Vault Configuration:**
- [ ] New unseal keys saved securely
- [ ] New root token saved securely
- [ ] Transit engine enabled (`transit/`)
- [ ] Profile signing key created (`transit/keys/profile-signing`)
- [ ] Controller policy created (`controller-policy`)
- [ ] New Controller token generated and saved to `.env.ce`

**Keycloak Configuration:**
- [ ] `dev` realm created
- [ ] `goose-controller` client created
- [ ] Client authentication enabled
- [ ] Service accounts roles enabled
- [ ] New client secret saved to `.env.ce`

**Ollama:**
- [ ] Finance Ollama: qwen3:0.6b model loaded
- [ ] Manager Ollama: qwen3:0.6b model loaded
- [ ] Legal Ollama: qwen3:0.6b model loaded

**Database:**
- [ ] Migrations completed (tables created)
- [ ] 8 default profiles seeded
- [ ] All 8 profiles signed (Vault Transit HMAC)
- [ ] 50 users uploaded from CSV

**Services:**
- [ ] Controller healthy, Vault authentication working
- [ ] Privacy Guard Finance healthy
- [ ] Privacy Guard Manager healthy
- [ ] Privacy Guard Legal healthy
- [ ] Privacy Proxy Finance healthy
- [ ] Privacy Proxy Manager healthy
- [ ] Privacy Proxy Legal healthy
- [ ] Goose Finance: profile fetched successfully
- [ ] Goose Manager: profile fetched successfully
- [ ] Goose Legal: profile fetched successfully

**Functional Tests:**
- [ ] Admin Dashboard accessible
- [ ] JWT token generation working
- [ ] User list shows 50 users
- [ ] Profile management shows 8 profiles
- [ ] CSV upload works
- [ ] Goose session starts successfully
- [ ] Agent Mesh tasks can be created (via API or Desktop)

---

## 📝 Summary

**Total Recovery Time:** 30-45 minutes

**Critical Steps (Can't Skip):**
1. Vault initialization (new keys)
2. Vault unsealing (3 of 5 keys)
3. Vault Transit configuration
4. Controller token generation
5. Keycloak realm recreation
6. Client secret regeneration
7. Ollama model downloads
8. Profile signing
9. CSV re-upload

**What's Automatic:**
- ✅ Volume creation
- ✅ Database schema (migrations)
- ✅ Default profile seeding
- ✅ Redis cache initialization

**What's Manual:**
- ❌ Vault initialization & unsealing
- ❌ Keycloak realm/client setup
- ❌ Ollama model downloads
- ❌ Environment variable updates
- ❌ CSV data upload

**Key Files to Update:**
- `/home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce`
  - `VAULT_UNSEAL_KEY_1` through `VAULT_UNSEAL_KEY_5` (NEW)
  - `VAULT_TOKEN` (NEW)
  - `OIDC_CLIENT_SECRET` (NEW)

**Backup Recommendations:**
- Export Keycloak `dev` realm regularly
- Document Vault policies in version control
- Keep `.env.ce.backup` with working configuration
- Save unseal keys in secure password manager

---

**Document End**  
**Related Documents:**
- Container_Management_Playbook.md (normal operations)
- System_Analysis_Report.md (architecture overview)
- Demo_Execution_Plan.md (demo procedures)
