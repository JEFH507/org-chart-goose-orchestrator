# Phase 5 Orchestration Prompt: Profile System + Privacy Guard MCP + Admin UI

**Version:** 1.0.0  
**Target:** Grant application ready (v0.5.0)  
**Timeline:** 1.5-2 weeks  
**Status:** Ready to begin

---

## 🎯 Mission

Implement a comprehensive profile system that enables zero-touch deployment for users. When a user signs in via OIDC, their entire Goose environment is auto-configured: LLM provider settings, MCP extensions, goosehints/gooseignore, recipes, memory preferences, and privacy controls. Additionally, build a Privacy Guard MCP extension for local PII protection (no upstream Goose dependency required) and an Admin UI for managing profiles and org charts.

**Key Innovation:** Enterprise governance through profiles that bundle provider restrictions, recipe automation, and privacy controls—making Goose attractive to enterprises without requiring extensive IT configuration.

---

## 📋 Before You Begin

### Prerequisites Check:
- [ ] Phase 4 complete (v0.4.0 tagged)
- [ ] All Phase 1-4 tests passing (6/6 integration tests)
- [ ] Docker services running: Keycloak, Vault, Postgres, Redis, Ollama
- [ ] Development environment: Rust nightly, Node.js 20+, npm 10+

### Required Files:
- [x] `Phase-5-Agent-State.json` (state tracking)
- [x] `Phase-5-Checklist.md` (task checklist)
- [x] `Phase-5-Orchestration-Prompt.md` (this file)
- [ ] `docs/tests/phase5-progress.md` (to be created)

### Read First:
1. `Technical Project Plan/master-technical-project-plan.md` (Phase 5 section)
2. `docs/product/productdescription.md` (product vision)
3. `goose-versions-references/gooseV1.12.1/documentation/docs/guides/config-files.md` (config.yaml spec)
4. `goose-versions-references/gooseV1.12.1/documentation/docs/guides/using-goosehints.md` (hints spec)
5. `goose-versions-references/gooseV1.12.1/documentation/docs/guides/using-gooseignore.md` (ignore spec)

---

## 🏗️ Architecture Overview

### Phase 5 Components:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ADMIN UI (SvelteKit)                        │
│  ┌──────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐ ┌─────────┐       │
│  │Dashboard │ │Sessions │ │Profiles │ │ Audit │ │Settings │       │
│  │(D3.js    │ │(Table)  │ │(Monaco) │ │(CSV)  │ │(Org CSV)│       │
│  │ OrgChart)│ │         │ │         │ │       │ │         │       │
│  └──────────┘ └─────────┘ └─────────┘ └───────┘ └─────────┘       │
│                              ↓                                      │
│                    JWT Auth (Keycloak OIDC)                         │
└─────────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────────┐
│                      CONTROLLER API (Rust/Axum)                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Profile Endpoints:                                          │   │
│  │  GET /profiles/{role}           → Full profile JSON        │   │
│  │  GET /profiles/{role}/config    → Generated config.yaml    │   │
│  │  GET /profiles/{role}/goosehints→ Global hints             │   │
│  │  GET /profiles/{role}/gooseignore→Global ignore            │   │
│  │  GET /profiles/{role}/recipes   → Recipe list              │   │
│  │  POST /admin/profiles           → Create profile           │   │
│  │  PUT /admin/profiles/{role}     → Update profile           │   │
│  │  POST /admin/profiles/{role}/publish→Sign with Vault       │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Org Chart Endpoints:                                        │   │
│  │  POST /admin/org/import         → Upload CSV               │   │
│  │  GET /admin/org/imports         → Import history           │   │
│  │  GET /admin/org/tree            → Hierarchy tree           │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Policy Engine (RBAC/ABAC):                                  │   │
│  │  Middleware: enforce_policy                                 │   │
│  │  Redis cache: policy:{role}:{tool} (TTL: 5 min)            │   │
│  │  Deny by default                                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────────┐
│                     POSTGRES DATABASE                               │
│  profiles (role PK, data JSONB, signature, timestamps)             │
│  policies (role, tool_pattern, allow, conditions JSONB, reason)    │
│  org_users (user_id PK, reports_to_id FK, name, role FK, email)    │
│  org_imports (id, filename, uploaded_by, users_created, status)    │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                   GOOSE CLIENT WORKFLOW                             │
│  1. User signs in (Keycloak OIDC) → JWT with role claim            │
│  2. Client calls GET /profiles/{role}                              │
│  3. Downloads config.yaml, goosehints, gooseignore, recipes         │
│  4. Saves to ~/.config/goose/ (config.yaml, .goosehints, etc.)     │
│  5. Privacy Guard MCP intercepts requests (optional)               │
│  6. User's Goose environment fully configured ✅                    │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                  PRIVACY GUARD MCP (Local PII Protection)           │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Request Flow:                                                │  │
│  │  Goose Client → Privacy Guard MCP                            │  │
│  │              → Apply redaction (rules/ner/hybrid)            │  │
│  │              → Tokenize PII ("John" → [PERSON_A])            │  │
│  │              → Store tokens locally (~/.goose/pii-tokens/)   │  │
│  │              → Forward to OpenRouter (only tokens)           │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Response Flow:                                               │  │
│  │  OpenRouter → Privacy Guard MCP                              │  │
│  │            → Detokenize ([PERSON_A] → "John")                │  │
│  │            → Send audit log to Controller                    │  │
│  │            → Delete tokens                                   │  │
│  │            → Return to Goose Client                          │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  Key: LLM provider NEVER sees raw PII ✅                            │
└─────────────────────────────────────────────────────────────────────┘
```

### Profile Structure:

```yaml
# Example: profiles/analyst.yaml
role: "analyst"
display_name: "Business Analyst"
description: "Data analysis, process optimization, time studies"

