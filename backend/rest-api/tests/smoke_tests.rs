//! Smoke tests: real HTTP server over TCP.
//!
//! These tests spin up the full Axum server bound to a random port,
//! send real HTTP requests via reqwest, and verify responses.
//! They complement the in-memory oneshot tests in `auth_tests.rs`
//! by exercising the real TCP/HTTP serving path.

mod common;

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use common::TestApp;

// ── SmokeTestApp helper ──────────────────────────────────────────────

struct SmokeTestApp {
    _app: TestApp,
    server_handle: JoinHandle<()>,
    client: reqwest::Client,
    base_url: String,
}

impl SmokeTestApp {
    async fn new() -> Self {
        let app = TestApp::new().await;
        let router = app.router.clone();

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind to random port");
        let addr: SocketAddr = listener.local_addr().expect("failed to get local address");

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("server error");
        });

        let base_url = format!("http://{addr}");
        let client = reqwest::Client::new();

        Self {
            _app: app,
            server_handle,
            client,
            base_url,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Drop for SmokeTestApp {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn smoke_health_returns_200_with_json_shape() {
    let app = SmokeTestApp::new().await;

    let resp = app
        .client
        .get(app.url("/health"))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("response is not valid JSON");

    assert_eq!(body["status"], "ok");
    assert_eq!(body["db"], "connected");
    assert_eq!(body["redis"], "connected");
}

#[tokio::test]
async fn smoke_full_auth_flow() {
    let app = SmokeTestApp::new().await;
    let email = "smoketest@example.com";
    let password = "strongpass123";

    // ── Step 1: Register ──────────────────────────────────────────
    let resp = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .expect("register request failed");

    assert_eq!(resp.status(), 201, "register should return 201 Created");

    let reg_body: serde_json::Value = resp.json().await.unwrap();

    let reg_access = reg_body["tokens"]["access_token"]
        .as_str()
        .expect("access_token should be a string");
    let _reg_refresh = reg_body["tokens"]["refresh_token"]
        .as_str()
        .expect("refresh_token should be a string");
    assert_eq!(reg_body["tokens"]["token_type"], "Bearer");
    assert!(reg_body["tokens"]["expires_in"].is_u64());
    assert_eq!(reg_body["user"]["email"], email);
    assert!(reg_body["user"]["id"].is_string());

    // ── Step 2: Login ─────────────────────────────────────────────
    let resp = app
        .client
        .post(app.url("/auth/login"))
        .json(&serde_json::json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .expect("login request failed");

    assert_eq!(resp.status(), 200, "login should return 200 OK");

    let login_body: serde_json::Value = resp.json().await.unwrap();

    let login_access = login_body["tokens"]["access_token"]
        .as_str()
        .expect("login access_token should be a string");
    let login_refresh = login_body["tokens"]["refresh_token"]
        .as_str()
        .expect("login refresh_token should be a string")
        .to_string();
    assert_eq!(login_body["tokens"]["token_type"], "Bearer");
    assert_eq!(login_body["user"]["email"], email);

    // Login tokens should differ from register tokens
    assert_ne!(login_access, reg_access);

    // ── Step 3: Refresh ───────────────────────────────────────────
    let resp = app
        .client
        .post(app.url("/auth/refresh"))
        .json(&serde_json::json!({
            "refresh_token": login_refresh,
        }))
        .send()
        .await
        .expect("refresh request failed");

    assert_eq!(resp.status(), 200, "refresh should return 200 OK");

    let refresh_body: serde_json::Value = resp.json().await.unwrap();

    let refreshed_access = refresh_body["tokens"]["access_token"]
        .as_str()
        .expect("refreshed access_token should be a string");
    assert_ne!(refreshed_access, login_access);
    assert_eq!(refresh_body["tokens"]["token_type"], "Bearer");
    assert!(refresh_body["tokens"]["expires_in"].is_u64());

    // ── Step 4: Logout ────────────────────────────────────────────
    let resp = app
        .client
        .post(app.url("/auth/logout"))
        .header("Authorization", format!("Bearer {refreshed_access}"))
        .send()
        .await
        .expect("logout request failed");

    assert_eq!(resp.status(), 204, "logout should return 204 No Content");
}
