use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

/// Task model - represents a task routed between agents
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Task {
    /// Unique task identifier
    pub id: Uuid,
    
    /// Task type (e.g., "budget_approval", "compliance_review")
    pub task_type: String,
    
    /// Task description
    pub description: Option<String>,
    
    /// Task-specific data (JSON)
    pub data: serde_json::Value,
    
    /// Role that created the task
    pub source: String,
    
    /// Role that should handle the task
    pub target: String,
    
    /// Current status
    pub status: String,
    
    /// Additional context (JSON)
    pub context: serde_json::Value,
    
    /// Distributed tracing identifier
    pub trace_id: Option<Uuid>,
    
    /// Idempotency key
    pub idempotency_key: Option<Uuid>,
    
    /// Task creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Task completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Request to create a new task
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub task_type: String,
    pub description: Option<String>,
    pub data: Option<serde_json::Value>,
    pub source: String,
    pub target: String,
    pub context: Option<serde_json::Value>,
    pub trace_id: Option<Uuid>,
    pub idempotency_key: Option<Uuid>,
}

/// Response after creating a task
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTaskResponse {
    pub id: Uuid,
    pub task_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
