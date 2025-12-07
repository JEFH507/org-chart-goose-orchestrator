# Phase 6 Critical Decisions Document

**Date:** 2025-11-07  
**Status:** DECISION REQUIRED BEFORE PHASE 6 START  
**Stakeholder:** Javier (JEFH507)  
**Purpose:** Make informed decisions on Privacy Guard integration and Phase 6 scope

---

## 🎯 Current Situation

### What We Have (Phase 5 Complete ✅)
- **Controller API:** 15 endpoints (profiles, org chart, audit, sessions)
- **Privacy Guard Service:** HTTP API (port 8089, regex + NER, deterministic masking)
- **6 Role Profiles:** Stored in Postgres, Vault HMAC-signed
- **Agent Mesh MCP:** Multi-agent coordination (Python, 977 lines)
- **Vault (Dev Mode):** HTTP, root token, in-memory
- **Database:** Postgres (profiles, org_users, audit logs)
- **All Tests Passing:** 50/50 integration tests ✅

### What We Need (Phase 6 Goals)
1. **Users can access the system** → Sign in before using interfaces
2. **Profiles auto-load** → User signs in → Config pushed to local setup
3. **PII protection works** → LLM never sees raw PII
4. **Production-ready** → Vault hardened, security audit complete
5. **Fully integrated MVP** → Not just a demo, actual working system

---

## 🔴 Critical Decision #1: Privacy Guard Integration Method

**The Core Problem:**
```
Current goose Desktop Flow:
  User types: "My SSN is 123-45-6789"
    ↓
  goose sends to OpenRouter API
    ↓
  ⚠️ LLM SEES RAW PII (Too late!)
    ↓
  LLM decides to call Privacy Guard MCP tool
    ↓
  Privacy Guard masks (but LLM already saw PII)
```

**Root Cause:** MCP tools are called BY the LLM, not BEFORE the LLM sees user input.

**Business Requirement:** Enterprise users need PII protection WITHOUT expensive local LLM hardware.

---

### Option A: Privacy Guard Proxy Server ⭐ FASTEST PATH

**What It Is:**
- HTTP proxy server (localhost:8090) intercepts LLM requests
- goose Desktop config change: `GOOSE_PROVIDER__OPENROUTER_BASE_URL: http://localhost:8090`
- Proxy masks PII → forwards to real OpenRouter → unmasks response

**Architecture:**
```
goose Desktop → Privacy Guard Proxy (localhost:8090) → OpenRouter
                 ↑ MASKS HERE                          ↑ Only sees masked
```

**Implementation:**
- **Language:** TypeScript/Node.js (Express server)
- **Lines:** ~500 TypeScript
- **Files:** `src/privacy-guard-proxy/` (new directory)
- **Docker:** New service in docker-compose

