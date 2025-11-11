use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::{ActivityLogEntry, DetectionMethod, PrivacyMode, RoutingMode, ProxyState};

/// Response for /api/status endpoint
#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub mode: PrivacyMode,
    pub detection_method: DetectionMethod,
    pub allow_override: bool,
    pub last_updated: DateTime<Utc>,
    pub activity_count: usize,
}

/// Response for /api/activity endpoint
#[derive(Serialize)]
pub struct ActivityResponse {
    pub entries: Vec<ActivityLogEntry>,
    pub total_count: usize,
}

/// Request body for /api/mode endpoint
#[derive(Deserialize)]
pub struct SetModeRequest {
    pub mode: PrivacyMode,
}

/// Request body for /api/detection endpoint
#[derive(Deserialize)]
pub struct SetDetectionMethodRequest {
    pub method: DetectionMethod,
}

/// Serve the Control Panel UI (embedded HTML)
pub async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("ui/index.html"))
}

/// GET /api/mode - Get current privacy mode
pub async fn get_mode(State(state): State<ProxyState>) -> Json<PrivacyMode> {
    let mode = state.get_mode().await;
    Json(mode)
}

/// PUT /api/mode - Set privacy mode
pub async fn set_mode(
    State(state): State<ProxyState>,
    Json(request): Json<SetModeRequest>,
) -> impl IntoResponse {
    state.set_mode(request.mode).await;
    (StatusCode::OK, Json(request.mode))
}

/// GET /api/status - Get service status
pub async fn get_status(State(state): State<ProxyState>) -> Json<StatusResponse> {
    let mode = state.get_mode().await;
    let detection_method = state.get_detection_method().await;
    let allow_override = state.get_allow_override().await;
    let activity_count = state.get_activity_count().await;
    
    Json(StatusResponse {
        status: "healthy".to_string(),
        mode,
        detection_method,
        allow_override,
        last_updated: Utc::now(),
        activity_count,
    })
}

/// GET /api/activity - Get recent activity log
pub async fn get_activity(State(state): State<ProxyState>) -> Json<ActivityResponse> {
    let entries = state.get_recent_activity(20).await;
    let total_count = state.get_activity_count().await;
    
    Json(ActivityResponse {
        entries,
        total_count,
    })
}

/// GET /api/detection - Get current detection method
pub async fn get_detection_method(State(state): State<ProxyState>) -> Json<DetectionMethod> {
    let method = state.get_detection_method().await;
    Json(method)
}

/// PUT /api/detection - Set detection method (only if override allowed)
pub async fn set_detection_method(
    State(state): State<ProxyState>,
    Json(request): Json<SetDetectionMethodRequest>,
) -> impl IntoResponse {
    match state.set_detection_method(request.method).await {
        Ok(_) => (StatusCode::OK, Json(request.method)).into_response(),
        Err(e) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": e
            })),
        ).into_response(),
    }
}

/// Combined settings request/response
#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub routing: RoutingMode,
    pub detection: DetectionMethod,
    pub privacy: PrivacyMode,
}

/// GET /api/settings - Get all current settings
pub async fn get_settings(State(state): State<ProxyState>) -> Json<Settings> {
    let routing = state.get_routing_mode().await;
    let detection = state.get_detection_method().await;
    let privacy = state.get_mode().await;
    
    Json(Settings {
        routing,
        detection,
        privacy,
    })
}

/// PUT /api/settings - Update all settings at once
pub async fn set_settings(
    State(state): State<ProxyState>,
    Json(request): Json<Settings>,
) -> impl IntoResponse {
    // Update routing mode
    state.set_routing_mode(request.routing).await;
    
    // Update detection method (only if override allowed)
    if let Err(e) = state.set_detection_method(request.detection).await {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": e
            })),
        ).into_response();
    }
    
    // Update privacy mode
    state.set_mode(request.privacy).await;
    
    (StatusCode::OK, Json(request)).into_response()
}
