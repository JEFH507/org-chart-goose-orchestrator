# Vault Operations Guide

**Version**: 1.0.0  
**Last Updated**: 2025-11-07  
**Status**: Dev Mode (Phase 5) → Production Upgrade (Phase 6)

---

## Table of Contents

1. [Overview](#overview)
2. [Current Dev Mode Setup](#current-dev-mode-setup)
3. [Admin Workflow](#admin-workflow)
4. [Testing Vault Integration](#testing-vault-integration)
5. [Phase 6 Production Upgrade](#phase-6-production-upgrade)
6. [Security Considerations](#security-considerations)
7. [Troubleshooting](#troubleshooting)

---

## 1. Overview

### Purpose

HashiCorp Vault provides **cryptographic services** for the org-chart-goose-orchestrator, specifically:

- **Profile Integrity**: HMAC-SHA256 signatures prevent tampering with role profiles
- **Future Use Cases**: PII encryption, PKI, dynamic database credentials (Phase 7+)

### Architecture

```
Admin publishes profile (POST /admin/profiles/{role}/publish)
  │
  ▼
Controller generates profile hash
  │
  ▼
Vault Transit Engine (HMAC-SHA256)
  │  Key: profile-signing
  │  Algorithm: sha2-256
  │
  ▼
Signature: vault:v1:BASE64_HMAC
  │
  ▼
Stored in profile.signature field (Postgres JSONB)
  │
  ▼
User loads profile (GET /profiles/{role})
  │
  ▼
(Phase 6) Controller verifies signature against current profile data
```

### Current State (Phase 5)

**Status**: ✅ **Dev Mode Integrated** (2025-11-07)

**What Works**:
- ✅ Vault running in Docker (hashicorp/vault:1.18.3)
- ✅ Transit engine auto-enabled (vault-init.sh)
- ✅ Controller environment variables configured
- ✅ D9 endpoint signing profiles (HMAC-SHA256)
- ✅ Signatures stored in database

**What's Missing (Phase 6)**:
- ⏳ TLS/HTTPS encryption
- ⏳ AppRole authentication (replace root token)
- ⏳ Persistent storage (Raft or Consul)
- ⏳ Audit device (compliance logging)
- ⏳ Signature verification on profile load

---

## 2. Current Dev Mode Setup

### 2.1 Docker Compose Configuration

**File**: `deploy/compose/ce.dev.yml`

```yaml
services:
  vault:
    image: hashicorp/vault:1.18.3
    container_name: ce_vault
    ports:
      - "8200:8200"
    environment:
      VAULT_DEV_ROOT_TOKEN_ID: "root"          # ← INSECURE (dev only)
      VAULT_DEV_LISTEN_ADDRESS: "0.0.0.0:8200"
    command: server -dev -dev-listen-address=0.0.0.0:8200
    healthcheck:
      test: ["CMD", "vault", "status"]
      interval: 5s
      timeout: 3s
      retries: 3
      start_period: 10s
    cap_add:
      - IPC_LOCK

  vault-init:
    image: curlimages/curl:8.11.1
    container_name: ce_vault_init
    environment:
      VAULT_ADDR: http://vault:8200
      VAULT_TOKEN: root
    volumes:
      - ./vault-init.sh:/vault-init.sh:ro
    command: ["/bin/sh", "/vault-init.sh"]
    depends_on:
      vault:
        condition: service_healthy
    restart: "no"  # Run once

  controller:
    # ... other config ...
    environment:
      VAULT_ADDR: ${VAULT_ADDR:-http://vault:8200}
      VAULT_TOKEN: ${VAULT_TOKEN:-root}
    depends_on:
      vault:
        condition: service_healthy
```

### 2.2 Transit Engine Initialization

**File**: `deploy/compose/vault-init.sh`

```bash
#!/bin/sh
# Vault initialization script for dev mode

set -e

VAULT_ADDR="${VAULT_ADDR:-http://localhost:8200}"
VAULT_TOKEN="${VAULT_TOKEN:-root}"

echo "Waiting for Vault to be ready..."
until curl -sf "$VAULT_ADDR/v1/sys/health" > /dev/null 2>&1; do
    sleep 1
done

echo "Vault is ready. Initializing..."

# Enable Transit engine (idempotent)
echo "Enabling Transit engine..."
curl -sf -X POST \
    -H "X-Vault-Token: $VAULT_TOKEN" \
    "$VAULT_ADDR/v1/sys/mounts/transit" \
    -d '{"type":"transit"}' \
    2>/dev/null || echo "Transit engine already enabled"

echo "Vault initialization complete."
```

**Purpose**:
- Waits for Vault to be healthy
- Enables Transit engine on first startup
- Idempotent (safe to run multiple times)

### 2.3 Environment Variables

**Controller Service**:
```bash
VAULT_ADDR=http://vault:8200      # Vault API endpoint
VAULT_TOKEN=root                   # Root token (INSECURE - dev only)
```

**Verification**:
```bash
docker exec ce_controller sh -c 'echo "VAULT_ADDR=$VAULT_ADDR VAULT_TOKEN=$VAULT_TOKEN"'
# Output: VAULT_ADDR=http://vault:8200 VAULT_TOKEN=root
```

### 2.4 Vault Configuration

**Dev Mode Settings**:
- **Storage**: In-memory (ephemeral, lost on restart)
- **TLS**: Disabled (HTTP only)
- **Authentication**: Root token (`root`)
- **Audit**: Disabled
- **Unsealing**: Auto-unsealed (dev mode)

**Transit Engine**:
- **Key**: `profile-signing` (auto-created on first sign)
- **Algorithm**: HMAC-SHA256
- **Type**: Encryption key (can derive HMAC keys)

---

## 3. Admin Workflow

### 3.1 Create Profile (D7)

**Endpoint**: `POST /admin/profiles`

```bash
# Get admin JWT token
ADMIN_TOKEN=$(curl -s -X POST "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=phase5test" \
  -d "password=test123" \
  -d "grant_type=password" \
  -d "client_id=goose-controller" \
  | jq -r '.access_token')

# Create profile
curl -X POST "http://localhost:8088/admin/profiles" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d @profiles/finance.json
```

**Response** (201 Created):
```json
{
  "role": "finance",
  "created_at": "2025-11-07T04:29:14.087327749+00:00"
}
```

**Database State**:
- Profile stored in Postgres (`profiles` table)
- `signature` field is `null` (not yet signed)

---

### 3.2 Update Profile (D8)

**Endpoint**: `PUT /admin/profiles/{role}`

```bash
# Partial update (JSON merge)
curl -X PUT "http://localhost:8088/admin/profiles/finance" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Finance Team Agent (Updated)",
    "description": "Updated description for Vault testing"
  }'
```

**Response** (200 OK):
```json
{
  "role": "finance",
  "updated_at": "2025-11-07T04:29:20.123456789+00:00"
}
```

**Behavior**:
- Partial JSON merge (via `json-patch` library)
- Validation enforced on merged result
- `signature` field remains `null` (not re-signed automatically)

---

### 3.3 Publish Profile (D9) - Vault Signing

**Endpoint**: `POST /admin/profiles/{role}/publish`

```bash
# Sign profile with Vault
curl -X POST "http://localhost:8088/admin/profiles/finance/publish" \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

**Response** (200 OK):
```json
{
  "role": "finance",
  "signature": "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc=",
  "signed_at": "2025-11-07T04:29:31.058861974+00:00"
}
```

**Vault Operations**:

1. **Controller fetches profile** from Postgres
2. **Serialize profile to canonical JSON** (deterministic)
3. **Call Vault Transit API**:
   ```bash
   curl -X POST "http://vault:8200/v1/transit/hmac/profile-signing/sha2-256" \
     -H "X-Vault-Token: root" \
     -d '{"input": "BASE64_PROFILE_JSON"}'
   ```
4. **Vault returns HMAC signature**:
   ```json
   {
     "data": {
       "hmac": "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="
     }
   }
   ```
5. **Controller updates profile**:
   ```rust
   profile.signature = Some(Signature {
       algorithm: "sha2-256",
       vault_key: "transit/keys/profile-signing",
       signed_at: Some(Utc::now().to_rfc3339()),
       signed_by: Some("admin@example.com"),
       signature: Some("vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="),
   });
   ```
6. **Store updated profile** in Postgres

**Database State**:
- `signature` field now populated with Vault HMAC
- `updated_at` timestamp reflects signing time

---

### 3.4 Re-Publishing After Updates

**Scenario**: Admin updates profile description, needs to re-sign

```bash
# Update profile
curl -X PUT "http://localhost:8088/admin/profiles/finance" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{"description": "New description"}'

# Re-publish (generates new signature)
curl -X POST "http://localhost:8088/admin/profiles/finance/publish" \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

**Behavior**:
- New signature generated (HMAC determinism + content changes)
- Old signature replaced
- `signed_at` timestamp updated

**Example**:
```
Old signature: vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc=
New signature: vault:v1:8xnfR1Up92Ha1F0CmJoiwXwqK4sRpeFoYizlexdC9ld=
```

---

## 4. Testing Vault Integration

### 4.1 Health Check

```bash
# Vault API health
curl http://localhost:8200/v1/sys/health
```

**Expected Response** (200 OK):
```json
{
  "initialized": true,
  "sealed": false,
  "standby": false,
  "performance_standby": false,
  "replication_performance_mode": "disabled",
  "replication_dr_mode": "disabled",
  "server_time_utc": 1699315200,
  "version": "1.18.3",
  "cluster_name": "vault-cluster-abc123",
  "cluster_id": "def456-ghi789"
}
```

### 4.2 Transit Engine Status

```bash
# Check Transit engine
curl -H "X-Vault-Token: root" \
  http://localhost:8200/v1/sys/mounts/transit
```

**Expected Response** (200 OK):
```json
{
  "type": "transit",
  "description": "",
  "config": {
    "default_lease_ttl": 0,
    "max_lease_ttl": 0,
    "force_no_cache": false
  }
}
```

### 4.3 Profile Signing Key

```bash
# Check if profile-signing key exists
curl -H "X-Vault-Token: root" \
  http://localhost:8200/v1/transit/keys/profile-signing
```

**Response** (after first sign):
```json
{
  "data": {
    "name": "profile-signing",
    "type": "aes256-gcm96",
    "deletion_allowed": false,
    "derived": false,
    "exportable": false,
    "allow_plaintext_backup": false,
    "keys": {
      "1": 1699315200
    },
    "min_decryption_version": 1,
    "min_encryption_version": 0,
    "supports_encryption": true,
    "supports_decryption": true,
    "supports_derivation": true,
    "supports_signing": false
  }
}
```

**Note**: Key is auto-created on first HMAC operation.

### 4.4 Manual HMAC Test

```bash
# Test HMAC signing manually
echo -n '{"role":"finance","display_name":"Finance Agent"}' | base64 | \
  curl -X POST "http://localhost:8200/v1/transit/hmac/profile-signing/sha2-256" \
    -H "X-Vault-Token: root" \
    -d @- \
    --data-urlencode 'input@-'
```

**Expected Response**:
```json
{
  "data": {
    "hmac": "vault:v1:SOME_BASE64_HMAC_SIGNATURE"
  }
}
```

### 4.5 Integration Test Script

**Run**: `tests/integration/test_admin_profiles.sh`

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./tests/integration/test_admin_profiles.sh
```

**Expected Results**:
- ✓ D7 PASS: Profile created (201)
- ✓ D8 PASS: Profile updated (200)
- ✓ D9 PASS: Profile signed with Vault (200, signature format: `vault:v1:*`)
- ✓ Re-publish test: Signature changed after update

---

## 5. Phase 6 Production Upgrade

### 5.1 TLS/HTTPS Setup (Required)

**Goal**: Encrypt Vault API traffic

**Steps**:

1. **Generate TLS certificates**:
   ```bash
   # Using Let's Encrypt or self-signed cert
   openssl req -x509 -newkey rsa:4096 \
     -keyout vault-key.pem -out vault-cert.pem \
     -days 365 -nodes \
     -subj "/CN=vault.example.com"
   ```

2. **Update Vault config** (`deploy/vault/config.hcl`):
   ```hcl
   listener "tcp" {
     address     = "0.0.0.0:8200"
     tls_cert_file = "/vault/certs/vault-cert.pem"
     tls_key_file  = "/vault/certs/vault-key.pem"
   }
   ```

3. **Update docker-compose** (`deploy/compose/ce.prod.yml`):
   ```yaml
   vault:
     volumes:
       - ./vault/certs:/vault/certs:ro
       - ./vault/config.hcl:/vault/config/vault.hcl:ro
     command: server -config=/vault/config/vault.hcl
   ```

4. **Update controller env vars**:
   ```bash
   VAULT_ADDR=https://vault:8200  # HTTPS!
   VAULT_CACERT=/path/to/ca.pem   # CA certificate
   ```

5. **Test**:
   ```bash
   curl --cacert ca.pem https://vault:8200/v1/sys/health
   ```

**Estimated Time**: 2 hours

---

### 5.2 AppRole Authentication (Required)

**Goal**: Replace root token with AppRole (role_id + secret_id)

**Steps**:

1. **Enable AppRole auth**:
   ```bash
   vault auth enable approle
   ```

2. **Create role for controller**:
   ```bash
   vault write auth/approle/role/controller-role \
     secret_id_ttl=120m \
     token_ttl=60m \
     token_max_ttl=120m \
     policies="controller-policy"
   ```

3. **Create policy** (`deploy/vault/controller-policy.hcl`):
   ```hcl
   # Transit engine permissions
   path "transit/hmac/profile-signing/*" {
     capabilities = ["create", "read", "update"]
   }
   
   path "transit/verify/profile-signing/*" {
     capabilities = ["create", "read", "update"]
   }
   
   # Key management (read-only)
   path "transit/keys/profile-signing" {
     capabilities = ["read"]
   }
   ```

4. **Generate role_id and secret_id**:
   ```bash
   # Get role_id (static)
   ROLE_ID=$(vault read -field=role_id auth/approle/role/controller-role/role-id)
   
   # Generate secret_id (rotatable)
   SECRET_ID=$(vault write -field=secret_id -f auth/approle/role/controller-role/secret-id)
   ```

5. **Update controller code** (`src/vault/client.rs`):
   ```rust
   pub fn from_approle(vault_addr: &str, role_id: &str, secret_id: &str) -> Result<Self, String> {
       // Login to get client token
       let login_response = reqwest::blocking::Client::new()
           .post(format!("{}/v1/auth/approle/login", vault_addr))
           .json(&serde_json::json!({
               "role_id": role_id,
               "secret_id": secret_id
           }))
           .send()
           .map_err(|e| format!("AppRole login failed: {}", e))?;
       
       let token = login_response.json::<AppRoleLoginResponse>()
           .map_err(|e| format!("Parse login response failed: {}", e))?
           .auth.client_token;
       
       // Create Vault client with token
       let client = VaultClient::new(vault_addr, &token)?;
       
       // TODO: Implement token renewal (token_ttl = 60m)
       
       Ok(client)
   }
   ```

6. **Update env vars**:
   ```bash
   # Replace VAULT_TOKEN with VAULT_ROLE_ID and VAULT_SECRET_ID
   VAULT_ADDR=https://vault:8200
   VAULT_ROLE_ID=abc123...
   VAULT_SECRET_ID=def456...
   ```

7. **Implement token renewal** (background task):
   ```rust
   // Renew token every 45 minutes (before 60m expiry)
   tokio::spawn(async move {
       loop {
           tokio::time::sleep(Duration::from_secs(45 * 60)).await;
           vault_client.renew_token().await;
       }
   });
   ```

**Estimated Time**: 3 hours

---

### 5.3 Persistent Storage (Required)

**Goal**: Survive Vault restarts (Raft or Consul backend)

**Option A: Raft Storage (Recommended)**

**Steps**:

1. **Create Vault config** (`deploy/vault/config.hcl`):
   ```hcl
   storage "raft" {
     path = "/vault/data"
     node_id = "vault-1"
   }
   
   listener "tcp" {
     address     = "0.0.0.0:8200"
     tls_cert_file = "/vault/certs/vault-cert.pem"
     tls_key_file  = "/vault/certs/vault-key.pem"
   }
   
   api_addr = "https://vault:8200"
   cluster_addr = "https://vault:8201"
   ui = true
   ```

2. **Update docker-compose**:
   ```yaml
   vault:
     volumes:
       - vault_data:/vault/data  # Persistent volume
       - ./vault/config.hcl:/vault/config/vault.hcl:ro
     command: server -config=/vault/config/vault.hcl
   
   volumes:
     vault_data:
       driver: local
   ```

3. **Initialize Vault** (first startup):
   ```bash
   # Initialize (generates unseal keys + root token)
   vault operator init -key-shares=5 -key-threshold=3
   
   # Save output securely (unseal keys + root token)
   # Example output:
   # Unseal Key 1: abc123...
   # Unseal Key 2: def456...
   # Unseal Key 3: ghi789...
   # Unseal Key 4: jkl012...
   # Unseal Key 5: mno345...
   # Initial Root Token: pqr678...
   ```

4. **Unseal Vault** (after restart):
   ```bash
   # Provide 3 of 5 unseal keys
   vault operator unseal <key1>
   vault operator unseal <key2>
   vault operator unseal <key3>
   ```

5. **Automate unseal** (optional, requires Vault Enterprise or AWS KMS):
   ```hcl
   seal "awskms" {
     region     = "us-east-1"
     kms_key_id = "arn:aws:kms:us-east-1:123456789012:key/..."
   }
   ```

**Option B: Consul Storage**

**Steps**:

1. **Deploy Consul cluster** (3+ nodes)
2. **Update Vault config**:
   ```hcl
   storage "consul" {
     address = "consul:8500"
     path    = "vault/"
   }
   ```

**Estimated Time**: 2 hours (Raft) or 4 hours (Consul)

---

### 5.4 Audit Device (Compliance)

**Goal**: Log all Vault operations for compliance

**Steps**:

1. **Enable file audit device**:
   ```bash
   vault audit enable file file_path=/vault/logs/audit.log
   ```

2. **Update docker-compose**:
   ```yaml
   vault:
     volumes:
       - vault_logs:/vault/logs  # Persistent audit logs
   ```

3. **Configure log rotation** (logrotate):
   ```
   /vault/logs/audit.log {
     daily
     rotate 90
     compress
     delaycompress
     missingok
     notifempty
   }
   ```

4. **Verify audit logs**:
   ```bash
   tail -f /vault/logs/audit.log
   # Example log entry:
   # {"time":"2025-11-07T04:29:31Z","type":"request","auth":{"client_token":"hmac-sha256:..."},"request":{"operation":"create","path":"transit/hmac/profile-signing/sha2-256"}}
   ```

**Estimated Time**: 1 hour

---

### 5.5 Signature Verification (Controller Code)

**Goal**: Verify signature matches profile data on load

**Steps**:

1. **Add verification to profile loading** (`src/controller/routes/profiles.rs`):
   ```rust
   pub async fn get_profile(
       State(state): State<AppState>,
       Path(role): Path<String>,
   ) -> Result<Json<Profile>, StatusCode> {
       // Fetch profile from database
       let profile = db::profiles::get_profile(&state.pool, &role)
           .await
           .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
           .ok_or(StatusCode::NOT_FOUND)?;
       
       // Verify signature (if present)
       if let Some(signature) = &profile.signature {
           let vault_client = VaultClient::from_env()
               .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
           
           // Serialize profile to canonical JSON (exclude signature field)
           let mut profile_copy = profile.clone();
           profile_copy.signature = None;
           let profile_json = serde_json::to_string(&profile_copy)
               .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
           
           // Verify HMAC
           let verified = vault_client
               .verify_hmac("profile-signing", &profile_json, &signature.signature.unwrap())
               .await
               .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
           
           if !verified {
               warn!("Profile signature verification failed for role: {}", role);
               return Err(StatusCode::FORBIDDEN);  // Tampered profile!
           }
       }
       
       Ok(Json(profile))
   }
   ```

2. **Implement Vault verify endpoint** (`src/vault/transit.rs`):
   ```rust
   pub async fn verify_hmac(
       &self,
       key_name: &str,
       data: &str,
       signature: &str,
   ) -> Result<bool, String> {
       let input_b64 = base64::encode(data);
       
       let response: serde_json::Value = self.client
           .post(format!("{}/v1/transit/verify/{}/sha2-256", self.address, key_name))
           .header("X-Vault-Token", &self.token)
           .json(&serde_json::json!({
               "input": input_b64,
               "hmac": signature
           }))
           .send()
           .await
           .map_err(|e| format!("Vault verify failed: {}", e))?
           .json()
           .await
           .map_err(|e| format!("Parse verify response failed: {}", e))?;
       
       Ok(response["data"]["valid"].as_bool().unwrap_or(false))
   }
   ```

**Estimated Time**: 2 hours

---

### 5.6 Production Deployment Checklist

**Before Go-Live**:

- [ ] TLS certificates generated and installed
- [ ] VAULT_ADDR updated to HTTPS
- [ ] AppRole configured with controller-policy
- [ ] VAULT_TOKEN removed from env vars
- [ ] VAULT_ROLE_ID and VAULT_SECRET_ID configured
- [ ] Token renewal implemented
- [ ] Raft storage configured with persistent volume
- [ ] Vault initialized and unsealed
- [ ] Unseal keys stored securely (offline)
- [ ] Audit device enabled and logs rotating
- [ ] Signature verification implemented in controller
- [ ] Integration tests passing with production config
- [ ] Disaster recovery procedures documented

**Total Estimated Time**: 4-6 hours (excluding testing)

---

## 6. Security Considerations

### 6.1 Dev Mode Risks

**⚠️ CRITICAL: Do NOT use dev mode in production!**

**Risks**:
1. **In-memory storage**: All data lost on restart
2. **No TLS**: API traffic unencrypted (man-in-the-middle attacks)
3. **Root token**: Hardcoded `root` token (full admin access)
4. **No audit logs**: No compliance trail
5. **Auto-unsealed**: No unseal key protection

**Mitigation**: Phase 6 production upgrade addresses all risks.

---

### 6.2 Key Management

**Transit Key: profile-signing**

**Protection**:
- ✅ Key never leaves Vault
- ✅ Key material not exportable
- ✅ HMAC operations audited
- ⏳ (Phase 6) Key rotation policy

**Key Rotation** (Phase 6):
```bash
# Rotate key (creates new version)
vault write -f transit/keys/profile-signing/rotate

# Re-sign all profiles with new key version
for role in $(psql -U controller -d controller -t -c "SELECT role FROM profiles"); do
  curl -X POST "http://localhost:8088/admin/profiles/$role/publish" \
    -H "Authorization: Bearer $ADMIN_TOKEN"
done
```

---

### 6.3 Access Control

**Current** (Phase 5):
- Root token has **full access** to all Vault operations

**Phase 6** (AppRole):
- Controller has **limited access** via `controller-policy`:
  - Transit HMAC operations (sign, verify)
  - Transit key read (metadata only)
  - **NO** key deletion, export, or admin operations

**Future** (Phase 7+):
- Separate AppRoles for different services (Privacy Guard, Admin UI, etc.)
- Least-privilege policies

---

### 6.4 Compliance

**SOC 2 / ISO 27001 Requirements**:

1. **Encryption in Transit**: TLS/HTTPS (Phase 6)
2. **Audit Logging**: File audit device (Phase 6)
3. **Access Control**: AppRole with policies (Phase 6)
4. **Key Rotation**: Annual rotation policy (Phase 7)
5. **Disaster Recovery**: Unseal key backup + Raft snapshots (Phase 6)

**Audit Trail Example**:
```json
{
  "time": "2025-11-07T04:29:31Z",
  "type": "request",
  "auth": {
    "client_token": "hmac-sha256:abc123",
    "accessor": "hmac-sha256:def456",
    "display_name": "approle:controller-role",
    "policies": ["controller-policy", "default"]
  },
  "request": {
    "id": "ghi789",
    "operation": "create",
    "path": "transit/hmac/profile-signing/sha2-256",
    "data": {
      "input": "BASE64_PROFILE_JSON"
    }
  },
  "response": {
    "data": {
      "hmac": "vault:v1:SIGNATURE"
    }
  }
}
```

---

## 7. Troubleshooting

### Problem 1: Vault Not Starting

**Symptom**: `docker ps` shows vault container exited

**Debug**:
```bash
docker logs ce_vault
```

**Common Causes**:
1. Port 8200 already in use
2. Invalid config file (syntax error)
3. Insufficient memory (IPC_LOCK capability)

**Fix**:
```bash
# Check port
lsof -i :8200

# Validate config
vault server -config=/vault/config/vault.hcl -test

# Check capability
docker inspect ce_vault | grep IPC_LOCK
```

---

### Problem 2: Transit Engine Not Enabled

**Symptom**: D9 publish returns 404 error

**Debug**:
```bash
curl -H "X-Vault-Token: root" http://localhost:8200/v1/sys/mounts/transit
```

**Fix**:
```bash
# Enable manually
curl -X POST -H "X-Vault-Token: root" \
  http://localhost:8200/v1/sys/mounts/transit \
  -d '{"type":"transit"}'

# Or restart vault-init service
docker restart ce_vault_init
```

---

### Problem 3: Signature Verification Fails (Phase 6)

**Symptom**: Profile loads return 403 Forbidden

**Cause**: Profile modified after signing (tampering or manual database edit)

**Debug**:
```bash
# Check signature in database
psql -U controller -d controller -c "SELECT role, config->'signature' FROM profiles WHERE role = 'finance';"

# Verify manually
curl -X POST "http://localhost:8200/v1/transit/verify/profile-signing/sha2-256" \
  -H "X-Vault-Token: root" \
  -d '{"input": "BASE64_PROFILE_JSON", "hmac": "vault:v1:SIGNATURE"}'
```

**Fix**: Re-publish profile
```bash
curl -X POST "http://localhost:8088/admin/profiles/finance/publish" \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

---

### Problem 4: Vault Sealed After Restart

**Symptom**: Vault API returns 503 Service Unavailable

**Debug**:
```bash
vault status
# Output: Sealed: true
```

**Fix**: Unseal with 3 of 5 keys
```bash
vault operator unseal <key1>
vault operator unseal <key2>
vault operator unseal <key3>
```

---

### Problem 5: AppRole Token Expired (Phase 6)

**Symptom**: Controller logs show "permission denied" errors

**Debug**:
```bash
# Check token TTL
vault token lookup $VAULT_TOKEN
```

**Fix**: Implement token renewal (see Phase 6 AppRole section)

---

## Appendix A: Vault CLI Reference

```bash
# Health check
vault status

# Enable Transit engine
vault secrets enable transit

# Create HMAC key
vault write -f transit/keys/profile-signing

# Sign data
echo -n '{"role":"finance"}' | base64 | \
  vault write transit/hmac/profile-signing/sha2-256 input=-

# Verify signature
vault write transit/verify/profile-signing/sha2-256 \
  input=BASE64_DATA \
  hmac=vault:v1:SIGNATURE

# Rotate key
vault write -f transit/keys/profile-signing/rotate

# List keys
vault list transit/keys

# Read key metadata
vault read transit/keys/profile-signing
```

---

## Appendix B: Related Documentation

- **Profile Specification**: `docs/profiles/SPEC.md`
- **VERSION_PINS**: `VERSION_PINS.md` (Vault 1.18.3)
- **ADR**: `docs/adr/0016-ce-profile-signing-key-management.md`
- **Integration Tests**: `tests/integration/test_admin_profiles.sh`
- **Progress Log**: `docs/tests/phase5-progress.md` (Vault Integration Enabled)

---

## Appendix C: Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-07 | Initial guide (Phase 5 dev mode + Phase 6 production plan) |

---

**End of Vault Operations Guide**
