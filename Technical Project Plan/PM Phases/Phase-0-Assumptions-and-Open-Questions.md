# Phase 0 — Assumptions & Open Questions

## Assumptions

- Developers have Docker available and CPU-only Ollama is acceptable for Phase 0.
- OIDC IdP for CE is Keycloak, run in dev mode without TLS for local testing.
- No service implementation is required in Phase 0; only scaffolding and placeholders.
- Server posture remains metadata-only as per ADR-0005/0012; no transcripts or content in DB stubs.
- HTTP-only interop is sufficient for early demos; no message bus (ADR-0001).
- Signing for profile bundles uses Ed25519; key custody to be documented later; Phase 0 uses placeholders only.

## Open Questions

- Which minimal models should Ollama pull for guard demos later (e.g., qwen2.5:0.5b vs llama3.2:1b)? For Phase 0, list but do not pull.
- Include MinIO in Phase 0 compose by default or keep it opt-in via env?
- Preferred port mappings for shared developer environments—standardize or per-user overrides?
- Add a Makefile for standard tasks (make up/down/health) or keep pure compose/scripts?
- What signing key distribution process will be used for profile bundles in CE? Developer-generated keys or repo-stored test keys (with warnings)?
- Add Spectral or Redocly CLI to validate OpenAPI in Phase 0 CI, or defer to Phase 1?
- Any compliance wording required in docs now (SOC2/GDPR posture summaries) or wait until audit endpoints land?
