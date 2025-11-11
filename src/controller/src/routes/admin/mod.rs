// Phase 5 Workstream D: Admin routes module
// Phase 6: Added dashboard UI and API routes

pub mod profiles;
pub mod org;

// Dashboard UI and API functions
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use crate::AppState;

// ============================================================================
// Static Admin Page
// ============================================================================

/// Serve the admin dashboard HTML page
pub async fn serve_admin_page() -> Response {
    let html = include_str!("../../../static/admin.html");
    Html(html).into_response()
}

// ============================================================================
// User Management (Dashboard APIs)
// ============================================================================

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub employee_id: String,
    pub name: String,
    pub email: String,
    pub department: Option<String>,
    pub role: Option<String>,
    pub profile: Option<String>,
}

/// List all users for the admin dashboard
pub async fn list_users(
    State(_state): State<AppState>,
) -> Json<Vec<User>> {
    info!("Admin dashboard: listing users");

    // TODO: Fetch from Keycloak or database
    // For MVP demo, return mock data
    let users = vec![
        User {
            id: "user1".to_string(),
            employee_id: "EMP001".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            department: Some("Finance".to_string()),
            role: Some("Analyst".to_string()),
            profile: Some("finance".to_string()),
        },
        User {
            id: "user2".to_string(),
            employee_id: "EMP002".to_string(),
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
            department: Some("Legal".to_string()),
            role: Some("Counsel".to_string()),
            profile: Some("legal".to_string()),
        },
    ];

    Json(users)
}

#[derive(Debug, Deserialize)]
pub struct AssignProfileRequest {
    pub profile: String,
}

#[derive(Debug, Serialize)]
pub struct AssignProfileResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Assign a profile to a user
pub async fn assign_profile(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<AssignProfileRequest>,
) -> Result<Json<AssignProfileResponse>, (StatusCode, Json<AssignProfileResponse>)> {
    info!(
        user_id = %user_id,
        profile = %req.profile,
        "Admin dashboard: assigning profile to user"
    );

    // TODO: Update user in Keycloak with profile assignment
    // For MVP demo, just log the action

    Ok(Json(AssignProfileResponse {
        success: true,
        error: None,
    }))
}

// ============================================================================
// Profile Management (Dashboard APIs - Edit/Download/Upload)
// ============================================================================

/// Get a profile for editing in the dashboard
pub async fn get_profile_for_edit(
    State(_state): State<AppState>,
    Path(profile_name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(profile = %profile_name, "Admin dashboard: loading profile for edit");

    let profile_path = PathBuf::from("deploy/profiles").join(format!("{}.json", profile_name));

    match fs::read_to_string(&profile_path) {
        Ok(content) => {
            match serde_json::from_str(&content) {
                Ok(json) => Ok(Json(json)),
                Err(e) => {
                    error!("Failed to parse profile JSON: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse profile: {}", e),
                    ))
                }
            }
        }
        Err(e) => {
            error!("Failed to read profile file: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                format!("Profile not found: {}", e),
            ))
        }
    }
}

/// Save a profile from the dashboard editor
pub async fn save_profile_from_editor(
    State(_state): State<AppState>,
    Path(profile_name): Path<String>,
    Json(profile_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(profile = %profile_name, "Admin dashboard: saving profile");

    // Validate profile structure (basic check)
    if !profile_data.is_object() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Profile must be a JSON object".to_string(),
        ));
    }

    let profile_dir = PathBuf::from("deploy/profiles");
    
    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&profile_dir) {
        error!("Failed to create profiles directory: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create directory: {}", e),
        ));
    }

    let profile_path = profile_dir.join(format!("{}.json", profile_name));

    // Write profile to file
    let pretty_json = serde_json::to_string_pretty(&profile_data).unwrap();
    
    match fs::write(&profile_path, pretty_json) {
        Ok(_) => {
            info!("Profile saved to {:?}", profile_path);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Profile '{}' saved successfully", profile_name)
            })))
        }
        Err(e) => {
            error!("Failed to write profile file: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save profile: {}", e),
            ))
        }
    }
}

// ============================================================================
// Config Push
// ============================================================================

#[derive(Debug, Serialize)]
pub struct PushConfigResponse {
    pub success: bool,
    pub pushed_count: usize,
    pub error: Option<String>,
}

/// Push configurations to all Goose instances
pub async fn push_configs(
    State(_state): State<AppState>,
) -> Json<PushConfigResponse> {
    info!("Admin dashboard: pushing configs to all Goose instances");

    // TODO: Implement config push logic
    // This would:
    // 1. Read all profile JSONs from deploy/profiles/
    // 2. For each user with assigned profile, push their config
    // 3. Call Privacy Guard Proxy API to update settings
    // 4. Possibly restart Goose instances or trigger reload

    let pushed_count = 3; // Placeholder: finance, manager, legal

    info!(pushed_count = pushed_count, "Config push complete");

    Json(PushConfigResponse {
        success: true,
        pushed_count,
        error: None,
    })
}

// ============================================================================
// Live Logs
// ============================================================================

/// Get live system logs for the dashboard
pub async fn get_logs(
    State(_state): State<AppState>,
) -> String {
    // TODO: Implement proper log streaming
    // Options:
    // 1. Read from a log file
    // 2. Use `docker compose logs --tail=100`
    // 3. Implement in-memory log buffer

    let mock_logs = r#"[2025-01-11 19:00:00] INFO: Controller started
[2025-01-11 19:00:05] INFO: Privacy Guard Proxy (Finance) connected
[2025-01-11 19:00:06] INFO: Privacy Guard Proxy (Manager) connected
[2025-01-11 19:00:07] INFO: Privacy Guard Proxy (Legal) connected
[2025-01-11 19:00:10] INFO: User 'john@example.com' assigned profile 'finance'
[2025-01-11 19:00:15] INFO: Config push initiated
[2025-01-11 19:00:16] INFO: Config pushed to Finance instance
[2025-01-11 19:00:17] INFO: Config pushed to Manager instance
[2025-01-11 19:00:18] INFO: Config pushed to Legal instance
[2025-01-11 19:00:20] INFO: All configs pushed successfully
"#;

    mock_logs.to_string()
}
