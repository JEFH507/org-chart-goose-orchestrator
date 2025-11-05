-- Migration: 001_create_schema
-- Description: Initial database schema for session persistence, task routing, approvals, and audit logging
-- Phase: 4 (Storage/Metadata + Session Persistence)
-- Version: v0.4.0
-- Created: 2025-11-05

-- ============================================================================
-- Table: sessions
-- Purpose: Stores agent session state for cross-agent workflows
-- ============================================================================

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    role VARCHAR(50) NOT NULL,
    task_id UUID,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'active', 'completed', 'failed', 'expired')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for sessions table
CREATE INDEX idx_sessions_task_id ON sessions(task_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);

COMMENT ON TABLE sessions IS 'Agent session state for cross-agent workflows';
COMMENT ON COLUMN sessions.id IS 'Session identifier (UUID v4)';
COMMENT ON COLUMN sessions.role IS 'Agent role (Finance, Manager, etc.)';
COMMENT ON COLUMN sessions.task_id IS 'Associated task (optional FK to tasks.id)';
COMMENT ON COLUMN sessions.status IS 'Session status: pending/active/completed/failed/expired';
COMMENT ON COLUMN sessions.metadata IS 'Flexible context storage (JSONB)';

-- ============================================================================
-- Table: tasks
-- Purpose: Stores cross-agent task routing information
-- ============================================================================

CREATE TABLE tasks (
    id UUID PRIMARY KEY,
    task_type VARCHAR(50) NOT NULL CHECK (task_type IN ('notification', 'approval', 'routing')),
    description TEXT NOT NULL,
    from_role VARCHAR(50) NOT NULL,
    to_role VARCHAR(50) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    trace_id UUID NOT NULL,
    idempotency_key UUID UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for tasks table
CREATE UNIQUE INDEX idx_tasks_idempotency_key ON tasks(idempotency_key);
CREATE INDEX idx_tasks_trace_id ON tasks(trace_id);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);

COMMENT ON TABLE tasks IS 'Cross-agent task routing information';
COMMENT ON COLUMN tasks.id IS 'Task identifier (UUID v4)';
COMMENT ON COLUMN tasks.task_type IS 'Task category: notification/approval/routing';
COMMENT ON COLUMN tasks.description IS 'Human-readable summary';
COMMENT ON COLUMN tasks.from_role IS 'Source agent role';
COMMENT ON COLUMN tasks.to_role IS 'Target agent role';
COMMENT ON COLUMN tasks.data IS 'Task payload (arbitrary JSON)';
COMMENT ON COLUMN tasks.trace_id IS 'Distributed tracing ID';
COMMENT ON COLUMN tasks.idempotency_key IS 'Deduplication key (must be unique)';

-- ============================================================================
-- Table: approvals
-- Purpose: Stores approval workflow state
-- ============================================================================

CREATE TABLE approvals (
    id UUID PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    approver_role VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'approved', 'rejected')),
    decision_at TIMESTAMP WITH TIME ZONE,
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for approvals table
CREATE INDEX idx_approvals_task_id ON approvals(task_id);
CREATE INDEX idx_approvals_status ON approvals(status);

COMMENT ON TABLE approvals IS 'Approval workflow state';
COMMENT ON COLUMN approvals.id IS 'Approval identifier (UUID v4)';
COMMENT ON COLUMN approvals.task_id IS 'Associated task (FK to tasks.id)';
COMMENT ON COLUMN approvals.approver_role IS 'Role that approved/rejected';
COMMENT ON COLUMN approvals.status IS 'Approval status: pending/approved/rejected';
COMMENT ON COLUMN approvals.decision_at IS 'When decision was made (NULL if pending)';
COMMENT ON COLUMN approvals.notes IS 'Approval comments/rationale';

-- ============================================================================
-- Table: audit_events
-- Purpose: Stores audit trail for compliance and debugging
-- ============================================================================

CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    role VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    trace_id UUID NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for audit_events table
CREATE INDEX idx_audit_events_trace_id ON audit_events(trace_id);
CREATE INDEX idx_audit_events_timestamp ON audit_events(timestamp DESC);
CREATE INDEX idx_audit_events_event_type ON audit_events(event_type);

COMMENT ON TABLE audit_events IS 'Audit trail for compliance and debugging';
COMMENT ON COLUMN audit_events.id IS 'Event identifier (UUID v4)';
COMMENT ON COLUMN audit_events.event_type IS 'Event category (e.g., task_routed, approval_requested)';
COMMENT ON COLUMN audit_events.role IS 'Agent role that triggered event';
COMMENT ON COLUMN audit_events.timestamp IS 'Event timestamp';
COMMENT ON COLUMN audit_events.trace_id IS 'Links to distributed trace';
COMMENT ON COLUMN audit_events.metadata IS 'Event-specific data (JSONB)';

-- ============================================================================
-- Views (optional utility views)
-- ============================================================================

-- View: active_sessions
-- Purpose: Quick lookup of active sessions
CREATE VIEW active_sessions AS
SELECT id, role, task_id, created_at, updated_at
FROM sessions
WHERE status = 'active'
ORDER BY updated_at DESC;

COMMENT ON VIEW active_sessions IS 'Quick lookup of active sessions';

-- View: pending_approvals
-- Purpose: Quick lookup of pending approvals
CREATE VIEW pending_approvals AS
SELECT a.id, a.task_id, a.approver_role, a.created_at, t.description, t.from_role, t.to_role
FROM approvals a
JOIN tasks t ON a.task_id = t.id
WHERE a.status = 'pending'
ORDER BY a.created_at ASC;

COMMENT ON VIEW pending_approvals IS 'Quick lookup of pending approvals with task context';

-- ============================================================================
-- Schema Verification
-- ============================================================================

-- Verify all tables exist
DO $$
DECLARE
    table_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
    AND table_name IN ('sessions', 'tasks', 'approvals', 'audit_events');
    
    IF table_count < 4 THEN
        RAISE EXCEPTION 'Schema creation failed: expected 4 tables, found %', table_count;
    END IF;
    
    RAISE NOTICE 'Schema migration 001 completed successfully: % tables created', table_count;
END $$;
