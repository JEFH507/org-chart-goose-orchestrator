# ADR-0028: Profile Storage Using PostgreSQL JSONB

**Status**: Accepted  
**Date**: 2025-11-07  
**Deciders**: Javier (Project Owner)  
**Phase**: Phase 5

---

## Context

Phase 5 introduces role-based profiles (finance, legal, developer, hr, executive, support) with complex nested configurations including providers, extensions, policies, privacy settings, recipes, and Vault signatures. We needed to decide on the storage format and database schema.

**Requirements**:
1. Store 60+ fields per profile (nested structures)
2. Support partial updates (e.g., update only privacy settings)
3. Enable fast retrieval by role (primary key)
4. Allow future querying (e.g., "find all profiles with local_only=true")
5. Maintain backward compatibility (profiles can evolve)
6. Store Vault HMAC signatures as part of profile

**Constraints**:
- PostgreSQL 17.2 already deployed
- Rust controller with serde_json
- Profile schema may evolve (new fields in Phase 6+)

---

## Decision

We will store profiles using **PostgreSQL JSONB column** with the following schema:

```sql
CREATE TABLE profiles (
    role VARCHAR(255) PRIMARY KEY,
    display_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    config JSONB NOT NULL,  -- Complete Profile struct
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_profiles_config ON profiles USING GIN (config);
```

**Storage Strategy**:
- `role`: Primary key (e.g., "finance", "legal")
- `display_name`, `description`: Denormalized for quick listing
- `config`: Full Profile struct as JSONB (all 60+ fields)
- GIN index on `config` for future JSONB queries

**Update Strategy**:
- **Full Replace**: Overwrite entire `config` JSONB (used by PUT endpoint)
- **Partial Merge**: Use `jsonb_set()` for specific paths (e.g., signature)
- **Signature Field**: Nested within `config.signature` (optional field)

---

## Rationale

### Option 1: Relational Normalization (Rejected)
**Approach**: Separate tables for providers, extensions, policies, etc.

**Pros**:
- Traditional relational design
- Enforces referential integrity

**Cons**:
- ❌ Requires 10+ tables (profiles, providers, extensions, policies, privacy_rules, recipes, automated_tasks, goosehints, gooseignore, env_vars)
- ❌ Complex JOIN queries for profile retrieval
- ❌ Schema migrations for every profile field addition
- ❌ Partial updates require multi-table transactions
- ❌ Poor performance for nested structures (N+1 queries)

**Rejected**: Too complex for profile use case, poor performance

---

### Option 2: JSON Column (Text) (Rejected)
**Approach**: Store profile as JSON text (VARCHAR or TEXT)

**Pros**:
- Simple storage

**Cons**:
- ❌ No indexing (cannot query by nested fields)
- ❌ No validation (any JSON accepted)
- ❌ Poor performance for partial updates (must parse entire JSON)
- ❌ No GIN index support

**Rejected**: Limited query capabilities, no indexing

---

### Option 3: JSONB Column (Selected) ✅
**Approach**: Store profile as JSONB with GIN index

**Pros**:
- ✅ Single query retrieval (3ms P50)
- ✅ GIN index for nested field queries (e.g., `config @> '{"privacy":{"local_only":true}}'`)
- ✅ Partial updates via `jsonb_set()` (signature field)
- ✅ Schema evolution without migrations (new fields just added)
- ✅ Backward compatible (old profiles work with new code)
- ✅ Native PostgreSQL binary format (faster than JSON text)
- ✅ Rust serde_json integration (deserialize to Profile struct)

**Cons**:
- ⚠️ Denormalized (duplicates data, but profiles are small ~10KB each)
- ⚠️ JSONB operators less familiar than SQL (learning curve)

**Selected**: Best balance of performance, flexibility, and simplicity

---

## Implementation Details

