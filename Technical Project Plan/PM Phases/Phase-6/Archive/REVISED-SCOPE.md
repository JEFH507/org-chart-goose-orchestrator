# Phase 6 REVISED Checklist - Real Working MVP

**Version:** 2.0 (Corrected Architecture)  
**Target:** v0.6.0 Production-Ready MVP  
**Timeline:** 3 weeks (14 actual days)

---

## ⚠️ CRITICAL CHANGES FROM V1.0

**What Changed:**
1. **Removed "User UI" workstream** (users use goose Desktop, not browser)
2. **Added "goose Desktop Fork"** workstream (real integration, not wrapper)
3. **Expanded testing** (test forked goose Desktop end-to-end)
4. **Clarified architecture** (Privacy Guard is HTTP, not MCP)

**Why:**
- Users interact with goose Desktop app (existing UI is excellent)
- Privacy Guard must be integrated into goose provider code (HTTP client)
- No "goose backend mode" exists (was a mistake in v1.0 plan)

---

## Workstream A: Vault Production Completion (2 days)

*(UNCHANGED FROM V1.0 - Keep existing checklist A1-A6)*

### A1: TLS/HTTPS Setup (2 hours)
- [ ] Generate TLS certificates
- [ ] Create deploy/vault/certs/ directory
- [ ] Update docker-compose.yml (mount /vault/certs)
- [ ] Update config.hcl (tls_cert_file, tls_key_file)
- [ ] Update VAULT_ADDR=https://vault:8200
- [ ] Test: `curl --cacert vault.crt https://vault:8200/v1/sys/health`

### A2: AppRole Authentication (3 hours)
- [ ] Enable AppRole: `vault auth enable approle`
- [ ] Create controller-policy.hcl (Transit permissions only)
- [ ] Create controller-role with policy
- [ ] Get role_id (static) and secret_id (rotatable)
- [ ] Update src/vault/client.rs (AppRole login function)
- [ ] Implement token renewal (background task, 45-min intervals)
- [ ] Remove VAULT_TOKEN from .env, add VAULT_ROLE_ID, VAULT_SECRET_ID
- [ ] Test: Controller startup → AppRole login → Success

### A3: Persistent Storage (2 hours)
- [ ] Update config.hcl (Raft storage backend)
- [ ] Update docker-compose.yml (vault-data volume)
- [ ] Initialize: `vault operator init -key-shares=5 -key-threshold=3`
- [ ] Save unseal keys + root token securely
- [ ] Document unseal procedure (3 of 5 keys)
- [ ] Test: Restart Vault → Unseal → Success

### A4: Audit Device (1 hour)
- [ ] Enable: `vault audit enable file file_path=/vault/logs/audit.log`
- [ ] Update docker-compose.yml (vault-logs volume)
- [ ] Create logrotate.conf (daily rotation, 30 days retention)
- [ ] Test: Vault operation → Check audit.log → Entry exists

### A5: Signature Verification (2 hours)
- [ ] Add verify_hmac() to src/vault/verify.rs
- [ ] Update GET /profiles/{role} (verify signature on load)
- [ ] Return 403 Forbidden if signature invalid
- [ ] Add audit log for verification failures
- [ ] Test: Load valid profile → 200 OK
- [ ] Test: Tamper profile in DB → 403 Forbidden

### A6: Integration Test (1 hour)
- [ ] Create tests/integration/phase6-vault-production.sh
- [ ] Test: TLS, AppRole, signing, verification, tamper detection
- [ ] Verify: All tests pass ✅

---

## Workstream B: Admin UI (SvelteKit) (3 days)

*(UNCHANGED FROM V1.0 - Keep existing checklist B1-B8)*

### B1: Setup (2 hours)
- [ ] `npm create svelte@latest admin-ui`
- [ ] Install: tailwindcss, d3, monaco-editor, @sveltejs/adapter-static
- [ ] Configure Tailwind, adapter-static
- [ ] Update Controller (serve static files at /admin)

### B2: Dashboard Page (4 hours)
- [ ] Create src/routes/admin/+page.svelte
- [ ] Add D3.js org chart (fetch /admin/org/tree)
- [ ] Add agent status cards (active sessions, profiles count)
- [ ] Add recent activity feed (last 10 sessions)
- [ ] Add Vault health indicator

