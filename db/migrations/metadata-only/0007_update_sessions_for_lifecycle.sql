-- Migration 0007: Update sessions table for Session Lifecycle FSM
-- Phase 6 Workstream A: Task A.2 - Database Persistence
-- Date: 2025-11-10
-- Purpose: Add FSM-specific columns for session lifecycle state tracking + 'paused' status

-- Update CHECK constraint to include 'paused' status
ALTER TABLE sessions DROP CONSTRAINT IF EXISTS sessions_status_check;
ALTER TABLE sessions ADD CONSTRAINT sessions_status_check 
  CHECK (status IN ('pending', 'active', 'paused', 'completed', 'failed', 'expired'));

-- Add FSM metadata column (stores transition history, pause reasons, etc.)
ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS fsm_metadata JSONB DEFAULT '{}'::jsonb;

-- Add last transition timestamp (tracks when last state change occurred)
ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS last_transition_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Add paused timestamp (tracks when session was paused, NULL if not paused)
ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS paused_at TIMESTAMP WITH TIME ZONE;

-- Add completed timestamp (tracks when session completed, NULL if not completed)
ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS completed_at TIMESTAMP WITH TIME ZONE;

-- Add failed timestamp (tracks when session failed, NULL if not failed)
ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS failed_at TIMESTAMP WITH TIME ZONE;

-- Create index on last_transition_at for efficient querying by recent activity
CREATE INDEX IF NOT EXISTS idx_sessions_last_transition 
ON sessions (last_transition_at DESC);

-- Create composite index on (role, status) for filtering sessions by role and state
CREATE INDEX IF NOT EXISTS idx_sessions_role_status 
ON sessions (role, status);

-- Create index on paused_at for finding paused sessions
CREATE INDEX IF NOT EXISTS idx_sessions_paused 
ON sessions (paused_at) 
WHERE paused_at IS NOT NULL;

-- Add comment to document the purpose of new columns
COMMENT ON COLUMN sessions.fsm_metadata IS 'FSM-specific metadata: transition history, pause reasons, error details, etc.';
COMMENT ON COLUMN sessions.last_transition_at IS 'Timestamp of last state transition (updated on every activate/pause/complete/fail event)';
COMMENT ON COLUMN sessions.paused_at IS 'Timestamp when session was paused (NULL if not paused or resumed)';
COMMENT ON COLUMN sessions.completed_at IS 'Timestamp when session completed successfully (NULL if not completed)';
COMMENT ON COLUMN sessions.failed_at IS 'Timestamp when session failed (NULL if not failed)';

-- Migration verification
DO $$
BEGIN
    -- Check that all new columns exist
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'fsm_metadata'
    ) THEN
        RAISE EXCEPTION 'Migration 0007 failed: fsm_metadata column not created';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sessions' AND column_name = 'last_transition_at'
    ) THEN
        RAISE EXCEPTION 'Migration 0007 failed: last_transition_at column not created';
    END IF;

    RAISE NOTICE 'Migration 0007 completed successfully';
END $$;
