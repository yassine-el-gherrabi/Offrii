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
use rest_api::handlers::health::{health_check, health_live};
use rest_api::handlers::{auth, categories, items};
use rest_api::repositories::category_repo::PgCategoryRepo;
use rest_api::repositories::item_repo::PgItemRepo;
use rest_api::repositories::refresh_token_repo::PgRefreshTokenRepo;
use rest_api::repositories::user_repo::PgUserRepo;
use rest_api::services::auth_service::PgAuthService;
use rest_api::services::category_service::PgCategoryService;
use rest_api::services::health_check::PgHealthCheck;
use rest_api::services::item_service::PgItemService;
use rest_api::traits::{
    AuthService, CategoryRepo, CategoryService, HealthCheck, ItemRepo, ItemService,
    RefreshTokenRepo, UserRepo,
};
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

    // Wire DI
    let user_repo: Arc<dyn UserRepo> = Arc::new(PgUserRepo::new(db.clone()));
    let refresh_token_repo: Arc<dyn RefreshTokenRepo> =
        Arc::new(PgRefreshTokenRepo::new(db.clone()));

    let auth: Arc<dyn AuthService> = Arc::new(PgAuthService::new(
        db.clone(),
        user_repo,
        refresh_token_repo,
        jwt.clone(),
    ));
    let item_repo: Arc<dyn ItemRepo> = Arc::new(PgItemRepo::new(db.clone()));
    let items: Arc<dyn ItemService> =
        Arc::new(PgItemService::new(db.clone(), item_repo, redis.clone()));
    let category_repo: Arc<dyn CategoryRepo> = Arc::new(PgCategoryRepo::new(db.clone()));
    let categories: Arc<dyn CategoryService> =
        Arc::new(PgCategoryService::new(category_repo, redis.clone()));
    let health: Arc<dyn HealthCheck> = Arc::new(PgHealthCheck::new(db, redis));

    let state = AppState {
        auth,
        jwt,
        health,
        items,
        categories,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_check))
        .nest("/auth", auth::router())
        .nest("/items", items::router())
        .nest("/categories", categories::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.api_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "listening");

    axum::serve(listener, app).await?;

    Ok(())
}
