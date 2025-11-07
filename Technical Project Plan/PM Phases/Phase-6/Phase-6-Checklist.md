# Phase 6 Execution Checklist

**Version:** 1.0  
**Target:** v0.6.0 Production-Ready Release  
**Timeline:** 2-3 weeks (10 actual days)

---

## Workstream A: Vault Production Completion (2 days)

### A1: TLS/HTTPS Setup (2 hours)
- [ ] Generate TLS certificates (OpenSSL self-signed for dev, Let's Encrypt for prod)
  ```bash
  openssl req -newkey rsa:2048 -nodes -keyout vault-key.pem -x509 -days 365 -out vault.crt
  ```
- [ ] Create deploy/vault/certs/ directory
- [ ] Update deploy/compose/docker-compose.yml:
  - [ ] Mount /vault/certs volume
  - [ ] Update VAULT_ADDR=https://vault:8200
- [ ] Update deploy/vault/config.hcl:
  - [ ] Add tls_cert_file="/vault/certs/vault.crt"
  - [ ] Add tls_key_file="/vault/certs/vault-key.pem"
- [ ] Test: `curl --cacert vault.crt https://vault:8200/v1/sys/health`
- [ ] Update docs/guides/VAULT.md (Section 5.1 - mark ✅ Complete)

### A2: AppRole Authentication (3 hours)
- [ ] Enable AppRole auth method
  ```bash
  vault auth enable approle
  ```
- [ ] Create controller-policy.hcl (Transit permissions only)
  ```hcl
  path "transit/hmac/profile-signing/*" {
    capabilities = ["create", "update"]
  }
  path "transit/verify/profile-signing/*" {
    capabilities = ["create", "update"]
  }
  ```
- [ ] Apply policy: `vault policy write controller-policy controller-policy.hcl`
- [ ] Create controller-role:
  ```bash
  vault write auth/approle/role/controller-role \
    token_policies="controller-policy" \
    token_ttl=1h \
    token_max_ttl=4h
  ```
- [ ] Get credentials:
  - [ ] role_id (static): `vault read -field=role_id auth/approle/role/controller-role/role-id`
  - [ ] secret_id (rotatable): `vault write -field=secret_id -f auth/approle/role/controller-role/secret-id`
- [ ] Update src/vault/client.rs:
  - [ ] Add `vault_approle_login()` function
  - [ ] Add token renewal background task (45-min intervals)
- [ ] Update deploy/compose/.env:
  - [ ] Remove VAULT_TOKEN=root
  - [ ] Add VAULT_ROLE_ID=...
  - [ ] Add VAULT_SECRET_ID=...
- [ ] Test: Controller startup → AppRole login → Vault sign → Success
- [ ] Update docs/guides/VAULT.md (Section 5.2 - mark ✅ Complete)

### A3: Persistent Storage (2 hours)
- [ ] Update deploy/vault/config.hcl:
  ```hcl
  storage "raft" {
    path = "/vault/data"
    node_id = "vault-1"
  }
  ```
- [ ] Update deploy/compose/docker-compose.yml:
  - [ ] Add vault-data volume
  - [ ] Mount /vault/data
- [ ] Initialize Vault (first startup):
  ```bash
  vault operator init -key-shares=5 -key-threshold=3
  ```
- [ ] Save unseal keys + root token securely (1Password, Vault, encrypted file)
- [ ] Document unseal procedure in docs/guides/VAULT.md:
  ```bash
  vault operator unseal <key-1>
  vault operator unseal <key-2>
  vault operator unseal <key-3>
  ```
- [ ] Test: Restart Vault → Unseal → AppRole auth → Success
- [ ] Update docs/guides/VAULT.md (Section 5.3 - mark ✅ Complete)

### A4: Audit Device (1 hour)
- [ ] Enable file audit device:
  ```bash
  vault audit enable file file_path=/vault/logs/audit.log
  ```
- [ ] Update deploy/compose/docker-compose.yml:
  - [ ] Add vault-logs volume
  - [ ] Mount /vault/logs
- [ ] Create deploy/vault/logrotate.conf:
  ```
  /vault/logs/audit.log {
    daily
    rotate 30
    compress
    missingok
    notifempty
  }
  ```
- [ ] Test: Perform Vault operation → Check /vault/logs/audit.log → Entry exists
- [ ] Update docs/guides/VAULT.md (Section 5.4 - mark ✅ Complete)

### A5: Signature Verification (2 hours)
- [ ] Update src/routes/profiles.rs (GET /profiles/{role}):
  ```rust
  // After loading profile from DB
  if let Some(signature) = &profile.signature {
      vault_client.verify_hmac(&profile_json, &signature.signature).await?;
  }
  ```
- [ ] Add src/vault/verify.rs:
  ```rust
  pub async fn verify_hmac(
      &self,
      data: &str,
      signature: &str,
  ) -> Result<bool> {
      let response = self.client
          .post(format!("{}/v1/transit/verify/profile-signing/sha2-256", self.addr))
          .json(&json!({
              "input": base64::encode(data),
              "hmac": signature
          }))
          .send()
          .await?;
      
      Ok(response.json::<VerifyResponse>().await?.data.valid)
  }
  ```
- [ ] Add audit log for verification failures (POST /audit/ingest)
- [ ] Test: 
  - [ ] Load valid profile → 200 OK
  - [ ] Tamper with profile in DB → 403 Forbidden
- [ ] Update docs/guides/VAULT.md (Section 5.5 - mark ✅ Complete)

### A6: Integration Test
- [ ] Create tests/integration/phase6-vault-production.sh:
  - [ ] Test TLS connection
  - [ ] Test AppRole auth
  - [ ] Test profile signing
  - [ ] Test signature verification
  - [ ] Test tamper detection (expect 403)
- [ ] Run test: `./tests/integration/phase6-vault-production.sh` → ✅ Pass

---

## Workstream B: Admin UI (SvelteKit) (3 days)

### B1: Setup (2 hours)
- [ ] Create SvelteKit project:
  ```bash
  npm create svelte@latest ui
  cd ui
  npm install
  ```
- [ ] Install dependencies:
  ```bash
  npm install -D tailwindcss @tailwindcss/typography postcss autoprefixer
  npm install d3 monaco-editor @sveltejs/adapter-static
  ```
- [ ] Configure Tailwind (tailwind.config.js, app.css)
- [ ] Configure adapter-static (svelte.config.js):
  ```javascript
  import adapter from '@sveltejs/adapter-static';
  export default {
    kit: {
      adapter: adapter({ pages: 'build', assets: 'build' })
    }
  };
  ```
- [ ] Update src/main.rs (Controller):
  ```rust
  use tower_http::services::ServeDir;
  
  let app = Router::new()
      .route("/profiles/:role", get(get_profile))  // API
      .nest_service("/admin", ServeDir::new("ui/build"));  // UI
  ```

### B2: Dashboard Page (4 hours)
- [ ] Create src/routes/admin/+page.svelte
- [ ] Add D3.js org chart visualization:
  ```typescript
  import * as d3 from 'd3';
  
  onMount(async () => {
    const orgData = await fetch('/admin/org/tree').then(r => r.json());
    renderOrgChart(orgData.tree);
  });
  ```
- [ ] Add agent status cards (active sessions, profiles count)
- [ ] Add recent activity feed (last 10 sessions)
- [ ] Add Vault health status indicator

### B3: Profiles Page (6 hours)
- [ ] Create src/routes/admin/profiles/+page.svelte
- [ ] Add profile list (6 roles: finance, legal, developer, hr, executive, support)
- [ ] Add Monaco YAML editor:
  ```typescript
  import * as monaco from 'monaco-editor';
  
  onMount(() => {
    editor = monaco.editor.create(editorContainer, {
      value: profileYaml,
      language: 'yaml',
      theme: 'vs-dark'
    });
  });
  ```
- [ ] Add Publish button (POST /admin/profiles/{role}/publish)
- [ ] Add Policy Tester:
  ```typescript
  async function testPolicy(role: string, tool: string) {
    const response = await fetch(`/admin/policies/test`, {
      method: 'POST',
      body: JSON.stringify({ role, tool })
    });
    const { allowed, reason } = await response.json();
    return { allowed, reason };
  }
  ```
- [ ] Add schema validation (show errors before saving)

### B4: Org Chart Page (4 hours)
- [ ] Create src/routes/admin/org/+page.svelte
- [ ] Add CSV upload widget (drag-and-drop):
  ```typescript
  async function handleFileUpload(event: Event) {
    const file = event.target.files[0];
    const formData = new FormData();
    formData.append('file', file);
    
    const response = await fetch('/admin/org/import', {
      method: 'POST',
      body: formData
    });
    
    const result = await response.json();
    imports = [...imports, result];
  }
  ```
- [ ] Add import history table
- [ ] Add tree visualization (D3.js hierarchical layout)

### B5: Audit Logs Page (3 hours)
- [ ] Create src/routes/admin/audit/+page.svelte
- [ ] Add table with filters:
  - [ ] Event type (dropdown)
  - [ ] Role (dropdown)
  - [ ] Date range (date picker)
  - [ ] Trace ID (search input)
- [ ] Add Export CSV button:
  ```typescript
  async function exportCsv() {
    const params = new URLSearchParams(filters);
    const csv = await fetch(`/audit/export?format=csv&${params}`).then(r => r.text());
    downloadFile('audit.csv', csv);
  }
  ```

### B6: Settings Page (3 hours)
- [ ] Create src/routes/admin/settings/+page.svelte
- [ ] Add Vault status section:
  - [ ] Sealed/Unsealed indicator
  - [ ] Version (1.18.3)
  - [ ] Key version
- [ ] Add system variables form:
  - [ ] Session retention days
  - [ ] Idempotency TTL
- [ ] Add service health checks:
  - [ ] Controller (GET /status)
  - [ ] Keycloak (GET /realms/dev)
  - [ ] Vault (GET /v1/sys/health)
  - [ ] Postgres (connection check)
  - [ ] Privacy Guard (GET /guard/status)
  - [ ] Ollama (GET /api/tags)

### B7: JWT Auth Integration (2 hours)
- [ ] Add src/lib/auth.ts:
  ```typescript
  export async function getJWT(): Promise<string> {
    // Keycloak OIDC redirect flow
    const params = new URLSearchParams({
      client_id: 'admin-ui',
      redirect_uri: window.location.origin + '/admin/callback',
      response_type: 'code',
      scope: 'openid'
    });
    window.location.href = `http://keycloak:8080/realms/dev/protocol/openid-connect/auth?${params}`;
  }
  ```
- [ ] Add callback handler (src/routes/admin/callback/+page.svelte)
- [ ] Store token in localStorage
- [ ] Add auth check to all admin routes

### B8: Build & Deploy (1 hour)
- [ ] Build UI: `cd ui && npm run build`
- [ ] Copy build/ to Controller static files directory
- [ ] Test: Open http://localhost:8088/admin → Dashboard loads

---

## Workstream C: User UI (Lightweight) (2 days)

### C1: Setup (1 hour)
- [ ] Create SvelteKit project:
  ```bash
  npm create svelte@latest user-ui
  cd user-ui
  npm install
  npm install -D tailwindcss
  ```
- [ ] Configure Tailwind
- [ ] Configure adapter-static

### C2: Profile Viewer Page (3 hours)
- [ ] Create src/routes/+page.svelte
- [ ] Fetch user profile:
  ```typescript
  onMount(async () => {
    const jwt = localStorage.getItem('jwt');
    const profile = await fetch('http://localhost:8088/profiles/finance', {
      headers: { 'Authorization': `Bearer ${jwt}` }
    }).then(r => r.json());
    
    displayProfile(profile);
  });
  ```
- [ ] Display profile card:
  - [ ] Display name
  - [ ] Role
  - [ ] Primary provider
  - [ ] Extensions list
  - [ ] Privacy settings
- [ ] Add download buttons:
  - [ ] Download config.yaml
  - [ ] Download .goosehints
  - [ ] Download .gooseignore
- [ ] Add privacy settings override UI:
  - [ ] Mode (dropdown: rules, ner, hybrid)
  - [ ] Strictness (dropdown: strict, moderate, permissive)
  - [ ] Categories (checkboxes: SSN, EMAIL, PHONE, etc.)

### C3: Chat Interface (4 hours)
- [ ] Create src/routes/chat/+page.svelte
- [ ] Connect to Goose Desktop backend:
  ```typescript
  async function sendMessage(text: string) {
    const response = await fetch('http://localhost:8090/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ message: text })
    });
    const data = await response.json();
    messages = [...messages, { role: 'user', content: text }, { role: 'assistant', content: data.response }];
  }
  ```
- [ ] Add message display (chat bubbles)
- [ ] Add PII redaction indicators:
  ```typescript
  function highlightRedactions(text: string): string {
    return text.replace(/\[PERSON_[A-Z]\]/g, '<span class="badge-redacted">$&</span>');
  }
  ```
- [ ] Add session history (localStorage)

### C4: Sessions Page (2 hours)
- [ ] Create src/routes/sessions/+page.svelte
- [ ] Fetch user sessions:
  ```typescript
  const sessions = await fetch('/sessions?user_id=current').then(r => r.json());
  ```
- [ ] Display table:
  - [ ] Session ID
  - [ ] Status
  - [ ] Created At
  - [ ] View details button
- [ ] Add privacy audit log view:
  ```typescript
  const auditLog = await fetch(`/privacy/audit?session_id=${sessionId}`).then(r => r.json());
  // Show what PII was detected/masked
  ```

### C5: Goose Backend Integration (2 hours)
- [ ] Document Goose HTTP mode:
  ```bash
  # Launch Goose in HTTP server mode
  goose serve --port 8090 --profile finance
  ```
- [ ] Add Privacy Guard MCP config:
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
- [ ] Test: User UI → Chat → Goose backend → Privacy Guard → LLM → Response

### C6: Build & Deploy (1 hour)
- [ ] Build: `cd user-ui && npm run build`
- [ ] Copy build/ to Controller static files
- [ ] Test: Open http://localhost:8088/ → Profile viewer loads

---

## Workstream D: Security Hardening (1 day)

### D1: Secrets Cleanup (2 hours)
- [ ] Grep for hardcoded secrets:
  ```bash
  grep -rn "password\|secret\|token\|api_key" src/ --exclude-dir=target > secrets_audit.txt
  ```
- [ ] Review results, remove any hardcoded secrets
- [ ] Move secrets to .env files (already .gooseignored)
- [ ] Create .env.example:
  ```bash
  # Vault
  VAULT_ADDR=https://vault:8200
  VAULT_ROLE_ID=your-role-id-here
  VAULT_SECRET_ID=your-secret-id-here
  
  # Database
  DATABASE_URL=postgresql://user:password@postgres:5432/controller
  
  # Keycloak
  KEYCLOAK_URL=http://keycloak:8080
  KEYCLOAK_CLIENT_ID=goose-controller
  KEYCLOAK_CLIENT_SECRET=your-client-secret-here
  ```

### D2: Environment Variable Audit (2 hours)
- [ ] Grep for all env vars:
  ```bash
  grep -rn "env::var\|std::env" src/ | sort | uniq > env_vars_audit.txt
  ```
- [ ] Categorize:
  - [ ] Required (VAULT_ADDR, DATABASE_URL, KEYCLOAK_URL)
  - [ ] Optional (LOG_LEVEL, PORT, REDIS_URL)
  - [ ] Secrets (VAULT_TOKEN → removed, use AppRole)
- [ ] Document in README.md:
  ```markdown
  ## Environment Variables
  
  **Required:**
  - VAULT_ADDR: Vault server URL
  - DATABASE_URL: PostgreSQL connection string
  - KEYCLOAK_URL: Keycloak server URL
  
  **Optional:**
  - LOG_LEVEL: Log level (default: info)
  - PORT: Server port (default: 8088)
  ```

### D3: Docker Compose Hardening (2 hours)
- [ ] Remove default passwords from docker-compose.yml
- [ ] Use .env file for secrets
- [ ] Add security_opt:
  ```yaml
  security_opt:
    - no-new-privileges:true
  ```
- [ ] Add read_only where possible
- [ ] Document security considerations in deploy/compose/README.md

### D4: SECURITY.md (1 hour)
- [ ] Create SECURITY.md:
  ```markdown
  # Security Policy
  
  ## Reporting a Vulnerability
  
  Email: security@example.com
  PGP Key: [link]
  
  Response time: 48 hours
  
  ## Supported Versions
  
  | Version | Supported |
  |---------|-----------|
  | 0.6.x   | ✅        |
  | < 0.6   | ❌        |
  
  ## CVE Remediation Process
  
  1. Assess severity (CVSS score)
  2. Patch within 7 days (critical), 30 days (high)
  3. Release security update
  4. Notify users via GitHub Security Advisory
  ```

### D5: README Security Section (1 hour)
- [ ] Add Security section to README.md:
  ```markdown
  ## Security
  
  - **Secrets Management:** All secrets in .env files (never committed)
  - **Vault Production:** Use AppRole, not root token
  - **TLS:** Enable HTTPS for Vault in production
  - **Audit:** Vault audit device logs all operations
  - **Reporting:** See SECURITY.md for vulnerability reporting
  ```

---

## Workstream E: Integration Testing (1 day)

### E1: Vault Production Flow (2 hours)
- [ ] Create tests/integration/phase6-vault-production.sh
- [ ] Test TLS connection
- [ ] Test AppRole auth
- [ ] Test profile signing
- [ ] Test signature verification
- [ ] Test tamper detection (expect 403)

### E2: Admin UI Smoke Tests (2 hours)
- [ ] Install Playwright: `npm install -D @playwright/test`
- [ ] Create tests/e2e/admin-ui.spec.ts:
  ```typescript
  test('Dashboard loads', async ({ page }) => {
    await page.goto('http://localhost:8088/admin');
    await expect(page.locator('h1')).toContainText('Dashboard');
  });
  
  test('Edit profile', async ({ page }) => {
    await page.goto('http://localhost:8088/admin/profiles');
    await page.click('text=Finance');
    await page.locator('.monaco-editor').fill('role: finance\n...');
    await page.click('text=Publish');
    await expect(page.locator('.success')).toContainText('Published');
  });
  ```
- [ ] Run: `npx playwright test` → ✅ Pass

### E3: User UI Smoke Tests (2 hours)
- [ ] Create tests/e2e/user-ui.spec.ts:
  ```typescript
  test('Profile viewer loads', async ({ page }) => {
    await page.goto('http://localhost:8088/');
    await expect(page.locator('.profile-card')).toBeVisible();
  });
  
  test('Chat with PII redaction', async ({ page }) => {
    await page.goto('http://localhost:8088/chat');
    await page.fill('textarea', 'Contact john.doe@acme.com');
    await page.click('text=Send');
    await expect(page.locator('.message')).toContainText('[EMAIL_A]');
  });
  ```
- [ ] Run: `npx playwright test` → ✅ Pass

### E4: Regression Tests (2 hours)
- [ ] Run Phase 1-5 test suites:
  ```bash
  ./tests/integration/test_oidc_login.sh
  ./tests/integration/test_jwt_verification.sh
  ./tests/integration/test_privacy_guard_regex.sh
  ./tests/integration/test_controller_routes.sh
  ./tests/integration/test_profile_loading.sh
  ./tests/integration/test_org_import.sh
  ```
- [ ] Verify: All pass (no regressions)

---

## Workstream F: Documentation (1 day)

### F1: Vault Guide Update (2 hours)
- [ ] Update docs/guides/VAULT.md:
  - [ ] Mark Section 5.1-5.5 as ✅ Complete
  - [ ] Add production deployment checklist
  - [ ] Add troubleshooting section (unseal, AppRole, audit)

### F2: Admin UI Guide (2 hours)
- [ ] Create docs/admin/ADMIN-UI-GUIDE.md:
  - [ ] Installation
  - [ ] Pages overview (Dashboard, Profiles, Org Chart, Audit, Settings)
  - [ ] Profile editing workflow
  - [ ] CSV import format
  - [ ] Policy testing

### F3: User UI Guide (2 hours)
- [ ] Create docs/user/USER-UI-GUIDE.md:
  - [ ] Installation
  - [ ] Profile viewer
  - [ ] Chat interface
  - [ ] Privacy settings override
  - [ ] Sessions view

### F4: Security Guide (1 hour)
- [ ] Create docs/security/SECURITY-HARDENING.md:
  - [ ] Secrets management best practices
  - [ ] Environment variable audit
  - [ ] Docker Compose security
  - [ ] Vault production hardening
  - [ ] TLS configuration

### F5: Migration Guide (1 hour)
- [ ] Create docs/MIGRATION-PHASE6.md:
  - [ ] Upgrading from v0.5.0 to v0.6.0
  - [ ] Breaking changes (none expected)
  - [ ] New features (Vault production, Admin UI, User UI)
  - [ ] Environment variable updates (.env file required)
  - [ ] Deployment changes (AppRole credentials)

---

## Final Deliverables

### Git Operations
- [ ] Commit all changes:
  ```bash
  git add .
  git commit -m "Phase 6 complete: Vault production + Admin UI + User UI + security hardening"
  git push origin main
  ```
- [ ] Tag release:
  ```bash
  git tag -a v0.6.0 -m "Phase 6 Release: Production-ready v0.6.0

