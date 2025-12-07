# Phase 6 MVP Scope - Demo Ready

**Version:** 1.0  
**Date:** 2025-11-11  
**Status:** In Progress  
**Goal:** Working demo with visual proof of concept in 6 hours

---

## 🎯 Demo Objective

Create a **visually compelling demo** that proves:
1. ✅ Multi-agent orchestration works (Finance ↔ Manager ↔ Legal)
2. ✅ Privacy Guard is local on user's CPU (per-instance isolation)
3. ✅ Admin can manage org chart via CSV + profile assignment
4. ✅ Each user controls their own privacy settings
5. ✅ Agent Mesh MCP enables cross-agent task routing

---

## ✅ IN SCOPE (Must Complete for Demo)

### Workstream D: Agent Mesh E2E + Privacy Validation

#### D.1: /tasks/route Endpoint ✅ COMPLETE
- [x] Endpoint implemented and tested
- [x] Tasks routed to correct agents
- [x] Audit logging working

#### D.2: Agent Mesh MCP Integration ✅ COMPLETE
- [x] MCP server loading correctly
- [x] All 4 tools available (send_task, notify, request_approval, fetch_status)
- [x] 3/4 tools proven working
- [x] Vault signing fixed
- [x] Finance → Manager task routing proven

#### D.3: Task Persistence (NEW - Critical for Demo)
**Goal:** Make fetch_status work

**Tasks:**
- [ ] Create migration 0008: tasks table
- [ ] Update POST /tasks/route to store tasks
- [ ] Create GET /tasks?target={role}&status={status} endpoint
- [ ] Update fetch_status to query tasks table
- [ ] Test full lifecycle: send_task → store → fetch_status

**Schema:**
```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(50) NOT NULL,
    description TEXT,
    data JSONB,
    source VARCHAR(50) NOT NULL,
    target VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    context JSONB,
    trace_id UUID,
    idempotency_key UUID,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    CHECK (status IN ('pending', 'active', 'completed', 'failed', 'cancelled'))
);

CREATE INDEX idx_tasks_target_status ON tasks(target, status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX idx_tasks_trace_id ON tasks(trace_id);
```

**Acceptance Criteria:**
- [x] Migration 0008 runs successfully
- [x] Tasks persist to database (not just logged)
- [x] fetch_status returns tasks (not 404)
- [x] Finance can query "did manager receive my task?"
- [x] Manager can query "what tasks are pending for me?"

**Estimated Time:** 2 hours

---

#### D.4: Privacy Guard Architecture Validation (NEW)
**Goal:** Prove "local on user CPU" concept with visual logs

##### D.4.1: Remove Proxy/Service Redundancy (30 mins)
- [ ] Remove duplicate masking logic from Proxy
- [ ] Update Proxy to delegate ALL masking to Service
- [ ] Proxy becomes pure router (bypass vs service)
- [ ] Simplify proxy.rs (remove masking.rs)

**Changes:**
```rust
// OLD (redundant)
Proxy has masking logic → calls Service
Service has masking logic

// NEW (clean)
Proxy = Router only
  ├─ Bypass mode → Forward to LLM directly
  └─ Service mode → Forward to Privacy Guard Service
Service = All masking logic
```

##### D.4.2: Per-Instance Privacy Guard Setup (2 hours)
**Goal:** Each goose gets own Privacy Guard stack

**Docker Compose Changes:**
```yaml
services:
  # Finance Privacy Stack
  ollama-finance:
    ports: ["11434:11434"]
    volumes: [ollama_finance:/root/.ollama]
  
  privacy-guard-finance:
    ports: ["8089:8089"]
    environment:
      OLLAMA_URL: http://ollama-finance:11434
  
  privacy-guard-proxy-finance:
    ports: ["8090:8090"]
    environment:
      PRIVACY_GUARD_URL: http://privacy-guard-finance:8089
      DEFAULT_DETECTION_METHOD: rules
  
  # Manager Privacy Stack (ports 8091, 8093, 11435)
  # Legal Privacy Stack (ports 8092, 8094, 11436)
```

**goose Container Updates:**
- Finance → http://privacy-guard-proxy-finance:8090
- Manager → http://privacy-guard-proxy-manager:8090
- Legal → http://privacy-guard-proxy-legal:8090

**Acceptance Criteria:**
- [x] 3 independent Ollama instances (no blocking)
- [x] 3 independent Privacy Guard Services
- [x] 3 independent Privacy Guard Proxies
- [x] Each Control Panel accessible (8090, 8091, 8092)
- [x] Finance Rules-only (10ms) while Legal AI-only (15s) - no blocking

##### D.4.3: Visible Log Validation (1 hour)
**Goal:** Demo shows Privacy Guard routing in real-time

