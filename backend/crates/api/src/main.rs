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

    let app = Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("server error");
}
