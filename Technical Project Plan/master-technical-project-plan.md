# Org-Chart Orchestrated AI Framework — Technical Project Plan
Version: v2.0 (Grant-Aligned)
Date: 2025-11-05
Previous: v1.0 (2025-10-27)
Alignment: Goose v1.12.00 architecture, Block Goose Innovation Grant ($100K/12mo)

## Objectives and Success Criteria
- Deliver a privacy-first, org-aware orchestration MVP that coordinates role-based “digital twin” agents using HTTP-only flows and OIDC SSO.
- Prove cross-agent task routing and approvals, deterministic pseudonymization, and auditable runs.
- Reuse Goose v1.12 capabilities (MCP-first, lead/worker models, OTLP-ready observability).

Success (MVP):
- E2E demo of multi-agent approval workflow with policy enforcement and audit trail.
- Privacy guard: mask-and-forward enforced, measurable accuracy/latency (≤500ms P50).
- Controller APIs published (minimal OpenAPI), Agent Mesh MCP verbs working.
- Deployable with CE defaults (Keycloak, Vault OSS, Postgres, Ollama; optional S3-compatible object storage — SeaweedFS default option; MinIO/Garage optional).
- Availability doc’d at 99.5% SLO; P50 ≤ 5s interactive agent tasks.

## Scope (MVP vs Post-MVP)
MVP (v1.2):
- Components: identity-auth-gateway, agent-mesh-mcp, directory-policy, controller-api, privacy-guard, audit-observability, model-orchestration, storage-metadata, packaging-deployment, security-secrets.
- Constraints: HTTP-only (no message bus), single-tenant per org, minimal metadata server-side, desktop-first agents.

Post-MVP (v1.3+):
- Policy composition/graph engine, message bus adapter, SCIM, mTLS, dashboards, multi-tenant hardening, advanced approvals, analytics.

## Architecture Overview and Alignment to Goose v1.12
Alignment:
- MCP-first tools and extension allowlists match governance needs.
- Lead/worker provider orchestration supports guard→planner/worker flows.
- Axum-based server (goosed) with OpenAPI and OTLP exporter re-used.
- Local-first state and OS keychain align with privacy-by-design.

Additions:
- Directory/Policy (signed role profiles, ABAC eval), HTTP Controller, Agent Mesh MCP verbs, Session Broker notions (scoped context) as minimal HTTP services.
- Identity via OIDC/JWT with bridge to goosed (X-Secret-Key compatibility).

Exclusions:
- No message bus in MVP; async handled via HTTP and polling.

## Work Breakdown Structure (WBS) and Timeline (Grant-Aligned: 12 months)

### Phases 0-3: COMPLETE ✅ (2 weeks actual vs 4 weeks estimated)
**Status:** Merged to main, tagged v0.3.0

Phase 0: Project Setup (S) ✅ COMPLETE
- Repo scaffolding, docker-compose CE defaults (Keycloak, Vault, Postgres, Ollama).
- Actual: 1 day

Phase 1: Identity & Security (M) ✅ COMPLETE
- OIDC login, JWT minting, Vault OSS wiring.
- Actual: 2 days

Phase 1.2: Controller JWT Verification (S) ✅ COMPLETE
- RS256 signature validation, JWKS caching, clock skew tolerance.
- Controller-side middleware integration.
- Actual: 1 day

Phase 2: Privacy Guard (M) ✅ COMPLETE
- Local runtime (regex + NER), deterministic pseudonymization (Vault keys), mask-and-forward.
- Actual: 3 days

Phase 2.2: Privacy Guard Enhancement (S) ✅ COMPLETE
- Local model integration (Ollama), preserve modes (Off/Detect/Mask/Strict).
- Actual: 2 days

Phase 2.5: Dependency Upgrades (S) ✅ COMPLETE
- Quarterly audit (Rust 1.83.0, Keycloak 26.0.4, Python 3.13.9, Vault 1.18.3).
- CVE remediation, compatibility validation.
- Actual: 1 day

Phase 3: Controller API + Agent Mesh (L) ✅ COMPLETE
- Controller API: 5 routes (tasks, sessions, approvals, profiles, audit), 21 unit tests, OpenAPI spec, JWT auth.
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status), 977 lines code, 5/6 integration tests.
- Cross-agent demo: Finance → Manager workflow, 5/5 test cases pass, 6/6 smoke tests.
- Actual: 2 days (estimated 9 days) — 78% faster

**Phases 0-3 Total:** 2 weeks (estimated 4 weeks)

---

### Phase 4: Storage/Metadata + Session Persistence (M) — GRANT Q1 MILESTONE
**Timeline:** Week 3-4 (1 week)  
**Target:** Grant application ready (v0.4.0)

Workstreams:
- A. Postgres Schema Design (~2 days)
  - Sessions table (id, role, task_id, status, created_at, updated_at, metadata)
  - Tasks table (id, type, from_role, to_role, payload, trace_id, idempotency_key)
  - Approvals table (id, task_id, approver_role, status, decision_at, notes)
  - Audit index (id, event_type, role, timestamp, trace_id, metadata)
  - Migrations (Diesel ORM for Rust, or SQL scripts)

- B. Session CRUD Operations (~2 days)
  - Controller routes: POST /sessions (create), GET /sessions/{id} (fetch), PUT /sessions/{id} (update)
  - Replace 501 responses with real data from Postgres
  - Session lifecycle: pending → active → completed/failed
  - Retention policies (7 days default, configurable)

- C. fetch_status Tool Completion (~1 day)
  - Update Agent Mesh fetch_status to call GET /sessions/{task_id}
  - Return real session status (not 501)
  - Integration tests updated (6/6 passing)

- D. Idempotency Deduplication (~1 day)
  - Redis cache for idempotency keys (TTL: 24 hours)
  - Controller middleware: check Redis before processing, return 200 if duplicate
  - Test duplicate request handling

- E. Progress Tracking (~15 min) 🚨 CHECKPOINT
  - Update Phase-4-Agent-State.json
  - Update phase4-progress.md
  - Commit to git, report to user

**Deliverables:**
- ✅ Postgres schema deployed (4 tables + migrations)
- ✅ Session persistence operational (POST/GET/PUT /sessions)
- ✅ fetch_status tool functional (no more 501 errors)
- ✅ Idempotency deduplication working (Redis-backed)
- ✅ Integration tests 6/6 passing
- ✅ Tagged release: v0.4.0

**Effort:** M (~1 week, 3-4 days)

---

### Phase 5: Profile System + Privacy Guard MCP + Admin UI (L) — GRANT Q1 MILESTONE
**Timeline:** Week 5-7 (1.5-2 weeks)  
**Target:** Grant application ready (v0.5.0)  
**Builds On:** Phases 1-4 (OIDC/JWT, Privacy Guard regex+NER, Controller API, Session Persistence)

---

#### **🎯 Phase 5 Goals:**
1. **Zero-Touch Profile Deployment:** User signs in → Profile auto-loaded → All configs applied
2. **Privacy Guard MCP:** Local PII protection (no upstream Goose dependency)
3. **Enterprise Governance:** Multi-provider controls, recipe automation, memory privacy
4. **Admin UI:** Org chart visualization, profile management, audit trail
5. **Full Integration Testing:** Prove Phase 1-4 stack still works + new features functional
6. **Backward Compatibility:** No breaking changes to existing Controller API or Agent Mesh

---

### Workstreams:

#### **A. Profile Bundle Format (1.5 days)**  
**Builds On:** Phase 3 Controller API (`GET /profiles/{role}` returns mock data)

**Extended Schema (YAML/JSON):**
```yaml
# Example: profiles/analyst.yaml
role: "analyst"
display_name: "Business Analyst"
description: "Data analysis, process optimization, time studies"

# NEW: LLM Provider Configuration
providers:
  primary:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
    temperature: 0.3
  planner:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
  worker:
    provider: "openrouter"
    model: "openai/gpt-4o-mini"
  allowed_providers: ["openrouter", "ollama"]  # Governance: only these providers allowed
  forbidden_providers: []  # Optional: explicitly deny providers

# ENHANCED: MCP Extensions (from Block registry)
extensions:
  - name: "github"
    enabled: true
    tools: ["list_issues", "create_issue", "add_comment"]
  - name: "agent_mesh"
    enabled: true
    tools: ["send_task", "request_approval", "notify", "fetch_status"]
  - name: "memory"  # NEW: Memory preferences
    enabled: true
    preferences:
      retention_days: 90
      auto_summarize: true
      include_pii: false  # GDPR/SOC2 compliance
  - name: "excel-mcp"  # NEW: Business analyst tools
    enabled: true
  - name: "sql-mcp"
    enabled: true

# NEW: Global Goosehints (Org-Wide Context)
goosehints:
  global: |
    # Analyst Role Context
    You are a business analyst for the organization.
    Focus on data-driven insights, process optimization, and time studies.
    
    When analyzing data:
    - Validate data sources before drawing conclusions
    - Document assumptions clearly
    - Provide statistical confidence intervals
    
    @README.md  # Auto-include project context
    @docs/data-dictionary.md

# NEW: Global Gooseignore (Privacy Protection)
gooseignore:
  global: |
    # Sensitive file patterns (org-wide)
    **/.env
    **/.env.*
    **/secrets.*
    **/credentials.*
    **/config/production.*
    
  # Templates for local .gooseignore (project-specific)
  local_templates:
    - path: "finance/budgets"
      content: |
        # Finance-specific exclusions
        **/employee_salaries.*
        **/bonus_data.*

# NEW: Recipes (Automated Workflows)
recipes:
  - name: "daily-kpi-report"
    description: "Generate daily KPI report at 9am"
    path: "recipes/analyst/daily-kpi-report.yaml"
    schedule: "0 9 * * 1-5"  # Mon-Fri 9am
    enabled: true
  
  - name: "process-bottleneck-analysis"
    description: "Weekly process efficiency analysis"
    path: "recipes/analyst/bottleneck-analysis.yaml"
    schedule: "0 10 * * 1"  # Monday 10am
    enabled: true
  
  - name: "time-study-analysis"
    description: "Monthly time study automation"
    path: "recipes/analyst/time-study.yaml"
    schedule: "0 9 1 * *"  # 1st of month
    enabled: true

# NEW: Automated Tasks (Scheduled Execution)
automated_tasks:
  - name: "daily-data-quality-check"
    recipe: "recipes/analyst/data-quality.yaml"
    schedule: "0 8 * * 1-5"
    enabled: true
    notify_on_failure: true

# RBAC/ABAC Policies
policies:
  - allow_tool: "excel-mcp__*"  # All Excel tools
  - allow_tool: "sql-mcp__query"
    conditions:
      - database: "analytics_*"  # Only analytics databases
  - deny_tool: "developer__shell"
    reason: "No arbitrary code execution for Analyst role"

# NEW: Privacy Guard Configuration (Per-Role Defaults)
privacy:
  mode: "hybrid"  # rules, ner, hybrid
  strictness: "moderate"  # strict, moderate, permissive
  allow_override: true  # Can user change settings?
  rules:
    - pattern: '\b\d{3}-\d{2}-\d{4}\b'  # SSN
      replacement: '[SSN]'
    - pattern: '\b[A-Z]{2}\d{6,8}\b'  # Employee ID
      replacement: '[EMP_ID]'
  pii_categories: ["SSN", "EMAIL", "PHONE", "EMPLOYEE_ID"]

# Environment Variables
env_vars:
  SESSION_RETENTION_DAYS: "90"
  PRIVACY_GUARD_MODE: "hybrid"
  DEFAULT_MODEL: "openrouter/anthropic/claude-3.5-sonnet"

# Signing (Vault-backed HMAC for tamper protection)
signature:
  algorithm: "HS256"
  vault_key: "transit/keys/profile-signing"
  signed_at: "2025-11-05T14:00:00Z"
  signed_by: "admin@company.com"
```

