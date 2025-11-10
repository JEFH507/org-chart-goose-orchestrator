# Vault Credentials Management Guide

**Created:** 2025-11-07  
**Purpose:** Secure management of Vault unseal keys and root tokens  
**Audience:** Developers and operators

---

## ⚠️ Security Principles

### NEVER do this:
- ❌ Paste credentials in chat/Slack/email
- ❌ Commit credentials to git
- ❌ Store credentials in plaintext files
- ❌ Share credentials via screenshots
- ❌ Log credentials to stdout/stderr

### ALWAYS do this:
- ✅ Store credentials in password manager (1Password, Bitwarden, LastPass)
- ✅ Use environment variables at runtime
- ✅ Rotate credentials regularly
- ✅ Use AppRole authentication (not root tokens) in production
- ✅ Keep unseal keys offline (paper backup in safe)

---

## 🔐 Initial Vault Setup (What You Just Did)

### Step 1: Initialize Vault (Generates Credentials)

```bash
docker exec ce_vault vault operator init -key-shares=1 -key-threshold=1
```

**Output (EXAMPLE - yours will be different):**
```
Unseal Key 1: ABC123...XYZ789

Initial Root Token: hvs.ABC123...XYZ789
```

**What to do:**
1. **Immediately** copy these to your password manager
2. Label them as: "Vault Dev - Unseal Key" and "Vault Dev - Root Token"
3. Add tags: `vault`, `dev`, `phase-6`
4. **DO NOT** paste them anywhere else

---

### Step 2: Unseal Vault (Use the Unseal Key)

```bash
docker exec ce_vault vault operator unseal <YOUR_UNSEAL_KEY>
```

**What this does:**
- "Unlocks" Vault so it can decrypt secrets
- Must be done after every Vault restart
- In production, you'd need 3 of 5 keys (Shamir secret sharing)

---

### Step 3: Authenticate with Root Token

```bash
docker exec -e VAULT_TOKEN=<YOUR_ROOT_TOKEN> ce_vault vault secrets enable transit
```

**What this does:**
- Proves you're an admin (root token = master password)
- Grants full permissions to configure Vault
- Should be replaced with AppRole in production (Phase 6 Task A2)

---

## 🔄 When You Need to Reinitialize Vault

**Scenario:** Credentials were accidentally exposed (like in chat logs)

### Full Reinitialization Process:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# 1. Stop Vault
docker compose -f deploy/compose/ce.dev.yml down vault vault-init

# 2. Delete the data (DESTROYS ALL SECRETS!)
docker volume rm compose_vault_data

# 3. Start Vault fresh
docker compose -f deploy/compose/ce.dev.yml up -d vault
sleep 10

# 4. Initialize with NEW credentials
docker exec ce_vault vault operator init -key-shares=1 -key-threshold=1

# 5. Save output to password manager (NOT in chat!)

# 6. Unseal with NEW unseal key
docker exec ce_vault vault operator unseal <NEW_UNSEAL_KEY>

# 7. Enable transit engine with NEW root token
docker exec -e VAULT_TOKEN=<NEW_ROOT_TOKEN> ce_vault vault secrets enable transit

# 8. Verify it works
docker exec -e VAULT_TOKEN=<NEW_ROOT_TOKEN> ce_vault vault secrets list
```

**CRITICAL:** After step 4, **STOP** and save credentials before proceeding!

---

## 📝 Where Credentials Are Used

### 1. Unseal Key
**Used:** After every Vault restart  
**Frequency:** Rare (only when container restarts)  
**Storage:** Password manager + paper backup in safe

**How to use:**
```bash
docker exec ce_vault vault operator unseal $(pass show vault-dev/unseal-key)
```

### 2. Root Token
**Used:** For admin operations (Phase 6 only, will be replaced)  
**Frequency:** Temporary (until AppRole is set up in A2)  
**Storage:** Password manager

**How to use:**
```bash
export VAULT_TOKEN=$(pass show vault-dev/root-token)
docker exec -e VAULT_TOKEN ce_vault vault <command>
```

---

## 🔧 Production Best Practices (Phase 6 A2-A4)

### 1. Use AppRole Instead of Root Token (Task A2)
**What:** Service authentication using role_id + secret_id  
**Why:** Root token = all permissions forever, AppRole = limited permissions with expiration  
**When:** Implemented in Phase 6 Task A2

### 2. Use 5 Unseal Keys with 3-of-5 Threshold (Task A3)
**What:** Split unseal key into 5 pieces, need any 3 to unlock  
**Why:** No single person can unseal Vault alone (prevents insider threats)  
**When:** Implemented in Phase 6 Task A3

**Example:**
```bash
vault operator init -key-shares=5 -key-threshold=3
```

**Key distribution:**
- Key 1: CTO (safe deposit box)
- Key 2: Lead Engineer (password manager)
- Key 3: DevOps Lead (password manager)
- Key 4: Security Officer (hardware security module)
- Key 5: CEO (paper in safe)

### 3. Enable Auto-Unseal with Cloud KMS (Production Only)
**What:** Use AWS KMS / GCP KMS / Azure Key Vault to auto-unseal  
**Why:** No manual intervention needed after restart  
**When:** Production deployment (not Phase 6)

**Example (AWS):**
```hcl
seal "awskms" {
  region     = "us-east-1"
  kms_key_id = "arn:aws:kms:us-east-1:123456789012:key/abc123"
}
```

---

## 🚨 Incident Response: Credentials Compromised

### If Credentials Are Exposed:

1. **IMMEDIATELY rotate:**
   ```bash
   # For root token
   vault token revoke <EXPOSED_TOKEN>
   
   # For AppRole (Phase 6 A2+)
   vault write -f auth/approle/role/controller-role/secret-id
   ```

2. **Reinitialize Vault** (follow process above)

3. **Update all services** with new credentials

4. **Audit logs:**
   ```bash
   vault audit list
   cat /vault/logs/audit.log | grep <EXPOSED_TOKEN>
   ```

5. **Document in incident log:**
   - When: Timestamp
   - What: Credential type exposed
   - How: Exposure method (chat log, git commit, etc.)
   - Actions: Rotation, reinitialization, etc.

---

## 📚 Quick Reference

### View Vault Status
```bash
docker exec ce_vault vault status
```

### Check if Unsealed
```bash
docker exec ce_vault vault status | grep Sealed
# Output: Sealed  false  (good)
# Output: Sealed  true   (need to unseal)
```

### List Enabled Secrets Engines
```bash
docker exec -e VAULT_TOKEN=<TOKEN> ce_vault vault secrets list
```

### Test Transit Engine
```bash
echo -n "test" | base64 | \
docker exec -i -e VAULT_TOKEN=<TOKEN> ce_vault \
  vault write transit/hmac/profile-signing/sha2-256 input=-
```

---

## 🔗 Related Documentation

- Phase 6 Checklist: `Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md`
- Vault Operations Guide: `docs/guides/VAULT.md`
- Security Policy: `SECURITY.md` (to be created in Phase 6 F5)

---

## 📅 Credential Rotation Schedule

| Credential Type | Rotation Frequency | Owner | Next Rotation |
|----------------|-------------------|-------|---------------|
| Root Token (dev) | After exposure OR Phase 6 A2 completion | Developer | A2 completion (switch to AppRole) |
| Unseal Key (dev) | Rarely (only if compromised) | Developer | As needed |
| AppRole Secret ID | Every 30 days | CI/CD | A2 + 30 days |
| TLS Certificates | Every 365 days | DevOps | 2026-11-07 |

---

**Remember:** Vault security is only as strong as your credential management. Treat these like the keys to your house!
