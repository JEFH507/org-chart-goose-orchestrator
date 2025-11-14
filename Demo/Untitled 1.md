# Answers (No Code Execution)

---

## #1: Command to Renew AppRole Token Before 1-Hour Expiration

**Short answer: You can't manually "renew" an AppRole token - you must restart the Controller.**

**What happens:**

- AppRole tokens are generated at Controller startup
- They live for 1 hour
- There's NO manual command to renew them while Controller is running
- The Controller code doesn't have renewal logic

**Your options:**

**Option A: Restart Controller (generates new token)**

`docker compose -f ce.dev.yml --profile controller restart controller # New AppRole token auto-generated, valid for 1 hour`

**Option B: Add VAULT_TOKEN to .env.ce (lasts 32 days)**

- Then you don't worry about 1-hour expiration

**Option C: Implement auto-renewal in Controller code** (future work)

---

## #2: Direct Token Persistence Without Volume Deletion

**YES, the direct token persists!**

**Scenario: `docker compose down` (WITHOUT `-v`)**

`docker compose -f ce.dev.yml --profile controller --profile privacy-guard \   --profile privacy-guard-proxy --profile ollama --profile multi-goose \   --profile redis down`

**What happens to VAULT_TOKEN:**

- ✅ Vault volume NOT deleted
- ✅ Token still valid in Vault database
- ✅ `.env.ce` still has VAULT_TOKEN value
- ✅ When you start Controller again: Token still works!

**Token lasts: 32 days (768 hours) from creation**

**After 32 days:** Token expires, you need to generate new one

**When you DON'T need to regenerate token:**

- Simple restart (no `-v`)
- Stop/start containers
- Restart Controller
- Restart Goose instances

**When you DO need to regenerate token:**

- Used `docker compose down -v` (volumes deleted)
- Token expired (after 32 days)
- Vault unsealed with different keys

---

## #3: Privacy Guard Masking Logs Issue - Root Cause

**YES, you're exactly right!**

**The cascade failure:**

1. **Profile signature verification failed (403)**  
    ↓
2. **Goose couldn't fetch profile from Controller**  
    ↓
3. **Goose generated default config:**
    
    `role: unknown extensions: {} api_base: <might be wrong>`
    
    ↓
4. **Privacy Guard Proxy NOT configured in extensions**  
    ↓
5. **Goose sends requests directly to OpenRouter**  
    ↓
6. **Privacy Guard never sees traffic**  
    ↓
7. **No masking logs appear**

**What's failing:**

- Vault token expired → Transit HMAC verification failed
- Without valid signature, profile rejected
- Without profile, Goose doesn't know:
    - "I am finance role"
    - "My LLM requests go through privacy-guard-proxy-finance:8090"
    - "I have Agent Mesh MCP extension"

**How to prove Privacy Guard works:**

**After you fix Vault token issue:**

1. Restart Controller (with VAULT_TOKEN in .env.ce)
2. Restart Goose Finance
3. Verify profile fetch succeeded:
    
    `docker logs ce_goose_finance 2>&1 | grep "Profile fetched" # Should show: ✓ Profile fetched successfully for role 'finance'`
    
4. In Goose Finance terminal, ask anything with PII:
    - "Analyze this email: [alice@company.com](mailto:alice@company.com), SSN 123-45-6789"
5. Check Privacy Guard logs:
    
    `docker logs ce_privacy_guard_finance 2>&1 | grep "audit" # Should show: Entity counts, redaction events`
    

**Why it wasn't working:**

- Profile fetch failed → Config wrong → Requests bypassed Privacy Guard

---

## #4: Admin Dashboard "Push Configs" Button Status

**Current Status: NOT FULLY INTEGRATED**

**What the button does (currently):**

`// When you click "Push Configs to All Instances" async function pushConfigs() {     const response = await fetch('/admin/push-configs', { method: 'POST' }); }`

**What the backend endpoint does:**

- Endpoint exists: `POST /admin/push-configs`
- Returns: `{ pushed_count: 0 }` (placeholder)
- **Does NOT actually push to Goose containers**

**What's missing:**

1. **WebSocket/SSE connection to Goose containers** (not implemented)
2. **Config reload mechanism in Goose** (not implemented)
3. **Profile assignment to Goose mapping** (partial)

