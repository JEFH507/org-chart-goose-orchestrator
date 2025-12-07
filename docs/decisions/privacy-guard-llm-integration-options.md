# Privacy Guard LLM Integration Options

**Date:** 2025-11-06  
**Status:** Decision Pending  
**Context:** Phase 5 H6 - Privacy Guard integration with goose  

---

## Problem Statement

**Current Architecture Flaw:**
```
User types "My SSN is 123-45-6789"
    ↓
goose sends to LLM (OpenRouter) ← ⚠️ PII LEAKED
    ↓
LLM decides to call Privacy Guard MCP tool
    ↓
Privacy Guard masks PII
    ↓
Too late - LLM already saw raw PII
```

**Root Cause:** MCP tools are called BY the LLM, not BEFORE the LLM sees user input.

**Your Insight:**
> "Note: I am guessing here is where we were wrong: OpenRouter → Privacy Guard MCP as is not goose 
> the one that reads the message and send to mcp, but goose send message to llm, and the the llm 
> use tool calling for calling the mcp...so at that point the llm already saw the unedited message."

**Business Requirement:** Enterprise users need PII protection WITHOUT requiring expensive local LLM hardware.

---

## Solution Options Summary

| # | Solution | Effort | Time | Fork? | Protects PII? | Production? |
|---|----------|--------|------|-------|---------------|-------------|
| 1 | **Privacy Guard Proxy** | Low | 1-2 weeks | ✅ No | ✅ Yes | ⚠️ Beta |
| 2 | **goose Desktop Fork** | Med | 2-3 weeks | ❌ Yes | ✅ Yes | ✅ Yes |
| 3 | **Standalone UI Client** | High | 4-6 weeks | ✅ No | ✅ Yes | ✅ Yes |
| 4 | **CLI Wrapper (Validation)** | Low | 1 day | ✅ No | ✅ Yes | ❌ No |
| 5 | **HTTP API Only** | Low | Done | ✅ No | ❌ NO | ❌ No |

---

## Option 1: Privacy Guard Proxy Server ⭐ RECOMMENDED FOR QUICK WIN

### Architecture
```
User Input
    ↓
goose Desktop
    ↓
[Privacy Guard Proxy] (localhost:8090) ← INTERCEPTS HTTP HERE
    ↓ (scans → masks → forwards)
OpenRouter API (only sees masked text: "My SSN is SSN_a1b2c3d4")
    ↓
LLM processes masked version
    ↓
Response → Proxy → Unmasks tokens → User sees real data
```

### How It Works
1. **User types:** "My SSN is 123-45-6789"
2. **goose Desktop sends** HTTP POST to http://localhost:8090/api/v1/chat/completions (proxy, not OpenRouter)
3. **Proxy intercepts:**
   - Calls Privacy Guard: `POST localhost:8089/guard/scan` → detects SSN
   - Calls Privacy Guard: `POST localhost:8089/guard/mask` → gets "My SSN is SSN_a1b2c3d4"
   - Stores mapping: `{session_id: {..., "SSN_a1b2c3d4": "123-45-6789"}}`
4. **Proxy forwards** masked text to OpenRouter
5. **OpenRouter/LLM** never sees real SSN ✅
6. **Response comes back** from LLM
7. **Proxy unmasks** any tokens in response
8. **User sees** real data (transparent)

### Implementation

**Files:**
```
src/privacy-guard-proxy/
├── src/
│   ├── server.ts              # Express server (main intercept logic)
│   ├── privacy-client.ts      # HTTP client for Privacy Guard API
│   ├── token-store.ts         # In-memory token mapping
│   ├── config.ts              # Configuration
│   └── types.ts               # TypeScript interfaces
├── package.json
├── tsconfig.json
├── Dockerfile
└── README.md
```

