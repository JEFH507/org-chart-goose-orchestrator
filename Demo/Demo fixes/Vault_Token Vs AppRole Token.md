
Nov 12

## Question 1: What token is used first, and where is this logic set?

**Location:** `src/controller/src/vault/mod.rs` (the Vault client initialization code)

`VAULT_TOKEN` first! Here's the actual priority(We want to change this when on production and add a Vault Agent to get the new AppRole Token before it expires):

```bash
// Pseudo-code from Controller:
if let Some(token) = env::var("VAULT_TOKEN") {
    // Use direct token
    client.set_token(&token);
} else if let (Some(role_id), Some(secret_id)) = (
    env::var("VAULT_ROLE_ID"),
    env::var("VAULT_SECRET_ID")
) {
    // Use AppRole authentication
    let auth = client.auth.approle.login(role_id, secret_id)?;
    client.set_token(&auth.client_token);
} else {
    return Err("No Vault credentials");
}
```
**VAULT_TOKEN takes precedence over AppRole.**

## Question 2: Does AppRole token auto-generate on rebuild?

**YES, it auto-generates on Controller startup!**

**What happens when you run Steps 1-12 of the[[Container_Management_Playbook]] :**

1. **Step 6** - Start Controller
2. Controller reads `.env.ce` → Finds `VAULT_ROLE_ID` + `VAULT_SECRET_ID`
3. Controller calls Vault: `POST /auth/approle/login` with those credentials
4. Vault returns NEW token (valid for 1 hour)
5. Controller uses that token for all Transit operations

**This happens EVERY TIME Controller starts**
## Question 3: Vault Token rebuild behavior

**If using Direct Token (`VAULT_TOKEN`):**

**After volume deletion (`docker compose down -v`):**

- ✅ Token is INVALID (Vault data wiped)
- ✅ You need to generate NEW token
- ✅ Update `.env.ce` with new token
- ✅ Then start Controller

**After simple restart (no volume deletion):**

- ✅ Token still valid (if not expired, total length of the Vault Toke is 32 days)
- ✅ No need to regenerate
- ✅ Just restart Controller

**AppRole credentials (`VAULT_ROLE_ID` + `VAULT_SECRET_ID`):**

- ✅ Also INVALID after volume deletion
- ✅ Generated during Vault initialization (your Step 4.1)
- ✅ Automatically added to `.env.ce` by script

---

## Question 4: AppRole token 1-hour expiration

**If you DON'T add VAULT_TOKEN to `.env.ce`:**

- ✅ Controller generates new token on startup (valid 1 hour)
- ✅ After 1 hour: Token expires
- ✅ Profile fetch fails with 403
- ✅ Agent Mesh tools won't load
- ✅ **You must restart Controller** to get new token

---

## Question 5: Command to check auto-renewal

**Check if Controller has token renewal logic(THIS IS NOT WORKING YET):**
```bash
grep -r "renew" src/controller/src/vault/
# If empty: NO auto-renewal implemented

```
**Check current token TTL (while Controller running):**
```bash
docker exec ce_controller env | grep VAULT_TOKEN
# Copy the token value, then:

docker exec ce_vault vault token lookup <TOKEN_VALUE>
# Shows: ttl, creation_time, expire_time
```

**Monitor token expiration in real-time:**

```

# Watch Controller logs for re-authentication
docker logs -f ce_controller 2>&1 | grep -i "vault\|auth\|token"

# If you see "AppRole login successful" repeatedly: AUTO-RENEWAL WORKING
# If you see it only once at startup: NO AUTO-RENEWAL
```


## Question 6: Where to add auto-renewal?

**File to modify:** `src/controller/src/vault/mod.rs`

**You would need to:**

1. Add background task spawner
2. Check token TTL every 30 minutes
3. Re-authenticate with AppRole if TTL < 5 minutes

**Complexity:** Medium (requires async Rust knowledge)

**Vault Agent is MUCH easier** (sidecar container, zero code changes)

# Answers (No Code Execution)

---

## 7: Command to Renew AppRole Token Before 1-Hour Expiration

**Short answer: You can't manually "renew" an AppRole token - you must restart the Controller.**

**What happens:**

- AppRole tokens are generated at Controller startup
- They live for 1 hour
- There's NO manual command to renew them while Controller is running
- The Controller code doesn't have renewal logic

**Your options:**

**Option A: Restart Controller (generates new token)**

`docker compose -f ce.dev.yml --profile controller restart controller # New AppRole token auto-generated, valid for 1 hour`

**Option B: Add VAULT_TOKEN to .env.ce (lasts 32 days)**

- Then you don't worry about 1-hour expiration

**Option C: Implement auto-renewal in Controller code** (future work)

---

## 8: Direct Token Persistence Without Volume Deletion

**YES, the direct token persists!**

**Scenario: `docker compose down` (WITHOUT `-v`)**

`docker compose -f ce.dev.yml --profile controller --profile privacy-guard \   --profile privacy-guard-proxy --profile ollama --profile multi-goose \   --profile redis down`

