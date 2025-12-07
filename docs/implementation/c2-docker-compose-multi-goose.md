# Task C.2: Docker Compose Multi-goose Configuration

**Phase:** 6 - Integration & Hardening  
**Workstream:** C - Multi-goose Test Environment  
**Date:** 2025-11-10  
**Status:** ✅ Complete

## Overview

Added 3 goose service containers to the Docker Compose configuration for multi-agent testing. Each container runs a goose CLI instance with a different role profile (finance, manager, legal), demonstrating the full auto-configuration workflow via Controller API.

## Implementation

### 1. Services Added to `deploy/compose/ce.dev.yml`

#### goose-finance
```yaml
goose-finance:
  build:
    context: ../../docker/goose
    dockerfile: Dockerfile
  image: goose-test:0.1.0
  container_name: ce_goose_finance
  environment:
    - GOOSE_ROLE=finance
    - CONTROLLER_URL=http://controller:8088
    - KEYCLOAK_URL=http://host.docker.internal:8080
    - KEYCLOAK_REALM=dev
    - KEYCLOAK_CLIENT_ID=goose-controller
    - KEYCLOAK_CLIENT_SECRET=${OIDC_CLIENT_SECRET}
    - OPENROUTER_API_KEY=${OPENROUTER_API_KEY}
    - PRIVACY_GUARD_PROXY_URL=http://privacy-guard-proxy:8090
    - GOOSE_PROVIDER=${GOOSE_PROVIDER:-openrouter/anthropic/claude-3.5-sonnet}
    - GOOSE_MODEL=${GOOSE_MODEL:-anthropic/claude-3.5-sonnet}
  extra_hosts:
    - "host.docker.internal:host-gateway"
  volumes:
    - goose_finance_workspace:/workspace
  depends_on:
    controller:
      condition: service_healthy
    privacy-guard-proxy:
      condition: service_healthy
  profiles: ["multi-goose"]
```

**Similar services:** goose-manager, goose-legal with respective roles

### 2. Volumes Added

```yaml
volumes:
  goose_finance_workspace:
    driver: local
  goose_manager_workspace:
    driver: local
  goose_legal_workspace:
    driver: local
```

Each goose instance gets an isolated workspace for file operations.

### 3. Profile Dependencies

To start the multi-goose environment, ALL these profiles are required:
- `controller` - goose Controller API
- `privacy-guard` - Privacy Guard service
- `privacy-guard-proxy` - Privacy Guard Proxy (LLM request router)
- `ollama` - Ollama (required by privacy-guard dependency)
- `multi-goose` - The 3 goose containers

**Start command:**
```bash
cd deploy/compose
docker compose -f ce.dev.yml \
  --profile controller \
  --profile privacy-guard \
  --profile privacy-guard-proxy \
  --profile ollama \
  --profile multi-goose \
  up -d
```

### 4. Configuration Flow

Each goose container follows this startup sequence (from Task C.1):

1. **Wait for Controller health check** (`/status` endpoint)
2. **Get JWT from Keycloak**
   - Uses `client_credentials` grant
   - Host header override: `Host: localhost:8080` (for issuer matching)
3. **Fetch profile from Controller** (`/profiles/{role}`)
   - Includes extensions, privacy settings, policies
   - Signature validation happens at Controller
4. **Generate config.yaml** from profile JSON
5. **Start goose session** with auto-configured settings

## Technical Decisions

### 1. Profile Dependency Chain

**Problem:** Docker Compose profiles with dependencies
- goose services depend on `privacy-guard-proxy` (profile)
- privacy-guard depends on `ollama` (profile)
- Activating only `multi-goose` fails validation

**Solution:** Document complete profile list required
- Tests use all profiles: `--profile controller --profile privacy-guard --profile privacy-guard-proxy --profile ollama --profile multi-goose`
- Alternative considered: Remove ollama dependency from privacy-guard (but this would break previous phase work)

### 2. Keycloak Access from Containers

**Problem:** JWT issuer must match Controller's `OIDC_ISSUER_URL`
- Controller expects: `http://localhost:8080/realms/dev`
- Container can't use `localhost` (it's the container itself)

**Solution:** Use `host.docker.internal` with Host header override
```yaml
extra_hosts:
  - "host.docker.internal:host-gateway"
```
Container entrypoint uses:
```bash
curl "http://host.docker.internal:8080/..." -H "Host: localhost:8080"
```

This makes Keycloak issue JWTs with the correct issuer.

### 3. Service vs Container Names

- **Service name:** `controller` (used in Docker network DNS)
- **Container name:** `ce_controller` (used for docker ps/logs)
- goose containers use service names: `CONTROLLER_URL=http://controller:8088`

### 4. Workspace Isolation

Each goose instance has its own volume:
- Prevents file conflicts between agents
- Simulates separate user workspaces
- Persists across container restarts

## Testing

Created comprehensive test suite: `tests/integration/test_multi_goose_startup.sh`