**Pros:**
- ✅ No goose fork needed
- ✅ Works with current goose Desktop (just config change)
- ✅ Fast to implement (1-2 days coding, 1 day testing)
- ✅ Toggleable (change URL to disable)
- ✅ Transparent UX (user doesn't notice)
- ✅ LLM never sees raw PII ✅

**Cons:**
- ⚠️ Adds latency (~50-200ms per request)
- ⚠️ Requires running separate service (systemd/Docker)
- ⚠️ Need to handle multiple providers (OpenRouter, Anthropic, OpenAI, Ollama)
- ⚠️ Token store must be memory-only (security concern)

**Effort:** 1-2 weeks total (coding + testing + docs)

---

### Option B: goose Desktop Fork with Privacy Layer ⭐ BEST UX

**What It Is:**
- Fork `block/goose` → `JEFH507/goose-enterprise`
- Modify Rust provider code to call Privacy Guard HTTP API BEFORE sending to LLM
- Add Privacy Guard settings in goose UI

**Architecture:**
```
User Input → Privacy Guard Client (Rust HTTP call) → Masked Text → OpenRouter
             ↑ MASKS IN GOOSE CODE                    ↑ Only sees masked
```

**Implementation:**
- **Fork:** https://github.com/block/goose → https://github.com/JEFH507/goose-enterprise
- **Language:** Rust (modify src/providers/)
- **Lines:** ~1,000 Rust (Privacy Guard client + provider integration + UI)
- **Files:** 
  - `src/enterprise/privacy_guard.rs` (HTTP client)
  - `src/providers/openrouter.rs` (MODIFY: add mask/unmask)
  - `src/ui/profile_settings.rs` (NEW: Privacy Guard tab)

**Pros:**
- ✅ Clean integration (no proxy)
- ✅ Better UX (in-app PII notifications, settings)
- ✅ No separate service to run
- ✅ Can show PII detection in real-time
- ✅ LLM never sees raw PII ✅

**Cons:**
- ❌ Requires fork maintenance (merge upstream changes monthly)
- ❌ Need Rust expertise (modify providers)
- ❌ Delayed updates from upstream (must wait for merges)
- ⚠️ Fork becomes "your problem" to maintain

**Effort:** 2-3 weeks (fork setup + coding + testing + docs)

---

### Option C: Standalone UI Client ("goose Enterprise") 🚫 TOO MUCH WORK

**What It Is:**
- Build entirely new desktop app (Electron or Tauri)
- Embed Privacy Guard as built-in feature
- Use goose CLI as backend (subprocess)

**Pros:**
- ✅ Full control over features
- ✅ Can bundle Privacy Guard + goose together
- ✅ Custom branding

**Cons:**
- ❌ **High development effort (4-6 weeks)**
- ❌ Need to reimplement goose Desktop UI from scratch
- ❌ Slower to market
- ❌ Full maintenance burden

**Recommendation:** ❌ NOT RECOMMENDED (too much work for MVP)

---

### Option D: CLI Wrapper Script ⚡ VALIDATION ONLY

**What It Is:**
- Bash script wraps `goose` CLI
- Prompts user → Calls Privacy Guard → Passes masked text to goose CLI

**Pros:**
- ✅ Works immediately (1 day to build)
- ✅ Perfect for concept validation

**Cons:**
- ❌ CLI-only (no GUI)
- ❌ Poor UX (manual workflow)
- ❌ Not production-ready

**Recommendation:** ✅ Use for QUICK VALIDATION this week, then choose Option A or B for production

---

### Decision Matrix

| Criteria | Option A (Proxy) | Option B (Fork) | Option C (Standalone) | Option D (CLI) |
|----------|------------------|-----------------|----------------------|----------------|
| **Time to MVP** | 1-2 weeks | 2-3 weeks | 4-6 weeks | 1 day (not production) |
| **No goose Fork?** | ✅ Yes | ❌ No | ✅ Yes | ✅ Yes |
| **LLM Protected?** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes (CLI only) |
| **Maintenance** | Low | Medium | High | None (throwaway) |
| **User Experience** | Good | Excellent | Excellent | Poor |
| **Complexity** | Low | Medium | High | Very Low |
| **Latency** | +50-200ms | +0ms (integrated) | +0ms | N/A |

---

## 📊 RECOMMENDED DECISION PATH

### Phase 1: Validation (This Week) ⚡
**Use Option D: CLI Wrapper**
- Build `scripts/privacy-goose.sh` (1 day)
- Test with real PII scenarios
- Verify Privacy Guard concept works
- **Decision Point:** Does masking protect PII effectively?

### Phase 2: Production Approach (Choose ONE)

#### If you choose SPEED → Option A (Proxy)
- Implement `src/privacy-guard-proxy/` (1 week)
- Deploy as Docker service
- Update goose Desktop config
- **Timeline:** 2 weeks to production MVP

#### If you choose QUALITY → Option B (Fork)
- Fork goose Desktop (1 day setup)
- Implement Privacy Guard integration (1.5 weeks)
- Add Profile Settings tab (1 day)
- **Timeline:** 3 weeks to production MVP

---

## 🔴 Critical Decision #2: User Authentication & Profile Loading

**The Requirement:**
> "Route users to sign in before they can access the actual interfaces, then their profiles will be pushed to their local setup."

### Current Architecture (Phase 5)
```
Controller API (JWT-protected)
  ├─ POST /profiles/{role}        ← Requires JWT
  ├─ GET /profiles/{role}/config  ← Generates config.yaml
  ├─ GET /profiles/{role}/goosehints
  └─ GET /profiles/{role}/gooseignore
```

### How goose Desktop Works Today
```
1. User launches goose Desktop
2. goose reads ~/.config/goose/config.yaml (local file)
3. No remote profile loading built-in
```

### Problem
**goose Desktop does NOT have:**
- Login screen
- Profile fetching from remote Controller
- JWT token management

**Current goose Desktop expects:**
- Local config.yaml file already exists
- User manually configured providers/extensions

---

### Solution Options for Profile Loading

#### Option 1: CLI Flag (Minimal, Works with Fork)
**How it works:**
```bash
# User runs:
goose-enterprise --profile finance --controller-url http://localhost:8088

# goose prompts:
Email: user@company.com
Password: ********

# goose:
1. Gets JWT from Keycloak
2. Calls GET /profiles/finance (with JWT)
3. Saves config.yaml to ~/.config/goose/
4. Launches goose with Finance profile
```

**Implementation:**
- Modify goose Desktop main.rs (add --profile flag)
- Add JWT auth helper (get token from Keycloak)
- Add Profile fetcher (HTTP client for Controller API)
- Save config files to ~/.config/goose/

**Pros:**
- ✅ Simple UX: `goose-finance` (wrapper script)
- ✅ Works with fork approach
- ✅ User sees login prompt, then goose loads

**Cons:**
- ⚠️ Requires goose Desktop fork (Option B from Decision #1)
- ⚠️ No GUI login (terminal prompt for credentials)

---

#### Option 2: Pre-Load Script (No Fork Needed)
**How it works:**
```bash
# User runs:
./scripts/load-profile.sh finance

# Script:
1. Prompts for credentials
2. Gets JWT from Keycloak
3. Calls Controller API
4. Saves config.yaml, .goosehints, .gooseignore
5. Launches: goose session start

# User sees goose Desktop with Finance profile loaded
```

**Implementation:**
- Create `scripts/load-profile.sh` (Bash script)
- Use curl to call Controller API
- Save files to ~/.config/goose/
- Launch goose

**Pros:**
- ✅ Works with current goose Desktop (no fork)
- ✅ Works with Option A (Proxy approach)
- ✅ Fast to implement (1 day)

**Cons:**
- ⚠️ Separate script (not integrated UX)
- ⚠️ User must run script before goose

---

#### Option 3: Admin Pre-Provision (Simplest)
**How it works:**
```bash
# Admin runs on user's machine:
curl -H "Authorization: Bearer $ADMIN_JWT" \
  http://localhost:8088/profiles/finance/config \
  > /home/user/.config/goose/config.yaml

curl -H "Authorization: Bearer $ADMIN_JWT" \
  http://localhost:8088/profiles/finance/goosehints \
  > /home/user/.config/goose/.goosehints

curl -H "Authorization: Bearer $ADMIN_JWT" \
  http://localhost:8088/profiles/finance/gooseignore \
  > /home/user/.config/goose/.gooseignore

# User launches goose:
goose session start
```

**Pros:**
- ✅ No goose fork needed
- ✅ Works immediately with current goose
- ✅ Admin control over profiles

**Cons:**
- ❌ Admin must manually provision each user
- ❌ No self-service for users
- ❌ Not scalable (manual work per user)

---

### Decision Matrix: Profile Loading

| Approach | goose Fork? | Self-Service? | UX | Effort |
|----------|-------------|---------------|-----|--------|
| **Option 1: CLI Flag** | ✅ Required | ✅ Yes | Good (terminal login) | 5 days |
| **Option 2: Pre-Load Script** | ❌ No | ✅ Yes | Fair (separate script) | 1 day |
| **Option 3: Admin Provision** | ❌ No | ❌ No | Good (user just opens goose) | 1 hour |

---

## 🎯 RECOMMENDED COMBINED DECISION

### Scenario A: Speed to Market (4 weeks total)

**Privacy Guard:** Option A (Proxy)  
**Profile Loading:** Option 2 (Pre-Load Script)

**What user does:**
```bash
# One-time setup:
./scripts/load-profile.sh finance
# Prompts for email/password
# Downloads config from Controller
# Saves to ~/.config/goose/

# Daily usage:
goose session start
# goose Desktop with Finance profile loaded
# Privacy Guard protects PII (via proxy)
```

**Timeline:**
- Week 1: Privacy Guard Proxy (5 days)
- Week 2: Profile Load Script + Testing (3 days) + Vault Production (2 days)
- Week 3: Admin UI (5 days)
- Week 4: Security Hardening + Docs (5 days)

**Result:** Production MVP in 4 weeks, no goose fork

---

### Scenario B: Best User Experience (6 weeks total)

**Privacy Guard:** Option B (Fork)  
**Profile Loading:** Option 1 (CLI Flag)

**What user does:**
```bash
# Daily usage:
goose-enterprise --profile finance
# Prompts for credentials (first time)
# Loads Finance profile from Controller
# Privacy Guard integrated
# Perfect UX ✨
```

**Timeline:**
- Week 1-2: goose Desktop Fork + Privacy Guard integration (10 days)
- Week 3: Profile Loading + Testing (5 days)
- Week 4: Vault Production + Admin UI (5 days)
- Week 5: Security Hardening + Docs (5 days)
- Week 6: Final testing + deployment (5 days)

**Result:** Production MVP in 6 weeks, excellent UX

---

## 📋 Questions to Answer

### For Privacy Guard Decision:
1. **What's more important:** Speed (4 weeks) or UX quality (6 weeks)?
2. **Are you comfortable maintaining a goose fork?** (monthly upstream merges)
3. **Is 50-200ms proxy latency acceptable?** (for Option A)
4. **Do you have Rust expertise** on the team? (for Option B)

### For Profile Loading Decision:
1. **Do users need self-service** or is admin provisioning okay for MVP?
2. **Is terminal login acceptable** or do you need GUI login?
3. **How many users will use the system?** (scalability concern)

### General:
1. **What's the target launch date?** (determines timeline)
2. **What's the priority:** Enterprise features or community OSS?
3. **Budget for development time?** (4 weeks vs 6 weeks)

---

## 🚀 Recommendation: START WITH VALIDATION

**This Week (Recommended):**
1. Build Option D: CLI Wrapper Script (1 day)
2. Test Privacy Guard integration (1 day)
3. Validate concept with stakeholders (1 day)
4. Make decision on Option A vs B (based on results)

**Script to Build:**
```bash
#!/bin/bash
# scripts/privacy-goose-validate.sh

PRIVACY_GUARD_URL="http://localhost:8089"

echo "Enter your message:"
read -r USER_INPUT

# Scan for PII
SCAN=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
  -H "Content-Type: application/json" \
  -d "{\"text\":\"$USER_INPUT\",\"mode\":\"hybrid\"}")

PII_COUNT=$(echo "$SCAN" | jq '.detections | length')

if [ "$PII_COUNT" -gt 0 ]; then
  echo "⚠️ Detected $PII_COUNT PII items. Masking..."
  
  MASK=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"$USER_INPUT\",\"method\":\"pseudonym\"}")
  
  MASKED_TEXT=$(echo "$MASK" | jq -r '.masked_text')
  echo "🔒 Masked: $MASKED_TEXT"
  
  USER_INPUT="$MASKED_TEXT"
fi

# Send to goose
echo "$USER_INPUT" | goose session start
```

**Test:**
```bash
./scripts/privacy-goose-validate.sh

# Input: My SSN is 123-45-6789
# Output:
# ⚠️ Detected 1 PII items. Masking...
# 🔒 Masked: My SSN is SSN_a1b2c3d4
# [goose starts with masked text]
```

**Decision Point:** If this works well → Proceed with full implementation

---

## 📅 Next Steps

1. **Review this document** with stakeholders
2. **Answer decision questions** (above)
3. **Choose approach:**
   - Scenario A (Speed): Privacy Proxy + Pre-Load Script
   - Scenario B (Quality): goose Fork + CLI Flag
4. **Build validation script** (Option D, this week)
5. **Test Privacy Guard concept** with real PII
6. **Make final decision** based on validation results
7. **Update Phase 6 plan** with chosen approach
8. **Start implementation**

---

## 📚 Related Documents

- Privacy Guard Options: `docs/decisions/privacy-guard-llm-integration-options.md`
- Current Phase 6 Draft: `Technical Project Plan/PM Phases/Phase-6/REVISED-SCOPE.md`
- Architecture Audit: `docs/architecture/SRC-ARCHITECTURE-AUDIT.md`
- Master Plan: `Technical Project Plan/master-technical-project-plan.md`

---

**Status:** ⏸️ AWAITING DECISION  
**Owner:** Javier (JEFH507)  
**Deadline:** Before Phase 6 start  
**Impact:** High (affects next 4-6 weeks of work)