### B3: Profiles Page (6 hours)
- [ ] Create src/routes/admin/profiles/+page.svelte
- [ ] Add profile list (6 roles)
- [ ] Add Monaco YAML editor
- [ ] Add Publish button (POST /admin/profiles/{role}/publish)
- [ ] Add Policy Tester (test tool access)
- [ ] Add schema validation

### B4: Org Chart Page (4 hours)
- [ ] Create src/routes/admin/org/+page.svelte
- [ ] Add CSV upload (drag-and-drop)
- [ ] Add import history table
- [ ] Add D3.js tree visualization

### B5: Audit Logs Page (3 hours)
- [ ] Create src/routes/admin/audit/+page.svelte
- [ ] Add filters (event_type, role, date range, trace_id)
- [ ] Add Export CSV button

### B6: Settings Page (3 hours)
- [ ] Create src/routes/admin/settings/+page.svelte
- [ ] Add Vault status (sealed/unsealed, version, key version)
- [ ] Add system variables form (retention days, TTLs)
- [ ] Add service health checks (6 services)

### B7: JWT Auth Integration (2 hours)
- [ ] Add Keycloak OIDC redirect flow
- [ ] Add callback handler
- [ ] Store token in localStorage
- [ ] Add auth check to all admin routes

### B8: Build & Deploy (1 hour)
- [ ] Build: `cd admin-ui && npm run build`
- [ ] Copy build/ to Controller static files
- [ ] Test: http://localhost:8088/admin → Dashboard loads

---

## Workstream C: goose Desktop Fork + Integration (5 days) 🚨 NEW CRITICAL

### C1: Fork goose Desktop Repository (2 hours)

- [ ] Fork https://github.com/block/goose to https://github.com/JEFH507/goose-enterprise
- [ ] Clone locally:
  ```bash
  git clone git@github.com:JEFH507/goose-enterprise.git
  cd goose-enterprise
  ```
- [ ] Verify build:
  ```bash
  cargo build --release
  ./target/release/goose --version
  ```
- [ ] Test: Launch goose Desktop (unchanged) → Verify it works
- [ ] Create branch: `git checkout -b feature/profile-integration`

**Deliverable:** Working goose Desktop fork (no changes yet)

---

### C2: Add Profile Loading (--profile flag) (1.5 days)

**Goal:** `goose-enterprise --profile finance` fetches config from Controller

**C2.1: Update CLI Args (2 hours)**
- [ ] Update src/main.rs:
  ```rust
  #[derive(Parser, Debug)]
  #[command(name = "goose-enterprise")]
  #[command(about = "goose Desktop with Enterprise Profile Integration")]
  struct Cli {
      /// Profile to load from Controller (finance, legal, developer, etc.)
      #[arg(long)]
      profile: Option<String>,
      
      /// Controller URL (default: http://localhost:8088)
      #[arg(long, default_value = "http://localhost:8088")]
      controller_url: String,
      
      /// Existing goose args
      #[command(flatten)]
      goose_args: GooseArgs,
  }
  ```

**C2.2: Implement Profile Fetcher (4 hours)**
- [ ] Create src/enterprise/profile_loader.rs:
  ```rust
  pub struct ProfileLoader {
      controller_url: String,
      jwt: String,
  }
  
  impl ProfileLoader {
      pub async fn fetch_profile(&self, role: &str) -> Result<ProfileConfig> {
          // 1. Fetch full profile
          let profile_response = reqwest::Client::new()
              .get(format!("{}/profiles/{}", self.controller_url, role))
              .header("Authorization", format!("Bearer {}", self.jwt))
              .send()
              .await?;
          
          if profile_response.status() != 200 {
              return Err(format!("Profile fetch failed: {}", profile_response.status()));
          }
          
          let profile: ProfileData = profile_response.json().await?;
          
          // 2. Fetch config.yaml
          let config_yaml = self.fetch_config(role).await?;
          
          // 3. Fetch goosehints
          let goosehints = self.fetch_goosehints(role).await?;
          
          // 4. Fetch gooseignore
          let gooseignore = self.fetch_gooseignore(role).await?;
          
          Ok(ProfileConfig {
              profile,
              config_yaml,
              goosehints,
              gooseignore,
          })
      }
      
      async fn fetch_config(&self, role: &str) -> Result<String> {
          let response = reqwest::Client::new()
              .get(format!("{}/profiles/{}/config", self.controller_url, role))
              .header("Authorization", format!("Bearer {}", self.jwt))
              .send()
              .await?;
          
          Ok(response.text().await?)
      }
      
      // Similar for goosehints, gooseignore
  }
  ```

