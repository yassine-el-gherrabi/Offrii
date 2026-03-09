use std::collections::VecDeque;
use std::sync::{Arc, Mutex as StdMutex};

/// Non-breached password that passes OWASP policy checks, shared across all test modules.
#[allow(dead_code)]
pub const TEST_PASSWORD: &str = "Str0ng!P@ssw0rd#2026x";

/// Alternate strong password for change/reset tests.
#[allow(dead_code)]
pub const NEW_PASSWORD: &str = "N3wStr0ng!P@ss#2026z";

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::routing::get;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use testcontainers::ContainerAsync;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use tower::ServiceExt;
use tower_http::trace::TraceLayer;

use rest_api::AppState;
use rest_api::config::database::{create_pg_pool, create_redis_client};
use rest_api::errors::AppError;
use rest_api::handlers::health::health_check;
use rest_api::handlers::{auth, categories, items, push_tokens, share_links, shared, users};
use rest_api::repositories::category_repo::PgCategoryRepo;
use rest_api::repositories::item_repo::PgItemRepo;
use rest_api::repositories::push_token_repo::PgPushTokenRepo;
use rest_api::repositories::refresh_token_repo::PgRefreshTokenRepo;
use rest_api::repositories::share_link_repo::PgShareLinkRepo;
use rest_api::repositories::user_repo::PgUserRepo;
use rest_api::services::auth_service::PgAuthService;
use rest_api::services::category_service::PgCategoryService;
use rest_api::services::health_check::PgHealthCheck;
use rest_api::services::item_service::PgItemService;
use rest_api::services::push_token_service::PgPushTokenService;
use rest_api::services::share_link_service::PgShareLinkService;
use rest_api::services::user_service::PgUserService;
use rest_api::traits::{
    AuthService, CategoryRepo, CategoryService, EmailService, HealthCheck, ItemRepo, ItemService,
    NotificationOutcome, NotificationRequest, NotificationService, PushTokenRepo, PushTokenService,
    RefreshTokenRepo, ShareLinkRepo, ShareLinkService, UserRepo, UserService,
};
use rest_api::utils::jwt::JwtKeys;

struct SpyEmailService {
    last_code: Arc<StdMutex<Option<String>>>,
}

#[async_trait::async_trait]
impl EmailService for SpyEmailService {
    async fn send_password_reset_code(&self, _to: &str, code: &str) -> Result<(), AppError> {
        *self.last_code.lock().unwrap() = Some(code.to_string());
        Ok(())
    }
}

#[allow(dead_code)]
pub struct SpyNotificationService {
    pub sent: Arc<StdMutex<Vec<(String, String, String)>>>,
    /// Per-call outcomes to return; defaults to `Sent` when empty.
    pub outcomes: Arc<StdMutex<VecDeque<NotificationOutcome>>>,
}

