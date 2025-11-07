# Phase 6 Architecture Clarification - Questions Answered

**Date:** 2025-11-07  
**Context:** User Questions about Real MVP and User UI Architecture

---

## Question 1: By end of Phase 6, do I have real working MVP?

### Short Answer: **YES, but with revised scope**

**Original Phase 6 Plan Had a Critical Flaw:**
- ❌ Assumed "User UI (SvelteKit browser)" that wraps Goose Desktop
- ❌ Assumed "Goose Desktop backend mode" (doesn't exist)
- ❌ Assumed Privacy Guard as MCP (already moved to HTTP in Phase 5)

**CORRECTED Architecture:**
- ✅ Users use **Goose Desktop directly** (forked version: `goose-enterprise`)
- ✅ Privacy Guard is **HTTP service** (Phase 5, already working)
- ✅ Goose Desktop fork integrates with Controller + Privacy Guard

---

### What You Have by End of Phase 6 (REAL MVP):

#### 1. **End Users (Finance, Legal, HR, etc.)**

**User Experience:**
```bash
# Launch Goose Desktop with Finance profile
goose-enterprise --profile finance
```

**What Happens:**
1. Goose prompts for credentials (email/password)
2. Gets JWT from Keycloak
3. Fetches Finance profile from Controller (GET /profiles/finance)
4. Downloads config.yaml, .goosehints, .gooseignore
5. Saves to ~/.config/goose/
6. Launches Goose Desktop (existing UI - Home, Chat, History, Recipes, Scheduler, Extensions, Settings)
7. **User interacts with Goose Desktop app** (not a browser)

**When User Chats:**
```
User types: "Analyze budget for john.doe@acme.com"
  ↓
Goose Desktop (forked) → Privacy Guard HTTP (POST /guard/mask)
  ↓
Privacy Guard returns: "Analyze budget for EMAIL_7a3f9b"
  ↓
Goose Desktop → OpenRouter API (masked text)
  ↓
OpenRouter → Claude 3.5 Sonnet (sees only EMAIL_7a3f9b)
  ↓
Response: "EMAIL_7a3f9b has overspent by 20%"
  ↓
Goose Desktop → Privacy Guard HTTP (POST /guard/reidentify)
  ↓
Privacy Guard returns: "john.doe@acme.com has overspent by 20%"
  ↓
User sees: "john.doe@acme.com has overspent by 20%" (original PII)
```

**Key Point:** User interacts with **Goose Desktop app** (existing UI is excellent - see screenshots)

---

#### 2. **Admins**

**Admin Experience:**
```
Open browser → http://localhost:8088/admin
  ↓
Keycloak login (admin@example.com)
  ↓
Admin UI (SvelteKit)
```

**Admin Can:**
- View org chart (D3.js visualization)
- Create/edit profiles (Monaco YAML editor)
- Publish profiles (triggers Vault signing)
- Upload org chart CSV
- View audit logs
- Monitor Vault status (sealed/unsealed)

---

#### 3. **IT/Security**

**What They Get:**
- Vault production-ready (TLS, AppRole, Raft, audit)
- Profile signatures (tamper detection)
- No secrets in repo (.env.example)
- Privacy Guard HTTP service (masks PII)
- Audit logs (all operations tracked)

---

### What's Missing for "Real Working MVP"?

**NOTHING CRITICAL IF WE FIX THE SCOPE!**

**Original Gap:**
- ❌ No "Goose Desktop backend mode" (doesn't exist)
- ❌ Privacy Guard MCP integration incomplete (Phase 5 test showed MCP too late in pipeline)

**Solution:**
- ✅ Fork Goose Desktop → Add Controller profile loading
- ✅ Fork Goose Desktop → Integrate Privacy Guard HTTP client into provider code
- ✅ Users use forked Goose Desktop (not a separate browser UI)

**With this correction, Phase 6 delivers REAL MVP** ✅

---

## Question 2: Explain User Architecture Better

### Your Question:
> "The user will be interacting exclusively with the browser UI right? ... Does goose desktop as a backend mode, why not cli, etc maybe I am not understanding. Why you have here Privacy Guard MCP, but now I thought we fully moved to Privacy guard HTTP API, no?"

### Answer: I WAS WRONG IN THE ORIGINAL PLAN

Let me explain the **CORRECT architecture** based on Goose Desktop screenshots you provided:

---

### CORRECT Architecture (Phase 6)

```
┌─────────────────────────────────────────────────────────────────┐
│                        END USER                                  │
│                                                                  │
│  User launches:                                                  │
│  $ goose-enterprise --profile finance                           │
│                                                                  │
│  (OR shortcut: $ goose-finance)                                 │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ↓
┌─────────────────────────────────────────────────────────────────┐
│              GOOSE DESKTOP APP (Forked)                         │
│              Binary: goose-enterprise                           │
│              Language: Rust                                     │
│                                                                  │
│  UI Tabs (from screenshots):                                    │
│  • Home (session stats, recent chats)                          │
│  • Chat (conversation interface)                                │
│  • History (past sessions)                                      │
│  • Recipes (view/import/run)                                    │
│  • Scheduler (manage automated tasks)                           │
│  • Extensions (enable/disable MCP extensions)                   │
│  • Settings (models, chat, session, app)                        │
│  • NEW: Profile (current profile, privacy overrides)           │
│                                                                  │
│  Startup Flow (NEW CODE):                                       │
│  1. Parse --profile flag                                        │
│  2. Prompt for credentials (if JWT expired)                    │
│  3. Get JWT from Keycloak                                       │
│  4. Fetch profile from Controller (GET /profiles/finance)      │
│  5. Fetch config.yaml (GET /profiles/finance/config)           │
│  6. Fetch goosehints (GET /profiles/finance/goosehints)        │
│  7. Fetch gooseignore (GET /profiles/finance/gooseignore)      │
│  8. Save to ~/.config/goose/                                    │
│  9. Launch Goose UI (existing code, no changes)                │
│                                                                  │
│  Chat Flow (NEW CODE in Provider):                              │
│  User types → Privacy Guard HTTP (mask) → LLM → Privacy Guard  │
│  HTTP (unmask) → User sees                                      │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ↓ HTTP Requests
┌─────────────────────────────────────────────────────────────────┐
│                    BACKEND SERVICES                              │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐                    │
│  │  Controller API  │  │ Privacy Guard    │                    │
│  │  Port: 8088      │  │ HTTP Service     │                    │
│  │                  │  │ Port: 8089       │                    │
│  │  Endpoints:      │  │                  │                    │
│  │  • GET /profiles │  │  Endpoints:      │                    │
│  │    /{role}       │  │  • POST /guard/  │                    │
│  │  • GET /profiles │  │    mask          │                    │
│  │    /{role}/config│  │  • POST /guard/  │                    │
│  │  • GET /profiles │  │    reidentify    │                    │
│  │    /{role}/      │  │  • GET /guard/   │                    │
│  │    goosehints    │  │    status        │                    │
│  └──────────────────┘  └──────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

### Key Points:

1. **NO Browser UI for End Users:**
   - Users interact with **Goose Desktop app** (existing UI is great - see screenshots)
   - No need to rebuild UI from scratch
   - Just fork Goose Desktop, add profile loading + Privacy Guard integration

2. **NO "Goose Backend Mode":**
   - I was wrong to suggest this
   - Goose Desktop is a **desktop application** (Tauri + Rust)
   - It's NOT a CLI that can run in "server mode"
   - It's a **GUI app** with sidebar (Home, Chat, History, etc.)

3. **Privacy Guard is HTTP, NOT MCP:**
   - Phase 5 already uses Privacy Guard as HTTP service (port 8089)
   - Goose Desktop fork will call Privacy Guard HTTP API
   - NOT MCP (MCP was too late in pipeline, as you discovered)

4. **Users Interact with Goose Desktop App:**
   - See screenshot 1: Home screen (session stats, recent chats)
   - See screenshot 2: Goosehints editor (Monaco-like editor)
   - See screenshot 3: Recipes page (import, run recipes)
   - See screenshot 4: Extensions page (enable/disable MCP extensions)
   - See screenshot 5: Settings page (Model, Chat, Session, App tabs)
   - **This existing UI is excellent!** No need to rebuild in browser.

---

### How Users Get Configured (CORRECTED Flow)

```
Step 1: Admin creates Finance profile in Admin UI
  ↓
Admin publishes profile (Vault signs)
  ↓
Admin assigns user@company.com → finance role (in Keycloak)

Step 2: User launches Goose Desktop
  ↓
$ goose-enterprise --profile finance
  ↓
Goose prompts for credentials (if JWT expired)
  ↓
User enters email/password
  ↓
Goose gets JWT from Keycloak (role=finance)
  ↓
Goose fetches profile from Controller (authenticated)
  ↓
Goose downloads config.yaml, goosehints, gooseignore
  ↓
Goose saves to ~/.config/goose/
  ↓
Goose Desktop launches (UI appears - see screenshots)

Step 3: User chats
  ↓
User types: "Review contract for john@acme.com"
  ↓
Goose calls Privacy Guard HTTP: POST /guard/mask
  ↓
Privacy Guard returns: "Review contract for EMAIL_7a3f9b"
  ↓
Goose calls OpenRouter (masked text)
  ↓
Claude responds: "EMAIL_7a3f9b contract looks good"
  ↓
Goose calls Privacy Guard HTTP: POST /guard/reidentify
  ↓
Privacy Guard returns: "john@acme.com contract looks good"
  ↓
User sees: "john@acme.com contract looks good" (in Chat tab)
```

**User NEVER opens a browser for chatting.** They use **Goose Desktop app** (Tauri GUI).

---

### What About "Lightweight User UI"?

**Original Plan Said:**
> User Browser (SvelteKit Lightweight UI)
>   ↓ (HTTP/SSE)
> Goose Desktop (Backend Mode)
>   ↓ (MCP stdio)
> Privacy Guard MCP (Middleware)

**This Was WRONG. Here's Why:**

1. **Goose Desktop is NOT a backend:**
   - Goose Desktop is a **Tauri app** (Rust + WebView)
   - It has a GUI (see screenshots - sidebar, tabs, Monaco editor)
   - It cannot run in "backend mode" (no HTTP server)
   - It's designed for **desktop use** (Windows, macOS, Linux)

2. **Users don't need a separate browser UI:**
   - Goose Desktop UI is already excellent (see screenshots)
   - Has: Chat, History, Recipes, Scheduler, Extensions, Settings
   - Has: Monaco editor for goosehints (screenshot 2)
   - Has: Extension manager (screenshot 4)
   - **Why rebuild this in browser?** Just use it!

3. **Privacy Guard is HTTP, not MCP:**
   - Phase 5 already built Privacy Guard as HTTP service
   - Phase 5 tests showed **MCP too late in pipeline** (can't intercept provider calls)
   - Solution: Fork Goose Desktop → Add HTTP client calls to Privacy Guard in provider code

**CORRECTED Architecture:**

```
User Launches Goose Desktop (forked binary: goose-enterprise)
  ↓
Goose Desktop UI (Tauri app - existing UI, unchanged)
  ↓ (on startup)
Controller HTTP (fetch profile, config, hints, ignore)
  ↓ (on each LLM call)
Privacy Guard HTTP (mask before sending to LLM)
  ↓
OpenRouter/Cloud LLMs (receive masked text only)
  ↓ (response)
Privacy Guard HTTP (unmask before showing user)
  ↓
Goose Desktop UI (user sees unmasked response)
```

**No browser. No separate UI. Just forked Goose Desktop with profile loading + Privacy Guard integration.**

---

## Revised User Experience (Based on Goose Screenshots)

### What Users See:

**Home Tab** (Screenshot 1):
- Session count (392 total sessions)
- Token usage (676.71M total tokens)
- Recent chats list
- **NEW:** Current profile indicator (Finance Manager)

**Chat Tab** (existing):
- Conversation interface
- **NEW:** Privacy Guard status indicator (🛡️ 3 PII entities masked)

**History Tab** (existing):
- Past sessions
- **NO CHANGES**

**Recipes Tab** (Screenshot 3):
- List of recipes (from profile)
- Import Recipe button
- **ENHANCED:** Recipes auto-populated from Finance profile (monthly-budget-close, weekly-spend-report, quarterly-forecast)

**Scheduler Tab** (existing):
- Manage automated tasks
- **ENHANCED:** Auto-populated from profile.automated_tasks

**Extensions Tab** (Screenshot 4):
- Enable/disable MCP extensions
- **ENHANCED:** Extensions pre-configured from profile (github enabled, developer disabled)

**Settings Tab** (Screenshots 5-6):
- Model tab: Reset Provider and Model
- Chat tab: Conversation limits, prompt injection detection, response styles
- Session tab: (not shown)
- App tab: (not shown)
- **NEW: Profile tab:**
  - Current profile: Finance Manager
  - Privacy Guard overrides (mode, strictness, categories)
  - Reload profile button

---

### What Admins See (Admin UI - SvelteKit Browser):

**Admin opens browser:**
```
http://localhost:8088/admin
```

**Admin UI Pages:**
1. Dashboard (D3.js org chart, agent status, recent activity)
2. Profiles (Monaco YAML editor, publish button)
3. Org Chart (CSV upload, tree viz)
4. Audit Logs (table, filters, export CSV)
5. Settings (Vault status, system variables, service health)

**Admins interact with browser.** Users interact with Goose Desktop app.

---

## What Needs to Be Built (Corrected Workstreams)

### REMOVED from Phase 6:
- ❌ Workstream C (User UI - SvelteKit) - Not needed, users use Goose Desktop

### ADDED to Phase 6:
- ✅ Workstream C (Goose Desktop Fork) - Real integration

---

### Workstream C: Goose Desktop Fork (5 days) - CRITICAL

**Goal:** Fork Goose Desktop, add:
1. Profile loading from Controller (--profile flag)
2. Privacy Guard HTTP client (mask/unmask)
3. Profile tab in Settings (view profile, privacy overrides)
4. Privacy Guard status indicators (in Chat tab)

**Why Fork?**
- Goose Desktop is open-source (Apache-2.0)
- Upstreaming profile features takes 3-6 months
- Forking lets us move fast, upstream later
- Fork name: `goose-enterprise` (our brand)

**What Gets Modified in Fork:**

**1. CLI Args (src/main.rs):**
```rust
#[derive(Parser)]
struct Cli {
    /// NEW: Profile to load from Controller
    #[arg(long)]
    profile: Option<String>,
    
    /// NEW: Controller URL
    #[arg(long, default_value = "http://localhost:8088")]
    controller_url: String,
    
    // Existing args...
}
```

**2. Profile Loader (NEW: src/enterprise/profile_loader.rs):**
- Fetch profile from Controller
- Download config.yaml, goosehints, gooseignore
- Save to ~/.config/goose/
- Show progress to user

**3. Privacy Guard HTTP Client (NEW: src/enterprise/privacy_guard.rs):**
```rust
pub struct PrivacyGuardClient {
    url: String,  // http://localhost:8089
    session_id: String,
}

impl PrivacyGuardClient {
    pub async fn mask(&self, text: &str) -> Result<MaskResult> {
        // POST /guard/mask
    }
    
    pub async fn reidentify(&self, masked_text: &str, jwt: &str) -> Result<String> {
        // POST /guard/reidentify
    }
}
```

**4. Provider Integration (MODIFY: src/providers/*.rs):**
- Update OpenRouter provider: Add Privacy Guard HTTP calls before/after LLM
- Update OpenAI provider: Same
- Update Anthropic provider: Same
- Update Ollama provider: Same (for consistency)

**5. Profile Settings Tab (NEW: src/ui/profile_tab.rs or .tsx):**
- Show current profile (display_name, role, providers, extensions)
- Privacy Guard overrides (mode, strictness, categories)
- Reload profile button

**6. Privacy Guard Status (MODIFY: src/ui/chat_tab.rs or .tsx):**
- Add status indicator in Chat tab: "🛡️ 3 PII entities masked"
- Show tooltip on hover (EMAIL x2, SSN x1)

---

### Why This is Better Than "Browser UI"

**Goose Desktop UI (from screenshots) already has:**
- ✅ Excellent chat interface (Chat tab)
- ✅ Session history (History tab)
- ✅ Recipe management (Recipes tab - screenshot 3)
- ✅ Scheduler (Scheduler tab)
- ✅ Extension manager (Extensions tab - screenshot 4)
- ✅ Settings (Models, Chat, Session, App - screenshots 5-6)
- ✅ Monaco editor for goosehints (screenshot 2)

**Why rebuild all this in a browser?** Just fork Goose Desktop and add:
- Profile loading (--profile flag)
- Privacy Guard integration (HTTP client)
- Profile tab (view profile, overrides)

**Development Time:**
- Browser UI: 5 days (rebuild everything)
- Fork Goose Desktop: 5 days (add profile + Privacy Guard)

**Same effort, but forking gives better UX** (users get existing Goose features + our additions).

---

## Privacy Guard: HTTP NOT MCP (Clarified)

### Why HTTP Instead of MCP?

**From Phase 5 Testing:**
> "Privacy Guard MCP integration was tested but found too late in pipeline. MCP tools intercept tool calls, not provider calls. PII needs to be masked BEFORE it reaches OpenRouter/Anthropic APIs. Solution: HTTP service that provider code calls directly."

**MCP Architecture (TOO LATE):**
```
User types: "Email john@acme.com"
  ↓
Goose Desktop processes prompt
  ↓
Goose calls MCP tool (e.g., github__create_issue)
  ↓ ← MCP can intercept HERE (but prompt already sent to LLM)
Privacy Guard MCP (too late - LLM already saw PII)
```

**HTTP Architecture (CORRECT):**
```
User types: "Email john@acme.com"
  ↓
Goose Desktop processes prompt
  ↓
Provider code (OpenRouter) → Privacy Guard HTTP (mask first)
  ↓
Privacy Guard returns: "Email EMAIL_7a3f9b"
  ↓
Provider sends to OpenRouter API (masked)
  ↓
OpenRouter → Claude API (sees only EMAIL_7a3f9b)
  ↓
Response: "EMAIL_7a3f9b contacted successfully"
  ↓
Provider → Privacy Guard HTTP (unmask)
  ↓
User sees: "john@acme.com contacted successfully"
```

**Privacy Guard is HTTP service (Phase 5 already built, working, 60/60 tests passing).**

---

## Will the Lightweight UI Have Basics?

### Short Answer: **NO - Users use Goose Desktop app directly**

**What I Mistakenly Proposed:**
- "Lightweight User UI (SvelteKit browser)" with profile viewer, chat, sessions

**Why This Was Wrong:**
- Goose Desktop already has excellent UI (see screenshots)
- Rebuilding in browser is **wasteful** (5 days to duplicate existing features)
- Users expect **desktop app**, not browser (Goose Desktop is flagship UI)

**What Users Actually Get (Goose Desktop):**

**Existing Features (from screenshots):**
- ✅ Chat interface (Screenshot 1 - conversation, send button)
- ✅ Session history (History tab)
- ✅ Recipes (Screenshot 3 - import, run, manage)
- ✅ Scheduler (automated tasks)
- ✅ Extensions (Screenshot 4 - enable/disable MCP extensions)
- ✅ Settings (Screenshots 5-6 - Model, Chat, Session, App tabs)
- ✅ Goosehints editor (Screenshot 2 - Monaco editor)
- ✅ File upload (existing in Goose Desktop)

**NEW Features (Phase 6 fork adds):**
- ✅ **Profile Tab** (view current profile, privacy overrides)
- ✅ **Privacy Guard Status** (in Chat tab - show PII masked count)
- ✅ **Profile Loading** (--profile flag fetches from Controller)
- ✅ **Auto-Configuration** (extensions, recipes, hints, ignore from profile)

**All the basics you asked about are ALREADY IN GOOSE DESKTOP:**
- ✅ Modify privacy guard settings → Settings > Profile tab (NEW)
- ✅ Select MCP extensions → Extensions tab (existing - screenshot 4)
- ✅ Upload files → Chat interface (existing)
- ✅ Upload local goosehints → Screenshot 2 shows editor (existing)
- ✅ Upload local gooseignore → Similar editor (existing)
- ✅ Settings session control → Settings tab (existing - screenshots 5-6)
- ✅ Global goosehint → Profile tab shows profile default, Settings allows local override
- ✅ Global gooseignore → Same pattern
- ✅ Recipes section → Screenshot 3 (existing)
- ✅ Scheduler → Scheduler tab (existing)
- ✅ History → History tab (existing)

**We just need to:**
- Add Profile tab (show profile from Controller)
- Add Privacy Guard overrides (in Profile tab)
- Add Privacy Guard status (in Chat tab)
- Integrate Privacy Guard HTTP client (in provider code)

---

## Revised Phase 6 Scope

### What We Build:

**1. Vault Production (2 days) - Backend**
- TLS, AppRole, Raft, audit, signature verification
- No UI changes

**2. Admin UI (3 days) - Browser**
- 5 pages for IT admins (Dashboard, Profiles, Org Chart, Audit, Settings)
- Browser-based (SvelteKit)
- Admins configure profiles, publish, monitor

**3. Goose Desktop Fork (5 days) - User Experience**
- Fork Goose Desktop (goose-enterprise)
- Add --profile flag (fetch from Controller)
- Add Privacy Guard HTTP client (mask/unmask)
- Add Profile tab (Settings)
- Add Privacy Guard status (Chat tab)
- **Users get Goose Desktop app** (not browser)

**4. Security Hardening (1 day)**
- Secrets cleanup, .env.example, SECURITY.md

**5. Integration Testing (2 days)**
- Vault production tests
- Admin UI smoke tests (Playwright)
- **Goose Desktop E2E test** (critical: profile load → chat → Privacy Guard → LLM)
- Regression tests (Phase 1-5)

**6. Documentation (1 day)**
- Vault guide update
- **Goose Enterprise install guide** (NEW)
- Admin UI guide
- Security guide
- Migration guide

**Total:** 14 days (3 weeks calendar)

---

## Real Working MVP Checklist

**Phase 6 is complete when:**

### User Flow Works:
1. ✅ User runs: `goose-enterprise --profile finance`
2. ✅ Goose prompts for credentials
3. ✅ Goose fetches Finance profile from Controller
4. ✅ Goose Desktop launches with Finance config
5. ✅ User chats: "Email john@acme.com about budget"
6. ✅ Privacy Guard masks: "Email EMAIL_7a3f9b about budget"
7. ✅ OpenRouter sees only masked text
8. ✅ User sees unmasked response (john@acme.com)
9. ✅ Legal profile enforces local-only (Ollama, no cloud)
10. ✅ Extensions auto-configured (github enabled for Finance)

### Admin Flow Works:
1. ✅ Admin opens: http://localhost:8088/admin
2. ✅ Admin logs in (Keycloak OIDC)
3. ✅ Admin edits Finance profile (Monaco YAML editor)
4. ✅ Admin publishes profile (Vault signs)
5. ✅ Admin uploads org chart CSV
6. ✅ Admin views audit logs
7. ✅ Admin monitors Vault status (sealed/unsealed)

### Security Works:
1. ✅ Vault production-ready (TLS, AppRole, Raft, audit)
2. ✅ Profile signatures verified (tamper detection)
3. ✅ No secrets in repo (grep confirms)
4. ✅ Privacy Guard blocks cloud for Legal profile
5. ✅ JWT authentication working

### Testing Works:
1. ✅ 75+ integration tests passing
2. ✅ End-to-end test validates full flow
3. ✅ Performance targets met (P50 < 5s)
4. ✅ No regressions (Phase 1-5 tests pass)

**If all 28 checkboxes above are ✅, you have a REAL WORKING MVP.** ✅

---

## What's Missing from Original Phase 6 Plan?

**Critical Missing Piece:**
- ❌ "Goose Desktop backend mode" (doesn't exist, was a mistake)
- ❌ "User UI (SvelteKit browser)" (not needed, use Goose Desktop)
- ❌ "Privacy Guard MCP" (already HTTP in Phase 5)

**Corrected Plan:**
- ✅ Fork Goose Desktop (goose-enterprise)
- ✅ Add profile loading (--profile flag)
- ✅ Add Privacy Guard HTTP client (provider integration)
- ✅ Add Profile tab (Settings)

**This gives you a REAL MVP** (not just a demo).

---

## How End Users Connect to System

### User Journey (Step-by-Step):

**Week 1: Admin Setup**
1. Admin deploys infrastructure (Docker Compose)
2. Admin creates Finance profile in Admin UI
3. Admin uploads org chart CSV
4. Admin publishes Finance profile (Vault signs)
5. Admin assigns user@company.com → finance role in Keycloak

**Week 2: User Onboarding**
1. IT sends user installation email:
   ```
   Subject: Your Goose Enterprise Setup
   
   Hi [Name],
   
   You've been assigned the Finance Manager profile.
   
   Installation:
   1. Download Goose Enterprise: [link to goose-enterprise binary]
   2. Install: sudo dpkg -i goose-enterprise_0.6.0_amd64.deb
   3. Launch: goose-finance (shortcut)
   4. Sign in with your company email
   
   Your profile includes:
   - Primary LLM: Claude 3.5 Sonnet
   - Extensions: GitHub (budget tracking)
   - Privacy: Strict (all PII masked before cloud)
   - Recipes: Monthly budget close, weekly spend report
   
   Questions? Contact support@company.com
   ```

2. User installs goose-enterprise binary (deb package or .dmg)
3. User runs: `goose-finance` (wrapper for `goose-enterprise --profile finance --controller-url https://controller.company.com`)
4. Goose prompts: "Email: " → User types company email
5. Goose prompts: "Password: " → User types password
6. Goose fetches JWT from Keycloak (role=finance)
7. Goose fetches Finance profile from Controller
8. Goose downloads config.yaml, goosehints, gooseignore
9. Goose saves to ~/.config/goose/
10. **Goose Desktop launches** (GUI app appears - see screenshots)

**Week 2-∞: Daily Usage**
1. User double-clicks "Goose Finance" desktop shortcut
2. Goose Desktop opens (JWT cached in keyring, no re-auth)
3. User clicks "Chat" tab
4. User types: "Analyze Q4 budget for john.doe@acme.com"
5. Goose calls Privacy Guard: POST /guard/mask
6. Privacy Guard returns: "Analyze Q4 budget for EMAIL_7a3f9b"
7. Goose calls Claude API (masked text)
8. Claude responds: "EMAIL_7a3f9b has overspent by 20%"
9. Goose calls Privacy Guard: POST /guard/reidentify
10. Privacy Guard returns: "john.doe@acme.com has overspent by 20%"
11. User sees: "john.doe@acme.com has overspent by 20%" (in Chat tab)

**User NEVER opens a browser for chatting.** They use **Goose Desktop app**.

---

## Architecture Diagram (CORRECTED)

```
┌─────────────────────────────────────────────────────────────────┐
│                         END USERS                                │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Finance    │  │    Legal     │  │      HR      │          │
│  │    User      │  │    User      │  │    User      │          │
│  │              │  │              │  │              │          │
│  │ Uses:        │  │ Uses:        │  │ Uses:        │          │
│  │ Goose        │  │ Goose        │  │ Goose        │          │
│  │ Desktop App  │  │ Desktop App  │  │ Desktop App  │          │
│  │ (GUI)        │  │ (GUI)        │  │ (GUI)        │          │
│  │              │  │              │  │              │          │
│  │ Launch:      │  │ Launch:      │  │ Launch:      │          │
│  │ goose-finance│  │ goose-legal  │  │ goose-hr     │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                  │                  │                  │
└─────────┼──────────────────┼──────────────────┼──────────────────┘
          │                  │                  │
          │ (On startup)     │                  │
          ↓                  ↓                  ↓
┌─────────────────────────────────────────────────────────────────┐
│                    CONTROLLER API                                │
│                    Port: 8088                                    │
│                                                                  │
│  Profile Loading (on startup):                                  │
│  • GET /profiles/finance        → Full profile JSON            │
│  • GET /profiles/finance/config → config.yaml                  │
│  • GET /profiles/finance/goosehints → .goosehints             │
│  • GET /profiles/finance/gooseignore → .gooseignore           │
│                                                                  │
│  Saved to: ~/.config/goose/ (Goose Desktop reads on launch)    │
└─────────────────────────────────────────────────────────────────┘

User chats in Goose Desktop:
┌─────────────────────────────────────────────────────────────────┐
│  User types: "Email john@acme.com about contract"              │
│  (in Goose Desktop Chat tab)                                    │
└──────────────┬──────────────────────────────────────────────────┘
               │
               ↓
┌─────────────────────────────────────────────────────────────────┐
│           GOOSE DESKTOP (Provider Code)                         │
│           NEW: Privacy Guard HTTP Client                        │
│                                                                  │
│  1. Before LLM call:                                            │
│     POST http://localhost:8089/guard/mask                       │
│     {"text": "Email john@acme.com about contract"}            │
│     ↓                                                            │
│     Returns: {"masked_text": "Email EMAIL_7a3f9b about..."}    │
│                                                                  │
│  2. Send to LLM (masked):                                       │
│     POST https://openrouter.ai/api/v1/chat/completions         │
│     {"messages": [{"content": "Email EMAIL_7a3f9b..."}]}       │
│     ↓                                                            │
│     Returns: {"message": "EMAIL_7a3f9b contract sent"}         │
│                                                                  │
│  3. After LLM call:                                             │
│     POST http://localhost:8089/guard/reidentify                │
│     {"masked_text": "EMAIL_7a3f9b contract sent"}              │
│     ↓                                                            │
│     Returns: {"original_text": "john@acme.com contract sent"}  │
│                                                                  │
│  4. Show to user:                                               │
│     Display in Chat tab: "john@acme.com contract sent"         │
└─────────────────────────────────────────────────────────────────┘

Admin uses browser:
┌─────────────────────────────────────────────────────────────────┐
│                 ADMIN USER                                       │
│                                                                  │
│  Opens browser: http://localhost:8088/admin                    │
│  Uses Admin UI (SvelteKit - 5 pages)                           │
│                                                                  │
│  Can:                                                            │
│  • Edit profiles (Monaco YAML editor)                           │
│  • Publish profiles (Vault signing)                             │
│  • Upload org chart CSV                                          │
│  • View audit logs                                               │
│  • Monitor Vault status                                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Summary: Two User Experiences

### End Users (Finance, Legal, HR, etc.):
- **Interface:** Goose Desktop app (Tauri GUI - see screenshots)
- **Launch:** `goose-enterprise --profile finance`
- **Features:** Chat, History, Recipes, Scheduler, Extensions, Settings, **Profile** (new)
- **Privacy:** Privacy Guard HTTP (mask before cloud LLM)
- **Configuration:** Auto-loaded from Controller (no manual setup)

### Admins (IT, Security, Ops):
- **Interface:** Admin UI (SvelteKit browser - http://localhost:8088/admin)
- **Features:** Dashboard, Profiles, Org Chart, Audit, Settings
- **Tasks:** Create profiles, publish (Vault sign), upload org chart, monitor

**Two different UIs for two different user types.** ✅

---

## What Makes This a Real MVP (Not Just Demo)?

### Real MVP Must Have:
1. ✅ **Production-ready Vault** (TLS, AppRole, Raft, audit, verify)
2. ✅ **Profile auto-loading** (--profile flag fetches from Controller)
3. ✅ **Privacy Guard working** (HTTP service, mask before cloud)
4. ✅ **Admin UI functional** (profile editor, org chart, Vault monitor)
5. ✅ **Security hardened** (no secrets in repo, environment audit)
6. ✅ **End-to-end tested** (user signs in → loads profile → chats → PII masked → LLM → unmasked)
7. ✅ **Deployable** (Docker Compose + Goose Desktop binary)

**Phase 6 delivers ALL 7 of these.** ✅

**Demo vs MVP:**
- Demo: Mock data, hardcoded values, no authentication, "works on my machine"
- **MVP**: Real data, real auth, real privacy protection, real Vault, deployable, tested

**Phase 6 is MVP, not demo.** ✅

---

## Final Answer to Your Questions

### Question 1: Do I have real working MVP by end of Phase 6?

**YES, with revised scope:**
- Add 4 days for Goose Desktop fork (14 days total)
- Replace "User UI (browser)" with "Goose Desktop Fork"
- Keep Vault production, Admin UI, Security, Testing, Docs

**What makes it real:**
- Profile loading works (--profile flag)
- Privacy Guard works (HTTP integration in providers)
- Vault production works (TLS, AppRole, Raft, audit, verify)
- Admin UI works (profile editor, org chart, monitoring)
- End-to-end tested (75+ tests passing)

**This is a production-ready system you can deploy and use.** ✅

---

### Question 2: Explain User Browser → Goose architecture?

**Corrected Explanation:**

**Users do NOT use a browser for chatting.** They use **Goose Desktop app** (Tauri GUI).

**Correct Architecture:**
```
End User → Goose Desktop App (GUI - see screenshots)
  ↓ (on startup)
Controller HTTP (fetch profile)
  ↓ (on each chat message)
Privacy Guard HTTP (mask PII)
  ↓
Cloud LLMs (OpenRouter/Anthropic/OpenAI)
  ↓ (response)
Privacy Guard HTTP (unmask PII)
  ↓
Goose Desktop App (display to user)
```

**Admin → Browser:**
```
Admin → Browser (http://localhost:8088/admin)
  ↓
Admin UI (SvelteKit - 5 pages)
  ↓
Controller HTTP (create/edit/publish profiles)
```

**Two separate UIs:**
- End Users: Goose Desktop app (fork with profile loading)
- Admins: Browser (SvelteKit Admin UI)

---

**Does this clarify the architecture?** ✅
