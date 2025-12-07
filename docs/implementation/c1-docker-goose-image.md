# Task C.1: Docker goose Image - Implementation Summary

**Status:** ✅ COMPLETE  
**Completed:** 2025-11-10 19:35  
**Tests:** 12/12 passing  
**Workstream:** C (Multi-goose Test Environment)

---

## 🎯 Objective

Create a Docker image that runs goose with automatic configuration (no manual setup or keyring).

---

## ✅ What Was Built

### 1. Dockerfile (docker/goose/Dockerfile)

**Base Image:** ubuntu:24.04 (676MB final size)

**Installed Components:**
- goose CLI v1.13.1 (official installer)
- Python 3 with yaml, requests libraries
- System tools: curl, jq, nano, vim, netcat
- X11 libraries (required by goose)

**Key Features:**
- No keyring support (Docker limitation on Ubuntu)
- All configuration via environment variables
- Scripts embedded in image (entrypoint + config generator)

---

### 2. Entrypoint Script (docker-goose-entrypoint.sh)

**Purpose:** Auto-configure goose and start session

**Flow:**
```
1. Wait for Controller to be ready (health check)
2. Get JWT from Keycloak (client_credentials grant)
   - Uses host.docker.internal:8080
   - Adds Host: localhost:8080 header
   - Ensures JWT issuer matches Controller expectation
3. Fetch profile from Controller API (with JWT auth)
4. Generate config.yaml from profile JSON
5. Start goose session (non-interactive)
```

**Environment Variables:**
- `GOOSE_ROLE` - Which profile to fetch (finance, legal, manager, etc.)
- `CONTROLLER_URL` - Controller API endpoint
- `KEYCLOAK_CLIENT_SECRET` - OAuth2 client secret
- `OPENROUTER_API_KEY` - LLM API key
- `PRIVACY_GUARD_PROXY_URL` - Privacy proxy endpoint

---

### 3. Config Generator (generate-goose-config.py)

**Purpose:** Convert Controller profile JSON to goose config.yaml

**Input:** Profile JSON from Controller API
**Output:** config.yaml in goose format

**Key Mappings:**
```python
Profile JSON              →  config.yaml
────────────────────────────────────────
providers.api_base        →  api_base
extensions[]              →  extensions[]
privacy                   →  privacy
policies                  →  policies
role                      →  role
display_name              →  display_name
```

**Special Handling:**
- API key via env var (not keyring): `api_key_env: OPENROUTER_API_KEY`
- Privacy Guard Proxy URL override
- Extensions array preserved

**Example Output:**
```yaml
provider: openrouter
model: openai/gpt-4o-mini
api_key_env: OPENROUTER_API_KEY
api_base: http://ce_privacy_guard_proxy:8090/v1
extensions:
  - name: agent_mesh
    enabled: true
  - name: github
    enabled: true
role: finance
display_name: Finance Team Agent
privacy:
  mode: hybrid
  strictness: strict
```

---

### 4. Test Script (tests/integration/test_docker_goose_image.sh)

**12 Comprehensive Tests:**

1. ✅ Docker image exists
2. ✅ goose installation (v1.13.1)
3. ✅ Python and YAML library
4. ✅ Config generation script exists
5. ✅ JWT acquisition from Keycloak
6. ✅ JWT issuer correct (localhost:8080)
7. ✅ Profile fetch from Controller
8. ✅ Profile has valid signature
9. ✅ Config.yaml generated successfully
10. ✅ Config uses Privacy Guard Proxy
11. ✅ Config has correct role
12. ✅ No keyring dependencies

**All 12 tests passing!**

---

## 🔑 Key Technical Decisions

### 1. Host.docker.internal + Host Header Override

**Problem:** JWT issuer mismatch
- Controller expects: `iss: http://localhost:8080/realms/dev`
- Container requests from Docker network: `http://keycloak:8080`
- JWT issued with: `iss: http://keycloak:8080/realms/dev`
- Mismatch → 401 Unauthorized

**Solution:**
```bash
# Request from host.docker.internal (maps to host machine)
curl -X POST "http://host.docker.internal:8080/realms/dev/protocol/openid-connect/token" \
  -H "Host: localhost:8080" \  # Override Host header
  ...
```

**Result:**
- Keycloak sees `Host: localhost:8080`
- Issues JWT with `iss: http://localhost:8080/realms/dev`
- Controller accepts JWT ✅
- No .env.ce changes needed ✅
- All existing tests continue to work ✅

