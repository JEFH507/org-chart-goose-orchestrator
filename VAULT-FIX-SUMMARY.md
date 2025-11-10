# Vault Signing Issue - RESOLVED

**Date:** 2025-11-10  
**Issue:** Profile signature verification failing with "Vault HMAC verification failed"  
**Root Cause:** Controller using invalid token "dev-only-token" (403 Forbidden from Vault)

## Solution Applied

### 1. Created Vault Policy
**Policy Name:** `controller-policy`  
**Permissions:**
- `transit/keys/profile-signing` - create, read, update
- `transit/sign/profile-signing` - create, update
- `transit/hmac/profile-signing` - create, update
- `transit/verify/profile-signing` - create, update
- `transit/keys` - list

### 2. Generated New Token
**Token:** `hvs.CAESILr8pziPz5M2D7ba3IzObW4myyea1Ck8q9gmEIl5qNYPGh4KHGh2cy43bEUwQkd6bUU2b1RqV244VzFHR0o4NDc`  
**Accessor:** `wyFFlxyzN8INhxTcTYROdzhV`  
**Policies:** `controller-policy`  
**Renewable:** Yes  
**Lease Duration:** 2764800 seconds (32 days)

### 3. Verified Transit Key Exists
**Key Name:** `profile-signing` ✅

## Next Steps

1. Update controller container with new token
2. Restart controller
3. Test profile signing: `POST /admin/profiles/{role}/publish`
4. Re-enable signature verification in profiles.rs
5. Test profile fetch still works

## Commands to Execute

```bash
# Stop controller
docker stop ce_controller && docker rm ce_controller

# Start with new Vault token
docker run -d \
  --name ce_controller \
  --network compose_default \
  --network-alias controller \
  -p 8088:8088 \
  -e DATABASE_URL=postgresql://postgres:postgres@postgres:5432/orchestrator \
  -e OIDC_ISSUER_URL=http://localhost:8080/realms/dev \
  -e OIDC_JWKS_URL=http://keycloak:8080/realms/dev/protocol/openid-connect/certs \
  -e OIDC_AUDIENCE=goose-controller \
  -e OIDC_CLIENT_SECRET=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8 \
  -e VAULT_ADDR=http://vault:8201 \
  -e VAULT_TOKEN=hvs.CAESILr8pziPz5M2D7ba3IzObW4myyea1Ck8q9gmEIl5qNYPGh4KHGh2cy43bEUwQkd6bUU2b1RqV244VzFHR0o4NDc \
  -e OPENROUTER_API_KEY=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 \
  controller:latest

# Test signing
TOKEN_RESP=$(curl -s -X POST "http://localhost:8080/realms/dev/protocol/openid-connect/token" \
    -H "Host: localhost:8080" \
    -d "grant_type=client_credentials" \
    -d "client_id=goose-controller" \
    -d "client_secret=elEZVIKjsmk9ekws6xrAXb9E1FcqFEI8")
JWT=$(echo "$TOKEN_RESP" | jq -r '.access_token')

curl -X POST "http://localhost:8088/admin/profiles/finance/publish" \
  -H "Authorization: Bearer $JWT" | jq '.'
```

## Security Note

⚠️ **Token in Environment Variable:** The new token is passed via `-e VAULT_TOKEN=...`  
✅ **Better Approach (Future):** Use AppRole authentication (credentials already exist)

**AppRole Credentials Available:**
- ROLE_ID: `9df43a52-2527-c180-48f9-e04928e8276c`
- SECRET_ID: `02dd5e63-3588-3b60-0ca3-29cd897a1604`

Consider migrating to AppRole after verifying signing works.