**C2.3: Integrate into Main Startup (2 hours)**
- [ ] Update src/main.rs main() function:
  ```rust
  #[tokio::main]
  async fn main() -> Result<()> {
      let cli = Cli::parse();
      
      // NEW: Profile loading
      if let Some(profile_name) = cli.profile {
          // Get JWT (from keyring or prompt user)
          let jwt = get_jwt_token(&cli.controller_url).await?;
          
          // Fetch profile from Controller
          let loader = ProfileLoader::new(cli.controller_url.clone(), jwt);
          let profile_config = loader.fetch_profile(&profile_name).await?;
          
          // Save to ~/.config/goose/
          save_profile_config(&profile_config).await?;
          
          println!("✅ Profile loaded: {}", profile_name);
      }
      
      // Existing goose startup
      run_goose(cli.goose_args).await
  }
  ```

**C2.4: JWT Token Helper (2 hours)**
- [ ] Create src/enterprise/auth.rs:
  ```rust
  pub async fn get_jwt_token(controller_url: &str) -> Result<String> {
      // Option 1: Check keyring first
      if let Ok(jwt) = keyring::get_password("goose-enterprise", "jwt") {
          if !is_jwt_expired(&jwt)? {
              return Ok(jwt);
          }
      }
      
      // Option 2: Prompt user for credentials
      println!("Authentication required");
      print!("Email: ");
      io::stdout().flush()?;
      let mut email = String::new();
      io::stdin().read_line(&mut email)?;
      
      let password = rpassword::prompt_password("Password: ")?;
      
      // Get JWT from Keycloak
      let keycloak_url = env::var("KEYCLOAK_URL")
          .unwrap_or("http://localhost:8080".into());
      
      let response = reqwest::Client::new()
          .post(format!("{}/realms/dev/protocol/openid-connect/token", keycloak_url))
          .form(&[
              ("client_id", "goose-controller"),
              ("grant_type", "password"),
              ("username", email.trim()),
              ("password", &password),
              ("scope", "openid"),
          ])
          .send()
          .await?;
      
      let token_response: TokenResponse = response.json().await?;
      
      // Save to keyring (for future)
      keyring::set_password("goose-enterprise", "jwt", &token_response.access_token)?;
      
      Ok(token_response.access_token)
  }
  ```

**C2.5: Test Profile Loading (1 hour)**
- [ ] Test: `goose-enterprise --profile finance --controller-url http://localhost:8088`
- [ ] Verify: Prompts for credentials
- [ ] Verify: Fetches config from Controller
- [ ] Verify: Saves to ~/.config/goose/
- [ ] Verify: goose launches with Finance profile
- [ ] Verify: Primary provider is Claude 3.5 Sonnet (from profile)
- [ ] Verify: Extensions match profile (github enabled)

---

### C3: Add Privacy Guard HTTP Client (2 days)

**Goal:** Integrate Privacy Guard HTTP service into goose provider code

**C3.1: Create Privacy Guard Client (4 hours)**
- [ ] Create src/enterprise/privacy_guard.rs:
  ```rust
  pub struct PrivacyGuardClient {
      url: String,
      session_id: String,
  }
  
  impl PrivacyGuardClient {
      pub async fn mask(&self, text: &str) -> Result<MaskResult> {
          let response = reqwest::Client::new()
              .post(format!("{}/guard/mask", self.url))
              .json(&json!({
                  "text": text,
                  "tenant_id": "user-org",
                  "session_id": self.session_id,
              }))
              .send()
              .await?;
          
          if response.status() != 200 {
              return Err(format!("Privacy Guard mask failed: {}", response.status()));
          }
          
          let result: MaskResponse = response.json().await?;
          
          Ok(MaskResult {
              masked_text: result.masked_text,
              entities_detected: result.entities.len(),
              entity_types: result.entities.iter()
                  .map(|e| e.entity_type.clone())
                  .collect(),
          })
      }
      
      pub async fn reidentify(&self, masked_text: &str, jwt: &str) -> Result<String> {
          let response = reqwest::Client::new()
              .post(format!("{}/guard/reidentify", self.url))
              .header("Authorization", format!("Bearer {}", jwt))
              .json(&json!({
                  "masked_text": masked_text,
                  "tenant_id": "user-org",
                  "session_id": self.session_id,
              }))
              .send()
              .await?;
          
          if response.status() != 200 {
              return Err(format!("Privacy Guard reidentify failed: {}", response.status()));
          }
          
          let result: ReidentifyResponse = response.json().await?;
          Ok(result.original_text)
      }
  }
  ```

