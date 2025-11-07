use goose_controller::{AppState, auth, guard_client, api, routes, status, health, audit_ingest};
use goose_controller::middleware as goose_middleware;

use axum::{
    routing::{get, post, put},
    Router,
    Json,
    middleware,
};
// DEBUG: boot marker for container startup

use axum::http::StatusCode;
use sqlx::postgres::PgPoolOptions;
use redis::Client as RedisClient;
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

    // Initialize Redis connection (Phase 4 - Idempotency)
    let redis_client = match std::env::var("REDIS_URL") {
        Ok(url) => {
            info!(message = "connecting to redis", url = %url);
            match RedisClient::open(url) {
                Ok(client) => {
                    match redis::aio::ConnectionManager::new(client).await {
                        Ok(conn) => {
                            info!(message = "redis connected");
                            Some(conn)
                        }
                        Err(e) => {
                            warn!(message = "redis connection manager failed", error = %e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!(message = "redis client creation failed", error = %e);
                    None
                }
            }
        }
        Err(_) => {
            warn!(message = "REDIS_URL not set, idempotency deduplication disabled");
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

    // Phase 6 A5: Initialize Vault client for signature verification
    let vault_client = match goose_controller::vault::VaultClient::from_env().await {
        Ok(client) => {
            info!(message = "Vault client initialized - signature verification enabled");
            // Health check
            match client.health_check().await {
                Ok(_) => info!(message = "Vault health check passed"),
                Err(e) => warn!(message = "Vault health check failed", error = %e),
            }
            Some(client)
        }
        Err(e) => {
            warn!(
                message = "Vault client initialization failed",
                error = %e,
                note = "Profile signature verification disabled"
            );
            None
        }
    };

    let mut app_state = AppState::new(guard_client.clone(), jwt_config.clone());
    if let Some(pool) = db_pool {
        app_state = app_state.with_db_pool(pool);
    }
    if let Some(redis) = redis_client {
        app_state = app_state.with_redis_client(redis);
    }
    if let Some(vault) = vault_client {
        app_state = app_state.with_vault_client(vault);
    }

    // Check if idempotency middleware is enabled (Phase 4)
    let idempotency_enabled = std::env::var("IDEMPOTENCY_ENABLED")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);
    
    if idempotency_enabled && app_state.redis_client.is_some() {
        info!(message = "idempotency deduplication enabled");
    } else {
        info!(message = "idempotency deduplication disabled");
    }

    // Build router with conditional JWT and idempotency middleware
    let app = if let Some(config) = jwt_config {
        // Phase 3: Protected routes require JWT
        let mut protected = Router::new()
            .route("/audit/ingest", post(audit_ingest))
            .route("/tasks/route", post(routes::tasks::route_task))
            .route("/sessions", get(routes::sessions::list_sessions))
            .route("/sessions", post(routes::sessions::create_session))
            .route("/sessions/:id", get(routes::sessions::get_session))
            .route("/sessions/:id", put(routes::sessions::update_session))
            .route("/approvals", post(routes::approvals::submit_approval))
            .route("/profiles/:role", get(routes::profiles::get_profile))
            .route("/profiles/:role/config", get(routes::profiles::get_config))
            .route("/profiles/:role/goosehints", get(routes::profiles::get_goosehints))
            .route("/profiles/:role/gooseignore", get(routes::profiles::get_gooseignore))
            .route("/profiles/:role/local-hints", get(routes::profiles::get_local_hints))
            .route("/profiles/:role/recipes", get(routes::profiles::get_recipes))
            .route("/privacy/audit", post(routes::privacy::submit_audit_log))
            // Phase 5 Workstream D: Admin routes (D7-D12)
            .route("/admin/profiles", post(routes::admin::profiles::create_profile))
            .route("/admin/profiles/:role", put(routes::admin::profiles::update_profile))
            .route("/admin/profiles/:role/publish", post(routes::admin::profiles::publish_profile))
            .route("/admin/org/import", post(routes::admin::org::import_csv))
            .route("/admin/org/imports", get(routes::admin::org::get_import_history))
            .route("/admin/org/tree", get(routes::admin::org::get_org_tree));
        
        // Phase 4: Apply idempotency middleware if enabled (before JWT middleware)
        if idempotency_enabled {
            protected = protected.route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                goose_middleware::idempotency_middleware
            ));
        }
        
        protected = protected.route_layer(middleware::from_fn_with_state(config, jwt_middleware));

        // Public routes (status + health + OpenAPI spec)
        Router::new()
            .route("/status", get(status))
            .route("/health", get(health))
            .route("/api-docs/openapi.json", get(openapi_spec))
            .merge(protected)
            .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE)) // Phase 3: 1MB limit on all requests
            .with_state(app_state)
            .fallback(fallback_501)
    } else {
        // No JWT verification (dev mode without OIDC)
        let mut routes = Router::new()
            .route("/status", get(status))
            .route("/health", get(health))
            .route("/api-docs/openapi.json", get(openapi_spec))
            .route("/audit/ingest", post(audit_ingest))
            .route("/tasks/route", post(routes::tasks::route_task))
            .route("/sessions", get(routes::sessions::list_sessions))
            .route("/sessions", post(routes::sessions::create_session))
            .route("/sessions/:id", get(routes::sessions::get_session))
            .route("/sessions/:id", put(routes::sessions::update_session))
            .route("/approvals", post(routes::approvals::submit_approval))
            .route("/profiles/:role", get(routes::profiles::get_profile))
            .route("/profiles/:role/config", get(routes::profiles::get_config))
            .route("/profiles/:role/goosehints", get(routes::profiles::get_goosehints))
            .route("/profiles/:role/gooseignore", get(routes::profiles::get_gooseignore))
            .route("/profiles/:role/local-hints", get(routes::profiles::get_local_hints))
            .route("/profiles/:role/recipes", get(routes::profiles::get_recipes))
            .route("/privacy/audit", post(routes::privacy::submit_audit_log))
            // Phase 5 Workstream D: Admin routes (D7-D12)
            .route("/admin/profiles", post(routes::admin::profiles::create_profile))
            .route("/admin/profiles/:role", put(routes::admin::profiles::update_profile))
            .route("/admin/profiles/:role/publish", post(routes::admin::profiles::publish_profile))
            .route("/admin/org/import", post(routes::admin::org::import_csv))
            .route("/admin/org/imports", get(routes::admin::org::get_import_history))
            .route("/admin/org/tree", get(routes::admin::org::get_org_tree));
        
        // Phase 4: Apply idempotency middleware if enabled
        if idempotency_enabled {
            routes = routes.route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                goose_middleware::idempotency_middleware
            ));
        }
        
        routes
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