**Core Logic:**
```typescript
// src/server.ts
import express from 'express';
import { PrivacyGuardClient } from './privacy-client';
import { TokenStore } from './token-store';

const app = express();
const privacy = new PrivacyGuardClient('http://localhost:8089');
const tokens = new TokenStore();

app.post('/api/v1/chat/completions', async (req, res) => {
  const { messages } = req.body;
  const userMsg = messages[messages.length - 1].content;
  
  // Scan for PII
  const scan = await privacy.scan(userMsg);
  
  if (scan.detections.length > 0) {
    // Mask PII
    const masked = await privacy.mask(userMsg);
    const sessionId = masked.session_id;
    
    // Store tokens for unmask
    tokens.store(sessionId, masked.redactions);
    
    // Replace message
    messages[messages.length - 1].content = masked.masked_text;
    
    // Forward to OpenRouter
    const llmResp = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: req.headers,
      body: JSON.stringify({ ...req.body, messages })
    });
    
    // Unmask response
    let content = llmResp.data.choices[0].message.content;
    content = tokens.unmask(sessionId, content);
    
    // Clean up
    tokens.delete(sessionId);
    
    res.json({ ...llmResp.data, choices: [{ message: { content }}]});
  } else {
    // No PII, pass through
    const llmResp = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: req.headers,
      body: JSON.stringify(req.body)
    });
    res.json(llmResp.data);
  }
});

app.listen(8090);
```

**goose Configuration:**
```yaml
# ~/.config/goose/config.yaml
GOOSE_PROVIDER__OPENROUTER_BASE_URL: http://localhost:8090/api/v1
PRIVACY_GUARD_ENABLED: true
```

### Pros
- ✅ No goose fork needed
- ✅ Works with current goose Desktop
- ✅ Fast to implement (1-2 days coding)
- ✅ Toggleable (change URL to disable)
- ✅ Transparent UX
- ✅ LLM never sees raw PII

### Cons
- ⚠️ Adds latency (~50-200ms per request)
- ⚠️ Requires running separate service
- ⚠️ Need to handle multiple LLM providers (OpenRouter, Anthropic, OpenAI)
- ⚠️ Token store must be memory-only (security)

### Validation
- Use existing 50/50 integration tests
- Add proxy-specific tests for mask/unmask
- Benchmark latency impact
- Security audit of token storage

**Effort:** 1-2 weeks  
**Maintenance:** Low (stable API)

---

## Option 2: goose Desktop Fork with Privacy Layer

### Architecture
```
User Input (ChatInput.tsx)
    ↓
[Privacy Guard Hook] ← INTERCEPTS IN UI CODE
    ↓ (scans → masks before submit)
Masked Text → goose Backend → OpenRouter
    ↓
LLM processes masked version (never sees PII)
    ↓
Response → Unmask → Display
```

### How It Works
1. **User types** in goose Desktop chat input
2. **Before submit**, React component calls Privacy Guard
3. **Privacy Guard** scans and masks in UI layer
4. **goose backend** only receives masked text
5. **OpenRouter/LLM** never sees raw PII ✅

### Implementation

**Fork:** `https://github.com/block/goose` → `goose-org-twin-ui`

**Files to Modify:**
```
goose-desktop/ (forked)
├── src/ui/
│   ├── components/
│   │   └── ChatInput.tsx        # MODIFY: Add Privacy Guard hook
│   ├── lib/
│   │   └── privacy-guard.ts     # NEW: Privacy Guard HTTP client
│   ├── settings/
│   │   └── PrivacySettings.tsx  # NEW: Privacy Guard config UI
│   └── types/
│       └── privacy.ts           # NEW: TypeScript types
```

**Code Changes:**
```typescript
// src/ui/components/ChatInput.tsx (MODIFIED)
import { usePrivacyGuard } from '../lib/privacy-guard';

export function ChatInput() {
  const { scan, mask, enabled } = usePrivacyGuard();
  const [showPiiWarning, setShowPiiWarning] = useState(false);
  
  async function handleSubmit(message: string) {
    if (enabled) {
      // Scan for PII
      const scanResult = await scan(message);
      
      if (scanResult.detections.length > 0) {
        // Show notification
        setShowPiiWarning(true);
        
        // Mask PII
        const masked = await mask(message);
        message = masked.masked_text;
        
        // Log for audit
        console.log(`🔒 Masked ${scanResult.detections.length} PII items`);
      }
    }
    
    // Send to goose backend (masked if PII found)
    await goose.sendMessage(message);
  }
  
  return (
    <div>
      {showPiiWarning && <Alert>🔒 PII detected and masked</Alert>}
      <input onSubmit={handleSubmit} />
    </div>
  );
}
```

