# Workstream B Structural Validation Tests

## Overview

This test suite validates the structural integrity of Phase 5 Workstream B deliverables **without requiring runtime dependencies** (policy engine, recipe executor, admin APIs).

These are **structural tests** (file format, syntax, schema compliance) not **behavioral tests** (runtime policy enforcement, recipe execution). Behavioral tests will be implemented in Workstream H (Integration Testing).

## Test Coverage

### 1. Profile YAML Schemas (`test_profile_schemas.sh`)

**What it tests:**
- All 6 profiles exist (`finance.yaml`, `manager.yaml`, `analyst.yaml`, `marketing.yaml`, `support.yaml`, `legal.yaml`)
- Valid YAML syntax (uses `yq` if available, falls back to basic checks)
- Required fields present: `role`, `display_name`, `providers`, `extensions`, `recipes`, `privacy`, `policies`, `signature`
- Role name matches filename
- Primary provider configured
- Extensions array properly formatted
- Privacy mode is valid (`strict`, `hybrid`, `moderate`, `rules`, `permissive`)
- Signature algorithm configured

**Test count:** 8 tests × 6 profiles = 48 tests

### 2. Recipe YAML Schemas (`test_recipe_schemas.sh`)

**What it tests:**
- All 18 recipe files exist and are readable
- Required fields present: `name`, `version`, `role`, `trigger`, `steps`
- Trigger has type configured
- Schedule triggers have cron expressions
- Cron expression format (5 or 6 fields)
- Steps array properly formatted
- Each step has an ID
- Tool references use valid format (`extension__tool`)

**Test count:** ~9 tests × 18 recipes = 162 tests

### 3. Goosehints Markdown Syntax (`test_goosehints_syntax.sh`)

**What it tests:**
- All 8 goosehints templates exist and are readable
- Files are not empty
- Contains Markdown headers
- Code blocks properly closed (even number of ` ``` ` markers)
- No broken Markdown link syntax
- Contains role-specific context
- Standard heading structure
- File size is reasonable (not truncated or corrupted)

**Test count:** 8 tests × 8 templates = 64 tests

### 4. Gooseignore Pattern Validation (`test_gooseignore_patterns.sh`)

**What it tests:**
- All 8 gooseignore templates exist and are readable
- Files are not empty
- Contains valid ignore patterns (non-comment, non-empty lines)
- Standard glob patterns present (`**/`, `*.`, etc.)
- No shell injection risks (dangerous characters)
- No duplicate patterns
- Has section comments for organization
- Role-specific patterns present (SSN/EIN for finance, attorney-client for legal, PII for support, etc.)

**Test count:** 8 tests × 8 templates = 64 tests

### 5. SQL Seed Script (`test_sql_seed.sh`)

**What it tests:**
- `seeds/profiles.sql` exists and is not empty
- Contains 6 INSERT statements (one per role)
- All role names present (`finance`, `manager`, `analyst`, `marketing`, `support`, `legal`)
- JSONB casting syntax present (`'::jsonb`)
- Parentheses balanced (syntax check)
- Verification SELECT queries present
- **Database load test** (if Postgres is available): SQL loads successfully in transaction (rollback test)

**Test count:** 8 tests

---

**Total tests:** ~346 structural validation tests

## Running the Tests

### Run All Tests

```bash
cd tests/workstream-b
./run_all_tests.sh
```

### Run Individual Test Suites

```bash
# Profile schemas only
./test_profile_schemas.sh

# Recipe schemas only
./test_recipe_schemas.sh

# Goosehints syntax only
./test_goosehints_syntax.sh

# Gooseignore patterns only
./test_gooseignore_patterns.sh

# SQL seed script only
./test_sql_seed.sh
```

## Dependencies

### Required
- `bash` (standard shell)
- `grep`, `sed`, `awk` (POSIX utilities)
- `find`, `wc` (file operations)

### Optional
- `yq` - Enhanced YAML validation (if not available, falls back to basic checks)
- `psql` + Docker - Database load testing (if not available, skips DB test)

## Exit Codes

- `0` - All tests passed
- `1` - One or more tests failed

## Example Output

```
==========================================
Phase 5 Workstream B Test Suite
==========================================
Structural Validation Tests
Date: 2025-11-05 22:30:00

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Test Suite 1/5: Profile YAML Schemas
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
=== Profile Schema Validation ===

Testing: profiles/finance.yaml
  ✅ File exists
  ✅ Valid YAML syntax
  ✅ Role name matches filename
  ✅ Primary provider configured
  ✅ Extensions array present
  ✅ Valid privacy mode
  ✅ Signature algorithm configured

Testing: profiles/manager.yaml
  ...

===================================
Profile Schema Validation Summary
===================================
Total tests: 48
Passed: 48
Failed: 0

✅ All profile schema tests passed!

...

==========================================
Workstream B Test Suite - Final Summary
==========================================
Test Suites Run: 5
Passed: 5
Failed: 0

✅ All test suites passed!

Deliverables validated:
  - 6 role profiles (YAML schemas)
  - 18 recipes (cron schedules, tool refs)
  - 8 goosehints templates (Markdown syntax)
  - 8 gooseignore templates (glob patterns)
  - 1 SQL seed script (Postgres JSONB)
```

## What's NOT Tested (Deferred to Workstream H)

These behavioral/integration tests require runtime components:

- **Policy enforcement** - Does Finance role actually get blocked from `developer__shell`?
- **Recipe execution** - Do cron jobs actually trigger and run?
- **Profile signing** - Does Vault HMAC signing work via `/admin/profiles/{role}/publish`?
- **Profile loading** - Does the profile loader service correctly transform YAML → goose config?
- **Privacy engine** - Do gooseignore patterns actually block file access?
- **Agent mesh** - Can agents communicate via `agent_mesh__notify`?
- **End-to-end flows** - Finance agent → Excel → Budget report

## CI/CD Integration

Add to GitHub Actions workflow (`.github/workflows/phase5-tests.yml`):

```yaml
- name: Run Workstream B Structural Tests
  run: |
    cd tests/workstream-b
    ./run_all_tests.sh
```

## Maintenance

- **Add new profiles:** Update `PROFILES` array in `test_profile_schemas.sh`
- **Add new recipes:** No changes needed (auto-discovered via `find`)
- **Add new goosehints:** No changes needed (auto-discovered via `find`)
- **Add new gooseignore:** Update role-specific checks in `test_gooseignore_patterns.sh`

## Troubleshooting

**Issue:** `yq: command not found`
- **Solution:** Tests will fall back to basic validation. Install `yq` for enhanced YAML checks:
  ```bash
  sudo snap install yq
  # or
  brew install yq
  ```

**Issue:** Database load test fails
- **Solution:** Ensure Postgres container is running:
  ```bash
  docker ps | grep ce_postgres
  ```
- Or skip database test (other tests still validate SQL syntax)

**Issue:** Permission denied
- **Solution:** Make scripts executable:
  ```bash
  chmod +x tests/workstream-b/*.sh
  ```

## Related Documentation

- **Master Plan:** `Technical Project Plan/master-technical-project-plan.md`
- **Phase 5 State:** `Technical Project Plan/PM Phases/Phase-5/Phase-5-Agent-State.json`
- **Progress Log:** `docs/tests/phase5-progress.md`
- **Workstream H Plan:** Integration testing strategy (behavioral tests)
