# VERSION_PINS

Pin all images explicitly (no :latest). Tags may evolve later; update via PR.

## Infrastructure (Phase 0)
- Keycloak: quay.io/keycloak/keycloak:24.0.4
- Vault: hashicorp/vault:1.17.6
- Postgres: postgres:16.4-alpine
- Ollama: ollama/ollama:0.3.14
- SeaweedFS: chrislusf/seaweedfs:3.68
- MinIO: minio/minio:RELEASE.2024-09-22T00-00-00Z
- Garage (optional): dxflrs/garage:0.9.3

## Application Services (Phase 1-2)
- Controller: Built from `src/controller/` (Rust 1.83, Axum 0.7)
- Privacy Guard: Built from `src/privacy-guard/` (Rust 1.83, Axum 0.7)
  - Image tag: `ghcr.io/jefh507/privacy-guard:0.1.0`
  - Size: 90.1MB (multi-stage build: rust:1.83-bookworm → debian:bookworm-slim)
  - Phase 2 baseline (Phase 2.2 may update with Ollama integration)

## Guard Models (Ollama) - Phase 2.2+

**Default (Phase 2.2):**
- Model: `qwen3:0.6b` (Alibaba Qwen3 0.6B Instruct)
- Size: 523MB
- Context: 40K tokens
- Release: Nov 2024
- Use case: CPU-friendly, 8GB RAM systems, NER for PII detection

**Alternatives:**
- Quality: `llama3.2:3b`, `qwen3:4b`, `gemma3:4b`
- 1B options: `llama3.2:1b`, `qwen3:1.7b`, `gemma3:1b`
- Tiny fallback: `tinyllama:1.1b`

**Configuration:**
```bash
OLLAMA_MODEL=qwen3:0.6b  # Default
GUARD_MODEL_ENABLED=false  # Opt-in (backward compatible)
```

## Notes
- Guard model tags: document selections in docs/guides/guard-model-selection.md; do not bundle weights in repo.
- Model selection updated 2025-11-04 (Phase 2.2): qwen3:0.6b chosen for smaller footprint, recency, and larger context vs llama3.2:1b.
- HTTP-only posture and metadata-only storage per ADR-0010/0012.
- Privacy guard image pinned at Phase 2 completion (2025-11-03), will update in Phase 2.2 with Ollama integration.