**What happens to VAULT_TOKEN:**

- ✅ Vault volume NOT deleted
- ✅ Token still valid in Vault database
- ✅ `.env.ce` still has VAULT_TOKEN value
- ✅ When you start Controller again: Token still works!

**Token lasts: 32 days (768 hours) from creation**

**After 32 days:** Token expires, you need to generate new one

**When you DON'T need to regenerate token:**

- Simple restart (no `-v`)
- Stop/start containers
- Restart Controller
- Restart Goose instances

**When you DO need to regenerate token:**

- Used `docker compose down -v` (volumes deleted)
- Token expired (after 32 days)
- Vault unsealed with different keys

---



##🚨 **VAULT AUTHENTICATION ISSUE - Not Logged In**

**The "403 permission denied" means you're not authenticated to Vault.**



### **Fix: Login with Root Token to obtain a VAULT_TOKEN**

```bash
# Step 1: Get the root token from .env.ce
# Since I can't read it, you need to copy it manually

# Find your root token in .env.ce:
grep VAULT_ROOT_TOKEN /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce

# Or if you have the original unseal output, use that root token

# Step 2: Login to Vault
docker exec -it ce_vault vault login

# Paste your root token when prompted
# (Should start with: hvs.CAESI... or s.xxx...)

# Step 3: Verify login worked
docker exec ce_vault vault token lookup

# Should show your token details

# Step 4: Now check if controller-policy exists
docker exec ce_vault vault policy read controller-policy

# Step 5: Generate new Controller token
NEW_TOKEN=$(docker exec ce_vault vault token create \
  -policy=controller-policy \
  -ttl=768h \
  -format=json | jq -r '.auth.client_token')

echo "New Vault token: $NEW_TOKEN"

```

---

### **Alternative: Use Root Token Directly (Quick for Demo, Not recommended)**

```bash

# If you don't have controller-policy, use root token temporarily
# Get root token from .env.ce (you'll need to manually copy it)

# Then update .env.ce:
# VAULT_TOKEN=<your-root-token>

# Restart Controller
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
docker compose -f ce.dev.yml --profile controller restart controller
```

Now Update .env.ce and restart:
```bash
# Step 1: Update .env.ce with new token
# Open in your text editor:
nano /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose/.env.ce

# Find this line:
# VAULT_TOKEN=
# 
# Replace with:
# VAULT_TOKEN=hvs.CAESINeT8qHenHe-8l3fS1dB2aA_RXDRROz6369qETEzOsEtGh4KHGh2cy5zQXg2dTJ6SE5IanBWRUZHclVnODlDMDk

# Save and exit (Ctrl+O, Enter, Ctrl+X)

```
---

## What We Created With This Process

**Type:** Controller Service Token (VAULT_TOKEN) (NOT a root token)

**Policy:** `controller-policy` (limited permissions)

**Relationship to Root Token:**

- Your **root token is STILL VALID** and unchanged
- Root token: Used for Vault administration (unseal, policy management, AppRole setup)
- Controller token: Used only by Controller service for Transit operations

## Policy Architecture - NOT Bypassed

**We did NOT bypass any policies!** Here's the architecture:

```
Vault Policies (Still Active):
├── controller-policy ← Controller token uses THIS
│   ├── transit/encrypt/profile-signing (allow)
│   ├── transit/verify/profile-signing (allow)
│   └── transit/sign/profile-signing (allow)
│
└── root policy ← Root token uses THIS
    └── Full admin access
```

**What the Controller token can do:**

```
`# ✅ ALLOWED (controller-policy permissions): vault write transit/verify/profile-signing input=... signature=... vault write transit/sign/profile-signing input=... vault write transit/encrypt/profile-signing plaintext=...
# ❌ DENIED (not in controller-policy): vault policy write ... vault secrets enable ... vault auth enable ... vault operator unseal ...
```

## AppRole vs Token - Two Auth Methods

**We now have BOTH methods configured:**

### 1. AppRole (Production Method)

In .env.ce: 
* VAULT_ROLE_ID=XXXX
* VAULT_SECRET_ID=XXXX
How it works: 
1. Controller authenticates: POST /auth/approle/login 
2. Vault returns token: lease_duration=3600 (1 hour) 
3. Controller uses token for Transit operations 
4. Token expires after 1 hour 
5. Controller re-authenticates automatically(NOT A FEATURE YET)

**Problem:** If Controller doesn't re-auth before expiration → 403 errors

### 2. Direct Token (Fallback)

In .env.ce: VAULT_TOKEN=XXX
How it works: 
1. Controller uses this token directly (no auth step) 
2. Token valid for 768h **(32 days)** 
3. Same permissions as AppRole token (controller-policy) 
4. No automatic renewal`

## Token Expiration Timeline

**32-day expiration applies to:**

- ✅ The new `VAULT_TOKEN` we created (768h = 32 days)

