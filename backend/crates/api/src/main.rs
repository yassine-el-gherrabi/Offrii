use std::env;

use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    let app = Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind to {addr}"));

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("server error");
}
