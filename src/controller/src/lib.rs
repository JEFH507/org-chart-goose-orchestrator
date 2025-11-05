// Library exports for testing
pub mod auth;
pub mod guard_client;
pub mod api;
pub mod routes;
pub mod models;
pub mod repository;

// Phase 4: Lifecycle management (lives outside controller for reusability)
#[path = "../../lifecycle/mod.rs"]
pub mod lifecycle;

use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use tracing::{info, warn};
use crate::guard_client::GuardClient;
use crate::auth::JwtConfig;

#[derive(Clone)]
pub struct AppState {
    pub guard_client: Arc<GuardClient>,
    pub jwt_config: Option<JwtConfig>,
    pub db_pool: Option<PgPool>,
}

impl AppState {
    pub fn new(guard_client: Arc<GuardClient>, jwt_config: Option<JwtConfig>) -> Self {
        Self {
            guard_client,
            jwt_config,
            db_pool: None,
        }
    }

    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }
}

// Re-export types needed by OpenAPI
#[derive(Serialize, ToSchema)]
pub struct StatusResponse<'a> {
    pub status: &'a str,
    pub version: &'a str,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AuditEvent {
    pub source: String,
    pub category: String,
    pub action: String,
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    pub trace_id: Option<String>,
    pub timestamp: Option<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub content: Option<String>,
}

/// Get system status
///
/// Returns the health status and version of the controller service.
#[utoipa::path(
    get,
    path = "/status",
    tag = "system",
    responses(
        (status = 200, description = "System status", body = StatusResponse),
    )
)]
pub async fn status() -> (StatusCode, Json<StatusResponse<'static>>) {
    // Version from Cargo
    let version = env!("CARGO_PKG_VERSION");
    (
        StatusCode::OK,
        Json(StatusResponse { status: "ok", version })
    )
}

/// Ingest audit event
///
/// Ingests an audit event for logging. Content fields will be automatically
/// masked by the Privacy Guard if enabled.
#[utoipa::path(
    post,
    path = "/audit/ingest",
    tag = "system",
    request_body = AuditEvent,
    responses(
        (status = 202, description = "Audit event accepted"),
        (status = 401, description = "Unauthorized - missing or invalid JWT"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn audit_ingest(
    State(state): State<AppState>,
    Json(mut event): Json<AuditEvent>,
) -> StatusCode {
    // Phase 2: Apply privacy guard if enabled and content present
    let redactions = if let Some(content) = event.content.as_deref() {
        match state.guard_client.mask_text(content, &event.source, event.trace_id.as_deref()).await {
            Ok(Some(mask_response)) => {
                // Update event content with masked text
                event.content = Some(mask_response.masked_text);
                Some(mask_response.redactions)
            }
            Ok(None) => {
                // Guard disabled or failed (fail-open)
                None
            }
            Err(e) => {
                warn!(message = "guard error", error = %e);
                None
            }
        }
    } else {
        None
    };

    // Log only metadata; do not persist in Phase 1
    info!(
        message = "audit.ingest",
        source = %event.source,
        category = %event.category,
        action = %event.action,
        subject = ?event.subject,
        trace_id = ?event.trace_id,
        has_metadata = %event.metadata.as_ref().map(|m| !m.is_null()).unwrap_or(false),
        has_content = %event.content.is_some(),
        redactions = ?redactions
    );
    StatusCode::ACCEPTED
}
