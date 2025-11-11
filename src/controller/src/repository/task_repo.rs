use sqlx::{PgPool, Result};
use sqlx::types::Uuid;
use crate::models::{Task, CreateTaskRequest};

/// Task repository for database operations
pub struct TaskRepository {
    pool: PgPool,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new task
    pub async fn create(&self, req: &CreateTaskRequest) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (
                task_type, description, data, source, target,
                status, context, trace_id, idempotency_key
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, task_type, description, data, source, target, status,
                      context, trace_id, idempotency_key, created_at, updated_at, completed_at
            "#,
        )
        .bind(&req.task_type)
        .bind(&req.description)
        .bind(req.data.as_ref().unwrap_or(&serde_json::json!({})))
        .bind(&req.source)
        .bind(&req.target)
        .bind("pending")  // Default status
        .bind(req.context.as_ref().unwrap_or(&serde_json::json!({})))
        .bind(req.trace_id)
        .bind(req.idempotency_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    /// Get a task by ID
    pub async fn get(&self, id: Uuid) -> Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, description, data, source, target, status,
                   context, trace_id, idempotency_key, created_at, updated_at, completed_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    /// List tasks for a specific target role
    pub async fn list_by_target(&self, target: &str, limit: i64) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, description, data, source, target, status,
                   context, trace_id, idempotency_key, created_at, updated_at, completed_at
            FROM tasks
            WHERE target = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(target)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// List pending tasks for a specific target role
    pub async fn list_pending(&self, target: &str, limit: i64) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, description, data, source, target, status,
                   context, trace_id, idempotency_key, created_at, updated_at, completed_at
            FROM tasks
            WHERE target = $1 AND status = 'pending'
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(target)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    /// Update task status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
            SET status = $2,
                completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END
            WHERE id = $1
            RETURNING id, task_type, description, data, source, target, status,
                      context, trace_id, idempotency_key, created_at, updated_at, completed_at
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    /// Check if a task with the given idempotency key already exists
    pub async fn find_by_idempotency_key(&self, key: Uuid) -> Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, description, data, source, target, status,
                   context, trace_id, idempotency_key, created_at, updated_at, completed_at
            FROM tasks
            WHERE idempotency_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }
}
