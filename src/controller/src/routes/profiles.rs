// Phase 5 Workstream D: Profile API Endpoints
// Replaces Phase 3 mock data with real Postgres-backed profiles

use axum::{
    extract::{State, Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;  // Trait for try_get() method on PgRow
use utoipa::ToSchema;
use tracing::{info, error};

pub use crate::AppState;
use crate::profile::schema::Profile;

/// Recipe summary for listing
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecipeSummary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub schedule: String,
    pub enabled: bool,
}

/// Recipes list response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecipesResponse {
    pub recipes: Vec<RecipeSummary>,
}

/// Query params for local hints
#[derive(Debug, Deserialize)]
pub struct LocalHintsQuery {
    pub path: String,
}

/// Custom error type for profile endpoints
#[derive(Debug)]
pub enum ProfileError {
    NotFound(String),
    Forbidden(String),
    DatabaseError(String),
    InternalError(String),
}

impl IntoResponse for ProfileError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProfileError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ProfileError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ProfileError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ProfileError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));
        
        (status, body).into_response()
    }
}

/// D1: Get complete profile by role
///
/// Returns full profile from Postgres (replaces Phase 3 mock data).
/// Requires JWT with matching role claim OR admin role.
#[utoipa::path(
    get,
    path = "/profiles/{role}",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier")
    ),
    responses(
        (status = 200, description = "Agent profile", body = Profile),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
        (status = 403, description = "Forbidden - role mismatch"),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Json<Profile>, ProfileError> {
    info!(message = "profile.get", role = %role);

    // Get database pool
    let pool = state.db_pool.as_ref()
        .ok_or_else(|| ProfileError::InternalError("Database not configured".to_string()))?;

    // Load profile from Postgres (using runtime query to avoid compile-time DB checks in Docker)
    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!(message = "profile.query.error", role = %role, error = %e);
            ProfileError::DatabaseError(format!("Database query failed: {}", e))
        })?
        .ok_or_else(|| ProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    // Extract JSONB data column
    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| ProfileError::DatabaseError(format!("Failed to extract data column: {}", e)))?;

    // Deserialize JSONB to Profile struct
    let profile: Profile = serde_json::from_value(data)
        .map_err(|e| {
            error!(message = "profile.deserialize.error", role = %role, error = %e);
            ProfileError::InternalError(format!("Failed to deserialize profile: {}", e))
        })?;

    info!(message = "profile.retrieved", role = %role);
    Ok(Json(profile))
}

/// D2: Generate config.yaml from profile
///
/// Generates Goose v1.12.1 config.yaml from profile data.
#[utoipa::path(
    get,
    path = "/profiles/{role}/config",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier")
    ),
    responses(
        (status = 200, description = "config.yaml generated", content_type = "text/plain"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_config(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Response, ProfileError> {
    info!(message = "profile.config.get", role = %role);

    // Load profile
    let pool = state.db_pool.as_ref()
        .ok_or_else(|| ProfileError::InternalError("Database not configured".to_string()))?;

    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| ProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| ProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| ProfileError::DatabaseError(format!("Failed to extract data column: {}", e)))?;

    let profile: Profile = serde_json::from_value(data)
        .map_err(|e| ProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    // Generate config.yaml
    let mut config = format!(
        "provider: {}\nmodel: {}\n",
        profile.providers.primary.provider,
        profile.providers.primary.model
    );

    if let Some(temp) = profile.providers.primary.temperature {
        config.push_str(&format!("temperature: {}\n", temp));
    }

    config.push_str("\nextensions:\n");
    for ext in &profile.extensions {
        config.push_str(&format!("  - name: {}\n", ext.name));
        config.push_str(&format!("    enabled: {}\n", ext.enabled));
    }

    info!(message = "profile.config.generated", role = %role);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(config.into())
        .unwrap())
}

/// D3: Get global goosehints
///
/// Returns global goosehints ready for ~/.config/goose/.goosehints
#[utoipa::path(
    get,
    path = "/profiles/{role}/goosehints",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier")
    ),
    responses(
        (status = 200, description = "Global goosehints", content_type = "text/plain"),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_goosehints(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Response, ProfileError> {
    info!(message = "profile.goosehints.get", role = %role);

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| ProfileError::InternalError("Database not configured".to_string()))?;

    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| ProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| ProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| ProfileError::DatabaseError(format!("Failed to extract data column: {}", e)))?;

    let profile: Profile = serde_json::from_value(data)
        .map_err(|e| ProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(profile.goosehints.global.into())
        .unwrap())
}

/// D4: Get global gooseignore
///
/// Returns global gooseignore ready for ~/.config/goose/.gooseignore
#[utoipa::path(
    get,
    path = "/profiles/{role}/gooseignore",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier")
    ),
    responses(
        (status = 200, description = "Global gooseignore", content_type = "text/plain"),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_gooseignore(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Response, ProfileError> {
    info!(message = "profile.gooseignore.get", role = %role);

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| ProfileError::InternalError("Database not configured".to_string()))?;

    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| ProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| ProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| ProfileError::DatabaseError(format!("Failed to extract data column: {}", e)))?;

    let profile: Profile = serde_json::from_value(data)
        .map_err(|e| ProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(profile.gooseignore.global.into())
        .unwrap())
}

/// D5: Get local hints template by path
///
/// Returns local hints template matching the provided path.
#[utoipa::path(
    get,
    path = "/profiles/{role}/local-hints",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier"),
        ("path" = String, Query, description = "Project path to match")
    ),
    responses(
        (status = 200, description = "Local hints template", content_type = "text/plain"),
        (status = 404, description = "Template not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_local_hints(
    State(state): State<AppState>,
    Path(role): Path<String>,
    Query(query): Query<LocalHintsQuery>,
) -> Result<Response, ProfileError> {
    info!(message = "profile.local_hints.get", role = %role, path = %query.path);

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| ProfileError::InternalError("Database not configured".to_string()))?;

    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| ProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| ProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| ProfileError::DatabaseError(format!("Failed to extract data column: {}", e)))?;

    let profile: Profile = serde_json::from_value(data)
        .map_err(|e| ProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    // Find matching template
    let template = profile.goosehints.local_templates
        .iter()
        .find(|t| t.path == query.path)
        .ok_or_else(|| ProfileError::NotFound(format!("No local hints template found for path: {}", query.path)))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(template.content.clone().into())
        .unwrap())
}

/// D6: List recipes for role
///
/// Returns list of recipes with schedules and enabled status.
#[utoipa::path(
    get,
    path = "/profiles/{role}/recipes",
    tag = "profiles",
    params(
        ("role" = String, Path, description = "Agent role identifier")
    ),
    responses(
        (status = 200, description = "Recipe list", body = RecipesResponse),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_recipes(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Json<RecipesResponse>, ProfileError> {
    info!(message = "profile.recipes.get", role = %role);

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| ProfileError::InternalError("Database not configured".to_string()))?;

    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| ProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| ProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| ProfileError::DatabaseError(format!("Failed to extract data column: {}", e)))?;

    let profile: Profile = serde_json::from_value(data)
        .map_err(|e| ProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    let recipes: Vec<RecipeSummary> = profile.recipes.iter().map(|r| RecipeSummary {
        name: r.name.clone(),
        description: r.description.clone(),
        schedule: r.schedule.clone(),
        enabled: r.enabled,
    }).collect();

    Ok(Json(RecipesResponse { recipes }))
}

#[cfg(test)]
#[path = "profiles_test.rs"]
mod profiles_test;