#[allow(dead_code)]
impl SpyNotificationService {
    pub fn new() -> Self {
        Self {
            sent: Arc::new(StdMutex::new(Vec::new())),
            outcomes: Arc::new(StdMutex::new(VecDeque::new())),
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for SpyNotificationService {
    async fn send_batch(&self, messages: &[NotificationRequest]) -> Vec<NotificationOutcome> {
        let mut sent = self.sent.lock().unwrap();
        let mut outcomes = self.outcomes.lock().unwrap();
        messages
            .iter()
            .map(|m| {
                sent.push((m.device_token.clone(), m.title.clone(), m.body.clone()));
                outcomes.pop_front().unwrap_or(NotificationOutcome::Sent)
            })
            .collect()
    }
}

#[allow(dead_code)]
pub struct TestApp {
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<Redis>,
    pub router: Router,
    pub db: PgPool,
    pub redis: redis::Client,
    pub last_reset_code: Arc<StdMutex<Option<String>>>,
}

#[allow(dead_code)]
impl TestApp {
    pub async fn new() -> Self {
        let pg_container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .unwrap();
        let redis_container = Redis::default().start().await.unwrap();

        let pg_host = pg_container.get_host().await.unwrap();
        let pg_port = pg_container.get_host_port_ipv4(5432).await.unwrap();
        let pg_url = format!("postgres://postgres:postgres@{pg_host}:{pg_port}/postgres");

        let redis_host = redis_container.get_host().await.unwrap();
        let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
        let redis_url = format!("redis://{redis_host}:{redis_port}");

        let db = create_pg_pool(&pg_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();

        let redis = create_redis_client(&redis_url).unwrap();
        let jwt = Arc::new(JwtKeys::generate().unwrap());

        // Wire DI — clone user_repo before it's consumed by auth
        let user_repo: Arc<dyn UserRepo> = Arc::new(PgUserRepo::new(db.clone()));
        let refresh_token_repo: Arc<dyn RefreshTokenRepo> =
            Arc::new(PgRefreshTokenRepo::new(db.clone()));

        let last_reset_code = Arc::new(StdMutex::new(None));
        let email_service: Arc<dyn EmailService> = Arc::new(SpyEmailService {
            last_code: last_reset_code.clone(),
        });

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

        // Share link service (before user_svc which consumes user_repo/item_repo)
        let share_link_repo: Arc<dyn ShareLinkRepo> = Arc::new(PgShareLinkRepo::new(db.clone()));
        let share_link_svc: Arc<dyn ShareLinkService> = Arc::new(PgShareLinkService::new(
            db.clone(),
            share_link_repo,
            item_repo.clone(),
            user_repo.clone(),
            "http://localhost:3000".to_string(),
        ));

        let user_svc: Arc<dyn UserService> =
            Arc::new(PgUserService::new(user_repo, item_repo, category_repo));
        let push_token_svc: Arc<dyn PushTokenService> =
            Arc::new(PgPushTokenService::new(push_token_repo));

        let redis_for_app = redis.clone();
        let state = AppState {
            auth,
            jwt,
            redis: redis_for_app,
            health,
            items,
            categories,
            users: user_svc,
            push_tokens: push_token_svc,
            share_links: share_link_svc,
        };

        let router = Router::new()
            .route("/health", get(health_check))
            .nest("/auth", auth::router())
            .nest("/items", items::router())
            .nest("/categories", categories::router())
            .nest("/users", users::router())
            .nest("/push-tokens", push_tokens::router())
            .nest("/share-links", share_links::router())
            .nest("/shared", shared::router())
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        Self {
            _pg_container: pg_container,
            _redis_container: redis_container,
            router,
            db,
            redis,
            last_reset_code,
        }
    }

    pub async fn post_json(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn post_json_with_auth(
        &self,
        uri: &str,
        body: &Value,
        token: &str,
    ) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn post_with_auth(&self, uri: &str, token: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn post_empty(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn register_user(&self, email: &str, password: &str) -> (StatusCode, Value) {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });
        self.post_json("/auth/register", &body).await
    }

    /// Register a user, asserting that registration succeeds (201).
    /// Use this when registration is a precondition, not the focus of the test.
    pub async fn setup_user(&self, email: &str, password: &str) -> Value {
        let (status, body) = self.register_user(email, password).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "precondition failed: registration should return 201, got {status}: {body}"
        );
        body
    }

    pub async fn register_user_with_name(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> (StatusCode, Value) {
        let body = serde_json::json!({
            "email": email,
            "password": password,
            "display_name": display_name,
        });
        self.post_json("/auth/register", &body).await
    }

    pub async fn get_with_auth(&self, uri: &str, token: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn put_json_with_auth(
        &self,
        uri: &str,
        body: &Value,
        token: &str,
    ) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("PUT")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn patch_json_with_auth(
        &self,
        uri: &str,
        body: &Value,
        token: &str,
    ) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("PATCH")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn delete_with_auth(&self, uri: &str, token: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    /// GET without auth, requesting JSON via Accept header.
    pub async fn get_json_no_auth(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header(header::ACCEPT, "application/json")
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    pub async fn get_with_accept(&self, uri: &str, accept: &str) -> (StatusCode, Vec<u8>) {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header(header::ACCEPT, accept)
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (status, bytes.to_vec())
    }

    pub async fn get_no_auth(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            (status, serde_json::from_slice(&bytes).unwrap())
        }
    }

    /// Create an item for a user, returning the item response body.
    /// Asserts 201 status.
    pub async fn create_item(&self, token: &str, body: &Value) -> Value {
        let (status, resp) = self.post_json_with_auth("/items", body, token).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "precondition failed: create_item should return 201, got {status}: {resp}"
        );
        resp
    }

    /// Create a category for a user, returning the category response body.
    /// Asserts 201 status.
    pub async fn create_category(&self, token: &str, body: &Value) -> Value {
        let (status, resp) = self.post_json_with_auth("/categories", body, token).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "precondition failed: create_category should return 201, got {status}: {resp}"
        );
        resp
    }

    /// Register a user and return the access token.
    pub async fn setup_user_token(&self, email: &str, password: &str) -> String {
        let body = self.setup_user(email, password).await;
        body["tokens"]["access_token"]
            .as_str()
            .expect("access_token should be a string")
            .to_string()
    }

    /// Retrieve the last reset code sent by the SpyEmailService.
    /// Polls until the code is available or timeout (5s) to avoid flaky sleeps.
    pub async fn get_last_reset_code(&self) -> Option<String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(code) = self.last_reset_code.lock().unwrap().clone() {
                return Some(code);
            }
            if std::time::Instant::now() >= deadline {
                return None;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    pub async fn post_raw(
        &self,
        uri: &str,
        body: &[u8],
        content_type: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method("POST").uri(uri);
        if let Some(ct) = content_type {
            builder = builder.header(header::CONTENT_TYPE, ct);
        }
        let req = builder.body(Body::from(body.to_vec())).unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            // Some Axum rejections return plain text, not JSON
            match serde_json::from_slice(&bytes) {
                Ok(v) => (status, v),
                Err(_) => (
                    status,
                    Value::String(String::from_utf8_lossy(&bytes).into()),
                ),
            }
        }
    }

    pub async fn post_with_raw_auth(
        &self,
        uri: &str,
        auth_header_value: &str,
    ) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::AUTHORIZATION, auth_header_value)
            .body(Body::empty())
            .unwrap();

        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();

        if bytes.is_empty() {
            (status, Value::Null)
        } else {
            match serde_json::from_slice(&bytes) {
                Ok(v) => (status, v),
                Err(_) => (
                    status,
                    Value::String(String::from_utf8_lossy(&bytes).into()),
                ),
            }
        }
    }
}

/// Assert that the response body has the expected `error.code` and a non-empty `error.message`.
#[allow(dead_code)]
pub fn assert_error(body: &Value, expected_code: &str) {
    assert_eq!(
        body["error"]["code"].as_str(),
        Some(expected_code),
        "expected error.code={expected_code}, got body={body}"
    );
    let msg = body["error"]["message"]
        .as_str()
        .expect("error.message should be a string");
    assert!(!msg.is_empty(), "error.message should not be empty");
}