**Tasks:**
1. Define JSON Schema for profile validation (Rust `serde` types)
2. Cross-field validation:
   - `allowed_providers` must include `primary.provider`
   - Recipe paths must exist in `recipes/` directory
   - Extension names must match Block registry catalog
3. Vault signing integration (reuse Phase 1 Vault client)
4. Postgres storage:
   ```sql
   CREATE TABLE profiles (
     role VARCHAR(50) PRIMARY KEY,
     display_name VARCHAR(100),
     data JSONB NOT NULL,  -- Full profile
     signature TEXT,       -- Vault HMAC
     created_at TIMESTAMP DEFAULT NOW(),
     updated_at TIMESTAMP DEFAULT NOW()
   );
   ```
5. Migration script (`sqlx migrate add create_profiles`)
6. Unit tests (15+ test cases: valid profile, invalid provider, missing fields, etc.)

**Backward Compatibility:**
- Phase 3 Controller API `GET /profiles/{role}` already exists
- This workstream replaces mock data with real Postgres-backed profiles
- **No API changes** (maintains existing contract)

---

#### **B. Role Profiles (6 roles × 3 recipes = 18 files) (2 days)**  
**Builds On:** Workstream A schema

**6 Roles:**
1. **Finance** (Budget approvals, compliance, reporting)
2. **Manager** (Team oversight, approvals, delegation)
3. **Analyst** (Business analysis, data insights, process optimization) — *REVISED from "Engineering"*
4. **Marketing** (Campaign management, analytics, content)
5. **Support** (Ticket triage, KB management, escalation)
6. **Legal** (Contract review, compliance, risk assessment) — *NEW*

**Profile Details:**

**1. Finance Profile** (`profiles/finance.yaml`)
- **Providers:** OpenRouter Claude 3.5 Sonnet (primary), GPT-4o-mini (worker)
- **Extensions:** `github` (read-only), `agent_mesh`, `memory` (no PII), `excel-mcp`
- **Goosehints:**
  ```yaml
  global: |
    You are the Finance team agent. Focus on budget compliance, cost tracking, and regulatory reporting.
    Always verify budget availability before approving spend requests.
    @finance/policies/approval-matrix.md
  local_templates:
    - path: "finance/budgets"
      content: |
        # Budget-specific context
        - Current fiscal year: FY2026
        - Budget cycle: Monthly close on 5th business day
  ```
- **Gooseignore:** `**/salary_data.*`, `**/bonus_plans.*`, `**/tax_records.*`
- **Recipes:** 
  - `monthly-budget-close` (schedule: `0 9 5 * *`)
  - `weekly-spend-report` (schedule: `0 10 * * 1`)
  - `quarterly-forecast` (schedule: `0 9 1 1,4,7,10 *`)
- **Privacy:** `strict` mode, `allow_override: false` (compliance requirement)

**2. Manager Profile** (`profiles/manager.yaml`)
- **Providers:** OpenRouter Claude 3.5 Sonnet (planning), GPT-4o (lead)
- **Extensions:** `agent_mesh`, `memory` (full context), `github`
- **Goosehints:**
  ```yaml
  global: |
    You are a team manager. Focus on delegation, approval workflows, and team coordination.
    Always document approval decisions with rationale.
  ```
- **Recipes:**
  - `daily-standup-summary` (schedule: `0 9 * * 1-5`)
  - `weekly-team-metrics` (schedule: `0 10 * * 1`)
  - `monthly-1on1-prep` (schedule: `0 9 1 * *`)
- **Privacy:** `moderate` mode, `allow_override: true`

**3. Analyst Profile** (`profiles/analyst.yaml`) — *REVISED*
- **Providers:** OpenRouter GPT-4 (data analysis), Claude 3.5 (insights)
- **Extensions:** `developer`, `excel-mcp`, `sql-mcp`, `agent_mesh`, `memory`
- **Goosehints:**
  ```yaml
  global: |
    You are a business analyst. Focus on data-driven insights, process optimization, and time studies.
    Validate data sources, document assumptions, provide confidence intervals.
    @docs/data-dictionary.md
  ```
- **Recipes:**
  - `daily-kpi-report` (schedule: `0 9 * * 1-5`)
  - `process-bottleneck-analysis` (schedule: `0 10 * * 1`)
  - `time-study-analysis` (schedule: `0 9 1 * *`)
- **Privacy:** `moderate` mode (data analysis may need full context)

**4. Marketing Profile** (`profiles/marketing.yaml`)
- **Providers:** OpenRouter GPT-4 (creative), Claude 3.5 (analytical)
- **Extensions:** `web-scraper`, `agent_mesh`, `memory`, `github`
- **Recipes:**
  - `weekly-campaign-report` (schedule: `0 10 * * 1`)
  - `monthly-content-calendar` (schedule: `0 9 1 * *`)
  - `competitor-analysis` (schedule: `0 9 1 * *`)
- **Privacy:** `permissive` mode (public data focus)

**5. Support Profile** (`profiles/support.yaml`)
- **Providers:** OpenRouter Claude 3.5 (empathy-optimized)
- **Extensions:** `github` (issue triage), `agent_mesh`, `memory`
- **Recipes:**
  - `daily-ticket-summary` (schedule: `0 9 * * 1-5`)
  - `weekly-kb-updates` (schedule: `0 10 * * 5`)
  - `monthly-satisfaction-report` (schedule: `0 9 1 * *`)
- **Privacy:** `strict` mode (customer data protection)

**6. Legal Profile** (`profiles/legal.yaml`) — *NEW*
- **Providers:** **Local-only** (Ollama llama3.2), forbidden: `["openrouter", "openai", "anthropic"]`
- **Extensions:** `agent_mesh`, `memory` (attorney-client privilege, zero PII retention)
- **Goosehints:**
  ```yaml
  global: |
    You are the Legal team agent. Focus on contract review, regulatory compliance, and risk assessment.
    All data must remain on local infrastructure (attorney-client privilege).
    @legal/compliance/gdpr-checklist.md
  ```
- **Gooseignore:** `**/contracts/*`, `**/legal_memos/*`, `**/litigation/*`
- **Recipes:**
  - `weekly-compliance-scan` (schedule: `0 9 * * 1`)
  - `contract-expiry-alerts` (schedule: `0 9 1 * *`)
  - `monthly-risk-assessment` (schedule: `0 9 1 * *`)
- **Privacy:** `strict` mode, `allow_override: false`, `local_only: true`
- **Memory:** `retention_days: 0` (ephemeral only, deleted on session close)

**Tasks:**
1. Create 6 profile YAML files (`profiles/*.yaml`)
2. Create 18 recipe YAML files (`recipes/{role}/*.yaml`) — placeholders for Phase 5.5 detailed implementation
3. Create goosehints templates (6 global + 10 local templates)
4. Create gooseignore templates (6 global + 8 local templates)
5. Seed Postgres `profiles` table (SQL insert script)
6. Integration tests: Load each profile, validate schema, verify signing

**Backward Compatibility:**
- Existing roles (Finance, Manager) from Phase 3 maintained
- New roles (Analyst, Legal) added without breaking existing workflows

---

#### **C. RBAC/ABAC Policy Engine (2 days)**  
**Builds On:** Phase 1 JWT roles, Phase 3 Controller API

**Policy Evaluation Functions:**
```rust
// src/policy/engine.rs
pub struct PolicyEngine {
    postgres_pool: PgPool,
    redis_cache: RedisClient,  // Cache policy results (TTL: 5 min)
}

impl PolicyEngine {
    /// Evaluate if role can use tool
    pub async fn can_use_tool(&self, role: &str, tool_name: &str) -> Result<bool> {
        // 1. Check cache
        if let Some(cached) = self.redis_cache.get(&format!("policy:{}:{}", role, tool_name)).await? {
            return Ok(cached);
        }
        
        // 2. Load profile from Postgres
        let profile = self.load_profile(role).await?;
        
        // 3. Evaluate policies
        for policy in &profile.policies {
            if policy.matches_tool(tool_name) {
                let result = policy.allow && policy.conditions_met(&context)?;
                // Cache result
                self.redis_cache.set(&format!("policy:{}:{}", role, tool_name), result, 300).await?;
                return Ok(result);
            }
        }
        
        // 4. Deny by default
        Ok(false)
    }
    
    /// Evaluate if role can access data
    pub async fn can_access_data(&self, role: &str, data_type: &str, context: &PolicyContext) -> Result<bool> {
        // Similar logic: check cache → load profile → evaluate conditions → deny by default
    }
}
```

**Extension Allowlists:**
- Finance: ❌ `developer__shell` (no code execution)
- Manager: ❌ `privacy-guard__disable` (cannot bypass privacy)
- Legal: ❌ all cloud providers (local-only)
- Analyst: ✅ `sql-mcp__query` (only `analytics_*` databases)

**Storage:**
```sql
CREATE TABLE policies (
  id SERIAL PRIMARY KEY,
  role VARCHAR(50) NOT NULL,
  tool_pattern VARCHAR(200) NOT NULL,  -- e.g., "github__*" or "sql-mcp__query"
  allow BOOLEAN DEFAULT FALSE,
  conditions JSONB,  -- {"database": "analytics_*"}
  reason TEXT,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_policies_role_tool ON policies(role, tool_pattern);
```