providers:
  primary:
    provider: "openrouter"
    model: "anthropic/claude-3.5-sonnet"
    temperature: 0.3
  worker:
    provider: "openrouter"
    model: "openai/gpt-4o-mini"
  allowed_providers: ["openrouter", "ollama"]
  forbidden_providers: []

extensions:
  - name: "github"
    enabled: true
    tools: ["list_issues", "create_issue"]
  - name: "agent_mesh"
    enabled: true
  - name: "memory"
    enabled: true
    preferences:
      retention_days: 90
      auto_summarize: true
      include_pii: false
  - name: "excel-mcp"
    enabled: true
  - name: "sql-mcp"
    enabled: true

goosehints:
  global: |
    You are a business analyst. Focus on data-driven insights.
    @README.md
    @docs/data-dictionary.md
  local_templates:
    - path: "finance/budgets"
      content: |
        # Budget-specific context
        Current fiscal year: FY2026

gooseignore:
  global: |
    **/.env
    **/secrets.*
  local_templates:
    - path: "finance/budgets"
      content: |
        **/employee_salaries.*

recipes:
  - name: "daily-kpi-report"
    path: "recipes/analyst/daily-kpi-report.yaml"
    schedule: "0 9 * * 1-5"
    enabled: true

policies:
  - allow_tool: "excel-mcp__*"
  - allow_tool: "sql-mcp__query"
    conditions:
      - database: "analytics_*"
  - deny_tool: "developer__shell"
    reason: "No arbitrary code execution"

privacy:
  mode: "hybrid"  # rules, ner, hybrid
  strictness: "moderate"
  allow_override: true
  rules:
    - pattern: '\b\d{3}-\d{2}-\d{4}\b'
      replacement: '[SSN]'
  pii_categories: ["SSN", "EMAIL", "PHONE"]

signature:
  algorithm: "HS256"
  vault_key: "transit/keys/profile-signing"
  signed_at: "2025-11-05T14:00:00Z"
  signed_by: "admin@company.com"
```

---

## 📝 Workstream Execution Plan

### Execution Order:
**Sequential execution recommended** (dependencies between workstreams)

```
A (Profile Format) 
  → B (Role Profiles) 
    → C (Policy Engine) 
      → D (API Endpoints) 
        → E (Privacy Guard MCP) 
          → F (Org Chart) 
            → G (Admin UI) 
              → H (Integration Testing) 
                → I (Documentation) 
                  → J (Progress Tracking)
