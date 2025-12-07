# Architecture Decisions Required Before D.3

**Date:** 2025-11-11  
**Status:** PENDING USER APPROVAL  
**Context:** Phase 6 D.2 complete, paused before D.3  
**Requirement:** User must review and approve all architecture changes before implementation

---

## Decision #1: Task Persistence Strategy

### Current State
- Tasks are **accepted** via POST /tasks/route
- Tasks are **logged** to audit trail (trace_id, task_id, target, task_type)
- Tasks are **NOT stored** in database
- GET /sessions/{task_id} returns 404
- fetch_status tool cannot retrieve task status

### Problem
- Manager agents cannot query "what tasks are waiting for me?"
- Finance agents cannot check "was my task approved?"
- No task queue or status tracking
- fetch_status tool non-functional

### User Requirement
"This is NOT a Phase 7 task - I want to fix the persistence issue before D.3"

### Options

#### Option A: Store Tasks in Existing `sessions` Table
**Approach:** Reuse sessions table, treat tasks as a type of session

**Schema mapping:**
- `id` → task_id (UUID)
- `role` → target agent role (e.g., "manager")
- `task_id` → source task reference (nullable, for approval requests)
- `status` → task status ("pending", "active", "completed")
- `metadata` → task payload (JSONB: {task_type, description, data})

**Pros:**
- ✅ No new migration needed (table exists)
- ✅ Reuse existing indexes
- ✅ Leverage FSM state machine for task lifecycle
- ✅ Session management logic already built

**Cons:**
- ❌ Conceptual confusion (tasks != sessions)
- ❌ Overloading session table with multiple purposes
- ❌ Future refactoring complexity

**Implementation Effort:** 1-2 hours (modify /tasks/route to insert into sessions)

---

#### Option B: Create New `tasks` Table
**Approach:** Dedicated table for task queue management

