// Library exports for testing
pub mod auth;
pub mod guard_client;
pub mod api;
pub mod routes;
pub mod models;
pub mod repository;
pub mod middleware;
pub mod policy; // Phase 5 Workstream C: RBAC/ABAC policy engine

// Phase 4: Lifecycle management (lives outside controller for reusability)
#[path = "../../lifecycle/mod.rs"]
pub mod lifecycle;

// Phase 5: Vault client (production-grade HashiCorp Vault integration)
#[path = "../../vault/mod.rs"]
pub mod vault;

// Phase 5: Profile system
#[path = "../../profile/mod.rs"]
pub mod profile;

// Phase 5: Org chart module
pub mod org;

use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use utoipa::ToSchema;
use tracing::{info, warn};
use crate::guard_client::GuardClient;
use crate::auth::JwtConfig;

#[derive(Clone)]
pub struct AppState {
    pub guard_client: Arc<GuardClient>,
    pub jwt_config: Option<JwtConfig>,
    pub db_pool: Option<PgPool>,
    pub redis_client: Option<ConnectionManager>,
}

impl AppState {
    pub fn new(guard_client: Arc<GuardClient>, jwt_config: Option<JwtConfig>) -> Self {
        Self {
            guard_client,
            jwt_config,
            db_pool: None,
            redis_client: None,
        }
    }

    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub fn with_redis_client(mut self, client: ConnectionManager) -> Self {
        self.redis_client = Some(client);
        self
    }
}

// Re-export types needed by OpenAPI
#[derive(Serialize, ToSchema)]
pub struct StatusResponse<'a> {
    pub status: &'a str,
    pub version: &'a str,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: String,
    pub redis: String,
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

/// Get health status with dependency checks
///
/// Returns detailed health status including database and Redis connectivity.
#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Health status with dependencies", body = HealthResponse),
    )
)]
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    use redis::AsyncCommands;
    
    let version = env!("CARGO_PKG_VERSION").to_string();
    
    // Check database connectivity
    let db_status = if let Some(pool) = &state.db_pool {
        match sqlx::query("SELECT 1").fetch_one(pool).await {
            Ok(_) => "connected".to_string(),
            Err(e) => format!("error: {}", e),
        }
    } else {
        "disabled".to_string()
    };
    
    // Check Redis connectivity
    let redis_status = if let Some(mut redis) = state.redis_client.clone() {
        // Use AsyncCommands trait for PING
        match redis.get::<_, Option<String>>("__ping__").await {
            Ok(_) => "connected".to_string(),
            Err(e) => format!("error: {}", e),
        }
    } else {
        "disabled".to_string()
    };
    
    let overall_status = if db_status.starts_with("error") || redis_status.starts_with("error") {
        "degraded".to_string()
    } else {
        "healthy".to_string()
    };
    
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: overall_status,
            version,
            database: db_status,
            redis: redis_status,
        })
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
