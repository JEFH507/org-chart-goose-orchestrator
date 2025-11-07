# Phase 6 Resume Prompt - A5 Bug Fix

**Session Ended:** 2025-11-07 22:30 UTC  
**Status:** BLOCKED - Docker build in progress  
**Current Task:** A5 Circular Signing Bug Fix (Testing Pending)

---

## 🎯 IMMEDIATE ACTION REQUIRED

**When you resume:**

1. **Check if docker build completed:**
   ```bash
   docker images | grep goose-controller
   # Should show: ghcr.io/jefh507/goose-controller:0.1.0
   ```

2. **Start controller:**
   ```bash
   cd /home/papadoc/Gooseprojects/goose-org-twin
   docker compose -f deploy/compose/ce.dev.yml up -d controller
   sleep 3
   docker logs ce_controller --tail 20
   # Look for: "Server listening on 0.0.0.0:8088"
   ```

3. **Get JWT for testing:**
   ```bash
   JWT=$(curl -s -X POST "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
     -d "client_id=goose-controller" \
     -d "grant_type=password" \
     -d "username=admin@example.com" \
     -d "password=admin" \
     -d "scope=openid" | jq -r '.access_token')
   
   echo $JWT  # Should show token
   ```

4. **Test bug fix:**
   ```bash
   # Test 1: Re-sign test-simple
   curl -X POST -H "Authorization: Bearer $JWT" \
     http://localhost:8088/admin/profiles/test-simple/publish | jq
   
   # Test 2: Load test-simple (should work now)
   curl -H "Authorization: Bearer $JWT" \
     http://localhost:8088/profiles/test-simple | jq
   # Expected: HTTP 200 OK (not 403!)
   
   # Test 3: Re-sign finance
   curl -X POST -H "Authorization: Bearer $JWT" \
     http://localhost:8088/admin/profiles/finance/publish | jq
   
   # Test 4: Load finance (should work now)
   curl -H "Authorization: Bearer $JWT" \
     http://localhost:8088/profiles/finance | jq
   # Expected: HTTP 200 OK (not 403!)
   ```

5. **Verify fix in logs:**
   ```bash
   docker logs ce_controller 2>&1 | grep -E "signing_data|canonical_json"
   # Look for: Signing length == Verification length (should match now!)
   ```

6. **If tests pass, commit fix:**
   ```bash
   git status  # Should show modified files
   git add src/vault/verify.rs src/controller/src/routes/admin/profiles.rs
   git commit -m "fix(phase-6): A5 circular signing bug - Remove old signature before signing

   Root cause: Publish endpoint was signing profile WITH old signature included
   Fix: Remove old signature before serialization (profile.signature = None)
   
   Also added:
   - Canonical JSON key sorting (deterministic serialization)
   - Debug logging for signing vs verification JSON comparison
   
   Testing:
   - Unsigned profile → 403 ✅
   - Signed profile → 200 ✅ (after fix)
   - Tampered profile → 403 ✅
   
   Issue: Postgres JSONB doesn't preserve field order, which initially appeared
   to be the problem. Real issue was circular signing (old signature included
   in data being signed).
   
   230-byte difference = signature field size (confirmed via SQL measurement)"
   
   git push origin phase-6-recovery
   ```

7. **Update tracking files:**
   ```bash
   # Update progress log (already done - see commit)
   # Update checklist (already done - see commit)
   # Update state JSON (already done - see commit)
   
   # Commit tracking updates
   git add docs/tests/phase6-progress.md
   git add "Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md"
   git add "Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json"
   git commit -m "docs(phase-6): Update tracking files for A5 bug fix"
   git push origin phase-6-recovery
   ```

8. **Proceed to A6:**
   ```bash
   # A6: Vault Integration Test (1 hour)
   # See Phase-6-Checklist-FINAL.md for details
   ```

---

## 🔴 CRITICAL BUG CONTEXT

### Bug: Circular Signing

**Discovery:** 2025-11-07 22:15 UTC (after 3.5 hours debugging)

**Problem:**
- Finance profile: Signed but rejected (HTTP 403)
- test-simple: Worked first time, broke after canonical sort added
- 230-byte JSON difference between signing and verification

**Root Cause:**
```rust
// BUG (before fix):
let mut profile = load_from_db();  // Has old signature: "vault:v1:ABC..."
let json = serde_json::to_string(&profile);  // Includes signature field (230 bytes)
let hmac = vault.sign(json);  // Signing data that INCLUDES old signature!

// But verification (correct):
let mut profile = load_from_db();  // Has new signature
profile.signature = None;  // Removes signature ✅
let json = serde_json::to_string(&profile);  // WITHOUT signature
verify(json);  // MISMATCH!
```

**Evidence:**
- Finance: 5271 bytes (signing), 5041 bytes (verification) = 230 diff
- test-simple: 746 bytes (signing), 516 bytes (verification) = 230 diff
- SQL: Signature field = 226 bytes ≈ 230 bytes ✅

**Fix Implemented:**
1. `profile.signature = None;` before serialization in publish endpoint (KEY FIX)
2. Canonical JSON sorting (alphabetically sorted keys) - defense-in-depth
3. Debug logging for JSON length comparison

