mod detection;
mod pseudonym;
mod redaction;
mod policy;
mod state;
mod audit;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/status", get(status_handler));

    let port = std::env::var("GUARD_PORT")
        .unwrap_or_else(|_| "8089".to_string())
        .parse::<u16>()
        .expect("GUARD_PORT must be a valid port number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Privacy Guard starting on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn status_handler() -> &'static str {
    "Privacy Guard - OK"
}