**Admin Dashboard Enhancement:**
- [ ] Add GET /admin/logs endpoint (real-time logs)
- [ ] Add /admin page: Live log viewer (websocket or polling)
- [ ] Show Privacy Guard flow: Proxy → Service → LLM

**Log Events to Show:**
```json
[Privacy Guard Proxy]
{"event":"request_received","user":"alice","mode":"service"}
{"event":"routing_to_service","url":"http://privacy-guard-finance:8089"}

[Privacy Guard Service]
{"event":"mask_request","method":"rules","text_length":156}
{"event":"pii_detected","types":["SSN","PERSON_NAME","SALARY"]}
{"event":"masking_complete","session_id":"abc123","duration_ms":8}

[Privacy Guard Proxy]
{"event":"forwarding_to_llm","provider":"openrouter","masked":true}
{"event":"response_received","status":200}

[Privacy Guard Service]
{"event":"unmask_request","session_id":"abc123"}
{"event":"unmasking_complete","duration_ms":2}

[Privacy Guard Proxy]
{"event":"response_returned","total_duration_ms":1245}
```

**Acceptance Criteria:**
- [x] Logs visible in Admin Dashboard
- [x] Shows Proxy → Service routing (not Proxy → LLM direct)
- [x] Shows PII detection and masking
- [x] Shows bypass mode (when enabled)

##### D.4.4: Bypass Mode Validation
**Goal:** Prove user control over privacy

**Demo Steps:**
1. Legal switches Control Panel to "Bypass" mode
2. Legal sends LLM request
3. Logs show: Proxy → LLM (Service skipped)
4. Logs show: "privacy_guard_bypassed" audit event

**Acceptance Criteria:**
- [x] Bypass mode works (no Service call)
- [x] Still logged for audit compliance
- [x] User can toggle Service ↔ Bypass

**Estimated Time:** 2 hours total for D.4

---

### Workstream A: Admin UI (NEW - Critical for Demo)

#### A.1: Minimal Admin Dashboard (2 hours)
**Goal:** Simple HTML UI for CSV import + profile assignment

**Pages:**
- [ ] GET /admin - Main dashboard
- [ ] CSV Upload section
- [ ] User table with profile assignment
- [ ] Live log viewer
- [ ] Config push button

**HTML Structure:**
```html
<!DOCTYPE html>
<html>
<head>
  <title>goose Orchestrator - Admin Dashboard</title>
  <style>
    /* Simple, clean CSS - purple/blue theme */
  </style>
</head>
<body>
  <h1>goose Orchestrator - Admin Dashboard</h1>
  
  <section id="csv-upload">
    <h2>1. Import Org Chart</h2>
    <input type="file" id="csvFile" accept=".csv">
    <button onclick="uploadCSV()">Upload CSV</button>
    <div id="uploadStatus"></div>
  </section>
  
  <section id="user-management">
    <h2>2. User Management</h2>
    <table id="usersTable">
      <thead>
        <tr>
          <th>Email</th>
          <th>Name</th>
          <th>Current Profile</th>
          <th>Assign Profile</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        <!-- Populated via JavaScript -->
      </tbody>
    </table>
  </section>
  
  <section id="config-push">
    <h2>3. Push Configurations</h2>
    <button onclick="pushConfigs()">Push to All goose Instances</button>
    <div id="pushStatus"></div>
  </section>
  
  <section id="live-logs">
    <h2>4. Live System Logs</h2>
    <div id="logViewer" style="height: 400px; overflow-y: scroll; 
                                 background: #1e1e1e; color: #00ff00; 
                                 font-family: monospace; padding: 10px;">
      <!-- Live logs appear here -->
    </div>
  </section>
  
  <script src="/admin/dashboard.js"></script>
</body>
</html>
```

**JavaScript Functions:**
```javascript
async function uploadCSV() {
  const file = document.getElementById('csvFile').files[0];
  const formData = new FormData();
  formData.append('file', file);
  
  const response = await fetch('/admin/org/import', {
    method: 'POST',
    body: formData
  });
  
  if (response.ok) {
    document.getElementById('uploadStatus').innerHTML = 
      `✅ ${await response.json().count} users imported`;
    loadUsers();
  }
}

async function assignProfile(userId, profile) {
  await fetch(`/admin/users/${userId}/assign-profile`, {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({profile})
  });
  
  loadUsers();
}

async function pushConfigs() {
  const response = await fetch('/admin/push-configs', {
    method: 'POST'
  });
  
  const result = await response.json();
  document.getElementById('pushStatus').innerHTML = 
    `✅ Pushed configs to ${result.count} instances`;
}

async function loadUsers() {
  const response = await fetch('/admin/users');
  const users = await response.json();
  
  const tbody = document.querySelector('#usersTable tbody');
  tbody.innerHTML = users.map(user => `
    <tr>
      <td>${user.email}</td>
      <td>${user.name}</td>
      <td>${user.profile || 'None'}</td>
      <td>
        <select onchange="assignProfile('${user.id}', this.value)">
          <option value="">-- Select --</option>
          <option value="finance">Finance</option>
          <option value="manager">Manager</option>
          <option value="legal">Legal</option>
          <option value="hr">HR</option>
          <option value="developer">Developer</option>
          <option value="analyst">Analyst</option>
          <option value="marketing">Marketing</option>
          <option value="support">Support</option>
        </select>
      </td>
      <td>${user.status || 'Pending'}</td>
    </tr>
  `).join('');
}

// Live log streaming (polling every 2s)
setInterval(async () => {
  const response = await fetch('/admin/logs?since=last');
  const logs = await response.json();
  
  const viewer = document.getElementById('logViewer');
  logs.forEach(log => {
    const div = document.createElement('div');
    div.textContent = `[${log.timestamp}] ${log.service}: ${log.message}`;
    viewer.appendChild(div);
  });
  
  viewer.scrollTop = viewer.scrollHeight; // Auto-scroll
}, 2000);

// Initialize
loadUsers();
```