**Controller Middleware (Axum layer):**
```rust
// src/middleware/policy.rs
pub async fn enforce_policy(
    State(engine): State<Arc<PolicyEngine>>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response> {
    let role = extract_role_from_jwt(&req)?;
    let tool_name = extract_tool_from_request(&req)?;
    
    if !engine.can_use_tool(&role, &tool_name).await? {
        return Err(Error::Forbidden(format!("Role {} cannot use tool {}", role, tool_name)));
    }
    
    Ok(next.run(req).await)
}
```

**Tasks:**
1. Implement `PolicyEngine` struct (200 lines Rust)
2. Postgres policy storage schema + seed data
3. Redis caching integration (reuse Phase 4 Redis client)
4. Axum middleware integration (inject into Controller routes)
5. Unit tests (25+ cases: allow, deny, conditions, cache hit/miss)
6. Integration tests: Finance tries `developer__shell` → 403 Forbidden

**Backward Compatibility:**
- New middleware layer, but defaults to `allow_all` for roles without policies
- Phase 1-4 workflows unaffected (no policies = no restrictions)

---

#### **D. Profile API Endpoints (8 routes) (1.5 days)**  
**Builds On:** Phase 3 Controller API, Phase 4 Postgres

**New Routes:**
```rust
// src/routes/profiles.rs

// 1. Get full profile (replaces Phase 3 mock)
GET /profiles/{role}
Response: {
  "role": "analyst",
  "display_name": "Business Analyst",
  "providers": {...},
  "extensions": [...],
  "goosehints": {...},
  "recipes": [...],
  "policies": [...],
  "privacy": {...}
}

// 2. Generate config.yaml for user
GET /profiles/{role}/config
Response: (generated config.yaml as text/plain)
```
provider: openrouter
model: anthropic/claude-3.5-sonnet
temperature: 0.3

extensions:
  - name: agent_mesh
    enabled: true
  - name: excel-mcp
    enabled: true
```

// 3. Get global goosehints
GET /profiles/{role}/goosehints
Response: (text/plain, ready to save as ~/.config/goose/.goosehints)
```
# Analyst Role Context
You are a business analyst...
@README.md
```

// 4. Get global gooseignore
GET /profiles/{role}/gooseignore
Response: (text/plain, ready to save as ~/.config/goose/.gooseignore)
```
**/.env
**/secrets.*
```

// 5. Get local hints template
GET /profiles/{role}/local-hints?path=<project_path>
Query: path=finance/budgets
Response: (text/plain, ready to save as finance/budgets/.goosehints)

// 6. List recipes
GET /profiles/{role}/recipes
Response: {
  "recipes": [
    {"name": "daily-kpi-report", "schedule": "0 9 * * 1-5", "enabled": true},
    ...
  ]
}

// 7. Create profile (admin only)
POST /admin/profiles
Body: {...profile YAML...}
Response: {"role": "analyst", "created_at": "2025-11-05T14:00:00Z"}

// 8. Update profile (admin only)
PUT /admin/profiles/{role}
Body: {...partial update...}
Response: {"role": "analyst", "updated_at": "2025-11-05T14:15:00Z"}

// 9. Publish profile (sign with Vault)
POST /admin/profiles/{role}/publish
Response: {"role": "analyst", "signature": "...", "signed_at": "2025-11-05T14:20:00Z"}
```

**Auth:**
- Routes 1-6: Require JWT with `role` claim matching requested role
- Routes 7-9: Require JWT with `admin` role claim

**Tasks:**
1. Implement 9 route handlers (300 lines Rust)
2. Vault signing for `publish` endpoint (reuse Phase 1 Vault client)
3. config.yaml generation (template from Goose v1.12.1 spec)
4. Unit tests (20+ cases: valid role, invalid role, admin auth, etc.)
5. Integration tests: Finance user fetches Finance profile → 200 OK, tries Legal profile → 403 Forbidden

**Backward Compatibility:**
- `GET /profiles/{role}` already exists from Phase 3
- This workstream replaces 501 responses with real data
- **No breaking changes** to existing API contract

---

#### **E. Privacy Guard MCP Extension (2 days)**  
**Builds On:** Phase 2 Privacy Guard (regex + NER), Phase 2.2 (Ollama integration)

**Why MCP vs Upstream Goose Change:**
- ✅ No dependency on upstream approval
- ✅ Users opt-in via extension config
- ✅ Fully under our control
- ✅ Can distribute via Block registry

**Architecture:**
```
Goose Client
  ↓ (sends prompt)
Privacy Guard MCP (stdio mode, runs locally)
  ↓ (applies redaction: rules + NER + hybrid)
  ↓ (tokenizes PII: "John Smith" → [PERSON_A])
  ↓ (stores tokens locally: ~/.goose/pii-tokens/session_abc123.json)
OpenRouter/Anthropic API
  ↓ (receives ONLY redacted text)
  ↓ (responds with tokens: "[PERSON_A] approved...")
Privacy Guard MCP
  ↓ (detokenizes response)
  ↓ (sends audit log to Controller: POST /privacy/audit)
Goose Client
  ↓ (user sees unredacted response)
```

**Implementation:**
```rust
// privacy-guard-mcp/src/main.rs
use mcp_server::{Server, Tool, ToolParam, ToolResult};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let server = Server::new("privacy-guard");
    
    // Register as provider proxy
    server.register_provider_proxy(ProviderProxyConfig {
        name: "privacy-guard",
        intercept_requests: true,
        intercept_responses: true,
    })?;
    
    // Request interceptor
    server.on_request(|req: ProviderRequest| async move {
        // 1. Load profile from Controller (cache locally)
        let profile = fetch_profile(&req.role).await?;
        
        // 2. Apply redaction
        let redacted = apply_redaction(&req.prompt, &profile.privacy).await?;
        
        // 3. Tokenize PII
        let (tokenized, token_map) = tokenize_pii(&redacted)?;
        
        // 4. Store tokens locally
        store_tokens(&req.session_id, &token_map).await?;
        
        // 5. Forward to LLM
        ProviderRequest {
            prompt: tokenized,
            ..req
        }
    });
    
    // Response interceptor
    server.on_response(|res: ProviderResponse| async move {
        // 1. Load tokens
        let token_map = load_tokens(&res.session_id).await?;
        
        // 2. Detokenize
        let restored = detokenize(&res.content, &token_map)?;
        
        // 3. Send audit log to Controller
        send_audit_log(&res.session_id, &token_map).await?;
        
        // 4. Clean up tokens
        delete_tokens(&res.session_id).await?;
        
        ProviderResponse {
            content: restored,
            ..res
        }
    });
    
    server.run_stdio().await
}

async fn apply_redaction(text: &str, config: &PrivacyConfig) -> Result<String> {
    match config.mode {
        PrivacyMode::Rules => apply_regex_rules(text, &config.rules),
        PrivacyMode::Ner => apply_ner_model(text).await,
        PrivacyMode::Hybrid => {
            let rules_result = apply_regex_rules(text, &config.rules)?;
            apply_ner_model(&rules_result).await
        }
    }
}

async fn apply_ner_model(text: &str) -> Result<String> {
    // Call Ollama NER model (reuse Phase 2.2 implementation)
    let ollama_url = env::var("OLLAMA_URL").unwrap_or("http://localhost:11434".to_string());
    let response = reqwest::Client::new()
        .post(format!("{}/api/generate", ollama_url))
        .json(&json!({
            "model": "llama3.2:latest",
            "prompt": format!("Redact PII in this text:\n\n{}", text),
            "stream": false
        }))
        .send()
        .await?;
    
    // Parse NER response, apply redactions
    // (Reuse Phase 2.2 logic)
}
```

**User Configuration (Goose config.yaml):**
```yaml
provider: openrouter
model: anthropic/claude-3.5-sonnet

# Privacy Guard MCP
modifiers:
  - name: privacy-guard
    type: stdio
    command: ["privacy-guard-mcp"]
    config:
      controller_url: "https://controller.company.com"
      mode: "hybrid"  # rules, ner, hybrid
      strictness: "moderate"  # strict, moderate, permissive
```

**User Override Settings (Goose Client UI):**
```
Settings → Privacy Guard
  Mode: [Hybrid ▼]  ← Dropdown: Rules Only / NER / Hybrid
  Strictness: [Moderate ▼]  ← Dropdown: Strict / Moderate / Permissive
  Categories: ☑ SSN  ☑ Email  ☑ Phone  ☑ Employee ID
  [Reset to Profile Defaults]  [Save Changes]
```

**Overrides stored locally:**
```yaml
# ~/.config/goose/privacy-overrides.yaml
mode: "rules"  # User downgraded from hybrid
strictness: "strict"  # User upgraded
disabled_categories: ["EMAIL"]  # User wants emails unredacted
```

**Tasks:**
1. Create `privacy-guard-mcp` Rust crate (500 lines)
2. Implement request/response interceptors
3. Token storage (encrypted JSON files in `~/.goose/pii-tokens/`)
4. Controller audit endpoint: `POST /privacy/audit` (metadata only, no content)
5. User override UI mockup (for Goose upstream proposal)
6. Integration tests:
   - Finance sends "John SSN 123-45-6789" → OpenRouter sees "[PERSON_A] SSN [SSN_XXX]"
   - Legal sends contract → Ollama local (never hits cloud)
7. Performance tests: P50 < 500ms for 80% of requests (regex-only)

**Backward Compatibility:**
- Fully optional: users without MCP config get no privacy protection (existing behavior)
- Phase 2/2.2 Privacy Guard still works for Controller-side protection
- **No breaking changes** to existing workflows

---

#### **F. Org Chart HR Import (1 day)**  
**NEW REQUIREMENT:** Admin uploads CSV → System builds org chart → Assigns profiles

**CSV Format:**
```csv
user_id,reports_to_id,name,role,email,department
1,,Alice Johnson,manager,alice@company.com,Executive
2,1,Bob Smith,finance,bob@company.com,Finance
3,1,Carol Lee,analyst,carol@company.com
4,1,David Kim,legal,david@company.com
5,2,Eve Martinez,finance,eve@company.com
```

