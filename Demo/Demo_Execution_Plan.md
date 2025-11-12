# 🎬 DEMO EXECUTION PLAN

**Version:** 1.0  
**Date:** 2025-11-12  
**Phase:** 6 - Ready for Demo Validation  
**Demo Duration:** 15-20 minutes  
**Preparation Time:** 10 minutes

---

## Pre-Demo Checklist (Complete 30 Minutes Before)

### Infrastructure Preparation

```bash
# 1. Navigate to project
cd /home/papadoc/Gooseprojects/goose-org-twin

# 2. Full system restart (CRITICAL - ensures latest code)
cd deploy/compose
docker compose -f ce.dev.yml --profile controller --profile multi-goose down
sleep 5

# 3. Start infrastructure
docker compose -f ce.dev.yml up -d postgres keycloak vault redis
sleep 45

# 4. Unseal Vault
cd ../..
./scripts/unseal_vault.sh
# Enter 3 unseal keys when prompted

# 5. Start Ollama instances
cd deploy/compose
docker compose -f ce.dev.yml --profile ollama --profile multi-goose up -d \
  ollama-finance ollama-manager ollama-legal
sleep 30

# 6. Start Controller
docker compose -f ce.dev.yml --profile controller up -d controller
sleep 20

# 7. Sign profiles (CRITICAL)
cd ../..
./scripts/sign-all-profiles.sh

# 8. Start Privacy Guard stack
cd deploy/compose
docker compose -f ce.dev.yml --profile multi-goose up -d \
  privacy-guard-finance privacy-guard-manager privacy-guard-legal
sleep 25

docker compose -f ce.dev.yml --profile multi-goose up -d \
  privacy-guard-proxy-finance privacy-guard-proxy-manager privacy-guard-proxy-legal
sleep 20

# 9. Rebuild Goose images (CRITICAL)
docker compose -f ce.dev.yml --profile multi-goose build --no-cache \
  goose-finance goose-manager goose-legal

# 10. Start Goose instances
docker compose -f ce.dev.yml --profile multi-goose up -d \
  goose-finance goose-manager goose-legal
sleep 20

# 11. Verify all healthy
docker compose -f ce.dev.yml ps | grep -E "healthy|running"

# 12. Upload organization chart
cd ../..
./admin_upload_csv.sh test_data/demo_org_chart.csv

# 13. Generate admin JWT token
./get_admin_token.sh
# COPY the localStorage command for browser
```

**Total Preparation Time:** ~10 minutes

---

## Window Layout Configuration

### Screen Arrangement (6-Window Layout)

```
┌─────────────────────────────────────────────────────────────┐
│                    PRIMARY MONITOR                          │
├────────────────────────────┬────────────────────────────────┤
│  TERMINAL 1: Finance       │  BROWSER 1: Admin Dashboard   │
│  docker exec -it           │  http://localhost:8088/admin  │
│  ce_goose_finance          │                                │
│  goose session             │                                │
├────────────────────────────┼────────────────────────────────┤
│  TERMINAL 2: Manager       │  BROWSER 2: Finance Control   │
│  docker exec -it           │  http://localhost:8096/ui     │
│  ce_goose_manager          │                                │
│  goose session             │                                │
├────────────────────────────┼────────────────────────────────┤
│  TERMINAL 3: Legal         │  BROWSER 3: Manager Control   │
│  docker exec -it           │  http://localhost:8097/ui     │
│  ce_goose_legal            │                                │
│  goose session             │                                │
└────────────────────────────┴────────────────────────────────┘

SECONDARY MONITOR (Optional):
┌─────────────────────────────────────────────────────────────┐
│  BROWSER 4: Legal Control Panel                            │
│  http://localhost:8098/ui                                   │
├─────────────────────────────────────────────────────────────┤
│  BROWSER 5: Vault Dashboard                                │
│  https://localhost:8200/ui/vault/dashboard                  │
└─────────────────────────────────────────────────────────────┘
```

### Pre-Demo Window Setup

