-- Rollback migration 0004: Drop org chart tables
-- This rollback removes org_users and org_imports tables

-- Drop indexes first
DROP INDEX IF EXISTS idx_org_users_role;
DROP INDEX IF EXISTS idx_org_users_reports_to;
DROP INDEX IF EXISTS idx_org_users_email;
DROP INDEX IF EXISTS idx_org_users_department;
DROP INDEX IF EXISTS idx_org_imports_status;
DROP INDEX IF EXISTS idx_org_imports_uploaded_at;

-- Drop trigger
DROP TRIGGER IF EXISTS update_org_users_updated_at ON org_users;

-- Drop tables (CASCADE removes foreign key constraints)
DROP TABLE IF EXISTS org_imports CASCADE;
DROP TABLE IF EXISTS org_users CASCADE;
