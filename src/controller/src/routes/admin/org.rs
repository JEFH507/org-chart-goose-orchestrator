// Phase 5 Workstream D: Org chart admin endpoints (D10-D12)

use axum::{
    extract::{State, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use tracing::{info, error};
use chrono::Utc;
use crate::AppState;
use crate::org::csv_parser::{CsvParser, CsvError};

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportResponse {
    pub import_id: i32,
    pub filename: String,
    pub users_created: i32,
    pub users_updated: i32,
    pub status: String,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportHistoryResponse {
    pub imports: Vec<ImportRecord>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportRecord {
    pub id: i32,
    pub filename: String,
    pub uploaded_by: String,
    pub uploaded_at: String,
    pub users_created: i32,
    pub users_updated: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrgTreeResponse {
    pub tree: Vec<OrgNode>,
    pub total_users: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrgNode {
    pub user_id: i32,
    pub name: String,
    pub role: String,
    pub email: String,
    pub department: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reports: Vec<OrgNode>,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum OrgError {
    NotFound(String),
    Forbidden(String),
    ValidationError(String),
    DatabaseError(String),
    InternalError(String),
}

impl IntoResponse for OrgError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            OrgError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            OrgError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            OrgError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            OrgError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            OrgError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<CsvError> for OrgError {
    fn from(err: CsvError) -> Self {
        match err {
            CsvError::ParseError(msg) | CsvError::ValidationError(msg) => OrgError::ValidationError(msg),
            CsvError::CircularReference(chain) => OrgError::ValidationError(format!("Circular reference detected: {:?}", chain)),
            CsvError::InvalidRole(role) => OrgError::ValidationError(format!("Invalid role: {}", role)),
            CsvError::DuplicateEmail(email) => OrgError::ValidationError(format!("Duplicate email: {}", email)),
            CsvError::DatabaseError(msg) => OrgError::DatabaseError(msg),
        }
    }
}

// ============================================================================
// D10: POST /admin/org/import - Upload CSV
// ============================================================================

/// Upload org chart CSV file
///
/// Expected CSV format:
/// ```csv
/// user_id,reports_to_id,name,role,email,department
/// 1,,Alice Smith,finance,alice@example.com,Finance
/// 2,1,Bob Jones,legal,bob@example.com,Legal
/// ```
///
/// Validation:
/// - All roles must exist in profiles table
/// - No circular references in reports_to_id chain
/// - Email uniqueness within CSV
/// - Upserts users (insert new, update existing)
#[utoipa::path(
    post,
    path = "/admin/org/import",
    request_body(content = String, description = "CSV file content", content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Import successful", body = ImportResponse),
        (status = 400, description = "Validation error", body = serde_json::Value),
        (status = 403, description = "Forbidden - admin only", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "admin"
)]
pub async fn import_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ImportResponse>), OrgError> {
    info!(message = "admin.org.import.start");

    // TODO: Validate admin role from JWT claims

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| OrgError::InternalError("Database not configured".to_string()))?;

    // Extract file from multipart form
    let mut filename = String::new();
    let mut csv_content = String::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| OrgError::InternalError(format!("Failed to read multipart field: {}", e)))? {
        
        let field_name = field.name().unwrap_or("").to_string();
        
        if field_name == "file" {
            filename = field.file_name().unwrap_or("unknown.csv").to_string();
            let data = field.bytes().await
                .map_err(|e| OrgError::InternalError(format!("Failed to read file bytes: {}", e)))?;
            csv_content = String::from_utf8(data.to_vec())
                .map_err(|e| OrgError::ValidationError(format!("Invalid UTF-8 in CSV: {}", e)))?;
        }
    }

    if csv_content.is_empty() {
        return Err(OrgError::ValidationError("No file uploaded".to_string()));
    }

    // Create import record (pending status)
    let import_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO org_imports (filename, uploaded_by, uploaded_at, status) VALUES ($1, $2, NOW(), 'pending') RETURNING id"
    )
    .bind(&filename)
    .bind("admin@example.com") // TODO: Extract from JWT claims
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(message = "import.record.insert.error", error = %e);
        OrgError::DatabaseError(format!("Failed to create import record: {}", e))
    })?;

    info!(message = "import.record.created", import_id = import_id);

    // Update status to processing
    sqlx::query("UPDATE org_imports SET status = 'processing' WHERE id = $1")
        .bind(import_id)
        .execute(pool)
        .await
        .map_err(|e| OrgError::DatabaseError(format!("Failed to update import status: {}", e)))?;

    // Parse and validate CSV
    let parser = CsvParser::new(pool.clone());
    let rows = parser.parse_csv(&csv_content)?;
    parser.validate_rows(&rows).await?;

    // Upsert users
    let (created, updated) = parser.upsert_users(&rows).await?;

    // Update import record with results
    let now = Utc::now();
    sqlx::query(
        "UPDATE org_imports SET users_created = $1, users_updated = $2, status = 'complete' WHERE id = $3"
    )
    .bind(created)
    .bind(updated)
    .bind(import_id)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(message = "import.record.update.error", import_id = import_id, error = %e);
        OrgError::DatabaseError(format!("Failed to update import record: {}", e))
    })?;

    info!(
        message = "admin.org.import.complete",
        import_id = import_id,
        created = created,
        updated = updated
    );

    Ok((StatusCode::CREATED, Json(ImportResponse {
        import_id,
        filename,
        users_created: created,
        users_updated: updated,
        status: "complete".to_string(),
        uploaded_at: now.to_rfc3339(),
    })))
}

