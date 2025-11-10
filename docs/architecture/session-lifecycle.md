# Session Lifecycle FSM

**Phase 6 Workstream A - Lifecycle Integration**  
**Date:** 2025-11-10  
**Status:** Implemented & Tested

## Overview

The Session Lifecycle Finite State Machine (FSM) manages the state transitions of user sessions throughout their lifecycle. This document describes the states, transitions, and implementation details.

## State Machine Diagram

```
                                    ┌──────────┐
                                    │          │
                        ┌───────────▶  PENDING │
                        │           │          │
                        │           └─────┬────┘
                        │                 │
                        │              activate
                        │                 │
                        │                 ▼
                   ┌────┴─────┐      ┌────────┐
                   │          │◀─────┤        │
                   │ EXPIRED  │      │ ACTIVE │
                   │          │      │        │
                   └──────────┘      └───┬─┬──┘
                        ▲                │ │
                        │                │ │ pause
                        │                │ │
                        │    ┌───────────┘ │
                        │    │             ▼
                        │    │       ┌──────────┐
                        │    │       │          │
                        │    │       │  PAUSED  │────┐
                        │    │       │          │    │ resume
                        │    │       └──────────┘◀───┘
                        │    │             │
                        │    │         expire
                        │    │             │
                        │    │             ▼
                        │    │       ┌───────────┐
                        │    │       │  EXPIRED  │
                        │    │       └───────────┘
                        │    │
                        │    │ complete
                        │    │
                        │    ▼
                  ┌───────────┐
                  │           │
                  │ COMPLETED │
                  │           │
                  └───────────┘
                        ▲
                        │
                     fail
                        │
                  ┌─────┴─────┐
                  │           │
                  │  FAILED   │
                  │           │
                  └───────────┘
```

## States

### Active States

| State      | Description                                  | Can Transition To           |
|------------|----------------------------------------------|-----------------------------|
| `PENDING`  | Session created, waiting for activation      | active, expired             |
| `ACTIVE`   | Session is actively running a task           | paused, completed, failed, expired |
| `PAUSED`   | Session temporarily suspended by user        | active (resume), expired    |

### Terminal States

| State       | Description                                 | Can Transition To |
|-------------|---------------------------------------------|-------------------|
| `COMPLETED` | Session finished successfully               | *none*            |
| `FAILED`    | Session encountered an error and terminated | *none*            |
| `EXPIRED`   | Session timed out or exceeded retention     | *none*            |

**Note:** Terminal states cannot transition to any other state. Once a session reaches a terminal state, it is immutable.

## Transitions

### Valid Transitions

| From       | Event      | To         | Notes                                    |
|------------|------------|------------|------------------------------------------|
| PENDING    | activate   | ACTIVE     | User starts working on a task            |
| PENDING    | *timeout*  | EXPIRED    | Session creation timeout                 |
| ACTIVE     | pause      | PAUSED     | User temporarily suspends work           |
| ACTIVE     | complete   | COMPLETED  | Task successfully finished               |
| ACTIVE     | fail       | FAILED     | Task encountered unrecoverable error     |
| ACTIVE     | *timeout*  | EXPIRED    | Inactivity timeout                       |
| PAUSED     | resume     | ACTIVE     | User resumes work                        |
| PAUSED     | *timeout*  | EXPIRED    | Paused too long (exceeds retention)      |

### Invalid Transitions

- **Terminal State Exits:** COMPLETED, FAILED, and EXPIRED cannot transition to any state
- **Skip States:** PENDING cannot directly go to COMPLETED/FAILED/PAUSED
- **Reverse Flow:** COMPLETED/FAILED cannot return to ACTIVE/PENDING

Any attempt to perform an invalid transition results in a `TransitionError::InvalidTransition` error.

## Database Schema

### Session Table Columns

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    role VARCHAR(255) NOT NULL,
    task_id UUID,
    status VARCHAR(50) NOT NULL 
        CHECK (status IN ('pending', 'active', 'paused', 'completed', 'failed', 'expired')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB DEFAULT '{}',
    
    -- Phase 6 A.2: FSM-specific columns
    fsm_metadata JSONB DEFAULT '{}',
    last_transition_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    paused_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    failed_at TIMESTAMP WITH TIME ZONE
);
```

### Timestamp Behavior

| Column               | Updated When                              | Cleared When                |
|----------------------|-------------------------------------------|-----------------------------|
| `updated_at`         | Every UPDATE operation                    | Never                       |
| `last_transition_at` | Every state transition                    | Never                       |
| `paused_at`          | Transition to PAUSED                      | Resume to ACTIVE            |
| `completed_at`       | Transition to COMPLETED                   | Never (terminal)            |
| `failed_at`          | Transition to FAILED                      | Never (terminal)            |

## API Endpoints

### Trigger Lifecycle Event

**Endpoint:** `PUT /sessions/{id}/events`

**Request Body:**
```json
{
  "event": "activate" | "pause" | "resume" | "complete" | "fail"
}
```

**Response:**
```json
{
  "session_id": "uuid",
  "agent_role": "pm",
  "state": "active",
  "metadata": {}
}
```

**Error Responses:**
- `400 Bad Request` - Invalid event or transition
- `404 Not Found` - Session does not exist
- `503 Service Unavailable` - SessionLifecycle not configured

### Examples

**Activate a pending session:**
```bash
curl -X PUT http://localhost:8088/sessions/{id}/events \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "activate"}'
```

**Pause an active session:**
```bash
curl -X PUT http://localhost:8088/sessions/{id}/events \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "pause"}'
```

**Resume a paused session:**
```bash
curl -X PUT http://localhost:8088/sessions/{id}/events \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "resume"}'
```

**Complete an active session:**
```bash
curl -X PUT http://localhost:8088/sessions/{id}/events \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d '{"event": "complete"}'
```

## Implementation

### Code Structure

**FSM Core:** `src/lifecycle/session_lifecycle.rs`
```rust
pub struct SessionLifecycle {
    repo: SessionRepository,
    retention_days: i32,
}

