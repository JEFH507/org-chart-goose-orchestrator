# Phase 0 Checklist (actionable)

- [ ] Create .github templates (PR + issues)
- [ ] Add CONTRIBUTING.md and commit-style doc
- [ ] Establish VERSION_PINS.md (Keycloak, Vault, Postgres, Ollama, S3 options)
- [ ] Add deploy/compose/ce.dev.yml with healthchecks
- [ ] Add deploy/compose/.env.ce with overridable ports
- [ ] Write docs/guides/dev-setup.md
- [ ] Write docs/guides/compose-ce.md
- [ ] Write docs/guides/keycloak-dev.md
- [ ] Write docs/security/secrets-bootstrap.md
- [ ] Add docs/api/controller/openapi.yaml (stub endpoints)
- [ ] Add docs/audit/audit-event.schema.json (stub)
- [ ] Add docs/policy/profile-bundle.schema.yaml (stub)
- [ ] Add config/profiles/sample/marketing.yaml.sig (placeholder)
- [ ] Add db/migrations/metadata-only/0001_init.sql (stub)
- [ ] Add docs/tests/smoke-phase0.md
- [ ] Verify docker compose up succeeds; health scripts pass
- [ ] Review against ADRs 0001–0016; update references as needed
- [ ] Record acceptance sign-off in CHANGELOG.md
