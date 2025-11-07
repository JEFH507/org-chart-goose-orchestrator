# Profile System Specification

**Version**: 1.0.0  
**Last Updated**: 2025-11-07  
**Status**: Production (Phase 5 MVP)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Schema Specification](#schema-specification)
4. [Deserialization Deep Dive](#deserialization-deep-dive)
5. [Validation Rules](#validation-rules)
6. [Database Storage](#database-storage)
7. [Profile Examples](#profile-examples)
8. [Troubleshooting](#troubleshooting)
9. [Testing Patterns](#testing-patterns)

---

## 1. Overview

### Purpose

The Profile System provides **role-based configuration management** for AI agents in the org-chart-goose-orchestrator. Each profile encapsulates all settings needed to run an agent for a specific organizational role (finance, legal, HR, etc.), ensuring consistent behavior and enforcing security policies.

### Key Features

- **Multi-Model Support**: Primary, planner, and worker model configurations
- **Extension Management**: MCP extension allowlists with per-tool permissions
- **Privacy Protection**: PII detection, redaction rules, and local-only enforcement
- **Policy Enforcement**: RBAC/ABAC rules for tool and data access
- **Workflow Automation**: Recipe scheduling and automated tasks
- **Cryptographic Signing**: Vault-backed HMAC signatures for integrity

### Components

1. **Schema** (`src/profile/schema.rs`): Rust types for profile structure
2. **Validator** (`src/profile/validator.rs`): Cross-field validation logic
3. **Database** (`src/db/profiles.rs`): JSONB storage and queries
4. **YAML Files** (`profiles/*.yaml`): Human-editable profile definitions
5. **Admin API** (`src/controller/routes/admin/profiles.rs`): CRUD + signing endpoints

---

## 2. Architecture

### Data Flow

```
Admin creates profile (YAML)
  → Controller validates (ProfileValidator)
    → Postgres stores (JSONB)
      → Admin publishes (Vault signs)
        → User loads profile (JWT auth)
          → Config generated (D2-D6 endpoints)
            → Agent runs with profile settings
```

### Component Relationships

```
┌─────────────────────────────────────────────────────────┐
│ profiles/finance.yaml                                    │  ← Human-editable YAML
│ (60+ fields: providers, extensions, policies, privacy)  │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ ProfileValidator::validate()                            │  ← 6-step validation
│ - Required fields, providers, recipes, extensions,      │
│   privacy, policies                                      │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ Postgres: profiles table                                │  ← JSONB storage
│ - role (PK), display_name, description, config (JSONB), │
│   signature (JSONB), created_at, updated_at              │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ Vault Transit Engine (HMAC-SHA256)                      │  ← Cryptographic signing
│ - Key: profile-signing                                   │
│ - Signature format: vault:v1:BASE64_HMAC                 │
└─────────────────────────────────────────────────────────┘
```

### File Organization

```
profiles/
  finance.yaml         # Budget approvals, compliance
  legal.yaml           # Attorney-client privilege, local-only
  developer.yaml       # Code review, deployment
  hr.yaml              # Hiring, performance reviews
  executive.yaml       # Strategic planning, reporting
  support.yaml         # Customer success, ticketing

src/profile/
  schema.rs            # Rust types (Profile, Providers, Extension, Policy, etc.)
  validator.rs         # Validation logic (6 rules)
  mod.rs               # Public API

src/db/
  profiles.rs          # CRUD operations (create, read, update, delete, publish)

src/controller/routes/admin/
  profiles.rs          # HTTP endpoints (D7-D9: create, update, publish)
```

---

## 3. Schema Specification

### 3.1 Top-Level Profile Structure

```rust
pub struct Profile {
    pub role: String,                           // PK (e.g., "finance")
    pub display_name: String,                   // Human-readable (e.g., "Finance Team Agent")
    pub description: String,                    // Role description
    pub providers: Providers,                   // LLM configuration
    pub extensions: Vec<Extension>,             // MCP extensions
    pub goosehints: GooseHints,                 // Global context
    pub gooseignore: GooseIgnore,               // Privacy protection
    pub recipes: Vec<Recipe>,                   // Workflow automation
    pub automated_tasks: Vec<AutomatedTask>,    // Scheduled execution
    pub policies: Vec<Policy>,                  // RBAC/ABAC rules
    pub privacy: PrivacyConfig,                 // Privacy Guard settings
    pub env_vars: HashMap<String, String>,      // Role defaults
    pub signature: Option<Signature>,           // Vault HMAC (optional)
}
```

**Required Fields**: `role`, `display_name`, `description`, `providers`, `extensions`, `goosehints`, `gooseignore`, `recipes`, `policies`, `privacy`

**Optional Fields**: `automated_tasks`, `env_vars`, `signature`

---

### 3.2 Providers Configuration

```rust
pub struct Providers {
    pub primary: ProviderConfig,                // Main model (required)
    pub planner: Option<ProviderConfig>,        // Planning model (optional, defaults to primary)
    pub worker: Option<ProviderConfig>,         // Worker model (optional, defaults to primary)
    pub allowed_providers: Vec<String>,         // Governance constraint (empty = all allowed)
    pub forbidden_providers: Vec<String>,       // Governance constraint (takes precedence)
}

pub struct ProviderConfig {
    pub provider: String,                       // "openrouter", "anthropic", "openai", "ollama"
    pub model: String,                          // e.g., "anthropic/claude-3.5-sonnet"
    pub temperature: Option<f32>,               // 0.0-1.0 (optional)
}
```

**Example** (Finance role):
```yaml
providers:
  primary:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
    temperature: 0.3  # Conservative for financial accuracy
  planner:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
    temperature: 0.2  # Even more conservative
  worker:
    provider: "openrouter"
    model: "openai/gpt-4o-mini"
    temperature: 0.4  # Routine tasks
  allowed_providers: ["openrouter"]
  forbidden_providers: []
```

**Lead-Worker Pattern**: Finance uses Claude 3.5 Sonnet (planner) + GPT-4o-mini (worker) for cost optimization on routine tasks.

---

### 3.3 Extensions Configuration

```rust
pub struct Extension {
    pub name: String,                           // Must match Block registry catalog
    pub enabled: bool,                          // Whether extension is active
    pub tools: Option<Vec<String>>,             // Tool allowlist (None = all tools allowed)
    pub preferences: Option<HashMap<String, serde_json::Value>>,  // Arbitrary JSON config
}
```

**Example** (Finance role):
```yaml
extensions:
  - name: "github"
    enabled: true
    tools: ["list_issues", "create_issue", "add_comment"]  # Budget tracking only
    
  - name: "memory"
    enabled: true
    preferences:
      retention_days: 90  # Quarterly reporting cycle
      auto_summarize: true
      include_pii: false  # No salary data
      
  - name: "excel-mcp"
    enabled: true  # Finance-specific spreadsheet operations
```

**Tool Allowlist**: Finance can list/create GitHub issues (budget tracking) but cannot use `developer__shell` (policy enforcement).

---

### 3.4 GooseHints Configuration

```rust
pub struct GooseHints {
    pub global: String,                         // Applied to all sessions
    pub local_templates: Vec<LocalTemplate>,    // Path-specific hints
}

pub struct LocalTemplate {
    pub path: String,                           // Path where template applies
    pub content: String,                        // Template content
}
```

**Example** (Finance role):
```yaml
goosehints:
  global: |
    # Finance Role Context
    You are the Finance team agent for the organization.
    Your primary responsibilities are:
    - Budget compliance and spend tracking
    - Regulatory reporting (SOX, GAAP)
    - Financial forecasting
    
    Financial Data Sources:
    @finance/policies/approval-matrix.md
    @finance/budgets/fy2026-budget.xlsx
    
    Key Metrics: Budget utilization, burn rate, variance to plan
```

**Purpose**: Provides role-specific context to LLM (e.g., approval thresholds, data sources, compliance requirements).

---

### 3.5 GooseIgnore Configuration

```rust
pub struct GooseIgnore {
    pub global: String,                         // Applied to all sessions
    pub local_templates: Vec<LocalTemplate>,    // Path-specific ignore patterns
}
```

**Example** (Finance role):
```yaml
gooseignore:
  global: |
    **/.env
    **/.env.*
    **/secrets.*
    **/credentials.*
    
    # Finance-specific exclusions
    **/salary_data.*
    **/bonus_plans.*
    **/tax_records.*
    **/employee_compensation.*
    **/payroll_*
    **/ssn_*
    **/banking_credentials.*
```

**Purpose**: Prevents LLM from accessing sensitive files (PII, credentials, financial data).

---

### 3.6 Recipes Configuration

```rust
pub struct Recipe {
    pub name: String,                           // Recipe identifier
    pub description: Option<String>,            // Human-readable description
    pub path: String,                           // Relative to recipes/{role}/
    pub schedule: String,                       // Cron expression
    pub enabled: bool,                          // Whether recipe is active
}
```

**Example** (Finance role):
```yaml
recipes:
  - name: "monthly-variance-report"
    description: "Generate variance report on 5th business day"
    path: "variance-report.yaml"
    schedule: "0 9 * * 1-5"  # Mon-Fri 9am
    enabled: true
```

**Cron Format**: Standard 5-field cron (minute hour day month weekday).

---

### 3.7 Automated Tasks Configuration

```rust
pub struct AutomatedTask {
    pub name: String,                           // Task identifier
    pub recipe: String,                         // Recipe to execute
    pub schedule: String,                       // Cron expression
    pub enabled: bool,                          // Whether task is active
    pub notify_on_failure: bool,                // Alert on task failure
}
```

**Example** (Finance role):
```yaml
automated_tasks:
  - name: "daily-budget-check"
    recipe: "budget-compliance-check"
    schedule: "0 8 * * 1-5"  # Mon-Fri 8am
    enabled: true
    notify_on_failure: true
```

---

### 3.8 Policies Configuration

```rust
pub struct Policy {
    pub rule_type: String,                      // "allow_tool", "deny_tool", "allow_data", "deny_data"
    pub pattern: String,                        // Wildcard pattern (e.g., "github__*")
    pub conditions: Option<HashMap<String, String>>,  // Context-based rules
    pub reason: Option<String>,                 // Human-readable explanation
}
```

**⚠️ CRITICAL: Dual Deserialization Format**

Policies support **two serialization formats** to accommodate YAML and JSON:

**Format 1: YAML (rule type as key)**
```yaml
policies:
  - allow_tool: "github__*"
    reason: "Budget tracking"
    
  - deny_tool: "developer__shell"
    reason: "No code execution for Finance"
    conditions:
      repo: "finance/*"
```

**Format 2: JSON (explicit rule_type field)**
```json
{
  "policies": [
    {"rule_type": "allow_tool", "pattern": "github__*", "reason": "Budget tracking"},
    {"rule_type": "deny_tool", "pattern": "developer__shell", "reason": "No code exec"}
  ]
}
```

**Custom Deserializer**: `impl<'de> Deserialize<'de> for Policy` (schema.rs:294-424) handles both formats.

**Conditions Format**: Object or array of single-key objects
```yaml
# Object format
conditions:
  repo: "finance/*"
  database: "analytics_*"

# Array format (YAML)
conditions:
  - repo: "finance/*"
  - database: "analytics_*"
```

---

### 3.9 Privacy Configuration

```rust
pub struct PrivacyConfig {
    pub mode: String,                           // "rules", "ner", "hybrid"
    pub strictness: String,                     // "strict", "moderate", "permissive"
    pub allow_override: bool,                   // Whether user can override
    pub local_only: Option<bool>,               // Force local providers (Ollama)
    pub retention_days: Option<i32>,            // Memory retention (0 = ephemeral)
    pub rules: Vec<RedactionRule>,              // Regex-based PII masking
    pub pii_categories: Vec<String>,            // NER categories (e.g., "SSN", "EMAIL")
}

pub struct RedactionRule {
    pub pattern: String,                        // Regex (e.g., r"\b\d{3}-\d{2}-\d{4}\b")
    pub replacement: String,                    // Replacement (e.g., "[SSN]")
    pub category: Option<String>,               // PII category
}
```

**Example** (Finance role - strict PII protection):
```yaml
privacy:
  mode: "strict"
  strictness: "strict"
  allow_override: false  # No user bypass for Finance
  retention_days: 30     # 30-day memory retention
  rules:
    - pattern: '\b\d{3}-\d{2}-\d{4}\b'  # SSN
      replacement: "[SSN]"
      category: "SSN"
    - pattern: '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'  # Email
      replacement: "[EMAIL]"
      category: "EMAIL"
  pii_categories: ["SSN", "EMAIL", "PHONE", "CREDIT_CARD", "BANK_ACCOUNT"]
```

**Example** (Legal role - local-only enforcement):
```yaml
privacy:
  mode: "strict"
  strictness: "strict"
  allow_override: false
  local_only: true       # Attorney-client privilege (Ollama only)
  retention_days: 0      # Ephemeral memory only
  pii_categories: ["SSN", "EMAIL", "PHONE", "ADDRESS", "CONTRACT_ID"]
```

**Privacy Modes**:
- `rules`: Regex-based redaction only
- `ner`: NER model (qwen3) detection + redaction
- `hybrid`: Both rules and NER (highest protection)

**Strictness Levels**:
- `strict`: Block on PII detection, no cloud providers
- `moderate`: Redact PII, allow cloud providers
- `permissive`: Log PII, no blocking

---

### 3.10 Environment Variables

```rust
pub env_vars: HashMap<String, String>
```

**Example** (Finance role):
```yaml
env_vars:
  BUDGET_APPROVAL_THRESHOLD: "10000"
  FINANCE_REPO: "finance/budget-tracking"
  REPORTING_TIMEZONE: "America/New_York"
```

**Purpose**: Role-specific defaults for tools and scripts.

---

### 3.11 Signature (Vault-backed HMAC)

```rust
pub struct Signature {
    pub algorithm: String,                      // "sha2-256" (HMAC-SHA256)
    pub vault_key: String,                      // Transit key path
    pub signed_at: Option<String>,              // ISO 8601 timestamp
    pub signed_by: Option<String>,              // Admin email
    pub signature: Option<String>,              // Base64-encoded HMAC
}
```

**Example** (after D9 publish):
```yaml
signature:
  algorithm: "sha2-256"
  vault_key: "transit/keys/profile-signing"
  signed_at: "2025-11-07T04:29:31.058861974+00:00"
  signed_by: "admin@example.com"
  signature: "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="
```

**Signature Format**: `vault:v1:BASE64_HMAC` (Vault transit engine)

**Purpose**: Prevents tampering with profile (e.g., Finance user granting themselves `developer__shell` tool).

**Phase 6 Verification**: On profile load, verify signature matches profile data (not implemented in Phase 5 MVP).

---

## 4. Deserialization Deep Dive

### 4.1 Serde Patterns Used

The Profile system uses **serde** for YAML/JSON deserialization with several advanced patterns:

#### Pattern 1: Optional Field Skipping
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub signature: Option<Signature>
```
- **Purpose**: Omit `null` values from JSON output
- **Effect**: Existing profiles without signatures serialize cleanly

#### Pattern 2: Default Values
```rust
#[serde(default)]
pub automated_tasks: Vec<AutomatedTask>
```
- **Purpose**: Provide fallback if field missing
- **Effect**: Empty vec if `automated_tasks` not in YAML

#### Pattern 3: Field Aliases
```rust
#[serde(skip_serializing_if = "Option::is_none", alias = "value")]
pub signature: Option<String>
```
- **Purpose**: Support multiple field names
- **Effect**: YAML can use `value` or `signature` for backward compat

#### Pattern 4: Custom Deserializers
```rust
impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    // ... 130 lines of custom logic ...
}
```
- **Purpose**: Handle YAML vs JSON format differences
- **Effect**: Policies work in both `{allow_tool: "pattern"}` and `{"rule_type": "allow_tool"}` formats

---

### 4.2 Common Deserialization Errors

#### Error 1: Missing Required Field
```
Error: missing field `description` at line 1 column 40
```

**Cause**: Profile struct has non-optional field without `#[serde(default)]`

**Fix**: Add all required fields to YAML
```yaml
role: "finance"
display_name: "Finance Agent"
description: "Budget approvals"  # ← REQUIRED
providers: ...
```

**Prevention**: Use complete profile template (see `profiles/finance.yaml`)

---

#### Error 2: Type Mismatch
```
Error: invalid type: string "0.3", expected f32 at line 12 column 20
```

**Cause**: YAML quoted number (`"0.3"`) instead of unquoted (`0.3`)

**Fix**:
```yaml
# ❌ WRONG
temperature: "0.3"

# ✅ CORRECT
temperature: 0.3
```

---

#### Error 3: Invalid Enum Value
```
Error: Invalid privacy mode: 'invalid_mode'. Must be one of: ["rules", "ner", "hybrid"]
```

**Cause**: Validator rejects value not in allowed set

**Fix**: Use valid privacy mode
```yaml
privacy:
  mode: "strict"  # Must be "rules", "ner", or "hybrid"
```

---

#### Error 4: Policy Deserialization Ambiguity
```
Error: missing field 'rule_type or policy rule (e.g., allow_tool)'
```

**Cause**: Policy has neither YAML format (rule type as key) nor JSON format (explicit `rule_type`)

**Fix**:
```yaml
# ❌ WRONG
policies:
  - pattern: "github__*"  # Missing rule type!

# ✅ CORRECT (YAML format)
policies:
  - allow_tool: "github__*"
    reason: "Budget tracking"

# ✅ CORRECT (JSON format)
policies:
  - rule_type: "allow_tool"
    pattern: "github__*"
    reason: "Budget tracking"
```

---

#### Error 5: Conditions Format Error
```
Error: Condition array items must be objects, got: String("finance/*")
```

**Cause**: Array format requires objects, not strings

**Fix**:
```yaml
# ❌ WRONG
conditions:
  - "finance/*"

# ✅ CORRECT (array of single-key objects)
conditions:
  - repo: "finance/*"

# ✅ CORRECT (object format)
conditions:
  repo: "finance/*"
```

---

### 4.3 Deserialization Flow Diagram

```
YAML file (profiles/finance.yaml)
  │
  ▼
serde_yaml::from_str()
  │
  ├──▶ Profile::deserialize()
  │      ├──▶ Providers::deserialize()
  │      │      ├──▶ ProviderConfig::deserialize() (primary)
  │      │      ├──▶ ProviderConfig::deserialize() (planner, optional)
  │      │      └──▶ ProviderConfig::deserialize() (worker, optional)
  │      │
  │      ├──▶ Vec<Extension>::deserialize()
  │      │      └──▶ Extension::deserialize() (for each)
  │      │
  │      ├──▶ GooseHints::deserialize()
  │      ├──▶ GooseIgnore::deserialize()
  │      ├──▶ Vec<Recipe>::deserialize()
  │      ├──▶ Vec<AutomatedTask>::deserialize() (#[serde(default)])
  │      │
  │      ├──▶ Vec<Policy>::deserialize()  ⚠️ CUSTOM DESERIALIZER
  │      │      │
  │      │      └──▶ PolicyVisitor::visit_map()
  │      │             ├──▶ Detect format (YAML vs JSON)
  │      │             ├──▶ Parse conditions (object vs array)
  │      │             └──▶ Build Policy struct
  │      │
  │      ├──▶ PrivacyConfig::deserialize()
  │      │      ├──▶ mode: String
  │      │      ├──▶ strictness: String
  │      │      ├──▶ local_only: Option<bool>
  │      │      └──▶ Vec<RedactionRule>::deserialize()
  │      │
  │      ├──▶ HashMap<String, String>::deserialize() (env_vars, #[serde(default)])
  │      └──▶ Option<Signature>::deserialize() (#[serde(skip_serializing_if)])
  │
  ▼
Profile struct instance
  │
  ▼
ProfileValidator::validate()
  ├──▶ validate_required_fields()
  ├──▶ validate_providers()
  ├──▶ validate_recipes()
  ├──▶ validate_extensions()
  ├──▶ validate_privacy()
  └──▶ validate_policies()
  │
  ▼
Validated Profile
  │
  ▼
Database INSERT (JSONB)
```

---

### 4.4 Debugging Deserialization

**Step 1: Check YAML syntax**
```bash
# Use yamllint or online YAML validator
yamllint profiles/finance.yaml
```

**Step 2: Test minimal profile**
```yaml
role: "test"
display_name: "Test"
description: "Test profile"
providers:
  primary:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
  allowed_providers: []
  forbidden_providers: []
extensions: []
goosehints:
  global: ""
gooseignore:
  global: ""
recipes: []
policies: []
privacy:
  mode: "moderate"
  strictness: "moderate"
  allow_override: true
```

**Step 3: Add fields incrementally**
```bash
# Test after each addition
curl -X POST http://localhost:8088/admin/profiles \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d @test-profile.json
```

**Step 4: Check Rust error messages**
```bash
# Controller logs show exact deserialization error
docker logs ce_controller | grep -A5 "deserialization error"
```

---

## 5. Validation Rules

### ProfileValidator Implementation

Location: `src/profile/validator.rs`

**6 Validation Rules**:

1. **Required Fields** (`validate_required_fields`)
   - `role` must be non-empty
   - `display_name` must be non-empty

2. **Providers** (`validate_providers`)
   - `primary.provider` must be in `allowed_providers` (if list non-empty)
   - `primary.provider` must NOT be in `forbidden_providers`
   - `planner.provider` must be in `allowed_providers` (if specified)
   - `worker.provider` must be in `allowed_providers` (if specified)
   - `temperature` must be in range [0.0, 1.0]

3. **Recipes** (`validate_recipes`)
   - Recipe paths must exist (if `recipes/` directory exists)
   - `schedule` field must be non-empty

4. **Extensions** (`validate_extensions`)
   - Extension `name` must be non-empty
   - Future: Validate against Block registry catalog (Phase 6)

5. **Privacy** (`validate_privacy`)
   - `mode` must be one of: `["rules", "ner", "hybrid"]`
   - `strictness` must be one of: `["strict", "moderate", "permissive"]`
   - Redaction rule `pattern` must be non-empty
   - Redaction rule `replacement` must be non-empty

6. **Policies** (`validate_policies`)
   - `rule_type` must be one of: `["allow_tool", "deny_tool", "allow_data", "deny_data"]`
   - `pattern` must be non-empty

---

### Validation Flow

```
POST /admin/profiles (D7) or PUT /admin/profiles/{role} (D8)
  │
  ▼
Deserialize request body → Profile struct
  │
  ▼
ProfileValidator::validate(&profile)
  │
  ├──▶ validate_required_fields() → role, display_name non-empty
  │
  ├──▶ validate_providers()
  │      ├──▶ Check primary.provider in allowed_providers
  │      ├──▶ Check primary.provider NOT in forbidden_providers
  │      ├──▶ Check planner/worker providers (if specified)
  │      └──▶ Check temperature range [0.0, 1.0]
  │
  ├──▶ validate_recipes()
  │      ├──▶ Check recipe paths exist (if recipes/ dir exists)
  │      └──▶ Check schedule non-empty
  │
  ├──▶ validate_extensions()
  │      └──▶ Check extension name non-empty
  │
  ├──▶ validate_privacy()
  │      ├──▶ Check mode in ["rules", "ner", "hybrid"]
  │      ├──▶ Check strictness in ["strict", "moderate", "permissive"]
  │      └──▶ Check redaction rules (pattern, replacement non-empty)
  │
  └──▶ validate_policies()
         ├──▶ Check rule_type in ["allow_tool", "deny_tool", "allow_data", "deny_data"]
         └──▶ Check pattern non-empty
  │
  ▼
If ALL validations pass → INSERT/UPDATE database
If ANY validation fails → Return 400 Bad Request with error message
```

---

### Example Validation Errors

**Error**: Primary provider not in allowed list
```
Error: Primary provider 'anthropic' not in allowed_providers list: ["openrouter"]
```

**Error**: Invalid privacy mode
```
Error: Invalid privacy mode: 'invalid_mode'. Must be one of: ["rules", "ner", "hybrid"]
```

**Error**: Temperature out of range
```
Error: Primary provider temperature must be between 0.0 and 1.0, got 1.5
```

**Error**: Empty policy pattern
```
Error: Policy pattern cannot be empty
```

---

## 6. Database Storage

### Schema

```sql
CREATE TABLE profiles (
    role VARCHAR(255) PRIMARY KEY,              -- Role identifier (e.g., "finance")
    display_name VARCHAR(255) NOT NULL,         -- Human-readable name
    description TEXT NOT NULL,                  -- Role description
    config JSONB NOT NULL,                      -- Complete Profile struct as JSONB
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for JSONB queries (future optimization)
CREATE INDEX idx_profiles_config ON profiles USING GIN (config);
```

### JSONB Storage Format

**Database Row Example** (finance profile):
```json
{
  "role": "finance",
  "display_name": "Finance Team Agent",
  "description": "Budget approvals and reporting",
  "config": {
    "role": "finance",
    "display_name": "Finance Team Agent",
    "description": "Budget approvals, compliance reporting, financial analysis",
    "providers": {
      "primary": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.3},
      "planner": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet", "temperature": 0.2},
      "worker": {"provider": "openrouter", "model": "openai/gpt-4o-mini", "temperature": 0.4},
      "allowed_providers": ["openrouter"],
      "forbidden_providers": []
    },
    "extensions": [
      {"name": "github", "enabled": true, "tools": ["list_issues", "create_issue"]},
      {"name": "memory", "enabled": true, "preferences": {"retention_days": 90}}
    ],
    "goosehints": {"global": "# Finance Role Context\n..."},
    "gooseignore": {"global": "**/.env\n**/salary_data.*\n..."},
    "recipes": [],
    "automated_tasks": [],
    "policies": [
      {"rule_type": "deny_tool", "pattern": "developer__shell", "reason": "No code execution"}
    ],
    "privacy": {
      "mode": "strict",
      "strictness": "strict",
      "allow_override": false,
      "retention_days": 30,
      "rules": [
        {"pattern": "\\b\\d{3}-\\d{2}-\\d{4}\\b", "replacement": "[SSN]", "category": "SSN"}
      ],
      "pii_categories": ["SSN", "EMAIL", "PHONE"]
    },
    "env_vars": {},
    "signature": {
      "algorithm": "sha2-256",
      "vault_key": "transit/keys/profile-signing",
      "signed_at": "2025-11-07T04:29:31.058861974+00:00",
      "signed_by": "admin@example.com",
      "signature": "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="
    }
  },
  "created_at": "2025-11-07T04:29:14.087327749+00:00",
  "updated_at": "2025-11-07T04:29:31.058861974+00:00"
}
```

**Note**: `config` field contains **complete** Profile struct (for backward compatibility and future schema evolution).

---

### CRUD Operations

**CREATE** (D7):
```rust
pub async fn create_profile(pool: &PgPool, profile: &Profile) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO profiles (role, display_name, description, config)
        VALUES ($1, $2, $3, $4)
        "#,
        profile.role,
        profile.display_name,
        profile.description,
        serde_json::to_value(profile).unwrap()
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

**READ** (D1):
```rust
pub async fn get_profile(pool: &PgPool, role: &str) -> Result<Option<Profile>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT config FROM profiles WHERE role = $1"#,
        role
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| r.config.and_then(|c| serde_json::from_value(c).ok())))
}
```

**UPDATE** (D8):
```rust
pub async fn update_profile(pool: &PgPool, role: &str, profile: &Profile) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE profiles
        SET display_name = $2, description = $3, config = $4, updated_at = CURRENT_TIMESTAMP
        WHERE role = $1
        "#,
        role,
        profile.display_name,
        profile.description,
        serde_json::to_value(profile).unwrap()
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

**PUBLISH** (D9):
```rust
pub async fn publish_profile(pool: &PgPool, role: &str, signature: &Signature) -> Result<(), sqlx::Error> {
    // Fetch current profile
    let profile = get_profile(pool, role).await?;
    
    // Update signature field
    profile.signature = Some(signature);
    
    // Update database
    update_profile(pool, role, &profile).await?;
    Ok(())
}
```

---

### JSONB Queries (Future Optimization)

```sql
-- Find profiles with specific extension enabled
SELECT role, display_name
FROM profiles
WHERE config->'extensions' @> '[{"name": "github", "enabled": true}]';

-- Find profiles with strict privacy mode
SELECT role, display_name
FROM profiles
WHERE config->'privacy'->>'mode' = 'strict';

-- Find profiles with specific tool policy
SELECT role, display_name
FROM profiles
WHERE config->'policies' @> '[{"rule_type": "deny_tool", "pattern": "developer__shell"}]';
```

---

## 7. Profile Examples

### 7.1 Finance Profile (Annotated)

```yaml
# ============================================================================
# FINANCE TEAM AGENT PROFILE
# ============================================================================
# Purpose: Budget approvals, compliance reporting, financial analysis
# Risk Level: HIGH (handles sensitive financial data)
# Privacy: Strict PII protection (SSN, salary data)
# ============================================================================

role: "finance"                                # ← PRIMARY KEY (unique)
display_name: "Finance Team Agent"             # ← Human-readable name
description: "Budget approvals, compliance reporting, financial analysis, and regulatory oversight"

# ----------------------------------------------------------------------------
# LLM PROVIDER CONFIGURATION
# Lead-Worker Pattern: Claude 3.5 Sonnet (planner) + GPT-4o-mini (worker)
# ----------------------------------------------------------------------------
providers:
  primary:
    provider: "openrouter"                     # ← Cloud provider (anonymized data OK)
    model: "anthropic/claude-3.5-sonnet"       # ← High-quality model for accuracy
    temperature: 0.3                           # ← Conservative (financial accuracy)
    
  planner:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
    temperature: 0.2                           # ← Even more conservative for planning
    
  worker:
    provider: "openrouter"
    model: "openai/gpt-4o-mini"                # ← Cost-optimized for routine tasks
    temperature: 0.4
    
  allowed_providers: ["openrouter"]            # ← Governance constraint
  forbidden_providers: []                      # ← No restrictions (cloud OK)

# ----------------------------------------------------------------------------
# MCP EXTENSIONS
# Tool Allowlist: Finance can read GitHub issues (budget tracking) but
# cannot use developer__shell (enforced via policies)
# ----------------------------------------------------------------------------
extensions:
  - name: "github"
    enabled: true
    tools: ["list_issues", "create_issue", "add_comment"]  # ← Budget tracking only
    
  - name: "agent_mesh"
    enabled: true
    tools: ["send_task", "request_approval", "notify", "fetch_status"]
    
  - name: "memory"
    enabled: true
    preferences:
      retention_days: 90                       # ← Quarterly reporting cycle
      auto_summarize: true
      include_pii: false                       # ← No salary data in memory
      
  - name: "excel-mcp"
    enabled: true                              # ← Finance-specific spreadsheets

# ----------------------------------------------------------------------------
# GLOBAL GOOSEHINTS (ORG-WIDE CONTEXT)
# Provides role-specific context to LLM (approval thresholds, data sources)
# ----------------------------------------------------------------------------
goosehints:
  global: |
    # Finance Role Context
    You are the Finance team agent for the organization.
    Your primary responsibilities are:
    - Budget compliance and spend tracking
    - Regulatory reporting (SOX, GAAP)
    - Financial forecasting and variance analysis
    - Approval workflows for budget requests
    
    When analyzing budgets:
    - Always verify budget availability before approving spend requests
    - Document all approval decisions with rationale
    - Flag unusual spending patterns for review
    - Maintain audit trail for compliance
    
    Financial Data Sources:
    @finance/policies/approval-matrix.md       # ← Approval thresholds
    @finance/budgets/fy2026-budget.xlsx        # ← Current budget
    
    Key Metrics to Track:
    - Budget utilization % by department
    - Burn rate vs forecast
    - Variance to plan (±5% threshold)
    - Days cash on hand

# ----------------------------------------------------------------------------
# GLOBAL GOOSEIGNORE (PRIVACY PROTECTION)
# Prevents LLM from accessing sensitive financial data
# ----------------------------------------------------------------------------
gooseignore:
  global: |
    **/.env
    **/.env.*
    **/secrets.*
    **/credentials.*
    
    # Finance-specific exclusions
    **/salary_data.*                           # ← Employee compensation
    **/bonus_plans.*                           # ← Executive bonuses
    **/tax_records.*                           # ← Tax filings
    **/employee_compensation.*
    **/payroll_*
    **/ssn_*                                   # ← Social Security Numbers
    **/banking_credentials.*                   # ← Bank account details

# ----------------------------------------------------------------------------
# RECIPE AUTOMATION (SCHEDULED WORKFLOWS)
# Currently empty - recipes planned for Phase 6
# ----------------------------------------------------------------------------
recipes: []

# ----------------------------------------------------------------------------
# AUTOMATED TASKS (CRON-BASED EXECUTION)
# Currently empty - automated tasks planned for Phase 6
# ----------------------------------------------------------------------------
automated_tasks: []

# ----------------------------------------------------------------------------
# RBAC/ABAC POLICY RULES
# Enforces tool access restrictions (e.g., no developer__shell for Finance)
# ----------------------------------------------------------------------------
policies:
  - deny_tool: "developer__shell"              # ← No arbitrary code execution
    reason: "Finance role must not execute arbitrary code for compliance"
    
  - allow_tool: "github__list_issues"
    reason: "Read budget tracking issues"
    conditions:
      - repo: "finance/*"                      # ← Only finance repos
      
  - allow_tool: "github__create_issue"
    reason: "Create budget request issues"
    conditions:
      - repo: "finance/budget-requests"
      - project: "budgeting"

# ----------------------------------------------------------------------------
# PRIVACY GUARD CONFIGURATION
# Strict PII protection: SSN, EMAIL, PHONE, CREDIT_CARD, BANK_ACCOUNT
# ----------------------------------------------------------------------------
privacy:
  mode: "strict"                               # ← "rules" | "ner" | "hybrid"
  strictness: "strict"                         # ← "strict" | "moderate" | "permissive"
  allow_override: false                        # ← No user bypass for Finance
  retention_days: 30                           # ← 30-day memory retention
  
  rules:
    - pattern: '\b\d{3}-\d{2}-\d{4}\b'         # ← SSN regex
      replacement: "[SSN]"
      category: "SSN"
      
    - pattern: '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'  # ← Email regex
      replacement: "[EMAIL]"
      category: "EMAIL"
      
  pii_categories: ["SSN", "EMAIL", "PHONE", "CREDIT_CARD", "BANK_ACCOUNT"]

# ----------------------------------------------------------------------------
# ENVIRONMENT VARIABLES (ROLE DEFAULTS)
# ----------------------------------------------------------------------------
env_vars:
  BUDGET_APPROVAL_THRESHOLD: "10000"           # ← $10K approval threshold
  FINANCE_REPO: "finance/budget-tracking"
  REPORTING_TIMEZONE: "America/New_York"

# ----------------------------------------------------------------------------
# CRYPTOGRAPHIC SIGNATURE (VAULT-BACKED HMAC)
# Added after POST /admin/profiles/finance/publish (D9 endpoint)
# ----------------------------------------------------------------------------
# signature:  # ← NOT present until profile is published
#   algorithm: "sha2-256"
#   vault_key: "transit/keys/profile-signing"
#   signed_at: "2025-11-07T04:29:31.058861974+00:00"
#   signed_by: "admin@example.com"
#   signature: "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="
```

---

### 7.2 Legal Profile (Local-Only)

```yaml
# ============================================================================
# LEGAL TEAM AGENT PROFILE
# ============================================================================
# Purpose: Attorney-client privilege enforcement, contract review
# Risk Level: CRITICAL (attorney-client communications)
# Privacy: Local-only processing (Ollama), ephemeral memory
# ============================================================================

role: "legal"
display_name: "Legal Counsel Agent"
description: "Contract review, legal research, privilege-protected communications"

# ----------------------------------------------------------------------------
# LLM PROVIDER: LOCAL-ONLY (Ollama)
# Attorney-client privilege requires local processing (no cloud providers)
# ----------------------------------------------------------------------------
providers:
  primary:
    provider: "ollama"                         # ← LOCAL ONLY (attorney-client privilege)
    model: "qwen3:14b"
    temperature: 0.2                           # ← Very conservative for legal accuracy
    
  allowed_providers: ["ollama"]                # ← ONLY local providers allowed
  forbidden_providers: ["openrouter", "anthropic", "openai"]  # ← No cloud!

# ----------------------------------------------------------------------------
# EXTENSIONS: Limited toolset for legal tasks
# ----------------------------------------------------------------------------
extensions:
  - name: "github"
    enabled: true
    tools: ["list_issues", "create_issue"]     # ← Track legal matters
    
  - name: "memory"
    enabled: true
    preferences:
      retention_days: 0                        # ← EPHEMERAL ONLY (no persistence)
      auto_summarize: false
      include_pii: false

# ----------------------------------------------------------------------------
# GOOSEHINTS: Legal context and procedures
# ----------------------------------------------------------------------------
goosehints:
  global: |
    # Legal Role Context
    You are the Legal team agent for the organization.
    Your primary responsibilities are:
    - Contract review and negotiation
    - Legal research and case analysis
    - Privilege-protected communications
    
    ⚠️ CRITICAL: Attorney-Client Privilege
    - ALL communications are privileged and confidential
    - NEVER use cloud providers (Ollama only)
    - NEVER persist memory (ephemeral only)
    - NEVER share with non-legal personnel

# ----------------------------------------------------------------------------
# GOOSEIGNORE: Protect privileged communications
# ----------------------------------------------------------------------------
gooseignore:
  global: |
    **/.env
    **/contracts/*                             # ← All contracts privileged
    **/legal_opinions/*                        # ← Attorney work product
    **/client_communications/*                 # ← Privileged comms
    **/litigation/*                            # ← Case files

recipes: []
automated_tasks: []

# ----------------------------------------------------------------------------
# POLICIES: Enforce local-only processing
# ----------------------------------------------------------------------------
policies:
  - deny_tool: "developer__shell"
    reason: "No arbitrary code execution"
    
  - deny_data: "cloud_*"
    reason: "Attorney-client privilege requires local processing only"

# ----------------------------------------------------------------------------
# PRIVACY: Local-only enforcement
# ----------------------------------------------------------------------------
privacy:
  mode: "strict"
  strictness: "strict"
  allow_override: false                        # ← NO user bypass
  local_only: true                             # ← FORCE LOCAL PROVIDERS (Ollama)
  retention_days: 0                            # ← EPHEMERAL MEMORY ONLY
  pii_categories: ["SSN", "EMAIL", "PHONE", "ADDRESS", "CONTRACT_ID"]

env_vars:
  LEGAL_REPO: "legal/matters"
```

---

## 8. Troubleshooting

### Problem 1: Profile Won't Load

**Symptom**: `GET /profiles/finance` returns 404

**Possible Causes**:
1. Profile not in database
2. Role name mismatch (case-sensitive)
3. Database connection issue

**Debug Steps**:
```bash
# Check if profile exists
psql -U controller -d controller -c "SELECT role, display_name FROM profiles WHERE role = 'finance';"

# Check controller logs
docker logs ce_controller | grep -A5 "profile"

# Verify JWT authentication
curl -H "Authorization: Bearer $JWT" http://localhost:8088/profiles/finance
```

**Solution**: Create profile via D7 endpoint
```bash
curl -X POST http://localhost:8088/admin/profiles \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d @profiles/finance.json
```

---

### Problem 2: Validation Fails on CREATE

**Symptom**: `POST /admin/profiles` returns 400 with validation error

**Common Errors**:

**Error**: "Primary provider 'anthropic' not in allowed_providers list"
```yaml
# ❌ WRONG
providers:
  primary:
    provider: "anthropic"
  allowed_providers: ["openrouter"]

# ✅ FIX
providers:
  primary:
    provider: "openrouter"  # Or add "anthropic" to allowed_providers
  allowed_providers: ["openrouter"]
```

**Error**: "Invalid privacy mode: 'invalid_mode'"
```yaml
# ❌ WRONG
privacy:
  mode: "invalid_mode"

# ✅ FIX
privacy:
  mode: "strict"  # Must be "rules", "ner", or "hybrid"
```

**Error**: "Profile role cannot be empty"
```yaml
# ❌ WRONG
role: ""

# ✅ FIX
role: "finance"  # Non-empty string
```

---

### Problem 3: Policy Deserialization Fails

**Symptom**: "missing field 'rule_type or policy rule'"

**Cause**: Policy has neither YAML format nor JSON format

**Fix**: Use one of the two valid formats
```yaml
# ✅ YAML FORMAT
policies:
  - allow_tool: "github__*"
    reason: "Budget tracking"

# ✅ JSON FORMAT
policies:
  - rule_type: "allow_tool"
    pattern: "github__*"
    reason: "Budget tracking"
```

---

### Problem 4: Signature Not Showing After Publish

**Symptom**: `POST /admin/profiles/finance/publish` returns 200, but `signature` field is null

**Possible Causes**:
1. Vault not running
2. Transit engine not enabled
3. Environment variables missing

**Debug Steps**:
```bash
# Check Vault status
curl http://localhost:8200/v1/sys/health

# Check Transit engine
curl -H "X-Vault-Token: root" http://localhost:8200/v1/transit/keys/profile-signing

# Check controller env vars
docker exec ce_controller sh -c 'echo "VAULT_ADDR=$VAULT_ADDR VAULT_TOKEN=$VAULT_TOKEN"'
```

**Solution**: Enable Transit engine
```bash
curl -X POST -H "X-Vault-Token: root" \
  http://localhost:8200/v1/sys/mounts/transit \
  -d '{"type":"transit"}'
```

---

### Problem 5: Local-Only Enforcement Not Working

**Symptom**: Legal profile allowed to use OpenRouter despite `local_only: true`

**Cause**: Phase 5 MVP does not enforce `local_only` in config generation (deferred to Phase 6)

**Workaround**: Use `forbidden_providers` to block cloud providers
```yaml
providers:
  primary:
    provider: "ollama"
  allowed_providers: ["ollama"]
  forbidden_providers: ["openrouter", "anthropic", "openai"]  # ← Explicit deny
```

**Phase 6 Fix**: Config generation (D2-D6) will enforce `privacy.local_only` by filtering providers.

---

## 9. Testing Patterns

### 9.1 Unit Tests

**Schema Deserialization Tests** (`src/profile/schema.rs:tests`):
```rust
#[test]
fn test_policy_yaml_format_deserialization() {
    let yaml = r#"
- allow_tool: "github__*"
  reason: "Allowed for this role"
"#;
    let policies: Vec<Policy> = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(policies[0].rule_type, "allow_tool");
    assert_eq!(policies[0].pattern, "github__*");
}

#[test]
fn test_policy_json_format_deserialization() {
    let json = r#"[{"rule_type": "allow_tool", "pattern": "github__*"}]"#;
    let policies: Vec<Policy> = serde_json::from_str(json).unwrap();
    assert_eq!(policies[0].rule_type, "allow_tool");
}
```

**Validation Tests** (`src/profile/validator.rs:tests`):
```rust
#[test]
fn test_invalid_provider_not_in_allowed_list() {
    let mut profile = create_valid_finance_profile();
    profile.providers.allowed_providers = vec!["ollama".to_string()];
    
    let result = ProfileValidator::validate(&profile);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not in allowed_providers"));
}

#[test]
fn test_invalid_privacy_mode() {
    let mut profile = create_valid_finance_profile();
    profile.privacy.mode = "invalid_mode".to_string();
    
    let result = ProfileValidator::validate(&profile);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid privacy mode"));
}
```

**Run Unit Tests**:
```bash
cd src/controller
cargo test profile --lib
```

---

### 9.2 Integration Tests

**Admin Endpoints Tests** (`tests/integration/test_admin_profiles.sh`):
```bash
#!/bin/bash
# Test D7: Create Profile
CREATE_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/profiles" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d @test-profile.json)

if echo "$CREATE_RESPONSE" | jq -e '.role' > /dev/null; then
  echo "✓ D7 PASS: Profile created"
else
  echo "✗ D7 FAIL: $CREATE_RESPONSE"
fi

# Test D8: Update Profile
UPDATE_RESPONSE=$(curl -s -X PUT "$CONTROLLER_URL/admin/profiles/test-role" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"display_name": "Updated Name"}')

# Test D9: Publish Profile
PUBLISH_RESPONSE=$(curl -s -X POST "$CONTROLLER_URL/admin/profiles/test-role/publish" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

if echo "$PUBLISH_RESPONSE" | jq -e '.signature' | grep -q "vault:v1:"; then
  echo "✓ D9 PASS: Profile signed with Vault"
else
  echo "✗ D9 FAIL: Signature missing or invalid"
fi
```

**Run Integration Tests**:
```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./tests/integration/test_admin_profiles.sh
```

---

### 9.3 Regression Tests

**H Workstream Test Suite** (60 tests):
```bash
# Profile Loading (H2)
./tests/integration/test_profile_loading.sh  # 10/10 tests

# Privacy Guard (H3)
./tests/integration/test_finance_pii_jwt.sh  # 8/8 tests
./tests/integration/test_legal_local_jwt.sh  # 10/10 tests

# Org Chart (H4)
./tests/integration/test_org_chart_jwt.sh    # 12/12 tests

# E2E Workflow (H6)
./tests/integration/test_e2e_workflow.sh     # 10/10 tests

# All Profiles (H6.1)
./tests/integration/test_all_profiles_comprehensive.sh  # 20/20 tests

# Performance (H7)
./tests/perf/api_latency_benchmark.sh        # 7/7 tests
```

**Expected Results**: 60/60 tests passing (100%)

---

### 9.4 Performance Testing

**Profile Loading Latency** (`tests/perf/api_latency_benchmark.sh`):
```bash
# Measure P50, P95, P99 latency for GET /profiles/{role}
for i in {1..100}; do
  START=$(date +%s%N)
  curl -s -H "Authorization: Bearer $JWT" http://localhost:8088/profiles/finance > /dev/null
  END=$(date +%s%N)
  LATENCY=$(( (END - START) / 1000000 ))  # Convert to ms
  echo "$LATENCY"
done | sort -n | awk '
  {a[NR]=$1}
  END {
    print "P50:", a[int(NR*0.5)]
    print "P95:", a[int(NR*0.95)]
    print "P99:", a[int(NR*0.99)]
  }
'
```

**Target Latencies** (Phase 5):
- P50: <20ms
- P95: <50ms
- P99: <100ms

**Actual Results** (as of 2025-11-07):
- P50: 19ms ✅
- P95: 45ms ✅
- P99: 78ms ✅

---

## Appendix A: Complete Field Reference

| Field | Type | Required | Default | Constraints |
|-------|------|----------|---------|-------------|
| `role` | String | ✅ | - | Non-empty, unique (PK) |
| `display_name` | String | ✅ | - | Non-empty |
| `description` | String | ✅ | - | - |
| `providers.primary.provider` | String | ✅ | "openrouter" | Must be in `allowed_providers` |
| `providers.primary.model` | String | ✅ | "anthropic/claude-3.5-sonnet" | - |
| `providers.primary.temperature` | f32 | ❌ | 0.3 | 0.0-1.0 |
| `providers.planner` | ProviderConfig | ❌ | None | Provider in `allowed_providers` |
| `providers.worker` | ProviderConfig | ❌ | None | Provider in `allowed_providers` |
| `providers.allowed_providers` | Vec<String> | ❌ | [] | Empty = all allowed |
| `providers.forbidden_providers` | Vec<String> | ❌ | [] | Takes precedence |
| `extensions[].name` | String | ✅ | - | Non-empty |
| `extensions[].enabled` | bool | ✅ | - | - |
| `extensions[].tools` | Vec<String> | ❌ | None | None = all tools |
| `extensions[].preferences` | HashMap | ❌ | None | Arbitrary JSON |
| `goosehints.global` | String | ✅ | "" | - |
| `goosehints.local_templates` | Vec<LocalTemplate> | ❌ | [] | - |
| `gooseignore.global` | String | ✅ | "**/.env..." | - |
| `gooseignore.local_templates` | Vec<LocalTemplate> | ❌ | [] | - |
| `recipes[]` | Vec<Recipe> | ✅ | [] | Recipe paths validated |
| `automated_tasks[]` | Vec<AutomatedTask> | ❌ | [] | - |
| `policies[].rule_type` | String | ✅ | - | "allow_tool", "deny_tool", "allow_data", "deny_data" |
| `policies[].pattern` | String | ✅ | - | Non-empty |
| `policies[].conditions` | HashMap | ❌ | None | Object or array format |
| `policies[].reason` | String | ❌ | None | - |
| `privacy.mode` | String | ✅ | "moderate" | "rules", "ner", "hybrid" |
| `privacy.strictness` | String | ✅ | "moderate" | "strict", "moderate", "permissive" |
| `privacy.allow_override` | bool | ❌ | true | - |
| `privacy.local_only` | bool | ❌ | None | Phase 6: enforced in config |
| `privacy.retention_days` | i32 | ❌ | None | 0 = ephemeral |
| `privacy.rules[]` | Vec<RedactionRule> | ❌ | [] | Pattern, replacement non-empty |
| `privacy.pii_categories[]` | Vec<String> | ❌ | [] | "SSN", "EMAIL", etc. |
| `env_vars` | HashMap | ❌ | {} | - |
| `signature` | Signature | ❌ | None | Added via D9 publish |

---

## Appendix B: Related Documentation

- **Architecture**: `docs/HOW-IT-ALL-FITS-TOGETHER.md`
- **API Reference**: `docs/api/controller/README.md`
- **Vault Integration**: `docs/guides/VAULT.md`
- **Privacy Guard**: `docs/guides/privacy-guard-integration.md`
- **Migration Guide**: `docs/MIGRATION-PHASE5.md`
- **Test Results**: `docs/tests/phase5-test-results.md`
- **Progress Log**: `docs/tests/phase5-progress.md`
- **ADRs**: `docs/adr/0016-ce-profile-signing-key-management.md`

---

## Appendix C: Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-07 | Initial specification (Phase 5 MVP) |

---

**End of Profile System Specification**