**Postgres Schema:**
```sql
CREATE TABLE org_users (
  user_id INTEGER PRIMARY KEY,
  reports_to_id INTEGER REFERENCES org_users(user_id),
  name VARCHAR(100),
  role VARCHAR(50) REFERENCES profiles(role),
  email VARCHAR(200) UNIQUE,
  department VARCHAR(100) NOT NULL,  -- NEW: Department/team name
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Department index for filtering
CREATE INDEX idx_org_users_department ON org_users(department);

CREATE TABLE org_imports (
  id SERIAL PRIMARY KEY,
  filename VARCHAR(200),
  uploaded_by VARCHAR(200),  -- Admin email
  uploaded_at TIMESTAMP DEFAULT NOW(),
  users_created INTEGER,
  users_updated INTEGER,
  status VARCHAR(20)  -- pending, processing, complete, failed
);
```

**API Endpoint:**
```rust
POST /admin/org/import
Content-Type: multipart/form-data
Body: {file: org_chart.csv}
Response: {
  "import_id": 123,
  "users_created": 5,
  "users_updated": 0,
  "status": "complete"
}

GET /admin/org/imports
Response: {
  "imports": [
    {"id": 123, "filename": "org_chart.csv", "uploaded_at": "...", "status": "complete"},
    ...
  ]
}

GET /admin/org/tree
Response: {
  "tree": {
    "user_id": 1,
    "name": "Alice Johnson",
    "role": "manager",
    "children": [
      {"user_id": 2, "name": "Bob Smith", "role": "finance", "children": [...]},
      ...
    ]
  }
}
```

**UI (Admin Settings Page):**
```
Org Chart Import
  [Upload CSV] ← File picker
  
  Import History:
  - org_chart.csv (Nov 5, 2025) - 5 users created ✅
  - org_update.csv (Oct 15, 2025) - 3 users updated ✅
  
  Current Org Chart:
  [D3.js tree visualization showing hierarchy]
```

**Tasks:**
1. Implement CSV parser (Rust `csv` crate)
2. Postgres schema + migrations
3. Upload endpoint + validation (check role exists in `profiles` table)
4. Tree builder (recursive query to build hierarchy)
5. Admin UI integration (file upload widget)
6. Unit tests (10+ cases: valid CSV, missing role, circular reports_to, etc.)

**Backward Compatibility:**
- New feature, no impact on existing workflows

---

#### **G. Admin UI (SvelteKit) (3 days)**  
**Technology:** SvelteKit + Tailwind CSS + D3.js + Monaco Editor

**Pages:**

**1. Dashboard** (`src/routes/+page.svelte`)
```svelte
<script lang="ts">
  import OrgChart from '$lib/components/OrgChart.svelte';  // D3.js tree
  import AgentStatus from '$lib/components/AgentStatus.svelte';
  import RecentActivity from '$lib/components/RecentActivity.svelte';
  
  // Fetch data
  onMount(async () => {
    orgData = await fetch('/admin/org/tree').then(r => r.json());
    agentStats = await fetch('/admin/agents/status').then(r => r.json());
    recentSessions = await fetch('/sessions?limit=10').then(r => r.json());
  });
</script>

<div class="grid grid-cols-2 gap-4">
  <div class="col-span-2">
    <h2>Organization Chart</h2>
    <OrgChart data={orgData} />
  </div>
  
  <div>
    <h3>Agent Status</h3>
    <AgentStatus stats={agentStats} />
  </div>
  
  <div>
    <h3>Recent Activity</h3>
    <RecentActivity sessions={recentSessions} />
  </div>
</div>
```

**2. Sessions** (`src/routes/sessions/+page.svelte`)
```svelte
<script>
  // Table with filters
  let filter = { role: 'all', status: 'all', date_range: 'week' };
  
  async function loadSessions() {
    const params = new URLSearchParams(filter);
    return fetch(`/sessions?${params}`).then(r => r.json());
  }
</script>

<div class="filters">
  <select bind:value={filter.role}>
    <option value="all">All Roles</option>
    <option value="finance">Finance</option>
    <option value="analyst">Analyst</option>
  </select>
  
  <select bind:value={filter.status}>
    <option value="all">All Status</option>
    <option value="active">Active</option>
    <option value="completed">Completed</option>
  </select>
</div>

<table>
  <thead>
    <tr>
      <th>Session ID</th>
      <th>Role</th>
      <th>Status</th>
      <th>Created At</th>
      <th>Actions</th>
    </tr>
  </thead>
  <tbody>
    {#each sessions as session}
      <tr>
        <td><a href="/sessions/{session.id}">{session.id}</a></td>
        <td><span class="badge role-{session.role}">{session.role}</span></td>
        <td><span class="badge status-{session.status}">{session.status}</span></td>
        <td>{session.created_at}</td>
        <td><button on:click={() => viewSession(session.id)}>View</button></td>
      </tr>
    {/each}
  </tbody>
</table>
```

**3. Profiles** (`src/routes/profiles/+page.svelte`)
```svelte
<script>
  import MonacoEditor from '$lib/components/MonacoEditor.svelte';  // YAML editor
  
  let selectedProfile = null;
  let editorContent = '';
  let isEditing = false;
  
  async function publishProfile() {
    await fetch(`/admin/profiles/${selectedProfile.role}/publish`, {
      method: 'POST',
      body: JSON.stringify({ data: editorContent })
    });
    alert('Profile published!');
  }
</script>

<div class="grid grid-cols-3 gap-4">
  <div>
    <h3>Roles</h3>
    <ul>
      {#each profiles as profile}
        <li on:click={() => selectProfile(profile)}>
          {profile.display_name}
        </li>
      {/each}
    </ul>
    <button on:click={() => createNewProfile()}>+ New Profile</button>
  </div>
  
  <div class="col-span-2">
    {#if selectedProfile}
      <h3>{selectedProfile.display_name}</h3>
      
      {#if isEditing}
        <MonacoEditor
          bind:value={editorContent}
          language="yaml"
        />
        <button on:click={publishProfile}>Publish</button>
        <button on:click={() => isEditing = false}>Cancel</button>
      {:else}
        <pre>{JSON.stringify(selectedProfile, null, 2)}</pre>
        <button on:click={() => isEditing = true}>Edit</button>
      {/if}
      
      <h4>Policy Tester</h4>
      <input type="text" placeholder="Tool name (e.g., developer__shell)" bind:value={testTool} />
      <button on:click={testPolicy}>Test Policy</button>
      <div class="result">{policyResult}</div>
    {/if}
  </div>
</div>
```

**4. Audit** (`src/routes/audit/+page.svelte`)
```svelte
<script>
  let filters = { event_type: 'all', role: 'all', trace_id: '' };
  
  async function exportCsv() {
    const csv = await fetch('/audit/export?format=csv&' + new URLSearchParams(filters))
      .then(r => r.text());
    downloadFile('audit.csv', csv);
  }
</script>

<div class="filters">
  <input type="text" placeholder="Trace ID" bind:value={filters.trace_id} />
  <select bind:value={filters.event_type}>
    <option value="all">All Events</option>
    <option value="privacy_redaction">Privacy Redaction</option>
    <option value="task_routed">Task Routed</option>
  </select>
  <button on:click={exportCsv}>Export CSV</button>
</div>

<table>
  <!-- Audit events table -->
</table>
```

**5. Settings** (`src/routes/settings/+page.svelte`)
```svelte
<script>
  import OrgImport from '$lib/components/OrgImport.svelte';
  import PrivacyGuardConfig from '$lib/components/PrivacyGuardConfig.svelte';
  
  let systemVars = { SESSION_RETENTION_DAYS: 90, IDEMPOTENCY_TTL: 86400 };
  
  async function saveSettings() {
    await fetch('/admin/settings', {
      method: 'PUT',
      body: JSON.stringify(systemVars)
    });
  }
</script>

<div class="settings-sections">
  <section>
    <h3>System Variables</h3>
    <label>
      Session Retention (days):
      <input type="number" bind:value={systemVars.SESSION_RETENTION_DAYS} />
    </label>
    <label>
      Idempotency TTL (seconds):
      <input type="number" bind:value={systemVars.IDEMPOTENCY_TTL} />
    </label>
    <button on:click={saveSettings}>Save</button>
  </section>
  
  <section>
    <h3>Privacy Guard</h3>
    <PrivacyGuardConfig />
  </section>
  
  <section>
    <h3>Org Chart Import</h3>
    <OrgImport />
  </section>
  
  <section>
    <h3>User-Profile Assignment</h3>
    <table>
      <thead>
        <tr>
          <th>Email</th>
          <th>Assigned Profile</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each users as user}
          <tr>
            <td>{user.email}</td>
            <td>
              <select bind:value={user.role}>
                {#each profiles as profile}
                  <option value={profile.role}>{profile.display_name}</option>
                {/each}
              </select>
            </td>
            <td><button on:click={() => assignProfile(user)}>Update</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </section>
  
  <section>
    <h3>Service Health</h3>
    <ul>
      <li>Controller: <span class="badge {controllerStatus}">{controllerStatus}</span></li>
      <li>Keycloak: <span class="badge {keycloakStatus}">{keycloakStatus}</span></li>
      <li>Vault: <span class="badge {vaultStatus}">{vaultStatus}</span></li>
      <li>Privacy Guard MCP: <span class="badge {guardStatus}">{guardStatus}</span></li>
      <li>Ollama: <span class="badge {ollamaStatus}">{ollamaStatus}</span></li>
    </ul>
  </section>
</div>
```

**Deployment:**
```rust
// src/main.rs (Controller)
use tower_http::services::ServeDir;

let app = Router::new()
    // API routes
    .route("/profiles/:role", get(get_profile))
    // ...
    
    // Serve UI static files
    .nest_service("/", ServeDir::new("ui/build"));

// Build UI
// cd ui && npm run build
```

**Tasks:**
1. Setup SvelteKit project (`npm create svelte@latest ui`)
2. Install dependencies (Tailwind, D3.js, Monaco Editor)
3. Implement 5 pages + 10 components (1,500 lines TypeScript)
4. JWT auth integration (Keycloak redirect flow)
5. API client (`src/lib/api.ts`)
6. Build configuration (output to `ui/build/`)
7. Integration tests (Playwright): Load dashboard → see org chart

**Backward Compatibility:**
- New feature, serves at `/` (root path)
- API routes remain at `/profiles`, `/sessions`, etc. (no conflicts)

---

#### **H. Integration Testing + Backward Compatibility (1 day)**  
**CRITICAL:** Prove Phase 1-4 stack still works + new features functional

**Test Suites:**

