# Identity/Auth Gateway — Component Plan

## Overview
Front door for OIDC SSO and JWT issuance; bridges to goosed (X-Secret-Key) for compatibility. Enforces short token TTLs and role claims injection from IdP → profiles.

KPIs
- Login success rate ≥ 99%
- Token TTL ≤ 30m, rotation within 5m grace
- Median login time ≤ 3s
- Authn errors < 0.5%

## Objectives
- Implement OIDC SSO, JWT mint/exchange, and goosed auth bridge.

## Scope
- MVP: OIDC Code Flow + PKCE, JWT (RS256) with tenant/role claims; gateway translates to goosed auth header.

## Responsibilities
- Token issuance/validation, claim normalization, session management, audit of auth events.

## WBS (Now/Next/Later; Effort)
- OIDC integration (Keycloak CE) [Now, M]
- JWT minting and verification libs [Now, S]
- Role claim mapping from IdP groups [Now, S]
- goosed bridge (X-Secret-Key or embedded middleware) [Now, M]
- Admin config (client IDs, redirect URIs) [Now, S]
- Audit hooks (login, token exchange) [Next, S]

## Timeline
- Weeks 1–2

## Milestones
- M1: Login + JWT + goosed bridge works locally; test flows documented.

## Requirements
### Functional
- /auth/login (redirect), /auth/callback, /auth/token (JWT), /auth/logout, /auth/.well-known/jwks.json
- Map IdP groups→roles; inject tenantId, role, userId into JWT.
- Bridge: set X-Secret-Key for goosed or embed JWT verifier middleware inside goosed routes.

### Non-functional
- Security: TLS, nonce/state, CSRF protection, PKCE, token TTL ≤ 30m.
- Availability: 99.5% monthly.
- Privacy: No PII persistence beyond session; logs redacted.

## API
Endpoints (HTTP, JSON)
- GET /auth/login → 302 redirect to IdP
- GET /auth/callback?code=... → 302 to app with session
- POST /auth/token → {access_token, expires_in, id_token}
- GET /.well-known/jwks.json → {keys:[{kid,kty,n,e}]}
- GET /auth/health → {status:"ok"}

Token claims (JWT)
- { sub, tenantId, userId, roles:[...], exp, iat, jti }

goosed bridge
- Option A: Gateway forwards requests adding X-Secret-Key to goosed.
- Option B: Contribute JWT verifier to goosed; deprecate X-Secret-Key.

## Integration
- With controller-api: Accept Bearer JWT on all controller endpoints.
- With goosed: Sidecar proxy adds X-Secret-Key based on validated JWT (MVP); post-MVP embed verifier.
- With directory-policy: Use its role profile endpoint to enrich claims mapping.

## Data
- No durable PII; ephemeral session cache (in-memory/Redis optional).
- JWKS keys (public), private signing key in Vault/KMS.
Retention
- Session cache TTL ≤ 30m; audit auth events retained 90 days.

## Risks
- Misaligned claims → policy bypass; mitigate with signed profiles and tests.
- Token leakage → short TTL, audience scoping, secure storage, rotate keys.
- Bridge fragility → embed verifier into goosed post-MVP.

## Test Plan
- Unit: JWT signing/verification; OIDC nonce/state validation.
- Integration: Full login flow with Keycloak; goosed route using bridge.
- E2E: Agent→controller API with Bearer JWT.
Acceptance
- Unauthorized without JWT, authorized with valid roles; rotation works.

## Runbook
- Rotate signing keys quarterly (Vault/KMS).
- Update IdP clients on domain changes.
- Monitor 4xx/5xx auth metrics; alerts on spikes.
SLOs
- 99.5% availability; p95 latency ≤ 150ms for token endpoint.

## Prompts
- N/A (no LLM prompts). Include policy text for consent banners if needed.
