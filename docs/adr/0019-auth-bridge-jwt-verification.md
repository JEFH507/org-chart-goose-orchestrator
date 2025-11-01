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
  - Validate issuer (iss), audience (aud), exp/nbf, signature
  - /status remains public; /audit/ingest requires valid Bearer token
- Auth bridge: HTTP gateway or reverse proxy passes Authorization through; no PII headers; optional X-Forwarded-* only
- Configuration via env:
  - OIDC_ISSUER_URL, OIDC_JWKS_URL, OIDC_AUDIENCE
- Observability: log claim metadata only, redact/avoid PII (e.g., subject may be hashed/pseudonymized)

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
