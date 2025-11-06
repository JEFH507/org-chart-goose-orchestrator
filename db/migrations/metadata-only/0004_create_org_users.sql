-- Phase 5 Workstream D: Org Chart Tables (D10-D12)
-- Migration: Create org_users and org_imports tables

-- Org users table (org chart hierarchy)
CREATE TABLE IF NOT EXISTS org_users (
    user_id INTEGER PRIMARY KEY,
    reports_to_id INTEGER REFERENCES org_users(user_id),
    name VARCHAR(100) NOT NULL,
    role VARCHAR(50) NOT NULL REFERENCES profiles(role),
    email VARCHAR(200) UNIQUE NOT NULL,
    department VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Org imports table (CSV upload history)
CREATE TABLE IF NOT EXISTS org_imports (
    id SERIAL PRIMARY KEY,
    filename VARCHAR(200) NOT NULL,
    uploaded_by VARCHAR(200) NOT NULL,
    uploaded_at TIMESTAMP NOT NULL DEFAULT NOW(),
    users_created INTEGER DEFAULT 0,
    users_updated INTEGER DEFAULT 0,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'processing', 'complete', 'failed'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_org_users_role ON org_users(role);
CREATE INDEX IF NOT EXISTS idx_org_users_reports_to ON org_users(reports_to_id);
CREATE INDEX IF NOT EXISTS idx_org_users_email ON org_users(email);
CREATE INDEX IF NOT EXISTS idx_org_users_department ON org_users(department);
CREATE INDEX IF NOT EXISTS idx_org_imports_status ON org_imports(status);
CREATE INDEX IF NOT EXISTS idx_org_imports_uploaded_at ON org_imports(uploaded_at DESC);

-- Auto-update trigger for org_users.updated_at
CREATE OR REPLACE FUNCTION update_org_users_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER org_users_updated_at
    BEFORE UPDATE ON org_users
    FOR EACH ROW
    EXECUTE FUNCTION update_org_users_updated_at();

-- Table comments
COMMENT ON TABLE org_users IS 'Organization chart user hierarchy';
COMMENT ON COLUMN org_users.user_id IS 'Unique user identifier (from HR system)';
COMMENT ON COLUMN org_users.reports_to_id IS 'Manager user_id (NULL for CEO/root)';
COMMENT ON COLUMN org_users.role IS 'Profile role (finance, manager, analyst, etc)';
COMMENT ON COLUMN org_users.email IS 'User email (unique)';
COMMENT ON COLUMN org_users.department IS 'Department/team name (e.g., "Finance", "Marketing", "Engineering")';

COMMENT ON TABLE org_imports IS 'CSV import history';
COMMENT ON COLUMN org_imports.filename IS 'Original CSV filename';
COMMENT ON COLUMN org_imports.uploaded_by IS 'Admin email who uploaded';
COMMENT ON COLUMN org_imports.users_created IS 'Number of new users created';
COMMENT ON COLUMN org_imports.users_updated IS 'Number of existing users updated';
COMMENT ON COLUMN org_imports.status IS 'Import status (pending/processing/complete/failed)';
