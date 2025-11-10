use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use tracing::{error, info};

pub use crate::AppState;
use crate::models::{CreateSessionRequest as DbCreateSessionRequest, SessionListResponse, UpdateSessionRequest};
use crate::repository::SessionRepository;
use crate::lifecycle::TransitionError;

/// Request to create a new session (API contract)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    /// Agent role initiating the session
    #[schema(example = "finance")]
    pub agent_role: String,
    
    /// Optional task ID to associate with this session
    pub task_id: Option<Uuid>,
    
    /// Optional session metadata
    pub metadata: Option<serde_json::Value>,
}

/// Request to update an existing session (API contract)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateSessionPayload {
    /// Optional task ID to associate with this session
    pub task_id: Option<Uuid>,
    
    /// Optional status update
    pub status: Option<String>,
    
    /// Optional metadata update
    pub metadata: Option<serde_json::Value>,
}

/// Pagination query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (default: 1)
    #[param(example = 1)]
    pub page: Option<i32>,
    
    /// Page size (default: 20, max: 100)
    #[param(example = 20)]
    pub page_size: Option<i32>,
}

/// Response for session creation
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionResponse {
    /// Unique session identifier
    #[schema(example = "session-550e8400-e29b-41d4-a716-446655440000")]
    pub session_id: String,
    
    /// Session status
    #[schema(example = "created")]
    pub status: String,
}

/// Session information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionResponse {
    /// Unique session identifier
    pub session_id: String,
    
    /// Agent role
    pub agent_role: String,
    
    /// Session state
    #[schema(example = "active")]
    pub state: String,
    
    /// Session metadata
    pub metadata: Option<serde_json::Value>,
}

/// Request to trigger a lifecycle event (Phase 6 A1: FSM integration)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SessionEventRequest {
    /// Event to trigger (activate, complete, fail, expire)
    #[schema(example = "activate")]
    pub event: String,
}

/// List sessions with pagination
///
/// Returns a paginated list of sessions from the database.
#[utoipa::path(
    get,
    path = "/sessions",
    tag = "sessions",
    params(PaginationParams),
    responses(
        (status = 200, description = "List of sessions", body = SessionListResponse),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Database not available"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<(StatusCode, Json<SessionListResponse>), (StatusCode, String)> {
    // Check if database is available
    let pool = state.db_pool.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Database not configured".to_string()))?;

    let repo = SessionRepository::new(pool.clone());
    
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);

    match repo.list(page, page_size).await {
        Ok((sessions, total)) => {
            info!(
                message = "sessions.list",
                count = sessions.len(),
                total = total,
                page = page,
                page_size = page_size
            );
            
            let response = SessionListResponse {
                sessions,
                total,
                page,
                page_size,
            };
            
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!(message = "sessions.list.error", error = %e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
        }
    }
}

