-- Phase 6: Add assigned_profile column to org_users
-- This allows admin to assign specific profiles to users independently of their role

ALTER TABLE org_users 
ADD COLUMN IF NOT EXISTS assigned_profile VARCHAR(50) REFERENCES profiles(role);

-- Index for performance
CREATE INDEX IF NOT EXISTS idx_org_users_assigned_profile ON org_users(assigned_profile);

-- Comment
COMMENT ON COLUMN org_users.assigned_profile IS 'Manually assigned profile (overrides role-based default)';
