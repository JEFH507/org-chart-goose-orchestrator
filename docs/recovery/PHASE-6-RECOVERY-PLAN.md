# Phase 6 Recovery Plan - Fresh Start from A1

**Created:** 2025-11-07 19:00 UTC  
**Reason:** Previous agent made security mistakes + storage confusion  
**Goal:** Clean restart from A1 with production-ready practices

---

## 🔴 Critical Issues Found

### 1. **Security Violations (ALREADY ADDRESSED)**
- ✅ Keys/secrets exposed in chat logs (remediated per progress log)
- ✅ Root token + unseal keys regenerated (2025-11-07 17:45 UTC)
- ✅ AppRole credentials regenerated after exposure
- ⚠️ **REMAINING:** User reports "multiple keys and users of Vault" (needs cleanup)

### 2. **Storage Configuration (CONFUSION)**
- Current vault.hcl shows: `storage "file"`
- User wants: **Raft storage** (production-ready)
- Previous agent intended to switch to Raft but didn't update config

### 3. **Environment Files (DUPLICATION)**
- Found 3 files:
  - `deploy/compose/.env.ce` ← User's working file (gitignored)
  - `deploy/compose/.env.ce.example` ← Example template (committed)
  - `deploy/compose/.env` ← Unknown duplicate
- User reports having TWO .env.ce files (need consolidation)

### 4. **Current State**
- State JSON: A3 next (Persistent Storage Raft)
- Vault config: Still file backend (not Raft)
- Containers: vault + controller running (healthy)
- Progress: A1 ✅, A2 ✅ (but potentially compromised)

---

## 🎯 Recovery Strategy: FULL RESET (Recommended)

**Why Full Reset:**
1. Clean slate eliminates ALL security debt
2. Ensures production-ready Raft from start
3. Only 6.5 hours work lost (A1+A2)
4. User has credentials in password manager (easy to re-enter)
5. No legacy confusion about "multiple Vault keys/users"

**Alternative (Not Recommended):**
- Partial reset (A3 only) - Faster but keeps security/credential confusion

---

## 📋 Recovery Steps

### Phase 1: Cleanup (30 minutes)

```bash
# Stop and remove Vault
cd deploy/compose
docker compose down vault
docker volume rm ce_vault_data 2>/dev/null || true

# Clean Vault artifacts
rm -rf deploy/vault/raft/* 2>/dev/null

# Check .env files (USER WILL IDENTIFY WHICH TO KEEP)
ls -la deploy/compose/.env*
```

**ASK USER:**
1. Which .env file has current working credentials?
2. Can we delete the duplicates?

---

### Phase 2: Fresh A1 - Vault with Raft (2 hours)

#### Step 1: Regenerate TLS Certificates
```bash
cd deploy/vault/certs
rm vault.crt vault-key.pem

openssl req -newkey rsa:2048 -nodes \
  -keyout vault-key.pem -x509 -days 365 \
  -out vault.crt \
  -subj "/CN=vault/O=OrgChart/C=US"

sudo chown 100:100 vault.crt vault-key.pem
sudo chmod 644 vault.crt
sudo chmod 400 vault-key.pem
```

#### Step 2: Update vault.hcl for Raft
```hcl
# File: deploy/vault/config/vault.hcl

ui = true

listener "tcp" {
  address       = "0.0.0.0:8200"
  tls_cert_file = "/vault/certs/vault.crt"
  tls_key_file  = "/vault/certs/vault-key.pem"
}

listener "tcp" {
  address     = "0.0.0.0:8201"
  tls_disable = true
}

# *** PRODUCTION STORAGE: Raft ***
storage "raft" {
  path    = "/vault/raft"
  node_id = "vault-ce-node1"
}

api_addr = "http://vault:8201"
cluster_addr = "https://vault:8202"
disable_mlock = true
```

#### Step 3: Update docker-compose
```yaml
# Change volume from vault_data to vault_raft
volumes:
  - vault_raft:/vault/raft
```

