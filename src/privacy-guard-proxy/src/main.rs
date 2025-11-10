mod control_panel;
mod proxy;
mod state;

use axum::{
    routing::{get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use state::ProxyState;

#[tokio::main]
async fn main() {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let privacy_guard_url = std::env::var("PRIVACY_GUARD_URL")
        .unwrap_or_else(|_| "http://privacy-guard:8089".to_string());
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8090".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    // Initialize shared state
    let state = ProxyState::new(privacy_guard_url.clone());
    
    info!("Privacy Guard Proxy starting...");
    info!("Privacy Guard URL: {}", privacy_guard_url);
    info!("Default mode: Auto");

    // Build Control Panel routes
    let control_panel_routes = Router::new()
        .route("/ui", get(control_panel::serve_ui))
        .route("/api/mode", get(control_panel::get_mode))
        .route("/api/mode", put(control_panel::set_mode))
        .route("/api/detection", get(control_panel::get_detection_method))
        .route("/api/detection", put(control_panel::set_detection_method))
        .route("/api/status", get(control_panel::get_status))
        .route("/api/activity", get(control_panel::get_activity));

    // Build Proxy routes
    let proxy_routes = Router::new()
        .route("/v1/chat/completions", post(proxy::proxy_chat_completions))
        .route("/v1/completions", post(proxy::proxy_completions));

    // Combine routes
    let app = Router::new()
        .merge(control_panel_routes)
        .merge(proxy_routes)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Privacy Guard Proxy listening on {}", addr);
    info!("📊 Control Panel UI: http://localhost:{}/ui", port);
    info!("🔒 Proxy endpoints: http://localhost:{}/v1/*", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
