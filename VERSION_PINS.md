# VERSION_PINS

**Last Updated:** 2025-11-17  
**Phase:** 6 (95% Complete)  
**Policy:** Pin all images explicitly (no :latest). Tags may evolve later; update via PR.

**Version Strategy:** Conservative upgrades - skip alpha/RC/breaking changes until stable releases available.

## Infrastructure (Phase 0, Phase 2.5 Upgrades)
- **Keycloak**: quay.io/keycloak/keycloak:26.0.4 (upgraded 2025-11-04, fixes CVE-2024-8883 HIGH)
  - Use: OIDC/JWT authentication for Controller API
  - Realm: `dev`, Client: `goose-controller`
  - Test users: phase5test/test123, admin/admin123
  
- **Vault**: hashicorp/vault:1.18.3 (upgraded 2025-11-04, latest LTS)
  - **Status**: ✅ **Fully Integrated** (Phase 5-6 - 2025-11-11)
  - **Use**: Profile integrity signing (HMAC via Transit engine), PII encryption keys (KV v2)
  - **Deployment Mode**: Dual listener architecture
    - HTTPS: port 8200 (external access, self-signed cert)
    - HTTP: port 8201 (internal Docker network, vaultrs client compatibility)
  - **Storage**: Raft backend (persistent, vault_raft volume)
  - **Authentication**:
    - Dev: Token-based (32-day TTL, VAULT_TOKEN env var)
    - Production: AppRole (controller-policy.hcl, 1-hour TTL recommended)
  - **Engines**:
    - Transit: profile-signing key (HMAC-SHA256), auto-initialized via vault-init.sh
    - KV v2: Ready for PII redaction rules (Phase 6+)
  - **Controller Integration**: `src/vault/` module (1,314 lines - client, transit, kv, verify)
  - **Admin Endpoints**: POST /admin/profiles/{role}/publish (Phase 6 D9 - Vault signing)
  - **Seal**: Shamir 3-of-5 keys (manual unseal via scripts/vault-unseal.sh)
  - **Audit**: File-based logging enabled (vault_logs volume, 10MB rotation)
  - **Test Coverage**: Phase 5 D7-D9 validated (create, update, publish/sign)
  - **Phase 6 Updates**: All 8 profiles signed and verified, 32-day token fix applied
  - **Production Requirements**: Cloud KMS auto-unseal, AppRole 1hr TTL, TLS all endpoints
  
- **Postgres**: postgres:17.2-alpine (upgraded 2025-11-04, latest stable with 5-year LTS)
  - **Database**: `orchestrator`
  - **Tables**: 10 total (Phase 1-6)
    - Phase 1: sessions_meta, tasks_meta, approvals_meta, audit_index
    - Phase 5: profiles, policies, org_users, org_imports, privacy_audit_logs
    - Phase 6: tasks (Agent Mesh persistence)
  - **Migrations**: sqlx metadata-only (0001-0009)
    - 0001: Base metadata tables
    - 0002: Profiles table with JSONB
    - 0003: Policies table (RBAC/ABAC)
    - 0004: Org users table
    - 0005: Privacy audit logs
    - 0006: Seed 8 profiles (finance, manager, legal, hr, analyst, developer, marketing, support)
    - 0007: Session lifecycle FSM columns
    - 0008: Tasks table for Agent Mesh (UUID, JSONB, triggers)
    - 0009: Add assigned_profile column to org_users
  - **Indexes**: 5 (tasks routing, temporal, tracing, idempotency, profile assignment)
  - **Triggers**: Auto-update updated_at on tasks table
  - **Constraints**: Foreign keys deferred to Phase 7
  - **Data Volume**: ~500MB (50 users, 8 profiles, task history)
  
- **Redis**: redis:7.4.1-alpine (Phase 4+)
  - Use: Profile caching (5-min TTL), session storage, idempotency deduplication
  - Port: 6379
  
- **Ollama**: ollama/ollama:0.12.9 (verified latest 2025-11-04 for qwen3:0.6b support)
  - Model: qwen3:0.6b (523MB, 40K context, Nov 2024)
  - Use: Privacy Guard NER (person/org/location detection)
  - Volume: `ollama_models` mounted at `/root/.ollama` (Phase 5 H0 - model persistence fix)
  - Performance: P50 = 10ms (Privacy Guard with NER, validated E9)
  
