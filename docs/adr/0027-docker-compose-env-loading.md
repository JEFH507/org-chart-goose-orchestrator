# ADR-0027: Docker Compose Environment Loading Strategy

**Status:** Accepted  
**Date:** 2025-11-06  
**Context:** Phase 5 Integration Testing revealed persistent issues with environment variable loading in docker-compose

## Problem

During Phase 5 integration testing, we encountered recurring issues where docker-compose services (controller, privacy-guard) were not loading environment variables from `.env.ce`:

1. **OIDC variables were blank** - causing JWT validation failures
2. **DATABASE_URL pointed to wrong database** - `postgres` instead of `orchestrator`
3. **Workaround of manually passing env vars** - not persistent across container restarts

### Root Cause

Docker Compose has two environment mechanisms:

1. **`${VAR}` substitution in compose file** - reads from **host shell environment** or `.env` file
2. **`env_file:` directive** - loads file **into container** runtime environment

Our `ce.dev.yml` used `${OIDC_ISSUER_URL}` syntax (substitution), but `.env.ce` was:
- Not automatically loaded by docker-compose (only `.env` is auto-loaded)
- Not accessible via `env_file:` directive due to security (.gooseignored)

## Decision

Use **symlink approach** to bridge docker-compose's auto-loading mechanism with our security requirements:

```bash
cd deploy/compose
ln -s .env.ce .env
```

### Why This Works

1. Docker Compose **automatically loads** `.env` file from compose directory
2. Symlink `.env → .env.ce` makes `.env.ce` values available for `${VAR}` substitution
3. Both `.env` and `.env.ce` are in `.gooseignore` - no secrets committed
4. Changes to `.env.ce` are immediately picked up via symlink

### Implementation

1. **Updated `.env.ce.example`**:
   - Fixed `DATABASE_URL` to point to `orchestrator` database
   - Added `OIDC_CLIENT_SECRET` placeholder with instructions

2. **Created `scripts/setup-env.sh`**:
   - Automates setup process
   - Creates `.env.ce` from template if missing
   - Creates symlink `.env → .env.ce`
   - Validates critical configuration

3. **Updated `docs/guides/compose-ce.md`**:
   - Added step 2: Create symlink for auto-loading
   - Added warnings about OIDC_CLIENT_SECRET and DATABASE_URL

4. **No changes to `ce.dev.yml`**:
   - Keep using `${VAR}` substitution syntax
   - No `env_file:` directive needed (since `.env` is auto-loaded)

## Consequences

### Positive

- ✅ **Persistent configuration** - survives container restarts
- ✅ **No manual env passing** - docker-compose handles it automatically
- ✅ **Security maintained** - `.env` and `.env.ce` both in `.gooseignore`
- ✅ **DX improved** - single `setup-env.sh` script for onboarding
- ✅ **No docker-compose warnings** - all `${VAR}` substitutions resolve

### Negative

- ⚠️ **Symlink indirection** - users must understand `.env` is a link
- ⚠️ **Manual setup step** - requires running `setup-env.sh` or creating symlink
- ⚠️ **Platform limitation** - symlinks on Windows require admin/developer mode

### Risks Mitigated

- 🔒 **Secrets exposure** - `.env` symlink is gooseignored (cannot be committed)
- 🔧 **Configuration drift** - setup script validates critical values
- 📋 **Documentation gap** - compose-ce.md now includes explicit setup instructions

## Alternatives Considered

### Option A: Use `--env-file` flag
```bash
docker compose --env-file .env.ce up
```
**Rejected:** Requires remembering flag every time; error-prone

### Option B: Add `env_file: [.env.ce]` directive
```yaml
controller:
  env_file:
    - .env.ce
```
**Rejected:** Only loads into container runtime, doesn't help `${VAR}` substitution in compose file

### Option C: Rename `.env.ce` to `.env`
**Rejected:** Would require updating all documentation and existing deployments

### Option D: Use environment variables in shell
```bash
export $(cat .env.ce | xargs) && docker compose up
```
**Rejected:** Not persistent; requires re-export on new shell

## Validation

Tested with fresh container restart:

```bash
# Before fix
$ docker exec ce_controller env | grep OIDC_ISSUER_URL
OIDC_ISSUER_URL=

$ docker exec ce_controller env | grep DATABASE_URL
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/postgres

# After fix (with symlink)
$ docker exec ce_controller env | grep OIDC_ISSUER_URL
OIDC_ISSUER_URL=http://localhost:8080/realms/dev

$ docker exec ce_controller env | grep DATABASE_URL
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/orchestrator
```

Controller logs confirm:
```json
{"message":"JWT verification enabled","issuer":"http://localhost:8080/realms/dev","audience":"goose-controller"}
{"message":"database connected"}
```

## References

- Docker Compose environment documentation: https://docs.docker.com/compose/environment-variables/
- `.gooseignore` configuration: project root
- Setup script: `scripts/setup-env.sh`
- Example config: `deploy/compose/.env.ce.example`
