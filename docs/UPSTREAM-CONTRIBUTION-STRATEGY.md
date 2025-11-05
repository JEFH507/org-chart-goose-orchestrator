# Upstream Contribution Strategy — Goose Core PRs

**Document Version:** 1.0  
**Last Updated:** 2025-11-05  
**Owner:** Engineering Team  
**Purpose:** Document innovations from this project that will be contributed upstream to Goose core

---

## Executive Summary

As part of the **Block Goose Innovation Grant** ($100K/12 months), this project will contribute **5 major PRs** to Goose core, representing patterns and features that benefit the broader Goose ecosystem.

**Target:** All 5 PRs merged by Month 8 (Q3)  
**Estimated Effort:** 25-35 days total (across 12 months)  
**Grant Application Value:** Demonstrates "previous collaboration" with Goose project

---

## Table of Contents

1. [Privacy Guard MCP Integration](#1-privacy-guard-mcp-integration)
2. [OIDC/JWT Middleware for MCP Servers](#2-oidcjwt-middleware-for-mcp-servers)
3. [Agent Mesh Protocol](#3-agent-mesh-protocol)
4. [Session Persistence Patterns](#4-session-persistence-patterns)
5. [Role Profile Specification](#5-role-profile-specification)
6. [Grant Application Mapping](#grant-application-mapping)
7. [Timeline and Success Metrics](#timeline-and-success-metrics)

---

## 1. Privacy Guard MCP Integration

### What We're Building (Phase 6 - Q2 Month 5)

**Component:** Privacy Guard service with MCP integration  
**Scope:** Mask PII in MCP tool inputs/outputs before sending to LLM

**Features:**
- MCP tool wrapper that intercepts tool calls
- Configurable modes: Off/Detect/Mask/Strict (user-selectable)
- Deterministic pseudonymization using Vault-backed keys
- Generic middleware pattern: any MCP tool → Privacy Guard → LLM
- Ollama NER model for accurate PII detection (vs regex fallback)
- Performance: P50 < 500ms for mask-and-forward operations

**Implementation:**
- Location: `src/privacy-guard/` (Rust/Axum service)
- MCP Integration: `src/agent-mesh/tools/` (Python MCP server)
- Pattern: Intercept tool call → mask JSON → call LLM → unmask response
- Storage: Vault KV v2 for deterministic mapping keys

---

### Upstream PR Proposal

**Target:** goose-mcp crate (Rust)  
**Change:** Add optional `privacy_guard` middleware to MCP tool registration

**API Design:**
```rust
use goose_mcp::{Server, Tool, PrivacyGuardConfig, PrivacyMode, PiiPattern};

let server = Server::new("my-extension");

server.register_tool(
    "web_search",
    Tool::new(web_search_handler)
        .with_privacy_guard(PrivacyGuardConfig {
            mode: PrivacyMode::Mask,
            guard_url: env::var("PRIVACY_GUARD_URL")?,
            pii_patterns: vec![
                PiiPattern::Email,
                PiiPattern::SSN,
                PiiPattern::CreditCard,
                PiiPattern::PhoneNumber,
            ],
            vault_config: Some(VaultConfig {
                url: env::var("VAULT_URL")?,
                token: env::var("VAULT_TOKEN")?,
                mount_path: "kv/privacy",
            }),
        })
);
```

**Workflow:**
1. User calls `web_search` tool with query containing PII
2. Privacy Guard middleware intercepts call
3. Masks PII: "Email john.doe@example.com" → "Email <EMAIL_1234>"
4. Calls LLM with masked query
5. Unmasks response before returning to user
6. Stores deterministic mapping in Vault (EMAIL_1234 → john.doe@example.com)

**Configuration:**
```yaml
# ~/.config/goose/profiles.yaml
extensions:
  web_search:
    type: mcp
    command: ["web-search-server"]
    privacy_guard:
      mode: "Mask"  # Off/Detect/Mask/Strict
      guard_url: "http://localhost:8089"
      vault_url: "http://localhost:8200"
      pii_patterns:
        - email
        - ssn
        - credit_card
        - phone_number
```

---

### Benefits to Goose Ecosystem

**For Enterprise Users:**
- ✅ PII protection out-of-the-box (no custom code)
- ✅ Compliance-ready (GDPR, CPRA, HIPAA)
- ✅ Deterministic pseudonymization (audit trail preserved)
- ✅ Configurable strictness (balance privacy vs usability)

**For MCP Tool Developers:**
- ✅ Generic middleware pattern (works with any tool)
- ✅ Minimal code changes (add `.with_privacy_guard()` to registration)
- ✅ Automatic PII detection (no manual regex patterns)

**For Goose Maintainers:**
- ✅ Differentiated enterprise feature
- ✅ Addresses #1 security concern (PII leakage to cloud LLMs)
- ✅ Proven production implementation (our project validates it)

---

### Documentation to Include

**New Files:**
- `docs/privacy-guard/OVERVIEW.md` - Architecture and use cases
- `docs/privacy-guard/QUICKSTART.md` - 5-minute setup guide
- `docs/privacy-guard/API.md` - Privacy Guard API reference
- `docs/privacy-guard/PATTERNS.md` - Common PII patterns and custom detection

**Updated Files:**
- `docs/goose-v1.12.00-technical-architecture-report.md` - Add Privacy Guard section
- `README.md` - Add "Enterprise Privacy" feature highlight
- `crates/goose-mcp/src/lib.rs` - Add PrivacyGuardConfig docs

**Examples:**
- `examples/privacy-guard-web-search/` - Web search with PII masking
- `examples/privacy-guard-github/` - GitHub tool with email masking
- `examples/privacy-guard-custom-patterns/` - Custom PII detection rules

---

### PR Timeline

**Month 5 (Q2):**
- Week 1-2: Extract Privacy Guard from project-specific code
- Week 3: Generalize middleware pattern for any MCP tool
- Week 4: Write documentation and examples

**Month 6 (Q2):**
- Week 1: Submit PR to goose-mcp crate
- Week 2-3: Iterate based on maintainer feedback
- Week 4: PR merged

**Estimated Effort:** 3-5 days (extraction, generalization, documentation, PR review cycle)

---

## 2. OIDC/JWT Middleware for MCP Servers

### What We're Building (Phase 5 - Q1 Week 6)

**Component:** Keycloak OIDC integration for Controller API  
**Scope:** JWT verification middleware for Axum-based MCP servers

**Features:**
- RS256 signature validation using JWKS (JSON Web Key Set)
- JWKS caching with configurable TTL (60 minutes default)
- Clock skew tolerance (±60 seconds default)
- Role-based access control (RBAC) using JWT claims
- Audience validation (prevent token reuse across services)
- Graceful degradation when OIDC disabled (dev mode)

**Implementation:**
- Location: `src/controller/src/auth.rs` (Rust/Axum middleware)
- Dependencies: `jsonwebtoken`, `reqwest` (JWKS fetching), `serde_json`
- Pattern: Axum middleware layer that extracts/validates JWT from Authorization header
- Claims: Extract `sub` (user ID), `preferred_username`, `realm_access.roles`

---

### Upstream PR Proposal

**Target:** goosed (Rust server framework)  
**Change:** Add `oidc_middleware` module for JWT verification

**API Design:**
```rust
use goose_server::{Router, OidcMiddleware, OidcConfig};
use axum::routing::{get, post};

let app = Router::new()
    .route("/tasks/route", post(route_task))
    .route("/sessions", get(list_sessions).post(create_session))
    .layer(OidcMiddleware::new(OidcConfig {
        issuer_url: "https://keycloak.example.com/realms/prod",
        jwks_url: "https://keycloak.example.com/realms/prod/protocol/openid-connect/certs",
        audience: "goose-server",
        clock_skew_secs: 60,
        jwks_cache_ttl_secs: 3600,
        required_roles: vec!["goose-user".to_string()],
    }));
```

**Workflow:**
1. Client sends request with `Authorization: Bearer <JWT>`
2. Middleware extracts JWT from header
3. Fetches JWKS from Keycloak (cached for 1 hour)
4. Validates signature (RS256), expiry, audience, issuer
5. Extracts claims (user ID, roles)
6. Injects claims into request extensions (available to handlers)
7. Returns 401 Unauthorized if validation fails

**Configuration:**
```yaml
# ~/.config/goose/server.yaml
oidc:
  enabled: true
  issuer_url: "https://keycloak.example.com/realms/prod"
  jwks_url: "https://keycloak.example.com/realms/prod/protocol/openid-connect/certs"
  audience: "goose-server"
  clock_skew_secs: 60
  jwks_cache_ttl_secs: 3600
  required_roles:
    - "goose-user"
```

---

### Benefits to Goose Ecosystem

**For Enterprise Users:**
- ✅ OIDC SSO out-of-the-box (Keycloak, Auth0, Okta, Azure AD)
- ✅ No custom authentication code required
- ✅ Role-based access control (RBAC)
- ✅ Audit-ready (JWT claims include user identity)

**For MCP Server Developers:**
- ✅ Generic middleware layer (works with any Axum route)
- ✅ Minimal code changes (add `.layer(OidcMiddleware::new(...))`)
- ✅ Claims available via request extensions (easy access in handlers)

**For Goose Maintainers:**
- ✅ Enterprise authentication standard (OIDC/OAuth2)
- ✅ Reduces security concerns (no custom auth logic)
- ✅ Proven production implementation (our project validates it)

---

### Documentation to Include

**New Files:**
- `docs/auth/OIDC-SETUP.md` - Keycloak, Auth0, Okta, Azure AD setup guides
- `docs/auth/JWT-CLAIMS.md` - Claims extraction and usage in handlers
- `docs/auth/RBAC.md` - Role-based access control patterns

**Updated Files:**
- `docs/goose-v1.12.00-technical-architecture-report.md` - Add OIDC auth section
- `README.md` - Add "Enterprise SSO" feature highlight
- `crates/goose-server/src/lib.rs` - Add OidcMiddleware docs

**Examples:**
- `examples/oidc-keycloak/` - Keycloak OIDC setup with Docker Compose
- `examples/oidc-auth0/` - Auth0 OIDC setup
- `examples/oidc-rbac/` - Role-based route protection

---

### PR Timeline

**Week 6 (Q1):**
- Days 1-2: Extract OIDC middleware from Controller code
- Days 3-4: Generalize for any Axum-based server
- Day 5: Write documentation and examples

**Month 4 (Q2):**
- Week 1: Submit PR to goosed crate
- Week 2-3: Iterate based on maintainer feedback
- Week 4: PR merged

**Estimated Effort:** 2-3 days (extraction, generalization, documentation, PR review cycle)

---

## 3. Agent Mesh Protocol

### What We're Building (Phase 5 - Q1 Week 7)

**Component:** Agent Mesh MCP extension (Python)  
**Scope:** Multi-agent coordination protocol with 4 tools

**Features:**
- **send_task**: Route task to another agent role (Finance → Manager)
- **request_approval**: Request approval from another agent (Budget > $10K → Manager approval)
- **notify**: Send notification to another agent (Engineering → Manager: "Deploy complete")
- **fetch_status**: Get task/session status (Is my approval request approved?)

**Implementation:**
- Location: `src/agent-mesh/` (Python MCP server)
- Tools: 4 tools, 977 lines of code
- Transport: HTTP/JSON (calls Controller API)
- Session tracking: Postgres-backed (Phase 4)
- Workflow: Agent A → Controller API → Agent B

**Protocol:**
```json
// send_task request
POST /tasks/route
{
  "target": "manager",
  "task": {
    "task_type": "budget_approval",
    "description": "Approve Q1 hiring budget ($50K)",
    "data": {"amount": 50000, "department": "Engineering"}
  },
  "context": {"quarter": "Q1-2026"}
}

// Response
HTTP 202 Accepted
{
  "task_id": "task-abc123",
  "status": "routed",
  "trace_id": "trace-xyz789"
}
```

---

### Upstream PR Proposal

**Target:** New MCP extension or core feature  
**Change:** Add `agent_mesh` MCP tools to Goose core (Python or Rust)

**Tool Specifications:**

#### Tool 1: send_task
```yaml
name: send_task
description: Route a task to another agent role
params:
  - name: target
    type: string
    required: true
    description: Target agent role (e.g., 'manager', 'finance')
  - name: task
    type: object
    required: true
    description: Task payload (task_type, description, data)
    properties:
      task_type:
        type: string
        description: Type of task (e.g., 'approval', 'notification')
      description:
        type: string
        description: Human-readable task description
      data:
        type: object
        description: Task-specific data
  - name: context
    type: object
    required: false
    description: Additional context (optional)
returns:
  type: object
  properties:
    task_id:
      type: string
      description: Unique task identifier (UUID)
    status:
      type: string
      description: Task status ('routed', 'pending', 'active')
    trace_id:
      type: string
      description: Distributed tracing ID
```

#### Tool 2: request_approval
```yaml
name: request_approval
description: Request approval for a task from another agent
params:
  - name: task_id
    type: string
    required: true
    description: Task identifier (from send_task)
  - name: approver_role
    type: string
    required: true
    description: Role of approver (e.g., 'manager', 'director')
  - name: reason
    type: string
    required: true
    description: Reason for approval request
  - name: decision
    type: string
    required: false
    default: "pending"
    description: Decision (pending/approved/rejected)
  - name: comments
    type: string
    required: false
    description: Additional comments
returns:
  type: object
  properties:
    approval_id:
      type: string
      description: Unique approval identifier (UUID)
    status:
      type: string
      description: Approval status ('pending', 'approved', 'rejected')
```

#### Tool 3: notify
```yaml
name: notify
description: Send notification to another agent
params:
  - name: target
    type: string
    required: true
    description: Target agent role
  - name: message
    type: string
    required: true
    description: Notification message
  - name: priority
    type: string
    required: false
    default: "normal"
    enum: ["low", "normal", "high"]
    description: Notification priority
returns:
  type: object
  properties:
    task_id:
      type: string
      description: Notification task ID
    status:
      type: string
      description: Delivery status ('routed', 'delivered')
```

#### Tool 4: fetch_status
```yaml
name: fetch_status
description: Get task/session status
params:
  - name: task_id
    type: string
    required: true
    description: Task identifier
returns:
  type: object
  properties:
    task_id:
      type: string
      description: Task identifier
    status:
      type: string
      description: Task status ('pending', 'active', 'completed', 'failed')
    assigned_agent:
      type: string
      description: Agent currently working on task
    created_at:
      type: string
      format: iso8601
      description: Task creation timestamp
    updated_at:
      type: string
      format: iso8601
      description: Last update timestamp
    result:
      type: object
      description: Task result (if completed)
```

---

### Benefits to Goose Ecosystem

**For Multi-Agent Workflows:**
- ✅ Enable agent-to-agent communication without custom infrastructure
- ✅ Standardized protocol (all agents use same MCP tools)
- ✅ Session tracking (audit trail of cross-agent workflows)
- ✅ Approval workflows (enforce business logic)

**For Enterprise Use Cases:**
- ✅ Budget approvals (Finance → Manager → Director)
- ✅ PR reviews (Engineering → Tech Lead → Architect)
- ✅ Compliance checks (Legal → Compliance → Executive)
- ✅ Escalations (Support → Engineering → Manager)

**For Goose Maintainers:**
- ✅ Differentiated enterprise feature (multi-agent orchestration)
- ✅ Proven protocol (our project validates it in production)
- ✅ Extensible design (custom task types, workflows)

---

### Documentation to Include

**New Files:**
- `docs/agent-mesh/OVERVIEW.md` - Multi-agent orchestration architecture
- `docs/agent-mesh/PROTOCOL.md` - Agent Mesh protocol specification
- `docs/agent-mesh/WORKFLOWS.md` - Common multi-agent workflow patterns
- `docs/agent-mesh/SECURITY.md` - Cross-agent security considerations

**Updated Files:**
- `docs/goose-v1.12.00-technical-architecture-report.md` - Add Agent Mesh section
- `README.md` - Add "Multi-Agent Orchestration" feature highlight

**Examples:**
- `examples/agent-mesh-approval-workflow/` - Budget approval (Finance → Manager)
- `examples/agent-mesh-escalation/` - Support escalation (Support → Engineering)
- `examples/agent-mesh-notification/` - Broadcast notifications

---

### PR Timeline

**Week 7 (Q1):**
- Days 1-3: Extract Agent Mesh from project-specific code
- Days 4-5: Generalize protocol specification
- Days 6-7: Write documentation and examples

**Month 7 (Q3):**
- Week 1-2: Submit PR to Goose core (new MCP extension)
- Week 3-4: Iterate based on community feedback
- Week 5: PR merged

**Estimated Effort:** 5-7 days (extraction, generalization, documentation, community engagement, PR review cycle)

---

## 4. Session Persistence Patterns

### What We're Building (Phase 4 - Q1 Week 4)

**Component:** Postgres-backed session storage  
**Scope:** Stateful session management for Goose servers

**Features:**
- Session CRUD operations (Create, Read, Update, Delete)
- Session lifecycle management (pending → active → completed/failed)
- Idempotency deduplication (Redis-backed, 24h TTL)
- Retention policies (7 days default, configurable)
- Audit trail (all session state changes logged)

**Implementation:**
- Location: `src/controller/src/repository/session_repo.rs` (Rust/sqlx)
- Database: Postgres 15+ with sqlx migrations
- Schema:
  - sessions table (id, role, task_id, status, created_at, updated_at, metadata)
  - tasks table (id, task_type, from_role, to_role, data, trace_id, idempotency_key)
  - approvals table (id, task_id, approver_role, status, decision_at, notes)
  - audit_events table (id, event_type, role, timestamp, trace_id, metadata)

**Idempotency Pattern:**
```rust
// First request
POST /tasks/route
Headers:
  Idempotency-Key: abc123-uuid
  
Response: HTTP 202 Accepted
  {"task_id": "task-xyz"}
  
Redis cache: SET idempotency:abc123-uuid {"task_id":"task-xyz"} EX 86400

// Duplicate request (same Idempotency-Key)
POST /tasks/route
Headers:
  Idempotency-Key: abc123-uuid
  
Response: HTTP 200 OK (cached)
  {"task_id": "task-xyz"}  // Same response, no duplicate processing
```

---

### Upstream PR Proposal

**Target:** goosed (server framework)  
**Change:** Add optional `session_persistence` module with pluggable backends

**API Design:**
```rust
use goose_server::{Router, SessionStore, PostgresSessionStore, RedisCache};
use sqlx::PgPool;

// Postgres backend
let pg_pool = PgPool::connect(&database_url).await?;
let session_store = PostgresSessionStore::new(pg_pool);

// Redis cache (optional, for idempotency)
let redis_pool = redis::Client::open(redis_url)?;
let cache = RedisCache::new(redis_pool);

let app = Router::new()
    .route("/sessions", get(list_sessions).post(create_session))
    .route("/sessions/:id", get(get_session).put(update_session))
    .with_state(AppState {
        session_store,
        cache,
    });
```

**Pluggable Backend Trait:**
```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: CreateSessionRequest) -> Result<Session>;
    async fn get(&self, id: Uuid) -> Result<Option<Session>>;
    async fn update(&self, id: Uuid, update: UpdateSessionRequest) -> Result<Option<Session>>;
    async fn list(&self, filter: SessionFilter) -> Result<Vec<Session>>;
    async fn delete(&self, id: Uuid) -> Result<bool>;
}

// Implementations:
// - PostgresSessionStore (default)
// - InMemorySessionStore (for testing)
// - RedisSessionStore (for distributed deployments)
// - DynamoDBSessionStore (for AWS deployments)
```

**Migration Support:**
```bash
# sqlx migrations
sqlx migrate add create_sessions_table
sqlx migrate run --database-url $DATABASE_URL

# Diesel migrations (alternative)
diesel migration generate create_sessions
diesel migration run
```

---

### Benefits to Goose Ecosystem

**For Production Deployments:**
- ✅ Session persistence out-of-the-box (no custom database code)
- ✅ Pluggable backends (Postgres, Redis, DynamoDB, etc.)
- ✅ Idempotency deduplication (prevent duplicate request processing)
- ✅ Audit trail (all state changes logged)

**For Developers:**
- ✅ Minimal code changes (add `.with_state(AppState { session_store, ... })`)
- ✅ Automatic schema migrations (sqlx or Diesel)
- ✅ Type-safe queries (compile-time validation with sqlx)

**For Goose Maintainers:**
- ✅ Production-ready pattern (our project validates it)
- ✅ Extensible design (support multiple backends)
- ✅ Enterprise feature (stateful workflows, audit compliance)

---

### Documentation to Include

**New Files:**
- `docs/session-persistence/OVERVIEW.md` - Session persistence architecture
- `docs/session-persistence/BACKENDS.md` - Available backends (Postgres, Redis, DynamoDB)
- `docs/session-persistence/MIGRATIONS.md` - Database schema setup and migrations
- `docs/session-persistence/IDEMPOTENCY.md` - Idempotency deduplication patterns

**Updated Files:**
- `docs/goose-v1.12.00-technical-architecture-report.md` - Add session persistence section
- `README.md` - Add "Stateful Workflows" feature highlight

**Examples:**
- `examples/session-persistence-postgres/` - Postgres backend setup
- `examples/session-persistence-redis/` - Redis backend setup
- `examples/session-persistence-idempotency/` - Idempotency testing

---

### PR Timeline

**Week 4 (Q1):**
- Days 1-2: Extract session persistence from Controller code
- Days 3-4: Generalize backend trait + implement Postgres/Redis backends
- Day 5: Write documentation and examples

**Month 4 (Q2):**
- Week 1: Submit PR to goosed crate
- Week 2-3: Iterate based on maintainer feedback
- Week 4: PR merged

**Estimated Effort:** 4-6 days (extraction, generalization, multiple backend support, documentation, PR review cycle)

---

## 5. Role Profile Specification

### What We're Building (Phase 5 - Q1 Week 6)

**Component:** Role profile system for agent configuration  
**Scope:** YAML/JSON spec for role-based extension allowlists and policies

**Features:**
- Profile format: YAML/JSON with role, display_name, description, extensions, recipes, policies, env_vars
- Signed profiles (Vault transit key or HMAC)
- Policy engine: can_use_tool(role, tool_name), can_access_data(role, data_type)
- Extension allowlists per role (Finance: no code execution, Manager: no PII access)
- Deny-by-default security model (explicit allow rules required)

**Implementation:**
- Location: `src/controller/src/directory/` (Rust profile validation + storage)
- Profile storage: Postgres or Vault KV v2
- Policy evaluation: RBAC/ABAC-lite engine
- Client integration: Goose client downloads profile, applies extension allowlists

**Profile Format:**
```yaml
# profiles/finance.yaml
role: finance
display_name: "Finance Team Agent"
description: "Budget approvals, reporting, compliance"

extensions:
  - name: github
    allowed_tools:
      - "list_issues"
      - "create_issue"
      - "add_comment"
    deny_tools:
      - "push_files"  # No code changes
  
  - name: agent_mesh
    allowed_tools:
      - "send_task"
      - "request_approval"
      - "notify"
      - "fetch_status"
  
  - name: developer
    deny_all: true  # No shell access for Finance role

policies:
  - allow_tool: "github__list_issues"
    conditions:
      - repo_pattern: "finance/*"  # Only finance repos
  
  - deny_tool: "developer__shell"
    reason: "No code execution for Finance role"
  
  - allow_data: "financial_reports"
    conditions:
      - classification: "public" or "internal"  # No confidential data

env_vars:
  PRIVACY_GUARD_MODE: "Strict"
  SESSION_RETENTION_DAYS: "90"  # Compliance requirement (SOX/GDPR)
  
signature:
  algorithm: "HMAC-SHA256"
  key_id: "vault:transit/goose/profile-signing"
  signature: "abc123..."  # Vault-generated signature
```

---

### Upstream PR Proposal

**Target:** Goose configuration + docs  
**Change:** Add `profiles/` directory with role profile specification

**Profile Validation:**
```rust
use goose_profiles::{Profile, ProfileValidator, PolicyEngine};

// Load and validate profile
let profile_yaml = std::fs::read_to_string("profiles/finance.yaml")?;
let profile: Profile = serde_yaml::from_str(&profile_yaml)?;

// Validate signature
let validator = ProfileValidator::new(vault_client);
validator.verify_signature(&profile)?;

// Evaluate policies
let policy_engine = PolicyEngine::new();
let can_use = policy_engine.can_use_tool(&profile, "github__list_issues", context)?;
let can_access = policy_engine.can_access_data(&profile, "financial_reports", classification)?;
```

**Client Integration:**
```yaml
# ~/.config/goose/config.yaml
profile:
  source: "https://controller.example.com/profiles/finance.yaml"
  auto_update: true
  update_interval_mins: 60
  enforce: true  # Block tools not in allowlist
```

**Policy Engine:**
```rust
pub struct PolicyEngine {
    // Deny-by-default: explicit allow required
}

impl PolicyEngine {
    pub fn can_use_tool(
        &self,
        profile: &Profile,
        tool_name: &str,
        context: &Context,
    ) -> Result<bool> {
        // Check allowlist
        // Evaluate conditions
        // Apply deny rules
    }
    
    pub fn can_access_data(
        &self,
        profile: &Profile,
        data_type: &str,
        classification: &str,
    ) -> Result<bool> {
        // Check data policies
        // Evaluate classification
    }
}
```

---

### Benefits to Goose Ecosystem

**For Organizations:**
- ✅ Role-based agent configuration (Finance, Manager, Engineering, etc.)
- ✅ Centralized policy management (admin pushes updates)
- ✅ Audit-ready (all policy decisions logged)
- ✅ Compliance-ready (enforce least privilege)

**For Security Teams:**
- ✅ Deny-by-default security model
- ✅ Extension allowlists (prevent unauthorized tool use)
- ✅ Data access controls (prevent PII leakage)
- ✅ Signed profiles (prevent tampering)

**For Goose Maintainers:**
- ✅ Enterprise governance feature (multi-user deployments)
- ✅ Proven specification (our project validates it)
- ✅ Extensible design (custom policies, workflows)

---

### Documentation to Include

**New Files:**
- `docs/profiles/SPECIFICATION.md` - Profile format specification
- `docs/profiles/POLICY-ENGINE.md` - Policy evaluation engine
- `docs/profiles/EXAMPLES.md` - 5 role profiles (Finance, Manager, Engineering, Marketing, Support)
- `docs/profiles/SIGNING.md` - Profile signing with Vault

**Updated Files:**
- `docs/goose-v1.12.00-technical-architecture-report.md` - Add role profiles section
- `README.md` - Add "Role-Based Configuration" feature highlight

**Examples:**
- `examples/profiles/finance.yaml` - Finance role profile
- `examples/profiles/manager.yaml` - Manager role profile
- `examples/profiles/engineering.yaml` - Engineering role profile
- `examples/profiles/marketing.yaml` - Marketing role profile
- `examples/profiles/support.yaml` - Support role profile

---

### PR Timeline

**Week 6 (Q1):**
- Days 1-3: Design profile specification (YAML schema, validation, signing)
- Days 4-5: Implement policy engine (can_use_tool, can_access_data)
- Days 6-7: Write documentation and 5 example profiles

**Month 8 (Q3):**
- Week 1-2: Submit PR to Goose core (new `profiles/` directory + docs)
- Week 3-4: Iterate based on community feedback (refine spec)
- Week 5: PR merged

**Estimated Effort:** 5-7 days (spec design, policy engine, validation schema, examples, documentation, PR review cycle)

---

## Grant Application Mapping

### Block Goose Innovation Grant Question

> "Have you contributed to the Goose project before? If so, please provide links to previous pull requests, issues, or other contributions."

### Answer (Post-Phase 12)

> Yes. As part of the **org-chart-goose-orchestrator** project (funded by Block Goose Innovation Grant #2026-Q1), we have contributed the following to Goose core:
>
> ### 1. Privacy Guard MCP Integration (PR #XXXX) - Merged Month 6
> - Middleware pattern for PII masking in MCP tools
> - Enterprise-ready privacy protection (GDPR, CPRA, HIPAA compliant)
> - Configurable modes: Off/Detect/Mask/Strict
> - Deterministic pseudonymization using Vault
> - **Impact:** Enterprise users get PII protection out-of-the-box
>
> ### 2. OIDC/JWT Middleware (PR #XXXX) - Merged Month 4
> - Keycloak SSO integration for goosed servers
> - RS256 signature validation, JWKS caching
> - Role-based access control (RBAC)
> - **Impact:** Enterprise deployments get OIDC SSO without custom code
>
> ### 3. Agent Mesh Protocol (PR #XXXX) - Merged Month 7
> - 4 MCP tools: send_task, request_approval, notify, fetch_status
> - Multi-agent coordination protocol (JSON over HTTP)
> - Session tracking and approval workflows
> - **Impact:** Enable multi-agent orchestration without custom infrastructure
>
> ### 4. Session Persistence Patterns (PR #XXXX) - Merged Month 4
> - Postgres-backed session storage with pluggable backends
> - Session lifecycle management (pending → active → completed/failed)
> - Idempotency deduplication (Redis-backed, 24h TTL)
> - **Impact:** Production deployments get stateful workflows
>
> ### 5. Role Profile Specification (PR #XXXX) - Merged Month 8
> - YAML/JSON profile format (role, extensions, policies, env_vars)
> - Policy engine: can_use_tool, can_access_data
> - Signed profiles (Vault transit key)
> - **Impact:** Organizations get role-based agent configuration
>
> ### Contribution Statistics
> - **Lines of Code:** ~2,500 (Rust + Python)
> - **Documentation:** ~5,000 lines (architecture docs, API references, examples)
> - **Examples:** 15 examples (Privacy Guard, OIDC, Agent Mesh, profiles)
> - **Tests:** ~1,200 lines (unit tests, integration tests)
>
> ### Community Engagement
> - 3 conference talks (FOSDEM, KubeCon, AI Engineer Summit)
> - 5 blog posts on each contribution
> - GitHub Discussions active (answering community questions)
> - External contributors adopting patterns (tracked via stars, forks)
>
> **GitHub Repository:** https://github.com/JEFH507/org-chart-goose-orchestrator

---

## Timeline and Success Metrics

### 12-Month Contribution Timeline

**Months 1-3 (Q1): Build Features**
- Week 3-4: Build Postgres session persistence (Phase 4)
- Week 5-6: Build OIDC middleware + role profiles (Phase 5)
- Week 7: Prepare grant application

**Months 4-6 (Q2): Early Upstream Work**
- Month 4: OIDC middleware PR merged ✅
- Month 4: Session Persistence PR merged ✅
- Month 5: Privacy Guard PR drafted
- Month 6: Privacy Guard PR merged ✅

**Months 7-9 (Q3): Core Contributions**
- Month 7: Agent Mesh protocol PR merged ✅
- Month 8: Role Profile Specification PR merged ✅

**Months 10-12 (Q4): Documentation + Community**
- Blog posts on each contribution (5 posts)
- Conference talks (FOSDEM, KubeCon, AI Engineer Summit)
- Engage with external contributors (GitHub Discussions)

---

### Success Metrics

#### Code Contributions
- ✅ **5 PRs merged** to Goose core (target: all 5 by Month 8)
- ✅ **2,500+ lines of code** contributed (Rust + Python)
- ✅ **5,000+ lines of documentation** contributed
- ✅ **15 examples** created (Privacy Guard, OIDC, Agent Mesh, profiles)
- ✅ **1,200+ lines of tests** contributed

#### Community Engagement
- ✅ **3 conference talks** (FOSDEM, KubeCon, AI Engineer Summit)
- ✅ **5 blog posts** referencing contributions
- ✅ **External contributors** adopting patterns (tracked via GitHub stars, forks, issues)
- ✅ **Goose maintainers** recognize project as "exemplar" for enterprise deployments

#### Business Impact
- ✅ **100 production deployments** (tracked via opt-in telemetry)
- ✅ **10 external contributors** (non-grant-funded PRs merged)
- ✅ **2 paid pilots** ($10K each — business validation)
- ✅ **Business model validated** (renewal contracts signed)

---

### Effort Breakdown

| PR | Phase | Estimated Days | Actual Days | Status |
|----|-------|----------------|-------------|--------|
| **1. Privacy Guard** | 6 (Q2 Month 5) | 3-5 days | TBD | Not Started |
| **2. OIDC Middleware** | 5 (Q1 Week 6) | 2-3 days | TBD | Not Started |
| **3. Agent Mesh** | 5 (Q1 Week 7) | 5-7 days | TBD | Not Started |
| **4. Session Persistence** | 4 (Q1 Week 4) | 4-6 days | TBD | Not Started |
| **5. Role Profiles** | 5 (Q1 Week 6) | 5-7 days | TBD | Not Started |
| **Blog Posts (5)** | Q4 Months 10-12 | 5 days | TBD | Not Started |
| **Conference Talks (3)** | Q4 Months 10-12 | 3 days | TBD | Not Started |
| **Community Engagement** | Q2-Q4 | 5 days | TBD | Not Started |
| **Total** | 12 months | **32-43 days** | TBD | - |

---

## Next Steps

### Immediate Actions (Phase 4-5)
1. **Build features** (Weeks 3-7) - Implement Postgres session persistence, OIDC middleware, role profiles
2. **Test in production** (Weeks 3-7) - Validate patterns with real workloads
3. **Document patterns** (Weeks 3-7) - Write architecture docs, API references, examples

### Q2 Actions (Months 4-6)
1. **Extract code** (Month 4) - Generalize OIDC middleware + session persistence
2. **Submit PRs** (Month 4) - Submit OIDC + Session Persistence PRs to Goose core
3. **Draft Privacy Guard PR** (Month 5) - Extract Privacy Guard patterns
4. **Merge PRs** (Month 6) - Privacy Guard PR merged

### Q3 Actions (Months 7-9)
1. **Submit Agent Mesh PR** (Month 7) - Most impactful contribution
2. **Submit Role Profiles PR** (Month 8) - Governance feature
3. **Merge PRs** (Months 7-8) - Agent Mesh + Role Profiles merged

### Q4 Actions (Months 10-12)
1. **Write blog posts** (Months 10-12) - 5 posts on each contribution
2. **Present at conferences** (Months 10-12) - FOSDEM, KubeCon, AI Engineer Summit
3. **Engage community** (Months 10-12) - GitHub Discussions, external contributors

---

**Document Owner:** Engineering Team  
**Last Updated:** 2025-11-05  
**Next Review:** 2025-12 (after Phase 5 complete)
