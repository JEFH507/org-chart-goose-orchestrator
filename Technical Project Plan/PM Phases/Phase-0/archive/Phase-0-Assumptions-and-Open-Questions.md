# Phase 0 — Assumptions & Open Questions (Updated with Resolutions)

## Assumptions

- Developers have Docker available and CPU-only Ollama is acceptable for Phase 0.
- OIDC IdP for CE is Keycloak, run in dev mode without TLS for local testing.
- No service implementation is required in Phase 0; only scaffolding and placeholders.
- Server posture remains metadata-only as per ADR-0005/0012; no transcripts or content in DB stubs.
- HTTP-only interop is sufficient for early demos; no message bus (ADR-0001).
- Signing for profile bundles uses Ed25519; developer-generated keys; no private keys in repo.

## Resolutions to Prior Open Questions

- Guard model choice: allow user selection, with default `llama3.2:1b` (CPU-friendly); quality mode `llama3.2:3b`; tiny fallback `tinyllama:1.1b`; optional `phi3:3.8b` and `qwen2.5:~1.5b`. Models are not bundled; first-run pull with consent; run a simple PII smoke test on change and fall back if failing. See docs/guides/guard-model-selection.md.
- Object storage in Phase 0: OFF by default (opt-in). Provide ALv2 default option SeaweedFS; document MinIO and Garage as AGPL alternatives; Ozone noted for future scale. See docs/guides/object-storage.md.
- Ports: Standardize defaults with `.env` overrides; add preflight checks. See docs/guides/ports.md and scripts/dev/preflight_ports.sh.
- Makefile vs scripts: Provide minimal Makefile delegating to scripts; CI uses scripts directly.
- Signing keys: Developer-generated Ed25519 keys; do not commit private keys. Optional demo keypair only in docs with warnings. See docs/security/profile-bundle-signing.md.
- OpenAPI lint: Add Spectral config now with local script; CI warn-only in Phase 0; enforce in Phase 1. See docs/api/linting.md and .spectral.yaml.
- Compliance wording: Add minimal posture statement now; detailed compliance later. See docs/compliance/posture.md.

## ADRs finalized in Phase 0 scope
- ADR 0014: CE Object Storage Default and Provider Policy — accepted
- ADR 0015: Guard Model Policy and Selection — accepted
- ADR 0016: CE Profile Signing Key Management — accepted
