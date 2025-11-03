# Phase 1.2 — Checklist

- [x] Initialize Phase-1.2 scaffolding and state (Nov 2)
- [x] Update Keycloak seed (realm/client/user/roles) and docs with JWT curl (Nov 3 - commit 668799e)
- [x] Implement controller JWT verification middleware (JWKS, iss/aud checks) (Nov 3 - commit b5fe058)
- [x] Document or add gateway auth bridge (compose profile optional) (Nov 3 - docs/guides/reverse-proxy-auth.md)
- [x] Validate Vault dev wiring; docs for reading/writing pseudo_salt and env export (Nov 3 - commit 46154ea)
- [x] Update compose and .env.ce samples for OIDC_* and PSEUDO_SALT (Nov 2 - prep phase)
- [x] Add docs/tests/smoke-phase1.2.md (JWT-protected ingest flow) (Nov 2 - prep phase)
- [x] Finalize ADR-0019 and ADR-0020 (Nov 2 - prep phase)
- [ ] Write Phase-1.2-Completion-Summary.md and update progress/state
- [ ] Run smoke tests and validate end-to-end flow (optional: local validation)

Note: No dedicated gateway for MVP; reverse proxy may pass Authorization; controller verifies JWT (ADR-0019).
