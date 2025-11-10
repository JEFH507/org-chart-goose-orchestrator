# Complete System Reference - Quick Start for New Agents

**Version:** 1.0.0  
**Created:** 2025-11-10  
**Purpose:** One-stop reference to prevent future agents from getting stuck

---

## 🎯 Quick Links

- **Startup Guide:** [STARTUP-GUIDE.md](./STARTUP-GUIDE.md) - Step-by-step service startup
- **Architecture Map:** [SYSTEM-ARCHITECTURE-MAP.md](./SYSTEM-ARCHITECTURE-MAP.md) - Where everything lives
- **Testing Guide:** [TESTING-GUIDE.md](./TESTING-GUIDE.md) - How to run tests

---

## 🚀 Quick Start (5-Minute Setup)

### 1. Start All Services

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Step 1: Start core infrastructure (60s)
docker compose -f ce.dev.yml up -d postgres keycloak vault
sleep 60

# Step 2: Unseal Vault (30s) - REQUIRES USER INPUT
cd ../..
./scripts/vault-unseal.sh
# Enter 3 unseal keys from password manager

# Step 3: Initialize Vault & Database (40s)
cd deploy/compose
docker compose -f ce.dev.yml up -d vault-init
sleep 10

cd ../..
docker exec ce_postgres psql -U postgres -c "CREATE DATABASE orchestrator;" 2>/dev/null || true
docker exec -i ce_postgres psql -U postgres -d orchestrator < deploy/migrations/001_create_schema.sql
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0002_create_profiles.sql
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_create_org_users.sql
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0005_create_privacy_audit_logs.sql

# Step 4: Start feature services (75s)
cd deploy/compose
docker compose -f ce.dev.yml --profile ollama up -d ollama
sleep 20
docker compose -f ce.dev.yml --profile ollama --profile privacy-guard up -d privacy-guard
sleep 15
docker compose -f ce.dev.yml --profile redis up -d redis
sleep 10
docker compose -f ce.dev.yml --profile controller --profile redis --profile ollama --profile privacy-guard up -d controller
sleep 20

# Step 5: Verify (10s)
docker ps --format "table {{.Names}}\t{{.Status}}"
curl -s http://localhost:8088/status | jq
curl -s http://localhost:8089/status | jq
```

**Total Time:** ~5 minutes (including Vault unseal)

---

## ⚠️ Critical Things New Agents Must Know

### 1. Vault ALWAYS Starts Sealed

**THE PROBLEM:** Vault requires 3 of 5 unseal keys after EVERY restart.

**SOLUTION:** 
```bash
./scripts/vault-unseal.sh
# Paste 3 keys from password manager
```

**WHY:** Security feature, not a bug. Prevents unauthorized access if server compromised.

---

### 2. Modules vs Services (Common Confusion!)

**MODULES (NOT Services):**
- `src/lifecycle/` - Rust library module (imported by Controller)
- `src/vault/` - Rust library module (imported by Controller)
- `src/profile/` - Rust library module (imported by Controller)

**SERVICES (Docker Containers):**
- `src/controller/` - HTTP API service (Axum, port 8088)
- `src/privacy-guard/` - PII masking service (Axum, port 8089)
- `src/agent-mesh/` - MCP extension (Python subprocess, no port)

**KEY INSIGHT:** 
- Lifecycle module is imported but NOT wired into routes yet (Phase 6 task)
- You can't "start" lifecycle - it's a library, not a service!

---

### 3. AppRole Tokens Expire After 1 Hour

**THE PROBLEM:** Controller logs show "invalid token" after ~1 hour.

**SYMPTOMS:**
- GET /profiles/{role} → 403 Forbidden
- Controller logs: "Vault authentication failed"

**SOLUTION:**
```bash
docker compose -f deploy/compose/ce.dev.yml --profile controller restart controller
sleep 10
```

**WHY:** AppRole tokens expire (renewable but not auto-renewed in current version).

---

### 4. Profile YAMLs ≠ Database Profiles (✅ NOW AUTOMATED!)

**UPDATE (2025-11-10):** This issue is now **RESOLVED**!

8 profile YAML files exist in `/profiles/`:
- finance.yaml, legal.yaml, manager.yaml, hr.yaml, developer.yaml, support.yaml, analyst.yaml, marketing.yaml

✅ **All 8 profiles** are now automatically loaded into database via migration:
- Migration: `db/migrations/metadata-only/0006_seed_profiles.sql`
- Auto-runs during database setup
- Idempotent (safe to re-run)

**NO MANUAL LOADING NEEDED!** Just run the migration during Step 5 of startup.

---

### 5. Docker Compose Profiles (Not User Profiles!)

**CONFUSION:** "Profile" has two meanings in this project!

**User Profiles:** Finance, Legal, Manager, etc. (role-based configs)

**Docker Compose Profiles:** Service groupings for optional components

```yaml
# Optional services (require --profile flag)
ollama:        # --profile ollama
privacy-guard: # --profile privacy-guard
redis:         # --profile redis
controller:    # --profile controller
```

**EXAMPLE:**
```bash
# Start all profiles
docker compose -f ce.dev.yml \
  --profile ollama \
  --profile privacy-guard \
  --profile redis \
  --profile controller \
  up -d
