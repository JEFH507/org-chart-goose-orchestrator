use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::info;

use crate::AppState;

/// Request to create a new session
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    /// Agent role initiating the session
    #[schema(example = "finance")]
    pub agent_role: String,
    
    /// Optional session metadata
    pub metadata: Option<serde_json::Value>,
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

/// List active sessions
///
/// Returns a list of active sessions. In Phase 3, this returns an empty array
/// as persistence is not yet implemented (deferred to Phase 4).
#[utoipa::path(
    get,
    path = "/sessions",
    tag = "sessions",
    responses(
        (status = 200, description = "List of sessions", body = Vec<SessionResponse>),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_sessions(
    State(_state): State<AppState>,
) -> (StatusCode, Json<Vec<SessionResponse>>) {
    // Phase 3: Return empty array (no persistence yet)
    // Phase 4 will query database
    info!(message = "sessions.list", count = 0);
    (StatusCode::OK, Json(vec![]))
}

/// Create a new session
///
/// Creates a new session for an agent. In Phase 3, sessions are ephemeral
/// and not persisted (deferred to Phase 4).
#[utoipa::path(
    post,
    path = "/sessions",
    tag = "sessions",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created", body = CreateSessionResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_session(
    State(_state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> (StatusCode, Json<CreateSessionResponse>) {
    // Generate session ID
    let session_id = format!("session-{}", Uuid::new_v4());

    info!(
        message = "session.created",
        session_id = %session_id,
        agent_role = %payload.agent_role,
        has_metadata = payload.metadata.is_some()
    );

    let response = CreateSessionResponse {
        session_id,
        status: "created".to_string(),
    };

    (StatusCode::CREATED, Json(response))
}
