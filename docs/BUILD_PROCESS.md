# Build Process Documentation

## Overview
This document defines the **canonical build and deployment process** for the Goose Orchestrator Controller service. Always follow this process to ensure consistency and avoid image/environment confusion.

## Critical Principles

1. **Single Source of Truth**: Docker Compose (`deploy/compose/ce.dev.yml`)
2. **Standard Image Tag**: Always `ghcr.io/jefh507/goose-controller:0.1.0`
3. **Environment Management**: `.env.ce` loaded via symlink (H0 fix)
4. **No Ad-hoc Builds**: Never use `docker build` or `docker run` directly

## Standard Build Process

### 1. Normal Build (with cache)
Use this for incremental changes when cache is valid:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin
docker compose -f deploy/compose/ce.dev.yml build controller
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller
```

**When to use**: Code changes where dependencies haven't changed

### 2. Clean Build (no cache)
Use this when you need to force a complete rebuild:

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Run build in background to avoid timeouts
nohup docker compose -f deploy/compose/ce.dev.yml build --no-cache controller > /tmp/controller_build.log 2>&1 &

# Monitor progress
tail -f /tmp/controller_build.log

# When complete, recreate container (IMPORTANT: use --force-recreate)
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller
```

**When to use**: 
- Dependency changes (Cargo.toml updates)
- Suspected cache corruption
- After fixing type mismatches or struct changes
- When `docker restart` doesn't pick up code changes

**Why background**: Rust release builds take 4+ minutes, which can timeout shell tools

### 3. Verify Deployment

After any build, verify the new image is running:

```bash
# Check image was created
docker images | grep goose-controller

# Check container is running new image
docker inspect ce_controller --format '{{.Image}}'

# Compare image SHAs - they should match
# Example output:
# ghcr.io/jefh507/goose-controller   0.1.0   e878df48be8a   2 minutes ago
# sha256:e878df48be8a903b859742d761d14dd5052247ae5fe9cb23300fb177c12166ee
```

**Critical**: If image SHAs don't match, use `--force-recreate`:
```bash
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller
```

## Image Tag Strategy

### Current Approach
- **Production tag**: `ghcr.io/jefh507/goose-controller:0.1.0`
- **No per-session tags**: Avoids proliferation of tags (h4-1762454293, etc.)
- **Semantic versioning**: Bump tag on major releases only

### Why This Works
- Docker Compose always pulls/builds to same tag
- SHA256 hashes provide actual version tracking
- Simpler deployment (no tag management needed)

### Future Consideration
If we need multiple concurrent versions:
```yaml
# deploy/compose/ce.dev.yml
services:
  controller:
    image: ghcr.io/jefh507/goose-controller:${CONTROLLER_VERSION:-0.1.0}
```

Then in `.env.ce`:
```bash
CONTROLLER_VERSION=0.2.0-phase5
```

## Environment Variables

### Loading Process (H0 Fix)
Environment variables loaded from `.env.ce` via symlink:

```bash
# Symlink structure
deploy/compose/.env -> ../../.env.ce

# Docker Compose automatically loads .env from compose directory
# All ${VAR:-default} substitutions work correctly
```

### Critical Variables
```bash
# Keycloak (must use container name, not localhost)
OIDC_ISSUER_URL=http://ce_keycloak:8080/realms/dev
OIDC_CLIENT_ID=goose-controller

# Database
DATABASE_URL=postgresql://postgres:postgres@ce_postgres:5432/orchestrator_dev

# Redis
REDIS_URL=redis://ce_redis:6379
```

