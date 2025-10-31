# Secrets Bootstrap (Phase 0 — Dev Mode)

Scope: Dev-only posture. No production secrets; do not commit tokens or keys.

## Vault (Dev Mode)
- Compose runs Vault in dev mode with a known root token (default `root`).
- On startup, Vault prints the root token to logs; treat it as ephemeral.
- Access: `http://localhost:${VAULT_PORT:-8200}`
- Health: `GET /v1/sys/health` should return 200/472 depending on seal state.
- Do not store secrets in repo. Use local env or ephemeral storage.

## Keycloak (Dev Mode)
- Admin UI: `http://localhost:${KEYCLOAK_PORT:-8080}`
- Default credentials: `admin` / `admin` (dev-only)
- Create a dev realm and clients as needed for local testing. This repo does not seed data in Phase 0.

## Next Phases
- Phase 1 will add optional seeding scripts and safer defaults.
- See ADR-0016 for key/signing policy and docs/security/profile-bundle-signing.md for signing notes.