```

---

### 6. Database Migrations Must Run in Order

**CRITICAL ORDER:**

1. `001_create_schema.sql` - Core tables (sessions, tasks, approvals, audit_events)
2. `0002_create_profiles.sql` - Profiles table
3. `0004_create_org_users.sql` - Org users table (references profiles.role)
4. `0005_create_privacy_audit_logs.sql` - Privacy audit logs

**WHY:** Foreign key dependency: `org_users.role` → `profiles.role`

**WRONG ORDER = ERROR:** `relation "profiles" does not exist`

---

### 7. Privacy Guard Requires Ollama

**THE PROBLEM:** Starting privacy-guard alone fails.

**ERROR:** `service "privacy-guard" depends on undefined service "ollama"`

**SOLUTION:**
```bash
# Start BOTH profiles together
docker compose -f ce.dev.yml --profile ollama --profile privacy-guard up -d
```

**WHY:** Privacy Guard uses Ollama for NER model inference (qwen3:0.6b).

---

## 📂 File Locations Reference

### Where to Find Things

| What You Need | Location | Type |
|---------------|----------|------|
| **Start services** | `deploy/compose/ce.dev.yml` | Docker Compose |
| **Environment config** | `deploy/compose/.env.ce` | ENV file (secrets) |
| **Unseal Vault** | `scripts/vault-unseal.sh` | Bash script |
| **Get JWT token** | `scripts/get-jwt-token.sh` | Bash script |
| **Database migrations** | `deploy/migrations/`, `db/migrations/metadata-only/` | SQL files |
| **Profile YAMLs** | `profiles/*.yaml` | YAML files |
| **Controller code** | `src/controller/` | Rust (Axum service) |
| **Privacy Guard code** | `src/privacy-guard/` | Rust (Axum service) |
| **Lifecycle module** | `src/lifecycle/` | Rust (library module) |
| **Vault module** | `src/vault/` | Rust (library module) |
| **Profile module** | `src/profile/` | Rust (library module) |
| **Agent Mesh extension** | `src/agent-mesh/` | Python (MCP server) |
| **Test scripts** | `scripts/test-*.sh` | Bash scripts |
| **Test results** | `docs/tests/phase*-progress.md` | Markdown docs |

---

## 🔍 Common Troubleshooting

### "Vault is sealed"

```bash
./scripts/vault-unseal.sh
```

### "Database orchestrator does not exist"

```bash
docker exec ce_postgres psql -U postgres -c "CREATE DATABASE orchestrator;"
```

### "Profile not found" (403/404)

```bash
# Option 1: Restart controller (fresh Vault token)
docker compose -f deploy/compose/ce.dev.yml --profile controller restart controller

# Option 2: Check if profile exists in DB
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT role FROM profiles WHERE role = 'finance';"
```

### "Connection refused" to Privacy Guard

```bash
# Start with ollama profile
docker compose -f deploy/compose/ce.dev.yml \
  --profile ollama --profile privacy-guard up -d privacy-guard
```

### "Table does not exist"

```bash
# Run migrations in order
docker exec -i ce_postgres psql -U postgres -d orchestrator < deploy/migrations/001_create_schema.sql
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0002_create_profiles.sql
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_create_org_users.sql
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0005_create_privacy_audit_logs.sql
```

---

## 🧪 Testing Quick Reference

### Run All Tests

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# 1. Get JWT
export JWT=$(./scripts/get-jwt-token.sh)

# 2. Privacy Guard test (8 tests, ~2 min)
./scripts/test-finance-pii-jwt.sh

# 3. Vault production test (7 tests, ~2 min)
./scripts/test-vault-production.sh

# 4. Manual API tests
curl -H "Authorization: Bearer $JWT" http://localhost:8088/profiles/finance | jq
curl -s http://localhost:8088/status | jq
curl -s http://localhost:8089/status | jq
```

### Expected Results

```
FINANCE PII TEST: 8/8 PASSED ✅
VAULT PRODUCTION TEST: 7/7 PASSED ✅
```

---

## 📊 Service Ports

| Service | Port | URL | Notes |
|---------|------|-----|-------|
| Controller | 8088 | http://localhost:8088 | Main API |
| Privacy Guard | 8089 | http://localhost:8089 | PII masking |
| Keycloak | 8080 | http://localhost:8080 | Auth (admin/admin) |
| Vault HTTPS | 8200 | https://localhost:8200 | External access |
| Vault HTTP | 8201 | http://localhost:8201 | Internal Docker |
| Postgres | 5432 | postgresql://... | Database |
| Redis | 6379 | redis://localhost:6379 | Cache |
| Ollama | 11434 | http://localhost:11434 | LLM models |

---

## 🔐 Credentials Reference

**⚠️ NEVER commit these to git!**

### Keycloak
- URL: http://localhost:8080
- Admin: admin / admin
- Realm: dev
- Client: goose-controller
- Client Secret: `<YOUR_KEYCLOAK_CLIENT_SECRET>` (in .env.ce - get from Keycloak admin console)

### Postgres
- Host: localhost:5432
- User: postgres
- Password: postgres
- Database: orchestrator

### Vault
- URL: https://localhost:8200 (HTTPS) or http://localhost:8201 (HTTP)
- Root Token: `root` (dev mode - CHANGE IN PRODUCTION)
- Unseal Keys: 3 of 5 (in password manager)
- AppRole ID: `<YOUR_VAULT_ROLE_ID>` (in .env.ce - get from `vault read auth/approle/role/orchestrator-controller/role-id`)
- AppRole Secret: `<YOUR_VAULT_SECRET_ID>` (in .env.ce, expires 1hr - get from `vault write -f auth/approle/role/orchestrator-controller/secret-id`)

### Redis
- Host: localhost:6379
- No password (dev mode)

---

## 📈 Service Dependencies

```
Controller requires:
├─ Postgres (healthy)
├─ Vault (healthy, unsealed)
└─ Keycloak (for JWT verification)

Privacy Guard requires:
├─ Vault (healthy, unsealed)
└─ Ollama (healthy, model loaded)

Vault Init requires:
└─ Vault (healthy, unsealed)
```

**Startup Order:**
1. Postgres, Keycloak, Vault (parallel)
2. Unseal Vault (manual)
3. Vault Init
4. Database migrations
5. Ollama
6. Privacy Guard
7. Redis
8. Controller

---

## 🎓 Learning Resources

### For Understanding Architecture
- **System Architecture Map:** `docs/operations/SYSTEM-ARCHITECTURE-MAP.md`
- **Phase 3 Completion:** `Technical Project Plan/PM Phases/Phase-3/Phase-3-Completion-Summary.md`
- **Phase 5 Architecture:** `docs/architecture/PHASE5-ARCHITECTURE.md`

### For Understanding Modules
- **Lifecycle Module:** `src/lifecycle/mod.rs` + `docs/tests/phase5-progress.md`
- **Vault Module:** `src/vault/mod.rs` + `docs/guides/VAULT.md`
- **Profile Module:** `src/profile/mod.rs` + `db/migrations/metadata-only/0002_create_profiles.sql`

### For Understanding Privacy Guard
- **Privacy Guard Architecture:** `docs/architecture/ARCHITECTURE-ALIGNED-RECOMMENDATIONS.md`
- **Test Results:** `docs/tests/phase5-test-results.md`

### For Understanding Agent Mesh
- **Agent Mesh Testing:** `Technical Project Plan/PM Phases/Phase-3/TESTING-STRATEGY.md`
- **Agent Mesh Readme:** `src/agent-mesh/README.md`

---

## 🛑 Known Limitations

### 1. Lifecycle Module Not Wired
- **Status:** Code complete, imported, but NOT called in routes
- **Impact:** Session lifecycle FSM not active
- **Fix:** Phase 6 task (wire into Controller routes)

### 2. Only Finance Profile in Database
- **Status:** 7 profile YAMLs exist, only 1 loaded
- **Impact:** Legal, Manager, HR, Developer, Support, Analyst, Marketing not usable
- **Fix:** Profile loading script (Phase 6 task)

### 3. AppRole Token Expiry
- **Status:** Tokens expire after 1 hour
- **Impact:** Controller needs restart after 1 hour
- **Fix:** Automated token renewal (Phase 6 or 7)

### 4. No Automated Vault Unseal
- **Status:** Manual unseal required after restart
- **Impact:** Requires human intervention
- **Fix:** Auto-unseal (Cloud KMS, transit unseal, or operator pattern)

### 5. Database Not Persisted
- **Status:** Postgres data in ephemeral container storage
- **Impact:** Data lost on `docker compose down -v`
- **Fix:** Add volume mount for postgres data (production requirement)

---

## 📝 Cheat Sheet

### One-Liner Commands

```bash
# Get all container status
docker ps --format "table {{.Names}}\t{{.Status}}"

# Check if Vault is unsealed
docker exec ce_vault vault status | grep Sealed

# Get JWT token
export JWT=$(./scripts/get-jwt-token.sh)

# Test Controller
curl -H "Authorization: Bearer $JWT" http://localhost:8088/status | jq

# Test Privacy Guard
curl http://localhost:8089/status | jq

# Check database tables
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"

# Check database profiles
docker exec ce_postgres psql -U postgres -d orchestrator -c \
  "SELECT role, display_name FROM profiles;"

# Restart Controller (fresh Vault token)
docker compose -f deploy/compose/ce.dev.yml --profile controller restart controller && sleep 10

# Full shutdown
cd deploy/compose && docker compose -f ce.dev.yml \
  --profile controller --profile redis --profile ollama --profile privacy-guard down

# Check logs
docker logs ce_controller --tail 50
docker logs ce_privacy_guard --tail 50
docker logs ce_vault --tail 50
```

---

## ✅ Phase 6 Readiness Checklist

Before starting Phase 6 planning:

- [x] All services start successfully
- [x] Vault unseals correctly
- [x] Database migrations run without errors
- [x] 8 tables created (verified)
- [x] Privacy Guard test: 8/8 passing
- [x] Comprehensive profile test: 20/20 passing
- [x] All 6 main profiles loading successfully
- [x] All 8 profiles exist in database (analyst, developer, finance, hr, legal, manager, marketing, support)
- [x] Postgres persistence configured (postgres_data volume)
- [x] Keycloak persistence configured (keycloak_data volume)
- [x] Profile seed migration created (0006_seed_profiles.sql)
- [x] HR and Developer profile YAMLs created
- [x] Complete documentation written
- [x] Keycloak seed script updated
- [x] JWT authentication working (client_credentials grant)
- [ ] Lifecycle module wired into routes (TODO - Phase 6)
- [ ] Multi-Goose test environment designed (TODO - Phase 6)
- [ ] Agent Mesh E2E tests planned (TODO - Phase 6)
- [ ] Privacy Guard Proxy built (TODO - Phase 6)

**Status:** ✅ **Ready for Phase 6 planning!**

---

## 🚧 Next Steps for Phase 6

1. **Create Profile Loading Script** - Load all 8 profiles into database
2. **Wire Lifecycle Module** - Integrate session FSM into Controller routes
3. **Design Multi-Goose Test Environment** - Docker Goose containers
4. **Build Privacy Guard Proxy** - Intercept LLM calls for PII masking
5. **E2E Agent Mesh Testing** - Cross-agent communication (Finance ↔ Manager ↔ Legal)
6. **Admin UI** - CSV import, profile assignment, user management
7. **Full Integration Testing** - All 6 profiles, all workflows

---

**Document Version:** 1.0.0  
**Maintained By:** Goose Orchestrator Agent  
**Last Reviewed:** 2025-11-10  
**Next Review:** After Phase 6 completion
