# ADR 0020: Vault OSS Wiring for Keys and Secrets (CE Dev) — Phase 1.2

## Context
Vault OSS is the CE default for secrets and key management. Phase 1.2 needs minimal wiring to support deterministic pseudonymization and future secure storage of sensitive config. No secrets are committed to git. Dev environment uses Vault in -dev mode for convenience.

## Decision
- Use Vault KV v2 mounted at `secret/` for development.
- Create a minimal policy for read/list of specific prefixes required by the controller in dev.
- Store only non-sensitive development salts/keys related to pseudonymization (if used in dev); production guidance deferred to later phases.
- Do not store JWT signing keys in Vault; rely on Keycloak JWKS for verification.
- Controller reads required values via environment injection (Vault integration scripted, not hard-coded in app for Phase 1.2).

## Technical details
- scripts/dev/vault_dev_bootstrap.sh initializes dev server, mounts KV v2, and creates a read/list policy.
- docs/security/secrets-bootstrap.md provides copy/paste commands to write/read example values (e.g., `secret/goosed/pseudo_salt`).
- Controller can accept PSEUDO_SALT env var (optional) to hash or pseudonymize subject metadata before logging.

## Security & privacy impact
- Maintains metadata-only logging; supports pseudonymization without persisting sensitive data.
- Clear boundary: IdP manages JWTs; Vault manages internal salts or keys (dev only in this phase).

## Operational impact
- Adds a repeatable dev bootstrap for Vault with idempotent script.
- No runtime dependency on Vault for core Phase 1.2 auth path (JWT verification uses JWKS).

## Consequences
- Aids future expansion to policy-driven secrets and key rotation in later phases.

## Alternatives considered
- Embedding secrets in .env: rejected; Vault is preferred even in dev for repeatability and hygiene.

## Decision lifecycle
- Status: Accepted for Phase 1.2
- Revisit: Phase 2+ for integrating dynamic secrets, transit engine, or PKI if needed.

## References
- docs/security/secrets-bootstrap.md
- scripts/dev/vault_dev_bootstrap.sh
- ADR-0003 (Secrets and Key Management)
