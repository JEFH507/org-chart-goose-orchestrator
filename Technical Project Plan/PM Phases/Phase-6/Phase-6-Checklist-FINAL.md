# Phase 6 Checklist - Proxy + Scripts Approach

**Version:** 3.0 (Architecture-Aligned)  
**Decision:** Proxy + Scripts (validated approach)  
**Target:** v0.6.0 Production-Ready MVP  
**Timeline:** 14 days (3 weeks calendar)

---

## 🎯 Phase 6 Goals

**Deliver working MVP with:**
1. ✅ Users sign in (via setup script)
2. ✅ Profiles auto-load from Controller
3. ✅ PII protection (proxy intercepts LLM requests)
4. ✅ Production-ready (Vault hardened, security audited)
5. ✅ Admin UI (profile management, org chart, audit)
6. ✅ All tests passing (75+ integration tests)

---

## 📋 VALIDATION PHASE (Do First - 1 day)

### V1: Run Privacy Guard Validation ✅ READY
- [x] Validation script created: `scripts/privacy-goose-validate.sh`
- [ ] Start Privacy Guard service:
  ```bash
  cd /home/papadoc/Gooseprojects/goose-org-twin
  docker compose -f deploy/compose/ce.dev.yml up -d privacy-guard
  ```
- [ ] Run validation:
  ```bash
  ./scripts/privacy-goose-validate.sh
  ```
- [ ] Expected: 6/6 tests pass (SSN, Email, Phone, Multiple PII, Credit Card, No PII)
- [ ] If all pass → Proceed to Workstream A
- [ ] If any fail → Fix Privacy Guard, retry

**Deliverable:** Validated Privacy Guard concept ✅

---

## 📋 WORKSTREAM A: Vault Production Completion (2 days)

### A1: TLS/HTTPS + Raft Setup (2 hours) ✅ COMPLETE (Recovery 2025-11-07)

**Recovery Note:** A1 completed with Raft storage from the start (production-ready approach)

- [x] Generate TLS certificates (FRESH 2025-11-07 20:00):
  ```bash
  cd deploy/vault/certs
  openssl req -newkey rsa:2048 -nodes -keyout vault-key.pem \
    -x509 -days 365 -out vault.crt \
    -subj "/CN=vault/O=OrgChart/C=US"
  ```
  
- [x] Create `deploy/vault/config/vault.hcl` with Raft storage:
  ```hcl
  # Dual listener (HTTPS 8200, HTTP 8201)
  listener "tcp" {
    address     = "0.0.0.0:8200"
    tls_cert_file = "/vault/certs/vault.crt"
    tls_key_file  = "/vault/certs/vault-key.pem"
  }
  listener "tcp" {
    address     = "0.0.0.0:8201"
    tls_disable = true  # HTTP for vaultrs
  }
  
  # RAFT STORAGE (production-ready, HA-capable)
  storage "raft" {
    path    = "/vault/raft"
    node_id = "vault-ce-node1"
  }
  ```
  
- [x] Update `deploy/compose/ce.dev.yml`:
  ```yaml
  vault:
    volumes:
      - ../vault/certs:/vault/certs:ro
      - ../vault/config:/vault/config:ro
      - ../vault/policies:/vault/policies:ro
      - vault_raft:/vault/raft  # Raft storage volume
    ports:
      - "8200:8200"  # HTTPS
      - "8201:8201"  # HTTP (internal)
      - "8202:8202"  # Cluster
  
  volumes:
    vault_raft:
      driver: local
  ```
  
- [x] Initialize Vault (5 keys, 3 threshold - production-ready):
  ```bash
  docker exec -it ce_vault vault operator init
  # User saved: 5 unseal keys + root token to password manager
  ```
  
- [x] Unseal Vault (3 of 5 keys):
  ```bash
  docker exec -it ce_vault vault operator unseal  # key 1
  docker exec -it ce_vault vault operator unseal  # key 2
  docker exec -it ce_vault vault operator unseal  # key 3
  ```
  
- [x] Enable Transit engine:
  ```bash
  vault secrets enable transit
  vault write -f transit/keys/profile-signing
  ```
  
- [x] Test HTTPS + Raft:
  ```bash
  curl -k https://localhost:8200/v1/sys/health
  # Returns: {"initialized":true,"sealed":false,"storage_type":"raft","ha_enabled":true}
  ```

**Deliverable:** Vault HTTPS + Raft storage enabled ✅

---

### A2: AppRole Authentication (3 hours) ✅ COMPLETE (Recovery 2025-11-07)

**Recovery Note:** A2 completed with fresh AppRole credentials (2025-11-07 20:10)

- [x] Create `deploy/vault/policies/controller-policy.hcl` (already exists, correct paths):
  ```hcl
  # Transit engine access for profile signing
  path "transit/hmac/profile-signing" {
    capabilities = ["create", "update"]
  }
  
  path "transit/verify/profile-signing" {
    capabilities = ["create", "update"]
  }
  
  # Read transit key metadata + key creation
  path "transit/keys/profile-signing" {
    capabilities = ["read", "create", "update"]
  }
  
  # Token management
  path "auth/token/renew-self" {
    capabilities = ["update"]
  }
  
  path "auth/token/lookup-self" {
    capabilities = ["read"]
  }
  ```

- [x] Create `scripts/vault-setup-approle.sh`
- [x] Run script, save credentials to `.env` (ROLE_ID: b9319621-f88f-62ac-2bea-503cdbccf0d4)
- [x] Update `src/vault/mod.rs` - add VaultAuth enum + from_env()
- [x] Update `src/vault/client.rs` - add AppRole login + token renewal
- [x] Update `deploy/vault/config.hcl` - dual listener (HTTPS 8200, HTTP 8201)
- [x] Update `deploy/compose/ce.dev.yml` - AppRole env vars + dual ports
- [x] Create Transit key "profile-signing" (with root token)
- [x] Test: Controller starts → AppRole login → Profile signing succeeds ✅

**Architecture Decision:** Dual listener approach due to vaultrs 0.7.x TLS limitation  
**Security:** Credentials regenerated twice after exposure incidents, stored in password manager

**Deliverable:** Vault AppRole authentication working ✅

---

### A3: Persistent Storage (Raft) (0 hours) ✅ COMPLETE (Integrated with A1)

**NOTE:** A3 was completed during A1 recovery (2025-11-07 20:00 UTC) when Raft storage was configured.

**What we did:**
- [x] Updated `deploy/vault/config/vault.hcl` to use Raft:
  ```hcl
  storage "raft" {
    path    = "/vault/raft"
    node_id = "vault-ce-node1"
  }
  ```
  
- [x] Updated `deploy/compose/ce.dev.yml` to use vault_raft volume:
  ```yaml
  vault:
    volumes:
      - vault_raft:/vault/raft
  
  volumes:
    vault_raft:
      driver: local
  ```
  
- [x] Initialized Vault with 5 keys, 3 threshold (production-ready):
  ```bash
  docker exec -it ce_vault vault operator init
  # Output: 5 unseal keys + 1 root token
  # User saved all to password manager
  ```
  
- [x] Unsealing script exists: `scripts/vault-unseal.sh`
  
- [x] Verified Raft storage working:
  ```bash
  docker exec ce_vault vault status
  # Output: Storage Type = raft, HA Enabled = true
  ```

**Result:**
- ✅ Raft integrated storage active
- ✅ HA-capable (high availability ready)
- ✅ Persistent across restarts (vault_raft Docker volume)
- ✅ Production-ready configuration

**Deliverable:** Vault persistent Raft storage operational ✅

---

### A4: Audit Device (1 hour) ✅ COMPLETE (Recovery 2025-11-07 20:28)

**Recovery Note:** A4 completed during recovery session (2025-11-07 20:28 UTC)

