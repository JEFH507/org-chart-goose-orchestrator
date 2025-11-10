# Multi-Goose Setup Guide

**Version:** 1.0  
**Last Updated:** 2025-11-10  
**Phase:** 6 - Workstream C

## Overview

This guide explains how to start and use the Multi-Goose test environment with 3 independent Goose agents (Finance, Manager, Legal) communicating via Agent Mesh.

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Goose Finance│     │ Goose Manager│     │  Goose Legal │
│  (Container) │     │  (Container) │     │  (Container) │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       │ Agent Mesh Tools   │                    │
       │ (send_task, etc.)  │                    │
       └────────────────────┴────────────────────┘
                            │
                            v
                   ┌─────────────────┐
                   │   Controller    │
                   │  (Task Routing) │
                   └────────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              v             v             v
       ┌───────────┐ ┌──────────────┐ ┌────────┐
       │ Keycloak  │ │Privacy Guard │ │Postgres│
       │  (Auth)   │ │   Proxy      │ │  (DB)  │
       └───────────┘ └──────────────┘ └────────┘
```

## Prerequisites

### 1. Environment Configuration

Ensure these variables are set in your environment configuration:
- `OPENROUTER_API_KEY` - Required for Goose LLM calls
- `OIDC_CLIENT_SECRET` - Keycloak client secret
- `VAULT_DEV_ROOT_TOKEN` - Vault root token

### 2. Profiles Configured

All 8 agent profiles must be seeded in the database with:
- Agent Mesh extension enabled
- Privacy Guard Proxy endpoint configured
- Valid Vault signatures

Run the profile signing script if needed:
```bash
./scripts/sign-all-profiles.sh
```

## Starting the Environment

### Quick Start (All Services)

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

**Wait Time:** ~30-60 seconds for all services to become healthy

### Verify Startup

```bash
# Check all services are running
docker compose -f ce.dev.yml ps

# Expected output (should show HEALTHY or running status):
# ce_controller             running (healthy)
# ce_privacy_guard_proxy    running (healthy)
# ce_goose_finance          running
# ce_goose_manager          running
# ce_goose_legal            running
```

### Check Logs

```bash
# Finance agent logs
docker logs ce_goose_finance

# Should show:
# ✓ Controller is ready
# ✓ JWT token acquired  
# ✓ Profile fetched successfully
# ✓ config.yaml generated
# Starting Goose session for role: finance
```

## Agent Profiles

Each agent automatically receives its profile from the Controller:

| Container | Role | Extensions | Workspace |
|-----------|------|------------|-----------|
| ce_goose_finance | finance | github, agent_mesh, memory, excel-mcp | /workspace |
| ce_goose_manager | manager | github, agent_mesh, memory | /workspace |
| ce_goose_legal | legal | agent_mesh, memory | /workspace |

### Key Features

**Auto-Configuration:**
- No manual config needed
- Profile fetched from Controller API
- config.yaml generated automatically
- JWT authentication handled automatically

**Workspace Isolation:**
- Each agent has its own Docker volume
- Files created by one agent are NOT visible to others
- Ensures data isolation and privacy

**Agent Mesh Enabled:**
- All 3 agents can communicate via Agent Mesh tools
- 4 tools available: send_task, request_approval, notify, fetch_status
- All communication routed through Controller API

## Using Agent Mesh

### Testing via Controller API

Agent Mesh tools make HTTP calls to Controller API. Example:

```bash
# Note: Replace $YOUR_KEYCLOAK_SECRET with actual secret

# Get JWT token
TOKEN=$(curl -s -X POST http://localhost:8080/realms/dev/protocol/openid-connect/token \
    -d "client_id=goose-controller" \
    -d "grant_type=client_credentials" \
    -d "client_secret=$YOUR_KEYCLOAK_SECRET" \
    | jq -r '.access_token')

# Send task from Finance to Manager
curl -X POST http://localhost:8088/tasks/route \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -H "Idempotency-Key: $(uuidgen)" \
    -d '{
        "target": "manager",
        "task": {"type": "budget_approval", "amount": 50000},
        "context": {"department": "Engineering"}
    }'
