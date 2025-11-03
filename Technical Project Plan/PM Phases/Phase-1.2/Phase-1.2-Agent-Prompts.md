# Phase 1.2 — Identity & Security (M) Alignment Prompt (Execution-Oriented)

Purpose: Fill gaps from Phase 1 and deliver the original Phase 1 Identity & Security (M): OIDC login, JWT minting, controller-side JWT verification (no dedicated gateway for MVP), and Vault OSS wiring. Build strictly on Phase-0 and Phase-1 outputs, the master plan, ADRs, and product brief.

Authoritative references to USE and UPDATE:
- Master plan: Technical Project Plan/master-technical-project-plan.md
- Prior phases:
  - Phase-0: Technical Project Plan/PM Phases/Phase-0/
  - Phase-1: Technical Project Plan/PM Phases/Phase-1/
- ADRs: docs/adr/ (notably 0002, 0003, 0005, 0012, 0018, 0019, 0020)
- Product: docs/product/productdescription.md
- Goose refs: goose-versions-references/gooseV1.12.00/
- State/logs: Phase-1.2-Agent-State.json (this phase), docs/tests/phase1-progress.md

Guardrails (DO NOT VIOLATE):
- HTTP-only orchestrator; metadata-only server model (no PII/content persistence)
- No secrets in git; .env.ce samples only; Vault dev for local
- Keep CI stable; limit compose-health to Postgres; auth smoke is local/manual
- Persist state and progress per protocol (state JSON, progress log)

Git and environment inputs (defaults):
- Git remote (SSH): git@github.com:JEFH507/org-chart-goose-orchestrator.git
- Git identity: Javier / 132608441+JEFH507@users.noreply.github.com
- Default branch: main
- Controller port: 8088
- Dev DATABASE_URL: postgresql://postgres:postgres@localhost:5432/postgres
- OIDC envs to add (samples in .env.ce, not committed):
  - OIDC_ISSUER_URL=http://keycloak:8080/realms/dev
  - OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
  - OIDC_AUDIENCE=goose-controller
- Vault dev token (root for dev only), mount: secret/

Pause/Resume protocol:
- State JSON for this phase: Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Agent-State.json
- After each milestone, update state JSON and append to docs/tests/phase1-progress.md with timestamp and branch/PR
- To resume: read state JSON and last progress entry, continue from next unchecked task

Outputs to produce in this phase:
- ADRs: 0019 (Auth bridge + JWT verification), 0020 (Vault wiring) — already added, update if needed
- Controller: JWT verification middleware, /audit/ingest protected; /status public
- Keycloak seeding: realm/client/user; curl to obtain JWT
- Vault wiring: dev bootstrap validated; pseudonymization salt optional
- Compose: profiles stable; healthchecks tuned
- Docs: update guides and smoke tests (Phase 1.2 auth smoke)
- CI: unchanged scope; add docs-only checks if needed
- Phase summary + repo audit note at end

Execution steps

0) Initialize phase scaffolding
- Create folder: Technical Project Plan/PM Phases/Phase-1.2/
  - Files:
    - Phase-1.2-Agent-Prompts.md (this file)
    - Phase-1.2-Checklist.md
    - Phase-1.2-Agent-State.json
    - Phase-1.2-Open-Questions.md (optional)
- Update docs/tests/phase1-progress.md with a start entry

1) Keycloak (OIDC) seeding and token acquisition
- Update scripts/dev/keycloak_seed.sh to ensure:
  - dev realm created, client (public/confidential as needed), role(s), and test user
  - print client_id, auth endpoints; if confidential, provide how to get client_secret (not stored)
- Update docs/guides/keycloak-dev.md with curl to get token (password grant for dev), and jwt decode note
- Validate: ensure we can obtain a JWT (expiration short for dev is fine)

2) Controller — JWT verification
- Add middleware in src/controller to verify Authorization: Bearer <JWT>
  - Fetch JWKS from OIDC_JWKS_URL; validate iss, aud, exp, nbf
  - Apply to /audit/ingest only; /status remains public
  - Env vars: OIDC_ISSUER_URL, OIDC_JWKS_URL, OIDC_AUDIENCE
  - Log only metadata (e.g., sub hashed/pseudonymized if PSEUDO_SALT is present)
- Update docs/architecture/observability.md to reflect claim metadata posture

3) Gateway-to-goosed auth bridge
- If we add a gateway container (optional):
  - Define a compose profile (gateway), forward Authorization header, and set healthchecks
- If not adding a container: document the reverse proxy pattern and expectations in ADR-0019 and a short guide

4) Vault OSS wiring (dev)
- Confirm scripts/dev/vault_dev_bootstrap.sh mounts secret/ (KV v2) and writes a sample key (e.g., pseudo_salt)
- Update docs/security/secrets-bootstrap.md with explicit steps to read/write that value and export env for controller
- Ensure no secrets in git; .env.ce documents variables

5) Compose and healthchecks
- Keep ce.dev.yml healthchecks as tuned (ADR-0018)
- Add sample .env.ce variables for OIDC_ISSUER_URL, OIDC_JWKS_URL, OIDC_AUDIENCE, PSEUDO_SALT
- Local controller-only profile remains working; optional gateway profile

6) Smoke tests (Phase 1.2)
- New doc: docs/tests/smoke-phase1.2.md:
  - Pre-req: seed Keycloak, obtain JWT
  - docker exec controller: curl /status → 200
  - docker exec controller: POST /audit/ingest with Bearer token → 202
  - same endpoint without token → 401/403

7) ADRs and docs updates
- Ensure ADR-0019 and ADR-0020 are final (update if implementation specifics change)
- Cross-reference in master plan and Phase-1.2 summary

8) Closeout
- Write Phase-1.2-Completion-Summary.md (what/why/changes/errors/fixes/validation)
- Update docs/tests/phase1-progress.md and state JSON
- Optional: repo audit delta and CHANGELOG update

Checklist (mirrored in Phase-1.2-Checklist.md)
- [ ] Initialize Phase-1.2 scaffolding and state
- [ ] Keycloak seed updates + docs
- [ ] Controller JWT verification middleware
- [ ] Gateway auth bridge (container or documented pattern)
- [ ] Vault dev wiring validated + docs
- [ ] Compose/env samples updated
- [ ] Smoke tests for JWT path
- [ ] ADRs finalized (0019, 0020)
- [ ] Phase-1.2 summary + audit delta

Notes for operators
- Do not commit secrets; use Vault dev and .env.ce samples
- Keep CI lean; run JWT smokes locally
- Always update Phase-1.2-Agent-State.json after each milestone
