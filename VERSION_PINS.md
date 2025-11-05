# VERSION_PINS

Pin all images explicitly (no :latest). Tags may evolve later; update via PR.

## Infrastructure (Phase 0, Phase 2.5 Upgrades)
- Keycloak: quay.io/keycloak/keycloak:26.0.4 (upgraded 2025-11-04, fixes CVE-2024-8883 HIGH)
- Vault: hashicorp/vault:1.18.3 (upgraded 2025-11-04, latest LTS)
- Postgres: postgres:17.2-alpine (upgraded 2025-11-04, latest stable with 5-year LTS)
- Ollama: ollama/ollama:0.12.9 (verified latest 2025-11-04 for qwen3:0.6b support)
- SeaweedFS: chrislusf/seaweedfs:3.68
- MinIO: minio/minio:RELEASE.2024-09-22T00-00-00Z
- Garage (optional): dxflrs/garage:0.9.3

## Application Services (Phase 1-3)
- Controller: Built from `src/controller/` (Rust 1.83, Axum 0.7)
  - Phase 3 additions: OpenAPI with utoipa 4.2.3, 5 routes (tasks, sessions, approvals, profiles), idempotency middleware
- Privacy Guard: Built from `src/privacy-guard/` (Rust 1.83, Axum 0.7)
  - Image tag: `ghcr.io/jefh507/privacy-guard:0.1.0`
  - Size: 90.1MB (multi-stage build: rust:1.83-bookworm → debian:bookworm-slim)
  - Phase 2 baseline (Phase 2.2 may update with Ollama integration)
- **Agent Mesh MCP:** Built from `src/agent-mesh/` (Python 3.13)
  - Version: 0.1.0 (Phase 3 baseline)
  - Runtime: Python 3.13.9 (python:3.13-slim Docker image)
  - Dependencies: mcp 1.20.0, requests 2.32.5, pydantic 2.12.3, python-dotenv 1.0.1
  - Tools: send_task, request_approval, notify, fetch_status (4 MCP tools)
  - Deployment: MCP stdio server for Goose extension loading
  - Added: 2025-11-05 (Phase 3, Workstream B)

## Guard Models (Ollama) - Phase 2.2+

**Default (Phase 2.2):**
- Model: `qwen3:0.6b` (Alibaba Qwen3 0.6B Instruct)
- Size: 523MB
- Context: 40K tokens
- Release: Nov 2024
- Use case: CPU-friendly, 8GB RAM systems, NER for PII detection

**Alternatives (Post-MVP - User Selectable):**
- `gemma3:1b` (Google, Dec 2024, 600MB, 8K context)
- `phi4:3.8b-mini` (Microsoft, Dec 2024, 2.3GB, 16K context, best accuracy)
- Other: `llama3.2:3b`, `qwen3:4b` (requires more RAM)

**Configuration:**
```bash
OLLAMA_MODEL=qwen3:0.6b  # Default
GUARD_MODEL_ENABLED=false  # Opt-in (backward compatible)
```

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