- [x] Update `deploy/compose/ce.dev.yml`:
  ```yaml
  vault:
    volumes:
      - vault_logs:/vault/logs  # Added for audit
  
  volumes:
    vault_logs:  # Added volume
      driver: local
  ```

- [x] Enable audit device:
  ```bash
  export VAULT_TOKEN='<root-token-from-password-manager>'
  docker exec -e VAULT_TOKEN=$VAULT_TOKEN ce_vault \
    vault audit enable file file_path=/vault/logs/audit.log
  ```
  **Output:** `Success! Enabled the file audit device at: file/`

- [x] Verify audit device enabled:
  ```bash
  docker exec -e VAULT_TOKEN=$VAULT_TOKEN ce_vault vault audit list
  ```
  **Output:**
  ```
  Path     Type    Description
  ----     ----    -----------
  file/    file    n/a
  ```

- [x] Test audit logging:
  ```bash
  # Trigger audit entry
  docker exec -e VAULT_TOKEN=$VAULT_TOKEN ce_vault vault secrets list
  
  # Verify audit log contains entries
  docker exec ce_vault cat /vault/logs/audit.log | head -20
  ```
  **Result:** ✅ JSON audit entries visible, showing:
  - Request/response pairs logged
  - Tokens HMAC-hashed (security best practice)
  - Operations: `sys/audit/file` enable, `sys/audit` list, `sys/mounts` read
  - Timestamps, client IDs, mount points all captured

- [x] Log rotation configured (production-ready):
  **Approach 1 (Docker Native - ACTIVE):**
  ```yaml
  vault:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "30"
        compress: "true"
  ```
  **Approach 2 (System Logrotate - Optional):**
  - Script: `scripts/setup-vault-logrotate.sh`
  - Config: `deploy/vault/logrotate.conf`
  - Documentation: `docs/operations/VAULT-LOG-ROTATION.md`
  
  **Production-Ready:** Docker logging handles container logs automatically. System logrotate available for volume-based audit logs if needed.

**Deliverable:** Vault audit logging enabled ✅

**Time Spent:** 0.5 hours (vs 1 hour estimated)

**Architecture Notes:**
- Audit log path: `/vault/logs/audit.log` (inside container)
- Docker volume: `vault_logs` (persists across restarts)
- Format: JSON (one entry per line)
- Security: Tokens are HMAC-hashed (not plaintext)
- Captures: All Vault API operations (read, write, delete)

**Next Task:** A5 - Signature Verification (2 hours)

---

### A5: Signature Verification on Profile Load (2 hours)

- [ ] Create `src/vault/verify.rs`:
  ```rust
  pub async fn verify_profile_signature(
      profile: &Profile,
      vault_client: &VaultClient,
  ) -> Result<bool> {
      // Extract signature
      let sig = profile.signature.as_ref()
          .ok_or("Profile not signed")?;
      
      // Remove signature field for verification
      let mut profile_copy = profile.clone();
      profile_copy.signature = None;
      
      // Serialize to canonical JSON
      let canonical = serde_json::to_string(&profile_copy)?;
      
      // Call Vault verify endpoint
      let response = vault_client.request(
          Method::POST,
          &format!("/v1/transit/verify/{}/sha2-256", sig.vault_key),
          Some(json!({
              "input": base64::encode(canonical),
              "hmac": sig.signature
          }))
      ).await?;
      
      Ok(response["data"]["valid"].as_bool().unwrap_or(false))
  }
  ```

- [ ] Update `src/controller/src/routes/profiles.rs` - add verification:
  ```rust
  pub async fn get_profile(...) -> Result<Json<Profile>> {
      let profile: Profile = load_from_db(&role).await?;
      
      // Verify signature (Phase 6)
      if let Some(vault) = &state.vault_client {
          let valid = verify_profile_signature(&profile, vault).await
              .unwrap_or(false);
          
          if !valid {
              error!("Profile signature verification failed", role = %role);
              return Err(ProfileError::Forbidden(
                  "Profile signature invalid - possible tampering".into()
              ));
          }
      }
      
      Ok(Json(profile))
  }
  ```

- [ ] Test: Load signed profile → 200 OK
- [ ] Test: Tamper profile in DB → 403 Forbidden
- [ ] Test: Load unsigned profile → 403 Forbidden (reject unsigned)

**Deliverable:** Profile signature verification enforced ✅

---

### A6: Vault Integration Test (1 hour)

- [ ] Create `tests/integration/phase6-vault-production.sh`:
  ```bash
  #!/bin/bash
  # Test Vault production setup
  
  # Test TLS
  curl --cacert deploy/vault/certs/vault.crt https://localhost:8200/v1/sys/health
  
  # Test AppRole auth
  VAULT_TOKEN=$(vault write -field=token auth/approle/login \
    role_id=$VAULT_ROLE_ID secret_id=$VAULT_SECRET_ID)
  
  # Test signing
  curl -X POST http://localhost:8088/admin/profiles/finance/publish \
    -H "Authorization: Bearer $JWT"
  
  # Test verification
  PROFILE=$(curl http://localhost:8088/profiles/finance -H "Authorization: Bearer $JWT")
  echo "$PROFILE" | jq '.signature.signature' | grep "vault:v1:"
  
  # Test tamper detection
  docker exec ce_postgres psql -U postgres -d orchestrator \
    -c "UPDATE profiles SET data = jsonb_set(data, '{description}', '\"hacked\"') WHERE role = 'finance'"
  
  curl http://localhost:8088/profiles/finance -H "Authorization: Bearer $JWT"
  # Should return 403 Forbidden
  
  echo "✅ Vault production tests passed"
  ```

- [ ] Run test suite
- [ ] Verify: All tests pass ✅

**Deliverable:** Vault production validated ✅

**Workstream A Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM B: Admin UI (SvelteKit) (3 days)

### B1: SvelteKit Setup (2 hours)

- [ ] Create Admin UI:
  ```bash
  cd /home/papadoc/Gooseprojects/goose-org-twin
  npm create svelte@latest admin-ui
  # Choose: Skeleton project, TypeScript, Prettier, Playwright
  
  cd admin-ui
  npm install
  npm install -D tailwindcss postcss autoprefixer @tailwindcss/typography
  npm install d3 @types/d3
  npm install monaco-editor
  npm install @sveltejs/adapter-static
  ```

- [ ] Configure Tailwind (`tailwind.config.js`)
- [ ] Configure adapter-static (`svelte.config.js`)
- [ ] Create base layout (`src/routes/+layout.svelte`)
- [ ] Test: `npm run dev` → http://localhost:5173 works

**Deliverable:** SvelteKit project scaffold ✅

---

### B2: Dashboard Page (4 hours)

- [ ] Create `src/routes/+page.svelte`:
  ```svelte
  <script lang="ts">
    import { onMount } from 'svelte';
    import OrgChart from '$lib/components/OrgChart.svelte';
    
    let orgData = null;
    let stats = { profiles: 0, active_sessions: 0 };
    
    onMount(async () => {
      const jwt = localStorage.getItem('jwt');
      
      // Fetch org tree
      const orgRes = await fetch('http://localhost:8088/admin/org/tree', {
        headers: { 'Authorization': `Bearer ${jwt}` }
      });
      orgData = await orgRes.json();
      
      // Fetch stats
      const profilesRes = await fetch('http://localhost:8088/admin/profiles');
      stats.profiles = (await profilesRes.json()).length;
    });
  </script>
  
  <div class="container mx-auto p-6">
    <h1 class="text-3xl font-bold mb-6">Dashboard</h1>
    
    <div class="grid grid-cols-3 gap-4 mb-6">
      <div class="card bg-blue-50 p-4">
        <h3 class="font-bold">Profiles</h3>
        <p class="text-2xl">{stats.profiles}</p>
      </div>
      <div class="card bg-green-50 p-4">
        <h3 class="font-bold">Active Sessions</h3>
        <p class="text-2xl">{stats.active_sessions}</p>
      </div>
      <div class="card bg-yellow-50 p-4">
        <h3 class="font-bold">Vault Status</h3>
        <p class="text-2xl">Unsealed</p>
      </div>
    </div>
    
    {#if orgData}
      <OrgChart data={orgData} />
    {/if}
  </div>
  ```