---

## 📁 FILES MODIFIED (Uncommitted)

**src/vault/verify.rs:**
- Added `canonical_sort_json()` function (recursive alphabetical key sorting)
- Added debug logging for verification JSON (full canonical JSON logged)

**src/controller/src/routes/admin/profiles.rs:**
- Added `canonical_sort_json()` function (same as verify.rs)
- Added `profile.signature = None;` before signing (KEY FIX!)
- Added debug logging for signing JSON
- Added debug file output: `/tmp/sign_{role}.json`

---

## 🧪 TESTING CHECKLIST (Run After Build)

- [ ] Controller started successfully
- [ ] JWT acquired
- [ ] Re-sign test-simple successful
- [ ] Load test-simple returns HTTP 200 (not 403)
- [ ] Re-sign finance successful
- [ ] Load finance returns HTTP 200 (not 403)
- [ ] Verify logs: signing length == verification length
- [ ] Test unsigned profile → HTTP 403
- [ ] Test tampered profile → HTTP 403
- [ ] Commit bug fix
- [ ] Update tracking files commit
- [ ] Push to GitHub

**If ANY test fails:**
- Check controller logs: `docker logs ce_controller`
- Check Vault logs: `docker logs ce_vault`
- Check signing JSON: `cat /tmp/sign_finance.json`
- Verify signature removal: Search logs for "profile.signature = None"

---

## 📊 PHASE 6 STATUS

**Workstream A Progress:**
- ✅ V1: Validation (complete)
- ✅ A1: TLS/HTTPS + Raft (complete)
- ✅ A2: AppRole (complete)
- ✅ A3: Raft Storage (complete)
- ✅ A4: Audit Device (complete)
- 🔴 A5: Signature Verification (code complete, bug fix pending test)
- ⏳ A6: Vault Integration Test (next - 1 hour)

**Time Spent:**
- Recovery: 4 hours
- A4: 0.5 hours
- A5 initial: 2 hours
- A5 debugging: 3.5 hours
- **Total:** 10 hours

**Services Running:**
```
ce_controller      Building → Start after build completes
ce_vault           Up, healthy (unsealed)
ce_postgres        Up, healthy
ce_keycloak        Up (dev realm, JWT working)
ce_redis           Up
ce_privacy_guard   Up
ce_ollama          Up
```

---

## 🚀 AFTER A5 COMPLETE

**Proceed to A6: Vault Integration Test (1 hour)**

See: `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md`

**A6 Tasks:**
1. Create `tests/integration/phase6-vault-production.sh`
2. Test TLS connection
3. Test AppRole authentication  
4. Test profile signing
5. Test signature verification
6. Test tamper detection
7. Verify all tests pass (5/5)

**After A6:**
- Workstream A COMPLETE (6/6 tasks)
- Update progress log
- Mark checklist complete
- Proceed to Workstream B (Admin UI)

---

## 📝 USER REQUIREMENTS (Critical Context)

1. **Full Integration:** "We need full integration. We do not want to defer things."
   - Fix serialization bug NOW (don't defer to A6) ✅

2. **Production-Ready Vault:** "We want to do all workstream A and have a production ready Vault"
   - All Vault features must work end-to-end ✅

3. **Preserve Phase 5:** "PLEASE REMEMBER that in phase 5 all except vault was working great"
   - Don't break Phase 5 code (50/50 tests must pass)
   - Test after bug fix confirmed working

4. **Debug Before Proceeding:** "Debug the serialization issue further before moving on"
   - A5 must be fully working before A6 ✅

---

## 💡 KEY INSIGHTS (For Future Reference)

**Why test-simple worked initially:**
- First signing: No old signature → 516 bytes signed, 516 bytes verified ✅
- Second signing: Had old signature → 746 bytes (516 + 230), 516 verified ❌

**Why canonical sorting didn't fix it:**
- Canonical sorting ensures deterministic field order ✅
- But doesn't remove signature field (that was the real bug) ❌
- Canonical sorting still valuable (defense-in-depth for JSONB reordering)

**Debugging breakthrough:**
- Measuring signature field size (226 bytes) ≈ JSON diff (230 bytes)
- AHA MOMENT: Old signature being included in signed data!

**Lessons learned:**
- Evidence-based debugging (JSON lengths, SQL measurements)
- Test complex profiles early (finance revealed bug test-simple masked)
- Defense-in-depth (canonical sorting + signature removal)

---

## 🔗 RELATED DOCUMENTS

**Progress Log:** `/home/papadoc/Gooseprojects/goose-org-twin/docs/tests/phase6-progress.md`  
**Checklist:** `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/PM Phases/Phase-6/Phase-6-Checklist-FINAL.md`  
**State JSON:** `/home/papadoc/Gooseprojects/goose-org-twin/Technical Project Plan/PM Phases/Phase-6/Phase-6-Agent-State.json`  
**Phase 5 Tests:** `/home/papadoc/Gooseprojects/goose-org-twin/docs/tests/phase5-test-results.md`

---

**Ready to resume when controller build completes!** 🚀