**C3.2: Integrate into OpenRouter Provider (2 hours)**
- [ ] Update src/providers/openrouter.rs:
  ```rust
  pub struct OpenRouterProvider {
      client: reqwest::Client,
      model: String,
      // NEW: Privacy Guard integration
      privacy_guard: Option<PrivacyGuardClient>,
  }
  
  impl OpenRouterProvider {
      pub async fn send_message(&self, prompt: &str) -> Result<String> {
          // NEW: Mask PII before LLM call
          let (processed_prompt, mask_result) = if let Some(guard) = &self.privacy_guard {
              let result = guard.mask(prompt).await?;
              println!("🛡️  Privacy Guard: {} PII entities masked", result.entities_detected);
              (result.masked_text, Some(result))
          } else {
              (prompt.to_string(), None)
          };
          
          // Send to OpenRouter (masked text)
          let response = self.client
              .post("https://openrouter.ai/api/v1/chat/completions")
              .json(&json!({
                  "model": self.model,
                  "messages": [{"role": "user", "content": processed_prompt}]
              }))
              .send()
              .await?;
          
          let llm_response = response.json::<ChatResponse>().await?
              .choices[0].message.content.clone();
          
          // NEW: Unmask PII in response
          let final_response = if let Some(guard) = &self.privacy_guard {
              guard.reidentify(&llm_response, &self.jwt).await?
          } else {
              llm_response
          };
          
          Ok(final_response)
      }
  }
  ```

**C3.3: Integrate into Other Providers (4 hours)**
- [ ] Update src/providers/openai.rs (same pattern)
- [ ] Update src/providers/anthropic.rs (same pattern)
- [ ] Update src/providers/ollama.rs (same pattern for completeness)

**C3.4: Load Privacy Guard from Config (2 hours)**
- [ ] Update config parsing to load privacy_guard settings:
  ```yaml
  # ~/.config/goose/config.yaml (generated from profile)
  
  provider: openrouter
  model: anthropic/claude-3.5-sonnet
  
  # NEW: Privacy Guard configuration
  privacy_guard:
    enabled: true
    url: "http://localhost:8089"
    mode: "hybrid"  # From profile.privacy.mode
    strictness: "strict"  # From profile.privacy.strictness
  ```
- [ ] Update src/config/mod.rs to parse privacy_guard section
- [ ] Pass PrivacyGuardClient to providers

**C3.5: Test Privacy Guard Integration (2 hours)**
- [ ] Test: User chats "Contact john@acme.com"
- [ ] Verify: Console shows "🛡️ Privacy Guard: 1 PII entities masked"
- [ ] Verify: OpenRouter request (check network tab) contains "EMAIL_7a3f9b", NOT "john@acme.com"
- [ ] Verify: User sees response with "john@acme.com" (unmasked)
- [ ] Test with Legal profile (local-only, Ollama) → Privacy Guard blocks cloud

---

### C4: Add Profile Settings Tab (1 day)

**Goal:** Show current profile, allow privacy overrides (match goose Desktop UI style)

**C4.1: Create Profile Tab UI (4 hours)**
- [ ] Create src/ui/profile_settings.rs (or .tsx if using Tauri + React)
- [ ] Add tab to sidebar (after Extensions, before Settings)
- [ ] Display current profile:
  ```
  Profile Settings
  ┌─────────────────────────────────────────────────┐
  │ Current Profile: Finance Manager                │
  │ Last Synced: 2 hours ago                        │
  │                                                  │
  │ [Reload Profile from Controller]                │
  │                                                  │
  │ Profile Details:                                │
  │ ├─ Display Name: Finance Manager                │
  │ ├─ Role: finance                                │
  │ ├─ Primary Provider: Claude 3.5 Sonnet          │
  │ ├─ Worker Provider: GPT-4o-mini                 │
  │ ├─ Extensions: github, memory                   │
  │ ├─ Privacy Mode: Strict                         │
  │ └─ Recipes: 3 (monthly, weekly, quarterly)      │
  │                                                  │
  │ Privacy Guard Overrides:                        │
  │ Mode: [Hybrid ▼]                                │
  │ Strictness: [Strict ▼]                          │
  │ Categories:                                     │
  │   ☑ SSN     ☑ EMAIL    ☑ PHONE                 │
  │   ☐ PERSON  ☐ ORG      ☐ ACCOUNT                │
  │                                                  │
  │ [Reset to Profile Defaults]  [Save Overrides]  │
  └─────────────────────────────────────────────────┘
  ```

