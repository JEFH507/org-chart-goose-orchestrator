# Phase 6 Architecture-Aligned Recommendations

**Date:** 2025-11-07  
**Context:** Based on complete /src architecture audit  
**Purpose:** Recommend Phase 6 approach that aligns with proven architecture

---

## 🎯 Key Insight: Your Architecture is Already Sound

### What the Audit Proved ✅

From `docs/architecture/SRC-ARCHITECTURE-AUDIT.md`:
- **All 6 /src directories are active and purposeful**
- **Service vs. Module pattern is proven:**
  - 2 Services: controller (8088), privacy-guard (8089)
  - 3 Modules: lifecycle, profile, vault (imported by controller)
  - 1 MCP Extension: agent-mesh (Python stdio)
- **Integration works:** 50/50 tests passing
- **Zero wasted work** across Phases 1-5

**This means:** Your architecture decisions have been correct. Phase 6 should BUILD ON this pattern, not replace it.

---

## 🏗️ Architecture Pattern to Maintain

### The Proven Pattern (Phases 1-5)

```
┌─────────────────────────────────────────────────────┐
│           STANDALONE SERVICES (Docker)              │
├─────────────────────────────────────────────────────┤
│  • controller (port 8088) - Main API               │
│  • privacy-guard (port 8089) - PII masking         │
└─────────────────┬───────────────────────────────────┘
                  │
                  │ (imports)
                  ↓
┌─────────────────────────────────────────────────────┐
│             LIBRARY MODULES (Rust)                  │
├─────────────────────────────────────────────────────┤
│  • lifecycle - Session state machine               │
│  • profile - Schema/validation/signing             │
│  • vault - Vault HTTP client                       │
└─────────────────────────────────────────────────────┘
```

**This pattern works because:**
- Services are independently testable (docker-compose up)
- Modules are reusable (imported by controller)
- Clear separation of concerns
- All tests pass (proven integration)

---

## 🎯 Recommendation: Extend the Pattern, Don't Break It

### Phase 6 Should Add (Following Existing Patterns)

#### 1. Privacy Guard Proxy Service (NEW SERVICE) ⭐ RECOMMENDED

**Why this fits:**
- ✅ **Matches proven pattern:** Add new service (like privacy-guard, controller)
- ✅ **Docker-first:** Runs as container in docker-compose
- ✅ **HTTP-based:** Uses existing HTTP architecture
- ✅ **Independently testable:** Can test proxy separately
- ✅ **No fork needed:** Works with current Goose Desktop

**Architecture:**
```
Existing:
  controller (8088)
  privacy-guard (8089)

NEW:
  privacy-guard-proxy (8090)  ← Intercepts LLM requests
```

**Implementation follows controller pattern:**
```
src/privacy-guard-proxy/
├── Dockerfile           ← Like controller/Dockerfile
├── Cargo.toml          ← Like controller/Cargo.toml
├── src/
│   ├── main.rs         ← Like controller/main.rs (Axum server)
│   ├── proxy.rs        ← Proxy logic (mask → forward → unmask)
│   └── token_store.rs  ← In-memory token storage
└── tests/
    └── integration/    ← Like controller tests
```

**Why this is consistent:**
- Uses Rust (like controller, privacy-guard)
- Uses Axum HTTP (like controller)
- Docker service (like controller, privacy-guard)
- Integration tests (like all other services)

---

#### 2. Admin UI Service (NEW SERVICE - Already Planned)

**Fits the pattern:**
- ✅ Standalone service (SvelteKit)
- ✅ Calls Controller API (HTTP client)
- ✅ Served by Controller (static files at /admin)
- ✅ Already documented in Phase 6 plan

**No changes needed** - this was already correct in draft plan.

---

#### 3. Profile CLI Helper (NEW SCRIPT - Simple)

**Why this fits:**
- ✅ **Scripts pattern:** Like existing scripts/ directory
- ✅ **Minimal change:** Doesn't modify Goose Desktop
- ✅ **HTTP client:** Calls Controller API (existing pattern)
- ✅ **Local file generation:** Saves config.yaml (standard practice)