```

### Workstream Dependencies:
- **B depends on A**: Role profiles need schema from A
- **C depends on B**: Policy engine needs profiles to evaluate
- **D depends on A, B, C**: API endpoints serve profiles and enforce policies
- **E independent**: Privacy Guard MCP can be developed in parallel
- **F independent**: Org chart can be developed in parallel
- **G depends on D, F**: Admin UI needs API endpoints and org chart data
- **H depends on all**: Integration testing validates everything
- **I depends on all**: Documentation describes completed features
- **J final**: Progress tracking wraps up phase

---

## 🚨 CRITICAL: Strategic Checkpoint Protocol

**After EVERY workstream (A-I), you MUST:**

1. **Update `Phase-5-Agent-State.json`:**
   ```json
   {
     "workstreams": {
       "A": {
         "status": "complete",
         "actual_effort": "1.5 days",
         "checkpoints": [
           {"id": "A1", "status": "complete", "completed_at": "2025-11-05T10:30:00Z"},
           ...
           {"id": "A_CHECKPOINT", "status": "complete", "completed_at": "2025-11-05T12:00:00Z"}
         ]
       }
     }
   }
   ```

2. **Update `docs/tests/phase5-progress.md`:**
   ```markdown
   ## 2025-11-05 12:00 - Workstream A Complete ✅
   
   **Completed:**
   - Profile schema defined (src/profile/schema.rs)
   - Cross-field validation (src/profile/validator.rs)
   - Vault signing integration
   - Postgres migration (profiles table)
   - 15 unit tests passing
   
   **Next:** Workstream B (Role Profiles)
   ```

3. **Update `Phase-5-Checklist.md`:**
   - Mark all A tasks complete: `- [x] A1: Define JSON Schema...`
   - Mark A_CHECKPOINT complete

4. **Commit to git:**
   ```bash
   git add .
   git commit -m "Phase 5: Workstream A complete (Profile Bundle Format)"
   git push origin main
   ```

**Why This Matters:**
- If session ends mid-phase, we can resume from last checkpoint
- If context window limit reached, state is preserved
- Progress is visible to user at any time
- Phase 4 proved this pattern works successfully

**DO NOT skip checkpoints even if you think you can complete multiple workstreams in one session.**

---

## 🎬 Workstream A: Profile Bundle Format (1.5 days)

### Goal:
Define the profile schema, validation logic, Vault signing, and Postgres storage.

### Context:
- Phase 3 Controller API has `GET /profiles/{role}` returning mock data
- This workstream replaces mock with real Postgres-backed profiles
- **No API signature changes** (backward compatible)

### Tasks:

#### A1: Define JSON Schema (Rust serde types)
**File:** `src/profile/schema.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub role: String,
    pub display_name: String,
    pub description: String,
    pub providers: Providers,
    pub extensions: Vec<Extension>,
    pub goosehints: GooseHints,
    pub gooseignore: GooseIgnore,
    pub recipes: Vec<Recipe>,
    pub automated_tasks: Vec<AutomatedTask>,
    pub policies: Vec<Policy>,
    pub privacy: PrivacyConfig,
    pub env_vars: HashMap<String, String>,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Providers {
    pub primary: ProviderConfig,
    pub planner: Option<ProviderConfig>,
    pub worker: Option<ProviderConfig>,
    pub allowed_providers: Vec<String>,
    pub forbidden_providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: String,  // e.g., "openrouter"
    pub model: String,     // e.g., "anthropic/claude-3.5-sonnet"
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub name: String,
    pub enabled: bool,
    pub tools: Option<Vec<String>>,
    pub preferences: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseHints {
    pub global: String,
    pub local_templates: Vec<LocalTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseIgnore {
    pub global: String,
    pub local_templates: Vec<LocalTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTemplate {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub path: String,
    pub schedule: String,  // Cron expression
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedTask {
    pub name: String,
    pub recipe: String,
    pub schedule: String,
    pub enabled: bool,
    pub notify_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub rule_type: String,  // "allow_tool", "deny_tool", "allow_data", "deny_data"
    pub pattern: String,    // e.g., "github__*", "sql-mcp__query"
    pub conditions: Option<HashMap<String, String>>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub mode: String,       // "rules", "ner", "hybrid"
    pub strictness: String, // "strict", "moderate", "permissive"
    pub allow_override: bool,
    pub local_only: Option<bool>,
    pub rules: Vec<RedactionRule>,
    pub pii_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRule {
    pub pattern: String,    // Regex pattern
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: String,
    pub vault_key: String,
    pub signed_at: String,
    pub signed_by: String,
    pub signature: String,
}
```

**Validation:**
- Required fields: `role`, `display_name`, `providers`
- Type checking (serde handles this)

#### A2: Implement cross-field validation
**File:** `src/profile/validator.rs`

```rust
use crate::profile::schema::Profile;
use anyhow::{Result, bail};

pub struct ProfileValidator;

impl ProfileValidator {
    pub fn validate(profile: &Profile) -> Result<()> {
        // 1. allowed_providers must include primary.provider
        if !profile.providers.allowed_providers.contains(&profile.providers.primary.provider) {
            bail!(
                "Primary provider '{}' not in allowed_providers list",
                profile.providers.primary.provider
            );
        }
        
        // 2. Recipe paths must exist
        for recipe in &profile.recipes {
            let recipe_path = format!("recipes/{}/{}", profile.role, recipe.path);
            if !std::path::Path::new(&recipe_path).exists() {
                bail!("Recipe file not found: {}", recipe_path);
            }
        }
        
        // 3. Extension names must match Block registry catalog
        // (For MVP, we'll just validate they're non-empty)
        for ext in &profile.extensions {
            if ext.name.is_empty() {
                bail!("Extension name cannot be empty");
            }
        }
        
        // 4. Privacy mode validation
        let valid_modes = vec!["rules", "ner", "hybrid"];
        if !valid_modes.contains(&profile.privacy.mode.as_str()) {
            bail!("Invalid privacy mode: {}", profile.privacy.mode);
        }
        
        // 5. Strictness validation
        let valid_strictness = vec!["strict", "moderate", "permissive"];
        if !valid_strictness.contains(&profile.privacy.strictness.as_str()) {
            bail!("Invalid privacy strictness: {}", profile.privacy.strictness);
        }
        
        Ok(())
    }
}
```

#### A3: Vault signing integration
**File:** `src/profile/signer.rs` (reuse Phase 1 Vault client)

```rust
use crate::vault::client::VaultClient;
use crate::profile::schema::Profile;
use anyhow::Result;

pub struct ProfileSigner {
    vault_client: VaultClient,
}

impl ProfileSigner {
    pub fn new(vault_client: VaultClient) -> Self {
        Self { vault_client }
    }
    
    pub async fn sign(&self, profile: &Profile, signed_by: &str) -> Result<String> {
        // Serialize profile to JSON (excluding signature field)
        let profile_json = serde_json::to_string(profile)?;
        
        // HMAC with Vault transit key
        let signature = self.vault_client
            .transit_sign("profile-signing", &profile_json)
            .await?;
        
        Ok(signature)
    }
    
    pub async fn verify(&self, profile: &Profile, signature: &str) -> Result<bool> {
        let profile_json = serde_json::to_string(profile)?;
        
        self.vault_client
            .transit_verify("profile-signing", &profile_json, signature)
            .await
    }
}
```

#### A4: Postgres storage schema
**File:** `migrations/XXX_create_profiles.sql`

```sql
-- Create profiles table
CREATE TABLE profiles (
    role VARCHAR(50) PRIMARY KEY,
    display_name VARCHAR(100) NOT NULL,
    data JSONB NOT NULL,  -- Full profile JSON
    signature TEXT,       -- Vault HMAC signature
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Index for common queries
CREATE INDEX idx_profiles_display_name ON profiles(display_name);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_profiles_updated_at
    BEFORE UPDATE ON profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

**Run migration:**
```bash
sqlx migrate run
```

#### A5: Unit tests
**File:** `tests/unit/profile_validation_test.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_profile() {
        let profile = create_valid_finance_profile();
        assert!(ProfileValidator::validate(&profile).is_ok());
    }
    
    #[test]
    fn test_invalid_provider_not_in_allowed_list() {
        let mut profile = create_valid_finance_profile();
        profile.providers.allowed_providers = vec!["ollama".to_string()];
        // primary.provider is "openrouter" (not in allowed list)
        
        assert!(ProfileValidator::validate(&profile).is_err());
    }
    
    #[test]
    fn test_invalid_recipe_path() {
        let mut profile = create_valid_finance_profile();
        profile.recipes.push(Recipe {
            name: "nonexistent".to_string(),
            path: "recipes/finance/nonexistent.yaml".to_string(),
            schedule: "0 9 * * *".to_string(),
            enabled: true,
        });
        
        assert!(ProfileValidator::validate(&profile).is_err());
    }
    
    #[test]
    fn test_missing_required_fields() {
        let profile = Profile {
            role: "".to_string(),  // Empty role
            ..Default::default()
        };
        
        assert!(ProfileValidator::validate(&profile).is_err());
    }
    
    #[test]
    fn test_invalid_privacy_mode() {
        let mut profile = create_valid_finance_profile();
        profile.privacy.mode = "invalid_mode".to_string();
        
        assert!(ProfileValidator::validate(&profile).is_err());
    }
    
    // Add 10+ more test cases...
}
```

#### A_CHECKPOINT: Update logs
```bash
# 1. Update Phase-5-Agent-State.json
# 2. Update docs/tests/phase5-progress.md
# 3. Update Phase-5-Checklist.md (mark A tasks complete)
# 4. Commit to git
git add .
git commit -m "Phase 5: Workstream A complete (Profile Bundle Format)"
git push origin main
```

**Deliverables:**
- [x] `src/profile/schema.rs`
- [x] `src/profile/validator.rs`
- [x] `src/profile/signer.rs`
- [x] `migrations/XXX_create_profiles.sql`
- [x] `tests/unit/profile_validation_test.rs` (15+ tests passing)

**Backward Compatibility Check:**
- [ ] Phase 3 Controller API `GET /profiles/{role}` still works
- [ ] No API signature changes

**Estimated Time:** 1.5 days

---

## 🚨 RESUME PROTOCOL (If Session Ends or Context Limit Reached)

### How to Resume Work:

1. **Read State Files:**
   ```bash
   # 1. Check current status
   cat "Technical Project Plan/PM Phases/Phase-5/Phase-5-Agent-State.json" | jq '.status'
   
   # 2. Check which workstreams are complete
   cat "Technical Project Plan/PM Phases/Phase-5/Phase-5-Agent-State.json" | jq '.workstreams | to_entries[] | select(.value.status == "complete") | .key'
   
   # 3. Read progress log
   cat docs/tests/phase5-progress.md
   
   # 4. Check checklist for pending tasks
   grep "\[ \]" "Technical Project Plan/PM Phases/Phase-5/Phase-5-Checklist.md" | head -20
   ```

2. **Identify Resume Point:**
   ```bash
   # Find last completed checkpoint
   cat "Technical Project Plan/PM Phases/Phase-5/Phase-5-Agent-State.json" | \
     jq '.workstreams | to_entries[] | select(.value.checkpoints[-1].status == "complete") | {workstream: .key, last_checkpoint: .value.checkpoints[-1].id}'
   ```

3. **Resume Execution:**
   - If Workstream A complete → Start Workstream B
   - If Workstream B in progress → Check last completed checkpoint (B1, B2, etc.)
   - If Workstream B checkpoint B3 complete → Resume at B4
   - Always verify by running existing tests before adding new code

4. **Restore Context:**
   ```bash
   # Re-read key files
   cat goose-versions-references/gooseV1.12.1/documentation/docs/guides/config-files.md
   cat goose-versions-references/gooseV1.12.1/documentation/docs/guides/using-goosehints.md
   cat "Technical Project Plan/master-technical-project-plan.md" | grep -A 100 "Phase 5:"
   ```

5. **Verify Environment:**
   ```bash
   # Check services
   docker-compose ps
   
   # Run Phase 1-4 regression tests
   ./tests/integration/regression_suite.sh
   
   # Expected: All tests pass (6/6)
   ```

6. **Continue from Checkpoint:**
   - Start next uncompleted task from checklist
   - Follow workstream execution plan
   - **ALWAYS update logs at next checkpoint** (don't skip)

### Resume Example:

```markdown
## Resume Scenario: Session ended during Workstream B

**State Check:**
- Workstream A: ✅ Complete
- Workstream B: 🟡 In Progress
  - B1: ✅ Finance profile created
  - B2: ✅ Manager profile created
  - B3: ❌ Analyst profile (in progress, not complete)
  - B_CHECKPOINT: ❌ Not reached

**Action:**
1. Read existing Finance and Manager profile files
2. Continue Workstream B from B3 (Analyst profile)
3. Complete B3-B10 tasks
4. Update logs at B_CHECKPOINT
5. Move to Workstream C

**DO NOT:**
- Skip B_CHECKPOINT (logs MUST be updated)
- Assume B3 is complete without verifying files exist
- Start Workstream C before B is complete
```

---

## 📊 Success Metrics

At the end of Phase 5, you should have:

### Quantitative:
- [ ] 60+ files created
- [ ] 5,000+ lines of code written
- [ ] 6 role profiles operational
- [ ] 18 recipe templates created
- [ ] 12 new API endpoints functional
- [ ] 50+ unit tests passing
- [ ] 25+ integration tests passing
- [ ] 1 E2E workflow test passing
- [ ] Performance: P50 < 5s (API routes)
- [ ] Performance: P50 < 500ms (Privacy Guard regex-only)

### Qualitative:
- [ ] All Phase 1-4 tests still pass (no regressions)
- [ ] Admin UI deployed and accessible
- [ ] Org chart visualization working (D3.js)
- [ ] Privacy Guard MCP functional (PII never seen by LLM provider)
- [ ] Documentation complete (2,000+ lines Markdown)
- [ ] Grant application ready (v0.5.0 tagged)

### Acceptance Criteria:
- [ ] Finance user signs in → Profile auto-loaded → OpenRouter config applied
- [ ] Finance user sends "John SSN 123-45-6789" → OpenRouter sees "[PERSON_A] SSN [SSN_XXX]"
- [ ] Legal user sends contract → Ollama local (never hits cloud)
- [ ] Admin uploads org chart CSV → Tree visualization appears on dashboard
- [ ] Admin creates new profile → Profile appears in list → Users can be assigned

---

## 🎯 Next Actions

1. **Create progress log:**
   ```bash
   touch docs/tests/phase5-progress.md
   echo "# Phase 5 Progress Log" > docs/tests/phase5-progress.md
   echo "" >> docs/tests/phase5-progress.md
   echo "## $(date +%Y-%m-%d\ %H:%M) - Phase 5 Started" >> docs/tests/phase5-progress.md
   echo "" >> docs/tests/phase5-progress.md
   echo "**Initial State:**" >> docs/tests/phase5-progress.md
   echo "- Phase 4 complete (v0.4.0)" >> docs/tests/phase5-progress.md
   echo "- All Phase 1-4 tests passing (6/6)" >> docs/tests/phase5-progress.md
   echo "- Ready to begin Workstream A" >> docs/tests/phase5-progress.md
   git add docs/tests/phase5-progress.md
   git commit -m "Phase 5: Initialize progress log"
   git push origin main
   ```

2. **Begin Workstream A:**
   - Read this orchestration prompt fully
   - Read Phase 5 section in master plan
   - Read Goose v1.12.1 documentation (config.yaml, goosehints, gooseignore)
   - Start with A1: Define JSON Schema
   - Follow tasks A1 → A2 → A3 → A4 → A5 → A_CHECKPOINT

3. **Remember:**
   - Update logs at EVERY checkpoint (A-I)
   - Don't skip checkpoints
   - Commit to git after each checkpoint
   - Verify backward compatibility at each workstream

---

## 📚 Reference Documentation

### Must-Read Before Starting:
1. **Master Plan:** `Technical Project Plan/master-technical-project-plan.md` (Phase 5 section)
2. **Product Vision:** `docs/product/productdescription.md`
3. **Goose Config Spec:** `goose-versions-references/gooseV1.12.1/documentation/docs/guides/config-files.md`
4. **Goose Hints Spec:** `goose-versions-references/gooseV1.12.1/documentation/docs/guides/using-goosehints.md`
5. **Goose Ignore Spec:** `goose-versions-references/gooseV1.12.1/documentation/docs/guides/using-gooseignore.md`

### Helpful References:
- Phase 4 Orchestration Prompt (proven checkpoint pattern)
- Phase 3 Controller API (existing endpoints)
- Phase 2.2 Privacy Guard (Ollama NER logic to reuse)
- Phase 1 Vault Client (signing logic to reuse)

---

## ✅ Final Checklist Before Starting

- [ ] All prerequisite files read
- [ ] Phase 4 complete (v0.4.0 tagged)
- [ ] Docker services running
- [ ] Development environment ready
- [ ] Progress log initialized
- [ ] Understand checkpoint protocol
- [ ] Understand resume protocol
- [ ] Ready to begin Workstream A

---

**Good luck! Remember: Strategic checkpoints after EVERY workstream. Phase 4 proved this works. Let's make Phase 5 even better! 🚀**

---

**Version History:**
- v1.0.0 (2025-11-05): Initial orchestration prompt with resume section and strategic checkpoints
