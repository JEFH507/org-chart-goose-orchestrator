// Phase 5 Workstream D: Admin Profile Endpoints (D7-D9)
// Create, update, and sign profiles with Vault

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;
use tracing::{info, error};
use chrono::Utc;

use crate::AppState;
use crate::profile::schema::Profile;
use crate::profile::validator::ProfileValidator;
use crate::vault::transit::TransitOps;
use crate::vault::VaultConfig;
use crate::vault::client::VaultClient;

/// Recursively sort JSON object keys alphabetically for canonical serialization
/// This matches the canonical_sort_json function in src/vault/verify.rs
/// Critical for HMAC verification to work correctly with Postgres JSONB storage
fn canonical_sort_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();  // Alphabetical sort
            for key in keys {
                sorted.insert(
                    key.clone(),
                    canonical_sort_json(&map[key])  // Recursive sort
                );
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(
                arr.iter().map(canonical_sort_json).collect()
            )
        }
        other => other.clone(),
    }
}

/// Create profile response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateProfileResponse {
    pub role: String,
    pub created_at: String,
}

/// Update profile response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileResponse {
    pub role: String,
    pub updated_at: String,
}

/// Publish profile response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublishProfileResponse {
    pub role: String,
    pub signature: String,
    pub signed_at: String,
}

/// Custom error type for admin endpoints
#[derive(Debug)]
pub enum AdminProfileError {
    NotFound(String),
    Forbidden(String),
    ValidationError(String),
    DatabaseError(String),
    VaultError(String),
    InternalError(String),
}

impl IntoResponse for AdminProfileError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AdminProfileError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AdminProfileError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AdminProfileError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AdminProfileError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminProfileError::VaultError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AdminProfileError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));
        
        (status, body).into_response()
    }
}

/// D7: Create new profile (admin only)
///
/// Creates a new profile in Postgres. Requires admin role in JWT.
#[utoipa::path(
    post,
    path = "/admin/profiles",
    tag = "admin",
    request_body = Profile,
    responses(
        (status = 201, description = "Profile created", body = CreateProfileResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not admin"),
        (status = 500, description = "Internal error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_profile(
    State(state): State<AppState>,
    Json(mut profile): Json<Profile>,
) -> Result<(StatusCode, Json<CreateProfileResponse>), AdminProfileError> {
    info!(message = "admin.profile.create", role = %profile.role);

    // TODO: Check admin role from JWT claims (when JWT middleware integrated)
    
    // Validate profile
    ProfileValidator::validate(&profile)
        .map_err(|e| {
            error!(message = "profile.validation.error", role = %profile.role, error = %e);
            AdminProfileError::ValidationError(format!("Profile validation failed: {}", e))
        })?;

    // Remove signature if present (will be added on publish)
    profile.signature = None;

    // Get database pool
    let pool = state.db_pool.as_ref()
        .ok_or_else(|| AdminProfileError::InternalError("Database not configured".to_string()))?;

    // Serialize to JSONB
    let data = serde_json::to_value(&profile)
        .map_err(|e| AdminProfileError::InternalError(format!("Serialization failed: {}", e)))?;

    // Insert into Postgres
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO profiles (role, display_name, data, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&profile.role)
    .bind(&profile.display_name)
    .bind(&data)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(message = "profile.insert.error", role = %profile.role, error = %e);
        AdminProfileError::DatabaseError(format!("Failed to insert profile: {}", e))
    })?;

    info!(message = "admin.profile.created", role = %profile.role);
    
    Ok((
        StatusCode::CREATED,
        Json(CreateProfileResponse {
            role: profile.role,
            created_at: now.to_rfc3339(),
        })
    ))
}

/// D8: Update existing profile (admin only)
///
/// Updates an existing profile. Supports partial updates.
#[utoipa::path(
    put,
    path = "/admin/profiles/{role}",
    tag = "admin",
    params(
        ("role" = String, Path, description = "Role to update")
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Profile updated", body = UpdateProfileResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not admin"),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Internal error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Path(role): Path<String>,
    Json(partial_update): Json<serde_json::Value>,
) -> Result<Json<UpdateProfileResponse>, AdminProfileError> {
    info!(message = "admin.profile.update", role = %role);

    // TODO: Check admin role from JWT claims

    // Get database pool
    let pool = state.db_pool.as_ref()
        .ok_or_else(|| AdminProfileError::InternalError("Database not configured".to_string()))?;

    // Load existing profile
    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| AdminProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| AdminProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| AdminProfileError::InternalError(format!("Failed to get data column: {}", e)))?;

    let existing_profile: Profile = serde_json::from_value(data)
        .map_err(|e| AdminProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    // Merge partial update with existing profile
    let mut existing_value = serde_json::to_value(&existing_profile)
        .map_err(|e| AdminProfileError::InternalError(format!("Serialization failed: {}", e)))?;
    
    json_patch::merge(&mut existing_value, &partial_update);
    
    let updated_profile: Profile = serde_json::from_value(existing_value)
        .map_err(|e| AdminProfileError::ValidationError(format!("Invalid profile structure: {}", e)))?;

    // Validate merged profile
    ProfileValidator::validate(&updated_profile)
        .map_err(|e| AdminProfileError::ValidationError(format!("Profile validation failed: {}", e)))?;

    // Update in Postgres
    let now = Utc::now();
    let data = serde_json::to_value(&updated_profile)
        .map_err(|e| AdminProfileError::InternalError(format!("Serialization failed: {}", e)))?;

    sqlx::query("UPDATE profiles SET data = $1, updated_at = $2 WHERE role = $3")
        .bind(&data)
        .bind(now)
        .bind(&role)
        .execute(pool)
        .await
        .map_err(|e| AdminProfileError::DatabaseError(format!("Failed to update profile: {}", e)))?;

    info!(message = "admin.profile.updated", role = %role);
    
    Ok(Json(UpdateProfileResponse {
        role: role.clone(),
        updated_at: now.to_rfc3339(),
    }))
}