Features:
- Vault production integration (TLS, AppRole, Raft, audit, signature verification)
- Admin UI (5 pages: Dashboard, Profiles, Org Chart, Audit, Settings)
- User UI (3 pages: Profile, Chat, Sessions)
- Security hardening (no secrets in repo, .env.example, SECURITY.md)

Tests:
- 15+ integration tests passing
- Admin UI smoke tests (Playwright)
- User UI smoke tests (Playwright)
- Regression tests (Phase 1-5 still pass)

Documentation:
- Vault guide updated
- Admin UI guide
- User UI guide
- Security guide
- Migration guide
"
  git push origin v0.6.0
  ```

### Phase Completion
- [ ] Create Phase-6-Completion-Summary.md (deliverables, test results, metrics)
- [ ] Update Phase-6-Agent-State.json (final state)
- [ ] Update docs/tests/phase6-progress.md (final entry)

---

## Success Criteria

**Phase 6 is complete when:**
- ✅ All checklist items marked complete
- ✅ 15+ integration tests passing
- ✅ Vault production-ready (TLS, AppRole, Raft, audit, verify)
- ✅ Admin UI deployed (5 pages working)
- ✅ User UI deployed (3 pages working)
- ✅ Security hardened (no secrets, .env.example, SECURITY.md)
- ✅ Documentation complete (5 guides)
- ✅ Git tagged v0.6.0
- ✅ No regressions (Phase 1-5 tests pass)

---

**Total Estimated Time:** 10 days (2-3 weeks calendar time)

**Start with Workstream A (Vault Production) when ready!**
