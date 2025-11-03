# ADR 0019: Gateway-to-Goosed Auth Bridge and JWT Verification Policy (Phase 1.2)

## Context
The MVP orchestrator is HTTP-only and metadata-only by design (see ADR-0002, ADR-0005, ADR-0012). Phase 1.2 realigns to the original Phase 1 Identity & Security scope by implementing OIDC login via Keycloak (dev), JWT minting/verification, and an auth bridge between a gateway and the controller.

Key constraints:
- Privacy-first: do not store PII; log metadata only
- OSS Community Edition defaults (Keycloak, Vault, Postgres, optional S3 off)
- Compose-first dev environment; CI must remain stable

## Decision
- Identity provider: Keycloak (dev realm) for OIDC. Tokens minted by Keycloak.
- Controller authentication: Accepts Bearer JWT; verifies via JWKS from Keycloak.
  - Validate issuer (iss), audience (aud), exp/nbf (with small clock skew allowance, e.g., 60s), and signature (RS256).
  - Cache JWKS with a reasonable TTL and handle key rotation.
  - /status remains public; /audit/ingest requires valid Bearer token.
- MVP posture for edge: No dedicated gateway container required. A simple reverse proxy (optional) may pass Authorization through; the controller performs JWT validation.
- Configuration via env (dev defaults):
  - OIDC_ISSUER_URL=http://keycloak:8080/realms/dev
  - OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
  - OIDC_AUDIENCE=goose-controller
- Observability: log claim metadata only; avoid PII. If needed, hash the subject (sub) with PSEUDO_SALT before logging.

## Technical details
- Keycloak seeding script creates realm, client, roles, and a test user; docs provide curl to fetch JWT.
- Controller adds middleware for JWT verification using JWKS; failure → 401/403.
- Compose optionally includes a gateway profile; otherwise document pass-through pattern.

## Security & privacy impact
- Strengthens auth boundaries while preserving metadata-only server model.
- No JWT signing keys stored in repo; JWKS fetched from IdP.

## Operational impact
- Adds Keycloak dependency for protected endpoints.
- Local smoke tests cover token retrieval and authorized ingest.

## Consequences
- Requires env wiring and seeding for local dev.
- CI remains minimal; full auth smoke kept out of CI for stability.

## Alternatives considered
- Embedding an internal token service: rejected; Keycloak is CE default and standard.
- Allowing anonymous ingest: rejected for Phase 1.2; only /status remains public.

## Decision lifecycle
- Status: Accepted for Phase 1.2
- Revisit: Phase 2 for finer-grained scopes/roles and gateway enforcement policies.

## References
- ADR-0002, ADR-0005, ADR-0012, ADR-0018
- docs/guides/keycloak-dev.md (to be updated)
- docs/tests/smoke-phase1.2.md (to be added)