**1. Phase 1-4 Regression Tests:**
```bash
# Phase 1: OIDC/JWT
./tests/integration/test_oidc_login.sh
./tests/integration/test_jwt_verification.sh

# Phase 2: Privacy Guard (regex + NER)
./tests/integration/test_privacy_guard_regex.sh
./tests/integration/test_privacy_guard_ner.sh

# Phase 3: Controller API + Agent Mesh
./tests/integration/test_controller_routes.sh
./tests/integration/test_agent_mesh_tools.sh

# Phase 4: Session Persistence
./tests/integration/test_session_crud.sh
./tests/integration/test_idempotency.sh

# ALL MUST PASS (6/6 from Phase 4)
```

**2. Phase 5 New Feature Tests:**
```bash
# Profile System
./tests/integration/test_profile_loading.sh        # Finance user fetches Finance profile
./tests/integration/test_config_generation.sh      # Generate config.yaml
./tests/integration/test_goosehints_download.sh    # Download global hints
./tests/integration/test_recipe_sync.sh            # Sync recipes

# Privacy Guard MCP
./tests/integration/test_privacy_mcp_redaction.sh  # PII tokenization
./tests/integration/test_privacy_mcp_audit.sh      # Audit log sent to Controller

# Org Chart
./tests/integration/test_org_import.sh             # Upload CSV → build tree
./tests/integration/test_org_tree_api.sh           # GET /admin/org/tree

# Admin UI
./tests/integration/test_ui_dashboard.sh           # Playwright: load dashboard
./tests/integration/test_ui_profiles.sh            # Playwright: edit profile
```

**3. End-to-End Workflow:**
```bash
#!/bin/bash
# tests/integration/e2e_phase5.sh

# 1. Admin uploads org chart
curl -X POST -F "file=@test_org.csv" http://localhost:8000/admin/org/import

# 2. Analyst user signs in (Keycloak OIDC)
JWT=$(./tests/helpers/oidc_login.sh analyst@company.com password123)

# 3. Analyst fetches profile
PROFILE=$(curl -H "Authorization: Bearer $JWT" http://localhost:8000/profiles/analyst)
echo "$PROFILE" | jq '.providers.primary.model' | grep "anthropic/claude-3.5-sonnet"

# 4. Analyst downloads config.yaml
curl -H "Authorization: Bearer $JWT" http://localhost:8000/profiles/analyst/config > ~/.config/goose/config.yaml

# 5. Analyst downloads goosehints
curl -H "Authorization: Bearer $JWT" http://localhost:8000/profiles/analyst/goosehints > ~/.config/goose/.goosehints

# 6. Analyst sends task with PII (Privacy Guard MCP active)
RESPONSE=$(curl -X POST http://localhost:8000/tasks/route \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Analyze employee John Smith SSN 123-45-6789"}')

# 7. Verify OpenRouter never saw raw PII (check audit log)
AUDIT=$(curl -H "Authorization: Bearer $JWT" http://localhost:8000/privacy/audit?session_id=...)
echo "$AUDIT" | jq '.redactions' | grep "2"  # 2 redactions (name + SSN)

# 8. Admin views org chart in UI
TREE=$(curl -H "Authorization: Bearer $JWT" http://localhost:8000/admin/org/tree)
echo "$TREE" | jq '.tree.children | length' | grep "5"  # 5 direct reports

echo "✅ End-to-end test passed!"
```

**4. Performance Tests:**
```bash
# Latency targets
wrk -t4 -c100 -d30s --latency http://localhost:8000/profiles/analyst
# Target: P50 < 100ms (cached), P99 < 500ms

# Privacy Guard overhead
time ./tests/perf/test_privacy_guard_latency.sh
# Target: P50 < 500ms (regex-only), P99 < 2s (with NER)
```

**Tasks:**
1. Create 20+ integration test scripts (Bash + curl + jq)
2. Run full regression suite (Phase 1-4 tests)
3. Run new feature tests (Phase 5)
4. Run end-to-end workflow test
5. Performance validation (P50 < 5s target)
6. Document test results in `docs/tests/phase5-test-results.md`

**Acceptance Criteria:**
- ✅ All Phase 1-4 tests pass (no regressions)
- ✅ All Phase 5 tests pass (new features work)
- ✅ E2E workflow passes (full stack integration)
- ✅ Performance targets met (P50 < 5s)

---

#### **I. Documentation (1 day)**  
**Artifacts:**

1. **Profile Spec** (`docs/profiles/SPEC.md`)
   - Schema definition (YAML format)
   - Field descriptions (providers, extensions, goosehints, recipes, etc.)
   - Examples for all 6 roles
   - Validation rules

2. **Privacy Guard MCP Guide** (`docs/privacy/PRIVACY-GUARD-MCP.md`)
   - Installation (`cargo install privacy-guard-mcp`)
   - Configuration (Goose config.yaml)
   - User override settings
   - Troubleshooting

3. **Admin Guide** (`docs/admin/ADMIN-GUIDE.md`)
   - Org chart import (CSV format)
   - Profile creation/editing
   - User-profile assignment
   - Policy testing

4. **API Reference** (OpenAPI spec update)
   - Add 8 new profile endpoints
   - Add 3 org chart endpoints
   - Add 1 privacy audit endpoint

5. **Migration Guide** (`docs/MIGRATION-PHASE5.md`)
   - Upgrading from v0.4.0 to v0.5.0
   - Database migrations
   - Breaking changes (none expected)
   - New features overview

**Tasks:**
1. Write 5 documentation files (2,000+ lines Markdown)
2. Update OpenAPI spec (add 12 new endpoints)
3. Create diagrams (architecture, data flow, org chart example)
4. Record screenshots (UI pages for admin guide)
5. Proofread and publish to GitHub Pages

---

#### **J. Progress Tracking (~15 min) 🚨 CHECKPOINT**

**Tasks:**
1. Update `Technical Project Plan/PM Phases/Phase-5/Phase-5-Agent-State.json`
2. Update `docs/tests/phase5-progress.md` (timestamped entries)
3. Commit to git:
   ```bash
   git add .
   git commit -m "Phase 5 complete: Profile system + Privacy Guard MCP + Admin UI"
   git push origin main
   ```
4. Create GitHub release tag `v0.5.0`
5. Report to user: "Phase 5 complete! ✅"

---

### **Deliverables (Phase 5):**

**Code:**
- ✅ 60+ files (20 profile system, 18 recipes, 25 UI, 10 docs/tests)
- ✅ 5,000+ lines of code (Rust backend + SvelteKit frontend + Privacy Guard MCP)

**Features:**
- ✅ 6 role profiles (Finance, Manager, Analyst, Marketing, Support, Legal)
- ✅ 18 recipe templates (3 per role)
- ✅ Privacy Guard MCP (tokenization, local-only Legal, user overrides)
- ✅ Admin UI (5 pages: Dashboard, Sessions, Profiles, Audit, Settings)
- ✅ Org chart HR import (CSV → tree visualization)
- ✅ 12 new API endpoints (8 profiles, 3 org, 1 privacy audit)

**Database:**
- ✅ 3 new tables (`profiles`, `org_users`, `org_imports`)
- ✅ Migrations scripts (sqlx)

**Tests:**
- ✅ 50+ unit tests (profile validation, policy engine, API routes)
- ✅ 25+ integration tests (regression + new features)
- ✅ 1 end-to-end workflow test
- ✅ Performance validation (P50 < 5s ✅)

**Documentation:**
- ✅ 5 guides (2,000+ lines Markdown)
- ✅ OpenAPI spec updated (12 endpoints)
- ✅ Architecture diagrams (3 diagrams)

**Backward Compatibility:**
- ✅ Phase 1-4 tests pass (no regressions)
- ✅ Existing API contracts maintained
- ✅ Optional features (Privacy Guard MCP = opt-in)

**Tagged Release:**
- ✅ v0.5.0 (Grant application ready)

---

**Effort:** L (~1.5-2 weeks, 10-12 days with UI + Privacy Guard MCP + Integration Testing)

---

**Risk Mitigation:**
- Privacy Guard MCP: Fallback to Phase 2.2 Controller-side guard if MCP development blocked
- UI Complexity: Start with minimal UI (no Monaco editor), add features incrementally
- Org Chart Import: Defer to Phase 6 if CSV parsing issues arise (not critical for grant)
- Performance: Profile caching in Redis (Phase 4) already proven, reuse pattern

---

### Phase 5.5: Grant Application Demo (~1 week) — GRANT Q1 FINAL DELIVERABLE
**Timeline:** Week 7  
**Target:** Grant application submitted

Tasks:
- A. Demo Video (~2 days)
  - 5-minute walkthrough: Finance → Manager → Engineering cross-agent workflow
  - Show UI: org chart, session tracking, audit trail
  - Narration: problem, solution, impact
  - Tools: OBS Studio (screen recording), DaVinci Resolve (editing)

- B. Documentation (~2 days)
  - Architecture diagrams (system overview, deployment topologies, data flow)
  - API documentation (OpenAPI spec refinement, MCP tool reference)
  - User guide (quickstart, role profiles, troubleshooting)
  - Contributor guide (CONTRIBUTING.md, PR templates, issue templates)

- C. Performance Benchmarks (~1 day)
  - Latency: P50/P95/P99 for API routes (target: P50 < 5s)
  - Throughput: Requests/second (target: > 10 req/s)
  - Cost: Token usage per workflow (budget tracking)
  - Load testing: Apache Bench or wrk

- D. GitHub Repository Polish (~1 day)
  - README.md update (badges, demo GIF, features list, installation)
  - LICENSE file (Apache-2.0)
  - CODE_OF_CONDUCT.md (Contributor Covenant)
  - SECURITY.md (responsible disclosure policy)
  - Issue templates (bug report, feature request)
  - PR template (checklist, testing, docs)

- E. Grant Application Submission (~1 day)
  - Fill out Block Goose Innovation Grant form
  - Upload demo video, link GitHub repo
  - Submit answers (see docs/grant/GRANT-APPLICATION-ANALYSIS.md)
  - Post to Goose Discord/community

**Deliverables:**
- ✅ 5-minute demo video (YouTube/Vimeo public)
- ✅ Comprehensive documentation (architecture, API, user guide, contributor guide)
- ✅ Performance benchmarks documented (latency, throughput, cost)
- ✅ GitHub repository polished (README, LICENSE, COC, SECURITY, templates)
- ✅ Grant application submitted

**Effort:** S (~1 week, 5-7 days)

---

### Phase 6: Production Hardening + UIs + Vault Completion (L) — Q2 (2-3 weeks)
**Timeline:** Weeks 8-10  
**Target:** Production-ready v0.6.0