**Current workflow (manual):**

1. Upload CSV → Users created in database
2. Assign profiles → Updates `org_users.profile` column
3. **Goose fetches profile on startup only** (not dynamically)
4. **To apply changes: Restart Goose containers**

**How profile assignment works NOW:**

- User "Alice" (EMP001) assigned "finance" profile
- Stored in database: `org_users` table
- When Goose Finance starts:
    - Fetches profile for "finance" role (NOT per-user)
    - All Finance Goose instances share same profile
- Profile is role-based, not user-based

**Not integrated yet:**

- Real-time config push
- Per-user profile customization
- Dynamic reload without restart

---

## #5: How to Improve Agent Mesh MCP Limitations

**Current Limitations You Found:**

❌ `list_tasks` - Can't see all tasks for my role  
❌ `get_current_role` - Don't know my own role  
❌ `fetch_status` returns "unknown" fields

---

### **Improvement 1: Add `list_tasks` Tool**

**What to add:**

**New MCP Tool:** `agentmesh__list_tasks`

**Parameters:**

- `status` (optional) - Filter by: pending, active, completed
- `limit` (optional) - Max results (default 50)

**Backend API needed:**

```
GET /tasks?target={role}&status={status}&limit={limit}
```

**Returns:**

`[   {     "id": "task-123",     "task_type": "budget_approval",     "status": "pending",     "created_at": "2025-11-13T07:02:14Z"   } ]`

**Files to modify:**

1. `src/agent-mesh/tools/list_tasks.py` (create new)
2. `src/agent-mesh/agent_mesh_server.py` (register tool)
3. `src/controller/src/routes/tasks.rs` (add list endpoint)

---

### **Improvement 2: Add `get_current_role` Tool**

**What to add:**

**New MCP Tool:** `agentmesh__get_current_role`

**No parameters needed**

**How it works:**

- Reads from Goose config.yaml
- Returns current role (finance, manager, legal)

**Implementation:**

`# src/agent-mesh/tools/get_current_role.py import yaml def get_current_role():     with open('/root/.config/goose/config.yaml') as f:         config = yaml.safe_load(f)     return config.get('role', 'unknown')`

**Returns:**

`{   "role": "finance",   "display_name": "Finance Team" }`

---

### **Improvement 3: Enhance `fetch_status` Response**

**Problem:** Backend returns limited data, shows "unknown" for fields

**Current API response:**

`{   "id": "task-123",   "task_type": "budget_approval",   "status": "pending"   // Missing: data, context, timestamps }`

**Fix:** Update Controller to return full task details

**File to modify:** `src/controller/src/routes/tasks.rs`

**Change:**

`// Current (simplified response) json!({     "id": task.id,     "task_type": task.task_type,     "status": task.status }) // Enhanced (full response) json!({     "id": task.id,     "trace_id": task.trace_id,     "task_type": task.task_type,     "target": task.target,     "status": task.status,     "data": task.data,          // ← Add this     "context": task.context,    // ← Add this     "created_at": task.created_at,     "updated_at": task.updated_at,     "completed_at": task.completed_at })`

---

### **Improvement 4: Add `update_task_status` Tool**

**Use case:** Manager approves/rejects budget request

**New MCP Tool:** `agentmesh__update_task_status`

**Parameters:**

- `task_id` (required)
- `status` (required) - "approved", "rejected", "cancelled"
- `notes` (optional) - Reason for decision

**Backend API needed:**

```
PATCH /tasks/{id}/status
Body: { "status": "approved", "notes": "Budget approved for Q1" }
```

---

### **Summary of Improvements:**

|Tool|Priority|Complexity|Impact|
|---|---|---|---|
|`list_tasks`|HIGH|Medium|Enables workflow visibility|
|`get_current_role`|MEDIUM|Low|Self-awareness for agents|
|Enhanced `fetch_status`|HIGH|Low|Better task details|
|`update_task_status`|HIGH|Medium|Close the approval loop|

**Quick wins (do first):**

1. ✅ Enhanced `fetch_status` (backend only, 10 lines)
2. ✅ `get_current_role` (read config file, 5 lines)

**Bigger features (Phase 7?):**

1. 🔄 `list_tasks` (new endpoint + MCP tool)
2. 🔄 `update_task_status` (state machine logic)