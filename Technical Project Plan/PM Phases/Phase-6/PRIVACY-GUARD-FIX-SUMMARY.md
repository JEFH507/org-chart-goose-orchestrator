# Privacy Guard Per-Instance Fix - Summary

**Date**: 2025-11-11 19:00
**Status**: ✅ Code is READY - Build & Test Required

---

## What Was the Issue?

Your summary mentioned:
> "We need to fix this and make sure each goose instance/profile will receive this privacy guard proxy and the service with the correct updates"

---

## What's ALREADY Done ✅

### 1. Code Implementation (100% Complete)

All code is already implemented correctly:

**✅ Privacy Guard Service** (`src/privacy-guard/src/main.rs`)
- Accepts `detection_method` and `privacy_mode` via API parameters
- Uses user settings to control PII detection method (rules/hybrid/ai)
- Logs user settings for verification

**✅ Privacy Guard Proxy** (`src/privacy-guard-proxy/src/masking.rs` + `src/proxy.rs`)
- Converts enum settings to strings
- Passes user settings to Privacy Guard Service
- Maintains per-instance state (Finance, Manager, Legal)

**✅ Docker Compose** (`deploy/compose/ce.dev.yml`)
- 3 Ollama instances (Finance: 11435, Manager: 11436, Legal: 11437)
- 3 Privacy Guard Services (Finance: 8093, Manager: 8094, Legal: 8095)
- 3 Privacy Guard Proxies (Finance: 8096, Manager: 8097, Legal: 8098)
- 3 goose instances (each with own workspace volume)

**✅ Control Panel Flow**
```
User changes Control Panel UI
    ↓
PUT /api/settings → ProxyState updated
    ↓
goose sends request → Proxy (with settings)
    ↓
Proxy calls Service with detection_method + privacy_mode
    ↓
Service uses user-selected settings (Rules/Hybrid/AI)
```

---

## What Needs to Be Done 🔨

### Step 1: Build Updated Docker Images

The code is ready, but the Docker images need to be built with the latest changes:

```bash
# Build Privacy Guard Service (version 0.2.0 with API parameter support)
cd /home/papadoc/Gooseprojects/goose-org-twin/src/privacy-guard
docker build -t ghcr.io/jefh507/privacy-guard:0.2.0 .

# Build Privacy Guard Proxy (version 0.3.0 with settings forwarding)
cd /home/papadoc/Gooseprojects/goose-org-twin/src/privacy-guard-proxy
docker build -t ghcr.io/jefh507/privacy-guard-proxy:0.3.0 .
```

### Step 2: Update Docker Compose Image Versions

Edit `deploy/compose/ce.dev.yml` and update image versions:

**Find:**
```yaml
privacy-guard-finance:
  image: ghcr.io/jefh507/privacy-guard:0.1.0
  
privacy-guard-proxy-finance:
  image: ghcr.io/jefh507/privacy-guard-proxy:0.2.0
```

**Replace with:**
```yaml
privacy-guard-finance:
  image: ghcr.io/jefh507/privacy-guard:0.2.0  # ← NEW VERSION
  
privacy-guard-proxy-finance:
  image: ghcr.io/jefh507/privacy-guard-proxy:0.3.0  # ← NEW VERSION
```

Do the same for Manager and Legal instances.

### Step 3: Restart Services

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Stop current services
docker compose -f deploy/compose/ce.dev.yml --profile multi-goose down

# Start with new images
docker compose -f deploy/compose/ce.dev.yml --profile multi-goose up -d

# Verify all services are running
docker ps | grep -E "privacy-guard|ollama|goose"
```

Expected output: **12 containers** running
- 3 Ollama (finance, manager, legal)
- 3 Privacy Guard Services (finance, manager, legal)
- 3 Privacy Guard Proxies (finance, manager, legal)
- 3 goose instances (finance, manager, legal)

### Step 4: Test the Flow

**Test 1: Finance Control Panel (Rules Mode)**

```bash
# 1. Open Control Panel in browser
open http://localhost:8096/ui

# 2. Change detection method to "Rules Only"
curl -X PUT http://localhost:8096/api/settings \
  -H "Content-Type: application/json" \
  -d '{"routing":"service","detection":"rules","privacy":"auto"}'

# 3. Send test request
curl -X POST http://localhost:8096/v1/chat/completions \
  -H "Authorization: Bearer $OPENROUTER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Contact john@example.com"}]}'

# 4. Check logs (should show "Using rules-only detection")
docker logs ce_privacy_guard_finance 2>&1 | grep "detection_method"
```

**Expected Log:**
```
INFO privacy_guard: Received mask request with user settings detection_method="rules"
INFO privacy_guard: Using rules-only detection (fast ~10ms)
```

**Test 2: Manager Control Panel (Hybrid Mode)**

```bash
open http://localhost:8097/ui

curl -X PUT http://localhost:8097/api/settings \
  -d '{"routing":"service","detection":"hybrid","privacy":"auto"}'

docker logs ce_privacy_guard_manager 2>&1 | grep "detection_method"
```

**Expected Log:**
```
INFO privacy_guard: Received mask request with user settings detection_method="hybrid"
INFO privacy_guard: Using hybrid/AI detection (balanced ~100ms)
```

**Test 3: Legal Control Panel (AI Mode)**

```bash
open http://localhost:8098/ui