**C4.2: Implement Privacy Overrides (4 hours)**
- [ ] Create ~/.config/goose/privacy-overrides.yaml storage
- [ ] Implement Save Overrides:
  ```yaml
  # ~/.config/goose/privacy-overrides.yaml
  mode: "rules"  # User downgraded from hybrid
  strictness: "moderate"  # User downgraded from strict
  disabled_categories: ["PERSON"]  # User wants person names unmasked
  ```
- [ ] Merge overrides with profile defaults on load
- [ ] Apply to PrivacyGuardClient configuration

**C4.3: Add Reload Profile Button (2 hours)**
- [ ] Implement "Reload Profile from Controller" button
- [ ] Re-fetch config, hints, ignore from Controller
- [ ] Prompt to restart goose Desktop (config changes require restart)

---

### C5: Add Privacy Guard Status Indicator (4 hours)

**Goal:** Show Privacy Guard connection status + last session stats

**C5.1: Add Status to Settings Tab (2 hours)**
- [ ] Update Settings → Add Privacy Guard section:
  ```
  Privacy Guard
  ┌─────────────────────────────────────────────────┐
  │ Status: ✅ Connected (http://localhost:8089)    │
  │ Mode: Hybrid (rules + NER)                      │
  │ Strictness: Strict                              │
  │                                                  │
  │ Last Session:                                   │
  │ ├─ PII Detected: 3                              │
  │ ├─ Entity Types: EMAIL (2), SSN (1)            │
  │ ├─ Tokens: EMAIL_7a3f9b, EMAIL_2f8a1c, SSN_9d4e│
  │                                                  │
  │ [View Audit Log]  [Flush Tokens]                │
  └─────────────────────────────────────────────────┘
  ```

**C5.2: Implement Health Check (2 hours)**
- [ ] Add startup health check:
  ```rust
  async fn check_privacy_guard_health(url: &str) -> Result<bool> {
      let response = reqwest::Client::new()
          .get(format!("{}/guard/status", url))
          .send()
          .await?;
      
      Ok(response.status() == 200)
  }
  ```
- [ ] Show warning if Privacy Guard not reachable
- [ ] Add reconnect button

---

### C6: Update Goosehints/Gooseignore Editors (Optional - 4 hours)

**Goal:** Allow local editing of global hints/ignore (like Screenshot 2)

**Current goose Desktop has this!** (Screenshot 2 shows Monaco-like editor for goosehints)

**What to add:**
- [ ] "Reset to Profile Default" button
- [ ] "Save Local Override" button
- [ ] Show diff between profile default vs local override

**If time allows, implement. Otherwise defer to Phase 7.**

---

### C7: Build & Package goose-Enterprise (1 day)

**C7.1: Build (2 hours)**
- [ ] Update Cargo.toml (rename to goose-enterprise, version 0.6.0)
- [ ] Build release: `cargo build --release`
- [ ] Test on local machine

**C7.2: Create Installation Script (2 hours)**
- [ ] Create install.sh:
  ```bash
  #!/bin/bash
  # Install goose Enterprise
  
  # 1. Install binary
  sudo cp target/release/goose-enterprise /usr/local/bin/
  
  # 2. Create desktop entry
  cat > ~/.local/share/applications/goose-enterprise.desktop <<EOF
  [Desktop Entry]
  Name=goose Enterprise
  Exec=/usr/local/bin/goose-enterprise --profile \$PROFILE
  Icon=goose
  Type=Application
  Categories=Development;
  EOF
  
  # 3. Create launcher script
  cat > /usr/local/bin/goose-finance <<EOF
  #!/bin/bash
  /usr/local/bin/goose-enterprise --profile finance --controller-url \$CONTROLLER_URL
  EOF
  chmod +x /usr/local/bin/goose-finance
  
  echo "✅ goose Enterprise installed!"
  echo "Usage: goose-enterprise --profile <role>"
  echo "   or: goose-finance (Finance profile)"
  ```

