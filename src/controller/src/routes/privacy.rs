// Privacy audit endpoints

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::AppState;

/// Privacy audit log entry (metadata only - no content)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditLogEntry {
    /// Session ID
    pub session_id: String,
    /// Number of PII redactions
    pub redaction_count: usize,
    /// PII categories detected (e.g., ["SSN", "EMAIL"])
    pub categories: Vec<String>,
    /// Privacy mode used (Rules/NER/Hybrid)
    pub mode: String,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
}

/// Response after audit log submission
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditLogResponse {
    /// Status message
    pub status: String,
    /// Audit log ID
    pub id: i64,
}

/// Submit privacy audit log (metadata only)
///
/// Accepts audit log from Privacy Guard MCP extension.
/// Stores metadata only - never logs prompt/response content.
#[utoipa::path(
    post,
    path = "/privacy/audit",
    request_body = AuditLogEntry,
    responses(
        (status = 201, description = "Audit log created", body = AuditLogResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Database error")
    ),
    tag = "privacy"
)]
pub async fn submit_audit_log(
    State(state): State<AppState>,
    Json(entry): Json<AuditLogEntry>,
) -> Result<(StatusCode, Json<AuditLogResponse>), (StatusCode, String)> {
    // Validate input
    if entry.session_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "session_id is required".to_string()));
    }

    // Get database pool
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        error!("Database not configured");
        (StatusCode::INTERNAL_SERVER_ERROR, "Database not available".to_string())
    })?;

    // Insert audit log into database
    let result = sqlx::query(
        r#"
        INSERT INTO privacy_audit_logs (session_id, redaction_count, categories, mode, timestamp)
        VALUES ($1, $2, $3, $4, to_timestamp($5))
        RETURNING id
        "#
    )
    .bind(&entry.session_id)
    .bind(entry.redaction_count as i32)
    .bind(&entry.categories)
    .bind(&entry.mode)
    .bind(entry.timestamp)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => {
            let id: i64 = row.try_get("id")
                .map_err(|e| {
                    error!("Failed to get inserted ID: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve audit log ID".to_string())
                })?;

            info!(
                "Privacy audit log created: session={}, redactions={}, categories={:?}",
                entry.session_id, entry.redaction_count, entry.categories
            );

            Ok((
                StatusCode::CREATED,
                Json(AuditLogResponse {
                    status: "created".to_string(),
                    id,
                }),
            ))
        }
        Err(e) => {
            error!("Failed to insert audit log: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            session_id: "test-session-123".to_string(),
            redaction_count: 5,
            categories: vec!["SSN".to_string(), "EMAIL".to_string()],
            mode: "Hybrid".to_string(),
            timestamp: 1699564800,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test-session-123"));
        assert!(json.contains("\"redaction_count\":5"));
        assert!(json.contains("SSN"));
        assert!(json.contains("EMAIL"));
    }

    #[test]
    fn test_audit_log_response_serialization() {
        let response = AuditLogResponse {
            status: "created".to_string(),
            id: 42,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"created\""));
        assert!(json.contains("\"id\":42"));
    }
}
