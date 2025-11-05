use goose_controller::{AppState, auth, guard_client, api, routes, status, audit_ingest};

use axum::{
    routing::{get, post},
    Router,
    Json,
    middleware,
};
// DEBUG: boot marker for container startup

use axum::http::StatusCode;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;
use utoipa::OpenApi;
// TODO Phase 3: Re-enable Swagger UI integration after resolving axum 0.7 compatibility
// use utoipa_swagger_ui::SwaggerUi;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use auth::{JwtConfig, jwt_middleware};
use guard_client::GuardClient;
use api::openapi::ApiDoc;

// Phase 3: Request body size limit (1MB for all POST requests)
const MAX_BODY_SIZE: usize = 1024 * 1024; // 1 MB

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

    // Initialize database pool (Phase 4)
    let db_pool = match std::env::var("DATABASE_URL") {
        Ok(url) => {
            info!(message = "connecting to database");
            match PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
            {
                Ok(pool) => {
                    info!(message = "database connected");
                    Some(pool)
                }
                Err(e) => {
                    warn!(message = "database connection failed", error = %e);
                    None
                }
            }
        }
        Err(_) => {
            warn!(message = "DATABASE_URL not set, session persistence disabled");
            None
        }
    };

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

    let mut app_state = AppState::new(guard_client.clone(), jwt_config.clone());
    if let Some(pool) = db_pool {
        app_state = app_state.with_db_pool(pool);
    }

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
            .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE)) // Phase 3: 1MB limit on all requests
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
            .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE)) // Phase 3: 1MB limit on all requests
            .with_state(app_state)
            .fallback(fallback_501)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(message = "controller starting", port = port);
    let listener = TcpListener::bind(addr).await.expect("bind tcp");
    axum::serve(listener, app).await.unwrap();
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