**Acceptance Criteria:**
- [x] CSV upload works (50 users imported)
- [x] User table shows all imported users
- [x] Profile assignment dropdowns work
- [x] "Push Configs" triggers goose container config updates
- [x] Live logs show Privacy Guard routing
- [x] UI is simple but functional (no need for fancy design)

#### A.2: Admin API Routes (included in A.1 time)
**Routes to implement:**
```rust
// src/controller/src/routes/admin.rs

// Serve admin dashboard HTML
GET  /admin

// CSV import
POST /admin/org/import
  Request: multipart/form-data (CSV file)
  Response: {"count": 50, "users": [...]}

// List all users
GET  /admin/users
  Response: [{"id": "...", "email": "...", "name": "...", "profile": "..."}]

// Assign profile to user
POST /admin/users/{id}/assign-profile
  Request: {"profile": "finance"}
  Response: {"status": "assigned"}

// Push configs to goose instances
POST /admin/push-configs
  Response: {"count": 3, "instances": ["finance", "manager", "legal"]}

// Live logs (for log viewer)
GET  /admin/logs?since={timestamp}
  Response: [{"timestamp": "...", "service": "...", "message": "..."}]
```

**Estimated Time:** 2 hours total for A.1 + A.2

---

### Workstream V: Demo Validation (NEW - Replaces old full integration)

#### V.1: Demo Script Creation (30 mins)
**Goal:** Step-by-step demo script for screen recording

**File:** `docs/demo/DEMO-SCRIPT.md`

**Sections:**
1. Setup (Admin dashboard + 3 terminals + 3 control panels)
2. Phase 1: Admin CSV import + profile assignment (2 mins)
3. Phase 2: User auto-configuration (3 mins)
4. Phase 3: Privacy Guard validation with logs (5 mins)
5. Phase 4: Agent Mesh multi-agent communication (5 mins)
6. Phase 5: Per-instance CPU isolation proof (2 mins)

#### V.2: Screen Recording Setup (15 mins)
**Goal:** Configure OBS or SimpleScreenRecorder for 6-window layout

**Layout:**
```
┌─────────────────────────┬─────────────────────────┐
│  Terminal 1 (Finance)   │  Terminal 2 (Manager)   │
│  + Control Panel Tab    │  + Control Panel Tab    │
├─────────────────────────┼─────────────────────────┤
│  Terminal 3 (Legal)     │  Admin Dashboard        │
│  + Control Panel Tab    │  + Live Logs            │
└─────────────────────────┴─────────────────────────┘
```

#### V.3: Privacy Guard Log Validation
**Goal:** Visible proof that routing works correctly

