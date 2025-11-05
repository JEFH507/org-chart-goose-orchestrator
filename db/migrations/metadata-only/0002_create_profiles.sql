-- Phase 5: Profile System
-- Create profiles table for role-based configuration

-- Profiles table: stores complete role configuration as JSONB
CREATE TABLE IF NOT EXISTS profiles (
    role VARCHAR(50) PRIMARY KEY,
    display_name VARCHAR(100) NOT NULL,
    data JSONB NOT NULL,  -- Full profile JSON (schema defined in src/profile/schema.rs)
    signature TEXT,       -- Vault HMAC signature (optional, for tamper protection)
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);

-- Index for display_name lookups (admin UI browsing)
CREATE INDEX IF NOT EXISTS idx_profiles_display_name ON profiles(display_name);

-- Index for JSON queries (if needed for advanced filtering)
-- Example: Find profiles with specific privacy mode
-- SELECT * FROM profiles WHERE data->>'privacy'->>'mode' = 'strict';
CREATE INDEX IF NOT EXISTS idx_profiles_data_privacy_mode ON profiles((data->'privacy'->>'mode'));

-- Trigger to update updated_at column on row updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_profiles_updated_at
    BEFORE UPDATE ON profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE profiles IS 'Role-based configuration profiles for zero-touch deployment';
COMMENT ON COLUMN profiles.role IS 'Unique role identifier (e.g., finance, manager, analyst)';
COMMENT ON COLUMN profiles.display_name IS 'Human-readable role name (e.g., Finance Team Agent)';
COMMENT ON COLUMN profiles.data IS 'Complete profile JSON (providers, extensions, goosehints, recipes, policies, privacy)';
COMMENT ON COLUMN profiles.signature IS 'Vault Transit HMAC signature for tamper protection (optional)';
COMMENT ON COLUMN profiles.created_at IS 'Profile creation timestamp';
COMMENT ON COLUMN profiles.updated_at IS 'Last profile update timestamp (auto-updated on changes)';
