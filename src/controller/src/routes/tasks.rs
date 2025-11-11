use axum::{
    extract::{State, Json, Path},
    http::{StatusCode, HeaderMap},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::{info, warn, error};
use std::sync::Arc;

// Import AppState and repositories
pub use crate::AppState;
use crate::repository::TaskRepository;
use crate::models::{Task, CreateTaskRequest};

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

    // Get database pool
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        error!(message = "database not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let task_repo = TaskRepository::new(db_pool.clone());
    
    // Parse idempotency key as UUID
    let idempotency_uuid = Uuid::parse_str(idempotency_key).map_err(|_| {
        warn!(message = "invalid UUID format", idempotency_key = %idempotency_key);
        StatusCode::BAD_REQUEST
    })?;

    // Check for existing task with same idempotency key
    if let Ok(Some(existing_task)) = task_repo.find_by_idempotency_key(idempotency_uuid).await {
        info!(
            message = "task already exists (idempotent)",
            task_id = %existing_task.id,
            idempotency_key = %idempotency_key
        );
        
        let response = RouteTaskResponse {
            task_id: existing_task.id.to_string(),
            status: existing_task.status,
            trace_id,
        };
        
        return Ok((StatusCode::ACCEPTED, Json(response)));
    }

    // Parse trace ID as UUID (optional)
    let trace_uuid = if !trace_id.is_empty() {
        Uuid::parse_str(&trace_id).ok()
    } else {
        None
    };

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

    // Extract source from JWT claims (or use "unknown" for now)
    // TODO: Extract from JWT once auth middleware is enabled
    let source = "unknown".to_string();

    // Create task in database
    let create_req = CreateTaskRequest {
        task_type: payload.task.task_type.clone(),
        description: payload.task.description.clone(),
        data: payload.task.data.clone(),
        source,
        target: payload.target.clone(),
        context: payload.context.clone(),
        trace_id: trace_uuid,
        idempotency_key: Some(idempotency_uuid),
    };

    let task = task_repo.create(&create_req).await.map_err(|e| {
        error!(message = "failed to create task", error = %e, trace_id = %trace_id);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Emit audit event
    info!(
        message = "task.created",
        task_id = %task.id,
        target = %task.target,
        task_type = %task.task_type,
        trace_id = %trace_id,
        idempotency_key = %idempotency_key,
        has_context = payload.context.is_some()
    );

    let response = RouteTaskResponse {
        task_id: task.id.to_string(),
        status: task.status,
        trace_id,
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// Get a task by ID
///
/// Retrieves a task by its unique identifier. Used by fetch_status tool.
#[utoipa::path(
    get,
    path = "/tasks/{id}",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = Task),
        (status = 404, description = "Task not found"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, StatusCode> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        error!(message = "database not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let task_repo = TaskRepository::new(db_pool.clone());
    
    let task = task_repo.get(id).await.map_err(|e| {
        error!(message = "database error", error = %e, task_id = %id);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    task.ok_or_else(|| {
        info!(message = "task not found", task_id = %id);
        StatusCode::NOT_FOUND
    })
    .map(Json)
}

/// List tasks for a target role
///
/// Retrieves tasks assigned to a specific role. Useful for agents to query their pending tasks.
#[utoipa::path(
    get,
    path = "/tasks",
    tag = "tasks",
    params(
        ("target" = Option<String>, Query, description = "Filter by target role"),
        ("status" = Option<String>, Query, description = "Filter by status (pending, active, completed)"),
        ("limit" = Option<i64>, Query, description = "Maximum number of tasks to return (default: 50)")
    ),
    responses(
        (status = 200, description = "List of tasks", body = Vec<Task>),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_tasks(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        error!(message = "database not configured");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let task_repo = TaskRepository::new(db_pool.clone());
    
    let target = params.get("target");
    let status_filter = params.get("status");
    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    let tasks = if let Some(target_role) = target {
        if let Some("pending") = status_filter.map(|s| s.as_str()) {
            task_repo.list_pending(target_role, limit).await
        } else {
            task_repo.list_by_target(target_role, limit).await
        }
    } else {
        // Return empty list if no target specified (could enhance this later)
        Ok(Vec::new())
    }
    .map_err(|e| {
        error!(message = "database error", error = %e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(tasks))
}

#[cfg(test)]
#[path = "tasks_test.rs"]
mod tasks_test;
