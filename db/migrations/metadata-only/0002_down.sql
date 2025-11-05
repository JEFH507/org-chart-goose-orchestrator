-- Phase 5: Rollback profiles table migration

-- Drop trigger first (depends on function)
DROP TRIGGER IF EXISTS update_profiles_updated_at ON profiles;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_profiles_data_privacy_mode;
DROP INDEX IF EXISTS idx_profiles_display_name;

-- Drop table
DROP TABLE IF EXISTS profiles;