### Profile Schema (Rust)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub role: String,
    pub display_name: String,
    pub description: String,
    pub providers: Providers,
    pub extensions: Vec<Extension>,
    pub goosehints: GooseHints,
    pub gooseignore: GooseIgnore,
    pub recipes: Vec<Recipe>,
    pub automated_tasks: Vec<AutomatedTask>,
    pub policies: Vec<Policy>,
    pub privacy: PrivacyConfig,
    pub env_vars: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,  // Vault HMAC
}
```

### Database Operations

**Create Profile (D7)**:
```sql
INSERT INTO profiles (role, display_name, description, config)
VALUES ($1, $2, $3, $4::jsonb);
```

**Read Profile (D1)**:
```sql
SELECT config FROM profiles WHERE role = $1;
```

**Update Profile (D8)** - Full replace:
```sql
UPDATE profiles
SET config = $1::jsonb, updated_at = NOW()
WHERE role = $2;
```

**Publish Profile (D9)** - Partial update (signature only):
```sql
UPDATE profiles
SET config = jsonb_set(
    config,
    '{signature}',
    $1::jsonb
),
updated_at = NOW()
WHERE role = $2;
```

**Future Query Example** (Phase 6+):
```sql
-- Find all profiles with local_only privacy
SELECT role, display_name
FROM profiles
WHERE config @> '{"privacy":{"local_only":true}}';

-- Find all profiles allowing github extension
SELECT role
FROM profiles
WHERE config @> '{"extensions":[{"name":"github"}]}';
```

### GIN Index Usage
- Supports `@>` (contains) operator
- Supports `?` (key exists) operator
- Supports `->>` (extract text) operator
- Optimizes JSONB path queries

---

## Consequences

### Positive
1. ✅ **Fast Retrieval**: Single query, no JOINs (3ms P50)
2. ✅ **Flexible Schema**: Add fields without migrations
3. ✅ **Backward Compatible**: Old profiles work with new code
4. ✅ **Partial Updates**: `jsonb_set()` for signature field
5. ✅ **Future Queries**: GIN index enables complex filters
6. ✅ **Rust Integration**: serde_json deserializes to struct

### Negative
1. ⚠️ **Learning Curve**: JSONB operators less familiar than SQL
2. ⚠️ **Denormalized**: Data duplication (mitigated: profiles small ~10KB)
3. ⚠️ **Type Safety**: JSONB doesn't enforce schema (mitigated: Rust struct validation)

### Neutral
1. 🔄 **Profile Size**: ~10KB per profile (6 profiles = 60KB total, negligible)
2. 🔄 **Migration**: Simple ALTER TABLE for adding top-level columns

---

## Alternatives Considered

### NoSQL Database (e.g., MongoDB)
**Rejected**: Adds deployment complexity, PostgreSQL JSONB provides similar capabilities

### Key-Value Store (e.g., Redis)
**Rejected**: Profiles need persistence, transactions, and querying

### Separate Config Files
**Rejected**: No centralized management, no audit trail, no RBAC

---

## Validation

**Performance** (Phase 5 results):
- Profile retrieval: P50=3ms (target <5ms) ✅
- Profile update: P50=4ms (target <10ms) ✅
- Profile publish: P50=27ms (target <50ms) ✅

**Test Coverage**:
- 10 profile loading tests (H2)
- 3 admin profile tests (Vault)
- 20 comprehensive profile tests (H6.1)
- 100% passing (33/33 profile-related tests)

**Schema Evolution**:
- Signature field added in Phase 5 (no migration needed)
- Future fields (Phase 6+) can be added without breaking existing profiles

---

## Related Decisions

- **ADR-0016**: CE Profile Signing Key Management (Vault HMAC for signatures)
- **ADR-0011**: Directory Policy Profile Bundles (original profile concept)
- **ADR-0012**: Storage and Metadata Model (metadata-only storage strategy)

---

## References

- PostgreSQL JSONB: https://www.postgresql.org/docs/17/datatype-json.html
- GIN Index: https://www.postgresql.org/docs/17/gin-intro.html
- Profile Specification: `docs/profiles/SPEC.md`
- Database Schema: `src/controller/migrations/20241105_profiles.sql`