- [ ] Create `src/lib/components/OrgChart.svelte` (D3.js tree visualization)
- [ ] Test: Dashboard loads, shows stats

**Deliverable:** Dashboard page functional ✅

---

### B3: Profiles Page (6 hours)

- [ ] Create `src/routes/profiles/+page.svelte`:
  - [ ] Profile list sidebar (6 roles)
  - [ ] Monaco YAML editor (for selected profile)
  - [ ] Publish button (calls POST /admin/profiles/{role}/publish)
  - [ ] Policy tester (input: tool name, output: allow/deny)

- [ ] Create `src/lib/components/MonacoEditor.svelte`:
  ```svelte
  <script lang="ts">
    import { onMount } from 'svelte';
    import * as monaco from 'monaco-editor';
    
    export let value = '';
    export let language = 'yaml';
    
    let editorContainer: HTMLElement;
    let editor: monaco.editor.IStandaloneCodeEditor;
    
    onMount(() => {
      editor = monaco.editor.create(editorContainer, {
        value,
        language,
        theme: 'vs-dark',
        minimap: { enabled: false }
      });
      
      editor.onDidChangeModelContent(() => {
        value = editor.getValue();
      });
    });
  </script>
  
  <div bind:this={editorContainer} class="h-96"></div>
  ```

- [ ] Test: Edit profile, publish, verify signature created

**Deliverable:** Profile management page functional ✅

---

### B4: Org Chart Page (4 hours)

- [ ] Create `src/routes/org/+page.svelte`:
  - [ ] CSV upload widget (drag-and-drop)
  - [ ] Import history table (fetches GET /admin/org/imports)
  - [ ] D3.js tree visualization (same as Dashboard)

