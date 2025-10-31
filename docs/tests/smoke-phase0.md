# Smoke Tests — Phase 0

- Preflight ports: `make preflight` should report no conflicts (or you override in deploy/compose/.env.ce).
- OpenAPI lint: `make lint-openapi` runs Spectral (warn-only) and returns.
- Docs presence: ensure the following files exist and are readable:
  - docs/guides/dev-setup.md
  - docs/guides/compose-ce.md
  - docs/guides/guard-model-selection.md
  - docs/guides/object-storage.md
  - docs/guides/ports.md
  - docs/compliance/posture.md
  - docs/security/profile-bundle-signing.md
  - docs/api/linting.md
  - VERSION_PINS.md
- No compose up required in Phase 0. Compose file and healthchecks arrive in Phase 1.