**18 tests total (all passing ✅):**
1. Docker Compose file exists
2. Configuration is valid
3-5. Services defined (finance, manager, legal)
6-8. Workspace volumes defined
9. Services use `multi-goose` profile
10-12. Services have correct roles
13. Host header mapping present
14. Services depend on controller
15. Services depend on privacy-guard-proxy
16. Services use correct Docker image
17. Docker image exists locally
18. Profiles are signed in database

**Run tests:**
```bash
./tests/integration/test_multi_goose_startup.sh
```

## Configuration Requirements

### Environment Variables (.env.ce)

Required from previous phases:
- `OIDC_CLIENT_SECRET` - Keycloak client secret
- `OPENROUTER_API_KEY` - LLM API key
- `OIDC_ISSUER_URL=http://localhost:8080/realms/dev` (must match JWT issuer)

Optional:
- `GOOSE_PROVIDER` - Default: `openrouter/anthropic/claude-3.5-sonnet`
- `GOOSE_MODEL` - Default: `anthropic/claude-3.5-sonnet`

### Prerequisites

1. **Docker goose image** (from Task C.1)
   ```bash
   docker images | grep goose-test:0.1.0
   ```

2. **Profile signatures** (from Task C.1)
   ```bash
   ./scripts/sign-all-profiles.sh
   ```

3. **Base services running**
   - postgres (profiles always on)
   - keycloak (profiles always on)
   - vault (profiles always on)

## Usage

### Start Multi-goose Environment

```bash
cd deploy/compose
docker compose -f ce.dev.yml \
  --profile controller \
  --profile privacy-guard \
  --profile privacy-guard-proxy \
  --profile ollama \
  --profile multi-goose \
  up -d
```

### Monitor Startup

```bash
# Watch all services
docker compose -f ce.dev.yml logs -f

# Watch specific goose instance
docker logs -f ce_goose_finance

# Check running containers
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

### Verify Configuration

```bash
# Check if goose container auto-configured
docker exec ce_goose_finance cat ~/.config/goose/config.yaml

# Should show:
# - provider: openrouter/...
# - api_base: http://privacy-guard-proxy:8090/v1  (via Privacy Guard Proxy)
# - role: finance
# - extensions: [from profile]
# - privacy: [from profile]
```

### Stop Multi-goose Environment

```bash
# Stop only multi-goose services
docker compose -f ce.dev.yml --profile multi-goose down

# Or stop everything
docker compose -f ce.dev.yml down
```

## Files Modified

- `deploy/compose/ce.dev.yml` - Added 3 services, 3 volumes

## Files Created

- `tests/integration/test_multi_goose_startup.sh` - 18 comprehensive tests
- `docs/implementation/c2-docker-compose-multi-goose.md` - This document

## Integration Points

### With Task C.1 (Docker goose Image)
- Uses `goose-test:0.1.0` image
- Relies on entrypoint script auto-configuration
- Uses config generator Python script

### With Workstream A (Lifecycle Integration)
- Depends on Controller `/profiles/{role}` endpoint
- Requires Vault profile signatures

### With Workstream B (Privacy Guard Proxy)
- Routes all LLM requests through Privacy Guard Proxy
- `PRIVACY_GUARD_PROXY_URL=http://privacy-guard-proxy:8090`

### With Phase 3 (Authentication)
- Uses Keycloak OAuth2 client_credentials grant
- JWT validation at Controller

## Lessons Learned

### Docker Compose Profile Dependencies
**Issue:** Services with profiles can depend on other profile services, but Docker Compose doesn't auto-activate dependent profiles.

**Learning:** Must explicitly list ALL required profiles when starting services. Document the complete command prominently.

### YAML Config Format vs Shell Format
**Issue:** Test scripts checked for `GOOSE_ROLE=finance` but `docker compose config` outputs YAML: `GOOSE_ROLE: finance`

**Learning:** When testing config output, match the output format (YAML), not the input format (env file).

### Signature JSON Structure
**Issue:** Controller returns signature as nested object `{signature: {signature: "vault:v1:..."}}`

**Learning:** Always test API responses directly; don't assume flat structure. Use `jq -r '.signature.signature'`.

## Next Steps

**Task C.3:** Agent Mesh Configuration
- Configure agent-to-agent communication
- Implement peer discovery
- Add routing rules
- Enable collaborative workflows

**Task C.4:** Testing
- End-to-end multi-agent scenarios
- Verify agent mesh communication
- Validate policy enforcement across agents

## Success Criteria ✅

- [x] 3 goose services added to Docker Compose
- [x] Each service configured with different role
- [x] Workspace volumes for isolation
- [x] Dependencies configured (controller, privacy-guard-proxy)
- [x] All 18 tests passing
- [x] Documentation complete
- [x] No breaking changes to existing configuration

**Result:** Task C.2 complete. Multi-goose Docker environment ready for agent mesh configuration.
