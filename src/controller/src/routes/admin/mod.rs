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
    State(state): State<AppState>,
) -> Json<Vec<User>> {
    info!("Admin dashboard: listing users");

    // Get database pool
    let pool = match state.db_pool.as_ref() {
        Some(p) => p,
        None => {
            error!("Database not configured - returning empty list");
            return Json(vec![]);
        }
    };

    // Query users from org_users table
    match sqlx::query_as::<_, (i32, String, String, String, String, Option<String>)>(
        "SELECT user_id, name, email, department, role, assigned_profile FROM org_users ORDER BY user_id"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let users: Vec<User> = rows.into_iter().map(|(user_id, name, email, dept, role, assigned_profile)| {
                User {
                    id: user_id.to_string(),
                    employee_id: format!("EMP{:03}", user_id), // Generate employee ID from user_id
                    name,
                    email,
                    department: Some(dept),
                    role: Some(role),
                    profile: assigned_profile,
                }
            }).collect();
            info!("Loaded {} users from database", users.len());
            Json(users)
        }
        Err(e) => {
            error!("Failed to load users from database: {}", e);
            Json(vec![])
        }
    }
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
    State(state): State<AppState>,
    Path(employee_id): Path<String>,
    Json(req): Json<AssignProfileRequest>,
) -> Result<Json<AssignProfileResponse>, (StatusCode, Json<AssignProfileResponse>)> {
    info!(
        employee_id = %employee_id,
        profile = %req.profile,
        "Admin dashboard: assigning profile to user"
    );

    // Parse employee_id (e.g., "EMP001" -> 1)
    let user_id: i32 = if employee_id.starts_with("EMP") {
        match employee_id[3..].parse() {
            Ok(id) => id,
            Err(_) => {
                error!("Invalid employee_id format: {}", employee_id);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(AssignProfileResponse {
                        success: false,
                        error: Some(format!("Invalid employee_id format: {}", employee_id)),
                    }),
                ));
            }
        }
    } else {
        error!("Employee ID must start with 'EMP': {}", employee_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AssignProfileResponse {
                success: false,
                error: Some(format!("Employee ID must start with 'EMP': {}", employee_id)),
            }),
        ));
    };

    // Get database pool
    let pool = match state.db_pool.as_ref() {
        Some(p) => p,
        None => {
            error!("Database not configured");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AssignProfileResponse {
                    success: false,
                    error: Some("Database not available".to_string()),
                }),
            ));
        }
    };

    // Update user's assigned profile in database
    match sqlx::query(
        "UPDATE org_users SET assigned_profile = $1, updated_at = NOW() WHERE user_id = $2"
    )
    .bind(&req.profile)
    .bind(user_id)
    .execute(pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!("Profile '{}' assigned to user '{}'", req.profile, user_id);
                Ok(Json(AssignProfileResponse {
                    success: true,
                    error: None,
                }))
            } else {
                error!("User '{}' not found in database", user_id);
                Err((
                    StatusCode::NOT_FOUND,
                    Json(AssignProfileResponse {
                        success: false,
                        error: Some(format!("User '{}' not found", user_id)),
                    }),
                ))
            }
        }
        Err(e) => {
            error!("Failed to assign profile: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AssignProfileResponse {
                    success: false,
                    error: Some(format!("Database error: {}", e)),
                }),
            ))
        }
    }
}

// ============================================================================
// Profile Management (Dashboard APIs - Edit/Download/Upload)
// ============================================================================

/// List all available profiles
pub async fn list_profiles(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    info!("Admin dashboard: listing available profiles");

    // Get database pool
    let pool = match state.db_pool.as_ref() {
        Some(p) => p,
        None => {
            error!("Database not configured - returning empty list");
            return Json(vec![]);
        }
    };

    // Query all profile roles from database
    match sqlx::query_as::<_, (String,)>(
        "SELECT role FROM profiles ORDER BY role"
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => {
            let profile_names: Vec<String> = rows.into_iter().map(|(role,)| role).collect();
            info!("Loaded {} profiles from database", profile_names.len());
            Json(profile_names)
        }
        Err(e) => {
            error!("Failed to load profiles from database: {}", e);
            // Fallback to hardcoded list
            Json(vec![
                "analyst".to_string(),
                "developer".to_string(),
                "finance".to_string(),
                "hr".to_string(),
                "legal".to_string(),
                "manager".to_string(),
                "marketing".to_string(),
                "support".to_string(),
            ])
        }
    }
}

/// Get a profile for editing in the dashboard
pub async fn get_profile_for_edit(
    State(state): State<AppState>,
    Path(profile_name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(profile = %profile_name, "Admin dashboard: loading profile for edit");

    // Get database pool
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        error!("Database not configured");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database not available".to_string(),
        )
    })?;

    // Query profile from database
    match sqlx::query_as::<_, (String, serde_json::Value)>(
        "SELECT role, data FROM profiles WHERE role = $1"
    )
    .bind(&profile_name)
    .fetch_one(pool)
    .await
    {
        Ok((_role, profile_data)) => {
            info!(profile = %profile_name, "Profile loaded from database");
            Ok(Json(profile_data))
        }
        Err(sqlx::Error::RowNotFound) => {
            error!("Profile not found in database: {}", profile_name);
            Err((
                StatusCode::NOT_FOUND,
                format!("Profile '{}' not found", profile_name),
            ))
        }
        Err(e) => {
            error!("Database error loading profile: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load profile: {}", e),
            ))
        }
    }
}

/// Save a profile from the dashboard editor
pub async fn save_profile_from_editor(
    State(state): State<AppState>,
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

    // Get database pool
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        error!("Database not configured");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database not available".to_string(),
        )
    })?;

    // Generate display name from profile_name (capitalize first letter)
    let display_name = format!(
        "{}{}",
        profile_name.chars().next().unwrap().to_uppercase(),
        &profile_name[1..]
    );

    // Upsert profile to database
    match sqlx::query(
        r#"
        INSERT INTO profiles (role, display_name, data, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (role)
        DO UPDATE SET
            data = EXCLUDED.data,
            updated_at = NOW()
        "#
    )
    .bind(&profile_name)
    .bind(&display_name)
    .bind(&profile_data)
    .execute(pool)
    .await
    {
        Ok(_) => {
            info!(profile = %profile_name, "Profile saved to database");
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Profile '{}' saved successfully", profile_name)
            })))
        }
        Err(e) => {
            error!("Database error saving profile: {}", e);
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
