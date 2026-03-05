use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use rest_api::AppState;
use rest_api::config::app::Config;
use rest_api::config::database::{create_pg_pool, create_redis_client};
use rest_api::handlers::auth;
use rest_api::handlers::health::health_check;
use rest_api::utils::jwt::JwtKeys;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let env_filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(err) => {
            eprintln!("Invalid RUST_LOG value: {err}. Falling back to 'info'.");
            EnvFilter::new("info")
        }
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = Config::from_env()?;

    tracing::info!(port = config.api_port, "starting rest-api");

    let db = create_pg_pool(&config.database_url).await?;
    let redis = create_redis_client(&config.redis_url)?;
    let jwt = Arc::new(JwtKeys::from_env()?);

    let state = AppState { db, redis, jwt };

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.api_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "listening");

    axum::serve(listener, app).await?;

    Ok(())
}
