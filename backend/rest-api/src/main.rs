use std::sync::Arc;

use axum::Router;
use axum::http::{Method, header};
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use rest_api::AppState;
use rest_api::config::app::Config;
use rest_api::config::database::{create_pg_pool, create_redis_client};
use rest_api::handlers::health::{health_check, health_live};
use rest_api::handlers::{
    admin, auth, categories, circles, community_wishes, friends, items, notifications, push_tokens,
    share_links, shared, upload, users, wish_messages,
};
use rest_api::repositories::category_repo::PgCategoryRepo;
use rest_api::repositories::circle_event_repo::PgCircleEventRepo;
use rest_api::repositories::circle_invite_repo::PgCircleInviteRepo;
use rest_api::repositories::circle_item_repo::PgCircleItemRepo;
use rest_api::repositories::circle_member_repo::PgCircleMemberRepo;
use rest_api::repositories::circle_repo::PgCircleRepo;
use rest_api::repositories::community_wish_repo::PgCommunityWishRepo;
use rest_api::repositories::friend_repo::PgFriendRepo;
use rest_api::repositories::item_repo::PgItemRepo;
use rest_api::repositories::notification_repo::PgNotificationRepo;
use rest_api::repositories::push_token_repo::PgPushTokenRepo;
use rest_api::repositories::refresh_token_repo::PgRefreshTokenRepo;
use rest_api::repositories::share_link_repo::PgShareLinkRepo;
use rest_api::repositories::user_repo::PgUserRepo;
use rest_api::repositories::wish_message_repo::PgWishMessageRepo;
use rest_api::repositories::wish_report_repo::PgWishReportRepo;
use rest_api::services::apns_notification_service::ApnsNotificationService;
use rest_api::services::auth_service::PgAuthService;
use rest_api::services::category_service::PgCategoryService;
use rest_api::services::circle_service::PgCircleService;
use rest_api::services::community_wish_service::PgCommunityWishService;
use rest_api::services::email_service::ResendEmailService;
use rest_api::services::friend_service::PgFriendService;
use rest_api::services::health_check::PgHealthCheck;
use rest_api::services::item_service::PgItemService;
use rest_api::services::moderation_service::{NoopModerationService, OpenAIModerationService};
use rest_api::services::oauth_verifier::OAuthVerifier;
use rest_api::services::push_token_service::PgPushTokenService;
use rest_api::services::reminder_service::PgReminderService;
use rest_api::services::share_link_service::PgShareLinkService;
use rest_api::services::upload_service::{NoopUploadService, R2UploadService};
use rest_api::services::user_service::PgUserService;
use rest_api::services::wish_message_service::PgWishMessageService;
use rest_api::traits::{
    AuthService, CategoryRepo, CategoryService, CircleEventRepo, CircleInviteRepo, CircleItemRepo,
    CircleMemberRepo, CircleRepo, CircleService, CommunityWishRepo, CommunityWishService,
    EmailService, FriendRepo, FriendService, HealthCheck, ItemRepo, ItemService, ModerationService,
    NotificationRepo, NotificationService, PushTokenRepo, PushTokenService, RefreshTokenRepo,
    ReminderService, ShareLinkRepo, ShareLinkService, UploadService, UserRepo, UserService,
    WishMessageRepo, WishMessageService, WishReportRepo,
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
        config.app_base_url.clone(),
    ));

    let oauth_verifier = Arc::new(OAuthVerifier::new(
        config.google_client_id.clone(),
        config.apns_bundle_id.clone(),
    ));

    let auth: Arc<dyn AuthService> = Arc::new(PgAuthService::new(
        db.clone(),
        user_repo.clone(),
        refresh_token_repo,
        jwt.clone(),
        redis.clone(),
        email_service,
        oauth_verifier,
    ));
    let item_repo: Arc<dyn ItemRepo> = Arc::new(PgItemRepo::new(db.clone()));
    let circle_item_repo: Arc<dyn CircleItemRepo> = Arc::new(PgCircleItemRepo::new(db.clone()));
    let items: Arc<dyn ItemService> = Arc::new(PgItemService::new(
        db.clone(),
        item_repo.clone(),
        circle_item_repo.clone(),
        redis.clone(),
    ));
    let category_repo: Arc<dyn CategoryRepo> = Arc::new(PgCategoryRepo::new(db.clone()));
    let categories: Arc<dyn CategoryService> =
        Arc::new(PgCategoryService::new(category_repo.clone()));
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
        config.app_base_url.clone(),
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

    // Friend repo
    let friend_repo: Arc<dyn FriendRepo> = Arc::new(PgFriendRepo::new(db.clone()));

    // Circle service
    let circle_repo: Arc<dyn CircleRepo> = Arc::new(PgCircleRepo::new(db.clone()));
    let circle_member_repo: Arc<dyn CircleMemberRepo> =
        Arc::new(PgCircleMemberRepo::new(db.clone()));
    let circle_invite_repo: Arc<dyn CircleInviteRepo> =
        Arc::new(PgCircleInviteRepo::new(db.clone()));
    let circle_event_repo: Arc<dyn CircleEventRepo> = Arc::new(PgCircleEventRepo::new(db.clone()));
    let notification_repo: Arc<dyn NotificationRepo> =
        Arc::new(PgNotificationRepo::new(db.clone()));
    let share_rule_repo: Arc<dyn rest_api::traits::CircleShareRuleRepo> = Arc::new(
        rest_api::repositories::circle_share_rule_repo::PgCircleShareRuleRepo::new(db.clone()),
    );

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
        notification_repo.clone(),
        friend_repo.clone(),
        redis.clone(),
    ));

    // Friend service
    let friend_svc: Arc<dyn FriendService> = Arc::new(PgFriendService::new(
        db.clone(),
        friend_repo,
        user_repo.clone(),
        push_token_repo.clone(),
        notification_svc.clone(),
        notification_repo.clone(),
    ));

    // Community wish service
    let wish_repo: Arc<dyn CommunityWishRepo> = Arc::new(PgCommunityWishRepo::new(db.clone()));
    let report_repo: Arc<dyn WishReportRepo> = Arc::new(PgWishReportRepo::new(db.clone()));
    let message_repo: Arc<dyn WishMessageRepo> = Arc::new(PgWishMessageRepo::new(db.clone()));

    let moderation_svc: Arc<dyn ModerationService> = if config.moderation_enabled {
        let api_key = config
            .openai_api_key
            .expect("OPENAI_API_KEY must be set when MODERATION_ENABLED=true");
        Arc::new(OpenAIModerationService::new(api_key))
    } else {
        Arc::new(NoopModerationService)
    };

    let community_wish_svc: Arc<dyn CommunityWishService> = Arc::new(PgCommunityWishService::new(
        db.clone(),
        wish_repo.clone(),
        report_repo,
        user_repo.clone(),
        push_token_repo.clone(),
        notification_svc.clone(),
        notification_repo.clone(),
        moderation_svc,
        redis.clone(),
    ));

    let wish_message_svc: Arc<dyn WishMessageService> = Arc::new(PgWishMessageService::new(
        wish_repo,
        message_repo,
        user_repo.clone(),
        push_token_repo.clone(),
        notification_svc.clone(),
        notification_repo.clone(),
    ));

    // Reminder service (not in AppState — used only by the CRON job)
    let reminder_svc: Arc<dyn ReminderService> = Arc::new(PgReminderService::new(
        user_repo,
        item_repo,
        push_token_repo,
        redis.clone(),
        notification_svc,
    ));

    // Upload service — use R2 if credentials are configured, otherwise noop
    let upload_svc: Arc<dyn UploadService> = {
        let r2_account_id = std::env::var("R2_ACCOUNT_ID").ok();
        let r2_access_key = std::env::var("R2_ACCESS_KEY_ID").ok();
        let r2_secret_key = std::env::var("R2_SECRET_ACCESS_KEY").ok();
        let r2_bucket = std::env::var("R2_BUCKET_NAME").unwrap_or_else(|_| "offrii-media".into());
        let r2_public_url = std::env::var("R2_PUBLIC_URL").unwrap_or_default();

        if let (Some(account_id), Some(access_key), Some(secret_key)) =
            (r2_account_id, r2_access_key, r2_secret_key)
        {
            tracing::info!("R2 upload service configured (bucket: {r2_bucket})");
            Arc::new(
                R2UploadService::new(
                    &account_id,
                    &access_key,
                    &secret_key,
                    r2_bucket,
                    r2_public_url,
                )
                .await,
            )
        } else {
            tracing::warn!("R2 not configured — upload endpoint will return test URLs");
            Arc::new(NoopUploadService)
        }
    };

    // notification_repo already created above

    let state = AppState {
        auth,
        jwt,
        db: db.clone(),
        redis: redis.clone(),
        health,
        items,
        categories,
        users: user_svc,
        push_tokens: push_token_svc,
        share_links: share_link_svc,
        circles: circle_svc,
        friends: friend_svc,
        community_wishes: community_wish_svc,
        wish_messages: wish_message_svc,
        uploads: upload_svc,
        notifications: notification_repo,
        share_rules: share_rule_repo,
        app_base_url: config.app_base_url,
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
        .nest("/me/notifications", notifications::router())
        .nest("/share-links", share_links::router())
        .nest("/shared", shared::router())
        .nest("/circles", circles::router())
        .route("/join/{token}", get(circles::join_page))
        .route("/favicon.png", get(serve_favicon))
        .route("/favicon.ico", get(serve_favicon))
        .route("/legal/privacy", get(legal_privacy))
        .route("/legal/terms", get(legal_terms))
        .nest("/me", friends::router())
        .nest("/users", friends::search_router())
        .nest("/community/wishes", community_wishes::router())
        .merge(wish_messages::router())
        .nest("/upload", upload::router())
        .nest("/admin", admin::router())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024)) // 10 MB
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]),
        )
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state)
        .merge(SwaggerUi::new("/docs").url(
            "/api-doc/openapi.json",
            rest_api::openapi::ApiDoc::openapi(),
        ));

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