**C7.3: Documentation (2 hours)**
- [ ] Create docs/user/GOOSE-ENTERPRISE-INSTALL.md
- [ ] Installation guide
- [ ] Profile loading guide
- [ ] Privacy Guard setup
- [ ] Troubleshooting

**C7.4: Test on Clean System (2 hours)**
- [ ] Test installation on fresh Ubuntu VM
- [ ] Test: goose-enterprise --profile finance
- [ ] Verify: Config loaded, Privacy Guard working, LLM calls masked

---

## Workstream D: Security Hardening (1 day)

*(UNCHANGED FROM V1.0 - Keep existing checklist D1-D5)*

### D1: Secrets Cleanup (2 hours)
- [ ] Grep for hardcoded secrets: `grep -rn "password\|secret\|token" src/`
- [ ] Remove any hardcoded values
- [ ] Move to .env files

### D2: Environment Variable Audit (2 hours)
- [ ] Grep all env vars: `grep -rn "env::var\|std::env" src/ | sort | uniq`
- [ ] Categorize (required, optional, secrets)
- [ ] Document in README.md

### D3: Docker Compose Hardening (2 hours)
- [ ] Remove default passwords
- [ ] Use .env file for secrets
- [ ] Add security_opt (no-new-privileges)
- [ ] Add read_only where possible

### D4: Create .env.example (1 hour)
- [ ] Create .env.example with placeholder values:
  ```bash
  # Vault
  VAULT_ADDR=https://vault:8200
  VAULT_ROLE_ID=your-role-id-here
  VAULT_SECRET_ID=your-secret-id-here
  VAULT_CACERT=/path/to/ca.crt
  
  # Database
  DATABASE_URL=postgresql://user:password@postgres:5432/controller
  
  # Keycloak
  KEYCLOAK_URL=http://keycloak:8080
  KEYCLOAK_CLIENT_ID=goose-controller
  KEYCLOAK_CLIENT_SECRET=your-client-secret-here
  
  # Privacy Guard
  PRIVACY_GUARD_URL=http://localhost:8089
  
  # Controller
  CONTROLLER_URL=http://localhost:8088
  PORT=8088
  LOG_LEVEL=info
  ```

### D5: Create SECURITY.md (1 hour)
- [ ] Responsible disclosure policy
- [ ] Security contact email
- [ ] Supported versions
- [ ] CVE remediation process

### D6: Update README.md (1 hour)
- [ ] Add Security section
- [ ] Document secrets management
- [ ] Link to SECURITY.md

---

## Workstream E: Integration Testing (2 days) - EXPANDED

### E1: Vault Production Flow (2 hours)
- [ ] Create tests/integration/phase6-vault-production.sh
- [ ] Test: TLS connection, AppRole auth, signing, verification, tamper detection

### E2: Admin UI Smoke Tests (2 hours)
- [ ] Install Playwright: `npm install -D @playwright/test`
- [ ] Create tests/e2e/admin-ui.spec.ts:
  - [ ] Test: Dashboard loads
  - [ ] Test: Edit profile, publish
  - [ ] Test: Upload CSV org chart
  - [ ] Test: View audit logs
- [ ] Run: `npx playwright test` → All pass

### E3: goose Desktop End-to-End Test (4 hours) 🚨 CRITICAL NEW TEST
- [ ] Create tests/integration/phase6-goose-desktop-e2e.sh:
  ```bash
  #!/bin/bash
  # End-to-End Test: User signs in → Profile loads → Chat with PII → Privacy Guard works
  
  echo "=== Phase 6 End-to-End Test ==="
  
  # 1. Launch goose Desktop with Finance profile
  echo "Step 1: Launch goose Desktop with Finance profile"
  goose-enterprise --profile finance --controller-url http://localhost:8088 &
  GOOSE_PID=$!
  sleep 5  # Wait for startup
  
  # 2. Verify config loaded
  echo "Step 2: Verify config.yaml loaded"
  grep "anthropic/claude-3.5-sonnet" ~/.config/goose/config.yaml
  if [ $? -ne 0 ]; then
    echo "❌ Config not loaded from Controller"
    exit 1
  fi
  
  # 3. Verify Privacy Guard enabled
  echo "Step 3: Verify Privacy Guard enabled in config"
  grep "privacy_guard:" ~/.config/goose/config.yaml
  if [ $? -ne 0 ]; then
    echo "❌ Privacy Guard not enabled"
    exit 1
  fi
  
  # 4. Send chat message with PII (via goose CLI if available, or UI automation)
  echo "Step 4: Send chat with PII"
  # NOTE: May need to use goose API or UI automation
  
  # 5. Check Privacy Guard logs (verify mask was called)
  echo "Step 5: Verify Privacy Guard was called"
  docker logs privacy-guard | grep "POST /guard/mask"
  if [ $? -ne 0 ]; then
    echo "❌ Privacy Guard not called"
    exit 1
  fi
  
  # 6. Check LLM request didn't contain raw PII
  echo "Step 6: Verify LLM received masked text (manual verification needed)"
  echo "   Check OpenRouter logs for 'EMAIL_' tokens (not raw emails)"
  
  # 7. Cleanup
  kill $GOOSE_PID
  
  echo "✅ End-to-End test passed!"
  ```