```

## Workspace Management

Each agent has an isolated workspace:

```bash
# Finance workspace
docker exec ce_goose_finance ls -la /workspace

# Create a file (only visible to finance)
docker exec ce_goose_finance touch /workspace/budget-2026.xlsx

# Verify manager cannot see it
docker exec ce_goose_manager ls /workspace/budget-2026.xlsx
# Error: No such file or directory ✓
```

## Troubleshooting

### Containers won't start

**Symptom:** `service depends on undefined service`

**Solution:** Include ALL required profiles:
```bash
docker compose -f ce.dev.yml \
    --profile controller \
    --profile privacy-guard \
    --profile privacy-guard-proxy \
    --profile ollama \
    --profile multi-goose \
    up -d
```

### Failed to get JWT token

**Solutions:**
1. Verify Keycloak is running:
   ```bash
   curl http://localhost:8080/realms/dev | jq -r '.realm'
   # Should output: dev
   ```

2. Check client secret is configured in environment

### Profile signature invalid

**Solution:** Sign all profiles:
```bash
./scripts/sign-all-profiles.sh
```

### Agent Mesh tools not available

**Checks:**
1. Profile has agent_mesh extension:
   ```bash
   curl http://localhost:8088/profiles/finance | jq '.extensions[] | select(.name=="agent_mesh")'
   ```

2. Config includes agent_mesh:
   ```bash
   docker exec ce_goose_finance cat ~/.config/goose/config.yaml | grep -A 5 "agent_mesh:"
   ```

## Testing

Run the comprehensive test suite:

```bash
./tests/integration/test_multi_agent_communication.sh
```

**Expected:** 18/18 tests passing ✅

## Stopping the Environment

```bash
cd deploy/compose
docker compose -f ce.dev.yml --profile multi-goose down
```

## Cleanup (Removes Data!)

```bash
cd deploy/compose
docker compose -f ce.dev.yml --profile multi-goose down -v
```

## Related Documentation

- **Test Suite:** `tests/integration/test_multi_agent_communication.sh`
- **Agent Mesh Extension:** `src/agent-mesh/README.md`
- **Testing Guide:** `docs/operations/TESTING-GUIDE.md`

## Lessons Learned

### Critical Issues Resolved

1. **Goose Session Command** (v1.13.1)
   - ❌ `goose session start` does not exist
   - ✅ Use `goose session` (no subcommand)
   - Impact: Containers now start successfully

2. **Provider Configuration Format**
   - ❌ `GOOSE_PROVIDER=openrouter/anthropic/claude-3.5-sonnet` (invalid)
   - ✅ Separate: `GOOSE_PROVIDER=openrouter` and `GOOSE_MODEL=anthropic/claude-3.5-sonnet`
   - Impact: Goose sessions initialize correctly

3. **Container Keep-Alive**
   - ❌ `goose session` exits immediately without stdin
   - ✅ `tail -f /dev/null | goose session` keeps container running
   - Impact: Containers stay responsive for agent mesh communication

4. **Profile Signing Required**
   - ❌ Unsigned profiles rejected with HTTP 403
   - ✅ Sign all profiles using `/admin/profiles/{role}/publish` endpoint
   - ✅ Restart Controller if Vault token expires (1-hour TTL)
   - Impact: Profile signature verification enforces integrity

5. **Test Path Assumptions**
   - ❌ `~/.config` expands to host home directory in docker exec
   - ✅ Use absolute paths: `/root/.config/goose/config.yaml`
   - Impact: Tests correctly validate container configuration

### Docker Image Versions

- **v0.2.0:** Initial agent mesh integration (always-on)
- **v0.2.1:** Profile-controlled agent mesh (admin decides which roles get it)
- **v0.2.2:** Fixed `goose session start` → `goose session` command
- **v0.2.3:** Fixed provider format + container keep-alive ✅ **CURRENT**

### Performance Notes

- Container startup time: ~30 seconds (profile fetch + config generation + session init)
- Profile signature verification: ~50ms per profile
- Vault AppRole token TTL: 3600 seconds (1 hour, renewable)
- Memory per Goose container: ~200MB baseline
- Test suite execution time: ~90 seconds (including 30s startup wait)