/// D9: Publish profile (sign with Vault)
///
/// Signs profile with Vault HMAC and updates database.
#[utoipa::path(
    post,
    path = "/admin/profiles/{role}/publish",
    tag = "admin",
    params(
        ("role" = String, Path, description = "Role to publish")
    ),
    responses(
        (status = 200, description = "Profile signed", body = PublishProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not admin"),
        (status = 404, description = "Profile not found"),
        (status = 500, description = "Vault or database error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn publish_profile(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> Result<Json<PublishProfileResponse>, AdminProfileError> {
    info!(message = "admin.profile.publish", role = %role);

    // TODO: Check admin role from JWT claims
    // TODO: Extract email from JWT for signed_by field

    // Get database pool
    let pool = state.db_pool.as_ref()
        .ok_or_else(|| AdminProfileError::InternalError("Database not configured".to_string()))?;

    // Load profile
    let row = sqlx::query("SELECT data FROM profiles WHERE role = $1")
        .bind(&role)
        .fetch_optional(pool)
        .await
        .map_err(|e| AdminProfileError::DatabaseError(format!("Database query failed: {}", e)))?
        .ok_or_else(|| AdminProfileError::NotFound(format!("Profile not found for role: {}", role)))?;

    let data: serde_json::Value = row.try_get("data")
        .map_err(|e| AdminProfileError::InternalError(format!("Failed to get data column: {}", e)))?;

    let mut profile: Profile = serde_json::from_value(data)
        .map_err(|e| AdminProfileError::InternalError(format!("Failed to deserialize profile: {}", e)))?;

    // CRITICAL: Remove old signature before signing (avoid circular signing)
    // The signature must be computed on the profile WITHOUT the signature field
    profile.signature = None;

    // Create Vault client
    let vault_config = VaultConfig::from_env()
        .map_err(|e| AdminProfileError::VaultError(format!("Vault config error: {}", e)))?;
    
    let vault_client = VaultClient::new(vault_config)
        .await
        .map_err(|e| AdminProfileError::VaultError(format!("Vault client error: {}", e)))?;

    // Sign profile with Vault Transit
    let transit = TransitOps::new(vault_client);
    
    // Ensure key exists (idempotent)
    transit.ensure_key("profile-signing")
        .await
        .map_err(|e| AdminProfileError::VaultError(format!("Failed to ensure Vault key: {}", e)))?;
    
    // Serialize profile data for signing with canonical key ordering
    // This is critical for HMAC verification to work correctly with Postgres JSONB
    let value = serde_json::to_value(&profile)
        .map_err(|e| AdminProfileError::InternalError(format!("Failed to convert to JSON value: {}", e)))?;
    let profile_data = serde_json::to_string(&canonical_sort_json(&value))
        .map_err(|e| AdminProfileError::InternalError(format!("Serialization failed: {}", e)))?;

    // DEBUG: Log the canonical JSON being signed
    info!(
        message = "admin.profile.signing_data",
        role = %role,
        json_length = profile_data.len(),
        json_preview = %&profile_data[..profile_data.len().min(200)],
        "Canonical JSON for signing"
    );
    
    // DEBUG: Save full canonical JSON to file for analysis
    if let Err(e) = std::fs::write(
        format!("/tmp/sign_{}.json", role),
        &profile_data
    ) {
        error!("Failed to write debug file: {}", e);
    }

    let signature = transit.sign_hmac("profile-signing", profile_data.as_bytes(), Some("sha2-256"))
        .await
        .map_err(|e| AdminProfileError::VaultError(format!("Vault signing failed: {}", e)))?;

    // Update profile with signature
    let now = Utc::now();
    profile.signature = Some(crate::profile::schema::Signature {
        algorithm: "sha2-256".to_string(),
        vault_key: "transit/keys/profile-signing".to_string(),
        signed_at: Some(now.to_rfc3339()),
        signed_by: Some("admin@example.com".to_string()), // TODO: Extract from JWT claims
        signature: Some(signature.clone()),
    });

    // Save updated profile to database
    let data = serde_json::to_value(&profile)
        .map_err(|e| AdminProfileError::InternalError(format!("Serialization failed: {}", e)))?;

    sqlx::query("UPDATE profiles SET data = $1, updated_at = $2 WHERE role = $3")
        .bind(&data)
        .bind(now)
        .bind(&role)
        .execute(pool)
        .await
        .map_err(|e| AdminProfileError::DatabaseError(format!("Failed to update profile: {}", e)))?;

    info!(message = "admin.profile.published", role = %role, signature = %signature);
    
    Ok(Json(PublishProfileResponse {
        role: role.clone(),
        signature,
        signed_at: now.to_rfc3339(),
    }))
}