**Goals:**
1. Complete Vault production integration (TLS, AppRole, persistent storage, audit, signature verification)
2. Build Admin UI (SvelteKit - profile editor, org chart viz, Vault status)
3. Build lightweight User UI (Goose backend + Privacy Guard middleware)
4. Basic security hardening (secrets cleanup, environment variable audit)
5. Address security concerns (no secrets in repo, .env.example only)

---

#### **Workstreams:**

**A. Vault Production Completion (2 days)** - *Builds on Phase 5 Vault dev mode*
- **TLS/HTTPS Setup** (2 hours):
  * Generate TLS certificates (OpenSSL or Let's Encrypt)
  * Update Vault config (listener with tls_cert_file/tls_key_file)
  * Update VAULT_ADDR to https://vault:8200
  * Add VAULT_CACERT for cert validation
  * Test: `curl --cacert ca.crt https://vault:8200/v1/sys/health`

- **AppRole Authentication** (3 hours):
  * Enable AppRole auth method
  * Create controller-role with Transit permissions only
  * Generate role_id (static), secret_id (rotatable)
  * Update controller code (Rust: AppRole login function)
  * Implement token renewal (background task, 45-min intervals)
  * Remove VAULT_TOKEN from environment
  * Test: Controller auth → Vault sign → Success

- **Persistent Storage** (2 hours):
  * Configure Raft storage backend (recommended)
  * Update docker-compose with persistent volume
  * Initialize Vault (generates unseal keys + root token)
  * Document unseal procedure (3 of 5 keys)
  * Optional: Automate unseal with AWS KMS

- **Audit Device** (1 hour):
  * Enable file audit device (`vault audit enable file file_path=/vault/logs/audit.log`)
  * Mount /vault/logs volume
  * Configure log rotation (logrotate)
  * Verify: All Vault operations logged

- **Signature Verification** (2 hours):
  * Add verification to profile loading (GET /profiles/{role})
  * Implement verify_hmac function (Rust: calls Vault verify endpoint)
  * Reject tampered profiles (403 Forbidden if signature invalid)
  * Add audit log for verification failures

**Deliverables:**
- ✅ Vault production-ready (HTTPS, AppRole, Raft, audit)
- ✅ Profile signature verification enforced (tamper detection)
- ✅ Vault operations guide updated (docs/guides/VAULT.md)

**Effort:** M (10 hours actual, aligns with Phase 5 documented plan)

---

**B. Admin UI (SvelteKit) (3 days)** - *Builds on Phase 5 profiles + org chart*
**Technology:** SvelteKit + Tailwind CSS + D3.js + Monaco Editor

**Pages:**
1. **Dashboard** (`/admin`)
   - D3.js org chart visualization
   - Live agent status (active sessions)
   - Recent activity feed
   - Vault health status

2. **Profiles** (`/admin/profiles`)
   - Profile list (6 roles)
   - Monaco YAML editor for profile editing
   - Publish button (triggers Vault signing)
   - Policy tester (test tool access)

3. **Org Chart** (`/admin/org`)
   - CSV upload widget (drag-and-drop)
   - Import history table
   - Tree visualization (D3.js hierarchical layout)

4. **Audit Logs** (`/admin/audit`)
   - Table with filters (event_type, role, date range)
   - Export CSV button
   - Trace ID search

5. **Settings** (`/admin/settings`)
   - Vault status (sealed/unsealed, version, key version)
   - System variables (session retention, idempotency TTL)
   - Service health checks (Controller, Keycloak, Vault, Postgres, Privacy Guard, Ollama)

**Authentication:** Keycloak OIDC redirect flow (admin role required)

**Deployment:**
```rust
// src/main.rs (Controller)
use tower_http::services::ServeDir;

let app = Router::new()
    .route("/profiles/:role", get(get_profile))  // API
    .nest_service("/", ServeDir::new("ui/build"));  // UI static files
```

**Deliverables:**
- ✅ 5 admin pages functional
- ✅ JWT auth integrated (Keycloak OIDC)
- ✅ D3.js org chart visualization
- ✅ Monaco YAML editor for profiles
- ✅ Vault status dashboard

**Effort:** M (3 days: 1,500 lines TypeScript, 10 components)

---

**C. Lightweight User UI (2 days)** - *Goose backend + Privacy Guard middleware*
**Architecture:**
```
User Browser (SvelteKit Lightweight UI)
  ↓ (HTTP/SSE)
Goose Desktop (Backend Mode)
  ↓ (MCP stdio)
Privacy Guard MCP (Middleware)
  ↓ (LLM API)
OpenRouter/Cloud LLMs
```

**Features:**
1. **Profile Viewer** (`/`)
   - My Profile card (display_name, role, providers, extensions, privacy settings)
   - Download config buttons (config.yaml, .goosehints, .gooseignore)
   - Privacy settings override UI (mode, strictness, categories)

2. **Chat Interface** (`/chat`)
   - Goose Desktop backend (HTTP API mode)
   - Send prompts → Privacy Guard → LLM → Response
   - Session history (local browser storage)
   - PII redaction indicators (show what was masked)

3. **Sessions** (`/sessions`)
   - My sessions table (session_id, status, created_at)
   - View session details
   - Privacy audit log (what PII was detected/masked)

**Backend:** Goose Desktop HTTP mode (Phase 5 Goose already supports HTTP API)
```bash
# Launch Goose in HTTP server mode
goose serve --port 8090 --profile finance
```

**Privacy Guard Integration:**
```yaml
# User's config.yaml
modifiers:
  - name: privacy-guard
    type: stdio
    command: ["privacy-guard-mcp"]
    config:
      controller_url: "https://controller.company.com"
      mode: "hybrid"
      strictness: "moderate"
```

**Deliverables:**
- ✅ 3 user pages functional (Profile, Chat, Sessions)
- ✅ Goose Desktop backend integration
- ✅ Privacy Guard middleware working
- ✅ PII redaction indicators in UI

**Effort:** S (2 days: 800 lines TypeScript)

---

**D. Basic Security Hardening (1 day)**
**Focus:** Remove secrets from repo, environment variable audit

**Tasks:**
1. **Secrets Cleanup:**
   - Remove any hardcoded secrets from code (grep -r "password\|secret\|token" src/)
   - Move to .env files (already .gooseignored)
   - Create .env.example with placeholder values
   - Document secret management in README.md

2. **Environment Variable Audit:**
   ```bash
   # Audit all environment variables
   grep -r "env::var\|std::env" src/ | sort | uniq > env_vars_audit.txt
   
   # Categorize:
   # - Required (VAULT_ADDR, DATABASE_URL, KEYCLOAK_URL)
   # - Optional (LOG_LEVEL, PORT, REDIS_URL)
   # - Secrets (VAULT_TOKEN → remove, use AppRole)
   ```

3. **docker-compose Security:**
   - Remove default passwords (use .env file)
   - Add security_opt (no-new-privileges)
   - Add read_only where possible
   - Document security considerations in deploy/compose/README.md

4. **README Security Section:**
   ```markdown
   ## Security
   
   - **Secrets Management:** All secrets in .env files (never committed)
   - **Vault Production:** Use AppRole, not root token
   - **TLS:** Enable HTTPS for Vault in production
   - **Audit:** Vault audit device logs all operations
   - **Reporting:** security@example.com for vulnerabilities
   ```

5. **SECURITY.md:**
   - Responsible disclosure policy
   - Security contact email
   - PGP key for encrypted reports
   - CVE remediation process

**Deliverables:**
- ✅ No secrets in repo (verified with grep)
- ✅ .env.example created
- ✅ Environment variable audit complete
- ✅ docker-compose hardened
- ✅ SECURITY.md created

**Effort:** S (1 day: audit, cleanup, documentation)

---

**E. Integration Testing (1 day)**
**Tests:**
1. Vault production flow (AppRole auth → sign → verify → reject tampered)
2. Admin UI smoke tests (Playwright: load dashboard, edit profile, publish)
3. User UI smoke tests (Playwright: load profile, chat with PII, verify redaction)
4. Regression tests (Phase 1-5 still pass)

**Deliverables:**
- ✅ 15+ integration tests passing
- ✅ No regressions from Phase 1-5

---

**F. Documentation (1 day)**
1. Update Vault guide (docs/guides/VAULT.md) with production setup
2. Admin UI guide (docs/admin/ADMIN-UI-GUIDE.md)
3. User UI guide (docs/user/USER-UI-GUIDE.md)
4. Security guide (docs/security/SECURITY-HARDENING.md)
5. Migration guide (docs/MIGRATION-PHASE6.md)

---

**Deliverables (Phase 6):**
- ✅ Vault production-ready (TLS, AppRole, Raft, audit, verify)
- ✅ Admin UI deployed (5 pages, D3.js org chart, Monaco editor)
- ✅ User UI deployed (3 pages, Goose backend, Privacy Guard middleware)
- ✅ Security hardened (no secrets in repo, environment audit, SECURITY.md)
- ✅ 15+ integration tests passing
- ✅ Documentation complete (4 guides)
- ✅ Tagged release: v0.6.0

**Effort:** L (2-3 weeks: 10 days actual)

---

### Phase 7: Privacy Guard NER Quality Improvement (M) — Q2 (1 week)
**Timeline:** Week 11  
**Target:** Production-grade PII detection v0.7.0

**Goals:**
1. Improve NER detection quality based on real-world middleware findings
2. Fine-tune Ollama model OR engineer better prompts
3. Validate with corporate PII datasets
4. Optimize performance (smart model triggering)

---

#### **Workstreams:**

**A. Middleware Findings Analysis (1 day)**
- Collect real-world PII samples from Phase 6 User UI usage
- Analyze false positives (over-redaction)
- Analyze false negatives (missed PII)
- Categorize by entity type (PERSON, ORG, SSN, EMAIL, etc.)
- Prioritize improvements (focus on high-impact entities)

**B. NER Improvement (2 days)**
**Option 1: Fine-Tune Ollama Model** (if dataset large enough)
- Prepare training dataset (50+ annotated PII examples per entity type)
- Fine-tune llama3.2:3b (Ollama supports fine-tuning)
- Validate on held-out test set (precision/recall metrics)
- Deploy fine-tuned model
- Test: Compare old vs new model accuracy

**Option 2: Prompt Engineering** (if dataset too small for fine-tuning)
- Engineer few-shot prompts with examples:
  ```
  Redact PII in this text. Examples:
  - "John Smith SSN 123-45-6789" → "[PERSON_A] SSN [SSN_XXX]"
  - "Contact alice@acme.com" → "Contact [EMAIL_A]"
  
  Now redact: {user_text}
  ```
- Test prompt variations (zero-shot, one-shot, few-shot)
- Measure accuracy improvement (precision/recall)
- Deploy best prompt

**C. Performance Optimization (1 day)**
- **Smart Model Triggering:**
  ```rust
  async fn detect_pii(text: &str, config: &PrivacyConfig) -> Result<Vec<PiiEntity>> {
      // 1. Fast regex pass (P50=16ms)
      let regex_entities = apply_regex_rules(text, &config.rules)?;
      
      // 2. Check confidence
      if regex_entities.iter().all(|e| e.confidence > 0.9) {
          return Ok(regex_entities);  // ✅ Skip NER (240x speedup)
      }
      
      // 3. NER pass for low-confidence (P50=22s → selective usage)
      let ner_entities = apply_ner_model(text).await?;
      
      // 4. Merge results (NER augments regex)
      Ok(merge_entities(regex_entities, ner_entities))
  }
  ```
- Target: P50 < 500ms for 80-90% of requests (regex-only path)
- P99 < 5s for NER path (10-20% of requests)

- **Model Warm-Up:**
  ```rust
  // On startup, load model into memory
  #[tokio::main]
  async fn main() -> Result<()> {
      // Warm up NER model
      let _ = ollama_client.generate("warmup", &WarmupConfig {
          model: "llama3.2:3b",
          prompt: "Test warmup",
      }).await?;
      
      // Now model is loaded (eliminates cold start)
      run_privacy_guard_server().await
  }
  ```

**D. Validation (1 day)**
- **Corporate PII Dataset:**
  - 100+ real-world samples (anonymized)
  - Categories: Finance (SSN, account numbers), Legal (case IDs), HR (employee IDs)
  
- **Metrics:**
  - Precision: TP / (TP + FP) — Target: > 95%
  - Recall: TP / (TP + FN) — Target: > 90%
  - F1 Score: 2 * (Precision * Recall) / (Precision + Recall) — Target: > 92%
  
- **Performance:**
  - P50 latency: < 500ms (target)
  - P95 latency: < 2s (target)
  - P99 latency: < 5s (target)

**E. Documentation (1 day)**
- Update Privacy Guard guide (docs/privacy/PRIVACY-GUARD-NER.md)
- NER quality report (precision/recall metrics)
- Performance benchmarks (latency histograms)
- Troubleshooting guide (when to use regex-only vs NER vs hybrid)

---

**Deliverables (Phase 7):**
- ✅ NER quality improved (precision > 95%, recall > 90%)
- ✅ Performance optimized (P50 < 500ms for 80-90% of requests)
- ✅ Model warm-up implemented (no cold start)
- ✅ Smart model triggering (selective NER usage)
- ✅ Corporate PII dataset validated (100+ samples)
- ✅ Documentation complete (NER quality report, performance benchmarks)
- ✅ Tagged release: v0.7.0

**Effort:** M (1 week: 5 days actual)

---

### Grant-Aligned Timeline Summary

**Months 1-3 (Q1): Foundation & Grant Application**
- Weeks 1-2: Phase 0-3 complete ✅
- Weeks 3-4: Phase 4 (Storage/Metadata + Session Persistence)
- Weeks 5-6: Phase 5 (Directory/Policy + Profiles + Simple UI)
- Week 7: Phase 5.5 (Grant Application Demo & Submission)
- **Deliverable:** Grant application ready (v0.5.0)

**Months 4-6 (Q2): Production Hardening**
- Phase 6: Privacy Guard production-ready
- Phase 7: Audit/Observability enhanced
- First upstream PRs to Goose core
- **Deliverable:** Production-grade orchestration (v0.6.0)

**Months 7-9 (Q3): Scale & Features**
- Phase 8: Model Orchestration
- Phase 9: 10 Role Profiles library
- Phase 10: Kubernetes deployment
- **Deliverable:** Enterprise-ready platform (v0.7.0)

**Months 10-12 (Q4): Community & Sustainability**
- Phase 11: Advanced features (approval workflows, SCIM, compliance)
- Phase 12: Community engagement, blog posts, talks
- Business model validation (2 paid pilots)
- **Deliverable:** Sustainable OSS project (v1.0.0)

**Total Effort:** 12 months, $100K grant funding

**Effort Scale:** S ≤ 2d, M ~ 3–5d, L ~ 1–2w, XL > 2w

## Environment/Infra Plan
- CE defaults: Keycloak (OIDC), Vault OSS + KMS (dev: file KMS), Postgres, Ollama (guard); optional S3-compatible object storage (SeaweedFS default option; MinIO/Garage optional). Object storage is deferred until needed for large artifacts; it is not required to complete MVP flows.
- Topologies:
  - Desktop-only: Goose desktop with local guard and Mesh; optional local Keycloak (dev).
  - Dept/Org: Docker compose with controller + directory-policy + audit ingest + Postgres; agents desktop or container.
- Security zones:
  - Desktop zone (agent+guard), Org zone (controller/policy/audit), Secrets zone (Vault/KMS).

## Governance/Security/Compliance Posture
- Identity: OIDC SSO, short-lived JWT (≤30m), optional refresh via OIDC.
- Authorization: RBAC/ABAC-lite in directory-policy; extension allowlists enforce least privilege.
- Privacy: Mask-and-forward by default; deterministic mapping keys per-tenant; strict no-content server rule.
- Auditability: Structured logs, OTEL traces, audit events with redaction maps; exportable ndjson.
- Compliance-ready: SOC2-aligned practices; GDPR/CPRA pack later.

## Risks and Mitigations (Top 10)
1) Guard accuracy → Start conservative rules + evaluate; manual overrides; defense-in-depth.
2) Auth mismatch (OIDC/JWT vs X-Secret-Key) → Gateway bridge, standardized middleware.
3) Latency from guard + orchestration → Local guard optimization, prompt trimming, caching.
4) Security of inter-agent calls → JWT-bound requests, role claims, signed profiles; mTLS later.
5) Integration sprawl (MCP) → Curated allowlists per role; staged rollout; sandbox risky tools.
6) Data custody creep → Stateless content rule; redaction at source; periodic reviews.
7) Observability signal overload → Minimal baseline; dashboards later; sampling.
8) Cost overruns → Budget policy, downshift, token accounting; guard-first.
9) Change management → Seed role profiles, clear runbooks, training plan.
10) Single-tenant limits scale → Document path to multi-tenant; quotas post-MVP.