```typescript
// src/ui/lib/privacy-guard.ts (NEW)
export class PrivacyGuardClient {
  constructor(private baseUrl = 'http://localhost:8089') {}
  
  async scan(text: string) {
    const res = await fetch(`${this.baseUrl}/guard/scan`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        text,
        mode: 'hybrid',
        tenant_id: getCurrentUser()
      })
    });
    return res.json(); // { detections: [...] }
  }
  
  async mask(text: string) {
    const res = await fetch(`${this.baseUrl}/guard/mask`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        text,
        method: 'pseudonym',
        mode: 'hybrid',
        tenant_id: getCurrentUser()
      })
    });
    return res.json(); // { masked_text, redactions, session_id }
  }
}

export function usePrivacyGuard() {
  const settings = useSettings();
  const client = new PrivacyGuardClient(settings.privacyGuardUrl);
  
  return {
    enabled: settings.privacyGuardEnabled,
    scan: client.scan.bind(client),
    mask: client.mask.bind(client)
  };
}
```

### Pros
- ✅ Clean UI integration
- ✅ No separate proxy service
- ✅ Better UX (in-app notifications)
- ✅ Can show PII detection in real-time
- ✅ Settings UI for Privacy Guard config
- ✅ LLM never sees raw PII

### Cons
- ❌ Requires fork maintenance
- ❌ Need to merge upstream goose changes regularly
- ❌ Requires Electron/TypeScript/React skills
- ⚠️ Delayed updates from upstream
- ⚠️ Fork becomes "your problem" to maintain

### Validation
- Integration tests (50/50 suite)
- UI testing (E2E with Playwright)
- Performance testing (UI responsiveness)

**Effort:** 2-3 weeks (fork setup + UI changes)  
**Maintenance:** Medium (monthly upstream merges)

---

## Option 3: Standalone UI Client ("goose Enterprise")

### Architecture
```
[Custom Electron App]
    ↓
Privacy Guard (built-in middleware)
    ↓
goose CLI (subprocess)
    ↓
OpenRouter (only sees masked)
```

### How It Works
- Build entirely new desktop app
- Embed Privacy Guard as first-class feature
- Use goose CLI as backend (stdio communication)
- Brand as "goose Enterprise" or "Secure AI Assistant"

### Implementation

**Stack:**
- **Tauri** (Rust + WebView, lighter than Electron)
- **React/Svelte** for UI
- **goose CLI** as subprocess
- **Privacy Guard** as HTTP client

**Architecture:**
```
src/
├── ui/                    # React/Svelte frontend
│   ├── components/
│   ├── services/
│   │   └── privacy.ts     # Privacy Guard integration
│   └── main.tsx
├── backend/               # Tauri Rust backend
│   ├── goose.rs          # Subprocess management
│   ├── privacy.rs        # Privacy Guard client
│   └── main.rs
└── config/
    └── settings.json
```

### Pros
- ✅ Full control over UX/features
- ✅ No fork dependency on goose
- ✅ Can bundle Privacy Guard + goose together
- ✅ Single installer for enterprise users
- ✅ Custom branding/enterprise features
- ✅ LLM never sees raw PII

### Cons
- ❌ High development effort (4-6 weeks)
- ❌ Need to reimplement goose Desktop UI
- ❌ Slower to market
- ⚠️ Potential feature lag behind goose Desktop
- ⚠️ Full app ownership = full maintenance burden

**Effort:** 4-6 weeks (full app development)  
**Maintenance:** High (ongoing feature development)

---

## Option 4: goose CLI Wrapper Script ⭐ QUICK VALIDATION ONLY

### Purpose
**Validate Privacy Guard integration before building production solution**

