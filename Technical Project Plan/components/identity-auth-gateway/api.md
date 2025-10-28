# API

## Endpoints (HTTP, JSON)
- GET /auth/login → 302 redirect to IdP
- GET /auth/callback?code=... → 302 to app with session
- POST /auth/token → {access_token, expires_in, id_token}
- GET /.well-known/jwks.json → {keys:[{kid,kty,n,e}]}
- GET /auth/health → {status:"ok"}

## Token claims (JWT)
- { sub, tenantId, userId, roles:[...], exp, iat, jti }

## goosed bridge
- Option A: Gateway forwards requests adding X-Secret-Key to goosed.
- Option B: Contribute JWT verifier to goosed; deprecate X-Secret-Key.
