use crate::models::{CreateSessionRequest, Session, SessionStatus, UpdateSessionRequest};
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

/// Session repository for database operations
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new session
    pub async fn create(&self, req: CreateSessionRequest) -> Result<Session> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let session = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, role, task_id, status, created_at, updated_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, role, task_id, status, created_at, updated_at, metadata
            "#,
        )
        .bind(id)
        .bind(&req.role)
        .bind(req.task_id)
        .bind(SessionStatus::Pending)
        .bind(now)
        .bind(now)
        .bind(&req.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    /// Get a session by ID
    pub async fn get(&self, id: Uuid) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, role, task_id, status, created_at, updated_at, metadata
            FROM sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    /// Update a session
    pub async fn update(&self, id: Uuid, req: UpdateSessionRequest) -> Result<Option<Session>> {
        // First, get the current session to merge updates
        let current = match self.get(id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let updated_task_id = req.task_id.or(current.task_id);
        let updated_status = req.status.unwrap_or(current.status);
        let updated_metadata = req.metadata.unwrap_or(current.metadata);

        let session = sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET task_id = $2,
                status = $3,
                metadata = $4,
                updated_at = $5
            WHERE id = $1
            RETURNING id, role, task_id, status, created_at, updated_at, metadata
            "#,
        )
        .bind(id)
        .bind(updated_task_id)
        .bind(updated_status)
        .bind(&updated_metadata)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(session))
    }

    /// List sessions with pagination
    pub async fn list(&self, page: i32, page_size: i32) -> Result<(Vec<Session>, i64)> {
        let offset = (page - 1) * page_size;

        // Get total count
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&self.pool)
            .await?;

        // Get paginated sessions
        let sessions = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, role, task_id, status, created_at, updated_at, metadata
            FROM sessions
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((sessions, total.0))
    }

    /// List active sessions (used by active_sessions view)
    pub async fn list_active(&self) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, role, task_id, status, created_at, updated_at, metadata
            FROM sessions
            WHERE status = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(SessionStatus::Active)
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions)
    }

    /// Expire old sessions (lifecycle management)
    pub async fn expire_old_sessions(&self, retention_days: i32) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);

        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET status = $1, updated_at = $2
            WHERE created_at < $3 AND status IN ($4, $5)
            "#,
        )
        .bind(SessionStatus::Expired)
        .bind(Utc::now())
        .bind(cutoff)
        .bind(SessionStatus::Pending)
        .bind(SessionStatus::Active)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // Note: These tests require a running Postgres instance
    // Run with: cargo test --features test-db
    // (We'll add proper test fixtures in B4)

    async fn setup_test_pool() -> PgPool {
        // This would connect to test database
        // For now, just a placeholder
        unimplemented!("Test pool setup requires test database")
    }

    #[tokio::test]
    #[ignore] // Ignore until test database is configured
    async fn test_create_session() {
        let pool = setup_test_pool().await;
        let repo = SessionRepository::new(pool);

        let req = CreateSessionRequest {
            role: "pm".to_string(),
            task_id: None,
            metadata: serde_json::json!({"test": true}),
        };

        let session = repo.create(req).await.unwrap();
        assert_eq!(session.role, "pm");
        assert_eq!(session.metadata["test"], true);
    }
}
