use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::info;

use crate::AppState;

/// Request to submit an approval decision
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmitApprovalRequest {
    /// Task ID being approved/rejected
    #[schema(example = "task-550e8400-e29b-41d4-a716-446655440000")]
    pub task_id: String,
    
    /// Approval decision
    #[schema(example = "approved")]
    pub decision: String,
    
    /// Optional comments
    #[schema(example = "Approved for Q1 hiring plan")]
    pub comments: Option<String>,
}

/// Response for approval submission
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubmitApprovalResponse {
    /// Unique approval identifier
    #[schema(example = "approval-550e8400-e29b-41d4-a716-446655440000")]
    pub approval_id: String,
    
    /// Approval status
    #[schema(example = "accepted")]
    pub status: String,
}

/// Submit an approval decision
///
/// Submits an approval or rejection decision for a task. In Phase 3, approvals
/// are ephemeral and not persisted (deferred to Phase 4).
#[utoipa::path(
    post,
    path = "/approvals",
    tag = "approvals",
    request_body = SubmitApprovalRequest,
    responses(
        (status = 202, description = "Approval accepted", body = SubmitApprovalResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn submit_approval(
    State(_state): State<AppState>,
    Json(payload): Json<SubmitApprovalRequest>,
) -> (StatusCode, Json<SubmitApprovalResponse>) {
    // Generate approval ID
    let approval_id = format!("approval-{}", Uuid::new_v4());

    // Emit audit event
    info!(
        message = "approval.submitted",
        approval_id = %approval_id,
        task_id = %payload.task_id,
        decision = %payload.decision,
        has_comments = payload.comments.is_some()
    );

    let response = SubmitApprovalResponse {
        approval_id,
        status: "accepted".to_string(),
    };

    (StatusCode::ACCEPTED, Json(response))
}
