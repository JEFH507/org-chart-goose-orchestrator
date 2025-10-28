# Requirements

## Functional
- /auth/login (redirect), /auth/callback, /auth/token (JWT), /auth/logout, /auth/.well-known/jwks.json
- Map IdP groups→roles; inject tenantId, role, userId into JWT.
- Bridge: set X-Secret-Key for goosed or embed JWT verifier middleware inside goosed routes.

## Non-functional
- Security: TLS, nonce/state, CSRF protection, PKCE, token TTL ≤ 30m.
- Availability: 99.5% monthly.
- Privacy: No PII persistence beyond session; logs redacted.