**Common Error**: Using `localhost` in container context
- ❌ `http://localhost:8080/realms/dev` (container's localhost)
- ✅ `http://ce_keycloak:8080/realms/dev` (Docker network)

## Troubleshooting

### Issue: Code changes not reflected after rebuild

**Symptoms**: 
- Image SHA unchanged after `docker compose build`
- Tests still fail with old error

**Cause**: Docker cached build layers

**Solution**:
```bash
# Clean build
docker compose -f deploy/compose/ce.dev.yml build --no-cache controller
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller
```

### Issue: Container still using old image after build

**Symptoms**:
- `docker images` shows new SHA
- `docker inspect ce_controller` shows old SHA

**Cause**: `docker compose restart` doesn't recreate containers

**Solution**:
```bash
# Use --force-recreate flag
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller
```

### Issue: HTTP 501 (Not Implemented) on new routes

**Symptoms**: New API endpoints return HTTP 501

**Cause**: Routes not registered in `src/controller/src/main.rs`

**Solution**:
1. Add routes to both JWT-protected and non-JWT paths
2. Rebuild controller
3. Verify routes in logs: `docker logs ce_controller 2>&1 | grep "Registered routes"`

### Issue: JWT verification fails (JWKS fetch error)

**Symptoms**:
```
Failed to fetch JWKS: error sending request for url (http://localhost:8080/realms/dev/protocol/openid-connect/certs)
```

**Cause**: Using `localhost` instead of container name

**Solution**: Verify `.env.ce` has correct values:
```bash
OIDC_ISSUER_URL=http://ce_keycloak:8080/realms/dev  # Use ce_keycloak, not localhost
```

## Build Verification Checklist

After any deployment, run this checklist:

```bash
# 1. Check all 7 services healthy
docker ps --format "{{.Names}}\t{{.Status}}"

# 2. Verify controller image SHA
docker inspect ce_controller --format '{{.Image}}'
docker images | grep goose-controller | head -1

# 3. Check controller logs for errors
docker logs ce_controller --tail 50 2>&1 | grep -i error

# 4. Test health endpoint
curl -s http://localhost:8088/health | jq .

# 5. Run integration tests
./tests/integration/test_finance_pii_jwt.sh
./tests/integration/test_legal_local_jwt.sh
./tests/integration/test_org_chart_jwt.sh
```

Expected: 30/30 tests passing (8 Finance + 10 Legal + 12 Org Chart)

## Integration Test Strategy

### Test Philosophy
**REAL E2E**, not simulation:
- Real JWT tokens from Keycloak
- Real HTTP calls to Controller API
- Real database verification via PostgreSQL
- Real Privacy Guard service integration

### Test Files
```
tests/integration/
├── test_finance_pii_jwt.sh      # H1-H2: Finance PII redaction (8 tests)
├── test_legal_local_jwt.sh      # H3: Legal local-only (10 tests)
├── test_org_chart_jwt.sh        # H4: Org chart import (12 tests)
└── test_data/
    └── org_chart_sample.csv     # 10 sample users, 4 departments
```

### Running Tests
```bash
# Individual workstream
./tests/integration/test_org_chart_jwt.sh

# All Phase 5 tests
for test in tests/integration/test_*_jwt.sh; do
  echo "Running $test..."
  $test || echo "FAILED: $test"
done
```

## Common Mistakes to Avoid

### ❌ Don't Do This
```bash
# Ad-hoc docker build (loses compose context)
docker build -t goose-controller:custom .

# Manual docker run (bypasses environment loading)
docker run -e VAR=value goose-controller:custom

# Simple restart (doesn't pick up new image)
docker compose restart controller

# Using localhost in .env.ce
OIDC_ISSUER_URL=http://localhost:8080/realms/dev
```

### ✅ Do This Instead
```bash
# Use compose build
docker compose -f deploy/compose/ce.dev.yml build controller

# Use compose up with --force-recreate
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller

# Use container names in .env.ce
OIDC_ISSUER_URL=http://ce_keycloak:8080/realms/dev
```

## Phase 5 Build History

### H0: Environment Fix
- **Problem**: `.env.ce` not loaded by Docker Compose
- **Solution**: Created symlink `deploy/compose/.env -> ../../.env.ce`
- **Verification**: All ${VAR:-default} substitutions working

### H1-H3: Finance + Legal Tests
- **Builds**: Standard cached builds sufficient
- **Image**: `ghcr.io/jefh507/goose-controller:0.1.0` (SHA: 9caaffaa1f28)
- **Result**: 18/18 tests passing

### H4: Org Chart Tests
- **Problem 1**: Routes not registered (HTTP 501)
- **Fix**: Added admin routes to main.rs
- **Build 1**: Standard build (SHA: e878df48be8a)
- **Result**: 11/12 tests passing

- **Problem 2**: Timestamp type mismatch in org.rs  
- **Fix**: Changed `DateTime<Utc>` to `NaiveDateTime` in query
- **Build 2**: Required `--no-cache` to clear cached layers with old code
- **Image**: `ghcr.io/jefh507/goose-controller:0.1.0` (SHA: f0782faa48ba)
- **Result**: 12/12 tests passing ✅

**Lessons Learned**:
1. **Type/struct changes require --no-cache**: Docker layer cache doesn't detect type changes
2. **Always use --force-recreate**: Ensures container uses new image, not cached old one
3. **Verify image SHA**: Check `docker inspect ce_controller` matches `docker images` output
4. **Test immediately**: Run integration tests right after deployment to catch issues early

## Quick Reference

### One-Command Deploy
```bash
# From project root
docker compose -f deploy/compose/ce.dev.yml build --no-cache controller && \
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller && \
sleep 5 && \
docker logs ce_controller --tail 20
```

### One-Command Test Suite
```bash
# Run all Phase 5 integration tests
cd /home/papadoc/Gooseprojects/goose-org-twin && \
./tests/integration/test_finance_pii_jwt.sh && \
./tests/integration/test_legal_local_jwt.sh && \
./tests/integration/test_org_chart_jwt.sh
```

Expected output: `✅ ALL TESTS PASSED` (30/30)

---

**Document Version**: 1.0  
**Last Updated**: 2025-11-06  
**Phase**: Phase 5 (H4 completion)  
**Maintained By**: Phase 5 Integration Testing workstream
