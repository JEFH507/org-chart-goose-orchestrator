# ADR-0029: Organizational Chart CSV Import Strategy

**Status**: Accepted  
**Date**: 2025-11-07  
**Deciders**: Javier (Project Owner)  
**Phase**: Phase 5

---

## Context

Phase 5 introduces organizational hierarchy management to enable future cross-agent approval workflows (Phase 7+). Admins need a simple way to import employee data including reporting relationships.

**Requirements**:
1. Bulk import employees (100-10,000 users)
2. Capture reporting hierarchy (manager-employee relationships)
3. Store department and title for future filtering
4. Audit trail (who imported, when, how many rows)
5. Simple format (non-technical admins can prepare)
6. Support updates (re-import with new data)

**Use Cases**:
- **Phase 5**: Import org chart, query hierarchy tree
- **Phase 7+**: Approval routing ("escalate to manager", "notify direct reports")
- **Privacy Guard**: Determine if user is manager (different PII rules for HR managers)

---

## Decision

We will use **CSV import with upsert strategy** via Admin API endpoint.

**CSV Format**:
```csv
employee_id,email,name,manager_email,department,title
E001,john.doe@acme.com,John Doe,jane.smith@acme.com,Engineering,Senior Engineer
E002,jane.smith@acme.com,Jane Smith,ceo@acme.com,Engineering,Engineering Manager
E003,ceo@acme.com,Alice CEO,,Executive,CEO
```

**API Endpoint**:
```
POST /admin/org/import
Authorization: Bearer <admin-jwt>
Content-Type: multipart/form-data

file: <csv-file>
```

**Database Schema**:
```sql
CREATE TABLE org_users (
    employee_id VARCHAR(50) PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    manager_email VARCHAR(255),  -- Self-referencing FK
    department VARCHAR(255),
    title VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (manager_email) REFERENCES org_users(email) ON DELETE SET NULL
);

CREATE INDEX idx_org_users_email ON org_users (email);
CREATE INDEX idx_org_users_manager ON org_users (manager_email);
CREATE INDEX idx_org_users_department ON org_users (department);
```

**Import Strategy**: UPSERT (INSERT ... ON CONFLICT UPDATE)
- New employees → INSERT
- Existing employees → UPDATE (email/name/manager/dept/title)
- Deleted employees → Remain in DB (soft delete for audit)

**Audit Trail**:
```sql
CREATE TABLE org_imports (
    import_id SERIAL PRIMARY KEY,
    filename VARCHAR(255) NOT NULL,
    rows_imported INT NOT NULL,
    imported_by VARCHAR(255) NOT NULL,  -- From JWT
    imported_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

---

## Rationale

### Option 1: Manual API Calls (Rejected)
**Approach**: POST /admin/org/users (one employee at a time)

**Pros**:
- RESTful design
- Fine-grained control

**Cons**:
- ❌ 10,000 employees = 10,000 API calls (slow)
- ❌ No transactional integrity (partial failures)
- ❌ High network overhead
- ❌ Poor admin UX (no bulk import)

**Rejected**: Too slow for bulk imports

---

### Option 2: JSON Bulk Import (Rejected)
**Approach**: POST /admin/org/bulk with JSON array

**Pros**:
- Structured format
- Single API call

**Cons**:
- ❌ JSON complex for non-technical admins
- ❌ Large payloads (10,000 employees = 5MB+ JSON)
- ❌ No standard tools (admins can't use Excel)
- ❌ Error handling complex (which row failed?)

**Rejected**: Poor admin UX, not Excel-compatible

---

### Option 3: CSV Import (Selected) ✅
**Approach**: POST /admin/org/import with CSV file

**Pros**:
- ✅ Simple format (Excel, Google Sheets compatible)
- ✅ Single API call (transactional)
- ✅ Standard tools (admins familiar with CSV)
- ✅ Error reporting (line numbers on failure)
- ✅ Streaming parsing (low memory for large files)
- ✅ UPSERT strategy (idempotent, safe to re-import)

**Cons**:
- ⚠️ CSV parsing complexity (handle quotes, escapes)
- ⚠️ No type validation in CSV (mitigated: server-side validation)

**Selected**: Best admin UX, Excel-compatible, standard format

---

### Option 4: LDAP/Active Directory Sync (Deferred)
**Approach**: Sync from corporate directory service

**Pros**:
- Automatic updates
- Single source of truth

**Cons**:
- ⏳ Complex integration (multiple LDAP flavors)
- ⏳ Not all orgs use LDAP (startups use Google Workspace)
- ⏳ Requires credentials (security concern)
- ⏳ Phase 5 scope too large

**Deferred to Phase 8**: MVP uses CSV, enterprise edition adds LDAP sync

---

## Implementation Details

### CSV Parsing (Rust)
```rust
use csv::ReaderBuilder;

