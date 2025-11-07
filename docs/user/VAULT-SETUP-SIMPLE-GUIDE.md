# Simple Guide: Vault Setup (Fresh Recovery - 2025-11-07)

**Date:** 2025-11-07 20:00 UTC  
**For:** Non-technical users  
**Recovery:** Fresh start with production-ready Raft storage  
**Time:** 4 hours (setup complete)

---

## 🎯 What Is Vault?

Think of Vault as a **super-secure safe** for your digital secrets:
- Passwords
- Encryption keys  
- Digital signatures
- **NEW:** Built-in backup system (Raft storage)

In our project, we use it to **prove that profile files haven't been tampered with** (like a wax seal on a letter).

**What changed in recovery:**
- ✅ Upgraded storage: File → **Raft** (production-ready, HA-capable)
- ✅ Fresh credentials (all old keys/tokens replaced)
- ✅ One clean set of credentials (no more duplicates)

---

## 🔐 The 4 Steps We Did

### Step 1: Created a Lock and Key (TLS Certificates)

**What:** Made two files that encrypt network traffic  
**Why:** So hackers can't spy on secrets traveling over the network  
**Like:** Putting your letter in a locked envelope instead of a postcard

**Files created:**
- `vault.crt` - The padlock (public, anyone can see it)
- `vault-key.pem` - The key to the padlock (secret, only Vault can use it)

**Command you ran:**
```bash
openssl req -newkey rsa:2048 -nodes -keyout vault-key.pem -x509 -days 365 -out vault.crt -subj "/CN=vault/O=OrgChart/C=US"
```

---

### Step 2: Gave Vault Permission to Use the Key

**What:** Changed who "owns" the certificate files  
**Why:** Vault runs as a special user (UID 100) inside its container. It couldn't read files owned by your regular user.

**Like:** Giving your house key to your friend who's house-sitting

**Command you ran:**
```bash
sudo chown 100:100 vault.crt vault-key.pem
sudo chmod 400 vault-key.pem  # Make private key read-only
```

**What the permissions mean:**
- `400` = Only the owner can read (very secure!)
- `644` = Owner can read/write, others can read (for public cert)

---

### Step 3: Created a Brand New Safe (Initialize Vault)

**What:** Created Vault's master password and unlock keys  
**Why:** First-time setup - like setting the combination on a new safe  
**RECOVERY NOTE:** We used 5 keys (production-ready) instead of 1

**Command you ran:**
```bash
docker exec -it ce_vault vault operator init
```

**Output (you saved this to password manager):**
- **5 Unseal Keys:** Five different combinations to unlock the safe
  - You need ANY 3 of the 5 keys to unlock (redundancy!)
  - Like having 5 parts of a treasure map, only need 3 to find the treasure
- **Root Token:** The master admin password

**⚠️ SUPER IMPORTANT:** These should NEVER be written down in code, git, or chat! Only in your password manager.

**Why 5 keys?**
- **Security:** No single person can unlock Vault alone
- **Redundancy:** If you lose 2 keys, you still have 3 others
- **Production-ready:** Industry standard (3 of 5 threshold)

---

### Step 4: Unlocked the Safe (Unseal Vault)

**What:** Used 3 unseal keys to unlock Vault  
**Why:** Vault starts "locked" for security. You must unlock it to use it.

**Commands you ran (3 times with different keys):**
```bash
docker exec -it ce_vault vault operator unseal
# Paste unseal key 1 when prompted

docker exec -it ce_vault vault operator unseal
# Paste unseal key 2 when prompted

docker exec -it ce_vault vault operator unseal
# Paste unseal key 3 when prompted
```

**What happened:**
- After key 1: Progress 1/3 (still sealed)
- After key 2: Progress 2/3 (still sealed)
- After key 3: Progress 3/3 → **UNSEALED!** ✅

**Like:** Three people putting their keys in a bank vault together to open it.

---

## 🚀 How It Works Now

### Before (Insecure):
```
Your computer → Vault
    ↓ (unencrypted HTTP)
Hackers can see secrets! 😱
```

### After (Secure):
```
Your computer → Vault
    ↓ (encrypted HTTPS with TLS)
Hackers see gibberish! 🔐✅
```

---

## 🔄 When You Restart Your Computer

Vault will be "sealed" (locked) again. You need to unseal it:

```bash
# Check if Vault is sealed
docker exec ce_vault vault status

# If "Sealed: true", unseal it:
docker exec ce_vault vault operator unseal <YOUR_UNSEAL_KEY>
```

**Tip:** Keep your unseal key in your password manager for easy access!

