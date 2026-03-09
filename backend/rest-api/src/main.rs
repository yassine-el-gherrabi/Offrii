use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use rest_api::AppState;
use rest_api::config::app::Config;
use rest_api::config::database::{create_pg_pool, create_redis_client};
use rest_api::handlers::health::{health_check, health_live};
use rest_api::handlers::{
    auth, categories, circles, items, push_tokens, share_links, shared, users,
};
use rest_api::repositories::category_repo::PgCategoryRepo;
use rest_api::repositories::circle_event_repo::PgCircleEventRepo;
use rest_api::repositories::circle_invite_repo::PgCircleInviteRepo;
use rest_api::repositories::circle_item_repo::PgCircleItemRepo;
use rest_api::repositories::circle_member_repo::PgCircleMemberRepo;
use rest_api::repositories::circle_repo::PgCircleRepo;
use rest_api::repositories::item_repo::PgItemRepo;
use rest_api::repositories::push_token_repo::PgPushTokenRepo;
use rest_api::repositories::refresh_token_repo::PgRefreshTokenRepo;
use rest_api::repositories::share_link_repo::PgShareLinkRepo;
use rest_api::repositories::user_repo::PgUserRepo;
use rest_api::services::apns_notification_service::ApnsNotificationService;
use rest_api::services::auth_service::PgAuthService;
use rest_api::services::category_service::PgCategoryService;
use rest_api::services::circle_service::PgCircleService;
use rest_api::services::email_service::ResendEmailService;
use rest_api::services::health_check::PgHealthCheck;
use rest_api::services::item_service::PgItemService;
use rest_api::services::push_token_service::PgPushTokenService;
use rest_api::services::reminder_service::PgReminderService;
use rest_api::services::share_link_service::PgShareLinkService;
use rest_api::services::user_service::PgUserService;
use rest_api::traits::{
    AuthService, CategoryRepo, CategoryService, CircleEventRepo, CircleInviteRepo, CircleItemRepo,
    CircleMemberRepo, CircleRepo, CircleService, EmailService, HealthCheck, ItemRepo, ItemService,
    NotificationService, PushTokenRepo, PushTokenService, RefreshTokenRepo, ReminderService,
    ShareLinkRepo, ShareLinkService, UserRepo, UserService,
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

    // Wire DI — clone user_repo before it's consumed by auth
    let user_repo: Arc<dyn UserRepo> = Arc::new(PgUserRepo::new(db.clone()));
    let refresh_token_repo: Arc<dyn RefreshTokenRepo> =
        Arc::new(PgRefreshTokenRepo::new(db.clone()));

    let email_service: Arc<dyn EmailService> = Arc::new(ResendEmailService::new(
        &config.resend_api_key,
        config.email_from.clone(),
    ));

    let auth: Arc<dyn AuthService> = Arc::new(PgAuthService::new(
        db.clone(),
        user_repo.clone(),
        refresh_token_repo,
        jwt.clone(),
        redis.clone(),
        email_service,
    ));
    let item_repo: Arc<dyn ItemRepo> = Arc::new(PgItemRepo::new(db.clone()));
    let items: Arc<dyn ItemService> = Arc::new(PgItemService::new(
        db.clone(),
        item_repo.clone(),
        redis.clone(),
    ));
    let category_repo: Arc<dyn CategoryRepo> = Arc::new(PgCategoryRepo::new(db.clone()));
    let categories: Arc<dyn CategoryService> =
        Arc::new(PgCategoryService::new(category_repo.clone(), redis.clone()));
    let health: Arc<dyn HealthCheck> = Arc::new(PgHealthCheck::new(db.clone(), redis.clone()));

    // New services
    let push_token_repo: Arc<dyn PushTokenRepo> = Arc::new(PgPushTokenRepo::new(db.clone()));
    let user_svc: Arc<dyn UserService> = Arc::new(PgUserService::new(
        user_repo.clone(),
        item_repo.clone(),
        category_repo.clone(),
    ));
    let push_token_svc: Arc<dyn PushTokenService> =
        Arc::new(PgPushTokenService::new(push_token_repo.clone()));

    // Share link service
    let share_link_repo: Arc<dyn ShareLinkRepo> = Arc::new(PgShareLinkRepo::new(db.clone()));
    let share_link_svc: Arc<dyn ShareLinkService> = Arc::new(PgShareLinkService::new(
        db.clone(),
        share_link_repo,
        item_repo.clone(),
        user_repo.clone(),
        config.app_base_url,
    ));

    // APNs notification service
    let apns_key = std::fs::read(&config.apns_key_path)
        .map_err(|e| anyhow::anyhow!("Failed to read APNs key at {}: {e}", config.apns_key_path))?;
    let notification_svc: Arc<dyn NotificationService> = Arc::new(
        ApnsNotificationService::new(
            &apns_key,
            &config.apns_key_id,
            &config.apns_team_id,
            &config.apns_bundle_id,
            config.apns_sandbox,
        )
        .map_err(|e| anyhow::anyhow!("Failed to initialize APNs client: {e}"))?,
    );

    // Circle service
    let circle_repo: Arc<dyn CircleRepo> = Arc::new(PgCircleRepo::new(db.clone()));
    let circle_member_repo: Arc<dyn CircleMemberRepo> =
        Arc::new(PgCircleMemberRepo::new(db.clone()));
    let circle_invite_repo: Arc<dyn CircleInviteRepo> =
        Arc::new(PgCircleInviteRepo::new(db.clone()));
    let circle_item_repo: Arc<dyn CircleItemRepo> = Arc::new(PgCircleItemRepo::new(db.clone()));
    let circle_event_repo: Arc<dyn CircleEventRepo> = Arc::new(PgCircleEventRepo::new(db.clone()));
    let circle_svc: Arc<dyn CircleService> = Arc::new(PgCircleService::new(
        db.clone(),
        circle_repo,
        circle_member_repo,
        circle_invite_repo,
        circle_item_repo,
        circle_event_repo,
        item_repo.clone(),
        user_repo.clone(),
        push_token_repo.clone(),
        notification_svc.clone(),
        redis.clone(),
    ));

    // Reminder service (not in AppState — used only by the CRON job)
    let reminder_svc: Arc<dyn ReminderService> = Arc::new(PgReminderService::new(
        user_repo,
        item_repo,
        push_token_repo,
        redis.clone(),
        notification_svc,
    ));

    let state = AppState {
        auth,
        jwt,
        redis: redis.clone(),
        health,
        items,
        categories,
        users: user_svc,
        push_tokens: push_token_svc,
        share_links: share_link_svc,
        circles: circle_svc,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_check))
        .nest("/auth", auth::router())
        .nest("/items", items::router())
        .nest("/categories", categories::router())
        .nest("/users", users::router())
        .nest("/push-tokens", push_tokens::router())
        .nest("/share-links", share_links::router())
        .nest("/shared", shared::router())
        .nest("/circles", circles::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // CRON scheduler: run reminder job every hour at minute 0
    let sched = JobScheduler::new().await?;
    let reminder_redis = redis;
    sched
        .add(Job::new_async("0 0 * * * *", move |_, _| {
            let svc = reminder_svc.clone();
            let r = reminder_redis.clone();
            Box::pin(async move {
                rest_api::jobs::reminder_job::run(svc, r).await;
            })
        })?)
        .await?;
    sched.start().await?;

    let addr = format!("0.0.0.0:{}", config.api_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "listening");

    axum::serve(listener, app).await?;

    Ok(())
}
