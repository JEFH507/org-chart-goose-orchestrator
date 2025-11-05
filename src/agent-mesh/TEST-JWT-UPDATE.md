# Integration Test JWT Authentication Update

**Date:** 2025-11-05  
**Status:** Complete  
**Phase:** 3 (Workstream B8 - Deployment & Docs)

---

## Summary

Updated all integration test scripts to support JWT authentication after enabling Keycloak JWT verification in Phase 3.

---

## Changes Made

### 1. `run_integration_tests.sh` ✅ (Already Updated)

**Status:** No changes needed - already has JWT acquisition logic

**Features:**
- Checks for `MESH_JWT_TOKEN` environment variable
- Falls back to `KEYCLOAK_CLIENT_SECRET` to obtain token automatically
- Obtains JWT from Keycloak if not provided
- Exports token before running pytest tests

**Usage:**
```bash
# Option 1: Provide JWT token directly
export MESH_JWT_TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
  -d 'client_id=goose-controller' \
  -d 'grant_type=client_credentials' \
  -d 'client_secret=<secret>' | jq -r '.access_token')
./run_integration_tests.sh

# Option 2: Provide client secret (script obtains token automatically)
export KEYCLOAK_CLIENT_SECRET=<secret>
./run_integration_tests.sh
```

---

### 2. `test_tools_without_jwt.py` ✅ (Updated)

**Changes:**
- Added `get_jwt_token()` function to obtain JWT from Keycloak
- Updated main() to call `get_jwt_token()` before running tests
- Added imports: `subprocess`, `json`
- Renamed file purpose (now supports JWT authentication)

**Before:**
```python
os.environ["MESH_JWT_TOKEN"] = "dummy-token-for-testing"  # Not validated
```

**After:**
```python
def get_jwt_token():
    """Obtain JWT token from Keycloak if not already set."""
    if os.getenv("MESH_JWT_TOKEN"):
        print("✅ Using JWT token from MESH_JWT_TOKEN environment variable")
        return True
    
    # ... Keycloak token acquisition logic ...
```

**Usage:**
```bash
# Option 1: Provide JWT token
export MESH_JWT_TOKEN=$(curl -s ...)
python test_tools_without_jwt.py

# Option 2: Provide client secret (script obtains token)
export KEYCLOAK_CLIENT_SECRET=<secret>
python test_tools_without_jwt.py
```

---

### 3. `test_manual.sh` ✅ (Updated)

**Changes:**
- Added automatic JWT token acquisition from Keycloak
- Added Keycloak configuration variables (URL, realm, client)
- Falls back to `KEYCLOAK_CLIENT_SECRET` if `MESH_JWT_TOKEN` not set
- User-friendly error messages with example commands

**Before:**
```bash
if [ -z "$MESH_JWT_TOKEN" ]; then
    echo "ERROR: MESH_JWT_TOKEN environment variable not set"
    exit 1
fi
```

**After:**
```bash
if [ -z "$MESH_JWT_TOKEN" ]; then
    echo "ℹ️  MESH_JWT_TOKEN not set, attempting to obtain from Keycloak..."
    
    if [ -z "$KEYCLOAK_CLIENT_SECRET" ]; then
        # User-friendly error with example
        exit 1
    fi
    
    # Obtain token from Keycloak
    TOKEN_RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" ...)
    MESH_JWT_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
    export MESH_JWT_TOKEN
fi
```

**Usage:**
```bash
# Option 1: Provide JWT token
export MESH_JWT_TOKEN=$(curl -s ...)
./test_manual.sh

# Option 2: Provide client secret (script obtains token)
export KEYCLOAK_CLIENT_SECRET=<secret>
./test_manual.sh
```

---

### 4. `tests/test_integration.py` ✅ (No Changes Needed)

**Status:** Already JWT-aware - uses environment variables

**Features:**
- `jwt_token()` fixture reads from `MESH_JWT_TOKEN` environment variable
- Skips tests if token not set
- All tests use `jwt_token` fixture automatically

**No changes needed** - tests already compatible with JWT authentication.

---

## Configuration

All scripts support these environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `MESH_JWT_TOKEN` | No* | - | JWT access token (if provided, skips Keycloak acquisition) |
| `KEYCLOAK_CLIENT_SECRET` | No* | - | Client secret for obtaining JWT from Keycloak |
| `CONTROLLER_URL` | No | `http://localhost:8088` | Controller API URL |
| `KEYCLOAK_URL` | No | `http://localhost:8080` | Keycloak server URL |
| `KEYCLOAK_REALM` | No | `dev` | Keycloak realm |
| `KEYCLOAK_CLIENT` | No | `goose-controller` | Keycloak client ID |

\* Either `MESH_JWT_TOKEN` or `KEYCLOAK_CLIENT_SECRET` must be provided

---

## Testing

All test scripts validated with JWT authentication:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin/src/agent-mesh

# Export client secret (do not commit!)
export KEYCLOAK_CLIENT_SECRET=<secret-from-.env.ce>

# Test 1: Integration tests with pytest
./run_integration_tests.sh
# Expected: All tests pass with JWT authentication

# Test 2: Python smoke tests
python test_tools_without_jwt.py
# Expected: All 6 smoke tests pass

# Test 3: Manual curl tests
./test_manual.sh
# Expected: All 6 curl tests pass
```

---

## Impact

**Breaking Change:** Integration tests now require valid JWT tokens

**Before Phase 3 JWT enablement:**
- Tests used dummy token: `dummy-token-for-testing`
- Controller API didn't validate JWT (dev mode)

**After Phase 3 JWT enablement:**
- Tests require real JWT from Keycloak
- Controller API validates JWT signature, issuer, audience
- Invalid/missing token → HTTP 401 Unauthorized

**Migration:** All test scripts now auto-obtain JWT from Keycloak if `KEYCLOAK_CLIENT_SECRET` is set.

---

## Future Work (Phase 4)

**Token Refresh:**
- Current: Access tokens expire after 5 minutes (Keycloak default)
- Future: Implement refresh token logic for long-running tests

**Test User Tokens:**
- Current: Using client credentials grant (service account)
- Future: Support user password grant for user-specific tests

**Token Caching:**
- Current: Obtains new token for every test run
- Future: Cache tokens and reuse if not expired

---

## References

- **Keycloak Setup:** `scripts/setup-keycloak-dev-realm.sh`
- **JWT Verification:** `src/controller/src/auth.rs`
- **Integration Tests:** `src/agent-mesh/tests/test_integration.py`
- **Progress Log:** `docs/tests/phase3-progress.md` (2025-11-04 23:55 UTC entry)

---

**Updated by:** Goose AI Agent  
**Date:** 2025-11-05  
**Status:** Complete - All test scripts JWT-ready