/// Create a new session
///
/// Creates a new session for an agent and persists it to the database.
#[utoipa::path(
    post,
    path = "/sessions",
    tag = "sessions",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created", body = CreateSessionResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Database not available"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<CreateSessionResponse>), (StatusCode, String)> {
    // Check if database is available
    let pool = state.db_pool.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Database not configured".to_string()))?;

    let repo = SessionRepository::new(pool.clone());

    // Convert API request to database model
    let db_request = DbCreateSessionRequest {
        role: payload.agent_role.clone(),
        task_id: payload.task_id,
        metadata: payload.metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    match repo.create(db_request).await {
        Ok(session) => {
            info!(
                message = "session.created",
                session_id = %session.id,
                role = %session.role,
                status = ?session.status
            );

            let response = CreateSessionResponse {
                session_id: session.id.to_string(),
                status: "pending".to_string(),
            };

            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            error!(message = "session.create.error", error = %e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
        }
    }
}

/// Get a specific session by ID
///
/// Retrieves a session by its unique identifier.
#[utoipa::path(
    get,
    path = "/sessions/{id}",
    tag = "sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session found", body = SessionResponse),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Database not available"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<SessionResponse>), (StatusCode, String)> {
    // Check if database is available
    let pool = state.db_pool.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Database not configured".to_string()))?;

    let repo = SessionRepository::new(pool.clone());

    match repo.get(id).await {
        Ok(Some(session)) => {
            info!(message = "session.get", session_id = %id, role = %session.role);
            
            let response = SessionResponse {
                session_id: session.id.to_string(),
                agent_role: session.role,
                state: format!("{:?}", session.status).to_lowercase(),
                metadata: Some(session.metadata),
            };
            
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            info!(message = "session.get.not_found", session_id = %id);
            Err((StatusCode::NOT_FOUND, "Session not found".to_string()))
        }
        Err(e) => {
            error!(message = "session.get.error", error = %e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
        }
    }
}

/// Update a session
///
/// Updates an existing session with new metadata or status.
#[utoipa::path(
    put,
    path = "/sessions/{id}",
    tag = "sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    request_body = UpdateSessionPayload,
    responses(
        (status = 200, description = "Session updated", body = SessionResponse),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Database not available"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSessionPayload>,
) -> Result<(StatusCode, Json<SessionResponse>), (StatusCode, String)> {
    // Check if database is available
    let pool = state.db_pool.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Database not configured".to_string()))?;

    let repo = SessionRepository::new(pool.clone());

    // Convert API status string to SessionStatus if provided
    let status = if let Some(status_str) = payload.status.as_ref() {
        use crate::models::SessionStatus;
        match status_str.to_lowercase().as_str() {
            "pending" => Some(SessionStatus::Pending),
            "active" => Some(SessionStatus::Active),
            "completed" => Some(SessionStatus::Completed),
            "failed" => Some(SessionStatus::Failed),
            "expired" => Some(SessionStatus::Expired),
            _ => return Err((StatusCode::BAD_REQUEST, format!("Invalid status: {}", status_str))),
        }
    } else {
        None
    };

    let db_request = UpdateSessionRequest {
        task_id: payload.task_id,
        status,
        metadata: payload.metadata,
    };

    match repo.update(id, db_request).await {
        Ok(Some(session)) => {
            info!(
                message = "session.updated",
                session_id = %id,
                role = %session.role,
                status = ?session.status
            );
            
            let response = SessionResponse {
                session_id: session.id.to_string(),
                agent_role: session.role,
                state: format!("{:?}", session.status).to_lowercase(),
                metadata: Some(session.metadata),
            };
            
            Ok((StatusCode::OK, Json(response)))
        }
        Ok(None) => {
            info!(message = "session.update.not_found", session_id = %id);
            Err((StatusCode::NOT_FOUND, "Session not found".to_string()))
        }
        Err(e) => {
            error!(message = "session.update.error", error = %e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
        }
    }
}

/// Handle session lifecycle event (Phase 6 A1: FSM integration)
///
/// Triggers a lifecycle event on a session using the SessionLifecycle FSM.
/// Supported events: activate, pause, resume, complete, fail
#[utoipa::path(
    put,
    path = "/sessions/{id}/events",
    tag = "sessions",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    request_body = SessionEventRequest,
    responses(
        (status = 200, description = "Event processed", body = SessionResponse),
        (status = 400, description = "Invalid event or transition"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "SessionLifecycle not available"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn handle_session_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SessionEventRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), (StatusCode, String)> {
    // Check if SessionLifecycle is available
    let lifecycle = state.session_lifecycle.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "SessionLifecycle not configured".to_string()))?;

    // Parse event and trigger corresponding lifecycle method
    let session = match payload.event.to_lowercase().as_str() {
        "activate" => {
            lifecycle.activate(id)
                .await
                .map_err(|e| match e {
                    TransitionError::SessionNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Session not found".to_string())
                    }
                    TransitionError::InvalidTransition { from, to } => {
                        (StatusCode::BAD_REQUEST, format!("Invalid transition from {:?} to {:?}", from, to))
                    }
                    TransitionError::DatabaseError(msg) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg))
                    }
                })?
        }
        "complete" => {
            lifecycle.complete(id)
                .await
                .map_err(|e| match e {
                    TransitionError::SessionNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Session not found".to_string())
                    }
                    TransitionError::InvalidTransition { from, to } => {
                        (StatusCode::BAD_REQUEST, format!("Invalid transition from {:?} to {:?}", from, to))
                    }
                    TransitionError::DatabaseError(msg) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg))
                    }
                })?
        }
        "fail" => {
            lifecycle.fail(id)
                .await
                .map_err(|e| match e {
                    TransitionError::SessionNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Session not found".to_string())
                    }
                    TransitionError::InvalidTransition { from, to } => {
                        (StatusCode::BAD_REQUEST, format!("Invalid transition from {:?} to {:?}", from, to))
                    }
                    TransitionError::DatabaseError(msg) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg))
                    }
                })?
        }
        "pause" => {
            lifecycle.pause(id)
                .await
                .map_err(|e| match e {
                    TransitionError::SessionNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Session not found".to_string())
                    }
                    TransitionError::InvalidTransition { from, to } => {
                        (StatusCode::BAD_REQUEST, format!("Invalid transition from {:?} to {:?}", from, to))
                    }
                    TransitionError::DatabaseError(msg) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg))
                    }
                })?
        }
        "resume" => {
            lifecycle.resume(id)
                .await
                .map_err(|e| match e {
                    TransitionError::SessionNotFound(_) => {
                        (StatusCode::NOT_FOUND, "Session not found".to_string())
                    }
                    TransitionError::InvalidTransition { from, to } => {
                        (StatusCode::BAD_REQUEST, format!("Invalid transition from {:?} to {:?}", from, to))
                    }
                    TransitionError::DatabaseError(msg) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg))
                    }
                })?
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid event: {}. Supported events: activate, complete, fail, pause, resume", payload.event)
            ));
        }
    };

    info!(
        message = "session.event.processed",
        session_id = %id,
        event = %payload.event,
        new_status = ?session.status
    );

    let response = SessionResponse {
        session_id: session.id.to_string(),
        agent_role: session.role,
        state: format!("{:?}", session.status).to_lowercase(),
        metadata: Some(session.metadata),
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
#[path = "sessions_test.rs"]
mod sessions_test;