**Implementation:**
```bash
# scripts/setup-profile.sh (100 lines Bash)

1. Prompt user for credentials
2. Get JWT from Keycloak (curl)
3. Call GET /profiles/{role} (curl)
4. Call GET /profiles/{role}/config (curl)
5. Call GET /profiles/{role}/goosehints (curl)
6. Save to ~/.config/goose/
7. Launch Goose Desktop
```

**Why this is better than fork:**
- No Goose Desktop modification
- Uses existing Controller API (already tested)
- Follows scripts/ directory pattern
- Fast to implement (1 day)

---

## ❌ What NOT to Do (Breaks Architecture Pattern)

### Don't: Fork Goose Desktop

**Why it breaks the pattern:**
- ❌ **Introduces new maintenance burden** (monthly merges)
- ❌ **Violates service separation** (puts business logic in desktop app)
- ❌ **Diverges from upstream** (community can't use it)
- ❌ **Duplicates functionality** (Controller API already does this)

**Our architecture is:**
- **Controller API** = Business logic (profiles, org chart, policies)
- **Goose Desktop** = Generic UI (unchanged)
- **Scripts** = Glue layer (setup, automation)

**Fork would make it:**
- **Goose-Enterprise** = Business logic + UI (tightly coupled)
- **Controller API** = Becomes redundant?

**This violates the service separation we've proven works.**

---

### Don't: Build Standalone UI Client

**Why it breaks the pattern:**
- ❌ **Reimplements Goose Desktop** (waste of effort)
- ❌ **Goose Desktop is excellent** (no need to replace)
- ❌ **4-6 weeks effort** (doesn't align with proven fast iterations)

**Our architecture builds on Goose, not replaces it.**

---

## ✅ Recommended Phase 6 Architecture

### The Right Way: Extend Service Pattern

```
┌─────────────────────────────────────────────────────┐
│              GOOSE DESKTOP (Unchanged)              │
│              Uses: ~/.config/goose/*                │
└────────┬────────────────────────────────────────────┘
         │
         │ (configured by setup-profile.sh script)
         │
         ↓
┌─────────────────────────────────────────────────────┐
│                SCRIPTS LAYER (NEW)                   │
├─────────────────────────────────────────────────────┤
│  • setup-profile.sh - Fetch profile from Controller│
│  • privacy-goose.sh - Launch with Privacy Guard    │
└────────┬────────────────────────────────────────────┘
         │
         │ (HTTP calls)
         │
         ↓
┌─────────────────────────────────────────────────────┐
│              SERVICES LAYER (HTTP)                   │
├─────────────────────────────────────────────────────┤
│  • controller (8088) - Profile API                 │
│  • privacy-guard (8089) - PII masking              │
│  • privacy-guard-proxy (8090) - LLM intercept NEW! │
│  • admin-ui (served by controller /admin) NEW!     │
└────────┬────────────────────────────────────────────┘
         │
         │ (imports)
         │
         ↓
┌─────────────────────────────────────────────────────┐
│              MODULES LAYER (Rust libs)              │
├─────────────────────────────────────────────────────┤
│  • lifecycle - Session state machine               │
│  • profile - Schema/validation/signing             │
│  • vault - Vault HTTP client                       │
└─────────────────────────────────────────────────────┘
```

**This maintains:**
- ✅ Service independence (Docker containers)
- ✅ Module reusability (Rust library imports)
- ✅ Script automation (setup/launch helpers)
- ✅ No desktop app forking

---

## 📊 Effort Comparison: Aligned vs. Fork Approach

| Component | Aligned Approach | Fork Approach | Delta |
|-----------|------------------|---------------|-------|
| **Privacy Guard** | Proxy service (3 days) | Modify providers (5 days) | -2 days |
| **Profile Loading** | Bash script (1 day) | CLI flag + auth (3 days) | -2 days |
| **Vault Production** | 2 days | 2 days | Same |
| **Admin UI** | 3 days | 3 days | Same |
| **Security** | 1 day | 1 day | Same |
| **Testing** | 2 days | 3 days (test fork) | -1 day |
| **Docs** | 1 day | 2 days (fork guide) | -1 day |
| **TOTAL** | **13 days** | **19 days** | **-6 days faster** |

**Aligned approach saves 6 days (30% faster)**

---

## 🎯 Final Recommendation

### ⭐ RECOMMENDED: Architecture-Aligned Approach

**Privacy Guard:** Build Privacy Guard Proxy Service  
**Profile Loading:** Build setup-profile.sh script  
**User Flow:**
```bash
# One-time setup (per role):
./scripts/setup-profile.sh finance
# Prompts for email/password
# Fetches config from Controller
# Saves to ~/.config/goose/
# Done! ✅

# Daily usage:
goose session start
# Goose Desktop with Finance profile
# Privacy Guard protects PII (transparent)
```

**Why This is Right:**
1. ✅ **Maintains proven architecture** (service pattern)
2. ✅ **No Goose fork** (avoids maintenance burden)
3. ✅ **Fast to implement** (13 days vs 19 days)
4. ✅ **Uses existing APIs** (Controller already has profile endpoints)
5. ✅ **Independently testable** (proxy service has own tests)
6. ✅ **Scalable** (add more services as needed)

---

## 📋 Phase 6 Revised Workstreams (Architecture-Aligned)

### A. Vault Production Completion (2 days)
- TLS, AppRole, Raft, Audit, Signature Verification
- **No changes from draft plan** ✅

### B. Admin UI (SvelteKit) (3 days)
- 5 pages: Dashboard, Profiles, Org Chart, Audit, Settings
- **No changes from draft plan** ✅

### C. Privacy Guard Proxy Service (3 days) 🆕 REPLACES "Goose Fork"
- **C1:** Create proxy service (Rust/Axum, port 8090)
- **C2:** Implement mask → forward → unmask logic
- **C3:** Support multiple providers (OpenRouter, Anthropic, OpenAI)
- **C4:** Add Docker service to docker-compose
- **C5:** Integration tests (proxy-specific)
- **Follows controller/privacy-guard pattern** ✅

### D. Profile Setup Script (1 day) 🆕 REPLACES "User UI"
- **D1:** Create setup-profile.sh (Bash, 100 lines)
- **D2:** Implement JWT fetching (curl to Keycloak)
- **D3:** Implement profile fetching (curl to Controller)
- **D4:** Save config.yaml, .goosehints, .gooseignore
- **D5:** Test with all 6 roles
- **Follows scripts/ directory pattern** ✅

### E. Wire Lifecycle into Routes (1 day) 🆕 FROM TODO
- **E1:** Update session routes to use lifecycle module
- **E2:** Add state transition validation
- **E3:** Add auto-expiration cron job
- **Completes Phase 4 infrastructure** ✅

### F. Security Hardening (1 day)
- Secrets cleanup, .env.example, SECURITY.md
- **No changes from draft plan** ✅

### G. Integration Testing (2 days)
- Vault tests, Admin UI tests, Proxy tests, Profile setup tests, Regression
- **Expanded for new components** ✅

### H. Documentation (1 day)
- Vault guide, Admin UI guide, Proxy guide, Setup script guide, Security guide, Migration guide
- **Updated for new approach** ✅

**Total:** 14 days (3 weeks calendar) - SAME TIMELINE, BETTER ARCHITECTURE ✅

---

## 🔧 Implementation Details: Privacy Guard Proxy

### Why Rust (Not TypeScript)?

**Aligns with existing architecture:**
- Controller: Rust/Axum ✅
- Privacy Guard: Rust/Axum ✅
- Proxy: Rust/Axum ✅ (consistent!)

**Benefits:**
- Same language as other services
- Can import privacy-guard types (Rust modules)
- Performance (Rust is faster than Node.js)
- Type safety (compile-time checks)
- Team already knows Rust (proven in Phases 1-5)

### Proxy Service Structure

```
src/privacy-guard-proxy/
├── Cargo.toml
├── Dockerfile
├── README.md
└── src/
    ├── main.rs              # Axum server (like controller)
    ├── proxy.rs             # Proxy logic
    ├── privacy_client.rs    # HTTP client for privacy-guard
    ├── token_store.rs       # In-memory token storage
    └── providers/
        ├── openrouter.rs    # OpenRouter-specific proxy
        ├── anthropic.rs     # Anthropic-specific proxy
        └── openai.rs        # OpenAI-specific proxy
```

### Core Proxy Logic

```rust
// src/main.rs
use axum::{Router, routing::post, Json};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/chat/completions", post(proxy_chat_completions))
        .route("/health", get(health));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8090").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// src/proxy.rs
async fn proxy_chat_completions(
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>> {
    let privacy = PrivacyClient::new("http://privacy-guard:8089");
    
    // Extract user message
    let user_msg = req.messages.last().unwrap().content.clone();
    
    // Scan for PII
    let scan = privacy.scan(&user_msg).await?;
    
    if scan.detections.is_empty() {
        // No PII, pass through
        return forward_to_llm(&req).await;
    }
    
    // Mask PII
    let masked = privacy.mask(&user_msg).await?;
    let session_id = masked.session_id.clone();
    
    // Store tokens for unmask
    TOKENS.lock().unwrap().insert(session_id.clone(), masked.redactions);
    
    // Replace message
    let mut masked_req = req.clone();
    masked_req.messages.last_mut().unwrap().content = masked.masked_text;
    
    // Forward to LLM
    let llm_response = forward_to_llm(&masked_req).await?;
    
    // Unmask response
    let tokens = TOKENS.lock().unwrap().get(&session_id).unwrap().clone();
    let unmasked = unmask_text(&llm_response.choices[0].message.content, &tokens);
    
    // Clean up
    TOKENS.lock().unwrap().remove(&session_id);
    
    Ok(Json(ChatResponse {
        choices: vec![Choice {
            message: Message {
                content: unmasked,
                ..llm_response.choices[0].message.clone()
            }
        }],
        ..llm_response
    }))
}
```

### Goose Configuration (User's ~/.config/goose/config.yaml)

```yaml
# Generated by setup-profile.sh from Finance profile

provider: openrouter
model: anthropic/claude-3.5-sonnet
temperature: 0.3

# Privacy Guard Proxy (NEW - added by profile)
# Goose will use localhost:8090 instead of openrouter.ai
GOOSE_PROVIDER__OPENROUTER_BASE_URL: http://localhost:8090

extensions:
  - name: github
    enabled: true
  - name: agent_mesh
    enabled: true
```

**How it works:**
1. User types in Goose Desktop
2. Goose sends POST to http://localhost:8090/v1/chat/completions (proxy, not OpenRouter)
3. Proxy masks PII → forwards to OpenRouter → unmasks response
4. User sees unmasked response (transparent)

**No Goose modification needed!** ✅

---

## 🔧 Implementation Details: Profile Setup Script

### Why Bash Script (Not Goose Fork)?

**Aligns with existing architecture:**
- scripts/ directory exists ✅
- Other setup scripts (vault-init.sh) ✅
- Simple HTTP calls (curl) ✅
- No Goose dependency ✅

### Script Structure

```bash
#!/bin/bash
# scripts/setup-profile.sh

set -e

ROLE="${1:-finance}"  # Default to finance
CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"

echo "🔐 Setting up profile: $ROLE"

# 1. Get JWT token
echo "Authentication required for Controller API"
read -p "Email: " EMAIL
read -sp "Password: " PASSWORD
echo

JWT=$(curl -s -X POST "$KEYCLOAK_URL/realms/dev/protocol/openid-connect/token" \
  -d "client_id=goose-controller" \
  -d "grant_type=password" \
  -d "username=$EMAIL" \
  -d "password=$PASSWORD" \
  -d "scope=openid" | jq -r '.access_token')

if [ "$JWT" == "null" ]; then
  echo "❌ Authentication failed"
  exit 1
fi

echo "✅ Authenticated as $EMAIL"

# 2. Fetch profile
echo "📥 Fetching profile: $ROLE"
PROFILE=$(curl -s -H "Authorization: Bearer $JWT" \
  "$CONTROLLER_URL/profiles/$ROLE")

if echo "$PROFILE" | jq -e '.error' > /dev/null; then
  echo "❌ Profile fetch failed: $(echo $PROFILE | jq -r '.error')"
  exit 1
fi

echo "✅ Profile retrieved"

# 3. Fetch config.yaml
echo "📥 Fetching config.yaml"
curl -s -H "Authorization: Bearer $JWT" \
  "$CONTROLLER_URL/profiles/$ROLE/config" \
  > ~/.config/goose/config.yaml

# 4. Fetch goosehints
echo "📥 Fetching .goosehints"
curl -s -H "Authorization: Bearer $JWT" \
  "$CONTROLLER_URL/profiles/$ROLE/goosehints" \
  > ~/.config/goose/.goosehints

# 5. Fetch gooseignore
echo "📥 Fetching .gooseignore"
curl -s -H "Authorization: Bearer $JWT" \
  "$CONTROLLER_URL/profiles/$ROLE/gooseignore" \
  > ~/.config/goose/.gooseignore

# 6. Add Privacy Guard Proxy to config (if Privacy Guard enabled in profile)
PRIVACY_MODE=$(echo "$PROFILE" | jq -r '.privacy.mode')
if [ "$PRIVACY_MODE" != "null" ]; then
  echo "🛡️ Enabling Privacy Guard (mode: $PRIVACY_MODE)"
  echo "" >> ~/.config/goose/config.yaml
  echo "# Privacy Guard Proxy (auto-configured)" >> ~/.config/goose/config.yaml
  echo "GOOSE_PROVIDER__OPENROUTER_BASE_URL: http://localhost:8090" >> ~/.config/goose/config.yaml
fi

echo ""
echo "✅ Profile setup complete!"
echo ""
echo "📋 Profile: $ROLE"
echo "📋 Provider: $(echo $PROFILE | jq -r '.providers.primary.model')"
echo "📋 Privacy: $PRIVACY_MODE"
echo "📋 Extensions: $(echo $PROFILE | jq -r '.extensions | length') enabled"
echo ""
echo "🚀 Launch Goose Desktop:"
echo "   goose session start"
```

### Usage

```bash
# Finance user setup (one-time):
./scripts/setup-profile.sh finance

# Legal user setup (one-time):
./scripts/setup-profile.sh legal

# Daily usage (no setup needed):
goose session start
```

**Wrapper scripts for convenience:**
```bash
# scripts/goose-finance.sh
#!/bin/bash
./scripts/setup-profile.sh finance
goose session start

# scripts/goose-legal.sh
#!/bin/bash
./scripts/setup-profile.sh legal
goose session start
```

---

## 🎯 Phase 6 Revised Scope (Architecture-Aligned)

### Workstreams (14 days total)

**A. Vault Production Completion (2 days)**
- TLS, AppRole, Raft, Audit, Signature Verification
- ✅ Already planned correctly

**B. Admin UI (SvelteKit) (3 days)**
- 5 pages: Dashboard, Profiles, Org Chart, Audit, Settings
- ✅ Already planned correctly

**C. Privacy Guard Proxy Service (3 days) 🆕 NEW**
- Rust/Axum HTTP proxy (port 8090)
- Mask → Forward → Unmask logic
- Support 3+ providers (OpenRouter, Anthropic, OpenAI)
- Docker service in docker-compose
- Integration tests
- ✅ Follows proven service pattern

**D. Profile Setup Scripts (1 day) 🆕 NEW**
- setup-profile.sh (Bash, ~150 lines)
- JWT auth helper
- Config file generation
- Wrapper scripts (goose-finance, goose-legal)
- ✅ Follows scripts/ directory pattern

**E. Wire Lifecycle into Routes (1 day) 🆕 FROM TODO**
- Update session routes to use lifecycle module
- Add state transition validation
- ✅ Completes Phase 4 infrastructure

**F. Security Hardening (1 day)**
- Secrets cleanup, .env.example, SECURITY.md
- ✅ Already planned correctly

**G. Integration Testing (2 days)**
- Vault tests
- Admin UI tests (Playwright)
- Proxy tests (mask/unmask)
- Setup script tests (all 6 roles)
- Regression tests (Phase 1-5)
- ✅ Covers all new components

**H. Documentation (1 day)**
- Vault guide update (production complete)
- Admin UI guide (5 pages)
- **Privacy Guard Proxy guide** (NEW)
- **Profile Setup guide** (NEW)
- Security hardening guide
- Migration guide (v0.5.0 → v0.6.0)

---

## ✅ What You Get: Real Working MVP

### User Experience
```bash
# Day 1: Setup (one-time, 2 minutes)
./scripts/setup-profile.sh finance
# Email: user@company.com
# Password: ********
# ✅ Profile setup complete!

# Day 2+: Daily usage
goose session start
# Goose Desktop opens
# Configured with Finance profile
# Privacy Guard protects PII (transparent)
```

### Admin Experience
```bash
# Admin logs into Web UI
http://localhost:8088/admin
# Dashboard shows org chart (D3.js)
# Edit profiles (Monaco YAML editor)
# Upload org chart CSV
# Publish profiles (Vault signing)
```

### What Works (Critical Path)
1. ✅ User signs in (Keycloak OIDC via setup script)
2. ✅ Profile loaded from Controller (HTTP API)
3. ✅ Config files saved locally (~/.config/goose/)
4. ✅ Goose Desktop launches with profile
5. ✅ User chats "Contact john@acme.com"
6. ✅ Privacy Guard Proxy masks → LLM sees "Contact EMAIL_7a3f9b"
7. ✅ LLM responds with token
8. ✅ Proxy unmasks → User sees "john@acme.com"
9. ✅ Admin can manage profiles in Web UI
10. ✅ Vault signs profiles (tamper protection)

**THIS IS A COMPLETE, WORKING MVP** ✅

---

## 🚀 Recommendation Summary

### ✅ DO THIS (Architecture-Aligned)
- Build Privacy Guard Proxy (Rust service, port 8090)
- Build Profile Setup Script (Bash, scripts/)
- Wire lifecycle into routes (complete Phase 4)
- Build Admin UI (SvelteKit, already planned)
- Complete Vault production (already planned)

### ❌ DON'T DO THIS (Breaks Architecture)
- Don't fork Goose Desktop (adds complexity, breaks service pattern)
- Don't build standalone UI (waste of effort, Goose Desktop is excellent)
- Don't skip Privacy Guard Proxy (PII protection requires it)

### 📅 Timeline
- **Week 1:** Vault Production (2d) + Proxy Service (3d)
- **Week 2:** Admin UI (3d) + Setup Scripts (1d) + Lifecycle (1d)
- **Week 3:** Security (1d) + Testing (2d) + Docs (1d) + Buffer (1d)
- **Total:** 14 days (3 weeks calendar)

### 💰 Cost Comparison
- **Aligned approach:** 14 days (follows proven patterns)
- **Fork approach:** 19 days (adds fork maintenance)
- **Savings:** 5 days (26% faster)

---

## ✅ Next Actions

1. **Review this document** (30 minutes)
2. **Decide:** Aligned approach (recommended) OR Fork approach (better UX, more work)
3. **If Aligned:**
   - Update Phase-6-Checklist.md with new workstreams (C, D, E)
   - Remove "Goose Fork" workstream
   - Add "Privacy Guard Proxy" workstream
   - Add "Profile Setup Scripts" workstream
4. **If Fork:**
   - Keep existing REVISED-SCOPE.md plan
   - Accept 19-day timeline
   - Plan for upstream merge maintenance
5. **Validate with CLI wrapper first** (build scripts/privacy-goose-validate.sh)
6. **Start Phase 6** after decision made

---

**Status:** ⏸️ DECISION PENDING  
**Recommendation:** ✅ Architecture-Aligned Approach (Proxy + Scripts)  
**Confidence:** HIGH (based on proven Phases 1-5 patterns)
