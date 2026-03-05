use std::sync::Arc;

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
use rest_api::handlers::auth;
use rest_api::handlers::health::health_check;
use rest_api::repositories::refresh_token_repo::PgRefreshTokenRepo;
use rest_api::repositories::user_repo::PgUserRepo;
use rest_api::services::auth_service::PgAuthService;
use rest_api::services::health_check::PgHealthCheck;
use rest_api::services::token_cache::RedisTokenCache;
use rest_api::traits::{AuthService, HealthCheck, RefreshTokenRepo, TokenCache, UserRepo};
use rest_api::utils::jwt::JwtKeys;

#[allow(dead_code)]
pub struct TestApp {
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<Redis>,
    pub router: Router,
    pub db: PgPool,
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
        let jwt = Arc::new(JwtKeys::from_env().unwrap());

        // Wire DI
        let user_repo: Arc<dyn UserRepo> = Arc::new(PgUserRepo::new(db.clone()));
        let refresh_token_repo: Arc<dyn RefreshTokenRepo> =
            Arc::new(PgRefreshTokenRepo::new(db.clone()));
        let token_cache: Arc<dyn TokenCache> = Arc::new(RedisTokenCache::new(redis.clone()));

        let auth: Arc<dyn AuthService> = Arc::new(PgAuthService::new(
            db.clone(),
            user_repo,
            refresh_token_repo,
            token_cache,
            jwt.clone(),
        ));
        let health: Arc<dyn HealthCheck> = Arc::new(PgHealthCheck::new(db.clone(), redis));

        let state = AppState { auth, jwt, health };

        let router = Router::new()
            .route("/health", get(health_check))
            .nest("/auth", auth::router())
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        Self {
            _pg_container: pg_container,
            _redis_container: redis_container,
            router,
            db,
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
            (status, serde_json::from_slice(&bytes).unwrap())
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
