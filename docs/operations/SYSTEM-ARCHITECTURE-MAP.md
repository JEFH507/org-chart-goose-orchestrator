# System Architecture Map - Where Everything Lives

**Version:** 1.0.0  
**Last Updated:** 2025-11-10  
**Purpose:** Complete reference for understanding where code, configs, and modules live

---

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Source Code Structure](#source-code-structure)
3. [Module Relationships](#module-relationships)
4. [Configuration Files](#configuration-files)
5. [Database Schema](#database-schema)
6. [Testing Structure](#testing-structure)
7. [Deployment Structure](#deployment-structure)

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User (Goose Desktop)                     │
│                       agent-mesh MCP Extension                   │
└────────────────────┬────────────────────────────────────────────┘
                     │ HTTP + JWT
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Controller (Port 8088)                      │
│  ┌───────────┐  ┌──────────┐  ┌─────────┐  ┌───────────────┐  │
│  │  Routes   │  │  Guards  │  │  Admin  │  │  Middleware   │  │
│  │  Module   │  │  Module  │  │  Module │  │  (JWT/Auth)   │  │
│  └───────────┘  └──────────┘  └─────────┘  └───────────────┘  │
│         │              │             │              │           │
│         └──────────────┴─────────────┴──────────────┘           │
│                         │                                        │
│            Uses: Lifecycle, Vault, Profile (Rust modules)       │
└────────────┬───────────────────────┬────────────────────────────┘
             │                       │
             │                       └──────────────┐
             ▼                                      ▼
┌──────────────────────────────────────┐  ┌─────────────────────┐
│   Privacy Guard Proxy (Port 8090)    │  │   HashiCorp Vault   │
│  ┌────────────────────────────────┐  │  │   (Port 8200/8201)  │
│  │ Control Panel UI               │  │  │  ┌───────────────┐  │
│  │ - Mode: Auto/Bypass/Strict     │  │  │  │ Transit Keys  │  │
│  │ - Provider: OpenRouter/        │  │  │  │ AppRole Auth  │  │
│  │   Anthropic/OpenAI             │  │  │  │ KV Secrets    │  │
│  └────────────────────────────────┘  │  │  └───────────────┘  │
│  ┌────────────────────────────────┐  │  └─────────────────────┘
│  │ LLM Proxy with PII Protection  │  │             │
│  └────────────────────────────────┘  │             │
└──────────┬───────────────────────────┘             │
           │                                          │
           ▼                                          │
┌──────────────────────┐                             │
│   Privacy Guard      │◄────────────────────────────┘
│   (Port 8089)        │  Pseudonymization
│  ┌────────────────┐  │
│  │ Regex Rules    │  │
│  │ (22 patterns)  │  │
│  └────────────────┘  │
│  ┌────────────────┐  │
│  │ NER (Ollama)   │  │
│  │ qwen3:0.6b     │◄─┐
│  └────────────────┘  │
└──────────────────────┘
             │
             └────────────┐
                          ▼
            ┌──────────────────────┐     ┌────────────────────────┐
            │   Ollama (11434)     │     │   Postgres (5432)      │
            │  ┌────────────────┐  │     │  ┌──────────────────┐  │
            │  │ qwen3:0.6b     │  │     │  │ profiles         │  │
            │  │ (522 MB)       │  │     │  │ org_users        │  │
            │  └────────────────┘  │     │  │ sessions         │  │
            └──────────────────────┘     │  │ tasks            │  │
                                          │  │ approvals        │  │
┌─────────────────────────┐              │  │ audit_events     │  │
│  Keycloak (8080)        │              │  │ privacy_audit    │  │
│  ┌───────────────────┐  │              │  └──────────────────┘  │
│  │ Realm: dev        │  │              └────────────────────────┘
│  │ Client: goose-    │  │
│  │   controller      │  │              ┌────────────────────────┐
│  │ Grant: password + │  │              │   Redis (6379)         │
│  │   client_creds    │  │              │  ┌──────────────────┐  │
│  └───────────────────┘  │              │  │ Idempotency      │  │
└─────────────────────────┘              │  │ Cache (LRU)      │  │
                                          │  └──────────────────┘  │
                                          └────────────────────────┘
```

---

## Source Code Structure

### Root Directory Layout

```
goose-org-twin/
├── src/                          # All Rust source code
│   ├── controller/               # Main API service (Axum)
│   ├── privacy-guard/            # PII masking service (Axum)
│   ├── privacy-guard-proxy/      # LLM proxy with PII protection (Axum)
│   ├── agent-mesh/               # MCP extension (Python)
│   ├── lifecycle/                # Rust module (NOT a service)
│   ├── profile/                  # Rust module (NOT a service)
│   └── vault/                    # Rust module (NOT a service)
├── deploy/                       # Deployment configs
│   ├── compose/                  # Docker Compose files
│   ├── vault/                    # Vault configs & certs
│   └── migrations/               # Database migrations
├── db/                           # Additional DB files
│   └── migrations/               # Metadata-only migrations
├── profiles/                     # Profile YAML files
├── recipes/                      # Automated task recipes
├── scripts/                      # Helper scripts
├── docs/                         # Documentation
└── tests/                        # Test files
```

---

## Module Relationships

### Controller Service Dependencies

**File:** `src/controller/src/lib.rs`

```rust
// Controller imports these modules:

#[path = "../../lifecycle/mod.rs"]
pub mod lifecycle;  // ← Module imported but NOT wired into routes yet

#[path = "../../vault/mod.rs"]
pub mod vault;      // ← Used in admin routes (profile signing)

#[path = "../../profile/mod.rs"]
pub mod profile;    // ← Used in profile routes (validation, schema)
```

**Key Insight:** These are **Rust library modules**, NOT separate services!

### Lifecycle Module (src/lifecycle/)

**Status:** ✅ Code complete, ✅ Wired into routes (Phase 6 Workstream A - 2025-11-10)

**Files:**
- `src/lifecycle/mod.rs` - Main module
- `src/lifecycle/session_lifecycle.rs` - Session lifecycle FSM implementation

**Purpose:** Manages session lifecycle (pending → active → paused → completed/failed/expired)

**✅ NOW INTEGRATED:** Wired into Controller routes in Phase 6 Workstream A:
- Route: `PUT /sessions/{id}/events` (handle FSM transitions)
- Events: activate, pause, resume, complete, fail
- Database persistence with fsm_metadata, timestamps
- Tests: 17/17 passing

### Vault Module (src/vault/)

**Status:** ✅ Code complete, ✅ Used in production

**Files:**
- `src/vault/mod.rs` - Module exports
- `src/vault/client.rs` - Vault HTTP client
- `src/vault/transit.rs` - Transit engine (sign/verify)
- `src/vault/verify.rs` - HMAC verification for profiles
- `src/vault/auth.rs` - AppRole authentication

**Used By:**
- `src/controller/src/routes/admin/profiles.rs` - Profile signing/verification
- `src/controller/src/main.rs` - AppRole token acquisition on startup

**Purpose:** 
- Sign profiles with Vault Transit HMAC
- Verify profile signatures (tamper detection)
- AppRole authentication for Controller

### Profile Module (src/profile/)

**Status:** ✅ Code complete, ✅ Used in production

**Files:**
- `src/profile/mod.rs` - Module exports
- `src/profile/schema.rs` - Profile Rust struct (deserialize YAML/JSON)
- `src/profile/validator.rs` - Profile validation logic
- `src/profile/signer.rs` - Profile signing integration

**Used By:**
- `src/controller/src/routes/profiles.rs` - GET /profiles/{role}
- `src/controller/src/routes/admin/profiles.rs` - POST /admin/profiles, PUT /admin/profiles/{role}

**Purpose:**
- Define Profile schema (role, providers, extensions, privacy, etc.)
- Validate profiles before storage
- Sign profiles with Vault

### Privacy Guard Proxy Architecture (src/privacy-guard-proxy/)

**Status:** ✅ Code complete, ✅ Tests passing (35/35 - Phase 6 Workstream B)

**Files:**
- `src/privacy-guard-proxy/src/main.rs` - Service entry point & HTTP server
- `src/privacy-guard-proxy/src/state.rs` - Shared application state & configuration
- `src/privacy-guard-proxy/src/proxy.rs` - LLM proxy logic & request routing
- `src/privacy-guard-proxy/src/masking.rs` - PII masking/unmasking integration
- `src/privacy-guard-proxy/src/provider.rs` - LLM provider configurations (OpenRouter/Anthropic/OpenAI)
- `src/privacy-guard-proxy/src/content.rs` - Content-type detection & handling
- `src/privacy-guard-proxy/src/control_panel.rs` - Control Panel API routes
- `src/privacy-guard-proxy/ui/index.html` - Control Panel UI (single-page HTML/JS)

**Purpose:**
- Transparent LLM proxy with PII protection
- 3 operation modes: Auto (detect PII), Bypass (no masking), Strict (always mask)
- 3 LLM providers: OpenRouter, Anthropic (Claude), OpenAI (GPT)
- Control Panel UI for mode/provider selection
- Content-type aware handling (JSON vs streaming)

**Modes:**
- **Auto Mode:** Scans for PII, masks only if detected
- **Bypass Mode:** No PII detection/masking (pass-through)
- **Strict Mode:** Always masks, even if no PII detected

**Flow Diagram:**
```
User → Privacy Guard Proxy (8090)
       ├─ Mode: Auto
       │  ├─ Scan for PII → Privacy Guard (8089)
       │  ├─ If PII: Mask → LLM Provider → Unmask
       │  └─ If no PII: Direct → LLM Provider
       ├─ Mode: Bypass
       │  └─ Direct → LLM Provider (no masking)
       └─ Mode: Strict
          └─ Always Mask → LLM Provider → Unmask
```

**API Endpoints:**
- `POST /v1/chat/completions` - LLM proxy endpoint (OpenAI-compatible)
- `GET /api/status` - Service health & configuration
- `POST /api/mode` - Change mode (auto/bypass/strict)
- `GET /api/providers` - List LLM providers
- `POST /api/provider` - Change LLM provider
- `GET /ui` - Control Panel UI

**Control Panel Features:**
- Real-time mode switching (Auto/Bypass/Strict)
- LLM provider selection (OpenRouter/Anthropic/OpenAI)
- Status monitoring (Privacy Guard connectivity, current mode)
- Single-page HTML UI (no external dependencies)

**Tests:**
- Unit tests: 20/20 passing (state, content detection, provider configs)
- Integration tests: 15/15 passing (proxy flow, mode switching, content-type handling)
- Coverage: Auto mode, Bypass mode, Strict mode, JSON/streaming responses

**Dependencies:**
- Privacy Guard (port 8089) - Required for PII detection/masking
- LLM Provider API keys - Configured via environment variables

---

## Configuration Files

### Docker Compose Configuration

**Main File:** `deploy/compose/ce.dev.yml`

**Key Sections:**

```yaml
services:
  # Core infrastructure (always running)
  postgres:      # No profile required
  keycloak:      # No profile required
  vault:         # No profile required
  
  # Feature services (profile-gated)
  ollama:        # Profile: ["ollama"]
  privacy-guard: # Profile: ["privacy-guard"], depends on: vault, ollama
  redis:         # Profile: ["redis"]
  controller:    # Profile: ["controller"], depends on: postgres, vault
```

**Profiles Explained:**

A Docker Compose "profile" is a way to group optional services.

- **No profile** = Always starts
- **With profile** = Only starts when explicitly enabled

**Example:**

```bash
# Start only postgres, keycloak, vault (no profiles)
docker compose -f ce.dev.yml up -d

# Start ollama (requires --profile ollama)
docker compose -f ce.dev.yml --profile ollama up -d

# Start multiple profiles
docker compose -f ce.dev.yml \
  --profile ollama \
  --profile privacy-guard \
  --profile controller \
  up -d
```

---

### Environment Configuration

**File:** `deploy/compose/.env.ce`

**⚠️ CRITICAL:** This file is `.gooseignored` (never committed to git)

**Key Variables:**

```bash
# Database
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/orchestrator

# OIDC (Keycloak)
OIDC_ISSUER_URL=http://localhost:8080/realms/dev
OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
OIDC_AUDIENCE=goose-controller
OIDC_CLIENT_SECRET=<YOUR_KEYCLOAK_CLIENT_SECRET>  # SECRET! Get from Keycloak admin console

# Vault AppRole (Controller auth)
VAULT_ROLE_ID=<YOUR_VAULT_ROLE_ID>     # Get from: vault read auth/approle/role/orchestrator-controller/role-id
VAULT_SECRET_ID=<YOUR_VAULT_SECRET_ID>   # Get from: vault write -f auth/approle/role/orchestrator-controller/secret-id (expires 1hr)
VAULT_SKIP_VERIFY=true  # Dev only (self-signed cert)

# Privacy Guard
GUARD_MODEL_ENABLED=true
OLLAMA_MODEL=qwen3:0.6b
PSEUDO_SALT=<YOUR_PSEUDO_SALT>  # Pseudonymization salt - generate random string

# Redis
REDIS_URL=redis://redis:6379
IDEMPOTENCY_ENABLED=true
```

**How Variables Are Used:**

1. Docker Compose reads `.env.ce` (via symlink `.env → .env.ce`)
2. Variables are passed to containers as environment variables
3. Services read them at runtime (e.g., `std::env::var("DATABASE_URL")`)

---

### Vault Configuration

**Directory:** `deploy/vault/`

**Files:**

```
deploy/vault/
├── certs/
│   ├── vault.crt       # Self-signed TLS certificate
│   └── vault.key       # Private key for TLS
├── config/
│   └── vault.hcl       # Vault server configuration
└── policies/
    └── (future policies)
```

**Vault Configuration:** `deploy/vault/config/vault.hcl`

```hcl
# Dual listener setup (Phase 6)
listener "tcp" {
  address     = "0.0.0.0:8200"  # HTTPS (external)
  tls_cert_file = "/vault/certs/vault.crt"
  tls_key_file  = "/vault/certs/vault.key"
}

listener "tcp" {
  address     = "0.0.0.0:8201"  # HTTP (internal Docker)
  tls_disable = true
}

# Raft storage (production-ready HA)
storage "raft" {
  path = "/vault/raft"
  node_id = "vault-1"
}

# Enable audit logging
audit {
  type = "file"
  path = "/vault/logs/audit.log"
}
```

**Why Two Listeners?**

- **Port 8200 (HTTPS):** External access, CLI tools, secure communication
- **Port 8201 (HTTP):** Internal Docker network (vaultrs Rust library compatibility)

---

## Database Schema

### Tables Overview

**Total Tables:** 8

```
orchestrator database
├── profiles             # Profile configurations (JSONB data column)
├── org_users            # Org chart users (references profiles.role)
├── org_imports          # Org chart import history
├── sessions             # User sessions (task execution context)
├── tasks                # Tasks routed between agents
├── approvals            # Approval workflows
├── audit_events         # Audit trail for all operations
└── privacy_audit_logs   # Privacy Guard PII detection logs
```

### Migration Files

**Location:** `deploy/migrations/` and `db/migrations/metadata-only/`

```
deploy/migrations/
└── 001_create_schema.sql  # Creates: sessions, tasks, approvals, audit_events

db/migrations/metadata-only/
├── 0002_create_profiles.sql           # Creates: profiles
├── 0004_create_org_users.sql          # Creates: org_users, org_imports
└── 0005_create_privacy_audit_logs.sql # Creates: privacy_audit_logs
```

**Why "metadata-only"?**

These migrations only create table schemas (metadata), not data.  
Profile data is loaded via Controller API (`POST /admin/profiles`).

### Profiles Table Structure

```sql
CREATE TABLE profiles (
  role         TEXT PRIMARY KEY,              -- Unique role identifier
  data         JSONB NOT NULL,                -- Full profile as JSON
  created_at   TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  updated_at   TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  display_name VARCHAR(100) NOT NULL,
  signature    TEXT                           -- Vault HMAC signature
);
```

**Key Fields:**

- `role`: Primary key (e.g., "finance", "legal", "manager")
- `data`: Full profile stored as JSONB (searchable, indexable)
- `signature`: Vault Transit HMAC for tamper protection

**Example `data` JSONB:**

```json
{
  "role": "finance",
  "display_name": "Finance Team Agent",
  "providers": { ... },
  "extensions": [ ... ],
  "policies": [ ... ],
  "privacy": { ... },
  "recipes": [ ... ],
  "signature": {
    "algorithm": "sha2-256",
    "signature": "vault:v1:ABC123...",
    "signed_at": "2025-11-09T23:04:16Z",
    "signed_by": "admin@example.com"
  }
}
```

---

## Testing Structure

### Test Files Location

```
docs/tests/
├── phase3-progress.md         # Agent Mesh testing (Layer 1-3)
├── phase5-test-results.md     # Vault, Privacy Guard tests
├── phase5-progress.md         # Phase 5 workstream completion
├── phase6-progress.md         # Phase 6 ongoing tests
└── smoke-phase3.md            # Smoke test procedures

scripts/
├── get-jwt-token.sh           # JWT acquisition for testing
├── vault-unseal.sh            # Vault unseal helper
└── test-*.sh                  # Various test scripts
```

### Test Scripts

**Phase 5 Tests:**

```bash
scripts/
├── test-finance-pii-jwt.sh    # Privacy Guard + Finance profile integration
├── test-vault-production.sh   # Vault Transit, AppRole, tamper detection
└── test-legal-local.sh        # Legal profile local testing (skipped - no profile in DB)
```

**Running Tests:**

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Get JWT token first
JWT=$(./scripts/get-jwt-token.sh)

# Run Finance PII test
./scripts/test-finance-pii-jwt.sh

# Run Vault production test
./scripts/test-vault-production.sh
```

**Test Results Documentation:**

All test results are logged in `docs/tests/phase*-progress.md` files.

---

## Deployment Structure

### Docker Images

**Built Images:**

```
ghcr.io/jefh507/goose-controller:0.1.0     # Controller API (Rust/Axum)
ghcr.io/jefh507/privacy-guard:0.1.0        # Privacy Guard (Rust/Axum)
```

**Official Images:**

```
postgres:17.2-alpine                        # Database
quay.io/keycloak/keycloak:26.0.4           # Auth server
hashicorp/vault:1.18.3                     # Secrets management
ollama/ollama:0.12.9                       # LLM model server
redis:7.4.1-alpine                         # Cache/idempotency
```

### Build Context

**Controller Dockerfile:** `src/controller/Dockerfile`

```dockerfile
# Build context: /home/papadoc/Gooseprojects/goose-org-twin
# Can access: src/controller, src/vault, src/profile, src/lifecycle
```

**Privacy Guard Dockerfile:** `src/privacy-guard/Dockerfile`

```dockerfile
# Build context: src/privacy-guard
# Standalone service, no module dependencies
```

### Volume Mounts

**Persistent Data:**

```yaml
volumes:
  postgres_data:   # Postgres database cluster (profiles, sessions, etc.) ✅ NEW
  keycloak_data:   # Keycloak realm/client configuration ✅ NEW
  vault_raft:      # Vault Raft storage (encrypted)
  vault_logs:      # Vault audit logs
  ollama_models:   # Ollama model cache (522 MB for qwen3:0.6b)
  redis_data:      # Redis persistence (AOF + RDB)
```

**✅ Production-Ready:** All data now persists across container restarts!

**Config Mounts (Read-Only):**

```yaml
vault:
  volumes:
    - ../vault/certs:/vault/certs:ro         # TLS certificates
    - ../vault/config:/vault/config:ro       # vault.hcl
    - ../vault/policies:/vault/policies:ro   # Vault policies

privacy-guard:
  volumes:
    - ../../deploy/compose/guard-config:/etc/guard-config:ro  # Privacy rules
```

---

## Key File Reference

### Configuration Files

| File | Purpose | Committed to Git? |
|------|---------|-------------------|
| `deploy/compose/ce.dev.yml` | Main docker-compose file | ✅ Yes |
| `deploy/compose/.env.ce` | Secrets & environment | ❌ No (.gooseignored) |
| `deploy/compose/.env.ce.example` | Template for .env.ce | ✅ Yes |
| `deploy/vault/config/vault.hcl` | Vault server config | ✅ Yes |
| `deploy/vault/certs/vault.crt` | Self-signed TLS cert | ✅ Yes (dev only) |

### Source Code Files

| File | Purpose | Type |
|------|---------|------|
| `src/controller/src/main.rs` | Controller service entry point | Service (Rust) |
| `src/controller/src/lib.rs` | Module imports (lifecycle, vault, profile) | Library |
| `src/controller/src/routes/profiles.rs` | Profile API routes | Routes |
| `src/controller/src/routes/admin/profiles.rs` | Admin profile endpoints | Routes |
| `src/lifecycle/mod.rs` | Session lifecycle FSM | Module (NOT service) |
| `src/vault/mod.rs` | Vault integration | Module (NOT service) |
| `src/profile/mod.rs` | Profile schema & validation | Module (NOT service) |
| `src/privacy-guard/src/main.rs` | Privacy Guard service | Service (Rust) |
| `src/agent-mesh/agent_mesh_server.py` | Agent Mesh MCP server | Extension (Python) |

### Database Files

| File | Purpose | Order |
|------|---------|-------|
| `deploy/migrations/001_create_schema.sql` | Core tables | 1st |
| `db/migrations/metadata-only/0002_create_profiles.sql` | Profiles table | 2nd |
| `db/migrations/metadata-only/0004_create_org_users.sql` | Org users table | 3rd |
| `db/migrations/metadata-only/0005_create_privacy_audit_logs.sql` | Privacy logs | 4th |

### Profile Files

| File | Status | Notes |
|------|--------|-------|
| `profiles/finance.yaml` | ✅ Loaded in DB | Signed with Vault |
| `profiles/legal.yaml` | ❌ Not loaded | YAML exists, needs loading |
| `profiles/manager.yaml` | ❌ Not loaded | YAML exists, needs loading |
| `profiles/hr.yaml` | ❌ Not loaded | YAML created 2025-11-10 |
| `profiles/developer.yaml` | ❌ Not loaded | YAML created 2025-11-10 |
| `profiles/support.yaml` | ❌ Not loaded | YAML exists, needs loading |
| `profiles/analyst.yaml` | ❌ Not loaded | YAML exists, needs loading |
| `profiles/marketing.yaml` | ❌ Not loaded | YAML exists, needs loading |

---

## Service Communication Patterns

### Controller → Vault

**Protocol:** HTTP (port 8201 internal)  
**Authentication:** AppRole (VAULT_ROLE_ID + VAULT_SECRET_ID)  
**Usage:**
- Profile signing: `POST /v1/transit/sign/profile-signing`
- Profile verification: `POST /v1/transit/verify/profile-signing`

### Controller → Privacy Guard

**Protocol:** HTTP (port 8089)  
**Authentication:** None (internal Docker network)  
**Usage:**
- PII scanning: `POST /scan`
- PII masking: `POST /mask`

### Controller → Postgres

**Protocol:** PostgreSQL wire protocol (port 5432)  
**Authentication:** Username/password  
**Connection Pool:** SQLx managed

### Controller → Keycloak

**Protocol:** HTTP (port 8080)  
**Authentication:** JWT verification via JWKS  
**Usage:**
- Fetch JWKS: `GET /realms/dev/protocol/openid-connect/certs`

### Privacy Guard → Vault

**Protocol:** HTTP (port 8201 internal)  
**Usage:**
- Pseudonymization key storage (KV v2)

### Privacy Guard → Ollama

**⚠️ IMPORTANT:** Ollama and Privacy Guard are **SEPARATE containers**!

- `ce_ollama` - ollama/ollama:0.12.9 image (port 11434)
- `ce_privacy_guard` - privacy-guard:0.1.0 image (port 8089)

**Protocol:** HTTP (port 11434)  
**Usage:**
- NER model inference: `POST /api/generate`
- Privacy Guard calls Ollama via Docker network: `http://ollama:11434`

---

## Next Steps

For detailed guides on specific topics:

- **Startup Process:** `/docs/operations/STARTUP-GUIDE.md`
- **Profile Loading:** `/docs/operations/PROFILE-LOADING-GUIDE.md` (to be created)
- **Testing:** `/docs/tests/phase6-progress.md`
- **API Reference:** `/docs/api/openapi-v0.5.0.yaml`

---

**Document Version:** 1.0.0  
**Maintained By:** Goose Orchestrator Agent  
**Last Reviewed:** 2025-11-10
