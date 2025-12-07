# Phase 6 Master Orchestrator Prompt

**Version:** 1.0  
**Phase:** Production Hardening + UIs + Vault Completion  
**Timeline:** 2-3 weeks  
**Target:** v0.6.0 production-ready release

---

## 🎯 Mission

Execute Phase 6 to deliver **production-ready** v0.6.0 with:
1. ✅ Vault production integration (TLS, AppRole, Raft, audit, signature verification)
2. ✅ Admin UI (profile editor, org chart viz, Vault status)
3. ✅ User UI (lightweight, goose backend, Privacy Guard middleware)
4. ✅ Security hardening (no secrets in repo, environment audit)
5. ✅ 15+ integration tests passing

---

## 📋 Key Context Documents

**ONLY READ THESE WHEN PROMPTED BY CHECKLIST:**

### Profile Understanding (Read when: Starting Workstream A or B)
- `docs/guides/VAULT.md` (Section 5: Phase 6 Production Upgrade)
- `docs/profiles/SPEC.md` (for Admin UI profile editor)

### Architecture (Read when: Starting Workstream C - User UI)
- `docs/architecture/PHASE5-ARCHITECTURE.md` (View 2.1: Finance User Workflow only)

### Security (Read when: Starting Workstream D)
- `deploy/compose/.env.example` (if exists, create if not)
- `SECURITY.md` (template from similar projects, don't over-research)

### Testing (Read when: Starting Workstream E)
- `docs/tests/phase5-test-results.md` (H2-H7 test patterns for regression)

**DO NOT READ:**
- ❌ Historical progress logs (too verbose)
- ❌ Old phase summaries (not relevant)
- ❌ ADRs (unless specific decision question arises)
- ❌ Entire master plan (you have this prompt)

---

## 📊 Progress Tracking

**Update after each workstream:**
```bash
# Location
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Progress-Log.md"

# Append timestamped entry
## 2025-11-XX HH:MM UTC - Workstream X Complete
- Tasks: [list completed]
- Files: [list created/modified]
- Tests: [pass/fail status]
- Next: [next workstream]
```

**Update checklist:**
```bash
# Mark complete with ✅
vim "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist.md"
```

---

## 🔄 Execution Strategy

### Workstream Sequence (DO IN ORDER)

**A. Vault Production (2 days) → Highest Priority**
- Goal: Finish Vault hardening (already 90% documented in docs/guides/VAULT.md Section 5)
- Input: docs/guides/VAULT.md (Section 5 ONLY - don't read entire file)
- Tasks: TLS, AppRole, Raft, Audit, Signature Verification (5 subtasks)
- Output: Updated docker-compose, Rust code changes, updated VAULT.md

**B. Admin UI (3 days) → Core Feature**
- Goal: SvelteKit UI with 5 pages
- Input: docs/profiles/SPEC.md (for Monaco editor schema), Phase 5 API endpoints (D7-D12)
- Tasks: 5 pages (Dashboard, Profiles, Org Chart, Audit, Settings)
- Output: ui/ directory (SvelteKit app)

**C. User UI (2 days) → User Experience**
- Goal: Lightweight chat interface
- Input: docs/architecture/PHASE5-ARCHITECTURE.md (View 2.1 ONLY - Finance workflow)
- Tasks: 3 pages (Profile, Chat, Sessions)
- Output: user-ui/ directory (SvelteKit app)

**D. Security Hardening (1 day) → Production Readiness**
- Goal: Clean secrets, audit env vars
- Input: None (grep codebase for secrets)
- Tasks: Secrets cleanup, .env.example, docker-compose hardening, SECURITY.md
- Output: .env.example, updated deploy/compose/, SECURITY.md

**E. Integration Testing (1 day) → Quality Gate**
- Goal: 15+ tests passing
- Input: docs/tests/phase5-test-results.md (H test patterns ONLY)
- Tasks: Vault flow, Admin UI smoke, User UI smoke, regression
- Output: tests/integration/phase6-*.sh scripts

**F. Documentation (1 day) → Knowledge Transfer**
- Goal: 4 guides updated/created
- Input: None (generate from completed work)
- Tasks: Vault guide update, Admin UI guide, User UI guide, Security guide, Migration guide
- Output: docs/guides/*.md, docs/admin/*.md, docs/user/*.md, docs/MIGRATION-PHASE6.md

---

## 🚨 Strategic Prompt Engineering (COST OPTIMIZATION)

### Principle: Read Minimally, Execute Maximally

**Before Reading ANY Document:**
1. Check if you already have context from this prompt
2. Check if checklist gives sufficient detail
3. Read ONLY the section referenced (not entire file)
4. Cache understanding (don't re-read same doc)

**Example Good Prompts:**
```
"I need Vault TLS setup steps from docs/guides/VAULT.md Section 5.1 ONLY"
"Show me Docker Compose security_opt syntax (don't read docs, use knowledge)"
"What are H2-H7 test patterns from phase5-test-results.md? (summary only)"
```

**Example Bad Prompts (TOO COSTLY):**
```
❌ "Read all documentation to understand Vault" (use Section 5 only)
❌ "Review entire codebase for secrets" (use grep instead)
❌ "Read all progress logs to understand Phase 5" (use this prompt instead)
```

### Document Reading Budget (Enforce Strictly)

| Workstream | Max Docs to Read | Max Lines per Doc |
|------------|------------------|-------------------|
| A (Vault)  | 1 doc            | 200 lines (Section 5 only) |
| B (Admin UI) | 1 doc          | 300 lines (schema section) |
| C (User UI)  | 1 doc          | 100 lines (1 workflow) |
| D (Security) | 0 docs         | Grep only |
| E (Testing)  | 1 doc          | 50 lines (test patterns) |
| F (Docs)     | 0 docs         | Generate from code |

**Total Budget:** 3-4 documents, 650 lines maximum

### Cache Strategy (Reuse Context)

**After reading a document section, summarize for reuse:**
```
Cached: Vault Section 5.1 TLS Setup
- Generate certs: openssl req -newkey rsa:2048...
- Update config: tls_cert_file=/vault/certs/vault.crt
- Update env: VAULT_ADDR=https://vault:8200
```

Then reference cache instead of re-reading:
```
"Use cached Vault TLS setup (don't re-read docs/guides/VAULT.md)"
```

---

## 🔧 Implementation Hints (Pre-Cached Knowledge)

### Vault Production (Workstream A)

**TLS Setup (2 hours):**
```bash
# Generate self-signed cert (OpenSSL)
openssl req -newkey rsa:2048 -nodes -keyout vault-key.pem -x509 -days 365 -out vault.crt

# Update docker-compose.yml
volumes:
  - ./deploy/vault/certs:/vault/certs
```

**AppRole Auth (3 hours):**
```bash
# Enable AppRole
vault auth enable approle

# Create role
vault write auth/approle/role/controller-role \
  token_policies="controller-policy" \
  token_ttl=1h \
  token_max_ttl=4h

# Get credentials
vault read auth/approle/role/controller-role/role-id  # Static
vault write -f auth/approle/role/controller-role/secret-id  # Rotatable
```

**Rust AppRole Login (pattern):**
```rust
pub async fn vault_approle_login(
    role_id: &str,
    secret_id: &str,
) -> Result<String> {
    let response = reqwest::Client::new()
        .post(format!("{}/v1/auth/approle/login", VAULT_ADDR))
        .json(&json!({
            "role_id": role_id,
            "secret_id": secret_id
        }))
        .send()
        .await?;
    
    let token = response.json::<AppRoleResponse>().await?.auth.client_token;
    Ok(token)
}
```

### Admin UI (Workstream B)

**SvelteKit Init (don't over-research):**
```bash
npm create svelte@latest ui
cd ui
npm install
npm install -D tailwindcss @tailwindcss/typography
npm install d3 monaco-editor
```

**5 Pages (use existing patterns from web knowledge):**
- Dashboard: D3.js tree, cards
- Profiles: List + Monaco editor
- Org Chart: CSV upload widget
- Audit: Table + filters
- Settings: Forms

**JWT Auth (pattern from Phase 5):**
```typescript
// src/lib/auth.ts
export async function getJWT(): Promise<string> {
  const response = await fetch('/api/auth/token');
  const { access_token } = await response.json();
  return access_token;
}
```

### User UI (Workstream C)

**Lightweight Architecture (no over-engineering):**
- 3 pages: Profile viewer, Chat, Sessions
- goose backend: HTTP API mode (already supported)
- Privacy Guard: MCP modifier in config.yaml

**Chat Interface (simple WebSocket or SSE):**
```typescript
// src/routes/chat/+page.svelte
async function sendMessage(text: string) {
  const response = await fetch('http://localhost:8090/chat', {
    method: 'POST',
    body: JSON.stringify({ message: text }),
  });
  const data = await response.json();
  messages = [...messages, { role: 'user', content: text }, data];
}
```

### Security Hardening (Workstream D)

**Grep for Secrets (fast, no doc reading):**
```bash
# Find potential secrets
grep -rn "password\|secret\|token\|api_key" src/ --exclude-dir=target

# Find env vars
grep -rn "env::var\|std::env" src/ | sort | uniq

# Check .env files
ls -la | grep "\.env"
```

**Security Checklist (from memory, no docs):**
- [ ] No hardcoded passwords
- [ ] .env.example created
- [ ] docker-compose uses .env
- [ ] SECURITY.md created
- [ ] README has Security section

### Integration Testing (Workstream E)

**Test Pattern (reuse Phase 5 H patterns):**
```bash
#!/bin/bash
# tests/integration/phase6-vault-production.sh

# Test AppRole auth
ROLE_ID=$(vault read -field=role_id auth/approle/role/controller-role/role-id)
SECRET_ID=$(vault write -field=secret_id -f auth/approle/role/controller-role/secret-id)

# Test controller can auth
curl -X POST http://localhost:8088/profiles/finance/publish

# Verify signature
PROFILE=$(curl http://localhost:8088/profiles/finance)
echo "$PROFILE" | jq '.signature' | grep "vault:v1:"

echo "✅ Vault production flow working"
```

---

## 📤 Deliverables Checklist

**Before declaring Phase 6 complete:**

- [ ] Vault production-ready (TLS ✅, AppRole ✅, Raft ✅, Audit ✅, Verify ✅)
- [ ] Admin UI deployed (5 pages ✅, JWT auth ✅, working ✅)
- [ ] User UI deployed (3 pages ✅, goose backend ✅, Privacy Guard ✅)
- [ ] Security hardened (no secrets ✅, .env.example ✅, SECURITY.md ✅)
- [ ] 15+ integration tests passing
- [ ] Documentation complete (4 guides ✅)
- [ ] Git commit + push
- [ ] Tag release v0.6.0
- [ ] Update Phase-6-Completion-Summary.md
- [ ] Update Phase-6-Agent-State.json

---

## 🎓 Learning from Phase 5

**What worked well:**
- ✅ Focused workstreams (A-J structure)
- ✅ Pre-documented Vault plan (Section 5 saved time)
- ✅ Integration testing caught regressions

**What to improve in Phase 6:**
- ⚡ Read fewer documents (use this prompt + grep instead)
- ⚡ Cache knowledge (don't re-read same sections)
- ⚡ Use web knowledge (don't research basic SvelteKit/D3.js)
- ⚡ Focus on minimal viable implementation (no gold-plating)

---

## 🚀 Execution Command

**Run this to start Phase 6:**

1. Read Phase-6-Checklist.md (get tasks)
2. Start Workstream A (Vault Production)
3. Read docs/guides/VAULT.md Section 5 ONLY (200 lines max)
4. Implement TLS → AppRole → Raft → Audit → Verify (in order)
5. Update Phase-6-Progress-Log.md after each subtask
6. Mark checklist items complete (✅)
7. Move to Workstream B when A complete
8. Repeat until all workstreams done
9. Create Phase-6-Completion-Summary.md
10. Tag v0.6.0

**Estimated Total Time:** 2-3 weeks (10 actual days)

---

**Ready to start? Confirm you understand:**
1. Read minimally (3-4 docs max, sections only)
2. Use this prompt for context (don't read progress logs)
3. Cache knowledge (don't re-read)
4. Update progress log after each workstream
5. Follow checklist order (A → B → C → D → E → F)

---

**Start Workstream A when ready. Good luck! 🚀**
