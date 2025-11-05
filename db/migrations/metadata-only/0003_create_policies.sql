-- Phase 5 Workstream C: RBAC/ABAC Policy Storage
-- Migration: 0003_create_policies
-- Purpose: Create policies table for role-based tool access control
-- Dependencies: 0002_create_profiles (profiles table must exist for FK)

-- Policies table: Defines which tools each role can use
CREATE TABLE IF NOT EXISTS policies (
    id SERIAL PRIMARY KEY,
    role VARCHAR(50) NOT NULL,  -- FK to profiles.role (e.g., 'finance', 'manager')
    tool_pattern VARCHAR(200) NOT NULL,  -- Tool pattern (exact match or glob: 'github__*')
    allow BOOLEAN NOT NULL DEFAULT FALSE,  -- true = allow, false = deny
    conditions JSONB,  -- ABAC conditions (e.g., {"database": "analytics_*"})
    reason TEXT,  -- Human-readable reason for deny policies
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_policies_role ON policies(role);
CREATE INDEX IF NOT EXISTS idx_policies_role_tool ON policies(role, tool_pattern);
CREATE INDEX IF NOT EXISTS idx_policies_tool ON policies(tool_pattern);

-- Comments for documentation
COMMENT ON TABLE policies IS 'RBAC/ABAC policies for role-based tool access control (Phase 5 Workstream C)';
COMMENT ON COLUMN policies.id IS 'Auto-incrementing policy identifier';
COMMENT ON COLUMN policies.role IS 'Role name (FK to profiles.role)';
COMMENT ON COLUMN policies.tool_pattern IS 'Tool name pattern (exact: developer__shell, glob: github__*)';
COMMENT ON COLUMN policies.allow IS 'true = allow tool usage, false = deny';
COMMENT ON COLUMN policies.conditions IS 'JSONB conditions for attribute-based access (e.g., database patterns)';
COMMENT ON COLUMN policies.reason IS 'Human-readable explanation for deny policies';
COMMENT ON COLUMN policies.created_at IS 'Policy creation timestamp';
COMMENT ON COLUMN policies.updated_at IS 'Last policy update timestamp';

-- Auto-update trigger for updated_at
CREATE OR REPLACE FUNCTION update_policies_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_policies_updated_at
    BEFORE UPDATE ON policies
    FOR EACH ROW
    EXECUTE FUNCTION update_policies_updated_at();

-- Verification
DO $$
BEGIN
    RAISE NOTICE '=== Policy Table Created ===';
    RAISE NOTICE 'Table: policies';
    RAISE NOTICE 'Indexes: idx_policies_role, idx_policies_role_tool, idx_policies_tool';
    RAISE NOTICE 'Trigger: trg_update_policies_updated_at';
    RAISE NOTICE 'Ready for seed data (seeds/policies.sql)';
END $$;