- SeaweedFS: chrislusf/seaweedfs:3.68
- MinIO: minio/minio:RELEASE.2024-09-22T00-00-00Z
- Garage (optional): dxflrs/garage:0.9.3

## Application Services (Phase 1-6)

### Controller API (Phase 1-6)
- **Source**: `src/controller/` (Rust 1.83, Axum 0.7)
- **Image Tag**: `ghcr.io/jefh507/goose-controller:0.1.0` → **latest** (Phase 6 in progress)
- **Size**: 103MB (multi-stage build: rust:1.83.0-bookworm → debian:bookworm-slim)
- **Code Size**: 320L main.rs, 245L lib.rs, 8 modules
- **Endpoints**: 26 API routes (expanded in Phase 6)
  - Phase 1-3: Auth, sessions, tasks, approvals (5 routes)
  - Phase 4: Idempotency middleware, Redis caching
  - Phase 5: Profile management (12 routes)
    - Profile endpoints: GET /profiles/{role}, /config, /goosehints, /gooseignore, /local-hints, /recipes (6 routes)
    - Admin profile: POST/PUT /admin/profiles, POST /admin/profiles/{role}/publish (3 routes)
    - Org chart: POST /admin/org/import, GET /admin/org/imports, GET /admin/org/tree (3 routes)
  - **Phase 6: Admin Dashboard** (9 routes)
    - GET /admin (HTML dashboard)
    - GET /admin/users (user list)
    - POST /admin/users/:id/assign-profile (profile assignment)
    - GET /admin/profiles/list (profile selector)
    - GET /admin/dashboard/profiles/:profile (profile editor fetch)
    - PUT /admin/dashboard/profiles/:profile (profile editor save)
    - POST /admin/push-configs (push to goose containers)
    - GET /admin/logs (live log streaming)
- **Database**: 10 tables (migrations 0001-0009)
- **Dependencies**:
  - axum 0.7 (0.8.6 available, deferred - breaking changes)
  - tokio 1.48 (✅ current, upgraded 2025-11-05)
  - sqlx 0.8 (0.9.0-alpha available, deferred - alpha)
  - redis 0.27 (1.0.0-rc.3 available, deferred - RC)
  - vaultrs 0.7.4 (✅ current, upgraded 2025-11-05)
  - csv 1.3, json-patch 1.2 (Phase 5)
- **Performance**: Sub-20ms P50 latency (validated H7, 250-333x faster than targets)
- **Updated**: 2025-11-11 (Phase 6 Admin Dashboard complete)

### Privacy Guard HTTP Service (Phase 2-6)
- **Source**: `src/privacy-guard/` (Rust 1.83, Axum 0.7)
- **Image Tag**: `ghcr.io/jefh507/privacy-guard:0.1.0` → **0.2.0** (Phase 6 multi-instance)
- **Size**: 106MB (multi-stage build: rust:1.83.0-bookworm → debian:bookworm-slim)
- **Code Size**: 661L main.rs, 7 modules (3,929L total)
- **Endpoints**: 5 routes
  - /status - Health check
  - /guard/scan - PII detection only
  - /guard/mask - Full masking pipeline
  - /guard/reidentify - Reverse pseudonymization (JWT protected)
  - /internal/flush-session - Session cleanup
- **Detection Methods**:
  - Rules-only: Regex patterns (60+ rules) - <10ms latency
  - AI: Ollama NER (qwen3:0.6b) - ~15s latency
  - Hybrid: AI fallback to rules - <100ms typical
- **Modules**: detection, redaction, policy, audit, ollama_client, pseudonym, state
- **Dependencies**: axum 0.7, regex 1, hmac 0.12, fpe 0.6, reqwest 0.12
- **Phase 6 Deployment**: 3 independent instances
  - Finance: :8093 (rules-only, GUARD_MODEL_ENABLED=false)
  - Manager: :8094 (hybrid, GUARD_MODEL_ENABLED=true)
  - Legal: :8095 (AI-only, GUARD_MODEL_ENABLED=true)
- **Performance**: 
  - Rules-only: P50 <10ms ✅
  - Hybrid: P50 <100ms (can spike to 15s on NER)
  - AI-only: P50 ~15s (thorough legal compliance)
- **Known Limitations**:
  - JWT validation basic (TODO: RS256/JWKS - Phase 7)
  - OTLP trace ID placeholder (TODO: W3C header extraction - Phase 7)