### E4: Regression Tests (2 hours)
- [ ] Run all Phase 1-5 tests:
  ```bash
  ./tests/integration/test_oidc_login.sh
  ./tests/integration/test_profile_loading.sh
  ./tests/integration/test_org_import.sh
  # ... all 60 tests
  ```
- [ ] Verify: All pass (no regressions)

### E5: Performance Validation (2 hours)
- [ ] Test profile loading latency (target: P50 < 100ms)
- [ ] Test Privacy Guard overhead (target: P50 < 500ms)
- [ ] Test end-to-end chat latency (target: P50 < 5s)

---

## Workstream F: Documentation (1 day)

### F1: Vault Guide Update (2 hours)
- [ ] Update docs/guides/VAULT.md:
  - [ ] Mark Section 5.1-5.5 as ✅ Complete (production setup done)
  - [ ] Add troubleshooting (unseal, AppRole token renewal, audit logs)

### F2: goose Enterprise Install Guide (2 hours) 🚨 NEW
- [ ] Create docs/user/GOOSE-ENTERPRISE-INSTALL.md:
  - [ ] Fork setup (for developers who want to build)
  - [ ] Binary installation (for end users)
  - [ ] Profile loading (--profile flag usage)
  - [ ] Privacy Guard setup (launch service, configure)
  - [ ] Troubleshooting (JWT expiration, profile loading failures)

### F3: Admin UI Guide (2 hours)
- [ ] Create docs/admin/ADMIN-UI-GUIDE.md:
  - [ ] Pages overview
  - [ ] Profile editing (Monaco YAML editor)
  - [ ] Org chart CSV upload
  - [ ] Vault status monitoring

### F4: Security Guide (1 hour)
- [ ] Create docs/security/SECURITY-HARDENING.md:
  - [ ] Secrets management
  - [ ] Environment variable audit
  - [ ] Docker Compose security
  - [ ] Vault production hardening

### F5: Migration Guide (1 hour)
- [ ] Create docs/MIGRATION-PHASE6.md:
  - [ ] Upgrading from v0.5.0 to v0.6.0
  - [ ] New requirement: goose Desktop fork (goose-enterprise binary)
  - [ ] Vault production setup
  - [ ] Breaking changes (must use goose-enterprise, not goose)
  - [ ] Environment variable changes (.env file required)

---

## Final Deliverables (REVISED)

### Code Deliverables
- [ ] Vault production-ready (TLS, AppRole, Raft, audit, verify)
- [ ] Admin UI (5 pages: Dashboard, Profiles, Org, Audit, Settings)
- [ ] **goose Desktop Fork (goose-enterprise):**
  - [ ] --profile flag (fetch config from Controller)
  - [ ] Privacy Guard HTTP client (mask/reidentify)
  - [ ] Profile Settings tab (view profile, privacy overrides)
  - [ ] Privacy Guard status (in Settings)
  - [ ] Integrated into all providers (OpenRouter, OpenAI, Anthropic, Ollama)
- [ ] Security hardened (no secrets in repo, .env.example, SECURITY.md)

### Testing Deliverables
- [ ] Vault production tests (5 tests)
- [ ] Admin UI smoke tests (Playwright, 8 tests)
- [ ] **goose Desktop E2E test (1 critical test):**
  - [ ] Sign in → Load profile → Chat with PII → Privacy Guard masks → LLM → Unmask → User sees original
- [ ] Regression tests (Phase 1-5, 60 tests)
- [ ] Performance tests (profile loading, Privacy Guard, chat latency)
- [ ] **Total:** 75+ tests passing ✅

