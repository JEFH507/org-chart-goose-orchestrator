mod auth;
mod guard_client;
mod api;
mod routes;

use axum::{
    routing::{get, post},
    Router,
    Json,
    middleware,
    extract::State,
};
// DEBUG: boot marker for container startup

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use utoipa::{ToSchema, OpenApi};
// TODO Phase 3: Re-enable Swagger UI integration after resolving axum 0.7 compatibility
// use utoipa_swagger_ui::SwaggerUi;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use auth::{JwtConfig, jwt_middleware};
use guard_client::GuardClient;
use api::openapi::ApiDoc;

#[derive(Serialize, ToSchema)]
struct StatusResponse<'a> {
    status: &'a str,
    version: &'a str,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
struct AuditEvent {
    // Minimal metadata-only shape per ADR-0008/0010; extend later
    source: String,
    category: String,
    action: String,
    // opaque id/reference/pseudonymized ids
    subject: Option<String>,
    #[serde(rename = "traceId")]
    trace_id: Option<String>,
    timestamp: Option<String>,
    // strictly metadata, no content-bearing fields
    metadata: Option<serde_json::Value>,
    // Optional content field for guard masking (Phase 2)
    #[serde(default)]
    content: Option<String>,
}

#[derive(Clone)]
struct AppState {
    guard_client: Arc<GuardClient>,
    jwt_config: Option<JwtConfig>,
}

#[tokio::main]
async fn main() {
    // Structured JSON logs by default
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .init();

    let port: u16 = std::env::var("CONTROLLER_PORT").ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8088);

    // Initialize guard client (Phase 2)
    let guard_client = Arc::new(GuardClient::from_env());
    if guard_client.is_enabled() {
        info!(message = "privacy guard integration enabled");
    } else {
        info!(message = "privacy guard integration disabled");
    }

    // Initialize JWT config (optional; skip JWT verification if not configured)
    let jwt_config = match JwtConfig::from_env() {
        Ok(config) => {
            info!(
                message = "JWT verification enabled",
                issuer = %config.issuer,
                audience = %config.audience
            );
            Some(config)
        }
        Err(e) => {
            warn!(
                message = "JWT verification disabled (missing config)",
                reason = %e
            );
            None
        }
    };

    let app_state = AppState {
        guard_client: guard_client.clone(),
        jwt_config: jwt_config.clone(),
    };

    // Build router with conditional JWT middleware
    let app = if let Some(config) = jwt_config {
        // Phase 3: Protected routes require JWT
        let protected = Router::new()
            .route("/audit/ingest", post(audit_ingest))
            .route("/tasks/route", post(routes::tasks::route_task))
            .route("/sessions", get(routes::sessions::list_sessions))
            .route("/sessions", post(routes::sessions::create_session))
            .route("/approvals", post(routes::approvals::submit_approval))
            .route("/profiles/:role", get(routes::profiles::get_profile))
            .route_layer(middleware::from_fn_with_state(config, jwt_middleware));

        // Public routes (status + OpenAPI spec)
        Router::new()
            .route("/status", get(status))
            .route("/api-docs/openapi.json", get(openapi_spec))
            .merge(protected)
            .with_state(app_state)
            .fallback(fallback_501)
    } else {
        // No JWT verification (dev mode without OIDC)
        Router::new()
            .route("/status", get(status))
            .route("/api-docs/openapi.json", get(openapi_spec))
            .route("/audit/ingest", post(audit_ingest))
            .route("/tasks/route", post(routes::tasks::route_task))
            .route("/sessions", get(routes::sessions::list_sessions))
            .route("/sessions", post(routes::sessions::create_session))
            .route("/approvals", post(routes::approvals::submit_approval))
            .route("/profiles/:role", get(routes::profiles::get_profile))
            .with_state(app_state)
            .fallback(fallback_501)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(message = "controller starting", port = port);
    let listener = TcpListener::bind(addr).await.expect("bind tcp");
    axum::serve(listener, app).await.unwrap();
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
async fn status() -> (StatusCode, Json<StatusResponse<'static>>) {
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
async fn audit_ingest(
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

/// Get OpenAPI specification
///
/// Returns the OpenAPI 3.0 specification in JSON format.
async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn fallback_501() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
