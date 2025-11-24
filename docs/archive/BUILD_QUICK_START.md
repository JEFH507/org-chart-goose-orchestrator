# Quick Start: Controller Build & Deploy

## TL;DR - Just Want to Deploy?

```bash
cd /home/papadoc/Gooseprojects/goose-org-twin

# Clean build (force fresh compilation)
docker compose -f deploy/compose/ce.dev.yml build --no-cache controller

# Deploy (force container recreation)
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller

# Verify (should see 12/12, 10/10, 8/8)
./tests/integration/test_org_chart_jwt.sh
./tests/integration/test_legal_local_jwt.sh
./tests/integration/test_finance_pii_jwt.sh
```

**Expected**: 30/30 tests passing ✅

---

## When to Use --no-cache

### ✅ Use --no-cache for:
- Type changes (e.g., `DateTime<Utc>` → `NaiveDateTime`)
- Struct modifications (adding/removing fields)
- After fixing compilation errors
- When tests fail with old behavior after code changes

### ❌ Don't need --no-cache for:
- Dependency updates only (Cargo.toml changes)
- Comment changes
- Documentation updates
- First build of the day

---

## Quick Verification

```bash
# Check new image was built
docker images | grep goose-controller | head -1

# Check container is running NEW image
docker inspect ce_controller --format '{{.Image}}'

# SHAs should match! If not:
docker compose -f deploy/compose/ce.dev.yml up -d --force-recreate controller
```

---

## Troubleshooting One-Liners

```bash
# Environment variables loading?
docker exec ce_controller env | grep -E "(OIDC|DATABASE|REDIS)"

# Database connected to orchestrator (not postgres)?
docker exec ce_controller env | grep DATABASE_URL
# Should show: orchestrator (not postgres DB)

# JWT verification enabled?
docker logs ce_controller 2>&1 | grep "JWT verification"

# Services healthy?
docker ps --format "{{.Names}}\t{{.Status}}"
# Should show: 7/7 healthy (controller, postgres, keycloak, vault, redis, ollama, privacy_guard)
```

---

## Image Tag Reference

**Current Tag**: `ghcr.io/jefh507/goose-controller:0.1.0`

**Recent SHAs** (for debugging):
- `f0782faa48ba` - H4 complete (12/12 tests, timestamp fix)
- `e878df48be8a` - H4 routes added (11/12 tests)
- `9caaffaa1f28` - H1-H3 complete (18/18 tests)

**Check your SHA**:
```bash
docker inspect ce_controller --format '{{.Image}}' | cut -d: -f2 | cut -c1-12
# Should show: f0782faa48ba (latest)
```

---

## Full Documentation

See [BUILD_PROCESS.md](BUILD_PROCESS.md) for comprehensive details:
- Environment variable management
- Image tagging strategy
- Troubleshooting guide
- Phase 5 build history
- Common mistakes

---

**Quick Start Version**: 1.0  
**Last Updated**: 2025-11-06 19:10  
**For**: Resume sessions without context buildup