### Documentation Deliverables
- [ ] Vault guide updated (production complete)
- [ ] **goose Enterprise install guide** (NEW)
- [ ] Admin UI guide
- [ ] Security hardening guide
- [ ] Migration guide (v0.5.0 → v0.6.0)

### Git Deliverables
- [ ] goose fork repo: https://github.com/JEFH507/goose-enterprise
- [ ] Main repo: All Phase 6 changes committed
- [ ] Tag release: v0.6.0
- [ ] Update Phase-6-Completion-Summary.md
- [ ] Update Phase-6-Agent-State.json (final state)

---

## Success Criteria (Real MVP)

**Phase 6 is complete when ALL of these work:**

### Critical Path Tests:
1. ✅ **Profile Loading:** `goose-enterprise --profile finance` → Fetches config from Controller
2. ✅ **Authentication:** User prompted for credentials → JWT obtained → Profile loaded
3. ✅ **Privacy Guard:** User chats "Contact john@acme.com" → LLM sees "Contact EMAIL_7a3f9b" → User sees "john@acme.com"
4. ✅ **Provider Selection:** Finance profile uses Claude 3.5 Sonnet, Legal uses Ollama (local-only)
5. ✅ **Extension Loading:** Finance profile has github extension enabled automatically
6. ✅ **Goosehints Injection:** Finance-specific hints appear in prompts
7. ✅ **Gooseignore Protection:** .env files excluded from context
8. ✅ **Vault Signing:** Admin publishes profile → Vault signs → Signature stored
9. ✅ **Signature Verification:** User loads profile → Vault verifies → 200 OK (or 403 if tampered)
10. ✅ **Admin UI:** Admin logs in → Edits profile → Publishes → Org chart uploads

### This is a REAL MVP when all 10 tests pass! ✅

---

## Effort Summary (REVISED)

| Workstream | Original Effort | Revised Effort | Reason |
|------------|----------------|----------------|--------|
| A. Vault Production | 2 days | 2 days | Unchanged |
| B. Admin UI | 3 days | 3 days | Unchanged |
| C. User UI (SvelteKit) | 2 days | **0 days** | ❌ Removed (users use goose Desktop) |
| **C. goose Desktop Fork** | **0 days** | **5 days** | 🚨 NEW (real integration) |
| D. Security Hardening | 1 day | 1 day | Unchanged |
| E. Integration Testing | 1 day | 2 days | Expanded (goose Desktop E2E) |
| F. Documentation | 1 day | 1 day | Unchanged |
| **TOTAL** | **10 days** | **14 days** | **+4 days for real MVP** |

**Timeline:** 3 weeks (calendar time with overhead)

---

## What You Get at End of Phase 6

### Working Features:
1. **End User Experience:**
   - Launch: `goose-finance` (wrapper script)
   - goose Desktop loads Finance profile automatically
   - Chat with PII protection (masked before cloud LLM)
   - Extensions pre-configured (github for budget tracking)
   - Goosehints injected (Finance-specific context)
   - Gooseignore protects secrets

2. **Admin Experience:**
   - Web UI at http://localhost:8088/admin
   - Create/edit profiles (Monaco YAML editor)
   - Publish profiles (Vault signing)
   - Upload org chart (CSV)
   - View audit logs
   - Monitor Vault status

3. **Security:**
   - Vault production-ready (TLS, AppRole, Raft, audit)
   - Profile signatures verified (tamper detection)
   - No secrets in repo (.env.example only)
   - Privacy Guard masks PII before cloud

4. **Testing:**
   - 75+ tests passing
   - End-to-end test validates full flow
   - Performance targets met

**THIS IS A REAL, WORKING MVP** ✅

---

## What's Still Missing (Post-Phase 6)

**Phase 7 will add:**
- Better NER detection (fine-tune or prompt engineering)
- Performance optimization (smart model triggering)
- Corporate PII dataset validation

**Future (Phase 8+):**
- Multi-agent coordination (Agent Mesh workflows)
- Approval workflows
- Advanced RBAC (policy composition)
- SCIM integration
- Kubernetes deployment

**But Phase 6 gives you a complete, working, production-ready system** ✅

---

**Estimated Total Time:** 14 days (3 weeks calendar)

**Critical Success Factor:** goose Desktop fork with Privacy Guard integration (Workstream C)

**End Result:** Real MVP that you can demo, deploy, and use in production ✅
