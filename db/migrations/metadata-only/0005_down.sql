-- Rollback Migration: Drop privacy_audit_logs table
-- Description: Removes Privacy Guard MCP audit logs table
-- Phase: 5 Workstream E (E5)
-- Date: 2025-11-06

-- Drop indexes first
DROP INDEX IF EXISTS idx_privacy_audit_logs_created_at;
DROP INDEX IF EXISTS idx_privacy_audit_logs_mode;
DROP INDEX IF EXISTS idx_privacy_audit_logs_timestamp;
DROP INDEX IF EXISTS idx_privacy_audit_logs_session_id;

-- Drop table
DROP TABLE IF EXISTS privacy_audit_logs;

-- Verification (should return no rows)
SELECT table_name 
FROM information_schema.tables 
WHERE table_name = 'privacy_audit_logs';