```bash
# Terminal 1: Finance Goose
gnome-terminal --window --geometry=120x40+0+0 --title="Finance Goose" -- \
  bash -c "cd /home/papadoc/Gooseprojects/goose-org-twin && echo 'Finance Goose Ready. Press Enter to start session...'; read; docker exec -it ce_goose_finance goose session"

# Terminal 2: Manager Goose
gnome-terminal --window --geometry=120x40+960+0 --title="Manager Goose" -- \
  bash -c "cd /home/papadoc/Gooseprojects/goose-org-twin && echo 'Manager Goose Ready. Press Enter to start session...'; read; docker exec -it ce_goose_manager goose session"

# Terminal 3: Legal Goose
gnome-terminal --window --geometry=120x40+0+600 --title="Legal Goose" -- \
  bash -c "cd /home/papadoc/Gooseprojects/goose-org-twin && echo 'Legal Goose Ready. Press Enter to start session...'; read; docker exec -it ce_goose_legal goose session"

# Browser windows (Firefox tabs)
firefox --new-window \
  "http://localhost:8088/admin" \
  "http://localhost:8096/ui" \
  "http://localhost:8097/ui" \
  "http://localhost:8098/ui" \
  "https://localhost:8200/ui/vault/dashboard"
```

---

## Demo Script Timeline

### Part 0: Introduction (2 minutes)

**Talking Points:**
- "Welcome to the Goose Orchestrator demo"
- "Enterprise-ready multi-agent system with privacy-first architecture"
- "6 windows showing: 3 Goose agents + 3 privacy control panels + admin dashboard"
- "All running locally - zero cloud dependencies"

**Show:**
- Point to each window
- Explain screen layout
- Mention database-driven configuration

---

### Part 1: System Architecture Overview (2 minutes)

**Show Admin Dashboard (Browser 1):**

1. **Quick Links Banner:**
   - Point to Keycloak, Vault, Privacy Guard, API Docs links
   - Explain: "All infrastructure integrated"

2. **Admin Dashboard Sections:**
   - CSV Upload: "Import organizational hierarchy"
   - User Management: "Assign profiles to users"
   - Profile Management: "Edit role-based configurations"
   - Config Push: "Deploy to all instances"
   - Live Logs: "Real-time system monitoring"

**Talking Points:**
- "Admin manages everything from this single UI"
- "All configuration stored in PostgreSQL database"
- "Changes persist across restarts"

---

### Part 2: CSV Organization Chart Upload (1 minute)

**Action:**
1. Click "CSV Upload" section
2. Click "Select CSV File" button
3. Choose: `/home/papadoc/Gooseprojects/goose-org-twin/test_data/demo_org_chart.csv`
4. Wait for upload (should be instant - already uploaded in prep)

**Expected Result:**
```
✅ Successfully imported! Created: 0, Updated: 50
```

**Talking Points:**
- "50 users imported from CSV"
- "3 departments: Finance, Legal, Operations"
- "Organizational hierarchy captured"

**If Already Uploaded:**
- "Already imported 50 users during setup"
- Scroll to User Management to show users

---

### Part 3: User Management & Profile Assignment (2 minutes)

**Show User Management Section:**

1. **Scroll through user list:**
   - Show 50 users visible
   - Point to Employee ID, Name, Email, Department columns

2. **Assign Profiles:**
   ```
   - EMP001 (Alice) → Finance
   - EMP002 (Bob) → Manager
   - EMP003 (Carol) → Legal
   ```

3. **Click "Assign Profile" buttons**

**Expected Result:**
- Green success messages: "✅ Profile assigned successfully"

**Talking Points:**
- "Admin controls which users get which roles"
- "Profile determines: privacy level, allowed tools, LLM access"
- "Assignment stored in database - survives restarts"

---

### Part 4: Profile Management (3 minutes)

**Show Profile Management Section:**

1. **Select "Finance" from dropdown**
   - JSON editor shows Finance profile configuration

2. **Highlight Key Fields:**
   ```json
   {
     "privacy": {
       "guard_mode": "auto",
       "content_handling": "mask"
     },
     "extensions": ["github", "agent_mesh", "memory", "excel-mcp"],
     "providers": {
       "api_base": "http://privacy-guard-proxy-finance:8090/v1"
     }
   }
   ```

3. **Explain Each Section:**
   - **Privacy:** Rules-only (< 10ms)
   - **Extensions:** Tools available to Finance role
   - **Providers:** Routed through Privacy Guard Proxy

4. **Demonstrate Download/Upload:**
   - Click "Download Profile JSON"
   - Show file saved
   - Explain: "Power users can edit offline"

