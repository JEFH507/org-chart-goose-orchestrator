# Smoke Tests — Phase 1.2 (JWT-protected ingest)

Prereqs
- Keycloak (dev realm) is up; seed script created client `goose-controller` and a test user.
- Controller is running (compose profile or local): /status must work without auth.
- Vault dev is up (optional): PSEUDO_SALT loaded in env.

Environment (example .env or shell)
```
OIDC_ISSUER_URL=http://keycloak:8080/realms/dev
OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs
OIDC_AUDIENCE=goose-controller
CONTROLLER_URL=http://controller:8088
```

1) Get a dev token from Keycloak (password grant; dev-only)
```
TOKEN=$(curl -s -X POST \
  -d "grant_type=password" \
  -d "client_id=goose-controller" \
  -d "username=testuser" \
  -d "password=testpassword" \
  http://keycloak:8080/realms/dev/protocol/openid-connect/token | jq -r .access_token)

echo "$TOKEN" | cut -c1-32; echo "..."
```

2) Public status endpoint
```
curl -s ${CONTROLLER_URL}/status
```
Expected: JSON with status ok and version.

3) Protected ingest — success
```
curl -i -X POST ${CONTROLLER_URL}/audit/ingest \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"event":"demo","who":"alice@example.com"}'
```
Expected: 202 Accepted. Controller logs show metadata only; subject may be hashed if PSEUDO_SALT is set.

4) Protected ingest — no token
```
curl -i -X POST ${CONTROLLER_URL}/audit/ingest \
  -H "Content-Type: application/json" \
  -d '{"event":"demo"}'
```
Expected: 401/403.

5) Protected ingest — wrong audience (optional test)
- Create/modify a client with a different audience or set OIDC_AUDIENCE mismatch in controller env.
- Repeat step 3; expect 401/403.

Notes
- JWT validation: iss, aud, exp, nbf (small clock skew) and RS256 signature via JWKS.
- Reverse proxy (optional) should pass the Authorization header unchanged.
- These smokes are local/manual by design to keep CI stable.
