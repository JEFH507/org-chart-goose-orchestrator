use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::{Session, SessionStatus, UpdateSessionRequest};
use crate::repository::SessionRepository;

/// Session lifecycle manager
///
/// Manages session state transitions and auto-expiration.
pub struct SessionLifecycle {
    repo: SessionRepository,
    retention_days: i32,
}

impl SessionLifecycle {
    pub fn new(pool: PgPool, retention_days: i32) -> Self {
        Self {
            repo: SessionRepository::new(pool),
            retention_days,
        }
    }

    /// Transition a session from one state to another
    ///
    /// State machine:
    /// - pending → active (session starts)
    /// - active → completed (session finishes successfully)
    /// - active → failed (session encounters error)
    /// - pending/active → expired (session timeout)
    ///
    /// Invalid transitions are rejected.
    pub async fn transition(
        &self,
        session_id: Uuid,
        new_status: SessionStatus,
    ) -> Result<Session, TransitionError> {
        // Get current session
        let session = self
            .repo
            .get(session_id)
            .await
            .map_err(|e| TransitionError::DatabaseError(e.to_string()))?
            .ok_or(TransitionError::SessionNotFound(session_id))?;

        // Validate transition
        if !is_valid_transition(&session.status, &new_status) {
            return Err(TransitionError::InvalidTransition {
                from: session.status.clone(),
                to: new_status,
            });
        }

        // Perform update
        let req = UpdateSessionRequest {
            task_id: None,
            status: Some(new_status.clone()),
            metadata: None,
        };

        let updated = self
            .repo
            .update(session_id, req)
            .await
            .map_err(|e| TransitionError::DatabaseError(e.to_string()))?
            .ok_or(TransitionError::SessionNotFound(session_id))?;

        info!(
            message = "session.transition",
            session_id = %session_id,
            from = ?session.status,
            to = ?new_status
        );

        Ok(updated)
    }

    /// Expire old sessions
    ///
    /// Runs periodically to expire sessions older than retention_days.
    /// Returns the number of sessions expired.
    pub async fn expire_old_sessions(&self) -> Result<u64, String> {
        match self.repo.expire_old_sessions(self.retention_days).await {
            Ok(count) => {
                if count > 0 {
                    info!(
                        message = "sessions.expired",
                        count = count,
                        retention_days = self.retention_days
                    );
                }
                Ok(count)
            }
            Err(e) => {
                warn!(message = "sessions.expire.error", error = %e);
                Err(e.to_string())
            }
        }
    }

    /// Check if a session can transition to active state
    pub async fn can_activate(&self, session_id: Uuid) -> Result<bool, String> {
        let session = self
            .repo
            .get(session_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        Ok(matches!(session.status, SessionStatus::Pending))
    }

    /// Activate a pending session
    pub async fn activate(&self, session_id: Uuid) -> Result<Session, TransitionError> {
        self.transition(session_id, SessionStatus::Active).await
    }

    /// Complete an active session
    pub async fn complete(&self, session_id: Uuid) -> Result<Session, TransitionError> {
        self.transition(session_id, SessionStatus::Completed).await
    }

    /// Fail an active session
    pub async fn fail(&self, session_id: Uuid) -> Result<Session, TransitionError> {
        self.transition(session_id, SessionStatus::Failed).await
    }

    /// Pause an active session
    pub async fn pause(&self, session_id: Uuid) -> Result<Session, TransitionError> {
        self.transition(session_id, SessionStatus::Paused).await
    }

    /// Resume a paused session
    pub async fn resume(&self, session_id: Uuid) -> Result<Session, TransitionError> {
        self.transition(session_id, SessionStatus::Active).await
    }
}

/// Validate session state transition
fn is_valid_transition(from: &SessionStatus, to: &SessionStatus) -> bool {
    use SessionStatus::*;

    match (from, to) {
        // Pending can transition to active or expired
        (Pending, Active) => true,
        (Pending, Expired) => true,

        // Active can transition to paused, completed, failed, or expired
        (Active, Paused) => true,
        (Active, Completed) => true,
        (Active, Failed) => true,
        (Active, Expired) => true,

        // Paused can transition to active (resume) or expired (timeout)
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

#[derive(Debug, thiserror::Error)]
pub enum TransitionError {
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: SessionStatus,
        to: SessionStatus,
    },

    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        use SessionStatus::*;

        // Pending transitions
        assert!(is_valid_transition(&Pending, &Active));
        assert!(is_valid_transition(&Pending, &Expired));
        assert!(!is_valid_transition(&Pending, &Completed));
        assert!(!is_valid_transition(&Pending, &Failed));
        assert!(!is_valid_transition(&Pending, &Paused));

        // Active transitions
        assert!(is_valid_transition(&Active, &Paused));
        assert!(is_valid_transition(&Active, &Completed));
        assert!(is_valid_transition(&Active, &Failed));
        assert!(is_valid_transition(&Active, &Expired));
        assert!(!is_valid_transition(&Active, &Pending));

        // Paused transitions
        assert!(is_valid_transition(&Paused, &Active));
        assert!(is_valid_transition(&Paused, &Expired));
        assert!(!is_valid_transition(&Paused, &Completed));
        assert!(!is_valid_transition(&Paused, &Failed));
        assert!(!is_valid_transition(&Paused, &Pending));

        // Terminal states cannot transition
        assert!(!is_valid_transition(&Completed, &Failed));
        assert!(!is_valid_transition(&Failed, &Completed));
        assert!(!is_valid_transition(&Expired, &Active));

        // Same state (no-op)
        assert!(is_valid_transition(&Pending, &Pending));
        assert!(is_valid_transition(&Active, &Active));
        assert!(is_valid_transition(&Paused, &Paused));
    }
}