**1-hour expiration applies to:**

- ✅ AppRole-generated tokens (3600s = 1 hour)

**Never expires (unless you revoke it):**

- ✅ Root token
- ✅ AppRole credentials (ROLE_ID + SECRET_ID)

## Controller Authentication Logic

**When Controller starts, it checks:**

`if VAULT_TOKEN is set:     Use VAULT_TOKEN directly     Skip AppRole authentication else if VAULT_ROLE_ID and VAULT_SECRET_ID are set:     Authenticate with AppRole    Get short-lived token     Use that token else:     ERROR: No Vault credentials configured`

**With both configured:**

- Controller uses `VAULT_TOKEN` (direct, simpler)
- AppRole credentials stay valid (can switch back anytime)

## Why This is Safe

**1. Limited Permissions:**

`# Verify what the token can do: docker exec -e VAULT_TOKEN=hvs.CAESINeT... ce_vault \   vault token lookup  # Output shows: policies: ["controller-policy"]  # ← NOT root! ttl: 767h59m renewable: false`

**2. Same Policy as AppRole:**

`# AppRole-generated token: policies: ["controller-policy"] # Direct token we created: policies: ["controller-policy"] # Identical permissions!`

**3. Phase 6 Debugging Preserved:**

- Profile signing still uses Transit backend
- Signature verification still enforced
- Database integrity checks still active
- All security controls ACTIVE

## Comparison Table

| Aspect                  | Root Token    | AppRole Token     | Direct Token (New)        |
| ----------------------- | ------------- | ----------------- | ------------------------- |
| **Purpose**             | Vault admin   | Controller auth   | Controller auth           |
| **Policy**              | root          | controller-policy | controller-policy         |
| **TTL**                 | No expiration | 1 hour            | 32 days                   |
| **Renewable**           | N/A           | Yes(Not yet)      | No                        |
| **Can unseal Vault**    | ✅ Yes         | ❌ No              | ❌ No                      |
| **Can modify policies** | ✅ Yes         | ❌ No              | ❌ No                      |
| **Can sign profiles**   | ✅ Yes         | ✅ Yes             | ✅ Yes                     |
| **Security level**      | Full access   | Limited           | Limited (same as AppRole) |

## What We're Bypassing (Answer: Nothing!)

**Still Active:**

- ✅ Transit encryption/signing
- ✅ controller-policy restrictions
- ✅ Profile signature verification
- ✅ Database audit logging
- ✅ Privacy Guard masking

**Bypassed:**

- ❌ Nothing! (Just using different auth method)

## Recommendation for Your Setup

**Keep both configured:**

`# .env.ce should have: VAULT_ROLE_ID=xxxxxxx...      # Production method VAULT_SECRET_ID=xxxxxx...    # Production method VAULT_TOKEN=xxxxxx...    # Demo/dev fallback`

**Why?**

- AppRole = Production best practice (short-lived, renewable)
- Direct token = Demo/testing convenience (no expiration issues during long sessions)
- Both use same policy = same security

## Next Session Action

**No new concerns!** The token we created:

1. ✅ Is NOT a root token (limited to controller-policy)
2. ✅ Does NOT invalidate root token
3. ✅ Does NOT bypass Phase 6 security
4. ✅ Does NOT break AppRole (both can coexist)
5. ✅ Expires in 32 days (not 1 hour like AppRole tokens)

**Just add the line to .env.ce and restart Controller.** All your Phase 6 work is preserved! 🎯

## 1. AppRole Token Auto-Renewal

**Short answer:** AppRole tokens DON'T auto-renew in our current implementation. That's why we're adding the fallback token!

**How AppRole COULD auto-renew (not implemented):**

```bash
# Hypothetical auto-renewal logic (NOT in our Controller):
import schedule
import hvac

vault_client = hvac.Client(url=VAULT_ADDR)

def renew_token():
    # Check token TTL
    lookup = vault_client.lookup_token()
    ttl_remaining = lookup['data']['ttl']
    
    if ttl_remaining < 300:  # Less than 5 minutes
        # Re-authenticate with AppRole
        auth_response = vault_client.auth.approle.login(
            role_id=VAULT_ROLE_ID,
            secret_id=VAULT_SECRET_ID
        )
        new_token = auth_response['auth']['client_token']
        vault_client.token = new_token
        
# Run every 30 minutes
schedule.every(30).minutes.do(renew_token)
```

**Why we're using direct token instead:**

- Our Controller doesn't implement background renewal
- Adding renewal logic = more complexity
- For demo/dev: Long-lived token is simpler
- For production: You'd implement proper renewal OR use Vault Agent

**Vault Agent (Production Solution):**

```bash
# vault-agent-config.hcl
auto_auth {
  method "approle" {
    config = {
      role_id_file_path = "/vault/role-id"
      secret_id_file_path = "/vault/secret-id"
    }
  }
  
  sink "file" {
    config = {
      path = "/vault/token"  # Always fresh token here
    }
  }
}

```