mod auth;

use axum::{
    routing::{get, post},
    Router,
    Json,
    middleware,
};
// DEBUG: boot marker for container startup

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use auth::{JwtConfig, jwt_middleware};

#[derive(Serialize)]
struct StatusResponse<'a> {
    status: &'a str,
    version: &'a str,
}

#[derive(Deserialize, Serialize, Debug)]
struct AuditEvent {
    // Minimal metadata-only shape per ADR-0008/0010; extend later
    source: String,
    category: String,
    action: String,
    // opaque id/reference/pseudonymized ids
    subject: Option<String>,
    traceId: Option<String>,
    timestamp: Option<String>,
    // strictly metadata, no content-bearing fields
    metadata: Option<serde_json::Value>,
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

    // Build router with conditional JWT middleware
    let app = if let Some(config) = jwt_config {
        // Protected routes require JWT
        let protected = Router::new()
            .route("/audit/ingest", post(audit_ingest))
            .route_layer(middleware::from_fn_with_state(config.clone(), jwt_middleware))
            .with_state(config);

        // Public routes
        Router::new()
            .route("/status", get(status))
            .merge(protected)
            .fallback(fallback_501)
    } else {
        // No JWT verification (dev mode without OIDC)
        Router::new()
            .route("/status", get(status))
            .route("/audit/ingest", post(audit_ingest))
            .fallback(fallback_501)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(message = "controller starting", port = port);
    let listener = TcpListener::bind(addr).await.expect("bind tcp");
    axum::serve(listener, app).await.unwrap();
}

async fn status() -> (StatusCode, Json<StatusResponse<'static>>) {
    // Version from Cargo
    let version = env!("CARGO_PKG_VERSION");
    (
        StatusCode::OK,
        Json(StatusResponse { status: "ok", version })
    )
}

async fn audit_ingest(Json(event): Json<AuditEvent>) -> StatusCode {
    // Log only metadata; do not persist in Phase 1
    info!(
        message = "audit.ingest",
        source = %event.source,
        category = %event.category,
        action = %event.action,
        subject = ?event.subject,
        traceId = ?event.traceId,
        has_metadata = %event.metadata.as_ref().map(|m| !m.is_null()).unwrap_or(false)
    );
    StatusCode::ACCEPTED
}

async fn fallback_501() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
