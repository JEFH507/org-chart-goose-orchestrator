# Developer Setup (Phase 0)

Prerequisites
- Docker (Engine or Desktop)
- Bash, curl, ss, awk
- Optional: Node.js (for Spectral OpenAPI lint)

Steps
1) Port preflight
   - `make preflight` (or `scripts/dev/preflight_ports.sh`)
2) Configure env overrides (optional)
   - Create a local `deploy/compose/.env.ce` and adjust ports (see docs/guides/ports.md)
3) Compose (Phase 1 will include ce.dev.yml)
   - For now, we focus on scaffolding and docs; compose file lands in Phase 1.
4) Guard model (when using Ollama)
   - See docs/guides/guard-model-selection.md for defaults and alternatives.
5) OpenAPI lint (optional in Phase 0)
   - `make lint-openapi` (warn-only)

Notes
- Models are not bundled; you will be prompted to pull with consent when needed.
- Object storage is optional and OFF by default; see docs/guides/object-storage.md.

See also: [Secrets Bootstrap](../security/secrets-bootstrap.md).