impl SessionLifecycle {
    pub async fn activate(&self, session_id: Uuid) -> Result<Session, TransitionError>;
    pub async fn pause(&self, session_id: Uuid) -> Result<Session, TransitionError>;
    pub async fn resume(&self, session_id: Uuid) -> Result<Session, TransitionError>;
    pub async fn complete(&self, session_id: Uuid) -> Result<Session, TransitionError>;
    pub async fn fail(&self, session_id: Uuid) -> Result<Session, TransitionError>;
    
    async fn transition(&self, session_id: Uuid, new_status: SessionStatus) 
        -> Result<Session, TransitionError>;
}
```

**Validation Function:**
```rust
fn is_valid_transition(from: &SessionStatus, to: &SessionStatus) -> bool {
    use SessionStatus::*;
    
    match (from, to) {
        // Pending transitions
        (Pending, Active) => true,
        (Pending, Expired) => true,
        
        // Active transitions
        (Active, Paused) => true,
        (Active, Completed) => true,
        (Active, Failed) => true,
        (Active, Expired) => true,
        
        // Paused transitions
        (Paused, Active) => true,
        (Paused, Expired) => true,
        
        // Terminal states cannot transition
        (Completed, _) => false,
        (Failed, _) => false,
        (Expired, _) => false,
        
        // Same state is a no-op (allowed)
        (a, b) if std::mem::discriminant(a) == std::mem::discriminant(b) => true,
        
        // All other transitions are invalid
        _ => false,
    }
}
```

### Initialization

The SessionLifecycle is initialized in `main.rs` after database connection:

```rust
let retention_days = std::env::var("SESSION_RETENTION_DAYS")
    .ok()
    .and_then(|s| s.parse::<i32>().ok())
    .unwrap_or(30);

let session_lifecycle = SessionLifecycle::new(pool.clone(), retention_days);
app_state = app_state.with_session_lifecycle(session_lifecycle);
```

**Configuration:**
- `SESSION_RETENTION_DAYS` env var (default: 30 days)
- After retention period, inactive sessions are expired

## Testing

### Test Coverage

**Unit Tests:** `src/lifecycle/session_lifecycle.rs::tests`
- All valid transitions verified
- All invalid transitions rejected
- Terminal state protection
- Same-state no-op behavior

**Integration Tests:** `tests/integration/test_session_lifecycle_comprehensive.sh`
- 8 test scenarios
- 17 assertions
- All passing ✓

### Test Scenarios

1. **Create session → PENDING state**
   - Session created with `fsm_metadata = {"initial_state": "pending"}`
   - Verifies initial state is correct

2. **Activate → ACTIVE state**
   - PENDING → ACTIVE transition
   - Verifies `last_transition_at` updated

3. **Pause → PAUSED state**
   - ACTIVE → PAUSED transition
   - Verifies `paused_at` timestamp set

4. **Resume → ACTIVE state**
   - PAUSED → ACTIVE transition
   - Verifies `paused_at` timestamp cleared

5. **Complete → COMPLETED state**
   - ACTIVE → COMPLETED transition
   - Verifies `completed_at` timestamp set
   - Verifies terminal state protection (cannot transition from COMPLETED)

6. **Session persistence across Controller restart**
   - Restarts controller
   - Verifies session state persists in database

7. **Concurrent sessions**
   - Creates multiple sessions for same user
   - Verifies they can be independently activated/managed

8. **Session timeout simulation**
   - Verifies session can be expired
   - Notes: Full timeout testing requires background task

### Running Tests

```bash
# Run comprehensive integration tests
./tests/integration/test_session_lifecycle_comprehensive.sh

# Expected output:
# PASSED: 17
# FAILED: 0
# ✓ ALL TESTS PASSED
```

## Background Tasks

### Session Expiration

The `expire_old_sessions()` method is available for periodic cleanup:

```rust
pub async fn expire_old_sessions(&self) -> Result<u64, String> {
    // Expires sessions older than retention_days
    // Returns count of expired sessions
}
```

**Future Implementation:**
- Background cron job to call `expire_old_sessions()` every hour
- Expires sessions in PENDING or ACTIVE state older than retention period
- Logs expired session count

## Migration History

**Migration 0007:** `db/migrations/metadata-only/0007_update_sessions_for_lifecycle.sql`
- Added `fsm_metadata` JSONB column
- Added `last_transition_at`, `paused_at`, `completed_at`, `failed_at` timestamp columns
- Updated `sessions_status_check` constraint to include 'paused'
- Created indexes for efficient querying:
  - `idx_sessions_last_transition` - Recent activity queries
  - `idx_sessions_role_status` - Role + status filtering
  - `idx_sessions_paused` - Finding paused sessions

## References

- **Implementation:** `src/lifecycle/session_lifecycle.rs`
- **Routes:** `src/controller/src/routes/sessions.rs`
- **Models:** `src/controller/src/models/session.rs`
- **Repository:** `src/controller/src/repository/session_repo.rs`
- **Migration:** `db/migrations/metadata-only/0007_update_sessions_for_lifecycle.sql`
- **Tests:** `tests/integration/test_session_lifecycle_comprehensive.sh`

---

**Last Updated:** 2025-11-10  
**Phase:** Phase 6 Workstream A  
**Status:** ✅ Implemented & Tested