5. **Create New Profile (Optional):**
   - Enter "executive" in "Create New Profile" field
   - Click "Create Profile"
   - Show default template appears
   - Don't save - just demonstrate feature

**Talking Points:**
- "8 profiles: analyst, developer, finance, hr, legal, manager, marketing, support"
- "All stored in PostgreSQL database"
- "All signed by Vault (tamper-proof)"
- "Changes require Goose container restart to apply"

---

### Part 5: Privacy Guard Control Panels (3 minutes)

**Show Finance Control Panel (Browser 2 - http://localhost:8096/ui):**

1. **Point to Current Mode:**
   - "Mode: Auto (Smart Detection)"
   - "Status: Healthy"

2. **Show Mode Options:**
   - Auto: Smart Detection (Recommended)
   - Bypass: No Masking (Use Caution)
   - Strict: Maximum Privacy

3. **Activity Log:**
   - Scroll through recent activity
   - Point to any PII detection events

**Show Manager Control Panel (Browser 3 - http://localhost:8097/ui):**

1. **Different Detection Method:**
   - "Mode: Hybrid (Balanced)"
   - Explain: "Combines regex + LLM fallback"

**Show Legal Control Panel (Optional - Browser 4):**

1. **Strictest Mode:**
   - "Mode: AI-Only (Most Thorough)"
   - Explain: "~15s latency but maximum compliance"

**Talking Points:**
- "Per-instance Privacy Guard - each role has isolated stack"
- "Finance: Rules-only (< 10ms) - fastest"
- "Manager: Hybrid (< 100ms typical)"
- "Legal: AI-only (~15s) - most thorough"
- "CPU isolation: Legal's 15s doesn't block Finance's 10ms"
- "All running locally on user's CPU"

---

### Part 6: Vault Integration Demo (1 minute)

**Open Vault Dashboard (Browser 5 - https://localhost:8200/ui/vault/dashboard):**

1. **Login with root token** (from prep or .env.ce)

2. **Navigate to Transit Engine:**
   - Click "Secrets" → "transit"
   - Click "profile-signing" key
   - Show key exists

3. **Explain:**
   - "All profiles cryptographically signed"
   - "HMAC with sha2-256"
   - "Controller verifies signatures on fetch"
   - "Prevents profile tampering"

**Talking Points:**
- "Vault stores secrets securely"
- "Profile signatures ensure integrity"
- "AppRole authentication (production-ready)"

---

### Part 7: Goose Session & Agent Mesh Demo (4 minutes)

**⚠️ CRITICAL: Agent Mesh MCP Troubleshooting**

If you encounter "Transport closed" error during demo, **this is 95% a Vault issue**, not a Goose bug!

**Complete Fix Documentation (Read in Sequence):**
1. `Technical Project Plan/PM Phases/Phase-6/docs/MCP-EXTENSION-SUCCESS-SUMMARY.md` - Initial MCP loading (signature disabled)
2. `Technical Project Plan/PM Phases/Phase-6/docs/VAULT-FIX-SUMMARY.md` - Vault token + policy fix
3. `Technical Project Plan/PM Phases/Phase-6/docs/PHASE6-D-BREAKTHROUGH.md` - Signature verification re-enabled
4. `Technical Project Plan/PM Phases/Phase-6/docs/D2_COMPLETION_SUMMARY.md` - Final working state

**Root Cause: Vault Transit Signing Failures**

The "Transport closed" error appears when MCP extension fails to load due to profile signature verification errors:
- **Vault sealed** (requires 3-of-5 Shamir key unsealing)
- **Invalid Vault token** in Controller (403 Forbidden)
- **Profiles not signed** with Vault Transit HMAC
- **Signature verification failing** due to token/policy issues

**Quick Diagnostic Steps:**

```bash
# 1. Check Vault status (MOST COMMON)
docker exec ce_vault vault status | grep Sealed
# If "true", run: ./scripts/unseal_vault.sh

# 2. Check Controller Vault authentication
docker logs ce_controller | grep -i vault | grep -i error
# Look for: "403 Forbidden", "Invalid token", "HMAC verification failed"

# 3. Verify profiles are signed
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT role, (data->'signature'->>'signature') IS NOT NULL FROM profiles;"
# All should be 't' (true)

# 4. If signatures missing, re-sign:
./scripts/sign-all-profiles.sh

# 5. Restart Controller + Goose containers
cd deploy/compose
docker compose -f ce.dev.yml --profile controller restart controller
sleep 20
docker compose -f ce.dev.yml --profile multi-goose restart goose-finance goose-manager goose-legal
sleep 20

# 6. Verify MCP loaded
docker exec ce_goose_finance ps aux | grep agent_mesh
# Should see: python3 -m agent_mesh_server running
```

**If After All Vault Fixes Still Fails (Rare):**
Then use API workaround below (Goose CLI stdio bug)

---

**Finance Terminal (Terminal 1):**

1. **Start Session** (if not already started)
   ```bash
   # Press Enter if waiting
   # Or run: docker exec -it ce_goose_finance goose session
   ```

2. **Test Privacy Masking:**
   ```
   Prompt: "Analyze this text: My email is alice@company.com and SSN is 123-45-6789"
   ```

   **Expected:** Email and SSN should be masked (if rules-only working)

3. **Test Agent Mesh (Option A: Try MCP Tool First):**
   ```
   Prompt: "Use the agentmesh__send_task tool to send a budget approval task to the manager role for $125,000 Q1 Engineering budget"
   ```

   **If this works:** ✅ MCP extension functional!
   
   **If "Transport closed" error:** Use Option B (API workaround)

4. **Test Agent Mesh (Option B: API Workaround):**
   ```bash
   # In separate terminal:
   OIDC_CLIENT_SECRET=$(docker exec ce_controller env | grep OIDC_CLIENT_SECRET | cut -d= -f2)
   TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
     -d "grant_type=client_credentials" \
     -d "client_id=goose-controller" \
     -d "client_secret=${OIDC_CLIENT_SECRET}" \
     | jq -r '.access_token')

   curl -X POST http://localhost:8088/tasks/route \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -H "Idempotency-Key: $(uuidgen)" \
     -d '{
       "target": "manager",
       "task": {
         "task_type": "budget_approval",
         "description": "Approve $125K Q1 Engineering budget",
         "data": {"amount": 125000, "department": "Engineering"}
       }
     }' | jq '.'
   ```

   **Expected:** Returns task_id and status: "accepted"

**Manager Terminal (Terminal 2):**

1. **Start Session**
2. **Query Tasks:**
   ```bash
   # API call to show task exists:
   curl -H "Authorization: Bearer $TOKEN" \
     "http://localhost:8088/tasks?target=manager&status=pending&limit=10" | jq '.'
   ```

   **Expected:** Shows Finance's task in pending state

3. **Verify Task Persistence:**
   ```bash
   # Show task is in database (survives restarts)
   docker exec ce_postgres psql -U postgres -d orchestrator \
     -c "SELECT task_id, target, task_type, status FROM tasks ORDER BY created_at DESC LIMIT 5;"
   ```

   **Expected:** Shows tasks persisted to database (migration 0008)

**Legal Terminal (Terminal 3):**

1. **Demonstrate Isolation:**
   ```
   Prompt: "This is a confidential attorney-client communication. Client: John Doe (SSN: 987-65-4321)"
   ```

   **Expected:** SSN masked (AI-only detection, ~15s latency)

**Talking Points:**
- "3 independent Goose instances with different profiles"
- "Agent Mesh enables cross-role communication"
- "Tasks routed via Controller API"
- "All tasks persisted to database (survive restarts) - migration 0008"
- "Privacy Guard intercepts all LLM calls"
- "4 Agent Mesh tools: send_task, notify, request_approval, fetch_status"

---

### Part 8: System Logs Demonstration (2 minutes)

**Show Controller Logs:**

```bash
# Terminal 4 (new terminal):
docker logs ce_controller --tail=50 | grep -E "task.created|Profile fetched|Vault"
```

**Point Out:**
- `task.created` entries (Agent Mesh routing)
- `Profile fetched` entries (Goose startup)
- `Vault AppRole authentication successful` (security)

**Show Privacy Guard Logs:**

```bash
docker logs ce_privacy_guard_finance --tail=30 | grep -i "mask"
```

**Point Out:**
- PII detection events
- Masking actions
- Fast latency (rules-only)

**Talking Points:**
- "All activity logged for audit"
- "Controller orchestrates everything"
- "Privacy Guard shows PII protection working"

---

## Demo Success Metrics

### ✅ What Must Work

**Critical:**
1. Admin Dashboard loads and displays correctly
2. CSV upload shows 50 users
3. Profile assignment works (3 users assigned)
4. Privacy Guard Control Panels accessible (all 3)
5. Vault Dashboard accessible
6. At least one Goose session starts successfully

**Important:**
7. Privacy masking demonstrated (email/SSN masked)
8. Agent Mesh task routing demonstrated (via MCP tool OR API fallback)
9. System logs show expected activity
10. All services healthy (docker compose ps)
11. Task persistence demonstrated (database query shows tasks)

**Nice to Have:**
12. Profile download/upload demonstrated
13. Config push demonstrated
14. All 3 Goose terminals responsive
15. Real-time log updates visible

---

## Backup Plans

### If Agent Mesh MCP Shows "Transport Closed"

**This is a KNOWN ISSUE with documented solutions:**

**Immediate Diagnostic Steps:**
1. Check Vault status: `docker exec ce_vault vault status | grep Sealed`
2. If sealed: Run `./scripts/unseal_vault.sh`
3. Check Controller has valid Vault token:
   ```bash
   docker logs ce_controller | grep -i vault
   ```
4. Restart Controller if Vault token invalid:
   ```bash
   docker compose -f ce.dev.yml --profile controller restart controller
   ```

**Documented Solutions (See Phase 6 Docs):**
- **D2_COMPLETION_SUMMARY.md**: Complete Agent Mesh fix history
- **MCP-EXTENSION-SUCCESS-SUMMARY.md**: MCP extension configuration details
- **PHASE6-D-BREAKTHROUGH.md**: Breakthrough solution documentation
- **VAULT-FIX-SUMMARY.md**: Vault Transit signing fix (critical for MCP)

**Fallback Demonstration Options:**

**Option 1: Use API Directly**
- Use curl commands to demonstrate Agent Mesh (see Part 7, Option B)
- Show Controller API docs: http://localhost:8088/docs
- Explain: "API proves Agent Mesh working, MCP tool loading issue"

**Option 2: Show Database Persistence**
```bash
# Query tasks table to show persistence working
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT task_id, target, task_type, description, status, created_at FROM tasks ORDER BY created_at DESC LIMIT 10;"
```
- Explain: "Tasks persist to database (migration 0008)"
- Show: "Agent Mesh backend fully functional"

**Option 3: Show Controller Logs**
```bash
docker logs ce_controller | grep "task.created"
```
- Point to: Task routing events logged
- Explain: "Controller orchestration working"

### If Goose Containers Fail to Start

**Option 1: Show API Directly**
- Use curl commands to demonstrate Agent Mesh
- Show Controller API docs: http://localhost:8088/docs
- Explain: "API works, container startup issue"

**Option 2: Use Goose Desktop**
- Run Goose Desktop on host
- Configure Agent Mesh extension
- Demonstrate tools working (proven in testing)

**Option 3: Show Database**
- Query tasks table: `SELECT * FROM tasks;`
- Show persistence working
- Explain architecture

### If Privacy Guard Times Out

**Fallback to Rules-Only:**
```bash
# Switch all to rules-only (fast mode)
curl -X PUT http://localhost:8096/api/detection -d '{"method": "rules"}'
curl -X PUT http://localhost:8097/api/detection -d '{"method": "rules"}'
curl -X PUT http://localhost:8098/api/detection -d '{"method": "rules"}'
```

### If JWT Tokens Expire During Demo

**Quick Fix:**
```bash
# Generate new token
./get_admin_token.sh

# Update in browser
# F12 → Console:
localStorage.setItem('admin_token', 'NEW_TOKEN_HERE');
```

---

## Post-Demo Talking Points

### Technical Achievements

**Architecture:**
- ✅ Microservices (17 containers working together)
- ✅ Database-driven configuration (PostgreSQL)
- ✅ Cryptographic security (Vault Transit signing)
- ✅ Per-instance isolation (CPU, storage, privacy)

**Privacy:**
- ✅ Real-time PII detection (3 methods: rules/hybrid/AI)
- ✅ Local processing (zero cloud dependencies)
- ✅ Audit logging (compliance-ready)
- ✅ Configurable per role (different privacy levels)

**Scalability:**
- ✅ Agent Mesh (cross-role communication)
- ✅ Task persistence (database-backed, migration 0008)
- ✅ Idempotency (safe retries)
- ✅ Health checks (automatic recovery)

### Business Value

**Community Edition (Free/Open Source):**
- Run everything locally
- Zero cloud costs
- Complete privacy control
- Perfect for individuals/small teams

**Business Edition (SaaS Subscription):**
- Privacy Guard stays local (trust)
- Controller + Admin Dashboard in cloud (convenience)
- Centralized management
- Enterprise features (SSO, LDAP, audit)

---

## Recovery Procedures (If Demo Crashes)

### Full Reset (Last Resort)

```bash
# 1. Stop everything
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
docker compose -f ce.dev.yml --profile controller --profile multi-goose down

# 2. Wait 10 seconds
sleep 10

# 3. Re-run full startup sequence (see Pre-Demo Checklist)
# This takes ~10 minutes but guarantees working state
```

### Quick Restart (Specific Service)

```bash
# If Controller fails:
docker compose -f ce.dev.yml --profile controller restart controller
sleep 20

# If Goose container fails:
docker compose -f ce.dev.yml --profile multi-goose restart goose-finance
sleep 15

# If Privacy Proxy fails:
docker compose -f ce.dev.yml --profile multi-goose restart privacy-guard-proxy-finance
sleep 10
```

---

## Demo Debrief (After Demo)

### Questions to Anticipate

**Q: What happens if I change a profile?**
A: Save in Admin Dashboard → Restart affected Goose container → New config loads

**Q: How long do JWT tokens last?**
A: 10 hours for dev, configurable for production

**Q: Can I run this on Windows/Mac?**
A: Yes, Docker Compose works on all platforms

**Q: What's the resource usage?**
A: ~8GB disk, ~4GB RAM, moderate CPU (3 Ollama instances)

**Q: Is data persistent?**
A: Yes, all data in volumes (users, profiles, tasks, secrets)

**Q: How do I backup?**
A: Docker volume backup or postgres dump

**Q: What about production deployment?**
A: Kubernetes configs planned (Phase 7), already has health checks

**Q: What if Agent Mesh MCP shows "Transport closed"?**
A: Known issue with documented solutions - see Phase 6 docs folder. API fallback always works.

### Next Steps

**For Users:**
1. Star the GitHub repo
2. Join Discord/Slack for community
3. Read docs: `/home/papadoc/Gooseprojects/goose-org-twin/DEMO_GUIDE.md`
4. Try locally: Clone repo, run `docker compose up`

**For Contributors:**
5. Check CONTRIBUTING.md
6. Review open issues
7. Submit PRs for enhancements

---

## Agent Mesh MCP Troubleshooting Reference

**If you see "Transport closed" error during demo, refer to:**

### Critical Documentation Files:
1. **D2_COMPLETION_SUMMARY.md** - Complete Agent Mesh implementation history
2. **MCP-EXTENSION-SUCCESS-SUMMARY.md** - MCP extension configuration
3. **PHASE6-D-BREAKTHROUGH.md** - Breakthrough solution documentation
4. **VAULT-FIX-SUMMARY.md** - Vault Transit signing fix

### Common Issues & Solutions:

**Issue 1: Vault Sealed**
```bash
# Symptom: Profile fetch fails, MCP tools don't load
# Solution:
./scripts/unseal_vault.sh
docker compose -f ce.dev.yml --profile controller restart controller
```

**Issue 2: Invalid Vault Token**
```bash
# Symptom: Controller logs show "403 Forbidden" from Vault
# Solution: Check VAULT-FIX-SUMMARY.md for token regeneration
docker logs ce_controller | grep -i vault
```

**Issue 3: MCP Extension Not Loaded**
```bash
# Symptom: agentmesh__ tools not visible in Goose
# Solution:
docker logs ce_goose_finance | grep "agent_mesh"
# If missing, rebuild Goose containers
```

**Issue 4: Task Persistence Not Working**
```bash
# Symptom: fetch_status returns 404
# Solution: Check migration 0008 applied
docker exec ce_postgres psql -U postgres -d orchestrator \
  -c "SELECT COUNT(*) FROM tasks;"
# Should return count, not error
```

---

**Demo Plan End**  
**Estimated Total Time:** 15-20 minutes  
**Success Probability:** High (95% complete, tested)  
**Fallback Options:** 3 backup plans documented  
**Agent Mesh Status:** Functional with documented troubleshooting
