use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use tracing::info;

use crate::AppState;

/// Agent profile information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProfileResponse {
    /// Agent role identifier
    #[schema(example = "manager")]
    pub role: String,
    
    /// Agent capabilities
    #[schema(example = json!(["task_routing", "approval_workflow"]))]
    pub capabilities: Vec<String>,
    
    /// Agent metadata
    pub metadata: Option<serde_json::Value>,
}

/// Get agent profile by role
///
/// Returns profile information for an agent role. In Phase 3, this returns
/// mock data. Phase 4 will query the Directory Service for real profiles.
#[utoipa::path(
    get,
    path = "/profiles/{role}",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier")
    ),
    responses(
        (status = 200, description = "Agent profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    State(_state): State<AppState>,
    Path(role): Path<String>,
) -> (StatusCode, Json<ProfileResponse>) {
    // Phase 3: Return mock profile
    // Phase 4 will query Directory Service (LDAP/AD)
    
    info!(
        message = "profile.retrieved",
        role = %role
    );

    // Mock profile data
    let capabilities = match role.as_str() {
        "manager" => vec!["task_routing".to_string(), "approval_workflow".to_string()],
        "finance" => vec!["budget_requests".to_string(), "expense_tracking".to_string()],
        "engineering" => vec!["code_review".to_string(), "deployment".to_string()],
        _ => vec!["task_routing".to_string()],
    };

    let profile = ProfileResponse {
        role: role.clone(),
        capabilities,
        metadata: Some(serde_json::json!({
            "mock": true,
            "description": "Phase 3 mock profile - will be replaced by Directory Service in Phase 4"
        })),
    };

    (StatusCode::OK, Json(profile))
}