---

## ✅ What We Completed in Recovery

### Step 5: AppRole Authentication (A2) ✅ DONE

**What:** Replaced root token with AppRole credentials  
**Why:** Root token = skeleton key (all permissions forever). AppRole = temporary key (1 hour) that auto-expires.

**What you got:**
- **ROLE_ID:** Public identifier (safe to share, like a username)
- **SECRET_ID:** Secret password (never share!)

Both saved in your password manager ✅

**Like:** Switching from a master key to a temporary hotel room key that expires after checkout.

---

### Step 6: Raft Storage (A3) ✅ DONE

**What:** Enabled Raft storage backend  
**Why:** Built-in backup and high-availability (HA)

**What changed:**
- **Before:** File storage (single point of failure)
- **After:** Raft storage (production-ready, can survive node failures)

**Like:** Upgrading from one notebook to a synchronized cloud system with automatic backups.

---

## 🛠️ What's Next?

**Task A4 (Up Next):** Enable audit logs  
**Why:** Track every operation for compliance and security monitoring

**Task A5:** Signature verification on profile load  
**Why:** Detect tampering automatically

**Task A6:** Integration tests  
**Why:** Prove everything works end-to-end

---

## 📚 Analogy Summary

| Vault Concept | Real-World Analogy |
|--------------|-------------------|
| Vault | A super-secure safe |
| Unseal Key | Combination to open the safe |
| Root Token | Master admin password |
| TLS Certificate | Locked envelope for mail |
| Transit Engine | Wax seal maker (for document integrity) |
| Sealed State | Safe is locked |
| Unsealed State | Safe is open and usable |

---

## 🗑️ Deleting Old Vault Credentials

### YES, You Can Delete Old Credentials! ✅

**What to delete from your password manager:**
- ❌ Old unseal keys (from previous initializations)
- ❌ Old root tokens (replaced by new one)
- ❌ Old AppRole ROLE_IDs (replaced by new one)
- ❌ Old AppRole SECRET_IDs (replaced by new one)

**What to KEEP:**
- ✅ Current 5 unseal keys (generated 2025-11-07 20:00)
- ✅ Current root token (generated 2025-11-07 20:00)
- ✅ Current ROLE_ID (generated 2025-11-07 20:10)
- ✅ Current SECRET_ID (generated 2025-11-07 20:10)

**How to identify current vs old:**
- Current credentials = dated 2025-11-07 (today's recovery)
- Old credentials = earlier dates (before recovery)

**Why it's safe to delete:**
- Old Vault data was wiped during recovery
- Old credentials no longer work (Vault was reinitialized)
- Keeping old credentials = confusion risk

**Recommended password manager organization:**
```
Vault Credentials - CURRENT (2025-11-07 Recovery)
├── Unseal Key 1
├── Unseal Key 2
├── Unseal Key 3
├── Unseal Key 4
├── Unseal Key 5
├── Root Token
├── AppRole ROLE_ID
└── AppRole SECRET_ID

🗑️ Archive or Delete:
└── Vault Credentials - OLD (pre-2025-11-07) ← DELETE THESE
```

**IMPORTANT:** Only delete credentials labeled as "old" or with dates before 2025-11-07. Never delete your current credentials!

---

## ❓ Common Questions

**Q: Why do I need to unseal Vault after restart?**  
A: Security! Even if someone steals your hard drive, they can't get the secrets without the unseal key.

**Q: Can I skip the unseal step?**  
A: Not yet. In production, we'll use "auto-unseal" with cloud services (AWS KMS), but that's later.

**Q: What if I lose my unseal key?**  
A: You need 3 of 5 keys. If you lose 1-2 keys, you're fine. If you lose 3+, you'll need to reinitialize Vault (destroys all data). In true production, different people hold different keys.

**Q: Can I delete old Vault users/keys/roots/AppRoles?**  
A: **YES!** We did a fresh initialization, so old credentials are already invalid. Safe to delete from password manager. See "Deleting Old Vault Credentials" section above.

**Q: What if I accidentally delete current credentials?**  
A: For development, we can reinitialize Vault (loses data). For production, this is why we have 5 keys with different people (redundancy). **Always keep backups of current credentials!**

**Q: Why did we do a recovery/fresh start?**  
A: Previous agent had security incidents (exposed keys in chat) and storage confusion (file vs Raft). Fresh start = clean, secure foundation with production-ready Raft storage.

---

**You now understand Vault setup! 🎓**

**Next:** Continue to A4 (Audit Device) to track all Vault operations for compliance.
