use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Session status enum
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "varchar")]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "active")]
    Active,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
    #[sqlx(rename = "expired")]
    Expired,
}

/// Session model - maps to `sessions` table
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Session {
    pub id: Uuid,
    pub role: String,
    pub task_id: Option<Uuid>,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Request to create a new session
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    pub role: String,
    #[serde(default)]
    pub task_id: Option<Uuid>,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

/// Request to update an existing session
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSessionRequest {
    #[serde(default)]
    pub task_id: Option<Uuid>,
    #[serde(default)]
    pub status: Option<SessionStatus>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Session list response with pagination
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionListResponse {
    pub sessions: Vec<Session>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_serialization() {
        let status = SessionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
    }

    #[test]
    fn test_create_session_request_defaults() {
        let json = r#"{"role": "pm"}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.role, "pm");
        assert_eq!(req.task_id, None);
        assert_eq!(req.metadata, serde_json::json!({}));
    }
}
