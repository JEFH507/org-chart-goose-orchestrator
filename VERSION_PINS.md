# VERSION_PINS

Pin all images explicitly (no :latest). Tags may evolve later; update via PR.

## Infrastructure (Phase 0, Phase 2.5 Upgrades)
- **Keycloak**: quay.io/keycloak/keycloak:26.0.4 (upgraded 2025-11-04, fixes CVE-2024-8883 HIGH)
  - Use: OIDC/JWT authentication for Controller API
  - Realm: `dev`, Client: `goose-controller`
  - Test users: phase5test/test123, admin/admin123
  
- **Vault**: hashicorp/vault:1.18.3 (upgraded 2025-11-04, latest LTS)
  - **Status**: ✅ **Integrated** (Phase 5 - 2025-11-07)
  - **Use**: Profile integrity signing (HMAC via Transit engine), future PII encryption keys (KV v2)
  - **Dev Mode**: HTTP on port 8200, root token = "root", in-memory storage (not persistent)
  - **Transit Engine**: profile-signing key (HMAC-SHA256), auto-initialized via vault-init.sh
  - **KV v2 Engine**: Phase 6 ready for PII redaction rules
  - **Controller Integration**: `src/vault/` module (client, transit, kv - 700+ lines)
  - **Admin Endpoints**: POST /admin/profiles/{role}/publish (D9 - Vault signing)
  - **Test Coverage**: D7-D9 validated (create, update, publish/sign)
  - **Production**: Requires HTTPS, AppRole auth, persistent storage, audit device (Phase 6)
  
- **Postgres**: postgres:17.2-alpine (upgraded 2025-11-04, latest stable with 5-year LTS)
  - Database: `orchestrator`
  - Phase 5 tables: profiles, policies, org_users, org_imports, privacy_audit_logs (8 total)
  - Migrations: sqlx metadata-only (0001-0005)
  
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

## Application Services (Phase 1-5)

### Controller API (Phase 1-5)
- **Source**: `src/controller/` (Rust 1.83, Axum 0.7)
- **Image Tag**: `ghcr.io/jefh507/goose-controller:0.1.0` → **0.5.0** (Phase 5 complete)
- **Size**: 103MB (multi-stage build: rustlang/rust:nightly-bookworm → debian:bookworm-slim)
- **Endpoints**: 13 API routes
  - Phase 1-3: Auth, sessions, tasks, approvals (5 routes)
  - Phase 4: Idempotency middleware, Redis caching
  - **Phase 5 additions** (12 routes):
    - Profile endpoints: GET /profiles/{role}, /config, /goosehints, /gooseignore, /local-hints, /recipes (6 routes)
    - Admin profile: POST/PUT /admin/profiles, POST /admin/profiles/{role}/publish (3 routes)
    - Org chart: POST /admin/org/import, GET /admin/org/imports, GET /admin/org/tree (3 routes)
- **Database**: 8 tables (profiles, policies, org_users, org_imports, privacy_audit_logs, + Phase 1-4 tables)
- **Performance**: Sub-20ms P50 latency (validated H7, 250-333x faster than targets)
- **Updated**: 2025-11-06 (Phase 5 H workstream complete)

### Privacy Guard HTTP Service (Phase 2-5)
- **Source**: `src/privacy-guard/` (Rust 1.83, Axum 0.7)
- **Image Tag**: `ghcr.io/jefh507/privacy-guard:0.1.0`
- **Size**: 106MB (multi-stage build)
- **Endpoints**: 3 routes
  - Phase 2: Regex-based PII detection
  - Phase 2.2: Ollama NER integration (qwen3:0.6b)
  - Phase 5: JWT authentication, tenant isolation
- **Performance**: P50 = 10ms (validated E9, 50x faster than 500ms target)
- **Updated**: 2025-11-06 (Phase 5 H3 tests)

### Agent Mesh MCP (Phase 3)
- **Source**: `src/agent-mesh/` (Python 3.13)
- **Version**: 0.1.0 (Phase 3 baseline)
- **Runtime**: Python 3.13.9 (python:3.13-slim Docker image)
- **Dependencies**: mcp 1.20.0, requests 2.32.5, pydantic 2.12.3, python-dotenv 1.0.1
- **Tools**: send_task, request_approval, notify, fetch_status (4 MCP tools)
- **Deployment**: MCP stdio server for Goose extension loading
- **Added**: 2025-11-05 (Phase 3, Workstream B)

### Privacy Guard MCP Wrapper (Phase 5)
- **Source**: `src/privacy-guard-mcp-wrapper/` (Python 3.10+)
- **Version**: 0.1.0 (Phase 5 H6.2)
- **Dependencies**: mcp>=1.1.0, requests>=2.32.0, pydantic>=2.0.0
- **Tools**: scan_pii, mask_pii, set_privacy_mode, get_privacy_status (4 MCP tools)
- **Architecture**: Python stdio MCP server → HTTP → Privacy Guard Rust service (port 8089)
- **Purpose**: User-friendly conversational interface to Privacy Guard HTTP API
- **Status**: Code complete, pending Goose Desktop integration testing
- **Added**: 2025-11-06 (Phase 5, Workstream H)

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

## Notes
- Guard model tags: document selections in docs/guides/guard-model-selection.md; do not bundle weights in repo.
- Model selection updated 2025-11-04 (Phase 2.2): qwen3:0.6b chosen for smaller footprint, recency, and larger context vs llama3.2:1b.
- HTTP-only posture and metadata-only storage per ADR-0010/0012.
- Privacy guard image pinned at Phase 2 completion (2025-11-03), will update in Phase 2.2 with Ollama integration.
- Infrastructure and dev tool versions updated 2025-11-04 (Phase 2.5): Keycloak 26.0.4, Vault 1.18.3, Postgres 17.2, Python 3.13.9.
- Rust upgrade deferred (1.83.0 → 1.91.0 requires code changes for Clone trait bounds).