// ============================================================================
// D11: GET /admin/org/imports - Import history
// ============================================================================

/// Get org chart import history
///
/// Returns list of all CSV imports with metadata
#[utoipa::path(
    get,
    path = "/admin/org/imports",
    responses(
        (status = 200, description = "Import history", body = ImportHistoryResponse),
        (status = 403, description = "Forbidden - admin only", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "admin"
)]
pub async fn get_import_history(
    State(state): State<AppState>,
) -> Result<Json<ImportHistoryResponse>, OrgError> {
    info!(message = "admin.org.imports.get");

    // TODO: Validate admin role from JWT claims

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| OrgError::InternalError("Database not configured".to_string()))?;

    // Get all imports ordered by most recent first
    let records = sqlx::query_as::<_, (i32, String, String, chrono::DateTime<Utc>, i32, i32, String)>(
        r#"
        SELECT id, filename, uploaded_by, uploaded_at, users_created, users_updated, status
        FROM org_imports
        ORDER BY uploaded_at DESC
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(message = "imports.query.error", error = %e);
        OrgError::DatabaseError(format!("Failed to fetch import history: {}", e))
    })?;

    let total = records.len() as i64;
    let imports: Vec<ImportRecord> = records
        .into_iter()
        .map(|(id, filename, uploaded_by, uploaded_at, users_created, users_updated, status)| {
            ImportRecord {
                id,
                filename,
                uploaded_by,
                uploaded_at: uploaded_at.to_rfc3339(),
                users_created,
                users_updated,
                status,
            }
        })
        .collect();

    info!(message = "admin.org.imports.retrieved", count = total);

    Ok(Json(ImportHistoryResponse { imports, total }))
}

// ============================================================================
// D12: GET /admin/org/tree - Org chart hierarchy
// ============================================================================

/// Get org chart as hierarchical tree
///
/// Returns nested tree structure starting from root users (no manager)
/// Each node includes direct reports recursively
#[utoipa::path(
    get,
    path = "/admin/org/tree",
    responses(
        (status = 200, description = "Org chart tree", body = OrgTreeResponse),
        (status = 403, description = "Forbidden - admin only", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "admin"
)]
pub async fn get_org_tree(
    State(state): State<AppState>,
) -> Result<Json<OrgTreeResponse>, OrgError> {
    info!(message = "admin.org.tree.get");

    // TODO: Validate admin role from JWT claims

    let pool = state.db_pool.as_ref()
        .ok_or_else(|| OrgError::InternalError("Database not configured".to_string()))?;

    // Get all users
    let all_users = sqlx::query_as::<_, (i32, Option<i32>, String, String, String, String)>(
        "SELECT user_id, reports_to_id, name, role, email, department FROM org_users ORDER BY user_id"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(message = "org.tree.query.error", error = %e);
        OrgError::DatabaseError(format!("Failed to fetch org users: {}", e))
    })?;

    let total_users = all_users.len() as i32;

    // Build tree structure
    let tree = build_tree(&all_users);

    info!(message = "admin.org.tree.built", total_users = total_users);

    Ok(Json(OrgTreeResponse { tree, total_users }))
}

// ============================================================================
// Helper: Build recursive tree
// ============================================================================

fn build_tree(users: &[(i32, Option<i32>, String, String, String, String)]) -> Vec<OrgNode> {
    // Find root users (no manager)
    let roots: Vec<i32> = users
        .iter()
        .filter(|(_, reports_to, _, _, _, _)| reports_to.is_none())
        .map(|(user_id, _, _, _, _, _)| *user_id)
        .collect();

    // Build nodes for each root
    roots.into_iter()
        .filter_map(|user_id| build_node(user_id, users))
        .collect()
}

fn build_node(user_id: i32, all_users: &[(i32, Option<i32>, String, String, String, String)]) -> Option<OrgNode> {
    // Find this user's data
    let user = all_users.iter().find(|(uid, _, _, _, _, _)| *uid == user_id)?;
    let (_, _, name, role, email, department) = user;

    // Find all direct reports
    let direct_reports: Vec<i32> = all_users
        .iter()
        .filter(|(_, reports_to, _, _, _, _)| *reports_to == Some(user_id))
        .map(|(uid, _, _, _, _, _)| *uid)
        .collect();

    // Recursively build child nodes
    let reports: Vec<OrgNode> = direct_reports
        .into_iter()
        .filter_map(|child_id| build_node(child_id, all_users))
        .collect();

    Some(OrgNode {
        user_id,
        name: name.clone(),
        role: role.clone(),
        email: email.clone(),
        department: department.clone(),
        reports,
    })
}
