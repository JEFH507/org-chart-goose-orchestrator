# Phase 1.2 Updates Audit — 2025-11-02

## What Was Requested
User asked to:
1. Record decisions in ADRs and Master Technical Plan
2. Modify Phase-1.2-Agent-Prompts.md with concrete decisions
3. Add optional follow-ups:
   - Progress log entry
   - Checklist note about gateway
   - Smoke test doc
   - .env.ce.example with OIDC/PSEUDO_SALT entries

## Decisions Recorded

### User Confirmations (from conversation)
- ✓ Keycloak as dev IdP; dev-only for MVP; production out of scope
- ✓ Audience string: `goose-controller` (explained and confirmed)
- ✓ Naming: realm=`dev`, client_id=`goose-controller`, audience=`goose-controller`
- ✓ No dedicated auth gateway for MVP; reverse proxy optional; controller does JWT verification
- ✓ Vault KV v2 path: `secret/pseudonymization` with key `pseudo_salt` (explained and confirmed)
- ✓ Privacy Guard: Phase 2 = rules+NER (no local LLM yet); Phase 2.2 = add small local model
- ✓ Object storage: deferred until needed for large artifacts
- ✓ Department choice: deferred

## Completed Changes

### 1. ADR-0019 (docs/adr/0019-auth-bridge-jwt-verification.md)
- ✓ Updated Decision section:
  - Controller-side JWT verification (RS256, JWKS caching with TTL, small clock skew ~60s)
  - /status public; /audit/ingest protected
  - MVP posture: no dedicated gateway; reverse proxy optional (pass Authorization header)
  - Concrete dev env defaults:
    - OIDC_ISSUER_URL=http://keycloak:8080/realms/dev
    - OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
    - OIDC_AUDIENCE=goose-controller
  - Observability: log claim metadata only; hash sub with PSEUDO_SALT if present

### 2. ADR-0020 (docs/adr/0020-vault-oss-wiring.md)
- ✓ Updated Decision section:
  - Primary path: `secret/pseudonymization` with key `pseudo_salt`
  - Controller receives PSEUDO_SALT via env (exported from Vault in dev scripts)
  - App does not call Vault directly in Phase 1.2
- ✓ Fixed Technical details reference to match the path

### 3. Master Technical Plan (Technical Project Plan/master-technical-project-plan.md)
- ✓ Phase 1: Updated to include "controller-side JWT verification (RS256, JWKS caching; small clock skew), reverse proxy optional (no dedicated gateway for MVP)"
- ✓ Phase 2: Clarified "Local runtime (rules + regex + NER), deterministic pseudonymization keys (Vault dev: secret/pseudonymization:pseudo_salt), logs redaction; default mode = mask-and-forward."
- ✓ Added Phase 2.2: "Privacy Guard Enhancement (S) - Add a minimal local model to improve detection (kept local; no cloud exposure). Preserve the same modes (Off/Detect/Mask/Strict). Maintain mask-and-forward default."
- ✓ Environment plan: Added note that object storage is deferred until needed for large artifacts; not required for MVP flows.

### 4. Phase-1.2 Agent Prompt (Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Agent-Prompts.md)
- ✓ Purpose: Updated to say "controller-side JWT verification (no dedicated gateway for MVP)"
- ✓ OIDC env section: Added concrete dev values:
  - OIDC_ISSUER_URL=http://keycloak:8080/realms/dev
  - OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
  - OIDC_AUDIENCE=goose-controller

### 5. Progress Log (docs/tests/phase1-progress.md)
- ✓ Added timestamped entry [2025-11-02T18:25:00Z] documenting:
  - ADR-0019 and ADR-0020 updates
  - Master Plan updates (Phase 1, 2, 2.2; object storage note)
  - Phase-1.2 Agent Prompt updates
  - Next steps noted

### 6. Phase-1.2 Checklist (Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Checklist.md)
- ✓ Added note: "No dedicated gateway for MVP; reverse proxy may pass Authorization; controller verifies JWT (ADR-0019)."

### 7. Smoke Test Doc (docs/tests/smoke-phase1.2.md)
- ✓ Created complete smoke test document with:
  - Prerequisites (Keycloak seed, controller running)
  - Environment variables (OIDC_ISSUER_URL, OIDC_JWKS_URL, OIDC_AUDIENCE)
  - Step-by-step curl commands:
    1. Get dev token (password grant)
    2. Public /status endpoint
    3. Protected /audit/ingest with Bearer token (success)
    4. Protected /audit/ingest without token (failure)
    5. Optional: wrong audience test
  - Notes on JWT validation and reverse proxy behavior

### 8. Keycloak Dev Guide (docs/guides/keycloak-dev.md)
- ✓ Updated to use realm=`dev` (was `goose-dev`)
- ✓ Added curl command to get dev token
- ✓ Added notes about aud/iss expectations and link to smoke-phase1.2.md

### 9. .env.ce.example Update
- ⚠️ BLOCKED: File is in .gooseignore (`.env.*` pattern)
- Workaround documented below

## Known Issue: .env.ce.example

The file `deploy/compose/.env.ce.example` is restricted by .gooseignore pattern `.env.*`.

### Manual Action Required
Add these lines to `deploy/compose/.env.ce.example`:

```bash
# Controller
CONTROLLER_PORT=8088
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/postgres

# OIDC (Keycloak dev)
OIDC_ISSUER_URL=http://keycloak:8080/realms/dev
OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
OIDC_AUDIENCE=goose-controller

# Pseudonymization (optional for logging/IDs)
# PSEUDO_SALT is typically read from Vault then exported here for local runs
PSEUDO_SALT=CHANGE_ME_DEV_ONLY
```

### Alternative: Update .gooseignore
If you want Goose to manage .env.ce.example in future, edit `.gooseignore` to exclude it:
```
# Before:
.env.*

# After:
.env
.env.local
.env.*.local
!.env.ce.example
```

## Git Status
Modified files (ready to commit):
- Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Agent-Prompts.md
- Technical Project Plan/PM Phases/Phase-1.2/Phase-1.2-Checklist.md
- Technical Project Plan/master-technical-project-plan.md
- docs/adr/0019-auth-bridge-jwt-verification.md
- docs/adr/0020-vault-oss-wiring.md
- docs/guides/keycloak-dev.md
- docs/tests/phase1-progress.md

Untracked files:
- docs/tests/smoke-phase1.2.md (new)
- AUDIT-PHASE1.2-UPDATES.md (this file)

## Summary
✅ All requested changes completed EXCEPT .env.ce.example (blocked by .gooseignore)
✅ Trail is clean and documented
✅ ADRs, Master Plan, prompts, checklist, progress log all updated
✅ Smoke test doc created with complete curl examples
✅ Keycloak guide updated with token retrieval

## Next Steps for User
1. Manually add OIDC/PSEUDO_SALT entries to deploy/compose/.env.ce.example OR update .gooseignore
2. Review all changes with `git diff`
3. Commit these updates before proceeding to Phase 1.2 execution