## Dependencies
- Goose v1.12 (goosed, MCP, providers, OTLP).
- Keycloak (OIDC), Vault OSS + KMS, Postgres, Ollama + small models, optional S3-compatible object storage (SeaweedFS default option; MinIO/Garage optional), OpenRouter/one-API (optional).
- Linux/macOS desktops.

## Milestones and Acceptance Criteria (Grant-Aligned)

### Q1 Milestones (Months 1-3): Foundation & Grant Application
- **M1 (Week 2) ✅ ACHIEVED:** OIDC + JWT complete, Privacy Guard MVP operational, Vault integrated
- **M2 (Week 4) ✅ ACHIEVED:** Controller API + Agent Mesh complete, cross-agent demo working, session persistence added
- **M3 (Week 7) 🎯 TARGET:** Grant application ready
  - Postgres metadata complete (sessions, tasks, approvals, audit)
  - 5 role profiles operational (Finance, Manager, Engineering, Marketing, Support)
  - Simple web UI deployed (4 pages: Dashboard, Sessions, Profiles, Audit)
  - Demo video recorded (5 minutes, Finance → Manager → Engineering workflow)
  - Performance benchmarks documented (P50 < 5s, throughput > 10 req/s)
  - Tagged release: v0.5.0
  - Grant application submitted to Block

### Q2 Milestones (Months 4-6): Production Hardening
- **M4 (Month 5):** Privacy Guard production-ready
  - Ollama NER model loaded (accurate PII detection)
  - Comprehensive integration tests (MCP → Controller → Guard)
  - Performance: P50 < 500ms validated
  - Deterministic pseudonymization production-ready
  - Tagged release: v0.6.0

- **M5 (Month 6):** Audit/Observability enhanced, first upstream PRs
  - OTLP export to Grafana/Prometheus operational
  - Audit event schema extended for compliance
  - Dashboard templates deployed (org-level oversight, cost tracking)
  - Agent Mesh + Privacy Guard PRs submitted to Goose core
  - External contributor onboarding (1-2 contributors active)

### Q3 Milestones (Months 7-9): Scale & Features
- **M6 (Month 8):** Model Orchestration complete
  - Lead/worker selection operational (guard → planner → worker)
  - Cost-aware downshift implemented (GPT-4 → GPT-3.5)
  - Policy constraints enforced (Legal role: local-only models)
  - Token accounting and budget alerts functional
  - Tagged release: v0.7.0

- **M7 (Month 9):** Enterprise-ready platform
  - 10 role profiles library complete (all departments covered)
  - Kubernetes deployment operational (Helm charts, auto-scaling)
  - Production runbooks finalized (deployment, monitoring, troubleshooting)
  - 2 paid pilots underway ($10K each — business validation)

### Q4 Milestones (Months 10-12): Community & Sustainability
- **M8 (Month 11):** Advanced features deployed
  - Approval workflows configurable (multi-stage approvals)
  - SCIM integration operational (auto-provision agents from IdP)
  - Compliance packs available (GDPR, SOC2, HIPAA templates)
  - Policy composition working (role inheritance, override rules)

- **M9 (Month 12):** Sustainable OSS project
  - 100 production deployments (tracked via opt-in telemetry)
  - 10 external contributors (non-grant-funded PRs merged)
  - 5 upstreamed PRs to Goose core (Agent Mesh, Privacy Guard, docs)
  - 3 conference talks/blog posts (FOSDEM, KubeCon, AI Engineer Summit)
  - Business model validated (2 paid pilots complete, renewal contracts signed)
  - Tagged release: v1.0.0
  - Grant deliverables complete

**Success Criteria (12 Months):**
- ✅ Grant application submitted (Q1)
- ✅ Production-grade orchestration (Q2)
- ✅ Enterprise-ready platform (Q3)
- ✅ Sustainable OSS project (Q4)
- ✅ 100 deployments, 10 contributors, 5 upstream PRs
- ✅ Business model validated ($20K pilot revenue)