**Schema:**
```sql
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(50) NOT NULL,
    description TEXT,
    data JSONB,
    source VARCHAR(50) NOT NULL,      -- agent that sent task
    target VARCHAR(50) NOT NULL,      -- agent that should handle task
    status VARCHAR(20) NOT NULL,      -- pending/active/completed/failed
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

**Pros:**
- ✅ Clean separation of concerns
- ✅ Proper schema for task semantics
- ✅ Easy to query "tasks for manager"
- ✅ Future-proof for task queue features

**Cons:**
- ❌ Requires new migration (0008)
- ❌ Need to implement task lifecycle management
- ❌ More code to maintain

**Implementation Effort:** 3-4 hours (migration + route updates + tests)

---

#### Option C: Redis-Based Task Queue (Ephemeral)
**Approach:** Use Redis for fast, ephemeral task storage

**Structure:**
```
tasks:{target}:{task_id} → JSON task object
tasks:{target}:pending → List of pending task_ids
tasks:{target}:active → List of active task_ids
```

**Pros:**
- ✅ Very fast (in-memory)
- ✅ Auto-expiration (TTL support)
- ✅ Good for high-volume task routing

**Cons:**
- ❌ Not persistent across restarts
- ❌ Redis dependency (already have it, but not used)
- ❌ No historical task data

**Implementation Effort:** 2-3 hours (Redis client + key management)

---

### Recommendation
**Option B: Create New `tasks` Table**

**Reasoning:**
1. Clean architecture - tasks and sessions are different concepts
2. Proper schema for task semantics (source, target, task_type)
3. Easy to query and filter
4. Supports future features (task priorities, SLA tracking, auto-escalation)
5. Persistent across restarts (required for production)

**Trade-off:** Requires one new migration, but worth it for clarity

---

## Decision #2: Privacy Guard Architecture Refactor

### Current State
**Privacy Guard Service (port 8089):**
- Rust service with PII detection (regex + optional NER model)
- POST /mask endpoint
- POST /unmask endpoint
- GET /health endpoint

**Privacy Guard Proxy (port 8090):**
- Rust service that intercepts LLM requests
- Has **its own PII detection logic** in `src/privacy-guard-proxy/src/masking.rs`
- Calls Privacy Guard Service for some operations
- Has Control Panel UI for mode selection

### Problem
**Duplicate Logic:** Both Proxy and Service implement PII detection

**User's Original Intent:**
- Proxy = **Router + Settings Controller** (no masking logic)
- Service = **PII Detection Engine** (all masking logic)
- Proxy should **delegate** to Service for all masking

### Options

#### Option A: Refactor Proxy to Pure Router (RECOMMENDED)
**Changes:**
1. Remove `src/privacy-guard-proxy/src/masking.rs` (duplicate logic)
2. Update `src/privacy-guard-proxy/src/proxy.rs` to call Service for masking:
   ```rust
   // In proxy.rs
   async fn mask_content(&self, content: &str) -> Result<String> {
       // Call Privacy Guard Service
       let response = self.http_client
           .post(format!("{}/mask", self.guard_service_url))
           .json(&MaskRequest { text: content })
           .send()
           .await?;
       
       response.json::<MaskResponse>().await
   }
   ```
3. Update Control Panel UI to control **Service settings** (not just Proxy mode)
4. Add API endpoint: PUT /api/detection-method (Rules/Hybrid/AI-Only)
5. Forward detection method changes to Privacy Guard Service

**Pros:**
- ✅ Matches user's original design
- ✅ Single source of truth for PII detection
- ✅ Easier to maintain
- ✅ Control Panel controls actual detection (not just routing mode)

**Cons:**
- ❌ Requires refactor (2-3 hours)
- ❌ Extra HTTP hop (Proxy → Service adds ~1-2ms latency)

**Implementation Effort:** 2-3 hours

---

#### Option B: Keep Duplicate Logic (NOT RECOMMENDED)
**No changes - current implementation**

**Pros:**
- ✅ No work needed

**Cons:**
- ❌ Violates DRY principle
- ❌ Two codebases to maintain
- ❌ Inconsistent masking behavior risk
- ❌ Not what user intended

---

### Recommendation
**Option A: Refactor Proxy to Pure Router**

**Reasoning:**
1. Matches user's design intent
2. Single source of truth for PII detection
3. Control Panel can control actual detection method
4. Cleaner architecture
5. Minimal latency impact (~1-2ms per request)

---

## Decision #3: Deployment Topology Documentation

### Community Edition (Free, Open Source, Desktop-Only)

**Architecture:**
```
┌─────────────────────────────────────────┐
│         User's Computer (Local)         │
├─────────────────────────────────────────┤
│  goose Desktop                          │
│    ↓                                    │
│  Privacy Guard Proxy (8090)             │
│    ↓                                    │
│  Privacy Guard Service (8089)           │
│    ↓                                    │
│  LLM Provider (OpenRouter/Anthropic)    │
│                                         │
│  OPTIONAL:                              │
│  - Local Controller (orchestration)     │
│  - Local Vault (secrets)                │
│  - Local Postgres (profiles)            │
└─────────────────────────────────────────┘
```

**Components:**
- ✅ goose Desktop (user's machine)
- ✅ Privacy Guard Service (local PII detection)
- ✅ Privacy Guard Proxy (local request router)
- ⚠️ Controller (optional - for multi-agent workflows)
- ⚠️ Vault (optional - for profile signing)
- ⚠️ Postgres (optional - for profiles/tasks)

**Key Principle:** **100% local = 100% private**

**Use Cases:**
- Individual users
- Small teams
- Privacy-sensitive organizations
- Offline environments

---

### Business Edition (SaaS, Enterprise, Hybrid)

**Architecture:**
```
┌─────────────────────────────────────────┐
│         User's Computer (Local)         │
├─────────────────────────────────────────┤
│  goose Desktop                          │
│    ↓                                    │
│  Privacy Guard Proxy (8090) LOCAL       │
│    ↓                                    │
│  Privacy Guard Service (8089) LOCAL     │
│    ↓                                    │
└──────────────┬──────────────────────────┘
               │
               │ HTTPS (masked data only)
               ↓