---

### 2. No Keyring Support

**Community Guidance:** Ubuntu in Docker does NOT support keyring
- Source: https://github.com/block/goose/discussions/1496
- Official tutorial confirms: keyring fails in Docker Ubuntu

**Our Solution:**
- ALL config via environment variables
- API key: `OPENROUTER_API_KEY` env var
- goose reads from env (not keyring)
- Config uses: `api_key_env: OPENROUTER_API_KEY`

---

### 3. Profile Signing Fix

**Problem:** All 8 profiles had NULL signatures

**Solution:**
Created `scripts/sign-all-profiles.sh`:
- Uses Controller's `/admin/profiles/{role}/publish` endpoint
- Vault Transit HMAC signing (sha2-256)
- Idempotent (safe to re-run)
- Permanent (signatures persist in database)

**Result:**
- All 8 profiles now signed
- Signature validation working
- Controller accepts profile requests

---

## 📊 Acceptance Criteria

✅ **All Met:**

| Criterion | Status | Notes |
|-----------|--------|-------|
| Dockerfile builds | ✅ | 676MB, ubuntu:24.04 |
| goose starts without prompts | ✅ | Non-interactive mode |
| Profile fetched from Controller | ✅ | JWT auth working |
| config.yaml uses env vars | ✅ | No keyring needed |
| No keyring errors | ✅ | All config via env |
| JWT auth working | ✅ | Issuer matches Controller |

---

## 🧪 How to Test

### Run Full Test Suite:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
./tests/integration/test_docker_goose_image.sh
```

**Expected:** 12/12 tests passing

### Manual Test (Profile Fetch):

```bash
docker run --rm \
  --network compose_default \
  --add-host=host.docker.internal:host-gateway \
  -e GOOSE_ROLE=finance \
  -e CONTROLLER_URL=http://ce_controller:8088 \
  -e KEYCLOAK_CLIENT_SECRET=<your-secret> \
  -e OPENROUTER_API_KEY=sk-or-test \
  goose-test:latest \
  bash -c '
    TOKEN=$(curl -s -X POST \
      "http://host.docker.internal:8080/realms/dev/protocol/openid-connect/token" \
      -H "Host: localhost:8080" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "grant_type=client_credentials" \
      -d "client_id=goose-controller" \
      -d "client_secret=$KEYCLOAK_CLIENT_SECRET" | jq -r .access_token)
    
    curl -s -H "Authorization: Bearer $TOKEN" \
      "$CONTROLLER_URL/profiles/$GOOSE_ROLE" | jq ".role, .display_name"
  '
```

**Expected:**
```
"finance"
"Finance Team Agent"
```

---

## 📂 Files Created

| File | Lines | Purpose |
|------|-------|---------|
| docker/goose/Dockerfile | 67 | Docker image definition |
| docker/goose/docker-goose-entrypoint.sh | 113 | Auto-configuration script |
| docker/goose/generate-goose-config.py | 115 | Profile → config converter |
| tests/integration/test_docker_goose_image.sh | 200 | Validation test suite |
| scripts/sign-all-profiles.sh | 105 | Profile signing tool |
| docs/implementation/c1-docker-goose-image.md | (this file) | Implementation docs |

**Total:** ~600 lines of code + documentation

---

## 🚀 Next Steps

**Task C.2: Docker Compose Configuration**
- Add 3 goose services to ce.dev.yml (finance, manager, legal)
- Configure volumes for workspaces
- Configure extra_hosts for each service
- Test multi-goose startup

---

## 🎓 Lessons Learned

### 1. JWT Issuer Matching is Critical
- Keycloak uses the HTTP Host header to determine JWT issuer
- Controller validates JWT issuer matches OIDC_ISSUER_URL
- Solution: Override Host header to ensure match

### 2. Docker Networking Nuances
- Service names (keycloak) vs container names (ce_keycloak) are different
- host.docker.internal resolves to Docker host machine
- extra_hosts required for localhost mapping

### 3. Profile Signatures Must Be Managed
- Profiles seeded without signatures fail validation
- Signing must be done after database seeding
- Controller's /admin/profiles/{role}/publish endpoint handles this

### 4. No Keyring in Docker Ubuntu
- Keyring libraries don't work properly in Ubuntu containers
- Environment variables are the proper solution
- goose supports this via api_key_env configuration

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-10 19:35  
**Status:** Task C.1 Complete ✅