### Implementation
```bash
#!/bin/bash
# scripts/privacy-goose.sh

set -e

PRIVACY_GUARD_URL="${PRIVACY_GUARD_URL:-http://localhost:8089}"
TENANT_ID="${TENANT_ID:-${USER}}"

# Read user input
echo "Enter your message:"
read -r USER_INPUT

# Scan for PII
SCAN_RESULT=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/scan" \
  -H "Content-Type: application/json" \
  -d "{\"text\":\"$USER_INPUT\",\"mode\":\"hybrid\",\"tenant_id\":\"$TENANT_ID\"}")

DETECTION_COUNT=$(echo "$SCAN_RESULT" | jq '.detections | length')

# Mask if PII found
if [ "$DETECTION_COUNT" -gt 0 ]; then
  echo "⚠️  Detected $DETECTION_COUNT PII item(s). Masking before sending to LLM..."
  
  MASK_RESULT=$(curl -s -X POST "$PRIVACY_GUARD_URL/guard/mask" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"$USER_INPUT\",\"method\":\"pseudonym\",\"mode\":\"hybrid\",\"tenant_id\":\"$TENANT_ID\"}")
  
  MASKED_TEXT=$(echo "$MASK_RESULT" | jq -r '.masked_text')
  SESSION_ID=$(echo "$MASK_RESULT" | jq -r '.session_id')
  
  echo "🔒 Masked text: $MASKED_TEXT"
  echo "📋 Session ID: $SESSION_ID"
  
  USER_INPUT="$MASKED_TEXT"
else
  echo "✅ No PII detected. Sending original message."
fi

# Send to goose CLI
echo "$USER_INPUT" | goose session start
```

### Usage
```bash
chmod +x scripts/privacy-goose.sh
./scripts/privacy-goose.sh

# Example:
Enter your message:
> My SSN is 123-45-6789 and email is john@example.com

⚠️  Detected 2 PII item(s). Masking before sending to LLM...
🔒 Masked text: My SSN is SSN_a1b2c3d4 and email is EMAIL_x9y8z7w6
📋 Session ID: sess_12345

[goose CLI starts with masked text]
```

### Pros
- ✅ Works immediately with existing tools
- ✅ Zero code changes to goose
- ✅ Perfect for validation/proof-of-concept
- ✅ Easy to understand and modify

### Cons
- ❌ CLI-only (no GUI)
- ❌ Poor user experience (manual workflow)
- ❌ Not production-ready
- ❌ No unmask of responses

**Use Case:** Test Privacy Guard integration this week before deciding on production approach

**Effort:** 1 day  
**Maintenance:** None (throwaway)

---

## Option 5: HTTP API Only (No LLM Integration) ❌ NOT RECOMMENDED

### Architecture
```
Admin UI → Controller API → Privacy Guard
(Backend use only)

goose Desktop: Unchanged (no privacy protection)
```

### Description
- Privacy Guard exists ONLY for admin/backend tools
- goose Desktop users have NO PII protection from LLM
- Privacy Guard used for:
  - Scanning audit logs for PII
  - Masking session exports
  - Compliance reporting

### Why This Fails
**❌ Does NOT solve the core problem:**
- Users still send "My SSN is 123-45-6789" to OpenRouter
- LLM provider sees raw PII
- Compliance issues for enterprise
- Defeats the purpose of Privacy Guard

**Recommendation:** DO NOT USE - This option is a non-starter

---

## Recommended Execution Path

### ✅ Phase 1: Return to H6.1 Validation (This Week)

**Goal:** Complete Phase 5 H workstream validation BEFORE deciding on Privacy Guard LLM integration

**Tasks:**
1. ✅ H6.1 complete: 50/50 integration tests passing
2. H7: Performance validation (API latency)
3. H8: Test documentation
4. H_CHECKPOINT: Finalize tracking files

**Why:** Validate full system integration FIRST, then layer on Privacy Guard decisions

---

### ✅ Phase 2: Quick Privacy Validation (Optional This Week)

**IF** you want to validate Privacy Guard concept:

