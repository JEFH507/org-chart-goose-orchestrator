-- Migration: Create privacy_audit_logs table
-- Description: Store Privacy Guard MCP audit logs (metadata only - no content)
-- Phase: 5 Workstream E (E5)
-- Date: 2025-11-06

-- Privacy audit logs table (metadata only)
CREATE TABLE IF NOT EXISTS privacy_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    session_id VARCHAR(255) NOT NULL,
    redaction_count INTEGER NOT NULL DEFAULT 0,
    categories TEXT[] NOT NULL DEFAULT '{}',
    mode VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_privacy_audit_logs_session_id ON privacy_audit_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_privacy_audit_logs_timestamp ON privacy_audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_privacy_audit_logs_mode ON privacy_audit_logs(mode);
CREATE INDEX IF NOT EXISTS idx_privacy_audit_logs_created_at ON privacy_audit_logs(created_at DESC);

-- Table comment
COMMENT ON TABLE privacy_audit_logs IS 'Privacy Guard MCP audit logs (metadata only - never stores prompt/response content)';

-- Column comments
COMMENT ON COLUMN privacy_audit_logs.id IS 'Auto-incrementing primary key';
COMMENT ON COLUMN privacy_audit_logs.session_id IS 'Goose session identifier';
COMMENT ON COLUMN privacy_audit_logs.redaction_count IS 'Number of PII tokens generated';
COMMENT ON COLUMN privacy_audit_logs.categories IS 'PII categories detected (e.g., SSN, EMAIL)';
COMMENT ON COLUMN privacy_audit_logs.mode IS 'Privacy mode used (Rules/NER/Hybrid)';
COMMENT ON COLUMN privacy_audit_logs.timestamp IS 'Event timestamp from Privacy Guard';
COMMENT ON COLUMN privacy_audit_logs.created_at IS 'Record creation timestamp';

-- Verification query
SELECT 
    table_name,
    column_name,
    data_type,
    is_nullable
FROM information_schema.columns
WHERE table_name = 'privacy_audit_logs'
ORDER BY ordinal_position;

-- Verify indexes
SELECT 
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename = 'privacy_audit_logs'
ORDER BY indexname;