async fn security_headers(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    resp
}

/// Serve the Offrii favicon as a static PNG.
async fn serve_favicon() -> impl IntoResponse {
    const FAVICON_PNG: &[u8] = include_bytes!("../assets/favicon-32x32.png");
    (
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=604800"),
        ],
        FAVICON_PNG,
    )
}

async fn legal_privacy() -> axum::response::Html<String> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Politique de confidentialité — Offrii</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 680px; margin: 40px auto; padding: 0 20px; color: #1a1a1a; line-height: 1.7; }}
h1 {{ color: #FF6B6B; font-size: 24px; }}
h2 {{ color: #333; font-size: 18px; margin-top: 32px; }}
p {{ margin: 12px 0; }}
.date {{ color: #999; font-size: 14px; }}
</style>
</head>
<body>
<h1>Politique de confidentialité</h1>
<p class="date">Dernière mise à jour : 20 mars 2026</p>

<h2>1. Données collectées</h2>
<p>Offrii collecte les données suivantes lors de votre utilisation de l'application :</p>
<ul>
<li><strong>Compte</strong> : adresse email, nom d'affichage, photo de profil (optionnelle)</li>
<li><strong>Listes de souhaits</strong> : vos envies, catégories, priorités, liens et images associés</li>
<li><strong>Cercles</strong> : les groupes que vous créez ou rejoignez, et les membres associés</li>
<li><strong>Entraide</strong> : les besoins communautaires que vous publiez ou auxquels vous contribuez</li>
<li><strong>Messages</strong> : les messages échangés dans le cadre de l'entraide</li>
</ul>

<h2>2. Utilisation des données</h2>
<p>Vos données sont utilisées exclusivement pour :</p>
<ul>
<li>Le fonctionnement de l'application (authentification, partage, notifications)</li>
<li>L'envoi d'emails transactionnels (vérification, réinitialisation de mot de passe)</li>
<li>La modération du contenu communautaire (via analyse automatisée)</li>
</ul>

<h2>3. Partage des données</h2>
<p>Vos données ne sont jamais vendues. Elles sont partagées uniquement avec :</p>
<ul>
<li><strong>Les membres de vos cercles</strong> : vos envies partagées et votre profil public</li>
<li><strong>Les utilisateurs de l'entraide</strong> : vos besoins publiés et messages associés</li>
<li><strong>Nos sous-traitants techniques</strong> : hébergement (Hetzner, Allemagne), stockage (Cloudflare R2), emails (Resend)</li>
</ul>

<h2>4. Stockage et sécurité</h2>
<p>Vos données sont stockées sur des serveurs sécurisés en Europe (Hetzner, Allemagne). Les communications sont chiffrées en transit (TLS). Les mots de passe sont hachés avec Argon2.</p>

<h2>5. Vos droits</h2>
<p>Conformément au RGPD, vous pouvez :</p>
<ul>
<li>Accéder à vos données depuis votre profil dans l'application</li>
<li>Modifier ou supprimer vos données à tout moment</li>
<li>Supprimer votre compte (suppression définitive de toutes vos données)</li>
<li>Nous contacter : <a href="mailto:contact@offrii.com">contact@offrii.com</a></li>
</ul>

<h2>6. Connexion via Google / Apple</h2>
<p>Si vous utilisez la connexion Google ou Apple, nous recevons uniquement votre adresse email, votre nom et votre photo de profil. Nous n'accédons à aucune autre donnée de votre compte Google ou Apple.</p>

<p style="margin-top:40px;color:#999;font-size:13px;">Offrii — Offre, partage, fais plaisir.<br>Contact : <a href="mailto:contact@offrii.com" style="color:#FF6B6B;">contact@offrii.com</a></p>
</body>
</html>"#.to_string())
}

async fn legal_terms() -> axum::response::Html<String> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Conditions d'utilisation — Offrii</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 680px; margin: 40px auto; padding: 0 20px; color: #1a1a1a; line-height: 1.7; }}
h1 {{ color: #FF6B6B; font-size: 24px; }}
h2 {{ color: #333; font-size: 18px; margin-top: 32px; }}
p {{ margin: 12px 0; }}
.date {{ color: #999; font-size: 14px; }}
</style>
</head>
<body>
<h1>Conditions d'utilisation</h1>
<p class="date">Dernière mise à jour : 20 mars 2026</p>

<h2>1. Objet</h2>
<p>Offrii est une application de listes de souhaits et d'entraide communautaire. Ces conditions régissent votre utilisation du service.</p>

<h2>2. Inscription</h2>
<p>Vous devez fournir une adresse email valide. Vous êtes responsable de la confidentialité de vos identifiants. Un seul compte par personne.</p>

<h2>3. Utilisation acceptable</h2>
<p>Vous vous engagez à :</p>
<ul>
<li>Ne pas publier de contenu illicite, offensant ou trompeur</li>
<li>Ne pas usurper l'identité d'autrui</li>
<li>Ne pas utiliser l'application à des fins commerciales non autorisées</li>
<li>Respecter les autres utilisateurs dans les échanges communautaires</li>
</ul>

<h2>4. Contenu utilisateur</h2>
<p>Vous conservez la propriété de votre contenu. En le publiant, vous accordez à Offrii une licence limitée pour l'afficher dans le cadre du service. Le contenu signalé peut être modéré ou supprimé.</p>

<h2>5. Entraide communautaire</h2>
<p>L'entraide est un service de mise en relation. Offrii n'est pas responsable des engagements pris entre utilisateurs. Les besoins publiés doivent être légitimes et respectueux.</p>

<h2>6. Suppression de compte</h2>
<p>Vous pouvez supprimer votre compte à tout moment depuis l'application. La suppression est définitive et entraîne la suppression de toutes vos données.</p>

<h2>7. Limitation de responsabilité</h2>
<p>Offrii est fourni "tel quel". Nous ne garantissons pas la disponibilité permanente du service. Nous ne sommes pas responsables des échanges entre utilisateurs.</p>

<h2>8. Modification</h2>
<p>Ces conditions peuvent être modifiées. Les utilisateurs seront informés des changements significatifs.</p>

<p style="margin-top:40px;color:#999;font-size:13px;">Offrii — Offre, partage, fais plaisir.<br>Contact : <a href="mailto:contact@offrii.com" style="color:#FF6B6B;">contact@offrii.com</a></p>
</body>
</html>"#.to_string())
}