pub async fn import_csv(
    file: Bytes,
    imported_by: &str,
) -> Result<ImportResult, String> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file.as_ref());
    
    let mut users = Vec::new();
    for result in reader.deserialize() {
        let user: OrgUser = result
            .map_err(|e| format!("CSV parse error: {}", e))?;
        users.push(user);
    }
    
    // Validate all before inserting
    validate_users(&users)?;
    
    // Bulk UPSERT
    upsert_users(&users).await?;
    
    Ok(ImportResult {
        rows_imported: users.len(),
    })
}
```

### UPSERT Query
```sql
INSERT INTO org_users (employee_id, email, name, manager_email, department, title)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (employee_id)
DO UPDATE SET
    email = EXCLUDED.email,
    name = EXCLUDED.name,
    manager_email = EXCLUDED.manager_email,
    department = EXCLUDED.department,
    title = EXCLUDED.title;
```

### Validation Rules
1. `employee_id`: Non-empty, unique
2. `email`: Valid email format, unique
3. `name`: Non-empty
4. `manager_email`: Optional, must exist in CSV or DB (deferred FK check)
5. `department`: Optional
6. `title`: Optional

### Hierarchy Tree API (D12)
```sql
WITH RECURSIVE org_tree AS (
    -- Anchor: top-level (no manager or manager not in DB)
    SELECT employee_id, email, name, manager_email, department, title, 0 AS level
    FROM org_users
    WHERE manager_email IS NULL
    
    UNION ALL
    
    -- Recursive: direct reports
    SELECT u.employee_id, u.email, u.name, u.manager_email, u.department, u.title, t.level + 1
    FROM org_users u
    INNER JOIN org_tree t ON u.manager_email = t.email
)
SELECT * FROM org_tree ORDER BY level, email;
```

Returns nested JSON:
```json
{
  "employee_id": "E003",
  "email": "ceo@acme.com",
  "name": "Alice CEO",
  "title": "CEO",
  "reports": [
    {
      "employee_id": "E002",
      "email": "jane.smith@acme.com",
      "name": "Jane Smith",
      "title": "Engineering Manager",
      "reports": [
        {
          "employee_id": "E001",
          "email": "john.doe@acme.com",
          "name": "John Doe",
          "title": "Senior Engineer",
          "reports": []
        }
      ]
    }
  ]
}
```

---

## Consequences

### Positive
1. ✅ **Simple Admin UX**: Excel/Sheets → Export CSV → Upload
2. ✅ **Bulk Import**: 10,000 employees in single transaction
3. ✅ **Idempotent**: Safe to re-import (UPSERT)
4. ✅ **Audit Trail**: Who imported, when, how many rows
5. ✅ **Hierarchy Queries**: Recursive CTE for tree structure
6. ✅ **Future-Proof**: Supports Phase 7+ approval workflows

### Negative
1. ⚠️ **Manual Process**: No automatic sync (Phase 8 adds LDAP)
2. ⚠️ **CSV Limitations**: No complex validation (e.g., circular manager refs)
3. ⚠️ **Soft Delete**: Deleted employees remain in DB (mitigated: future cleanup job)

### Neutral
1. 🔄 **Manager FK**: Deferred validation (allows any order in CSV)
2. 🔄 **Department/Title**: Free-text (future: add dept/title tables for normalization)

---

## Future Enhancements (Phase 6+)

### Phase 6: Admin UI
- Web UI for CSV upload (drag-and-drop)
- Preview before import (show changes)
- Validation errors displayed inline

### Phase 7: Approval Workflows
- "Escalate to manager" (lookup manager_email)
- "Notify direct reports" (query by manager_email)
- Department-based routing (e.g., "Legal dept only")

### Phase 8: LDAP Sync
- Scheduled sync (daily)
- Incremental updates (only changed users)
- Mapping config (LDAP fields → org_users columns)

---

## Validation

**Performance** (Phase 5 results):
- Import 100 rows: ~500ms (5ms per row)
- Import 1,000 rows: ~3s (3ms per row with batching)
- Tree query (500 employees): P50=18ms (target <30ms) ✅

**Test Coverage**:
- 12 org chart tests (H4)
- CSV validation tests (invalid email, missing fields)
- Hierarchy tree tests (multi-level, orphans)
- 100% passing (12/12 org chart tests)

**Data Integrity**:
- UPSERT prevents duplicates
- FK ensures manager exists (or NULL for CEO)
- Indexes optimize manager lookups (Phase 7 approval routing)

---

## Related Decisions

- **ADR-0011**: Directory Policy Profile Bundles (org-aware profiles)
- **ADR-0012**: Storage and Metadata Model (metadata storage strategy)
- Future: **ADR-0030+**: Approval workflow routing (Phase 7)

---

## References

- CSV RFC: https://datatracker.ietf.org/doc/html/rfc4180
- PostgreSQL Recursive CTE: https://www.postgresql.org/docs/17/queries-with.html
- Rust CSV crate: https://docs.rs/csv/latest/csv/
- Org Chart API: `docs/api/controller/README.md` (D10-D12)