#### Step 4: Initialize Vault (USER MUST SAVE OUTPUT)
```bash
docker compose up -d vault
sleep 5

# Initialize (USER MUST SAVE ALL 5 KEYS + ROOT TOKEN)
docker exec -it ce_vault vault operator init

# Unseal (USER PASTES 3 KEYS)
docker exec -it ce_vault vault operator unseal  # key 1
docker exec -it ce_vault vault operator unseal  # key 2
docker exec -it ce_vault vault operator unseal  # key 3

# Enable Transit
export VAULT_TOKEN="<from-password-manager>"
docker exec -e VAULT_TOKEN=$VAULT_TOKEN ce_vault \
  vault secrets enable transit
docker exec -e VAULT_TOKEN=$VAULT_TOKEN ce_vault \
  vault write -f transit/keys/profile-signing
```

---

### Phase 3: Fresh A2 - AppRole (3 hours)

```bash
# Run AppRole setup
cd scripts
./vault-setup-approle.sh
# USER SAVES: ROLE_ID + SECRET_ID in password manager

# USER UPDATES: deploy/compose/.env.ce
# (We don't see this file, user edits directly)

# Restart controller
docker compose restart controller

# Test
docker compose logs controller | grep "AppRole"
# Should see: "AppRole authentication successful"
```

---

### Phase 4: Update Tracking (15 minutes)

Update 3 files:
1. **State JSON** - Mark A1-A3 complete, A4 next
2. **Progress log** - Document recovery
3. **Checklist** - Mark A1-A3 checked

Then commit:
```bash
git add .
git commit -m "feat(phase-6): Recovery - Fresh start A1-A3 with Raft

- Cleaned Vault data (removed file backend)
- Regenerated TLS certificates
- Switched to Raft storage (production-ready)
- Reinitialized Vault (user saved new credentials)
- Regenerated AppRole credentials
- Consolidated .env files

Security: All credentials in password manager, none in git/chat
Next: A4 (Audit Device)"

git push origin main
```

---

## 🚨 Security Rules (NEVER VIOLATE)

### Never Show in Chat:
- ❌ Vault unseal keys (any of 5)
- ❌ Vault root token
- ❌ AppRole SECRET_ID
- ❌ Contents of .env.ce file

### Safe to Show:
- ✅ AppRole ROLE_ID (public identifier)
- ✅ Configuration files
- ✅ Scripts
- ✅ Public certificates

### When Secrets Needed:
- ASK user to paste from password manager
- VERIFY command succeeded without seeing value
- NEVER read .env.ce (it's gitignored for security)

---

## ✅ Success Criteria

Recovery complete when:
1. ✅ Vault running with Raft storage
2. ✅ TLS enabled (dual listeners)
3. ✅ AppRole working (controller can sign profiles)
4. ✅ Only ONE .env.ce file
5. ✅ Only ONE set of Vault credentials (in password manager)
6. ✅ No secrets in git/chat/logs
7. ✅ State JSON: A1-A3 complete, A4 next
8. ✅ Progress log: Recovery documented
9. ✅ Checklist: A1-A3 marked ✅

---

## 📊 Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| Cleanup | 30 min | Stop Vault, consolidate .env |
| A1 Fresh | 2 hrs | TLS + Raft + Init |
| A2 Fresh | 3 hrs | AppRole + Test |
| A3 | 0 min | Done in A1 (Raft) |
| Tracking | 15 min | Update docs + commit |
| **TOTAL** | **5.75 hrs** | Clean foundation for A4+ |

---

## 📝 Questions for User

Before starting:

1. **Which .env file is your working file?**
   - .env.ce or .env?

2. **Do you have current credentials in password manager?**
   - If yes: We can verify current Vault state first
   - If no: Fresh start is mandatory

3. **Start recovery now or later?**
   - Now: I'll guide step-by-step
   - Later: Save this plan for next session

**Recommendation:** Start now with full reset. Takes 5.75 hours but gives clean production-ready foundation.