**Use Option 4: CLI Wrapper Script**
- Build `scripts/privacy-goose.sh` (1 day)
- Test with real PII scenarios:
  - SSN: 123-45-6789
  - Email: john@example.com
  - Phone: 555-123-4567
  - Credit Card: 4532-1234-5678-9010
- Document masking effectiveness
- Decide if concept is sound

**Deliverables:**
- `scripts/privacy-goose.sh` (working script)
- `docs/tests/privacy-guard-validation.md` (test results)
- Decision: Does masking protect PII effectively?

---

### ✅ Phase 3: Production Approach (Next Sprint)

**After Phase 1 & 2 complete, choose ONE:**

**Path A: Privacy Guard Proxy** (Recommended for speed)
- Implement `src/privacy-guard-proxy/` (1-2 weeks)
- Integrate with goose Desktop (config change)
- Deploy as systemd service
- **Time:** 2-3 weeks to production

**Path B: goose Desktop Fork** (Recommended for quality)
- Fork goose-desktop repository
- Implement Privacy Guard UI integration (2 weeks)
- Set up upstream merge strategy
- **Time:** 3-4 weeks to production

**Path C: Standalone UI** (Recommended for control)
- Design custom Electron/Tauri app (1 week)
- Implement UI + Privacy Guard (3 weeks)
- Package as installer
- **Time:** 6-8 weeks to production

---

## Decision Criteria

**Choose Option 1 (Proxy) if:**
- ✅ Need production solution FAST (< 3 weeks)
- ✅ Don't want to maintain goose fork
- ✅ Comfortable with proxy architecture
- ⚠️ Can tolerate 50-200ms latency

**Choose Option 2 (Fork) if:**
- ✅ Want best UX integration
- ✅ Have Electron/TypeScript expertise
- ✅ Can commit to upstream merges
- ⚠️ Okay with fork maintenance burden

**Choose Option 3 (Standalone) if:**
- ✅ Need full control over product
- ✅ Want custom enterprise features
- ✅ Have 6+ weeks for development
- ⚠️ Can build/maintain full desktop app

**Use Option 4 (CLI) if:**
- ✅ Just validating concept this week
- ❌ NOT for production use

**Avoid Option 5 (HTTP Only):**
- ❌ Does not protect PII in LLM requests
- ❌ Fails core requirement

---

## Open Questions

1. **Unmask responses?**
   - Should LLM responses containing tokens be unmasked?
   - Example: "Your SSN_a1b2c3d4 is valid" → "Your 123-45-6789 is valid"?
   - **Risk:** Unmasked PII in audit logs

2. **Token lifespan?**
   - How long to store tokens? (Current: session-scoped, deleted after response)
   - What if user wants to reference PII later in conversation?

3. **Fallback behavior?**
   - If Privacy Guard is offline, block requests (fail-closed) or allow (fail-open)?
   - **Recommendation:** Fail-closed for compliance

4. **Multi-provider support?**
   - Proxy needs to handle OpenRouter, Anthropic, OpenAI, local Ollama
   - Each has different API format

5. **Performance targets?**
   - What latency is acceptable? 50ms? 200ms? 500ms?
   - Need benchmarks with real Privacy Guard service

---

## Next Steps

### Immediate (This Week)
1. ✅ **Save this document** for reference
2. ✅ **Return to H6.1 validation** (finish Phase 5 H workstream)
3. ⚠️ **Optional:** Build CLI wrapper for quick privacy validation
4. ✅ **Document decision** after H workstream complete

### After H Workstream Complete
1. Review this document with stakeholders
2. Choose production approach (Option 1, 2, or 3)
3. Create implementation plan
4. Begin development

---

## Related Documents

- Phase 5 Technical Plan: `Technical Project Plan/master-technical-project-plan.md`
- Integration Test Results: `docs/tests/phase5-progress.md`
- Privacy Guard API: `src/privacy-guard/README.md`
- MCP Investigation: `src/privacy-guard-mcp-wrapper/README.md` (archived)

---

## Decision Log

**2025-11-06:** Document created  
**Status:** Pending - Complete H6.1 validation first  
**Next Review:** After Phase 5 H workstream complete
