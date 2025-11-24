# Migration Guide: Phase 4 → Phase 5

**Version**: 0.4.0 → 0.5.0  
**Release Date**: 2025-11-07  
**Migration Time**: ~30 minutes  
**Breaking Changes**: ❌ None

---

## Table of Contents

1. [Overview](#overview)
2. [What's New in Phase 5](#whats-new-in-phase-5)
3. [Breaking Changes](#breaking-changes)
4. [New Features](#new-features)
5. [Database Migrations](#database-migrations)
6. [Environment Variables](#environment-variables)
7. [API Changes](#api-changes)
8. [Testing Verification](#testing-verification)
9. [Rollback Procedures](#rollback-procedures)

---

## 1. Overview

### Release Summary

Phase 5 introduces **role-based profiles**, **organizational charts**, and **Vault integration** for cryptographic profile signing. This release focuses on enterprise-grade configuration management and identity-based policy enforcement.

### Upgrade Path

```
Phase 4 (0.4.0)                Phase 5 (0.5.0)
├── Privacy Guard HTTP API  →  ✅ Retained
├── Basic audit logging     →  ✅ Enhanced with org identity
├── Postgres storage        →  ✅ Extended with 3 new tables
└── Keycloak auth           →  ✅ Used for admin endpoints

                               + Role-based profiles (6 roles)
                               + Org chart management
                               + Vault HMAC signing
                               + 13 new endpoints
```

### Compatibility

- ✅ **Backward Compatible**: All Phase 4 endpoints still work
- ✅ **No Breaking Changes**: Existing integrations unaffected
- ✅ **Database**: Additive only (no schema changes to existing tables)
- ✅ **API**: All Phase 4 routes functional (no removals)

---

## 2. What's New in Phase 5

### 2.1 Role-Based Profiles

**Problem Solved**: Agents for different roles (finance, legal, HR) need different:
- LLM providers (e.g., legal requires local-only Ollama)
- Tool permissions (e.g., finance cannot use `developer__shell`)
- Privacy settings (e.g., legal needs ephemeral memory)
- Context and guardrails (e.g., finance hints about approval thresholds)

**Solution**: Profile System
- **6 Pre-configured Roles**: finance, legal, developer, hr, executive, support
- **60+ Configuration Fields**: providers, extensions, policies, privacy, recipes
- **YAML-based**: Human-editable `profiles/*.yaml` files
- **Database-backed**: Postgres JSONB storage for fast retrieval
- **Cryptographically Signed**: Vault HMAC-SHA256 prevents tampering

**Example Use Case**:
```
Finance user logs in → JWT role claim = "finance"
  → Controller loads finance profile
    → Config generated with:
      - Primary model: Claude 3.5 Sonnet (accurate for financial data)
      - Extensions: github (budget tracking), excel-mcp
      - Policies: deny_tool = developer__shell (no code execution)
      - Privacy: strict PII redaction (SSN, credit card)
      - Goosehints: "You are a finance agent. Approval threshold: $10K..."
```

**Documentation**: `docs/profiles/SPEC.md`

---

### 2.2 Organizational Chart Management

**Problem Solved**: Manual CSV imports for org structure were error-prone

**Solution**: Org Chart API
- **CSV Import**: `POST /admin/org/import` (D10 endpoint)
- **Tree API**: `GET /org/tree` (D11 endpoint) returns hierarchical JSON
- **User Lookup**: `GET /org/tree/{email}` (D12 endpoint) finds user + subtree
- **Database-backed**: `org_users` table stores employee data
- **Audit Trail**: `org_imports` table tracks import history

**Example Use Case**:
```
Admin uploads org chart CSV (employee_id, email, name, manager_email, department)
  → Controller validates + imports to database
    → Privacy Guard queries manager hierarchy
      → Finance user's request: "Show my team's budget"
        → Controller resolves "my team" = org subtree under user's manager
          → Returns budget data scoped to team members
```

**Documentation**: `docs/api/controller/org-endpoints.md`

---

### 2.3 Vault Integration

**Problem Solved**: Profile tampering risk (e.g., finance user grants themselves `developer__shell` tool)

**Solution**: HashiCorp Vault HMAC Signing
- **Transit Engine**: HMAC-SHA256 signatures on profile publish
- **Tamper Detection**: Signature verification prevents unauthorized changes
- **Admin Workflow**: Create → Update → Publish (sign) → Verify (Phase 6)
- **Dev Mode**: Integrated in Phase 5 (HTTP, root token, in-memory)
- **Production**: Phase 6 upgrade (HTTPS, AppRole, persistent storage, audit)

**Example Use Case**:
```
Admin creates finance profile → Stored unsigned in database
  → Admin publishes profile → Vault generates HMAC signature
    → Signature stored in profile.signature field
      → (Phase 6) User loads profile → Controller verifies signature
        → If signature mismatch → Reject profile (tampered!)
```

**Documentation**: `docs/guides/VAULT.md`

---

## 3. Breaking Changes

### ❌ None

**Verification**: All Phase 4 endpoints tested and passing:
- ✅ Privacy Guard HTTP API (`/privacy-guard/*`)
- ✅ Audit logging (`/audit/*`)
- ✅ Health checks (`/health`)

**Test Coverage**: 60/60 integration tests passing (Phase 5 test suite)

---

## 4. New Features

### 4.1 Profile System (Workstream H)

**Endpoints** (D-series):

| Endpoint | Method | Purpose | Auth |
|----------|--------|---------|------|
| `GET /profiles/{role}` | GET | Load profile by role | JWT (user) |
| `POST /admin/profiles` | POST | Create new profile | JWT (admin) |
| `PUT /admin/profiles/{role}` | PUT | Update existing profile | JWT (admin) |
| `POST /admin/profiles/{role}/publish` | POST | Sign profile with Vault | JWT (admin) |

**Config Generation Endpoints** (D2-D6):

| Endpoint | Method | Purpose | Auth |
|----------|--------|---------|------|
| `GET /config/providers` | GET | LLM provider config | JWT (user) |
| `GET /config/extensions` | GET | MCP extensions config | JWT (user) |
| `GET /config/goosehints` | GET | Global hints | JWT (user) |
| `GET /config/gooseignore` | GET | Global ignore patterns | JWT (user) |
| `GET /config/privacy` | GET | Privacy Guard config | JWT (user) |
| `GET /config/policies` | GET | RBAC/ABAC rules | JWT (user) |

**Features**:
- JWT-based role extraction (`jwt_role` claim from Keycloak)
- Profile loading with schema validation (6 rules)
- Deserialization from YAML/JSON (dual format support)
- Config generation for Goose agent initialization
- Vault HMAC signing for integrity protection

**Example Request**:
```bash
# User loads their profile
curl -H "Authorization: Bearer $JWT" \
  http://localhost:8088/profiles/finance
```

**Example Response**:
```json
{
  "role": "finance",
  "display_name": "Finance Team Agent",
  "description": "Budget approvals, compliance reporting",
  "providers": {
    "primary": {"provider": "openrouter", "model": "anthropic/claude-3.5-sonnet"},
    "allowed_providers": ["openrouter"]
  },
  "extensions": [
    {"name": "github", "enabled": true, "tools": ["list_issues", "create_issue"]}
  ],
  "policies": [
    {"rule_type": "deny_tool", "pattern": "developer__shell", "reason": "No code exec"}
  ],
  "privacy": {
    "mode": "strict",
    "strictness": "strict",
    "pii_categories": ["SSN", "EMAIL", "PHONE"]
  },
  "signature": {
    "algorithm": "sha2-256",
    "signed_at": "2025-11-07T04:29:31Z",
    "signature": "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="
  }
}
```

---

### 4.2 Org Chart Management (Workstream D)

**Endpoints**:

| Endpoint | Method | Purpose | Auth |
|----------|--------|---------|------|
| `POST /admin/org/import` | POST | Import CSV org chart | JWT (admin) |
| `GET /org/tree` | GET | Full org tree (hierarchical JSON) | JWT (user) |
| `GET /org/tree/{email}` | GET | User + subtree | JWT (user) |

**CSV Format**:
```csv
employee_id,email,name,manager_email,department,title
1,ceo@example.com,Alice CEO,,Executive,Chief Executive Officer
2,cfo@example.com,Bob CFO,ceo@example.com,Finance,Chief Financial Officer
3,john@example.com,John Doe,cfo@example.com,Finance,Senior Analyst
```

**Features**:
- CSV validation (required columns, email format)
- Duplicate detection (by email)
- Manager hierarchy resolution
- Tree building with cycle detection
- Database storage with import metadata

**Example Request**:
```bash
# Import org chart
curl -X POST -H "Authorization: Bearer $ADMIN_JWT" \
  -H "Content-Type: multipart/form-data" \
  -F "file=@org-chart.csv" \
  http://localhost:8088/admin/org/import
```

**Example Response** (Tree):
```json
{
  "email": "cfo@example.com",
  "name": "Bob CFO",
  "department": "Finance",
  "title": "Chief Financial Officer",
  "reports": [
    {
      "email": "john@example.com",
      "name": "John Doe",
      "department": "Finance",
      "title": "Senior Analyst",
      "reports": []
    }
  ]
}
```

---

### 4.3 Vault Integration (Phase 5 MVP)

**Vault Service**:
- Image: `hashicorp/vault:1.18.3`
- Mode: Dev mode (HTTP, root token, in-memory storage)
- Transit Engine: Enabled via `vault-init.sh` script
- Key: `profile-signing` (HMAC-SHA256)

**Environment Variables**:
```bash
VAULT_ADDR=http://vault:8200
VAULT_TOKEN=root
```

**Publish Workflow**:
```
Admin creates profile (D7)
  → Profile stored unsigned
    → Admin publishes profile (D9)
      → Controller serializes profile to JSON
        → Vault Transit HMAC operation
          → Signature stored in profile.signature field
```

**Signature Format**:
```json
{
  "algorithm": "sha2-256",
  "vault_key": "transit/keys/profile-signing",
  "signed_at": "2025-11-07T04:29:31.058861974+00:00",
  "signed_by": "admin@example.com",
  "signature": "vault:v1:6wmfS0Vo91Ga0E9BkInhWZvLJ3qQodEnXhykdywB8kc="
}
```

**Phase 6 Production**:
- TLS/HTTPS encryption
- AppRole authentication (replace root token)
- Persistent storage (Raft or Consul)
- Audit device (compliance logging)
- Signature verification on profile load

**Documentation**: `docs/guides/VAULT.md`

---

## 5. Database Migrations

### 5.1 New Tables

**Three new tables added** (no changes to existing tables):

#### Table 1: `profiles`
```sql
CREATE TABLE profiles (
    role VARCHAR(255) PRIMARY KEY,
    display_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_profiles_config ON profiles USING GIN (config);
```

**Purpose**: Store role-based profiles as JSONB  
**Columns**:
- `role`: Unique identifier (PK)
- `display_name`: Human-readable name
- `description`: Role description
- `config`: Complete Profile struct (JSONB)
- `created_at`, `updated_at`: Timestamps

---

#### Table 2: `org_users`
```sql
CREATE TABLE org_users (
    employee_id VARCHAR(50) PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    manager_email VARCHAR(255),
    department VARCHAR(255),
    title VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_org_users_email ON org_users (email);
CREATE INDEX idx_org_users_manager ON org_users (manager_email);
CREATE INDEX idx_org_users_department ON org_users (department);
```

**Purpose**: Store organizational hierarchy  
**Columns**:
- `employee_id`: Unique employee identifier (PK)
- `email`: Email address (unique)
- `name`: Full name
- `manager_email`: Manager's email (FK to email)
- `department`: Department name
- `title`: Job title

---

#### Table 3: `org_imports`
```sql
CREATE TABLE org_imports (
    import_id SERIAL PRIMARY KEY,
    filename VARCHAR(255) NOT NULL,
    rows_imported INT NOT NULL,
    imported_by VARCHAR(255) NOT NULL,
    imported_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

**Purpose**: Audit trail for org chart imports  
**Columns**:
- `import_id`: Auto-incrementing ID (PK)
- `filename`: CSV filename
- `rows_imported`: Number of employees imported
- `imported_by`: Admin email (from JWT)
- `imported_at`: Import timestamp

---

### 5.2 Migration Scripts

**Apply Migrations**:
```bash
# Run migrations
cd src/controller
sqlx migrate run --database-url "postgres://controller:password@localhost:5432/controller"
```

**Migration Files** (applied in order):
```
migrations/
  20251105_create_profiles_table.sql
  20251105_create_org_users_table.sql
  20251105_create_org_imports_table.sql
```

**Rollback** (if needed):
```bash
sqlx migrate revert --database-url "postgres://controller:password@localhost:5432/controller"
```

---

### 5.3 Data Population

**Profiles** (6 roles):
```bash
# Import profiles from YAML files
for role in finance legal developer hr executive support; do
  curl -X POST http://localhost:8088/admin/profiles \
    -H "Authorization: Bearer $ADMIN_JWT" \
    -H "Content-Type: application/json" \
    -d @profiles/${role}.json
done
```

**Org Chart** (CSV):
```bash
# Import org chart
curl -X POST http://localhost:8088/admin/org/import \
  -H "Authorization: Bearer $ADMIN_JWT" \
  -F "file=@org-chart-sample.csv"
```

---

## 6. Environment Variables

### 6.1 New Variables (Vault Integration)

**Controller Service** (`deploy/compose/ce.dev.yml`):
```yaml
controller:
  environment:
    # Existing vars (unchanged)
    DATABASE_URL: postgres://controller:password@postgres:5432/controller
    KEYCLOAK_URL: http://keycloak:8080
    PRIVACY_GUARD_URL: http://privacy-guard:8000
    
    # NEW: Vault integration (Phase 5)
    VAULT_ADDR: ${VAULT_ADDR:-http://vault:8200}
    VAULT_TOKEN: ${VAULT_TOKEN:-root}
```

**Default Values**: Fallback to dev mode if `.env` not present

**Production** (`.env` file):
```bash
# Phase 6 production values
VAULT_ADDR=https://vault.example.com:8200
VAULT_ROLE_ID=abc123...
VAULT_SECRET_ID=def456...
```

---

### 6.2 Existing Variables (Unchanged)

All Phase 4 environment variables remain the same:
- `DATABASE_URL`
- `KEYCLOAK_URL`
- `PRIVACY_GUARD_URL`
- `RUST_LOG`

**No changes required** for existing deployments.

---

## 7. API Changes

### 7.1 New Endpoints (13 total)

**Profile System** (9 endpoints):
- `GET /profiles/{role}` - Load profile by role
- `POST /admin/profiles` - Create profile
- `PUT /admin/profiles/{role}` - Update profile
- `POST /admin/profiles/{role}/publish` - Sign profile
- `GET /config/providers` - Provider config
- `GET /config/extensions` - Extensions config
- `GET /config/goosehints` - Goosehints
- `GET /config/gooseignore` - Gooseignore
- `GET /config/privacy` - Privacy config
- `GET /config/policies` - Policies

**Org Chart** (3 endpoints):
- `POST /admin/org/import` - Import CSV
- `GET /org/tree` - Full tree
- `GET /org/tree/{email}` - User subtree

**Privacy Guard** (1 enhanced endpoint):
- `POST /privacy-guard/filter` - Enhanced with org identity context

---

### 7.2 Modified Endpoints (None)

**All Phase 4 endpoints unchanged**:
- ✅ `POST /privacy-guard/filter` (backward compatible)
- ✅ `GET /health`
- ✅ `/audit/*` endpoints

---

### 7.3 Removed Endpoints (None)

**No removals** - fully backward compatible.

---

## 8. Testing Verification

### 8.1 Pre-Migration Tests

**Before upgrading**, verify Phase 4 functionality:
```bash
# Privacy Guard
curl -X POST http://localhost:8000/filter \
  -H "Content-Type: application/json" \
  -d '{"text":"My SSN is 123-45-6789","mode":"rules"}'

# Health check
curl http://localhost:8088/health
```

**Expected Results**:
- Privacy Guard: PII redacted
- Health: `{"status":"healthy"}`

---

### 8.2 Post-Migration Tests

**After upgrading to Phase 5**, run full test suite:

```bash
# H2: Profile Loading (10 tests)
./tests/integration/test_profile_loading.sh

# H3: Privacy Guard (18 tests)
./tests/integration/test_finance_pii_jwt.sh
./tests/integration/test_legal_local_jwt.sh

# H4: Org Chart (12 tests)
./tests/integration/test_org_chart_jwt.sh

# H6: E2E Workflow (10 tests)
./tests/integration/test_e2e_workflow.sh

# H6.1: All Profiles (20 tests)
./tests/integration/test_all_profiles_comprehensive.sh

# H7: Performance (7 tests)
./tests/perf/api_latency_benchmark.sh
```

**Expected Results**: 60/60 tests passing (100%)

---

### 8.3 Regression Tests

**Verify Phase 4 functionality still works**:
```bash
# Privacy Guard HTTP API
./tests/integration/test_privacy_guard_http.sh

# Audit logging
./tests/integration/test_audit_logging.sh
```

**Expected Results**: All Phase 4 tests still passing

---

### 8.4 Performance Validation

**Profile Loading Latency** (Target: P50 < 20ms):
```bash
./tests/perf/api_latency_benchmark.sh
```

**Expected Results**:
- Profile Loading P50: <20ms ✅
- Config Generation P50: <15ms ✅
- Privacy Guard P50: <10ms ✅

**Actual Results** (as of 2025-11-07):
- Profile Loading P50: 19ms
- Config Generation P50: 12ms
- Privacy Guard P50: 8ms

---

## 9. Rollback Procedures

### 9.1 Rollback Steps

**If issues occur after migration**:

1. **Stop services**:
   ```bash
   docker compose -f deploy/compose/ce.dev.yml down
   ```

2. **Revert database migrations**:
   ```bash
   sqlx migrate revert --database-url "postgres://controller:password@localhost:5432/controller"
   sqlx migrate revert  # Repeat 3 times (3 new tables)
   ```

3. **Restore Phase 4 docker-compose**:
   ```bash
   git checkout v0.4.0 deploy/compose/ce.dev.yml
   ```

4. **Restart services**:
   ```bash
   docker compose -f deploy/compose/ce.dev.yml up -d
   ```

5. **Verify Phase 4 functionality**:
   ```bash
   curl http://localhost:8088/health
   ```

**Estimated Time**: 10 minutes

---

### 9.2 Data Backup

**Before migration**, backup Postgres database:
```bash
pg_dump -U controller -d controller > phase4_backup.sql
```

**Restore if needed**:
```bash
psql -U controller -d controller < phase4_backup.sql
```

---

### 9.3 Zero-Downtime Rollback

**For production deployments**:

1. **Deploy Phase 5 to staging first**
2. **Run full test suite on staging**
3. **If issues found**:
   - Keep Phase 4 running in production
   - Fix issues on staging
   - Re-test
4. **If all tests pass**:
   - Blue-green deployment to production
   - Monitor for 24 hours
   - Rollback if metrics degrade

---

## Appendix A: Version Comparison

| Feature | Phase 4 (0.4.0) | Phase 5 (0.5.0) |
|---------|-----------------|-----------------|
| **Profile System** | ❌ | ✅ (6 roles) |
| **Org Chart** | ❌ | ✅ (CSV import) |
| **Vault Integration** | ❌ | ✅ (Dev mode) |
| **Privacy Guard** | ✅ HTTP API | ✅ Enhanced |
| **Audit Logging** | ✅ Basic | ✅ Org identity |
| **Endpoints** | ~10 | ~23 (+13 new) |
| **Database Tables** | ~5 | ~8 (+3 new) |
| **Test Coverage** | 40 tests | 60 tests |

---

## Appendix B: Deployment Checklist

**Pre-Migration**:
- [ ] Review release notes
- [ ] Backup Postgres database
- [ ] Test rollback procedure on staging
- [ ] Notify users of maintenance window

**Migration**:
- [ ] Apply database migrations
- [ ] Update docker-compose (add Vault service)
- [ ] Update environment variables (VAULT_ADDR, VAULT_TOKEN)
- [ ] Restart services
- [ ] Verify services healthy

**Post-Migration**:
- [ ] Run full test suite (60 tests)
- [ ] Verify Phase 4 endpoints still work
- [ ] Import initial profiles (6 roles)
- [ ] Import org chart CSV
- [ ] Monitor logs for errors
- [ ] Performance validation (latency targets)

**Production** (Phase 6 readiness):
- [ ] Plan Vault production upgrade (TLS, AppRole, persistent storage)
- [ ] Document disaster recovery procedures
- [ ] Schedule security review

---

## Appendix C: Related Documentation

- **Profile Specification**: `docs/profiles/SPEC.md`
- **Vault Guide**: `docs/guides/VAULT.md`
- **API Reference**: `docs/api/controller/README.md`
- **Test Results**: `docs/tests/phase5-test-results.md`
- **Progress Log**: `docs/tests/phase5-progress.md`
- **ADRs**: `docs/adr/0016-ce-profile-signing-key-management.md`

---

## Appendix D: Support

**Issues**: Create GitHub issue with `migration` label  
**Documentation**: See `docs/` directory  
**Community**: Discord #phase5-migration channel  

---

**End of Migration Guide**