- **Updated**: 2025-11-11 (Phase 6 multi-instance deployment)

### Privacy Guard Proxy (Phase 2-6)
- **Source**: `src/privacy-guard-proxy/` (Rust 1.83, Axum 0.7)
- **Image Tag**: `ghcr.io/jefh507/privacy-guard-proxy:0.2.0` → **0.3.0** (Phase 6 multi-instance)
- **Size**: ~95MB (multi-stage build: rust:1.83.0-bookworm → debian:bookworm-slim)
- **Code Size**: 92L main.rs, 6 modules (1,551L total)
- **Endpoints**: 4 routes
  - /api/status - Health check
  - /api/control/detection-method - Set detection mode
  - /api/control/privacy-mode - Set privacy mode
  - /* - Proxy all LLM provider requests
- **Features**:
  - Request/response interception and masking
  - Dynamic masking/unmasking with session tracking
  - Multi-provider support (OpenRouter, Ollama, Claude, OpenAI)
  - Control panel API for runtime configuration
- **Modules**: masking, provider, control_panel, content, proxy, state
- **Dependencies**: axum 0.7, tokio 1.35, tower 0.4, reqwest 0.11
- **Phase 6 Deployment**: 3 independent instances
  - Finance: :8096 → privacy-guard-finance:8089 (DEFAULT_DETECTION_METHOD=rules)
  - Manager: :8097 → privacy-guard-manager:8089 (DEFAULT_DETECTION_METHOD=hybrid)
  - Legal: :8098 → privacy-guard-legal:8089 (DEFAULT_DETECTION_METHOD=ai)
- **Updated**: 2025-11-11 (Phase 6 multi-instance deployment)

### Agent Mesh MCP (Phase 3, 6)
- **Source**: `src/agent-mesh/` (Python 3.13)
- **Version**: 0.1.0 (Phase 3 baseline)
- **Runtime**: Python 3.13.9 (python:3.13-slim Docker image)
- **Code Size**: 85L server, 4 tool modules (3,283L total with tests)
- **Dependencies**: mcp 1.20.0, requests 2.32.5, pydantic 2.12.3, python-dotenv 1.0.1
- **Tools**: 4 MCP tools (all working when Vault properly configured)
  - send_task - Route task to another agent (POST /tasks/route)
  - notify - Send notification to agent (POST /tasks/route with type=notification)
  - request_approval - Request approval from manager (POST /tasks/route with type=approval)
  - fetch_status - Check task status (GET /tasks?target={role})
- **Deployment**: MCP stdio server for goose extension loading
- **Integration**: Embedded in goose Docker containers at /opt/agent-mesh
- **Phase 6 Updates**: Task persistence (migration 0008), fetch_status functional
- **Test Coverage**: 22 functions, 81 test classes
- **Known Issues**: "Transport closed" in goose CLI containers (Vault unsealing 95%, goose stdio bug 5%)
- **Workaround**: Use goose Desktop (100% success rate) or API calls
- **Added**: 2025-11-05 (Phase 3, Workstream B)
- **Updated**: 2025-11-11 (Phase 6 task persistence)

### Privacy Guard MCP Wrapper (Phase 5)
- **Source**: `src/privacy-guard-mcp-wrapper/` (Python 3.10+)
- **Version**: 0.1.0 (Phase 5 H6.2)
- **Dependencies**: mcp>=1.1.0, requests>=2.32.0, pydantic>=2.0.0
- **Tools**: scan_pii, mask_pii, set_privacy_mode, get_privacy_status (4 MCP tools)
- **Architecture**: Python stdio MCP server → HTTP → Privacy Guard Rust service (port 8089)
- **Purpose**: User-friendly conversational interface to Privacy Guard HTTP API
- **Status**: Code complete, pending goose Desktop integration testing
- **Added**: 2025-11-06 (Phase 5, Workstream H)

### goose Container (Phase 6)
- **Source**: `docker/goose/` (Ubuntu 24.04 base)
- **Image Tag**: `goose-test:0.5.3` (local build)
- **Base Image**: ubuntu:24.04 (523MB final size)
- **goose CLI**: v1.13.1 (installed via official download_cli.sh script)
- **Entrypoint**: docker-goose-entrypoint.sh (6.3KB, profile fetch + config generation)
- **Config Generator**: generate-goose-config.py (Python 3, YAML generation)
- **System Dependencies**:
  - curl, ca-certificates, jq, nano, vim
  - libxcb1 (X11 libraries required by goose)
  - netcat-openbsd (networking)
  - python3, python3-pip, python3-yaml, python3-requests
- **Embedded Extensions**:
  - Agent Mesh MCP at /opt/agent-mesh (Python 3.13)
  - Dependencies: mcp, requests, pydantic, python-dotenv (installed with --break-system-packages)
- **Environment Variables**:
  - GOOSE_ROLE (finance|manager|legal)
  - CONTROLLER_URL (http://controller:8088)
  - KEYCLOAK_URL (http://host.docker.internal:8080)
  - PRIVACY_GUARD_PROXY_URL (http://privacy-guard-proxy-{role}:8090)
  - OPENROUTER_API_KEY (LLM provider access)
  - GOOSE_PROVIDER (default: openrouter)
  - GOOSE_MODEL (default: anthropic/claude-3.5-sonnet)
- **Phase 6 Deployment**: 3 independent containers
  - Finance: ce_goose_finance (volume: goose_finance_workspace)
  - Manager: ce_goose_manager (volume: goose_manager_workspace)
  - Legal: ce_goose_legal (volume: goose_legal_workspace)
- **Network**: extra_hosts with host.docker.internal (JWT issuer matching)
- **Startup Flow**:
  1. Fetch JWT token from Keycloak
  2. Fetch profile from Controller API
  3. Verify Vault signature on profile
  4. Generate config.yaml with Agent Mesh extension
  5. Start goose CLI session
- **Known Issues**:
  - MCP stdio "Transport closed" bug in CLI (use Desktop workaround)
  - Image staleness requires manual rebuild (docker compose build --no-cache)
- **Updated**: 2025-11-11 (Phase 6 multi-instance deployment)

## Additional Infrastructure (Phase 6)

### PgAdmin
- **Image**: dpage/pgadmin4:8.13
- **Container**: ce_pgadmin
- **Port**: 5050
- **Use**: Database administration and query interface
- **Credentials**: admin@company.com / admin
- **Volume**: pgadmin_data (persistent)

### Ollama Per-Instance (Phase 6)
- **Image**: ollama/ollama:0.12.9
- **Deployment**: 3 independent instances for CPU isolation
  - Finance: ce_ollama_finance (:11435, volume: ollama_finance)
  - Manager: ce_ollama_manager (:11436, volume: ollama_manager)
  - Legal: ce_ollama_legal (:11437, volume: ollama_legal)
- **Model**: qwen3:0.6b (523MB per instance, ~2GB total with metadata)
- **Purpose**: Prevent AI-only (15s) requests from blocking rules-only (<10ms) requests
- **Isolation Proven**: Finance finance requests not affected by Legal legal NER delays ✅

## Development Tools (Phase 3+)

### Python Runtime - Agent Mesh MCP Server
- **Docker Image:** python:3.13-slim
- **Version:** Python 3.13.9 (released 2025-11-04)
- **EOL:** 2029-10 (5-year support)
- **Use:** Agent Mesh MCP server (Phase 3), future Python-based extensions
- **Note:** System Python 3.12.3 compatible but Docker image preferred for consistency

### Rust Toolchain - Controller API & Extensions
- **Docker Image:** rust:1.83.0-bookworm (Phase 3+)
- **Version:** rustc 1.83.0 (90b35a623 2024-11-26)
- **Release Cycle:** 6-week rolling stable releases
- **Use:** Controller API, Privacy Guard, Rust-based MCP extensions
- **Cargo Edition:** 2021 (Cargo.toml edition field)
- **Note:** Rust 1.91.0 tested but requires code changes (Clone derives on Claims/JwksResponse). Deferred upgrade to post-Phase 3.
- **Future:** rust:1.91.0-bookworm available when code updated (8 versions newer)

## Dependency Tracking (Phase 6)

### Rust Crate Versions

**Controller (`src/controller/Cargo.toml`)**
| Crate | Current | Latest Available | Status | Notes |
|-------|---------|------------------|--------|-------|
| axum | 0.7 | 0.8.6 | ⚠️ Deferred | Breaking changes risk |
| tokio | 1.48 | ✅ Current | ✅ | Upgraded 2025-11-05 |
| sqlx | 0.8 | 0.9.0-alpha.1 | ⚠️ Deferred | Alpha release |
| redis | 0.27 | 1.0.0-rc.3 | ⚠️ Deferred | Release candidate |
| vaultrs | 0.7.4 | ✅ Current | ✅ | Upgraded 2025-11-05 |
| utoipa | 4.0 | 5.4.0 | ⚠️ Deferred | Breaking changes + Swagger UI issue |
| jsonwebtoken | 9.3 | ✅ Current | ✅ | JWT RS256 validation |
| serde | 1.0 | ✅ Current | ✅ | Core serialization |
| uuid | 1.6 | ✅ Current | ✅ | Task IDs |
| csv | 1.3 | ✅ Current | ✅ | Org chart imports |
| json-patch | 1.2 | ✅ Current | ✅ | Profile partial updates |

**Privacy Guard (`src/privacy-guard/Cargo.toml`)**
| Crate | Current | Status | Notes |
|-------|---------|--------|-------|
| axum | 0.7 | ✅ | Stable |
| tokio | 1 | ✅ | Full feature set |
| regex | 1 | ✅ | PII pattern matching |
| hmac | 0.12 | ✅ | HMAC-SHA256 for pseudonyms |
| sha2 | 0.10 | ✅ | Hash functions |
| fpe | 0.6 | ✅ | Format-preserving encryption |
| reqwest | 0.12 | ✅ | HTTP client for Ollama |

**Privacy Guard Proxy (`src/privacy-guard-proxy/Cargo.toml`)**
| Crate | Current | Status | Notes |
|-------|---------|--------|-------|
| axum | 0.7 | ✅ | Stable |
| tokio | 1.35 | ✅ | Async runtime |
| tower | 0.4 | ✅ | Middleware |
| tower-http | 0.5 | ✅ | CORS, tracing |
| reqwest | 0.11 | ✅ | HTTP forwarding |

### Python Package Versions

**Agent Mesh (`src/agent-mesh/pyproject.toml`)**
| Package | Current | Constraint | Status |
|---------|---------|------------|--------|
| mcp | 1.20.0 | >=1.20.0 | ✅ Current |
| requests | 2.32.5 | >=2.32.5 | ✅ Current |
| pydantic | 2.12.3 | >=2.12.3 | ✅ Current |
| python-dotenv | 1.0.1 | >=1.0.1 | ✅ Current |

## Shared Module Code Sizes (Phase 5-6)

- **vault/** (`src/vault/`): 1,314 lines
  - client.rs (278L) - Vault client initialization
  - transit.rs (225L) - HMAC signing/verification
  - kv.rs (265L) - KV v2 engine operations
  - verify.rs (327L) - Signature verification logic
  
- **profile/** (`src/profile/`): 1,428 lines
  - schema.rs (749L) - Profile data structures
  - signer.rs (225L) - Vault Transit integration
  - validator.rs (454L) - Profile validation logic
  
- **lifecycle/** (`src/lifecycle/`): 225 lines
  - session_lifecycle.rs (225L) - FSM state machine

**Total Shared Code**: 2,967 lines (critical infrastructure)

## Notes
- **Version Strategy**: Conservative - defer alpha/RC/breaking changes until stable
- **Guard Model**: qwen3:0.6b chosen (2025-11-04) for smaller footprint, recency, 40K context vs llama3.2:1b
- **Model Storage**: Document selections in docs/guides/guard-model-selection.md; do not bundle weights in repo
- **HTTP-only Posture**: Metadata-only storage per ADR-0010/0012
- **Image Tagging**: Privacy Guard evolved 0.1.0 → 0.2.0 (Phase 6 multi-instance)
- **Infrastructure Updates** (2025-11-04, Phase 2.5): Keycloak 26.0.4 (CVE fix), Vault 1.18.3, Postgres 17.2, Python 3.13.9
- **Rust Upgrade Deferred**: 1.83.0 → 1.91.0 requires Clone trait changes on JWT types
- **Phase 6 Additions** (2025-11-11): Admin Dashboard, Task Persistence, Multi-Instance Privacy Guard
- **Database Migrations**: 9 total (0001-0009), foreign keys deferred to Phase 7
- **Production Gaps**: Vault auto-unseal, JWT/JWKS validation, TLS all endpoints (Phase 7)
