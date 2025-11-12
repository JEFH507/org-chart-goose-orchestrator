# 🔧 CONTAINER MANAGEMENT PLAYBOOK

**Version:** 1.0  
**Date:** 2025-11-12  
**Phase:** 6 (Backend Integration & MVP Demo)

## Table of Contents
1. [Starting the System from Zero](#1-starting-the-system-from-zero)
2. [Restarting Individual Services](#2-restarting-individual-services)
3. [Applying Profile Changes](#3-applying-profile-changes)
4. [Admin JWT Token Management](#4-admin-jwt-token-management)
5. [Handling Service Failures](#5-handling-service-failures)
6. [Quick Reference Commands](#6-quick-reference-commands)

---

## 1. Starting the System from Zero

### Prerequisites Check

```bash
# Navigate to compose directory
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Verify .env.ce file exists
ls -la .env.ce
# Should show: -rw------- (permissions 600 for security)

# Verify required environment variables
grep -E "OIDC_CLIENT_SECRET|VAULT_TOKEN|OPENROUTER_API_KEY" .env.ce
# Should return 3 lines (secrets present)

# Check Docker daemon running
docker info > /dev/null 2>&1 && echo "✅ Docker running" || echo "❌ Docker not running"

# Check available disk space (need ~10GB)
df -h /var/lib/docker | awk 'NR==2 {print "Available:", $4}'
```

---

### Full System Startup (Step-by-Step)

#### Step 1: Clean Slate (Optional but Recommended for Demo)

```bash
# WARNING: This deletes ALL data!
# Only run if you want a fresh start

# Stop all containers
docker compose -f ce.dev.yml --profile controller --profile privacy-guard \
  --profile privacy-guard-proxy --profile ollama --profile multi-goose \
  --profile redis down

# Optional: Remove volumes (fresh database)
# CAUTION: This deletes all users, profiles, tasks, sessions!
docker volume rm compose_postgres_data compose_vault_raft 2>/dev/null || true

echo "✅ System cleaned"
```

**Data Loss Warning:**
- `compose_postgres_data` - Loses all users, profiles, tasks, sessions
- `compose_vault_raft` - Loses all secrets, signatures (requires re-init)
- **If you want to preserve data, skip volume deletion!**

#### Step 2: Start Infrastructure Layer

```bash
# Start postgres, keycloak, vault, redis
docker compose -f ce.dev.yml up -d postgres keycloak vault redis

# Wait for health checks
echo "Waiting for infrastructure to be healthy (45s)..."
for i in {1..45}; do
  sleep 1
  echo -n "."
done
echo ""

# Verify all healthy
docker compose -f ce.dev.yml ps postgres keycloak vault redis
# All should show "healthy" status
```

**Expected Output:**
```
NAME           IMAGE                     STATUS
ce_postgres    postgres:17.2-alpine      Up 45s (healthy)
ce_keycloak    quay.io/keycloak:26.0.4   Up 45s (healthy)
ce_vault       hashicorp/vault:1.18.3    Up 45s (healthy)
ce_redis       redis:7.4.1-alpine        Up 45s (healthy)
```

#### Step 3: Unseal Vault (CRITICAL)

```bash
# Navigate to project root
cd ../..

# Run unseal script
./scripts/unseal_vault.sh

# You will be prompted for 3 unseal keys
# Keys are in .env.ce or from initial Vault init

# Verification:
docker exec ce_vault vault status
# Should show: "Sealed: false"
```

**Troubleshooting Vault Unsealing:**
- If script not found: `chmod +x scripts/unseal_vault.sh`
- If keys don't work: Check `.env.ce` for `VAULT_UNSEAL_KEY_*`
- If Vault sealed: Requires 3 of 5 Shamir keys to unseal
- If keys lost: You must re-initialize Vault (all secrets lost)

**Vault Unsealing Procedure (Manual if Script Fails):**
```bash
# Enter unsealing mode
docker exec -it ce_vault vault operator unseal

# Paste first key, press Enter
# Repeat for second key
# Repeat for third key

# After 3 keys, Vault will unseal
# Verify:
docker exec ce_vault vault status | grep "Sealed"
# Should show: "Sealed: false"
```

#### Step 4: Initialize Database (if Fresh Start)

```bash
# Navigate back to compose directory
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# If you removed postgres_data volume, database is empty
# Check if migrations needed:
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM profiles;" 2>/dev/null || echo "Database needs initialization"

# If error, database doesn't exist yet - migrations will run automatically
# Controller applies migrations on startup
```

**Migrations Applied Automatically:**
- 0001: Initial schema (sessions, org_users tables)
- 0002: Add profiles table
- 0003-0006: Profile seeding and enhancements
- 0007: Session lifecycle (FSM columns)
- 0008: Tasks table (Agent Mesh persistence)
- 0009: assigned_profile column (user profile assignment)

#### Step 5: Sign Profiles (Required for Security)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Run signing script
./scripts/sign-all-profiles.sh

# Expected output:
# Successfully signed: 8
# Already signed: 0
# Failed: 0
```

**What This Does:**
- Fetches all 8 profiles from database
- Uses Vault Transit engine to generate HMAC signature (sha2-256)
- Stores signature in database (profile.signature.signature field)
- Controller verifies signatures on every profile fetch (prevents tampering)

**If Signing Fails:**
- Check Vault unsealed: `docker exec ce_vault vault status`
- Check Vault token valid: `echo $VAULT_TOKEN`
- Check Controller has correct VAULT_TOKEN in environment
- Restart Controller if token was updated

#### Step 6: Start Ollama Instances

```bash
cd deploy/compose

# Start all 3 Ollama instances
docker compose -f ce.dev.yml --profile ollama --profile multi-goose up -d \
  ollama-finance ollama-manager ollama-legal

# Wait for model pull (first time: ~2GB download per instance)
echo "Waiting for Ollama instances (30s)..."
sleep 30

# Verify health
docker compose -f ce.dev.yml ps ollama-finance ollama-manager ollama-legal
```

**Note:** First startup downloads qwen3:0.6b model (~2GB per instance = 6GB total).  
Subsequent startups are fast (~5s).

**Verify Models Loaded:**
```bash
docker exec ce_ollama_finance ollama list
docker exec ce_ollama_manager ollama list
docker exec ce_ollama_legal ollama list
# Each should show: qwen3:0.6b
```

#### Step 7: Start Controller

```bash
# Start Controller
docker compose -f ce.dev.yml --profile controller up -d controller

# Wait for health check
echo "Waiting for Controller (20s)..."
sleep 20

# Verify healthy
curl -s http://localhost:8088/status | jq '.'
# Should return: {"status": "healthy"}
```

**Verification:**
```bash
# Check Controller logs for successful startup
docker logs ce_controller --tail=20

# Should see:
# ✅ "Database connection pool established"
# ✅ "Vault AppRole authentication successful" OR "Vault token authentication successful"
# ✅ "Session lifecycle initialized"
# ✅ "Server listening on 0.0.0.0:8088"
```

**If Controller Fails to Start:**
```bash
# Check logs for errors
docker logs ce_controller | grep -i error

# Common issues:
# - Vault sealed: Unseal Vault (Step 3)
# - Database not ready: Wait longer, check postgres health
# - Invalid Vault token: Update VAULT_TOKEN in .env.ce, restart
```

#### Step 8: Start Privacy Guard Services

```bash
# Start all 3 Privacy Guard Services
docker compose -f ce.dev.yml --profile multi-goose up -d \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal

# Wait for health checks
echo "Waiting for Privacy Services (25s)..."
sleep 25

# Verify all healthy
docker compose -f ce.dev.yml ps | grep privacy-guard | grep -v proxy
```

**Expected Output:**
```
ce_privacy_guard_finance   Up 25s (healthy)
ce_privacy_guard_manager   Up 25s (healthy)
ce_privacy_guard_legal     Up 25s (healthy)
```

**Verify Detection Methods:**
```bash
# Finance: Rules-only (GUARD_MODEL_ENABLED=false)
docker logs ce_privacy_guard_finance | grep -i "model enabled"
# Should show: "Model detection: disabled"

# Manager: Hybrid (GUARD_MODEL_ENABLED=true)
docker logs ce_privacy_guard_manager | grep -i "model enabled"
# Should show: "Model detection: enabled"

# Legal: AI-only (GUARD_MODEL_ENABLED=true)
docker logs ce_privacy_guard_legal | grep -i "model enabled"
# Should show: "Model detection: enabled"
```

#### Step 9: Start Privacy Guard Proxies

```bash
# Start all 3 Privacy Guard Proxies
docker compose -f ce.dev.yml --profile multi-goose up -d \
  privacy-guard-proxy-finance privacy-guard-proxy-manager privacy-guard-proxy-legal

# Wait for health checks
echo "Waiting for Proxies (20s)..."
sleep 20

# Verify all healthy
docker compose -f ce.dev.yml ps | grep proxy
```

**Expected Output:**
```
ce_privacy_guard_proxy_finance   Up 20s (healthy)
ce_privacy_guard_proxy_manager   Up 20s (healthy)
ce_privacy_guard_proxy_legal     Up 20s (healthy)
```

**Verify Control Panels Accessible:**
```bash
# Finance Control Panel
curl -s http://localhost:8096/ui | grep -i "privacy" && echo "✅ Finance UI accessible"

# Manager Control Panel
curl -s http://localhost:8097/ui | grep -i "privacy" && echo "✅ Manager UI accessible"

# Legal Control Panel
curl -s http://localhost:8098/ui | grep -i "privacy" && echo "✅ Legal UI accessible"
```

#### Step 10: Rebuild & Start Goose Instances (CRITICAL)

```bash
# IMPORTANT: Rebuild with --no-cache to ensure latest code
docker compose -f ce.dev.yml --profile multi-goose build --no-cache \
  goose-finance goose-manager goose-legal

# Expected: 3-5 minutes build time

# Start all 3 Goose instances
docker compose -f ce.dev.yml --profile multi-goose up -d \
  goose-finance goose-manager goose-legal

# Wait for profile fetch
echo "Waiting for Goose instances (15s)..."
sleep 15

# Verify running (no health check on Goose containers)
docker compose -f ce.dev.yml ps goose-finance goose-manager goose-legal
```

**Verification:**
```bash
# Check Finance profile fetch
docker logs ce_goose_finance | grep "Profile fetched"
# Should see: "Profile fetched successfully: finance"

# Check Manager profile fetch
docker logs ce_goose_manager | grep "Profile fetched"
# Should see: "Profile fetched successfully: manager"

# Check Legal profile fetch
docker logs ce_goose_legal | grep "Profile fetched"
# Should see: "Profile fetched successfully: legal"
```

**If Profile Fetch Fails:**
```bash
# Check Controller accessible from container
docker exec ce_goose_finance curl -s http://controller:8088/status

# Check JWT token acquisition
docker logs ce_goose_finance | grep -i "jwt"

# Check profile exists in database
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role FROM profiles WHERE role='finance';"
```

#### Step 11: Upload Organization Chart (50 Users)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Upload CSV with 50 test users
./admin_upload_csv.sh test_data/demo_org_chart.csv

# Expected output:
# ✅ Successfully imported! Created: 0, Updated: 50
```

**Verify Users in Database:**
```bash
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM org_users;"

# Should return: 50
```

#### Step 12: Final System Health Check

```bash
# Check all services
docker compose -f ce.dev.yml ps

# Expected: 17 containers running
# - 4 infrastructure (postgres, keycloak, vault, redis)
# - 3 Ollama (finance, manager, legal)
# - 1 Controller
# - 3 Privacy Guard Services
# - 3 Privacy Guard Proxies
# - 3 Goose instances

# Verify critical endpoints
curl -s http://localhost:8088/status | jq -r '.status'  # Should: healthy
curl -s http://localhost:8096/api/status | jq -r '.status'  # Should: healthy
curl -s http://localhost:8097/api/status | jq -r '.status'  # Should: healthy
curl -s http://localhost:8098/api/status | jq -r '.status'  # Should: healthy
```

**Success Criteria:**
- ✅ All 17 containers running
- ✅ All health checks passing
- ✅ Vault unsealed
- ✅ Profiles signed (8 profiles)
- ✅ Users imported (50 users)
- ✅ Admin Dashboard accessible (http://localhost:8088/admin)

### Total Startup Time: ~4-5 minutes (first time: ~10 min with model downloads)

---

## 2. Restarting Individual Services

### When to Restart Each Service

| Service | When to Restart | Impact | Recovery Time |
|---------|----------------|--------|---------------|
| Controller | After code changes, config updates | ~20s downtime, all Goose instances disconnect | 20s |
| Goose instance | After profile changes in database | No impact on other instances | 15s |
| Privacy Proxy | After detection method changes | Brief request failures (~5s) | 10s |
| Privacy Service | After model changes | Longer startup (~20s), cascades to Proxy | 25s |
| Postgres | After schema changes (rare) | Full system restart required | N/A |
| Vault | After policy changes, unsealed | Must re-unseal! All services fail until unsealed | 2 min |
| Keycloak | After realm changes | JWT tokens invalidated, re-auth required | 30s |

### Controller Restart

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Rebuild Controller (if code changed)
docker compose -f ce.dev.yml --profile controller build --no-cache controller

# Restart
docker compose -f ce.dev.yml --profile controller restart controller

# Wait for healthy
sleep 20

# Verify
curl -s http://localhost:8088/status | jq '.'

# Check logs for errors
docker logs ce_controller --tail=50 | grep -i error
```

**Expected Downtime:** ~20 seconds  
**Impact:** All Goose instances will reconnect automatically  
**Verification:** Check Goose logs for "Profile fetched successfully"

### Goose Instance Restart (e.g., Finance)

```bash
# Stop instance
docker compose -f ce.dev.yml --profile multi-goose stop goose-finance

# Optional: Rebuild if code changed
docker compose -f ce.dev.yml --profile multi-goose build --no-cache goose-finance

# Start instance
docker compose -f ce.dev.yml --profile multi-goose up -d goose-finance

# Verify profile fetch
sleep 10
docker logs ce_goose_finance | grep "Profile fetched"
```

**Expected Downtime:** ~15 seconds  
**Impact:** Only affects Finance users, Manager/Legal unaffected

### Privacy Guard Proxy Restart

```bash
# Restart Proxy (e.g., Finance)
docker compose -f ce.dev.yml --profile multi-goose restart privacy-guard-proxy-finance

# Wait for healthy
sleep 10

# Verify
curl -s http://localhost:8096/api/status | jq '.'
```

**Expected Downtime:** ~10 seconds  
**Impact:** Brief LLM request failures (auto-retry should work)

### Privacy Guard Service Restart

```bash
# Restart Service (e.g., Manager)
docker compose -f ce.dev.yml --profile multi-goose restart privacy-guard-manager

# This will cascade to Proxy restart
sleep 20

# Verify both
curl -s http://localhost:8094/status | jq '.'  # Service
curl -s http://localhost:8097/api/status | jq '.'  # Proxy
```

**Expected Downtime:** ~25 seconds  
**Impact:** LLM requests fail during restart, then auto-recover

---

## 3. Applying Profile Changes

### Profile Change Workflow

```
1. Admin edits profile in Admin Dashboard (http://localhost:8088/admin)
   ↓
2. Profile saved to PostgreSQL database
   ↓
3. Goose container MUST restart to fetch new profile
   ↓
4. Goose entrypoint fetches profile from Controller API
   ↓
5. Python script generates new config.yaml
   ↓
6. Goose session starts with new configuration
```

### Step-by-Step Procedure

#### Step 1: Edit Profile in Admin Dashboard

```bash
# Open browser
xdg-open http://localhost:8088/admin

# In Profile Management section:
# 1. Select profile (e.g., "Finance")
# 2. Edit JSON in textarea (e.g., change privacy_mode to "strict")
# 3. Click "Save Profile Changes"
# 4. Wait for success message: "✅ Profile saved successfully"
```

**Alternatively: Download/Edit/Upload Method:**
```bash
# In Admin Dashboard:
# 1. Select profile (e.g., "Finance")
# 2. Click "Download Profile JSON"
# 3. Edit downloaded file in text editor
# 4. Click "Upload Profile JSON"
# 5. Select edited file
# 6. Click "Save Profile Changes"
```

#### Step 2: Verify Profile Saved to Database

```bash
# Query database to confirm change
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role, data->'privacy'->>'guard_mode' FROM profiles WHERE role='finance';"

# Expected output:
#   role   | guard_mode 
# ---------+------------
#  finance | strict
```

#### Step 3: Restart Affected Goose Container

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Stop Finance instance
docker compose -f ce.dev.yml --profile multi-goose stop goose-finance

# Start Finance instance
docker compose -f ce.dev.yml --profile multi-goose up -d goose-finance

# Wait for profile fetch
sleep 15
```

#### Step 4: Verify New Profile Loaded

```bash
# Check logs for profile fetch
docker logs ce_goose_finance | tail -50 | grep -A 5 "Profile fetched"

# Should see:
# "Profile fetched successfully: finance"
# "Generating config from profile"
# "Config written to /root/.config/goose/config.yaml"

# Verify config file contains new settings
docker exec ce_goose_finance cat /root/.config/goose/config.yaml | grep -A 3 "privacy"

# Should show updated privacy_mode
```

#### Step 5: Test in Goose Session

```bash
# Start interactive session
docker exec -it ce_goose_finance goose session

# In Goose prompt, test privacy behavior:
# > "Analyze this text: My email is john@example.com"

# Expected: Email should be masked (if strict mode)
```

### Profile Change Matrix

| Change Type | Requires Restart? | Test Method |
|-------------|-------------------|-------------|
| privacy_mode | ✅ Yes | Send test prompt with PII |
| extensions | ✅ Yes | Check tool availability |
| policies | ✅ Yes | Test restricted action |
| display_name | ❌ No (metadata only) | N/A |
| api_base | ✅ Yes | Check LLM provider logs |
| detection_method | ❌ No (Proxy handles live) | Change in Control Panel UI |

### Bulk Profile Changes (All 3 Instances)

```bash
# If you changed multiple profiles, restart all:
docker compose -f ce.dev.yml --profile multi-goose restart \
  goose-finance goose-manager goose-legal

# Wait for all profile fetches
sleep 20

# Verify all
for role in finance manager legal; do
  echo "=== $role ==="
  docker logs ce_goose_$role | grep "Profile fetched" | tail -1
done
```

---

## 4. Admin JWT Token Management

### Why JWT Tokens Are Needed

The Admin Dashboard uses JWT tokens for authentication on protected endpoints:
- CSV upload (`POST /admin/org/import`)
- User profile assignment (`POST /admin/users/:id/assign-profile`)
- Profile publishing (`POST /admin/profiles/:role/publish`)

**Token Expiration:** 10 hours (36000 seconds) for development

### Getting a New JWT Token

#### Option 1: Using Helper Script (Recommended)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Run helper script
./get_admin_token.sh
```

**Script Output:**
```
🔐 Generating JWT Token for Admin Dashboard...

✅ Token generated (valid for 10 hours)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 COPY THIS TOKEN:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MzE0MjE...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💡 How to use in Browser Console (Admin Dashboard):
1. Open http://localhost:8088/admin
2. Press F12 to open Developer Console
3. Paste this code:

localStorage.setItem('admin_token', 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...');

4. Refresh the page - uploads will now work!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### Option 2: Manual Token Generation

```bash
# Extract client secret from Controller environment
OIDC_CLIENT_SECRET=$(docker exec ce_controller env | grep OIDC_CLIENT_SECRET | cut -d= -f2)

# Request token from Keycloak
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=${OIDC_CLIENT_SECRET}" \
  | jq -r '.access_token')

# Display token
echo "Token: $TOKEN"
```

### Setting JWT Token in Browser

**Step-by-Step:**

1. **Open Admin Dashboard:**
   ```bash
   xdg-open http://localhost:8088/admin
   ```

2. **Open Browser Developer Console:**
   - Press `F12` (or `Ctrl+Shift+I` on Linux/Windows, `Cmd+Option+I` on Mac)
   - Click "Console" tab

3. **Paste localStorage Command:**
   ```javascript
   localStorage.setItem('admin_token', 'YOUR_TOKEN_HERE');
   ```
   Replace `YOUR_TOKEN_HERE` with actual token from script output

4. **Verify Token Stored:**
   ```javascript
   console.log(localStorage.getItem('admin_token'));
   // Should print your token
   ```

5. **Refresh Page:**
   - Press `F5` or `Ctrl+R`
   - Admin Dashboard will now use token for authenticated requests

### Token Storage Details

**Where is the token stored?**
- Browser localStorage (key: `admin_token`)
- Persists across browser sessions
- Specific to `http://localhost:8088` origin
- **Not sent to server automatically** - JavaScript in admin.html includes it in requests

**Token Auto-Usage:**
- Admin dashboard JavaScript reads token from localStorage
- Adds `Authorization: Bearer <token>` header to protected API calls
- CSV upload, profile assignment, config push all use this token

**Token Expiration Handling:**
- Tokens expire after 10 hours
- No auto-refresh (manual for demo)
- When expired: CSV upload returns 401 Unauthorized
- Solution: Get new token with `./get_admin_token.sh` and update localStorage

### Troubleshooting JWT Tokens

**Problem: CSV Upload Returns 401 Unauthorized**

```bash
# Check if token is set in browser
# Open Console, run:
localStorage.getItem('admin_token')
# If null, token not set

# Generate new token
./get_admin_token.sh

# Set in browser localStorage (see steps above)
```

**Problem: Token Expired (After 10 Hours)**

```bash
# Symptoms: Was working, now 401 errors
# Solution: Generate new token

cd /home/papadoc/Gooseprojects/goose-org-twin
./get_admin_token.sh

# Copy new token
# Update in browser localStorage
```

**Problem: Script Shows Empty Token**

```bash
# Check Keycloak is running
curl -s http://localhost:8080 | grep -i keycloak
# Should return HTML with "Keycloak"

# Check client secret is set
docker exec ce_controller env | grep OIDC_CLIENT_SECRET
# Should show: OIDC_CLIENT_SECRET=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8

# If empty, check .env.ce file
grep OIDC_CLIENT_SECRET /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce
```

### Token Security Note

**For Development:**
- Client secret hardcoded in script (`elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8`)
- Acceptable for local dev
- Token stored in browser localStorage (not secure long-term storage)

**For Production:**
- Use OAuth2 authorization code flow (redirect-based)
- Implement proper token refresh mechanism
- Use httpOnly cookies (not localStorage)
- Rotate client secrets regularly

---

## 5. Handling Service Failures

### Failure Detection

```bash
# Check all container statuses
docker compose -f ce.dev.yml ps

# Look for:
# - "Restarting" status (crash loop)
# - "Exit" status (failed to start)
# - Missing containers (not started)

# Check Docker events for crashes
docker events --since 5m | grep -E "die|stop"
```

### Scenario 1: Vault Sealed

**Symptoms:**
- Controller logs: "Vault authentication failed"
- API calls return 503 errors
- Profile fetch fails with "Vault error"

**Detection:**
```bash
docker exec ce_vault vault status | grep Sealed
# If "true", Vault is sealed
```

**Resolution:**
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Run unseal script
./scripts/unseal_vault.sh

# Enter 3 of 5 unseal keys when prompted

# Verify unsealed
docker exec ce_vault vault status | grep Sealed
# Should show: "Sealed: false"

# Restart Controller to re-authenticate
cd deploy/compose
docker compose -f ce.dev.yml --profile controller restart controller

# Wait for Controller healthy
sleep 20

# Verify Controller can access Vault
curl -s http://localhost:8088/status | jq '.'
```

**Time to Recover:** ~2 minutes

**Prevention:**
- Vault seals on restart (by design for security)
- Always unseal Vault after system restart
- Consider auto-unseal in production (cloud KMS)

### Scenario 2: Database Connection Lost

**Symptoms:**
- Controller logs: "connection pool exhausted"
- API calls return 500 Internal Server Error
- Profile fetch timeouts

**Detection:**
```bash
# Check Postgres health
docker compose -f ce.dev.yml ps postgres
# Should show: "healthy"

# Test connection
docker exec ce_postgres pg_isready -U postgres
# Should show: "accepting connections"

# Check Controller logs
docker logs ce_controller --tail=100 | grep -i "database"
```

**Resolution:**
```bash
# Option 1: Restart Postgres (if unhealthy)
docker compose -f ce.dev.yml restart postgres
sleep 20

# Option 2: Restart Controller (if Postgres is healthy)
docker compose -f ce.dev.yml --profile controller restart controller
sleep 20

# Verify connection
curl -s http://localhost:8088/status | jq '.'

# Test database query
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM profiles;"
# Should return: 8
```

**Time to Recover:** ~40 seconds

### Scenario 3: JWT Tokens Expired

**Symptoms:**
- Goose logs: "401 Unauthorized"
- Controller logs: "Invalid JWT token"
- Agent Mesh calls fail with auth errors
- Admin Dashboard CSV upload returns 401

**Detection:**
```bash
# Check Keycloak health
curl -s http://localhost:8080 | grep -i keycloak
# Should return HTML

# Try to get new token
OIDC_CLIENT_SECRET=$(docker exec ce_controller env | grep OIDC_CLIENT_SECRET | cut -d= -f2)
curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=${OIDC_CLIENT_SECRET}" | jq -r '.access_token'

# Should return token (long alphanumeric string)
```

**Resolution:**

**For Goose Instances:**
```bash
# Tokens auto-refresh on container restart (10-hour expiration for dev)
docker compose -f ce.dev.yml --profile multi-goose restart \
  goose-finance goose-manager goose-legal

# Verify new tokens fetched
docker logs ce_goose_finance | grep "JWT token acquired"
```

**For Admin Dashboard:**
```bash
# Generate new admin token
cd /home/papadoc/Gooseprojects/goose-org-twin
./get_admin_token.sh

# Copy localStorage command from output
# Paste in browser console (F12)
# Refresh page
```

**If Keycloak is Down:**
```bash
docker compose -f ce.dev.yml restart keycloak
sleep 30

# Restart all services that need JWT
docker compose -f ce.dev.yml --profile controller restart controller
docker compose -f ce.dev.yml --profile multi-goose restart \
  goose-finance goose-manager goose-legal

# Get new admin token
./get_admin_token.sh
```

**Time to Recover:** ~30 seconds (auto-refresh), ~60 seconds (Keycloak restart)

### Scenario 4: Privacy Guard Not Responding

**Symptoms:**
- LLM requests hang/timeout
- Privacy Proxy logs: "Failed to connect to Privacy Guard Service"
- 504 Gateway Timeout errors

**Detection:**
```bash
# Check Privacy Service health
curl -s http://localhost:8093/status | jq '.'  # Finance
curl -s http://localhost:8094/status | jq '.'  # Manager
curl -s http://localhost:8095/status | jq '.'  # Legal

# Should all return: {"status": "healthy"}

# Check Proxy health
curl -s http://localhost:8096/api/status | jq '.'  # Finance
curl -s http://localhost:8097/api/status | jq '.'  # Manager
curl -s http://localhost:8098/api/status | jq '.'  # Legal
```

**Resolution:**
```bash
# Restart Privacy Service + Proxy (e.g., Finance)
docker compose -f ce.dev.yml --profile multi-goose restart \
  privacy-guard-finance privacy-guard-proxy-finance

# Wait for cascading health checks
sleep 30

# Verify both healthy
curl -s http://localhost:8093/status | jq '.'
curl -s http://localhost:8096/api/status | jq '.'

# Test LLM request (via Proxy)
curl -X POST http://localhost:8096/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{"model": "gpt-3.5-turbo", "messages": [{"role": "user", "content": "test"}]}'
```

**Time to Recover:** ~30 seconds

### Scenario 5: Goose Container Stuck

**Symptoms:**
- Container running but no logs
- No response to `goose session` command
- CPU usage high/low (frozen)

**Detection:**
```bash
# Check container status
docker ps | grep goose

# Check CPU usage
docker stats --no-stream ce_goose_finance

# Check logs for errors
docker logs ce_goose_finance --tail=100 | grep -i error
```

**Resolution:**
```bash
# Option 1: Restart container
docker compose -f ce.dev.yml --profile multi-goose restart goose-finance

# Wait for profile fetch
sleep 15

# Verify profile loaded
docker logs ce_goose_finance | grep "Profile fetched"

# Option 2: Force rebuild + restart (if code issue)
docker compose -f ce.dev.yml --profile multi-goose build --no-cache goose-finance
docker compose -f ce.dev.yml --profile multi-goose up -d goose-finance

# Option 3: Nuclear (if persistent issue)
docker compose -f ce.dev.yml --profile multi-goose stop goose-finance
docker container rm -f ce_goose_finance
docker compose -f ce.dev.yml --profile multi-goose up -d goose-finance
```

**Time to Recover:** ~20 seconds (restart), ~5 minutes (rebuild)

### Scenario 6: Agent Mesh "Transport Closed" Error

**Symptom:** Goose shows "Transport closed" when calling Agent Mesh MCP tools

**⚠️ ROOT CAUSE:** This is **95% of the time a Vault issue**, not a Goose bug!

**Complete Documentation Available:**
- `Technical Project Plan/PM Phases/Phase-6/docs/VAULT-FIX-SUMMARY.md`
- `Technical Project Plan/PM Phases/Phase-6/docs/PHASE6-D-BREAKTHROUGH.md`
- `Technical Project Plan/PM Phases/Phase-6/docs/MCP-EXTENSION-SUCCESS-SUMMARY.md`
- `Technical Project Plan/PM Phases/Phase-6/docs/D2_COMPLETION_SUMMARY.md`

**Detection:**
```bash
# Check Goose logs for the error
docker logs ce_goose_finance | grep -i "transport"

# Should see:
# "Error: Transport closed" OR "MCP server connection failed"
```

**PRIMARY RESOLUTION STEPS (95% Success Rate):**

#### Step 1: Check Vault Status (MOST COMMON CAUSE)
```bash
# Verify Vault is unsealed
docker exec ce_vault vault status | grep Sealed

# If "Sealed: true", Vault MUST be unsealed:
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/unseal_vault.sh
# Enter 3 of 5 unseal keys when prompted

# Verify unsealing succeeded:
docker exec ce_vault vault status
# Should show: "Sealed: false"
```

#### Step 2: Check Vault Authentication in Controller
```bash
# Check Controller logs for Vault errors
docker logs ce_controller | grep -i vault | grep -i error

# Common errors:
# ❌ "Vault HMAC verification failed"
# ❌ "403 Forbidden" 
# ❌ "Invalid token"
# ❌ "permission denied"

# If errors found, check if Controller has valid Vault token:
docker exec ce_controller env | grep VAULT_TOKEN

# Should show: VAULT_TOKEN=hvs.CAESI...
# If "dev-only-token" or empty: Controller needs fresh token (see VAULT-FIX-SUMMARY.md)
```

#### Step 3: Verify Profile Signatures Exist
```bash
# Check if all profiles are signed in database
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role, (data->'signature'->>'signature') IS NOT NULL AS has_signature FROM profiles ORDER BY role;"

# All 8 profiles should show: has_signature = t

# If any show 'f' (false) or NULL, re-sign profiles:
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/sign-all-profiles.sh

# Expected output:
# Successfully signed: 8
# Already signed: 0
# Failed: 0
```

#### Step 4: Restart Controller After Vault Fix
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Restart Controller to re-authenticate with Vault
docker compose -f ce.dev.yml --profile controller restart controller

# Wait for healthy
sleep 20

# Verify Controller Vault connection
docker logs ce_controller | tail -50 | grep -i vault

# Should see:
# "Vault AppRole authentication successful" OR
# "Vault token authentication successful"
# "Profile signature valid - no tampering detected"
```

#### Step 5: Restart Goose Containers to Reload Profiles
```bash
# Restart all Goose instances to fetch fresh signed profiles
docker compose -f ce.dev.yml --profile multi-goose restart \
  goose-finance goose-manager goose-legal

# Wait for profile fetch
sleep 20

# Verify profiles loaded successfully
for role in finance manager legal; do
  echo "=== $role ==="
  docker logs ce_goose_$role | grep "Profile fetched"
done

# Should see for each:
# "Profile fetched successfully: {role}"
```

#### Step 6: Verify MCP Extension Loaded
```bash
# Check if Agent Mesh MCP server subprocess is running
docker exec ce_goose_finance ps aux | grep agent_mesh

# Should see: root ... python3 -m agent_mesh_server

# Check Goose logs for successful extension loading
docker logs ce_goose_finance | grep -i "agent_mesh\|extension"

# Should see:
# "Loading extension: agent_mesh"
# "MCP server started: agent_mesh"
```

#### Step 7: Test Agent Mesh Tools
```bash
# Start Goose session
docker exec -it ce_goose_finance goose session

# Test tool availability:
# Goose> "What tools do I have available?"
# Should list: agentmesh__send_task, agentmesh__notify, agentmesh__request_approval, agentmesh__fetch_status

# Test sending a task:
# Goose> "Use agentmesh__send_task to send a budget approval request to manager for $50,000"
```

**Expected: Tool executes successfully, no "Transport closed" error**

---

**IF ALL ABOVE STEPS PASS AND STILL SEE "Transport Closed":**

Then it may be the **rare Goose CLI stdio bug** (5% of cases):

**Secondary Root Cause:** Goose CLI v1.13.1 stdio subprocess spawning limitation

**Investigation Results:**
- ✅ Config format correct (verified YAML valid)
- ✅ MCP server works manually: `python3 -m agent_mesh_server` succeeds
- ✅ Tools appear in tool list: `agentmesh__*` visible
- ✅ Vault unsealed and profiles signed
- ❌ Tool calls still fail with "Transport closed"

**Workarounds:**

**Option 1: Use Goose Desktop (Proven to Work)**
```bash
# Goose Desktop on host machine has no stdio issues
# All 4 tools work perfectly (100% success rate)

# Evidence: Testing session 2025-11-11 10:02-10:22 EST
# - agentmesh__send_task: ✅ Working
# - agentmesh__notify: ✅ Working
# - agentmesh__request_approval: ✅ Working
# - agentmesh__fetch_status: ✅ Working
```

**Option 2: Demonstrate via API (For Demo)**
```bash
# Get JWT token
OIDC_CLIENT_SECRET=$(docker exec ce_controller env | grep OIDC_CLIENT_SECRET | cut -d= -f2)
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d "grant_type=client_credentials" \
  -d "client_id=goose-controller" \
  -d "client_secret=${OIDC_CLIENT_SECRET}" \
  | jq -r '.access_token')

# Send task via API (proves Agent Mesh backend working)
curl -X POST http://localhost:8088/tasks/route \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $(uuidgen)" \
  -d '{
    "target": "manager",
    "task": {
      "task_type": "budget_approval",
      "description": "Demo task from Finance to Manager",
      "data": {"amount": 50000}
    }
  }'

# Verify task created and persisted
curl -H "Authorization: Bearer $TOKEN" "http://localhost:8088/tasks?target=manager&status=pending" | jq '.'

# Show task in database (proves persistence - migration 0008)
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT task_id, target, task_type, status FROM tasks ORDER BY created_at DESC LIMIT 5;"
```

**Option 3: Show Controller Logs (Proves Routing Works)**
```bash
# Watch Controller logs for task routing
docker logs -f ce_controller | grep "task.created"

# When task sent, will show:
# {"message":"task.created","task_id":"...","target":"manager","task_type":"budget_approval"}
```

**Time to Resolve:** 2-5 minutes (Vault fix), Immediate (use API workaround)  
**Success Rate:** 95% resolved by Vault fix, 5% need Goose Desktop/API workaround

---

**Key Insight from Phase 6 Testing:**
- "Transport closed" is almost always Vault-related
- Vault unsealing is the #1 fix
- Profile signatures are the #2 fix
- Controller Vault token refresh is the #3 fix
- Only after ALL Vault checks should you assume Goose CLI bug

---

## 6. Quick Reference Commands

### System Status

```bash
# Check all services
docker compose -f ce.dev.yml ps

# Check specific service health
docker compose -f ce.dev.yml ps postgres keycloak vault redis

# Check logs (last 50 lines)
docker logs ce_controller --tail=50

# Follow logs in real-time
docker logs -f ce_controller

# Check resource usage
docker stats --no-stream
```

### Emergency Procedures

```bash
# Full system restart (preserves data)
docker compose -f ce.dev.yml --profile controller --profile multi-goose \
  --profile redis restart

# Stop everything (preserves data)
docker compose -f ce.dev.yml --profile controller --profile multi-goose down

# Nuclear option (DELETE ALL DATA - use with caution!)
docker compose -f ce.dev.yml down -v
# This removes ALL volumes! Only for complete reset.
```

### Health Check Endpoints

```bash
# Controller
curl http://localhost:8088/status

# Admin Dashboard
curl http://localhost:8088/admin | grep -i "admin dashboard"

# Privacy Proxy (Finance)
curl http://localhost:8096/api/status

# Privacy Proxy (Manager)
curl http://localhost:8097/api/status

# Privacy Proxy (Legal)
curl http://localhost:8098/api/status

# Privacy Service (Finance)
curl http://localhost:8093/status

# Keycloak
curl http://localhost:8080

# Postgres
docker exec ce_postgres pg_isready -U postgres
```

### Log Shortcuts

```bash
# Tail all Controller logs with timestamps
docker logs -f --since 5m ce_controller 2>&1 | grep -v "sqlx::query"

# Show only errors
docker logs ce_controller | grep -i error

# Show Goose profile fetch
docker logs ce_goose_finance | grep "Profile fetched"

# Show Privacy Guard masking activity
docker logs ce_privacy_guard_finance | grep "mask"

# Show Agent Mesh task routing
docker logs ce_controller | grep "task.created"

# Show JWT token acquisition
docker logs ce_goose_finance | grep "JWT token acquired"
```

### Database Queries

```bash
# List all profiles
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role FROM profiles ORDER BY role;"

# Count users
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM org_users;"

# List tasks
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT task_id, target, task_type, status FROM tasks ORDER BY created_at DESC LIMIT 10;"

# Check profile signatures
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role, (data->'signature'->>'signature') IS NOT NULL AS has_signature FROM profiles;"
```

### Vault Commands

```bash
# Check Vault status
docker exec ce_vault vault status

# Unseal Vault (manual)
docker exec -it ce_vault vault operator unseal

# List transit keys
docker exec ce_vault vault list transit/keys

# Check profile signing key
docker exec ce_vault vault read transit/keys/profile-signing
```

### Admin Token Helpers

```bash
# Generate admin JWT token
./get_admin_token.sh

# Check token expiration
TOKEN=$(./get_admin_token.sh | grep -A1 "COPY THIS TOKEN" | tail -1)
echo $TOKEN | cut -d'.' -f2 | base64 -d 2>/dev/null | jq '.exp' | xargs -I {} date -d @{}

# Upload CSV
./admin_upload_csv.sh test_data/demo_org_chart.csv
```

---

## 7. Data Persistence Guarantee

### What is Preserved on Full Restart?

When you run the full startup sequence with `docker compose down` (NO `-v` flag):

**✅ PRESERVED (in volumes):**
- Postgres data (users, profiles, tasks, sessions)
- Vault data (secrets, signatures, policies)
- Keycloak config (realm, clients, users)
- Redis data (if using appendonly persistence)
- Ollama models (qwen3:0.6b in each instance)
- Goose workspaces (files created in sessions)

**❌ LOST (in-memory):**
- Active Goose sessions (expected - sessions are not persistent)
- Redis cache entries (expected - cache is ephemeral)
- In-flight HTTP requests (expected)

### What is Lost on Volume Deletion?

If you run `docker compose down -v` or manually delete volumes:

**Data Loss:**
- ALL users from CSV upload
- ALL profile changes made in Admin UI
- ALL tasks created via Agent Mesh
- ALL Vault secrets and signatures
- ALL Keycloak config

**Recovery:**
- Migrations will re-create tables
- Profile seeding migration will re-populate 8 default profiles
- You must re-upload CSV org chart
- You must re-sign profiles
- You must re-unseal Vault (if vault_raft deleted)

### Safe Restart Procedure (No Data Loss)

```bash
# Stop all containers (preserves volumes)
docker compose -f ce.dev.yml --profile controller --profile multi-goose down

# Rebuild images (only if code changed)
docker compose -f ce.dev.yml --profile multi-goose build --no-cache

# Follow full startup sequence from Step 2 onwards
# (See section 1: Starting the System from Zero)

# Result: All data preserved, only in-memory state lost
```

---

**Playbook End**  
**Next Document:** Demo Execution Plan