**Test Cases:**
- [ ] Finance: Service mode → Logs show Proxy → Service → LLM
- [ ] Legal: Bypass mode → Logs show Proxy → LLM (Service skipped)
- [ ] Legal: AI-only → Logs show 15s NER call (doesn't block Finance)

#### V.4: Multi-Agent Communication Demo
**Test Cases:**
- [ ] Finance send_task → Manager fetch_status (sees task)
- [ ] Manager send_task → Legal fetch_status (sees task)
- [ ] Finance fetch_status → sees own sent task status
- [ ] All 4 tools working: send_task, notify, request_approval, fetch_status

#### V.5: Per-Instance CPU Isolation Demo
**Test Cases:**
- [ ] Legal uses AI-only (15s) while Finance uses Rules (10ms) - parallel
- [ ] Shows 3 independent Ollama instances
- [ ] Shows no blocking between instances

**Estimated Time:** 1 hour total for V.1-V.5

---

## ❌ OUT OF SCOPE (Deferred to Phase 7)

### Testing & Validation
- ❌ Automated privacy validation tests
- ❌ Automated integration test suites
- ❌ Performance benchmarking (automated)
- ❌ Security penetration testing
- ❌ Load testing (100+ concurrent users)

### Documentation
- ❌ Deployment topology diagrams (Community vs Business)
- ❌ Comprehensive architecture documentation
- ❌ Privacy Guard technical deep-dive
- ❌ Security hardening guide
- ❌ Operations runbooks

### Infrastructure
- ❌ JWT auto-refresh mechanism
- ❌ Kubernetes deployment configs
- ❌ Multi-tenant support
- ❌ Advanced audit features
- ❌ Backup/restore automation

### UI/UX
- ❌ Fancy admin dashboard design
- ❌ User portal (self-service profile view)
- ❌ D3.js org chart visualization
- ❌ Monaco YAML editor
- ❌ Real-time collaboration features

---

## 🎯 Success Criteria

### Must Show in Demo (Non-Negotiable)
1. ✅ **Admin Workflow:**
   - Upload CSV (50 users)
   - Assign 3 profiles (finance, manager, legal)
   - Push configs to all 3 goose instances
   - See status update in admin table

2. ✅ **User Auto-Configuration:**
   - 3 terminals log in as different users
   - Each auto-receives their assigned profile
   - Each sees their assigned extensions
   - Each has access to their Control Panel

3. ✅ **Privacy Guard Validation:**
   - Live logs show Proxy → Service → LLM routing
   - PII detection visible in logs (SSN, names, salaries)
   - Masking duration visible (<10ms for Rules)
   - Bypass mode visible (Proxy → LLM direct)

4. ✅ **Agent Mesh Communication:**
   - Finance sends task to Manager (visible in logs)
   - Manager fetches pending tasks (sees Finance's task)
   - Manager sends task to Legal (chain proven)
   - All 4 tools operational

5. ✅ **Per-Instance Isolation:**
   - Legal's 15s AI-only doesn't block Finance
   - Finance gets instant response (Rules-only)
   - 3 independent Ollama instances visible
   - "Local on user CPU" concept proven

### Nice to Have (Bonus)
- [ ] Prettier admin UI (if time allows)
- [ ] Export audit logs to CSV
- [ ] User table filtering/search
- [ ] Real-time config push status

---

## ⏱️ Timeline

### Total Estimated Time: 7 hours

**Hour 1: Documents** (this file)
- Create PHASE-6-MVP-SCOPE.md ✅
- Update Phase-6-Checklist.md
- Update Phase-6-Agent-State.json
- Update master-technical-project-plan.md

**Hour 2: Privacy Guard Refactor**
- Remove Proxy redundancy (30 mins)
- Per-instance docker-compose setup (1.5 hours)

**Hour 3-4: Task Persistence**
- Migration 0008 (30 mins)
- Update /tasks/route (30 mins)
- Create GET /tasks endpoint (30 mins)
- Test fetch_status (30 mins)

**Hour 5-6: Admin UI**
- HTML dashboard (1 hour)
- Admin API routes (1 hour)

**Hour 7: Demo Validation**
- Demo script creation (30 mins)
- Test all 5 demo phases (30 mins)

---

## 🚀 Go/No-Go Criteria

**Before proceeding to Phase 7, must have:**
- ✅ All 4 Agent Mesh tools working
- ✅ Privacy Guard routing visible in logs
- ✅ Per-instance isolation proven (no blocking)
- ✅ Admin can import CSV and assign profiles
- ✅ 3 goose instances auto-configure
- ✅ Demo script validated (all 5 phases working)
- ✅ Screen recording of full demo (15 minutes)

**Phase 7 can proceed when:**
- Demo proves concept to stakeholders
- Budget approved for Phase 7 UI development
- User feedback incorporated into Phase 7 scope

---

## 📝 Notes

### Why This Scope?
1. **Visual proof > automated tests:** Stakeholders need to SEE it working
2. **Demo-driven development:** Focus on what makes compelling demo
3. **Budget-conscious:** User is over budget, need fast MVP
4. **Proof of concept:** Validate architecture before investing in polish

### What Makes This Different?
- Original Phase 6 had 81+ automated tests → Too slow, not visual
- Original Phase 6 had 5 workstreams → Too broad, not focused
- Original Phase 6 deferred UI to Phase 7 → Can't demo without some UI

### Key Insights
- Admin UI is CRITICAL for demo (can't show CSV import without UI)
- Live logs are CRITICAL for proof (must see Privacy Guard routing)
- Per-instance setup is CRITICAL for vision (proves "local on user CPU")
- Agent Mesh is CRITICAL for value prop (multi-agent orchestration)

---

**Status:** Document created, ready for implementation  
**Next:** Update Phase-6-Checklist.md, Phase-6-Agent-State.json, master-technical-project-plan.md  
**Then:** Begin 6-hour implementation