- [ ] Create `src/lib/components/CsvUpload.svelte`:
  ```svelte
  <script lang="ts">
    async function handleUpload(file: File) {
      const formData = new FormData();
      formData.append('file', file);
      
      const jwt = localStorage.getItem('jwt');
      const response = await fetch('http://localhost:8088/admin/org/import', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${jwt}` },
        body: formData
      });
      
      const result = await response.json();
      alert(`Imported ${result.users_created} users`);
    }
  </script>
  
  <div class="dropzone" on:drop={handleDrop}>
    Drop CSV file here
  </div>
  ```

- [ ] Test: Upload CSV, view tree

**Deliverable:** Org chart management functional ✅

---

### B5: Audit Logs Page (3 hours)

- [ ] Create `src/routes/audit/+page.svelte`:
  - [ ] Filters (event_type, role, date range, trace_id)
  - [ ] Table (paginated)
  - [ ] Export CSV button

- [ ] Test: View audit logs, filter, export

**Deliverable:** Audit logs viewer functional ✅

---

### B6: Settings Page (3 hours)

- [ ] Create `src/routes/settings/+page.svelte`:
  - [ ] Vault status section
  - [ ] System variables form
  - [ ] Service health checks (6 services)

- [ ] Test: Update settings, verify saved

**Deliverable:** Settings page functional ✅

---

### B7: JWT Auth Integration (2 hours)

- [ ] Create `src/lib/auth.ts`:
  ```typescript
  export async function login(email: string, password: string): Promise<string> {
    const response = await fetch('http://localhost:8080/realms/dev/protocol/openid-connect/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        client_id: 'goose-controller',
        grant_type: 'password',
        username: email,
        password: password,
        scope: 'openid'
      })
    });
    
    const data = await response.json();
    return data.access_token;
  }
  ```

- [ ] Create login page (`src/routes/login/+page.svelte`)
- [ ] Add auth guard to all admin routes
- [ ] Test: Login → Redirects to dashboard

**Deliverable:** Authentication flow working ✅

---

### B8: Build & Deploy (1 hour)

- [ ] Build: `cd admin-ui && npm run build`
- [ ] Update Controller to serve static files:
  ```rust
  // src/main.rs
  use tower_http::services::ServeDir;
  
  let app = Router::new()
      .route("/profiles/:role", get(get_profile))
      // ... other routes
      .nest_service("/admin", ServeDir::new("admin-ui/build"));
  ```

- [ ] Test: http://localhost:8088/admin → Dashboard loads

**Deliverable:** Admin UI deployed ✅

**Workstream B Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM C: Privacy Guard Proxy Service (3 days)

### C1: Create Proxy Service Scaffold (4 hours)

- [ ] Create directory structure:
  ```bash
  mkdir -p src/privacy-guard-proxy/src
  ```

- [ ] Create `src/privacy-guard-proxy/Cargo.toml`:
  ```toml
  [package]
  name = "privacy-guard-proxy"
  version = "0.1.0"
  edition = "2021"
  
  [dependencies]
  axum = "0.7"
  tokio = { version = "1", features = ["full"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  reqwest = { version = "0.11", features = ["json"] }
  tracing = "0.1"
  tracing-subscriber = "0.3"
  tower-http = "0.5"
  ```

- [ ] Create `src/privacy-guard-proxy/src/main.rs`:
  ```rust
  use axum::{Router, routing::{get, post}};
  use std::net::SocketAddr;
  
  mod proxy;
  mod privacy_client;
  mod token_store;
  
  #[tokio::main]
  async fn main() {
      tracing_subscriber::fmt::init();
      
      let app = Router::new()
          .route("/health", get(health))
          .route("/v1/chat/completions", post(proxy::proxy_chat_completions));
      
      let addr = SocketAddr::from(([0, 0, 0, 0], 8090));
      let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
      
      tracing::info!("Privacy Guard Proxy listening on {}", addr);
      axum::serve(listener, app).await.unwrap();
  }
  
  async fn health() -> &'static str {
      "ok"
  }
  ```

- [ ] Test: `cargo run` → Proxy starts on port 8090

**Deliverable:** Proxy scaffold working ✅

---

### C2: Implement Privacy Guard Client (4 hours)

- [ ] Create `src/privacy-guard-proxy/src/privacy_client.rs`:
  ```rust
  use serde::{Deserialize, Serialize};
  
  #[derive(Clone)]
  pub struct PrivacyClient {
      base_url: String,
      client: reqwest::Client,
  }
  
  #[derive(Serialize)]
  struct ScanRequest {
      text: String,
      mode: String,
      tenant_id: String,
  }
  
  #[derive(Deserialize)]
  pub struct ScanResponse {
      pub detections: Vec<Detection>,
  }
  
  #[derive(Deserialize, Clone)]
  pub struct Detection {
      pub entity_type: String,
      pub text: String,
      pub start: usize,
      pub end: usize,
  }
  
  #[derive(Serialize)]
  struct MaskRequest {
      text: String,
      method: String,
      mode: String,
      tenant_id: String,
  }
  
  #[derive(Deserialize)]
  pub struct MaskResponse {
      pub masked_text: String,
      pub session_id: String,
      pub redactions: Vec<Redaction>,
  }
  
  #[derive(Deserialize, Clone)]
  pub struct Redaction {
      pub entity_type: String,
      pub original: String,
      pub replacement: String,
  }
  
  impl PrivacyClient {
      pub fn new(base_url: String) -> Self {
          Self {
              base_url,
              client: reqwest::Client::new(),
          }
      }
      
      pub async fn scan(&self, text: &str, tenant_id: &str) -> Result<ScanResponse> {
          let response = self.client
              .post(format!("{}/guard/scan", self.base_url))
              .json(&ScanRequest {
                  text: text.to_string(),
                  mode: "hybrid".to_string(),
                  tenant_id: tenant_id.to_string(),
              })
              .send()
              .await?;
          
          Ok(response.json().await?)
      }
      
      pub async fn mask(&self, text: &str, tenant_id: &str) -> Result<MaskResponse> {
          let response = self.client
              .post(format!("{}/guard/mask", self.base_url))
              .json(&MaskRequest {
                  text: text.to_string(),
                  method: "pseudonym".to_string(),
                  mode: "hybrid".to_string(),
                  tenant_id: tenant_id.to_string(),
              })
              .send()
              .await?;
          
          Ok(response.json().await?)
      }
  }
  ```

- [ ] Test: Client can call Privacy Guard API

**Deliverable:** Privacy Guard client functional ✅

---

### C3: Implement Token Store (2 hours)

- [ ] Create `src/privacy-guard-proxy/src/token_store.rs`:
  ```rust
  use std::collections::HashMap;
  use std::sync::{Arc, Mutex};
  use super::privacy_client::Redaction;
  
  pub struct TokenStore {
      store: Arc<Mutex<HashMap<String, Vec<Redaction>>>>,
  }
  
  impl TokenStore {
      pub fn new() -> Self {
          Self {
              store: Arc::new(Mutex::new(HashMap::new())),
          }
      }
      
      pub fn insert(&self, session_id: String, redactions: Vec<Redaction>) {
          self.store.lock().unwrap().insert(session_id, redactions);
      }
      
      pub fn get(&self, session_id: &str) -> Option<Vec<Redaction>> {
          self.store.lock().unwrap().get(session_id).cloned()
      }
      
      pub fn remove(&self, session_id: &str) {
          self.store.lock().unwrap().remove(session_id);
      }
      
      pub fn unmask(&self, session_id: &str, text: &str) -> String {
          if let Some(redactions) = self.get(session_id) {
              let mut result = text.to_string();
              for redaction in redactions {
                  result = result.replace(&redaction.replacement, &redaction.original);
              }
              result
          } else {
              text.to_string()
          }
      }
  }
  ```

- [ ] Test: Store tokens, retrieve, unmask

**Deliverable:** Token storage working ✅

---

### C4: Implement Proxy Logic (8 hours)

- [ ] Create `src/privacy-guard-proxy/src/proxy.rs`:
  ```rust
  use axum::{extract::State, Json};
  use serde::{Deserialize, Serialize};
  use tracing::{info, warn};
  
  #[derive(Clone)]
  pub struct ProxyState {
      pub privacy_client: PrivacyClient,
      pub token_store: TokenStore,
      pub openrouter_url: String,
  }
  
  #[derive(Deserialize)]
  pub struct ChatRequest {
      pub model: String,
      pub messages: Vec<Message>,
      #[serde(flatten)]
      pub other: serde_json::Value,
  }
  
  #[derive(Serialize, Deserialize, Clone)]
  pub struct Message {
      pub role: String,
      pub content: String,
  }
  
  #[derive(Serialize, Deserialize)]
  pub struct ChatResponse {
      pub id: String,
      pub choices: Vec<Choice>,
      #[serde(flatten)]
      pub other: serde_json::Value,
  }
  
  #[derive(Serialize, Deserialize)]
  pub struct Choice {
      pub index: usize,
      pub message: Message,
      pub finish_reason: Option<String>,
  }
  
  pub async fn proxy_chat_completions(
      State(state): State<ProxyState>,
      Json(mut req): Json<ChatRequest>,
  ) -> Result<Json<ChatResponse>, StatusCode> {
      // Extract last user message
      let user_msg = req.messages.iter()
          .filter(|m| m.role == "user")
          .last()
          .map(|m| m.content.clone())
          .unwrap_or_default();
      
      if user_msg.is_empty() {
          // No user message, pass through
          return forward_to_llm(&state, &req).await;
      }
      
      // Scan for PII
      let scan = state.privacy_client.scan(&user_msg, "proxy-user").await
          .map_err(|e| {
              warn!("Privacy Guard scan failed: {}", e);
              StatusCode::INTERNAL_SERVER_ERROR
          })?;
      
      if scan.detections.is_empty() {
          info!("No PII detected, passing through");
          return forward_to_llm(&state, &req).await;
      }
      
      info!("Detected {} PII entities, masking", scan.detections.len());
      
      // Mask PII
      let mask_result = state.privacy_client.mask(&user_msg, "proxy-user").await
          .map_err(|e| {
              warn!("Privacy Guard mask failed: {}", e);
              StatusCode::INTERNAL_SERVER_ERROR
          })?;
      
      let session_id = mask_result.session_id.clone();
      
      // Store tokens for unmask
      state.token_store.insert(session_id.clone(), mask_result.redactions);
      
      // Replace user message with masked version
      for msg in req.messages.iter_mut().rev() {
          if msg.role == "user" {
              msg.content = mask_result.masked_text.clone();
              break;
          }
      }
      
      info!("Forwarding masked request to LLM");
      
      // Forward to real LLM
      let mut llm_response = forward_to_llm(&state, &req).await?;
      
      // Unmask response
      if let Some(choice) = llm_response.choices.first_mut() {
          let unmasked = state.token_store.unmask(&session_id, &choice.message.content);
          choice.message.content = unmasked;
      }
      
      // Clean up tokens
      state.token_store.remove(&session_id);
      
      info!("Returned unmasked response to user");
      Ok(llm_response)
  }
  
  async fn forward_to_llm(
      state: &ProxyState,
      req: &ChatRequest,
  ) -> Result<Json<ChatResponse>, StatusCode> {
      let client = reqwest::Client::new();
      
      // Determine target URL based on model
      let target_url = if req.model.contains('/') {
          // OpenRouter format: "anthropic/claude-3.5-sonnet"
          format!("{}/api/v1/chat/completions", state.openrouter_url)
      } else {
          // Direct provider (Anthropic, OpenAI)
          // For now, default to OpenRouter
          format!("{}/api/v1/chat/completions", state.openrouter_url)
      };
      
      let response = client
          .post(&target_url)
          .json(req)
          .send()
          .await
          .map_err(|e| {
              warn!("LLM request failed: {}", e);
              StatusCode::BAD_GATEWAY
          })?;
      
      let llm_response: ChatResponse = response.json().await
          .map_err(|e| {
              warn!("LLM response parse failed: {}", e);
              StatusCode::BAD_GATEWAY
          })?;
      
      Ok(Json(llm_response))
  }
  ```

- [ ] Test: Send request with PII → Verify masked → Verify unmasked response

**Deliverable:** Proxy logic working ✅

---

### C5: Add Docker Service (2 hours)

- [ ] Create `src/privacy-guard-proxy/Dockerfile`:
  ```dockerfile
  FROM rust:1.83 as builder
  WORKDIR /app
  COPY Cargo.toml Cargo.lock ./
  COPY src ./src
  RUN cargo build --release
  
  FROM debian:bookworm-slim
  RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
  COPY --from=builder /app/target/release/privacy-guard-proxy /usr/local/bin/
  EXPOSE 8090
  CMD ["privacy-guard-proxy"]
  ```

- [ ] Update `deploy/compose/ce.dev.yml`:
  ```yaml
  privacy-guard-proxy:
    build:
      context: ../../src/privacy-guard-proxy
      dockerfile: Dockerfile
    image: ghcr.io/jefh507/privacy-guard-proxy:0.1.0
    container_name: ce_privacy_guard_proxy
    ports:
      - "8090:8090"
    environment:
      PRIVACY_GUARD_URL: http://privacy-guard:8089
      OPENROUTER_URL: https://openrouter.ai
      RUST_LOG: info
    depends_on:
      - privacy-guard
  ```

- [ ] Test: `docker compose up privacy-guard-proxy`

**Deliverable:** Proxy service containerized ✅

---

### C6: Integration Tests (4 hours)

- [ ] Create `tests/integration/phase6-privacy-proxy.sh`:
  ```bash
  #!/bin/bash
  # Test Privacy Guard Proxy
  
  # Test 1: Health check
  curl http://localhost:8090/health
  
  # Test 2: Pass-through (no PII)
  RESPONSE=$(curl -X POST http://localhost:8090/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{
      "model": "anthropic/claude-3.5-sonnet",
      "messages": [{"role": "user", "content": "What is 2+2?"}]
    }')
  
  echo "$RESPONSE" | jq '.choices[0].message.content'
  
  # Test 3: PII masking
  RESPONSE=$(curl -X POST http://localhost:8090/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{
      "model": "anthropic/claude-3.5-sonnet",
      "messages": [{"role": "user", "content": "My email is john@acme.com"}]
    }')
  
  # Verify: OpenRouter request log shows EMAIL_ token (not john@acme.com)
  # Verify: User response contains john@acme.com (unmasked)
  
  echo "✅ Privacy Guard Proxy tests passed"
  ```

- [ ] Run tests, verify all pass

**Deliverable:** Proxy integration validated ✅

**Workstream C Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM D: Profile Setup Scripts (1 day)

### D1: Create setup-profile.sh (4 hours)

- [ ] Create `scripts/setup-profile.sh`:
  ```bash
  #!/bin/bash
  # Setup user profile from Controller
  # Usage: ./scripts/setup-profile.sh <role>
  
  set -e
  
  ROLE="${1:-finance}"
  CONTROLLER_URL="${CONTROLLER_URL:-http://localhost:8088}"
  KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
  CONFIG_DIR="${HOME}/.config/goose"
  
  echo "🔐 Setting up Goose profile: $ROLE"
  echo ""
  
  # 1. Authenticate
  echo "Authentication required"
  read -p "Email: " EMAIL
  read -sp "Password: " PASSWORD
  echo ""
  
  JWT=$(curl -s -X POST "$KEYCLOAK_URL/realms/dev/protocol/openid-connect/token" \
    -d "client_id=goose-controller" \
    -d "grant_type=password" \
    -d "username=$EMAIL" \
    -d "password=$PASSWORD" \
    -d "scope=openid" | jq -r '.access_token')
  
  if [ "$JWT" == "null" ] || [ -z "$JWT" ]; then
    echo "❌ Authentication failed"
    exit 1
  fi
  
  echo "✅ Authenticated as $EMAIL"
  echo ""
  
  # 2. Fetch profile
  echo "📥 Fetching profile from Controller..."
  PROFILE=$(curl -s -H "Authorization: Bearer $JWT" \
    "$CONTROLLER_URL/profiles/$ROLE")
  
  if echo "$PROFILE" | jq -e '.error' > /dev/null 2>&1; then
    echo "❌ Profile fetch failed: $(echo $PROFILE | jq -r '.error')"
    exit 1
  fi
  
  echo "✅ Profile retrieved: $ROLE"
  
  # 3. Create config directory
  mkdir -p "$CONFIG_DIR"
  
  # 4. Fetch config.yaml
  echo "📥 Downloading config.yaml..."
  curl -s -H "Authorization: Bearer $JWT" \
    "$CONTROLLER_URL/profiles/$ROLE/config" \
    > "$CONFIG_DIR/config.yaml"
  
  # 5. Fetch goosehints
  echo "📥 Downloading .goosehints..."
  curl -s -H "Authorization: Bearer $JWT" \
    "$CONTROLLER_URL/profiles/$ROLE/goosehints" \
    > "$CONFIG_DIR/.goosehints"
  
  # 6. Fetch gooseignore
  echo "📥 Downloading .gooseignore..."
  curl -s -H "Authorization: Bearer $JWT" \
    "$CONTROLLER_URL/profiles/$ROLE/gooseignore" \
    > "$CONFIG_DIR/.gooseignore"
  
  # 7. Add Privacy Guard Proxy to config
  PRIVACY_MODE=$(echo "$PROFILE" | jq -r '.privacy.mode')
  if [ "$PRIVACY_MODE" != "null" ] && [ "$PRIVACY_MODE" != "off" ]; then
    echo ""
    echo "🛡️ Enabling Privacy Guard (mode: $PRIVACY_MODE)"
    echo "" >> "$CONFIG_DIR/config.yaml"
    echo "# Privacy Guard Proxy (auto-configured from profile)" >> "$CONFIG_DIR/config.yaml"
    echo "GOOSE_PROVIDER__OPENROUTER_BASE_URL: http://localhost:8090" >> "$CONFIG_DIR/config.yaml"
  fi
  
  echo ""
  echo "✅ Profile setup complete!"
  echo ""
  echo "📋 Summary:"
  echo "   Role: $ROLE"
  echo "   Display Name: $(echo $PROFILE | jq -r '.display_name')"
  echo "   Provider: $(echo $PROFILE | jq -r '.providers.primary.model')"
  echo "   Privacy Mode: $PRIVACY_MODE"
  echo "   Extensions: $(echo $PROFILE | jq -r '.extensions | length') enabled"
  echo ""
  echo "📁 Files saved to: $CONFIG_DIR"
  echo "   - config.yaml"
  echo "   - .goosehints"
  echo "   - .gooseignore"
  echo ""
  echo "🚀 Launch Goose Desktop:"
  echo "   goose session start"
  ```

- [ ] Make executable: `chmod +x scripts/setup-profile.sh`
- [ ] Test with finance role:
  ```bash
  ./scripts/setup-profile.sh finance
  ```

**Deliverable:** Setup script working ✅

---

### D2: Create Convenience Wrapper Scripts (2 hours)

- [ ] Create `scripts/goose-finance.sh`:
  ```bash
  #!/bin/bash
  ./scripts/setup-profile.sh finance
  goose session start
  ```

- [ ] Create scripts for all 6 roles:
  - `goose-finance.sh`
  - `goose-legal.sh`
  - `goose-developer.sh`
  - `goose-hr.sh`
  - `goose-executive.sh`
  - `goose-support.sh`

- [ ] Make all executable: `chmod +x scripts/goose-*.sh`
- [ ] Test each wrapper script

**Deliverable:** Convenience scripts for all roles ✅

---

### D3: Test All 6 Roles (2 hours)

- [ ] Test finance profile:
  - [ ] Run `./scripts/setup-profile.sh finance`
  - [ ] Verify config.yaml has Claude 3.5 Sonnet
  - [ ] Verify .goosehints has Finance context
  - [ ] Verify Privacy Guard enabled (strict mode)

- [ ] Test legal profile:
  - [ ] Run `./scripts/setup-profile.sh legal`
  - [ ] Verify config.yaml has Ollama (local-only)
  - [ ] Verify Privacy Guard enabled (strict + local_only)

- [ ] Test developer profile
- [ ] Test hr profile
- [ ] Test executive profile
- [ ] Test support profile

**Deliverable:** All 6 roles validated ✅

**Workstream D Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM E: Wire Lifecycle into Routes (1 day)

### E1: Update Session Routes (4 hours)

- [ ] Update `src/controller/src/routes/sessions.rs`:
  ```rust
  use crate::lifecycle::SessionLifecycle;
  
  pub async fn update_session(
      State(state): State<AppState>,
      Path(session_id): Path<Uuid>,
      Json(req): Json<UpdateSessionRequest>,
  ) -> Result<Json<Session>, SessionError> {
      // Get retention days from env (default 7)
      let retention_days = env::var("SESSION_RETENTION_DAYS")
          .ok()
          .and_then(|s| s.parse().ok())
          .unwrap_or(7);
      
      // Create lifecycle manager
      let lifecycle = SessionLifecycle::new(
          state.db_pool.as_ref()
              .ok_or(SessionError::InternalError("Database not configured".into()))?
              .clone(),
          retention_days
      );
      
      // Transition session (validates state machine)
      let updated = lifecycle.transition(session_id, req.status)
          .await
          .map_err(|e| match e {
              TransitionError::SessionNotFound(id) => 
                  SessionError::NotFound(id.to_string()),
              TransitionError::InvalidTransition { from, to } => 
                  SessionError::BadRequest(
                      format!("Invalid transition from {:?} to {:?}", from, to)
                  ),
              TransitionError::DatabaseError(msg) => 
                  SessionError::InternalError(msg),
          })?;
      
      Ok(Json(updated))
  }
  ```

- [ ] Add helper methods (activate, complete, fail):
  ```rust
  pub async fn activate_session(
      State(state): State<AppState>,
      Path(session_id): Path<Uuid>,
  ) -> Result<Json<Session>, SessionError> {
      let retention_days = env::var("SESSION_RETENTION_DAYS")
          .ok().and_then(|s| s.parse().ok()).unwrap_or(7);
      
      let lifecycle = SessionLifecycle::new(
          state.db_pool.as_ref().unwrap().clone(),
          retention_days
      );
      
      let session = lifecycle.activate(session_id).await
          .map_err(|e| SessionError::BadRequest(e.to_string()))?;
      
      Ok(Json(session))
  }
  ```

**Deliverable:** Lifecycle integrated into routes ✅

---

### E2: Add Auto-Expiration Cron (2 hours)

- [ ] Create `scripts/expire-sessions.sh`:
  ```bash
  #!/bin/bash
  # Expire old sessions (run via cron)
  
  RETENTION_DAYS="${SESSION_RETENTION_DAYS:-7}"
  
  docker exec ce_postgres psql -U postgres -d orchestrator -c "
    UPDATE sessions 
    SET status = 'expired', 
        updated_at = NOW() 
    WHERE status IN ('pending', 'active') 
      AND created_at < NOW() - INTERVAL '$RETENTION_DAYS days'
  "
  ```

- [ ] Add to crontab (optional):
  ```cron
  0 2 * * * /path/to/expire-sessions.sh
  ```

- [ ] Test: Create old session, run script, verify expired

**Deliverable:** Auto-expiration working ✅

---

### E3: Integration Tests (2 hours)

- [ ] Create `tests/integration/phase6-lifecycle.sh`:
  ```bash
  # Test invalid state transitions
  
  # Create session (pending)
  SESSION_ID=$(curl -X POST http://localhost:8088/sessions | jq -r '.session_id')
  
  # Try invalid transition: pending → completed (should fail)
  curl -X PUT http://localhost:8088/sessions/$SESSION_ID \
    -d '{"status": "completed"}' \
    -H "Content-Type: application/json"
  # Should return 400 Bad Request
  
  # Valid transition: pending → active
  curl -X PUT http://localhost:8088/sessions/$SESSION_ID \
    -d '{"status": "active"}'
  # Should return 200 OK
  
  echo "✅ Lifecycle validation working"
  ```

- [ ] Run tests, verify lifecycle enforces state machine

**Deliverable:** Lifecycle validation tested ✅

**Workstream E Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM F: Security Hardening (1 day)

### F1: Secrets Cleanup (2 hours)

- [ ] Grep for hardcoded secrets:
  ```bash
  grep -rn "password\|secret\|token\|api_key" src/ --exclude-dir=target
  ```
- [ ] Remove any hardcoded values
- [ ] Move to .env files
- [ ] Verify: `grep` finds no secrets in src/

**Deliverable:** No secrets in code ✅

---

### F2: Environment Variable Audit (2 hours)

- [ ] Grep all env vars:
  ```bash
  grep -rn "env::var\|std::env\|process.env" src/ | \
    grep -v "target/" | \
    awk -F'"' '{print $2}' | \
    sort | uniq > env_vars_audit.txt
  ```

- [ ] Categorize variables:
  - Required: VAULT_ADDR, DATABASE_URL, KEYCLOAK_URL
  - Optional: LOG_LEVEL, PORT, REDIS_URL
  - Secrets: VAULT_ROLE_ID, VAULT_SECRET_ID

- [ ] Document in README.md

**Deliverable:** Environment variables documented ✅

---

### F3: Create .env.example (1 hour)

- [ ] Create `deploy/compose/.env.example`:
  ```bash
  # Vault Configuration
  VAULT_ADDR=https://vault:8200
  VAULT_ROLE_ID=your-role-id-here
  VAULT_SECRET_ID=your-secret-id-here
  VAULT_CACERT=/vault/certs/vault.crt
  
  # Database
  DATABASE_URL=postgresql://postgres:your-password@postgres:5432/orchestrator
  
  # Keycloak
  KEYCLOAK_URL=http://keycloak:8080
  KEYCLOAK_ADMIN=admin
  KEYCLOAK_ADMIN_PASSWORD=your-admin-password
  
  # Privacy Guard
  PRIVACY_GUARD_URL=http://privacy-guard:8089
  GUARD_MODE=MASK
  
  # Privacy Guard Proxy
  PRIVACY_PROXY_URL=http://privacy-guard-proxy:8090
  OPENROUTER_URL=https://openrouter.ai
  
  # Controller
  CONTROLLER_URL=http://controller:8088
  CONTROLLER_PORT=8088
  
  # Redis
  REDIS_URL=redis://redis:6379
  
  # Session Management
  SESSION_RETENTION_DAYS=7
  IDEMPOTENCY_TTL=86400
  
  # Logging
  LOG_LEVEL=info
  RUST_LOG=info
  ```

- [ ] Add to .gitignore: `.env` (if not already)
- [ ] Document in README.md how to copy .env.example → .env

**Deliverable:** .env.example created ✅

---

### F4: Docker Compose Security (2 hours)

- [ ] Update `deploy/compose/ce.dev.yml` - add security options:
  ```yaml
  services:
    controller:
      security_opt:
        - no-new-privileges:true
      read_only: false  # Needs write for logs
      tmpfs:
        - /tmp
    
    privacy-guard:
      security_opt:
        - no-new-privileges:true
      read_only: false
      tmpfs:
        - /tmp
  ```

- [ ] Remove default passwords (use .env instead)
- [ ] Document security considerations in `deploy/compose/README.md`

**Deliverable:** Docker Compose hardened ✅

---

### F5: Create SECURITY.md (1 hour)

- [ ] Create `SECURITY.md`:
  ```markdown
  # Security Policy
  
  ## Supported Versions
  
  | Version | Supported          |
  | ------- | ------------------ |
  | 0.6.x   | :white_check_mark: |
  | 0.5.x   | :white_check_mark: |
  | < 0.5   | :x:                |
  
  ## Reporting a Vulnerability
  
  **DO NOT** open a public issue for security vulnerabilities.
  
  Email: security@example.com
  PGP Key: [Link to public key]
  
  Expected response time: 48 hours
  
  ## Security Best Practices
  
  - All secrets in .env files (never committed)
  - Vault AppRole authentication (not root token)
  - TLS enabled for Vault in production
  - Vault audit device logs all operations
  - Profile signatures verified on load
  - Privacy Guard masks PII before cloud LLM
  
  ## CVE Remediation Process
  
  1. Security team reviews CVE within 24h
  2. Patch developed within 7 days (critical) or 30 days (medium)
  3. Patch tested in staging
  4. Patch released with security advisory
  5. All users notified via GitHub Security Advisory
  ```

- [ ] Update README.md - add Security section:
  ```markdown
  ## Security
  
  - **Secrets Management:** All secrets in .env files (never committed)
  - **Vulnerability Reporting:** See [SECURITY.md](SECURITY.md)
  - **Vault Production:** TLS + AppRole + Audit enabled
  - **PII Protection:** Privacy Guard masks before LLM
  ```

**Deliverable:** Security documentation complete ✅

**Workstream F Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM G: Integration Testing (2 days)

### G1: Vault Production Tests (2 hours)

- [x] Script created: `tests/integration/phase6-vault-production.sh`
- [ ] Test TLS connection
- [ ] Test AppRole authentication
- [ ] Test profile signing
- [ ] Test signature verification
- [ ] Test tamper detection (403 on modified profile)
- [ ] Verify: 5/5 Vault tests pass ✅

---

### G2: Admin UI Smoke Tests (4 hours)

- [ ] Install Playwright:
  ```bash
  cd admin-ui
  npm install -D @playwright/test
  npx playwright install
  ```

- [ ] Create `admin-ui/tests/admin-ui.spec.ts`:
  ```typescript
  import { test, expect } from '@playwright/test';
  
  test('dashboard loads', async ({ page }) => {
    await page.goto('http://localhost:8088/admin');
    await expect(page.locator('h1')).toContainText('Dashboard');
  });
  
  test('edit and publish profile', async ({ page }) => {
    await page.goto('http://localhost:8088/admin/profiles');
    await page.click('text=Finance');
    await page.click('text=Edit');
    // Monaco editor interaction
    await page.click('text=Publish');
    await expect(page.locator('.success')).toBeVisible();
  });
  
  test('upload org chart CSV', async ({ page }) => {
    await page.goto('http://localhost:8088/admin/org');
    await page.setInputFiles('input[type=file]', 'tests/fixtures/org_chart.csv');
    await expect(page.locator('text=Imported')).toBeVisible();
  });
  ```

- [ ] Run: `npx playwright test`
- [ ] Verify: 8/8 UI tests pass ✅

---

### G3: Privacy Guard Proxy Tests (4 hours)

- [x] Script created: `tests/integration/phase6-privacy-proxy.sh`
- [ ] Test pass-through (no PII)
- [ ] Test masking (SSN, Email, Phone)
- [ ] Test unmasking (response contains original)
- [ ] Test multiple providers (OpenRouter, Anthropic)
- [ ] Verify: Privacy Guard called (check logs)
- [ ] Verify: LLM received masked text (not original)
- [ ] Verify: 6/6 proxy tests pass ✅

---

### G4: Profile Setup Scripts Tests (2 hours)

- [ ] Create `tests/integration/phase6-profile-setup.sh`:
  ```bash
  #!/bin/bash
  # Test profile setup script for all roles
  
  for role in finance legal developer hr executive support; do
    echo "Testing role: $role"
    
    # Run setup script (with test credentials)
    echo -e "testuser@example.com\ntestpassword" | ./scripts/setup-profile.sh $role
    
    # Verify files created
    test -f ~/.config/goose/config.yaml || exit 1
    test -f ~/.config/goose/.goosehints || exit 1
    test -f ~/.config/goose/.gooseignore || exit 1
    
    # Verify profile-specific config
    grep "$role" ~/.config/goose/.goosehints || exit 1
    
    echo "✅ $role profile setup works"
  done
  
  echo "✅ All 6 roles validated"
  ```

- [ ] Run tests
- [ ] Verify: 6/6 roles pass ✅

---

### G5: End-to-End Workflow Test (4 hours)

- [ ] Create `tests/integration/phase6-e2e.sh`:
  ```bash
  #!/bin/bash
  # Complete end-to-end workflow test
  
  echo "=== Phase 6 End-to-End Test ==="
  
  # 1. Start all services
  docker compose -f deploy/compose/ce.dev.yml up -d
  sleep 10
  
  # 2. Setup Finance profile
  echo -e "testuser@example.com\ntestpassword" | ./scripts/setup-profile.sh finance
  
  # 3. Verify config loaded
  grep "anthropic/claude-3.5-sonnet" ~/.config/goose/config.yaml || exit 1
  grep "localhost:8090" ~/.config/goose/config.yaml || exit 1
  
  # 4. Test Privacy Guard Proxy (simulated chat)
  RESPONSE=$(curl -X POST http://localhost:8090/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{
      "model": "anthropic/claude-3.5-sonnet",
      "messages": [{"role": "user", "content": "My SSN is 123-45-6789"}]
    }')
  
  # 5. Verify Privacy Guard was called
  docker logs ce_privacy_guard | grep "POST /guard/mask" || exit 1
  
  # 6. Admin UI: Publish profile
  ADMIN_JWT=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
    -d "client_id=goose-controller" \
    -d "grant_type=password" \
    -d "username=admin@example.com" \
    -d "password=admin" | jq -r '.access_token')
  
  curl -X POST http://localhost:8088/admin/profiles/finance/publish \
    -H "Authorization: Bearer $ADMIN_JWT"
  
  # 7. Verify signature created
  PROFILE=$(curl -H "Authorization: Bearer $ADMIN_JWT" \
    http://localhost:8088/profiles/finance)
  echo "$PROFILE" | jq '.signature.signature' | grep "vault:v1:" || exit 1
  
  echo "✅ End-to-End test PASSED!"
  ```

- [ ] Run test
- [ ] Verify: All steps pass ✅

---

### G6: Regression Tests (Phase 1-5) (2 hours)

- [ ] Run all Phase 1-5 integration tests:
  ```bash
  ./tests/integration/test_oidc_login.sh
  ./tests/integration/test_jwt_verification.sh
  ./tests/integration/test_privacy_guard_regex.sh
  ./tests/integration/test_controller_routes.sh
  ./tests/integration/test_profile_loading.sh
  # ... all 60 existing tests
  ```

- [ ] Verify: 60/60 tests pass (no regressions) ✅

---

### G7: Performance Validation (2 hours)

- [ ] Test profile loading latency:
  ```bash
  for i in {1..100}; do
    curl -w "%{time_total}\n" -o /dev/null -s \
      -H "Authorization: Bearer $JWT" \
      http://localhost:8088/profiles/finance
  done | awk '{sum+=$1; if($1>max) max=$1} END {print "P50:", sum/NR, "P99:", max}'
  ```
  - [ ] Target: P50 < 100ms ✅

- [ ] Test Privacy Guard Proxy overhead:
  ```bash
  for i in {1..100}; do
    curl -w "%{time_total}\n" -o /dev/null -s \
      -X POST http://localhost:8090/v1/chat/completions \
      -d '{"model":"test","messages":[{"role":"user","content":"Test with SSN 123-45-6789"}]}'
  done | awk '{sum+=$1; if($1>max) max=$1} END {print "P50:", sum/NR, "P99:", max}'
  ```
  - [ ] Target: P50 < 500ms ✅

- [ ] Document results in `docs/tests/phase6-performance.md`

**Deliverable:** Performance targets validated ✅

**Workstream G Complete** → Update progress log, mark checklist ✅

---

## 📋 WORKSTREAM H: Documentation (1 day)

### H1: Update Vault Guide (2 hours)

- [ ] Update `docs/guides/VAULT.md`:
  - [ ] Mark Section 5.1-5.5 as ✅ Complete
  - [ ] Add troubleshooting section:
    - Unseal procedure
    - AppRole token renewal
    - Audit log location
    - Certificate issues

**Deliverable:** Vault guide complete ✅

---

### H2: Create Privacy Guard Proxy Guide (2 hours)

- [ ] Create `docs/guides/PRIVACY-GUARD-PROXY.md`:
  ```markdown
  # Privacy Guard Proxy Guide
  
  ## Overview
  HTTP proxy that intercepts LLM requests, masks PII, forwards to cloud LLM.
  
  ## Architecture
  [Diagram showing Goose → Proxy → OpenRouter]
  
  ## Installation
  [Docker Compose instructions]
  
  ## Configuration
  [Goose config.yaml settings]
  
  ## Testing
  [How to verify it's working]
  
  ## Troubleshooting
  [Common issues]
  ```

**Deliverable:** Proxy guide complete ✅

---

### H3: Create Profile Setup Guide (2 hours)

- [ ] Create `docs/user/PROFILE-SETUP-GUIDE.md`:
  ```markdown
  # Profile Setup Guide
  
  ## Quick Start
  
  1. Run setup script:
     ```bash
     ./scripts/setup-profile.sh finance
     ```
  
  2. Enter credentials when prompted
  
  3. Launch Goose:
     ```bash
     goose session start
     ```
  
  ## What Happens
  [Diagram showing setup flow]
  
  ## All Roles
  [List of 6 roles with descriptions]
  
  ## Troubleshooting
  [Common issues]
  ```

**Deliverable:** Setup guide complete ✅

---

### H4: Create Admin UI Guide (2 hours)

- [ ] Create `docs/admin/ADMIN-UI-GUIDE.md`:
  - [ ] Overview (5 pages)
  - [ ] Dashboard usage
  - [ ] Profile editing (Monaco YAML)
  - [ ] Org chart CSV format
  - [ ] Publishing profiles (Vault signing)
  - [ ] Screenshots of each page

**Deliverable:** Admin guide complete ✅

---

### H5: Create Security Hardening Guide (1 hour)

- [ ] Create `docs/security/SECURITY-HARDENING.md`:
  - [ ] Secrets management (.env files)
  - [ ] Environment variable audit process
  - [ ] Docker Compose security options
  - [ ] Vault production checklist
  - [ ] PII protection verification

**Deliverable:** Security guide complete ✅

---

### H6: Create Migration Guide (1 hour)

- [ ] Create `docs/MIGRATION-PHASE6.md`:
  ```markdown
  # Migration Guide: v0.5.0 → v0.6.0
  
  ## Breaking Changes
  
  1. **Vault Configuration Required**
     - Must use AppRole (VAULT_ROLE_ID, VAULT_SECRET_ID)
     - Root token no longer supported
  
  2. **Privacy Guard Proxy**
     - New service required (port 8090)
     - Update Goose config to use proxy
  
  3. **Profile Setup**
     - Users must run setup-profile.sh before first use
     - Config files now fetched from Controller
  
  ## Migration Steps
  
  1. Update .env file (add VAULT_ROLE_ID, VAULT_SECRET_ID)
  2. Start new services: docker compose up privacy-guard-proxy
  3. Run setup-profile.sh for each user
  4. Verify: Privacy Guard Proxy working
  
  ## Rollback Plan
  
  [Instructions to rollback to v0.5.0 if needed]
  ```

**Deliverable:** Migration guide complete ✅

**Workstream H Complete** → Update progress log, mark checklist ✅

---

## 🎯 FINAL CHECKPOINT

### Pre-Release Validation

- [ ] All services running:
  ```bash
  docker compose -f deploy/compose/ce.dev.yml ps
  # Should show 9 services (added privacy-guard-proxy)
  ```

- [ ] All tests passing:
  - [ ] Validation: 6/6 ✅
  - [ ] Vault production: 5/5 ✅
  - [ ] Admin UI: 8/8 ✅
  - [ ] Privacy Proxy: 6/6 ✅
  - [ ] Profile setup: 6/6 ✅
  - [ ] End-to-end: 1/1 ✅
  - [ ] Regression: 60/60 ✅
  - **Total:** 92/92 tests ✅

- [ ] Documentation complete:
  - [ ] Vault guide ✅
  - [ ] Proxy guide ✅
  - [ ] Setup guide ✅
  - [ ] Admin guide ✅
  - [ ] Security guide ✅
  - [ ] Migration guide ✅

---

### Git Commit & Tag

- [ ] Commit all changes:
  ```bash
  git add .
  git commit -m "feat(phase-6): Production hardening + Admin UI + Privacy Proxy [v0.6.0]
  
  - Vault production-ready (TLS, AppRole, Raft, audit)
  - Admin UI deployed (5 pages, D3.js, Monaco editor)
  - Privacy Guard Proxy (masks PII before LLM)
  - Profile setup scripts (6 roles)
  - Lifecycle wired into routes
  - Security hardened (no secrets, .env.example)
  - 92/92 integration tests passing
  
  Breaking changes:
  - Requires Vault AppRole (not root token)
  - Requires setup-profile.sh for user onboarding
  - New service: privacy-guard-proxy (port 8090)
  
  Closes #XXX"
  ```

- [ ] Push to main:
  ```bash
  git push origin main
  ```

- [ ] Create release tag:
  ```bash
  git tag -a v0.6.0 -m "Release v0.6.0 - Production MVP
  
  Features:
  - Vault production (TLS, AppRole, Raft, audit)
  - Admin UI (profile management, org chart, audit logs)
  - Privacy Guard Proxy (PII protection for cloud LLMs)
  - Profile setup automation (6 roles)
  - Lifecycle state machine integrated
  - Security hardening complete
  
  Breaking Changes:
  - Vault AppRole required (see MIGRATION-PHASE6.md)
  - Profile setup script required for users
  
  Testing:
  - 92/92 integration tests passing
  - Performance validated (P50 < 500ms)
  
  Documentation:
  - 6 new guides (Vault, Proxy, Setup, Admin, Security, Migration)
  "
  
  git push origin v0.6.0
  ```

---

### Update Phase Tracking

- [ ] Create `Technical Project Plan/PM Phases/Phase-6/Phase-6-Completion-Summary.md`:
  ```markdown
  # Phase 6 Completion Summary
  
  **Version:** v0.6.0  
  **Status:** ✅ COMPLETE  
  **Date:** 2025-11-XX  
  **Timeline:** 14 days actual (3 weeks calendar)
  
  ## Deliverables Completed
  
  - ✅ Vault Production (TLS, AppRole, Raft, audit, signature verification)
  - ✅ Admin UI (5 pages: Dashboard, Profiles, Org, Audit, Settings)
  - ✅ Privacy Guard Proxy (Rust service, port 8090)
  - ✅ Profile Setup Scripts (6 roles: finance, legal, developer, hr, executive, support)
  - ✅ Lifecycle Integration (wired into session routes)
  - ✅ Security Hardening (no secrets, .env.example, SECURITY.md)
  - ✅ 92/92 Integration Tests Passing
  - ✅ 6 Documentation Guides
  
  ## Architecture Changes
  
  - Added: privacy-guard-proxy service (port 8090)
  - Modified: Vault (dev → production mode)
  - Added: scripts/setup-profile.sh
  - Modified: session routes (use lifecycle module)
  
  ## Testing Results
  
  - Vault production: 5/5 ✅
  - Admin UI: 8/8 ✅
  - Privacy Proxy: 6/6 ✅
  - Profile setup: 6/6 ✅
  - End-to-end: 1/1 ✅
  - Regression: 60/60 ✅
  - Performance: P50 < 500ms ✅
  
  ## Next Phase
  
  Phase 7: NER Quality Improvement + Recipe System
  ```

- [ ] Update `Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`:
  ```json
  {
    "phase": "6",
    "status": "COMPLETE",
    "current_workstream": "NONE",
    "completed_workstreams": ["A", "B", "C", "D", "E", "F", "G", "H"],
    "tests_passing": "92/92",
    "version": "v0.6.0",
    "completion_date": "2025-11-XX"
  }
  ```

- [ ] Update `docs/tests/phase6-progress.md` (final entry)

---

## ✅ PHASE 6 COMPLETE!

**When all checkboxes above are complete:**

**You will have:**
- ✅ Production-ready MVP (v0.6.0)
- ✅ Users can sign in → Load profiles → Chat with PII protection
- ✅ Admin can manage profiles, org chart, audit logs
- ✅ Vault hardened (TLS, AppRole, Raft, audit)
- ✅ 92/92 tests passing
- ✅ 6 documentation guides
- ✅ Ready for Phase 7 (NER quality + Recipe system)

**Timeline:** 14 days (3 weeks calendar time)

**Next:** Plan Phase 7! 🚀