curl -X PUT http://localhost:8098/api/settings \
  -d '{"routing":"service","detection":"ai","privacy":"strict"}'

docker logs ce_privacy_guard_legal 2>&1 | grep "detection_method"
```

**Expected Log:**
```
INFO privacy_guard: Received mask request with user settings detection_method="ai"
INFO privacy_guard: Using hybrid/AI detection (accurate ~15s)
```

---

## How It Works (Per-Instance Architecture)

### Current Setup

```
┌─────────────────────────────────────────────────┐
│           FINANCE GOOSE INSTANCE                 │
├─────────────────────────────────────────────────┤
│ goose Finance                                    │
│    ↓                                             │
│ Proxy Finance (port 8096)                        │
│    ├─ Control Panel: http://localhost:8096/ui   │
│    └─ Settings: Rules-only (default)            │
│         ↓                                        │
│ Privacy Guard Finance (port 8093)                │
│    └─ Ollama Finance (port 11435)               │
│       - GUARD_MODEL_ENABLED=false (rules-only)  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│           MANAGER GOOSE INSTANCE                 │
├─────────────────────────────────────────────────┤
│ goose Manager                                    │
│    ↓                                             │
│ Proxy Manager (port 8097)                        │
│    ├─ Control Panel: http://localhost:8097/ui   │
│    └─ Settings: Hybrid (default)                │
│         ↓                                        │
│ Privacy Guard Manager (port 8094)                │
│    └─ Ollama Manager (port 11436)               │
│       - GUARD_MODEL_ENABLED=true (hybrid mode)  │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│            LEGAL GOOSE INSTANCE                  │
├─────────────────────────────────────────────────┤
│ goose Legal                                      │
│    ↓                                             │
│ Proxy Legal (port 8098)                          │
│    ├─ Control Panel: http://localhost:8098/ui   │
│    └─ Settings: AI-only (default)               │
│         ↓                                        │
│ Privacy Guard Legal (port 8095)                  │
│    └─ Ollama Legal (port 11437)                 │
│       - GUARD_MODEL_ENABLED=true (AI mode)      │
└─────────────────────────────────────────────────┘
```

### Key Points

1. **Complete Isolation**: Each goose instance has its own:
   - Ollama model (separate CPU instance)
   - Privacy Guard Service (independent PII masking)
   - Privacy Guard Proxy (independent Control Panel)
   - Workspace volume (isolated file storage)

2. **User Control**: Each instance has its own Control Panel where users can change:
   - **Routing Mode**: Service (default) vs Bypass
   - **Detection Method**: Rules (fast) / Hybrid (balanced) / AI (thorough)
   - **Privacy Mode**: Auto (default) / Service-Bypass (dev) / Strict (production)

3. **No Blocking**: 
   - Finance using Rules (~10ms) doesn't wait for Legal using AI (~15s)
   - Each instance processes independently
   - No shared state between instances

---

## Quick Port Reference

| Service | Finance | Manager | Legal |
|---------|---------|---------|-------|
| **Ollama** | 11435 | 11436 | 11437 |
| **Privacy Guard** | 8093 | 8094 | 8095 |
| **Proxy** | 8096 | 8097 | 8098 |
| **Control Panel** | 8096/ui | 8097/ui | 8098/ui |

---

## Success Criteria

✅ Build completes without errors
✅ 12 containers running (docker ps)
✅ All 3 Control Panels accessible (8096/ui, 8097/ui, 8098/ui)
✅ Settings changes reflected in logs
✅ Each instance isolated (Finance settings ≠ Manager settings)

---

## Next Steps After Testing

1. **Update Phase 6 State Files**
   - Mark D.4.2 complete in `Phase-6-Agent-State.json`
   - Update `Phase-6-Checklist.md`
   - Append to `docs/tests/phase6-progress.md`

2. **Proceed to Admin Dashboard** (Admin.1-2)
   - Minimal HTML/JS admin interface
   - CSV upload for org chart
   - User/profile assignment
   - Config push to goose instances

3. **Create Demo Validation Script** (Demo.1)
   - 6-window layout documentation
   - Manual testing workflow
   - Expected outputs

---

## Documentation Reference

- **Full Implementation Details**: `docs/tests/PRIVACY-GUARD-PER-INSTANCE-COMPLETE.md`
- **UI→Service Integration**: `docs/tests/D4-UI-TO-SERVICE-COMPLETE.md`
- **Phase 6 MVP Scope**: `Technical Project Plan/PM Phases/Phase-6/PHASE-6-MVP-SCOPE.md`

---

## TL;DR

**The code is ready! You just need to:**

1. Build 2 Docker images (privacy-guard:0.2.0, privacy-guard-proxy:0.3.0)
2. Update ce.dev.yml image versions
3. Restart services (`docker compose --profile multi-goose up -d`)
4. Test with Control Panels (3 browser tabs: 8096/ui, 8097/ui, 8098/ui)

**Time Estimate**: 30 minutes (build + test)

**Questions?** Ask me to help with any specific step!
