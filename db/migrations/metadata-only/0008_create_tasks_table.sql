-- Migration 0008: Create tasks table for Agent Mesh task persistence
-- Author: Phase 6 D.3
-- Date: 2025-11-11
-- Purpose: Store tasks routed between agents, enable fetch_status functionality

-- Create tasks table
CREATE TABLE IF NOT EXISTS tasks (
    -- Primary identification
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Task metadata
    task_type VARCHAR(50) NOT NULL,
    description TEXT,
    data JSONB DEFAULT '{}'::jsonb,
    
    -- Routing information
    source VARCHAR(50) NOT NULL,  -- Role that created the task (e.g., 'finance')
    target VARCHAR(50) NOT NULL,  -- Role that should handle the task (e.g., 'manager')
    
    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    
    -- Additional context
    context JSONB DEFAULT '{}'::jsonb,
    
    -- Tracing and idempotency
    trace_id UUID,
    idempotency_key UUID,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CHECK (status IN ('pending', 'active', 'completed', 'failed', 'cancelled'))
);

-- Create indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_tasks_target_status ON tasks(target, status);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_trace_id ON tasks(trace_id) WHERE trace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_idempotency ON tasks(idempotency_key) WHERE idempotency_key IS NOT NULL;

-- Add comments for documentation
COMMENT ON TABLE tasks IS 'Stores tasks routed between agents via Agent Mesh';
COMMENT ON COLUMN tasks.id IS 'Unique task identifier (UUID)';
COMMENT ON COLUMN tasks.task_type IS 'Type of task (e.g., budget_approval, compliance_review)';
COMMENT ON COLUMN tasks.description IS 'Human-readable task description';
COMMENT ON COLUMN tasks.data IS 'Task-specific data (JSON)';
COMMENT ON COLUMN tasks.source IS 'Role that created the task';
COMMENT ON COLUMN tasks.target IS 'Role that should handle the task';
COMMENT ON COLUMN tasks.status IS 'Current status: pending, active, completed, failed, cancelled';
COMMENT ON COLUMN tasks.context IS 'Additional context (parent task, priority, etc.)';
COMMENT ON COLUMN tasks.trace_id IS 'Distributed tracing identifier';
COMMENT ON COLUMN tasks.idempotency_key IS 'Idempotency key for duplicate detection';
COMMENT ON COLUMN tasks.created_at IS 'Task creation timestamp';
COMMENT ON COLUMN tasks.updated_at IS 'Last update timestamp';
COMMENT ON COLUMN tasks.completed_at IS 'Task completion timestamp';

-- Create function to automatically update updated_at
CREATE OR REPLACE FUNCTION update_tasks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to auto-update updated_at
CREATE TRIGGER tasks_updated_at_trigger
    BEFORE UPDATE ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_tasks_updated_at();

-- Verification query (commented out for production)
-- SELECT 
--     table_name,
--     column_name,
--     data_type,
--     is_nullable
-- FROM information_schema.columns
-- WHERE table_name = 'tasks'
-- ORDER BY ordinal_position;
