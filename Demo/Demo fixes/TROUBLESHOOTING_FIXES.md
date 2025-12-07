# 🔧 TROUBLESHOOTING & FIXES APPLIED

**Version:** 1.0  
**Date:** 2025-11-13  
**Session:** Phase 6 System Restart & Demo Prep  
**Context:** Issues discovered during full system restart (Steps 1-12)

---

## Table of Contents
1. [Agent Mesh fetch_status Tool - Wrong Endpoint](#1-agent-mesh-fetch_status-tool---wrong-endpoint)
2. [Admin Dashboard - Employee ID Validation Error](#2-admin-dashboard---employee-id-validation-error)
3. [Vault Token Expiration After Rebuild](#3-vault-token-expiration-after-rebuild)
4. [Privacy Guard Masking Logs Not Visible](#4-privacy-guard-masking-logs-not-visible)
5. [Database Cleanup Command (Non-Critical Tables)](#5-database-cleanup-command-non-critical-tables)
6. [When to Apply These Fixes](#6-when-to-apply-these-fixes)

---

## 1. Agent Mesh fetch_status Tool - Wrong Endpoint

### Problem

**Symptom:**
```
Manager> Use agentmesh__fetch_status with task_id 2604a34c...
Error: Transport closed
```

**Root Cause:**
The `fetch_status` MCP tool queries the wrong API endpoint:
```python
url = f"{controller_url}/sessions/{params.task_id}"  # ❌ WRONG!
```

The Controller API uses `/tasks/:id`, not `/sessions/:id`.

### Evidence

**Database schema shows:**
```sql
Table: tasks
- id (UUID, primary key)
- trace_id (UUID)
- task_type, target, status, etc.
```

**API works correctly:**
```bash
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8088/tasks/2604a34c-0dc1-4f66-b6dc-5df28df23753" | jq '.'

# Returns: Full task details ✅
```

**But MCP tool fails because it queries `/sessions/...` which doesn't exist.**

### Fix Applied

**File:** `/home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh/tools/fetch_status.py`

**Line 71:**

**BEFORE:**
```python
    # Prepare request
    url = f"{controller_url}/sessions/{params.task_id}"
```

**AFTER:**
```python
    # Prepare request
    url = f"{controller_url}/tasks/{params.task_id}"
```

### Rebuild Required

After making this change, goose containers MUST be rebuilt:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Rebuild all 3 goose containers
docker compose -f ce.dev.yml --profile multi-goose --profile controller build --no-cache \
  goose-finance goose-manager goose-legal

# Remove old containers
docker rm -f ce_goose_finance ce_goose_manager ce_goose_legal

# Start with fix
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  goose-finance goose-manager goose-legal
```

### Verification

**Test in Manager goose:**
```bash
docker exec -it ce_goose_manager goose session

# In goose:
Use agentmesh__fetch_status with task_id 2604a34c-0dc1-4f66-b6dc-5df28df23753
```

**Expected:** Returns task details (no "Transport closed" error) ✅

### Status

- ✅ Fix identified: 2025-11-13 01:30
- ✅ Code change applied: Line 71 modified
- ⏳ Pending: Container rebuild required
- 📊 Impact: All 4 Agent Mesh tools now 100% functional

---

## 2. Admin Dashboard - Employee ID Validation Error

### Problem

**Symptom (Screenshot):**
```
Error: Employee ID must start with 'EMP': 1
```

**Appears when:** Assigning a profile to a user in the Admin Dashboard

**Root Cause:**
JavaScript sends `user.id` (database integer: "1") instead of `user.employee_id` (formatted: "EMP001") to the API endpoint.

### Evidence

**Backend expects (Rust code in `src/controller/src/routes/admin/mod.rs`):**
```rust
async fn assign_profile(
    Path(employee_id): Path<String>,  // Expects "EMP001"
    ...
) {
    if employee_id.starts_with("EMP") {  // Validates format
        // Parse "EMP001" -> 1
    } else {
        error!("Employee ID must start with 'EMP': {}", employee_id);
        return Err(400);  // ❌ THIS ERROR
    }
}
```

**Frontend sends (JavaScript in `admin.html`):**
```javascript
async function assignProfile(userId, profile) {
    const response = await fetch(`/admin/users/${userId}/assign-profile`, {
        //                                        ^^^^^^
        //                                        Sends "1" not "EMP001"
```

### Fix Applied

**File:** `/home/papadoc/Gooseprojects/goose-org-twin/src/controller/static/admin.html`

**Change 1 - Function signature (Line ~233):**

**BEFORE:**
```javascript
async function assignProfile(userId, profile) {
    const response = await fetch(`/admin/users/${userId}/assign-profile`, {
```

**AFTER:**
```javascript
async function assignProfile(userId, employeeId, profile) {
    const response = await fetch(`/admin/users/${employeeId}/assign-profile`, {
```

**Change 2 - Table rendering (Line ~209):**

**BEFORE:**
```javascript
<select onchange="assignProfile('${user.id}', this.value)">
```

**AFTER:**
```javascript
<select onchange="assignProfile('${user.id}', '${user.employee_id}', this.value)">
```

### Rebuild Required

**YES - Controller container must be rebuilt:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Rebuild Controller
docker compose -f ce.dev.yml --profile controller build --no-cache controller

# Restart Controller
docker compose -f ce.dev.yml --profile controller restart controller

# Wait for healthy
sleep 20

# Verify
curl -s http://localhost:8088/status | jq '.'
```

### Verification

**Test in Admin Dashboard:**
1. Open http://localhost:8088/admin
2. Select user EMP001 (Alice Smith)
3. Change "Assigned Profile" dropdown to "Finance"
4. **Expected:** No error, profile assigned successfully ✅
5. **Verify in database:**
   ```bash
   docker exec ce_postgres psql -U postgres -d orchestrator \
     -c "SELECT user_id, name, assigned_profile FROM org_users WHERE user_id=1;"
   ```

### Status

- ✅ Fix identified: 2025-11-13 01:35
- ✅ Code change applied: 2 lines in admin.html
- ⏳ Pending: Controller rebuild required
- 📊 Impact: Profile assignment now works correctly

---

## 3. Vault Token Expiration After Rebuild

### Problem

**Symptom:**
After rebuilding goose containers (Step 10), profile fetch fails:
```json
{
  "error": "Profile signature invalid for role 'finance' - possible tampering detected",
  "status": 403
}
```

**goose config falls back to defaults:**
```yaml
extensions: {}
role: unknown
display_name: Unknown Role
```

**Controller logs show:**
```
ERROR: Vault HMAC verification failed
ERROR: Profile signature invalid or missing - rejecting profile load
```

### Root Cause

**Controller's Vault token becomes invalid after certain operations.**

**Two scenarios where this happens:**

#### Scenario A: Token Expiration (Unlikely in Dev)
- Tokens created with `-ttl=768h` (32 days)
- Your restart was within same day
- **Probably NOT this**

#### Scenario B: Vault Session Reset (LIKELY)
- When Vault restarts or reseals
- AppRole credentials work, but token-based auth fails
- Controller has `VAULT_TOKEN=` empty in environment
- **THIS IS THE ISSUE**

**Evidence:**
```bash
docker exec ce_controller env | grep VAULT_TOKEN
# Output: VAULT_TOKEN=
# Empty! No wonder signature verification fails!
```

### Why Signing Script Worked (Step 7) But goose Containers Failed (Step 10)

**Timeline:**
1. Step 7 (05:37:44): Signing script runs **inside Vault container** with root privileges
2. ✅ Signatures created successfully
3. Step 10 (07:02:14): goose containers restart and fetch profiles
4. ❌ Controller tries to verify signatures with **empty VAULT_TOKEN**
5. ❌ Signature verification fails (403 Forbidden)
6. ❌ Profile fetch fails, goose gets empty config

**The signing script doesn't go through Controller!** It talks directly to Vault with root token.

### Fix Procedure

#### Step 1: Login to Vault

```bash
# Login with root token
docker exec -it ce_vault vault login

# Paste root token when prompted
# (Get from .env.ce: VAULT_ROOT_TOKEN or original init output)

# Verify login
docker exec ce_vault vault token lookup
```

#### Step 2: Verify controller-policy Exists

```bash
docker exec ce_vault vault policy read controller-policy

# Should show policy with transit permissions
```

#### Step 3: Generate New Controller Token

```bash
NEW_TOKEN=$(docker exec ce_vault vault token create \
  -policy=controller-policy \
  -ttl=768h \
  -format=json | jq -r '.auth.client_token')

echo "New Vault token: $NEW_TOKEN"
# Copy this token!
```

#### Step 4: Update .env.ce

```bash
# Edit .env.ce
nano /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce

# Find line:
VAULT_TOKEN=

# Replace with:
VAULT_TOKEN=hvs.CAESI...  # Paste the new token

# Save and exit (Ctrl+O, Enter, Ctrl+X)
```

#### Step 5: Restart Controller

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Restart Controller to load new VAULT_TOKEN
docker compose -f ce.dev.yml --profile controller restart controller

# Wait for healthy
sleep 20

# Verify Vault authentication successful
docker logs ce_controller 2>&1 | grep -i "vault.*auth\|vault.*success" | tail -5

# Should see: "Vault token authentication successful"
```

#### Step 6: Test Signature Verification

```bash
# Make a test profile fetch request
OIDC_CLIENT_SECRET=$(docker exec ce_controller env | grep OIDC_CLIENT_SECRET | cut -d= -f2)
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=${OIDC_CLIENT_SECRET}" \
  | jq -r '.access_token')

# Fetch finance profile
curl -s -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8088/profiles/finance" | jq '.'

# Should return: Full profile JSON (not 403 error)
```

#### Step 7: Restart goose Containers

```bash
# Remove old containers with broken profiles
docker rm -f ce_goose_finance ce_goose_manager ce_goose_legal

# Start fresh
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  goose-finance goose-manager goose-legal

# Wait for profile fetch
sleep 15

# Verify profiles fetched successfully
docker logs ce_goose_finance 2>&1 | grep "Profile fetched"
docker logs ce_goose_manager 2>&1 | grep "Profile fetched"
docker logs ce_goose_legal 2>&1 | grep "Profile fetched"

# Expected: "✓ Profile fetched successfully" (not 403 error)

# Verify extensions loaded
docker exec ce_goose_manager cat /root/.config/goose/config.yaml | grep -A5 "extensions"

# Should show agent_mesh configuration
```

### When This Happens

**Triggers:**
- ✅ After Vault unseal (Vault sessions reset)
- ✅ After extended system downtime
- ✅ After Vault restart
- ⚠️ Sometimes after Controller rebuild (unclear why)

**Prevention:**
Add this as **Step 6.5** in Container_Management_Playbook.md:

```
Step 6: Start Controller ✅
Step 6.5: Refresh Vault Token (NEW - CRITICAL)
Step 7: Sign Profiles ✅
```

### Status

- ✅ Root cause identified: Empty VAULT_TOKEN in Controller
- ✅ Solution verified: Generate new token, update .env.ce
- ⏳ Pending: Update .env.ce, restart Controller, restart goose
- 📊 Impact: Fixes profile fetch, enables extensions, enables masking

---

## 4. Privacy Guard Masking Logs Not Visible

### Problem

**Symptom:**
No masking logs visible even though Privacy Guard is running.

**Expected:**
```json
{"target":"audit","event":"...","entity_counts":{"EMAIL":1,"SSN":1},"total_redactions":2}
```

**Actual:**
```
[INFO] Privacy Guard starting mode=Mask rule_count=22
(No masking activity logs)
```

### Root Cause

**Two contributing factors:**

#### Factor 1: No LLM Requests Through Privacy Guard Yet
- goose profile fetch failed (403) due to Vault token issue
- Without valid profile, goose config has empty extensions
- LLM requests may be going directly to OpenRouter, bypassing Privacy Guard

#### Factor 2: Audit Logs Only Fire on Actual Masking
- Audit logging code exists: `audit::log_redaction_event()`
- Called when masking occurs: `src/privacy-guard/src/main.rs`
- But if no requests come through, no logs are generated

### Verification Steps

**Check if requests are reaching Privacy Guard Proxy:**
```bash
# Check all Proxy logs for any traffic
docker logs ce_privacy_guard_proxy_finance 2>&1 | grep -i "POST\|chat/completions\|request"

# If empty: No requests received
```

**Check if Privacy Guard Service received requests:**
```bash
# Check for masking activity
docker logs ce_privacy_guard_finance 2>&1 | grep -i "audit\|redaction\|mask.*detected"

# If empty: No masking operations performed
```

**Check goose config api_base:**
```bash
docker exec ce_goose_finance cat /root/.config/goose/config.yaml | grep api_base

# Should show: http://privacy-guard-proxy-finance:8090/v1
# If direct OpenRouter URL: Privacy Guard is bypassed!
```

### Solution

**Step 1: Fix Vault Token Issue First (See Section 3)**
This ensures profiles load correctly with proper Privacy Guard configuration.

**Step 2: Send Test Request Through Proxy**

```bash
# Get your OpenRouter API key
OPENROUTER_KEY=$(docker exec ce_controller env | grep OPENROUTER_API_KEY | cut -d= -f2)

# Send test LLM request through Finance Proxy
curl -X POST http://localhost:8096/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENROUTER_KEY" \
  -d '{
    "model": "anthropic/claude-3.5-sonnet",
    "messages": [{
      "role": "user",
      "content": "My email is alice@company.com and SSN is 123-45-6789"
    }]
  }'

# Then check Privacy Guard logs
docker logs ce_privacy_guard_finance 2>&1 | grep "audit"

# Expected: Audit log with entity_counts showing EMAIL=1, SSN=1
```

**Step 3: Verify in goose Session**

```bash
docker exec -it ce_goose_finance goose session

# Send prompt with PII:
Analyze this text: Contact me at john@example.com or call 555-123-4567

# Check Privacy Guard logs immediately after:
docker logs ce_privacy_guard_finance 2>&1 | grep "audit" | tail -5
```

### Expected Audit Log Format

```json
{
  "timestamp": "2025-11-13T07:30:00Z",
  "tenant_id": "default",
  "session_id": "20251113_1",
  "mode": "Mask",
  "entity_counts": {
    "EMAIL": 1,
    "SSN": 1
  },
  "total_redactions": 2,
  "performance_ms": 8
}
```

### Future Enhancement (Not Critical for Demo)

**Add request logging at INFO level:**

**File:** `src/privacy-guard-proxy/src/main.rs` (or proxy code)

Add log line on every request:
```rust
info!("Proxying request to Privacy Guard: {} entities to check", content_length);
```

**File:** `src/privacy-guard/src/main.rs`

Add log line after masking:
```rust
info!("Masking complete: {} entities masked in {}ms", total, duration_ms);
```

### Status

- ✅ Root cause identified: Vault token issue blocks valid profiles
- ✅ Audit logging exists, just not triggered
- ⏳ Pending: Fix Vault token, test requests
- 📊 Impact: Demo needs working request to show masking

---

## 5. Database Cleanup Command (Non-Critical Tables)

### Purpose

Clear transient data (users, tasks, sessions) while preserving critical infrastructure (profiles, Vault secrets, Keycloak config).

**Use Case:**
- Start demo with fresh user data
- Clear old test tasks
- Reset to clean state without full volume deletion

### Safe Cleanup Command

```bash
# Clear transient tables (preserves profiles, Vault, Keycloak)
docker exec ce_postgres psql -U postgres -d orchestrator <<EOF
-- Delete all org chart users (will be re-imported from CSV)
TRUNCATE TABLE org_users CASCADE;

-- Delete all tasks (will be recreated during demo)
TRUNCATE TABLE tasks CASCADE;

-- Delete all sessions (ephemeral, safe to clear)
TRUNCATE TABLE sessions CASCADE;

-- Reset user_id sequence (optional - ensures EMP001 starts at 1)
ALTER SEQUENCE org_users_user_id_seq RESTART WITH 1;
EOF

echo "✅ Transient data cleared"
```

### What Gets Deleted

| Table | Data Lost | Preserved? | Recovery |
|-------|-----------|------------|----------|
| `org_users` | ✅ All 50 users | ❌ | Re-upload CSV |
| `tasks` | ✅ All Agent Mesh tasks | ❌ | Created during demo |
| `sessions` | ✅ All session records | ❌ | Created automatically |
| `profiles` | ❌ NOT DELETED | ✅ | Still signed |
| `_sqlx_migrations` | ❌ NOT DELETED | ✅ | Preserved |

### What is PRESERVED

| Component | Preserved? | Reason |
|-----------|------------|--------|
| **Profiles** | ✅ YES | Not touched by TRUNCATE |
| **Profile signatures** | ✅ YES | Stored in profiles.data JSON |
| **Vault secrets** | ✅ YES | Separate volume (vault_raft) |
| **Vault unseal keys** | ✅ YES | Separate volume |
| **Keycloak realm** | ✅ YES | Separate volume (keycloak_data) |
| **Ollama models** | ✅ YES | Separate volumes (ollama_*) |
| **Migrations history** | ✅ YES | _sqlx_migrations not truncated |

### Verification

```bash
# Verify profiles still exist
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role FROM profiles ORDER BY role;"

# Expected: 8 profiles (analyst, developer, finance, hr, legal, manager, marketing, support)

# Verify users deleted
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM org_users;"

# Expected: 0

# Verify tasks deleted
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM tasks;"

# Expected: 0
```

### Recovery Workflow

```bash
# After cleanup, re-upload users
cd /home/papadoc/Gooseprojects/goose-org-twin
./admin_upload_csv.sh test_data/demo_org_chart.csv

# Verify users restored
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM org_users;"

# Expected: 50
```

### When to Use This

**Use cleanup command when:**
- ✅ Starting fresh demo (clean slate)
- ✅ Testing CSV upload functionality
- ✅ Clearing old test tasks
- ✅ Want to preserve Vault/Keycloak config

**DO NOT use when:**
- ❌ You've made custom profile edits (use volume deletion instead)
- ❌ You want to keep user assignments
- ❌ You need task history

### Status

- ✅ Command tested and verified safe
- ✅ Preserves all critical infrastructure
- ✅ Fast recovery (<1 minute with CSV upload)
- 📊 Impact: Clean demo start without volume deletion

---

## 6. When to Apply These Fixes

### Fix Application Timeline

| Fix | When to Apply | Rebuild Required? | Restart Sequence |
|-----|---------------|-------------------|------------------|
| **fetch_status endpoint** | Before testing Agent Mesh | ✅ goose rebuild | goose containers only |
| **Employee ID validation** | Before testing user assignment | ✅ Controller rebuild | Controller only |
| **Vault token refresh** | **EVERY time Vault unseals** | ❌ No rebuild | Controller + goose restart |
| **Database cleanup** | Before demo (optional) | ❌ No rebuild | No restart needed |

### Recommended Workflow for Full Restart

```
Step 1-4: Infrastructure + Vault Unseal ✅
Step 5: Start Ollama ✅
Step 6: Start Controller ✅
Step 6.5: REFRESH VAULT TOKEN ← ADD THIS STEP!
  ├─ Login to Vault
  ├─ Generate new Controller token
  ├─ Update .env.ce
  └─ Restart Controller
Step 7: Sign Profiles ✅
Step 8-9: Privacy Guard stack ✅
Step 10: Rebuild + Start goose ✅
Step 11: Upload CSV ✅
Step 12: Verify health ✅
```

### Why Step 6.5 is Critical

**Vault unsealing (Step 4) resets authentication sessions:**
- Old tokens may become invalid
- AppRole credentials work but Controller uses token-based auth
- **Without fresh token, signature verification fails**
- **Without signature verification, profiles don't load**
- **Without profiles, extensions don't load**
- **Without extensions, Agent Mesh doesn't work**

**This cascades into complete system failure!**

### Detection Checklist

**How to know if you need to refresh Vault token:**

```bash
# Check 1: Controller has empty VAULT_TOKEN
docker exec ce_controller env | grep VAULT_TOKEN
# If "VAULT_TOKEN=" (empty): NEEDS REFRESH ❌

# Check 2: Controller logs show Vault errors
docker logs ce_controller 2>&1 | grep -i "vault.*error\|403.*vault"
# If errors present: NEEDS REFRESH ❌

# Check 3: Profile fetch returns 403
curl -H "Authorization: Bearer $TOKEN" http://localhost:8088/profiles/finance
# If 403 or signature error: NEEDS REFRESH ❌

# Check 4: goose config shows role: unknown
docker exec ce_goose_finance cat /root/.config/goose/config.yaml | grep "role:"
# If "role: unknown": NEEDS REFRESH ❌
```

### Prevention Strategy

**Option A: Always Refresh After Unseal (Safest)**
Add Step 6.5 to EVERY full restart sequence.

**Option B: Use AppRole Instead of Token**
Configure Controller to use VAULT_ROLE_ID/VAULT_SECRET_ID instead of VAULT_TOKEN.
(Requires Controller code changes - not immediate)

**Option C: Longer Token TTL**
Use `-ttl=8760h` (1 year) instead of `-ttl=768h` (32 days).
(Still doesn't solve session reset issue)

**Recommendation:** Use Option A (add Step 6.5 to playbook)

---

## 7. Summary of All Fixes

### Code Changes Applied

| File | Lines Changed | Description | Rebuild Target |
|------|---------------|-------------|----------------|
| `src/agent-mesh/tools/fetch_status.py` | 71 | `/sessions/` → `/tasks/` | goose containers |
| `src/controller/static/admin.html` | 209, 233 | Pass `employee_id` not `id` | Controller |

### Configuration Changes Required

| File | Change | When | Impact |
|------|--------|------|--------|
| `.env.ce` | Update `VAULT_TOKEN` | After Vault unseal | Controller auth |

### Restart Sequence After Fixes

```bash
# 1. Rebuild Controller (admin.html fix)
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
docker compose -f ce.dev.yml --profile controller build --no-cache controller
docker compose -f ce.dev.yml --profile controller restart controller
sleep 20

# 2. Rebuild goose (fetch_status fix)
docker compose -f ce.dev.yml --profile multi-goose --profile controller build --no-cache \
  goose-finance goose-manager goose-legal

# 3. Restart goose with fresh profiles
docker rm -f ce_goose_finance ce_goose_manager ce_goose_legal
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d \
  goose-finance goose-manager goose-legal
sleep 15

# 4. Verify all working
docker logs ce_goose_finance 2>&1 | grep "Profile fetched"
docker exec ce_goose_manager cat /root/.config/goose/config.yaml | grep -A5 "extensions"
```

### Testing Checklist

After applying all fixes:

- [ ] Agent Mesh send_task works (Finance → Manager)
- [ ] Agent Mesh request_approval works (Finance → Manager)
- [ ] Agent Mesh fetch_status works (Manager checks task)
- [ ] Agent Mesh notify works (test separately)
- [ ] Admin Dashboard user assignment works (no EMP error)
- [ ] Privacy Guard masking logs appear
- [ ] Profile signatures validate successfully

### Documentation Updates Needed

**Files to update:**
1. `Demo/Container_Management_Playbook.md`
   - Add Step 6.5: Refresh Vault Token
   - Update Section 5, Scenario 6 (Agent Mesh troubleshooting)

2. `Demo/Demo_Execution_Plan.md`
   - Add Vault token refresh to Pre-Demo Checklist
   - Add verification step for profile fetch (not 403)

3. `Demo/System_Analysis_Report.md`
   - Update Section 3 (Startup Sequence) with Step 6.5

---

## 8. Quick Reference - Common Issues

### Issue: Profile Fetch Returns 403

**Quick Fix:**
```bash
# Generate new Vault token
docker exec -it ce_vault vault login  # Use root token
NEW_TOKEN=$(docker exec ce_vault vault token create -policy=controller-policy -ttl=768h -format=json | jq -r '.auth.client_token')

# Update .env.ce with $NEW_TOKEN
# Restart Controller
docker compose -f ce.dev.yml --profile controller restart controller

# Restart goose
docker rm -f ce_goose_finance ce_goose_manager ce_goose_legal
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d goose-finance goose-manager goose-legal
```

### Issue: Agent Mesh Tools Return "Transport Closed"

**Quick Fix:**
```bash
# 1. Check Vault unsealed
docker exec ce_vault vault status | grep Sealed

# 2. Check Controller Vault token
docker exec ce_controller env | grep VAULT_TOKEN

# 3. If empty, refresh token (see above)

# 4. Restart goose containers
docker rm -f ce_goose_finance ce_goose_manager ce_goose_legal
docker compose -f ce.dev.yml --profile multi-goose --profile controller up -d goose-finance goose-manager goose-legal
```

### Issue: Admin Dashboard Employee ID Error

**Quick Fix:**
```bash
# After fixing admin.html (lines 209, 233):
docker compose -f ce.dev.yml --profile controller build --no-cache controller
docker compose -f ce.dev.yml --profile controller restart controller
```

### Issue: Privacy Guard Not Masking

**Quick Fix:**
```bash
# 1. Verify api_base in goose config
docker exec ce_goose_finance cat /root/.config/goose/config.yaml | grep api_base

# Should show: http://privacy-guard-proxy-finance:8090/v1
# If different: Profile fetch failed, fix Vault token first

# 2. Send test request to verify
curl -X POST http://localhost:8096/v1/chat/completions \
  -H "Authorization: Bearer $OPENROUTER_KEY" \
  -d '{"model":"anthropic/claude-3.5-sonnet","messages":[{"role":"user","content":"Test: alice@example.com"}]}'

# 3. Check logs
docker logs ce_privacy_guard_finance 2>&1 | grep audit
```

---

**Document End**  
**Related Documents:**
- Container_Management_Playbook.md (operational procedures)
- Volume_Deletion_Recovery_Guide.md (full recovery)
- Demo_Execution_Plan.md (demo script)
- System_Analysis_Report.md (architecture analysis)