## RACI and Stakeholder Map
- R: Tech Lead (core orchestration), Security Lead (identity/keys), Infra Lead (deployments), PM (scope/priorities).
- A: CTO/Architect for final decisions (ADRs).
- C: Department champions (Marketing/Finance/Eng) for role profiles.
- I: Compliance/IT Ops (audit and SLOs).

## Now/Next/Later Roadmap
- Now (MVP): Identity bridge, Mesh MCP, Privacy Guard, Controller OpenAPI, Profiles/Policy-lite, Audit baseline, Cost-aware routing.
- Next (v1.3): Policy composition/graph, dashboards, Bus adapter interface, SCIM.
- Later (v1.4): Multi-tenant hardening, advanced approvals, compliance packs, analytics, mTLS.

---

## Upstream PR Opportunities — Contributing to Goose Core

**Purpose:** Document innovations built in this project that could benefit the broader Goose ecosystem. These contributions align with the Block Goose Innovation Grant's "previous collaboration" question and demonstrate community value.

### 🎯 Target: 5 Upstreamed PRs by Month 12

---

### 1. Privacy Guard MCP Integration Patterns (Phase 6 - Q2 Month 5)

**What We're Building:**
- MCP tool wrapper that masks PII before sending to LLM
- Generic middleware pattern: any MCP tool → Privacy Guard → LLM
- Configurable modes: Off/Detect/Mask/Strict (user-selectable)
- Deterministic pseudonymization (Vault-backed keys)

**Upstream PR Proposal:**
- **Component:** goose-mcp crate (Rust)
- **Change:** Add optional `privacy_guard` middleware to MCP tool registration
- **Example:**
  ```rust
  server.register_tool(
      "web_search",
      Tool::new(web_search_handler)
          .with_privacy_guard(PrivacyGuardConfig {
              mode: PrivacyMode::Mask,
              vault_url: env::var("VAULT_URL")?,
              pii_patterns: vec![Pattern::Email, Pattern::SSN, Pattern::CreditCard],
          })
  );
  ```
- **Benefit:** Enterprise users get PII protection out-of-the-box
- **Documentation:** Add privacy-first MCP patterns to Goose architecture docs
- **Timeline:** PR draft by Month 5, merged by Month 6
- **Estimated Effort:** 3-5 days (extract, generalize, document, PR review cycle)

---

### 2. OIDC/JWT Middleware for MCP Servers (Phase 5 - Q1 Week 6)

**What We're Building:**
- Keycloak OIDC integration for Controller API
- JWT verification middleware (RS256, JWKS caching, clock skew tolerance)
- Role-based access control (RBAC) using JWT claims

**Upstream PR Proposal:**
- **Component:** goosed (Rust server framework)
- **Change:** Add `oidc_middleware` module for JWT verification
- **Example:**
  ```rust
  let app = Router::new()
      .route("/tasks/route", post(route_task))
      .layer(OidcMiddleware::new(OidcConfig {
          issuer_url: "https://keycloak.example.com/realms/prod",
          jwks_url: "https://keycloak.example.com/realms/prod/protocol/openid-connect/certs",
          audience: "goose-server",
          clock_skew_secs: 60,
      }));
  ```
- **Benefit:** Enterprise deployments get OIDC SSO without custom middleware
- **Documentation:** Add OIDC setup guide to Goose server docs
- **Timeline:** PR draft by Week 6, merged by Month 4
- **Estimated Effort:** 2-3 days (extract, generalize, document, PR review cycle)

---

### 3. Agent Mesh Protocol — Multi-Agent Coordination (Phase 5 - Q1 Week 7)

**What We're Building:**
- Agent Mesh MCP: 4 tools (send_task, request_approval, notify, fetch_status)
- Cross-agent communication protocol (JSON over HTTP)
- Session tracking and approval workflows

**Upstream PR Proposal:**
- **Component:** New MCP extension or core feature
- **Change:** Add `agent_mesh` MCP tools to Goose core
- **Spec:**
  ```yaml
  tools:
    - name: send_task
      description: Route a task to another agent role
      params:
        - target: string (required) - target agent role
        - task: object (required) - task payload
        - context: object (optional) - additional context
    
    - name: request_approval
      description: Request approval from another agent
      params:
        - task_id: string (required) - task identifier
        - approver_role: string (required) - approver's role
        - reason: string (required) - approval reason
    
    - name: notify
      description: Send notification to another agent
      params:
        - target: string (required) - target agent role
        - message: string (required) - notification message
        - priority: string (optional) - low/normal/high
    
    - name: fetch_status
      description: Get task/session status
      params:
        - task_id: string (required) - task identifier
  ```
- **Benefit:** Enable multi-agent workflows without custom infrastructure
- **Documentation:** Add multi-agent orchestration guide to Goose docs
- **Timeline:** PR draft by Week 7, merged by Month 7
- **Estimated Effort:** 5-7 days (extract, generalize, document, community feedback, PR review cycle)

---

### 4. Session Persistence Patterns (Phase 4 - Q1 Week 4)

**What We're Building:**
- Postgres-backed session storage (sessions, tasks, approvals, audit)
- Session lifecycle management (pending → active → completed/failed)
- Idempotency deduplication (Redis-backed, 24h TTL)

**Upstream PR Proposal:**
- **Component:** goosed (server framework)
- **Change:** Add optional `session_persistence` module with pluggable backends
- **Example:**
  ```rust
  // Postgres backend
  let session_store = PostgresSessionStore::new(pool);
  
  // Redis cache (optional)
  let cache = RedisCache::new(redis_pool);
  
  let app = Router::new()
      .route("/sessions", get(list_sessions).post(create_session))
      .with_state(AppState { session_store, cache });
  ```
- **Benefit:** Production deployments get session persistence without custom implementation
- **Documentation:** Add stateful workflows guide to Goose docs
- **Timeline:** PR draft by Week 4, merged by Month 4
- **Estimated Effort:** 4-6 days (extract, generalize, support multiple backends, document, PR review cycle)

---

### 5. Role Profile Specification (Phase 5 - Q1 Week 6)

**What We're Building:**
- YAML/JSON profile spec: role, display_name, description, extensions, recipes, policies, env_vars
- Signed profiles (Vault transit key or HMAC)
- Policy engine: can_use_tool(role, tool_name), can_access_data(role, data_type)

**Upstream PR Proposal:**
- **Component:** Goose configuration + docs
- **Change:** Add `profiles/` directory with role profile spec
- **Spec Format:**
  ```yaml
  # profiles/finance.yaml
  role: finance
  display_name: "Finance Team Agent"
  description: "Budget approvals, reporting, compliance"
  
  extensions:
    - name: github
      allowed_tools: ["list_issues", "create_issue", "add_comment"]
    - name: agent_mesh
      allowed_tools: ["send_task", "request_approval", "notify", "fetch_status"]
  
  policies:
    - allow_tool: "github__list_issues"
      conditions:
        - repo: "finance/*"
    - deny_tool: "developer__shell"
      reason: "No code execution for Finance role"
  
  env_vars:
    PRIVACY_GUARD_MODE: "Strict"
    SESSION_RETENTION_DAYS: "90"  # Compliance requirement
  ```
- **Benefit:** Organizations get role-based agent configuration without custom code
- **Documentation:** Add role profiles guide to Goose docs
- **Timeline:** PR draft by Week 6, merged by Month 8
- **Estimated Effort:** 5-7 days (spec design, validation schema, examples, documentation, PR review cycle)

---

### Grant Application Mapping

**Block Goose Innovation Grant Question:**
> "Have you contributed to the Goose project before? If so, please provide links to previous pull requests, issues, or other contributions."

**Answer (Post-Phase 12):**
> Yes. As part of the org-chart-goose-orchestrator project (funded by Block Goose Innovation Grant #2026-Q1), we have contributed the following to Goose core:
>
> 1. **Privacy Guard MCP Integration** (PR #XXXX) - Merged Month 6
>    - Middleware pattern for PII masking in MCP tools
>    - Enterprise-ready privacy protection
>
> 2. **OIDC/JWT Middleware** (PR #XXXX) - Merged Month 4
>    - Keycloak SSO integration for goosed
>    - Role-based access control
>
> 3. **Agent Mesh Protocol** (PR #XXXX) - Merged Month 7
>    - Multi-agent coordination tools
>    - Cross-agent communication protocol
>
> 4. **Session Persistence Patterns** (PR #XXXX) - Merged Month 4
>    - Postgres-backed session storage
>    - Pluggable backend architecture
>
> 5. **Role Profile Specification** (PR #XXXX) - Merged Month 8
>    - YAML/JSON profile format
>    - Policy engine for tool allowlists
>
> All PRs include comprehensive documentation, tests, and examples. Total lines contributed: ~2,500 (code) + ~5,000 (docs).
>
> GitHub repository: https://github.com/JEFH507/org-chart-goose-orchestrator

---

### Success Metrics (12 Months)

- ✅ 5 PRs merged to Goose core (target: all 5 by Month 8)
- ✅ 2,500+ lines of code contributed
- ✅ 5,000+ lines of documentation contributed
- ✅ 3 conference talks/blog posts referencing contributions (FOSDEM, KubeCon, AI Engineer Summit)
- ✅ External contributors adopting patterns (tracked via GitHub stars, forks, issues)
- ✅ Goose maintainers recognize project as "exemplar" for enterprise deployments

---

### Community Engagement Strategy

**Months 4-6 (Q2): Early Upstream Work**
- Draft PRs for Privacy Guard + OIDC middleware
- Engage with Goose maintainers on design review
- Iterate based on feedback
- Merge 2 PRs (Privacy Guard, OIDC)

**Months 7-9 (Q3): Core Contributions**
- Submit Agent Mesh protocol PR (most impactful)
- Submit Session Persistence PR
- Merge 2 PRs (Agent Mesh, Session Persistence)

**Months 10-12 (Q4): Documentation + Community**
- Submit Role Profile Specification PR
- Write blog posts on each contribution
- Present at conferences (FOSDEM, KubeCon)
- Engage with external contributors (GitHub Discussions)
- Merge 1 PR (Role Profiles)

---

**Total Upstream Contribution Effort:** ~25-35 days (across 12 months)

**Timeline:**
- Month 4: OIDC middleware PR merged
- Month 5: Privacy Guard PR drafted
- Month 6: Privacy Guard PR merged
- Month 7: Agent Mesh PR merged + Session Persistence PR merged
- Month 8: Role Profiles PR merged
- Months 9-12: Blog posts, talks, community engagement
