# Complete System Startup Guide

**Version:** 1.0.0  
**Last Updated:** 2025-11-10  
**Author:** goose Orchestrator Agent

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Service Architecture](#service-architecture)
4. [Startup Sequence](#startup-sequence)
5. [Database Migrations](#database-migrations)
6. [Verification Steps](#verification-steps)
7. [Common Issues](#common-issues)
8. [Shutdown Procedure](#shutdown-procedure)

---

## Overview

This guide provides the **complete, step-by-step process** to start all services from a fresh state (e.g., after computer restart). Follow this guide to ensure proper initialization of all components.

**Total Startup Time:** ~5.5-7.5 minutes (includes health checks)

**Services Started:**
1. **Postgres** (Database) - Port 5432
2. **Keycloak** (OIDC/JWT Auth) - Port 8080
3. **Vault** (Secrets Management) - Ports 8200 (HTTPS), 8201 (HTTP), 8202 (Cluster)
4. **Ollama** (LLM Model Server) - Port 11434
5. **Privacy Guard** (PII Protection) - Port 8089
6. **Privacy Guard Proxy** (LLM Proxy with PII Protection) - Port 8090
7. **Redis** (Cache/Idempotency) - Port 6379
8. **Controller** (Main API) - Port 8088

---

## Prerequisites

### Required Software
- **Docker** 24.0+ with Docker Compose v2
- **curl** or **httpie** for testing
- **jq** for JSON parsing (optional but recommended)
- **Git** for version control

### Required Files
- `.env.ce` file in `deploy/compose/` with all secrets configured
- Vault unseal keys (3 of 5) stored in password manager
- AppRole credentials (`VAULT_ROLE_ID`, `VAULT_SECRET_ID`) in `.env.ce`

### Directory Structure
```
goose-org-twin/
├── deploy/
│   ├── compose/
│   │   ├── ce.dev.yml           # Main compose file
│   │   ├── .env.ce              # Environment config (DO NOT COMMIT)
│   │   ├── vault-init.sh        # Vault initialization script
│   │   └── guard-config/        # Privacy Guard rules
│   ├── vault/
│   │   ├── certs/               # TLS certificates
│   │   ├── config/              # Vault configuration
│   │   └── policies/            # Vault policies
│   └── migrations/
│       └── 001_create_schema.sql
├── db/
│   └── migrations/
│       └── metadata-only/
│           ├── 0002_create_profiles.sql
│           ├── 0004_create_org_users.sql
│           └── 0005_create_privacy_audit_logs.sql
├── scripts/
│   ├── vault-unseal.sh          # Interactive unseal helper
│   └── get-jwt-token.sh         # JWT acquisition
└── profiles/
    ├── finance.yaml
    ├── legal.yaml
    ├── manager.yaml
    ├── hr.yaml
    ├── developer.yaml
    ├── support.yaml
    ├── analyst.yaml
    └── marketing.yaml
```

---

## Service Architecture

### Core Services (Always Running)
- **Postgres** - Persistent data storage
- **Keycloak** - Authentication & JWT tokens
- **Vault** - Secrets management, profile signing

### Feature Services (Enabled via Profiles)
- **Ollama** - Profile: `ollama` (required for Privacy Guard)
- **Privacy Guard** - Profile: `privacy-guard` (PII masking)
- **Redis** - Profile: `redis` (idempotency, caching)
- **Controller** - Profile: `controller` (main API)

### Service Dependencies

```
Controller depends on:
  ├─ Postgres (healthy)
  └─ Vault (healthy, unsealed)

Privacy Guard depends on:
  ├─ Vault (healthy, unsealed)
  └─ Ollama (healthy, model loaded)

Vault Init depends on:
  └─ Vault (healthy, unsealed)
```

---

## Startup Sequence

### Step 1: Navigate to Compose Directory

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose
```

### Step 2: Start Core Infrastructure

**Start Postgres, Keycloak, Vault:**

```bash
docker compose -f ce.dev.yml up -d postgres keycloak vault
```

**Wait for health checks (~60 seconds):**

```bash
# Check status
docker ps --format "table {{.Names}}\t{{.Status}}"

# Expected output:
# NAMES         STATUS
# ce_keycloak   Up X seconds (healthy)
# ce_postgres   Up X seconds (healthy)
# ce_vault      Up X seconds (healthy)
```

**⚠️ Data Persistence Notes:**

- **Postgres:** Data persisted in `postgres_data` Docker volume (survives restarts)
- **Keycloak:** Realm and client configurations persisted in `keycloak_data` Docker volume (survives restarts)
- **Vault:** Raft storage and audit logs persisted in `vault_raft` and `vault_logs` volumes

**First-Time Setup:** On first run, Keycloak needs to be seeded with realm and client configuration.

**Seed Keycloak (First Time Only):**

```bash
# Run Keycloak seed script
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/dev/keycloak_seed_complete.sh
```

This script:
1. Creates `dev` realm
2. Creates `goose-controller` client
3. Configures client credentials grant
4. Sets up direct access grants (password grant)
5. Configures JWT token settings

**Verify Keycloak setup:**

```bash
# Get JWT token using password grant
./scripts/get-jwt-token.sh
# Should return a valid JWT token
```

**⏱️ Time:** ~60 seconds (+ 30 seconds for Keycloak seed on first run)

---

### Step 3: Unseal Vault

**⚠️ CRITICAL: Vault starts SEALED for security**

Vault requires **3 of 5 unseal keys** to unlock. Keys are stored in your password manager.

**Option A: Interactive Script (Recommended)**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./scripts/vault-unseal.sh
```

Follow prompts:
1. Paste unseal key 1, press Enter
2. Paste unseal key 2, press Enter
3. Paste unseal key 3, press Enter
4. Vault unseals, shows status

**Option B: Manual Unseal**

```bash
# Unseal with key 1
docker exec -it ce_vault vault operator unseal
# Paste first key when prompted

# Unseal with key 2
docker exec -it ce_vault vault operator unseal
# Paste second key when prompted

# Unseal with key 3
docker exec -it ce_vault vault operator unseal
# Paste third key when prompted
```

**Verify unsealed:**

```bash
docker exec ce_vault vault status | grep "Sealed"
# Expected: Sealed    false
```

**⏱️ Time:** ~30 seconds (manual entry)

---

### Step 4: Initialize Vault (Transit Engine)

**Start vault-init container:**

```bash
docker compose -f ce.dev.yml up -d vault-init
```

**Check logs:**

```bash
docker logs ce_vault_init

# Expected output:
# Waiting for Vault to be ready (HTTPS enabled)...
# Vault is ready. Initializing...
# Enabling Transit engine...
# Transit engine already enabled (or successfully enabled)
# Vault initialization complete.
```

**⏱️ Time:** ~10 seconds

---

### Step 5: Create Database and Run Migrations

**Create orchestrator database:**

```bash
docker exec ce_postgres psql -U postgres -c "CREATE DATABASE orchestrator;"
```

**Run migrations (in order):**

```bash
# Navigate to project root
cd /home/papadoc/Gooseprojects/goose-org-twin

# Migration 1: Core schema (sessions, tasks, approvals, audit_events)
docker exec -i ce_postgres psql -U postgres -d orchestrator < deploy/migrations/001_create_schema.sql

# Migration 2: Profiles table
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0002_create_profiles.sql

# Migration 3: Org users table
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0004_create_org_users.sql

# Migration 4: Privacy audit logs table
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0005_create_privacy_audit_logs.sql

# Migration 5: Seed profiles (8 profiles: finance, legal, manager, hr, developer, support, analyst, marketing)
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/0006_seed_profiles.sql
```

**Verify tables created:**

```bash
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"

# Expected tables:
#  public | approvals            | table
#  public | audit_events         | table
#  public | org_imports          | table
#  public | org_users            | table
#  public | policies             | table
#  public | privacy_audit_logs   | table
#  public | profiles             | table
#  public | sessions             | table
#  public | tasks                | table
```

**⏱️ Time:** ~30 seconds

---

### Step 6: Start Ollama (LLM Model Server)

**Start Ollama with profile:**

```bash
docker compose -f ce.dev.yml --profile ollama up -d ollama
```

**Wait for health check (~20 seconds):**

```bash
docker ps | grep ollama
# Expected: ce_ollama   Up X seconds (healthy)
```

**Verify model loaded:**

```bash
docker exec ce_ollama ollama list

# Expected output:
# NAME          ID              SIZE      MODIFIED
# qwen3:0.6b    7df6b6e09427    522 MB    X days ago
```

**⏱️ Time:** ~20 seconds

---

### Step 7: Start Privacy Guard

**Start Privacy Guard with both profiles:**

```bash
docker compose -f ce.dev.yml --profile ollama --profile privacy-guard up -d privacy-guard
```

**Wait for health check (~15 seconds):**

```bash
docker ps | grep privacy
# Expected: ce_privacy_guard   Up X seconds (healthy)
```

**Verify Privacy Guard:**

```bash
curl -s http://localhost:8089/status | jq

# Expected output:
# {
#   "status": "healthy",
#   "mode": "Mask",
#   "rule_count": 22,
#   "config_loaded": true,
#   "model_enabled": true,
#   "model_name": "qwen3:0.6b"
# }
```

**⏱️ Time:** ~15 seconds

---

### Step 8: Start Privacy Guard Proxy

**Start Privacy Guard Proxy with profile:**

```bash
docker compose -f ce.dev.yml --profile privacy-guard-proxy up -d privacy-guard-proxy
```

**Wait for health check (~15 seconds):**

```bash
docker ps | grep privacy-guard-proxy
# Expected: ce_privacy_guard_proxy   Up X seconds (healthy)
```

**Verify Privacy Guard Proxy:**

```bash
curl -s http://localhost:8090/api/status | jq

# Expected output:
# {
#   "status": "healthy",
#   "mode": "auto",
#   "llm_providers": {
#     "openrouter": "configured",
#     "anthropic": "configured", 
#     "openai": "configured"
#   },
#   "privacy_guard_url": "http://privacy-guard:8089"
# }
```

**Access Control Panel UI:**

```bash
# Open in browser
xdg-open http://localhost:8090/ui

# Control Panel features:
# - Mode selection (Auto/Bypass/Strict)
# - LLM provider selection (OpenRouter/Anthropic/OpenAI)
# - Real-time status monitoring
```

**⏱️ Time:** ~15 seconds

---

### Step 9: Start Redis

**Start Redis with profile:**

```bash
docker compose -f ce.dev.yml --profile redis up -d redis
```

**Wait for health check (~10 seconds):**

```bash
docker ps | grep redis
# Expected: ce_redis   Up X seconds (healthy)
```

**Verify Redis:**

```bash
docker exec ce_redis redis-cli ping
# Expected: PONG
```

**⏱️ Time:** ~10 seconds

---

### Step 9: Start Controller (Main API)

**Start Controller with all profiles:**

```bash
docker compose -f ce.dev.yml \
  --profile controller \
  --profile redis \
  --profile ollama \
  --profile privacy-guard \
  up -d controller
```

**Wait for health check (~20 seconds):**

```bash
docker ps | grep controller
# Expected: ce_controller   Up X seconds (healthy)
```

**Verify Controller:**

```bash
curl -s http://localhost:8088/status | jq

# Expected output:
# {
#   "status": "ok",
#   "version": "0.1.0"
# }
```

**⏱️ Time:** ~20 seconds

---

### Step 10: Verify All Services

**Check all containers are healthy:**

```bash
docker ps --format "table {{.Names}}\t{{.Status}}"

# Expected output:
# NAMES                    STATUS
# ce_controller            Up X seconds (healthy)
# ce_redis                 Up X seconds (healthy)
# ce_privacy_guard_proxy   Up X seconds (healthy)
# ce_privacy_guard         Up X seconds (healthy)
# ce_ollama                Up X seconds (healthy)
# ce_keycloak              Up X minutes (healthy)
# ce_postgres              Up X minutes (healthy)
# ce_vault                 Up X minutes (healthy)
```

**All services should show `(healthy)` status!**

---

## Database Migrations

### Migration Files Location

```
deploy/migrations/
  └── 001_create_schema.sql         # Core tables (sessions, tasks, approvals, audit_events)

db/migrations/metadata-only/
  ├── 0002_create_profiles.sql              # Profiles table
  ├── 0004_create_org_users.sql             # Org users table (CSV import)
  ├── 0005_create_privacy_audit_logs.sql    # Privacy audit logs
  └── 0006_seed_profiles.sql                # Seed 8 profiles (analyst, developer, finance, hr, legal, manager, marketing, support)
```

### Migration Order (IMPORTANT)

**Always run in this order:**

1. `001_create_schema.sql` - Creates core tables
2. `0002_create_profiles.sql` - Creates profiles table
3. `0004_create_org_users.sql` - Creates org_users (references profiles)
4. `0005_create_privacy_audit_logs.sql` - Creates privacy audit logs
5. `0006_seed_profiles.sql` - **NEW:** Seeds 8 profiles with data from YAML files

**Why order matters:** 
- `org_users` has a foreign key to `profiles.role`
- `0006_seed_profiles.sql` must run after `0002_create_profiles.sql` (table must exist)

**What `0006_seed_profiles.sql` does:**
- Loads all 8 profile configurations into database
- Uses `ON CONFLICT DO UPDATE` (idempotent - safe to re-run)
- Profiles: analyst, developer, finance, hr, legal, manager, marketing, support
- Each profile includes: role, display_name, providers, extensions, policies, privacy rules, recipes

### Manual Migration Commands

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Core schema
docker exec -i ce_postgres psql -U postgres -d orchestrator \
  < deploy/migrations/001_create_schema.sql

# Profiles table
docker exec -i ce_postgres psql -U postgres -d orchestrator \
  < db/migrations/metadata-only/0002_create_profiles.sql

# Org users table
docker exec -i ce_postgres psql -U postgres -d orchestrator \
  < db/migrations/metadata-only/0004_create_org_users.sql

# Privacy audit logs
docker exec -i ce_postgres psql -U postgres -d orchestrator \
  < db/migrations/metadata-only/0005_create_privacy_audit_logs.sql

# Seed profiles (8 profiles)
docker exec -i ce_postgres psql -U postgres -d orchestrator \
  < db/migrations/metadata-only/0006_seed_profiles.sql
```

### Verify Migrations

```bash
# List all tables
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"

# Count rows in each table
docker exec ce_postgres psql -U postgres -d orchestrator -c "
  SELECT 
    schemaname, tablename, 
    (xpath('/row/cnt/text()', query_to_xml('SELECT COUNT(*) AS cnt FROM ' || schemaname || '.' || tablename, false, true, '')))[1]::text::int AS row_count
  FROM pg_tables
  WHERE schemaname = 'public'
  ORDER BY tablename;
"
```

---

## Verification Steps

### 1. Service Health Checks

```bash
# All services status
docker ps --format "table {{.Names}}\t{{.Status}}"

# Controller
curl -s http://localhost:8088/status | jq

# Privacy Guard
curl -s http://localhost:8089/status | jq

# Keycloak (should return HTML)
curl -s http://localhost:8080/health | head -5

# Vault
docker exec ce_vault vault status

# Redis
docker exec ce_redis redis-cli ping

# Postgres
docker exec ce_postgres psql -U postgres -c "SELECT version();"

# Ollama
docker exec ce_ollama ollama list
```

### 2. Database Connectivity

```bash
# Check database exists
docker exec ce_postgres psql -U postgres -c "\l orchestrator"

# Check tables exist (should show 8 tables)
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"

# Check profiles table structure
docker exec ce_postgres psql -U postgres -d orchestrator -c "\d profiles"
```

### 3. Vault Transit Engine

```bash
# Check Transit is mounted
docker exec ce_vault vault secrets list | grep transit

# Expected output:
# transit/    transit    transit_xxxx    n/a
```

### 4. Controller API Routes

```bash
# Health check
curl -s http://localhost:8088/status

# OpenAPI spec
curl -s http://localhost:8088/api-docs/openapi.json | jq '.info'
```

---

## Common Issues

### Issue 1: Vault is Sealed

**Symptoms:**
- Controller fails to start with "Vault is sealed" error
- `docker exec ce_vault vault status` shows `Sealed: true`

**Solution:**
```bash
./scripts/vault-unseal.sh
# Enter 3 unseal keys when prompted
```

**Why it happens:** Vault always starts sealed for security. You must unseal after every restart.

---

### Issue 2: Database `orchestrator` Does Not Exist

**Symptoms:**
- Controller fails with "database orchestrator does not exist"

**Solution:**
```bash
docker exec ce_postgres psql -U postgres -c "CREATE DATABASE orchestrator;"
```

Then run migrations (see Step 5).

---

### Issue 3: Privacy Guard Can't Connect to Ollama

**Symptoms:**
- Privacy Guard health check fails
- Logs show "connection refused" to ollama:11434

**Solution:**
```bash
# Start both services together with profiles
docker compose -f ce.dev.yml --profile ollama --profile privacy-guard up -d
```

**Why it happens:** Privacy Guard requires Ollama service to be running (NER model).

---

### Issue 4: Controller AppRole Token Expired

**Symptoms:**
- Controller logs show "invalid token" from Vault
- API calls return 403 Forbidden after 1 hour

**Solution:**
```bash
# Restart controller to get fresh AppRole token
docker compose -f ce.dev.yml --profile controller restart controller

# Wait 10 seconds for health check
sleep 10 && docker ps | grep controller
```

**Why it happens:** AppRole tokens expire after 1 hour (renewable but not auto-renewed in current version).

---

### Issue 5: Tables Missing After Migrations

**Symptoms:**
- `\dt` shows fewer than 8 tables

**Solution:**
```bash
# Check which migration failed
docker exec ce_postgres psql -U postgres -d orchestrator -c "\dt"

# Re-run missing migration
docker exec -i ce_postgres psql -U postgres -d orchestrator < db/migrations/metadata-only/XXXX.sql
```

**Verify order:** Migrations must run in order (001 → 0002 → 0004 → 0005).

---

### Issue 6: Port Already in Use

**Symptoms:**
- `Error: port is already allocated`

**Solution:**
```bash
# Find process using port (example: 8088)
sudo lsof -i :8088

# Kill process
sudo kill -9 <PID>

# Or stop conflicting docker container
docker stop <container_name>
```

---

## Shutdown Procedure

### Graceful Shutdown (Recommended)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose

# Stop all services with all profiles
docker compose -f ce.dev.yml \
  --profile controller \
  --profile redis \
  --profile ollama \
  --profile privacy-guard \
  down
```

### Quick Shutdown (All Containers)

```bash
# Stop and remove all containers
docker compose -f ce.dev.yml down

# Also remove orphaned containers
docker compose -f ce.dev.yml down --remove-orphans
```

### Nuclear Option (Clean Slate)

**⚠️ WARNING: Destroys all data (volumes, networks)**

```bash
# Stop and remove everything
docker compose -f ce.dev.yml down -v

# This deletes:
# - All containers
# - All volumes (postgres data, vault data, redis data, ollama models)
# - All networks
```

**Use only when:**
- Starting from scratch
- Testing fresh installation
- Data corruption recovery

---

## Quick Reference

### One-Line Full Startup (After Vault Unseal)

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/deploy/compose && \
docker compose -f ce.dev.yml up -d postgres keycloak vault && \
sleep 60 && \
echo "Now unseal Vault with ./scripts/vault-unseal.sh" && \
read -p "Press Enter after Vault is unsealed..." && \
docker compose -f ce.dev.yml up -d vault-init && \
sleep 10 && \
docker exec ce_postgres psql -U postgres -c "CREATE DATABASE orchestrator;" && \
docker exec -i ce_postgres psql -U postgres -d orchestrator < ../../deploy/migrations/001_create_schema.sql && \
docker exec -i ce_postgres psql -U postgres -d orchestrator < ../../db/migrations/metadata-only/0002_create_profiles.sql && \
docker exec -i ce_postgres psql -U postgres -d orchestrator < ../../db/migrations/metadata-only/0004_create_org_users.sql && \
docker exec -i ce_postgres psql -U postgres -d orchestrator < ../../db/migrations/metadata-only/0005_create_privacy_audit_logs.sql && \
docker compose -f ce.dev.yml --profile ollama up -d ollama && \
sleep 20 && \
docker compose -f ce.dev.yml --profile ollama --profile privacy-guard up -d privacy-guard && \
sleep 15 && \
docker compose -f ce.dev.yml --profile redis up -d redis && \
sleep 10 && \
docker compose -f ce.dev.yml --profile controller --profile redis --profile ollama --profile privacy-guard up -d controller && \
sleep 20 && \
docker ps --format "table {{.Names}}\t{{.Status}}"
```

### Service Ports Quick Reference

| Service | Port | Protocol | URL |
|---------|------|----------|-----|
| Controller | 8088 | HTTP | http://localhost:8088 |
| Privacy Guard | 8089 | HTTP | http://localhost:8089 |
| Privacy Guard Proxy | 8090 | HTTP | http://localhost:8090 |
| Privacy Guard Proxy UI | 8090 | HTTP | http://localhost:8090/ui |
| Keycloak | 8080 | HTTP | http://localhost:8080 |
| Vault (HTTPS) | 8200 | HTTPS | https://localhost:8200 |
| Vault (HTTP) | 8201 | HTTP | http://localhost:8201 |
| Vault (Cluster) | 8202 | TCP | - |
| Postgres | 5432 | TCP | postgresql://localhost:5432 |
| Redis | 6379 | TCP | redis://localhost:6379 |
| Ollama | 11434 | HTTP | http://localhost:11434 |

---

## Next Steps

After successful startup:

1. **Load Profiles:** See `/docs/operations/PROFILE-LOADING-GUIDE.md`
2. **Run Tests:** See `/docs/tests/phase6-progress.md`
3. **JWT Tokens:** Run `./scripts/get-jwt-token.sh`
4. **API Testing:** See `/docs/api/TESTING-GUIDE.md`

---

**Document Version:** 1.0.0  
**Maintained By:** goose Orchestrator Agent  
**Last Reviewed:** 2025-11-10
