use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::{info, warn};

use crate::AppState;

/// Task payload for routing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskPayload {
    /// Task type (e.g., "budget_approval", "notification")
    #[schema(example = "budget_approval")]
    pub task_type: String,
    
    /// Task description
    #[schema(example = "Q1 budget approval request")]
    pub description: Option<String>,
    
    /// Additional task-specific data
    pub data: Option<serde_json::Value>,
}

/// Request to route a task to a target agent
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RouteTaskRequest {
    /// Target agent role (e.g., "manager", "finance")
    #[schema(example = "manager")]
    pub target: String,
    
    /// Task details
    pub task: TaskPayload,
    
    /// Optional context for the task
    pub context: Option<serde_json::Value>,
}

/// Response for task routing
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RouteTaskResponse {
    /// Unique task identifier
    #[schema(example = "task-550e8400-e29b-41d4-a716-446655440000")]
    pub task_id: String,
    
    /// Task status
    #[schema(example = "accepted")]
    pub status: String,
    
    /// Trace ID for correlation
    pub trace_id: String,
}

/// Route a task to a target agent
///
/// Routes a task to the specified target agent role. The request must include
/// an Idempotency-Key header with a valid UUID to prevent duplicate submissions.
/// 
/// Sensitive data in the task and context will be automatically masked by the
/// Privacy Guard if enabled.
#[utoipa::path(
    post,
    path = "/tasks/route",
    tag = "tasks",
    request_body = RouteTaskRequest,
    responses(
        (status = 202, description = "Task accepted for routing", body = RouteTaskResponse),
        (status = 400, description = "Bad request - missing or invalid Idempotency-Key"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 413, description = "Payload too large - exceeds 1MB limit"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn route_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut payload): Json<RouteTaskRequest>,
) -> Result<(StatusCode, Json<RouteTaskResponse>), StatusCode> {
    // Extract or generate trace ID
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Validate Idempotency-Key header
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            warn!(
                message = "missing idempotency key",
                trace_id = %trace_id
            );
            StatusCode::BAD_REQUEST
        })?;

    // Validate that Idempotency-Key is a valid UUID
    Uuid::parse_str(idempotency_key).map_err(|_| {
        warn!(
            message = "invalid idempotency key format",
            trace_id = %trace_id,
            idempotency_key = %idempotency_key
        );
        StatusCode::BAD_REQUEST
    })?;

    // Generate task ID
    let task_id = format!("task-{}", Uuid::new_v4());

    // Apply Privacy Guard masking if enabled
    let guard_start = std::time::Instant::now();
    if state.guard_client.is_enabled() {
        // Mask task payload
        if let Some(task_data) = &payload.task.data {
            match state.guard_client.mask_json(task_data, "task_data", Some(&trace_id)).await {
                Ok(Some(masked)) => {
                    payload.task.data = Some(masked);
                }
                Ok(None) => {
                    warn!(
                        message = "guard disabled or failed (fail-open)",
                        trace_id = %trace_id
                    );
                }
                Err(e) => {
                    warn!(
                        message = "guard error on task data",
                        error = %e,
                        trace_id = %trace_id
                    );
                }
            }
        }

        // Mask context
        if let Some(context) = &payload.context {
            match state.guard_client.mask_json(context, "task_context", Some(&trace_id)).await {
                Ok(Some(masked)) => {
                    payload.context = Some(masked);
                }
                Ok(None) => {
                    warn!(
                        message = "guard disabled or failed (fail-open)",
                        trace_id = %trace_id
                    );
                }
                Err(e) => {
                    warn!(
                        message = "guard error on context",
                        error = %e,
                        trace_id = %trace_id
                    );
                }
            }
        }

        let guard_duration = guard_start.elapsed();
        info!(
            message = "privacy guard applied",
            trace_id = %trace_id,
            duration_ms = guard_duration.as_millis()
        );
    }

    // Emit audit event
    info!(
        message = "task.routed",
        task_id = %task_id,
        target = %payload.target,
        task_type = %payload.task.task_type,
        trace_id = %trace_id,
        idempotency_key = %idempotency_key,
        has_context = payload.context.is_some()
    );

    let response = RouteTaskResponse {
        task_id,
        status: "accepted".to_string(),
        trace_id,
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}