┌─────────────────────────────────────────┐
│      Cloud Services (Shared/SaaS)       │
├─────────────────────────────────────────┤
│  Controller API (orchestration)         │
│  Vault (shared secrets/signing)         │
│  Postgres (profiles, tasks, audit)      │
│  Admin Dashboard (user management)      │
│  Audit Dashboard (compliance reporting) │
└─────────────────────────────────────────┘
```

**Components:**
- ✅ goose Desktop (user's machine)
- ✅ Privacy Guard Service (LOCAL - never cloud)
- ✅ Privacy Guard Proxy (LOCAL - never cloud)
- ☁️ Controller (CLOUD - shared orchestration)
- ☁️ Vault (CLOUD - shared secrets management)
- ☁️ Postgres (CLOUD - shared profiles/tasks/audit)
- ☁️ Admin Dashboard (CLOUD - user management)
- ☁️ Audit Dashboard (CLOUD - compliance reporting)

**Key Principle:** **Privacy Guard stays local, orchestration moves to cloud**

**Revenue Model:** Monthly subscription per user
- $X/month for hosted orchestration
- $Y/month for enterprise features (SSO, SAML, audit)
- $Z/month for dedicated deployment

**Use Cases:**
- Enterprise organizations
- Teams needing centralized orchestration
- Compliance-heavy industries
- Organizations wanting SaaS convenience with local privacy

---

### Decision Needed
**Which components belong in which edition?**

**Mandatory in Both:**
- goose Desktop
- Privacy Guard Service (local)
- Privacy Guard Proxy (local)

**Optional/Cloud:**
- Controller: Local in Community, Cloud in Business?
- Vault: Local in Community, Cloud in Business?
- Postgres: Local in Community, Cloud in Business?

**User to decide:**
- Can Community Edition users run without Controller at all? (Just Privacy Guard?)
- Does Community Edition include local Controller for multi-agent?
- Or is multi-agent orchestration Business Edition only?

---

## Decision #4: Privacy Guard Control Panel Enhancement

### Current State
**Control Panel UI shows:**
- Privacy Mode selector (Auto/Bypass/Strict)
- Activity log
- Status indicator

**User Expected:**
- Privacy Mode selector (Auto/Bypass/Strict)
- **Detection Method selector** (Rules/Hybrid/AI-Only)
- Activity log
- Status indicator
- **Control both Proxy AND Service settings**

### Options

#### Option A: Enhance Control Panel to Control Service
**Changes:**
1. Add Detection Method dropdown (Rules/Hybrid/AI-Only)
2. Add API endpoint: PUT /api/detection-method
3. Forward to Privacy Guard Service: PUT /detection-method
4. Update UI to show both Proxy mode and Service detection method
5. Add toggle: "Apply to both Proxy and Service"

**Pros:**
- ✅ User controls both components
- ✅ Matches user's expectations
- ✅ Single UI for all privacy settings

**Cons:**
- ❌ Requires Privacy Guard Service API changes (add /detection-method endpoint)
- ❌ Requires Proxy to call Service API (not just masking)

**Implementation Effort:** 1-2 hours

---

#### Option B: Keep Control Panel as Proxy-Only
**No changes - current implementation**

**Pros:**
- ✅ No work needed

**Cons:**
- ❌ Doesn't match user expectations
- ❌ Service settings hidden (only configurable via .env.ce)

---

### Recommendation
**Option A: Enhance Control Panel**

**Reasoning:**
1. Matches user's design intent
2. Better UX (one UI for all settings)
3. Enables runtime detection method changes
4. Supports architecture goal (Proxy controls Service)

---

## Summary of Decisions Required

| # | Decision | Options | User Input Needed | Estimated Effort |
|---|----------|---------|-------------------|------------------|
| 1 | Task Persistence | A: sessions table, B: new tasks table, C: Redis | Choose storage strategy | 1-4 hours |
| 2 | Privacy Guard Refactor | A: Pure router, B: Keep duplicate | Approve refactor plan | 2-3 hours |
| 3 | Deployment Topologies | Document Community vs Business | Approve component split | 1 hour (docs) |
| 4 | Control Panel Enhancement | A: Control Service too, B: Proxy-only | Approve UI changes | 1-2 hours |

**Total Estimated Effort:** 5-10 hours (depending on choices)

---

## Recommended Sequence

1. **Get User Decisions** (this session)
   - Task persistence strategy
   - Privacy Guard refactor approval
   - Deployment topology approval
   - Control Panel enhancement approval

2. **Create Architecture Document** (30 mins)
   - Deployment topology diagrams
   - Component ownership (local vs cloud)
   - Data flow diagrams

3. **Implement Task Persistence** (1-4 hours)
   - Based on user choice
   - Create migration if needed
   - Update /tasks/route to store tasks
   - Update GET /sessions or create GET /tasks endpoint
   - Test fetch_status tool

4. **Refactor Privacy Guard** (2-3 hours)
   - Remove duplicate logic from Proxy
   - Make Proxy call Service
   - Enhance Control Panel UI
   - Test end-to-end

5. **Integration Verification** (1 hour)
   - Verify Vault working
   - Verify tokens working
   - Verify Controller working
   - Verify Proxy working
   - Verify Guard working
   - Verify profiles working
   - Verify databases working

6. **Proceed to D.3** (2-3 days)
   - Privacy validation testing
   - With all components integrated

---

## Questions for User

### Task Persistence
**Q1:** How should we store tasks?
- [ ] A. Reuse sessions table (fast, but conceptual mismatch)
- [ ] B. Create new tasks table (clean, but requires migration)
- [ ] C. Use Redis (fast, but not persistent across restarts)

**Q2:** Do tasks need historical tracking?
- [ ] Yes - keep completed tasks for audit
- [ ] No - delete after completion

### Privacy Guard Architecture
**Q3:** Should we refactor Privacy Guard Proxy to remove duplicate logic?
- [ ] Yes - make it a pure router (call Service for all masking)
- [ ] No - keep current dual implementation
- [ ] Defer - fix in later phase

**Q4:** Should Control Panel UI control both Proxy mode AND Service detection method?
- [ ] Yes - single UI for all privacy settings
- [ ] No - keep separate (UI for Proxy, .env.ce for Service)

### Deployment Models
**Q5:** Community Edition components?
- [ ] goose Desktop + Privacy Guard only (no Controller)
- [ ] goose Desktop + Privacy Guard + local Controller
- [ ] Other (specify)

**Q6:** Business Edition SaaS model?
- [ ] Local: Privacy Guard Service + Proxy
- [ ] Cloud: Controller + Vault + Postgres + Admin UI
- [ ] Pricing: Monthly per-user subscription?

### Integration Verification
**Q7:** Should we re-enable Privacy Guard in Controller for D.3?
- [ ] Yes - enable with rules-only mode (fast)
- [ ] No - keep disabled until refactor complete
- [ ] Yes - enable with current mode (hybrid)

**Q8:** Should we fix JWT token expiration (5 min → 30 days)?
- [ ] Yes - request longer-lived tokens
- [ ] Yes - implement auto-refresh
- [ ] No - accept 5-min expiration for now

---

## User's Current Position (from discussion)

**Privacy Guard Design:**
> "I did not know that privacy guard proxy was duplicating services. I thought the proxy just routes the messages to and from Privacy Guard service (with the different modes)"

**Deployment Vision:**
> "I want this to be SaaS - local Privacy Guard but cloud Controller for orchestration. Monthly subscription model."

**Integration Requirement:**
> "Before we move to D.3 we need to have ALL previous work integrated (Vault, tokens, Controller, Proxy, Guard, profiles, databases)"

**Task Persistence:**
> "I want to fix the persistence issue, this is not a Phase 7 task"

**Architecture Changes:**
> "I want to do the proper architecture fix (not quick hack). Ask me if in doubt... present options and let me decide"

---

## Next Agent Actions

1. **READ THIS DOCUMENT** to understand all pending decisions
2. **PRESENT OPTIONS** to user in clear, concise format
3. **GET APPROVAL** for each decision before implementing
4. **CREATE DEPLOYMENT TOPOLOGY DIAGRAMS** after user approves models
5. **IMPLEMENT APPROVED CHANGES** in order (task persistence → Privacy Guard refactor → integration verification)
6. **VERIFY ALL COMPONENTS INTEGRATED** before proceeding to D.3
7. **ONLY THEN** proceed to D.3 Privacy Validation

---

**Status:** WAITING FOR USER INPUT  
**Updated:** 2025-11-11 10:25 EST  
**Next Session:** Present these options to user and get decisions
