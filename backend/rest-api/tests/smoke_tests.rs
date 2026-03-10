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
    let email = "test@example.com";
    let password = "xK9mQ2vL7nB4pR8sW3";

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

    // ── Step 5: Blacklisted token rejected ─────────────────────
    let resp = app
        .client
        .get(app.url("/categories"))
        .header("Authorization", format!("Bearer {refreshed_access}"))
        .send()
        .await
        .expect("post-logout request failed");

    assert_eq!(
        resp.status(),
        401,
        "access token should be rejected after logout"
    );
}

#[tokio::test]
async fn smoke_full_items_crud_flow() {
    let app = SmokeTestApp::new().await;
    let password = common::TEST_PASSWORD;

    // ── Step 0: Register and get access token ───────────────────
    let resp = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": "items-smoke@example.com",
            "password": password,
        }))
        .send()
        .await
        .expect("register failed");
    assert_eq!(resp.status(), 201);
    let reg: serde_json::Value = resp.json().await.unwrap();
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // ── Step 1: List items (empty) ──────────────────────────────
    let resp = app
        .client
        .get(app.url("/items"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list["pagination"]["total"], 0);
    assert_eq!(list["data"].as_array().unwrap().len(), 0);

    // ── Step 2: Create item (quick capture) ─────────────────────
    let resp = app
        .client
        .post(app.url("/items"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "name": "Smoke item" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let item: serde_json::Value = resp.json().await.unwrap();
    let item_id = item["id"].as_str().expect("item id should be a string");
    assert_eq!(item["name"], "Smoke item");
    assert_eq!(item["status"], "active");
    assert_eq!(item["priority"], 2); // default

    // ── Step 3: Get item by id ──────────────────────────────────
    let resp = app
        .client
        .get(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(fetched["id"], item_id);
    assert_eq!(fetched["name"], "Smoke item");

    // ── Step 4: Update item ─────────────────────────────────────
    let resp = app
        .client
        .put(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "name": "Updated smoke item",
            "priority": 1,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(updated["name"], "Updated smoke item");
    assert_eq!(updated["priority"], 1);

    // ── Step 5: Mark purchased ──────────────────────────────────
    let resp = app
        .client
        .put(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "status": "purchased" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let purchased: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(purchased["status"], "purchased");
    assert!(
        purchased["purchased_at"].is_string(),
        "purchased_at should be set"
    );

    // ── Step 5b: Re-purchase should be 409 ──────────────────────
    let resp = app
        .client
        .put(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "status": "purchased" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409, "re-purchase should return 409 Conflict");

    // ── Step 5c: Restore to active ────────────────────────────
    let resp = app
        .client
        .put(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "status": "active" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "restore should return 200 OK");
    let restored: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(restored["status"], "active");
    assert!(
        restored["purchased_at"].is_null(),
        "purchased_at should be cleared after restore"
    );

    // Re-purchase so subsequent steps still work with purchased state
    let resp = app
        .client
        .put(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "status": "purchased" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // ── Step 6: List with status filter ─────────────────────────
    let resp = app
        .client
        .get(app.url("/items?status=purchased"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list["pagination"]["total"], 1);
    assert_eq!(list["data"][0]["id"], item_id);

    // ── Step 7: Delete item (soft-delete) ───────────────────────
    let resp = app
        .client
        .delete(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // ── Step 8: Verify deleted — not in list, not gettable ──────
    let resp = app
        .client
        .get(app.url("/items"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list["pagination"]["total"], 0);

    let resp = app
        .client
        .get(app.url(&format!("/items/{item_id}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn smoke_claim_unclaim_flow() {
    let app = SmokeTestApp::new().await;
    let password = common::TEST_PASSWORD;

    // ── Register owner and claimer ──────────────────────────────
    let resp = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": "claim-smoke-owner@example.com",
            "password": password,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let reg: serde_json::Value = resp.json().await.unwrap();
    let owner_token = reg["tokens"]["access_token"].as_str().unwrap().to_string();

    let resp = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": "claim-smoke-claimer@example.com",
            "password": password,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let reg: serde_json::Value = resp.json().await.unwrap();
    let claimer_token = reg["tokens"]["access_token"].as_str().unwrap().to_string();

    // ── Create item ─────────────────────────────────────────────
    let resp = app
        .client
        .post(app.url("/items"))
        .bearer_auth(&owner_token)
        .json(&serde_json::json!({ "name": "Smoke gift" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let item: serde_json::Value = resp.json().await.unwrap();
    let item_id = item["id"].as_str().unwrap();
    assert_eq!(item["is_claimed"], false);

    // ── Claim ───────────────────────────────────────────────────
    let resp = app
        .client
        .post(app.url(&format!("/items/{item_id}/claim")))
        .bearer_auth(&claimer_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // ── Verify is_claimed via GET ───────────────────────────────
    let resp = app
        .client
        .get(app.url(&format!("/items/{item_id}")))
        .bearer_auth(&owner_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(fetched["is_claimed"], true);

    // ── Double-claim → 409 ──────────────────────────────────────
    let resp = app
        .client
        .post(app.url(&format!("/items/{item_id}/claim")))
        .bearer_auth(&claimer_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);

    // ── Unclaim ─────────────────────────────────────────────────
    let resp = app
        .client
        .delete(app.url(&format!("/items/{item_id}/claim")))
        .bearer_auth(&claimer_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // ── Verify is_claimed reset ─────────────────────────────────
    let resp = app
        .client
        .get(app.url(&format!("/items/{item_id}")))
        .bearer_auth(&owner_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(fetched["is_claimed"], false);

    // ── Self-claim → 400 ────────────────────────────────────────
    let resp = app
        .client
        .post(app.url(&format!("/items/{item_id}/claim")))
        .bearer_auth(&owner_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn smoke_full_categories_crud_flow() {
    let app = SmokeTestApp::new().await;
    let password = common::TEST_PASSWORD;

    // ── Step 0: Register and get access token ───────────────────
    let resp = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": "categories-smoke@example.com",
            "password": password,
        }))
        .send()
        .await
        .expect("register failed");
    assert_eq!(resp.status(), 201);
    let reg: serde_json::Value = resp.json().await.unwrap();
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // ── Step 1: List defaults (6) ─────────────────────────────
    let resp = app
        .client
        .get(app.url("/categories"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list.as_array().unwrap().len(), 6);

    // ── Step 2: Create "Voyages" with icon ────────────────────
    let resp = app
        .client
        .post(app.url("/categories"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "name": "Voyages", "icon": "plane" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let cat: serde_json::Value = resp.json().await.unwrap();
    let cat_id = cat["id"].as_str().unwrap();
    assert_eq!(cat["name"], "Voyages");
    assert_eq!(cat["icon"], "plane");
    assert_eq!(cat["is_default"], false);

    // ── Step 3: List → 7 ─────────────────────────────────────
    let resp = app
        .client
        .get(app.url("/categories"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list.as_array().unwrap().len(), 7);

    // ── Step 4: Update "Voyages" → "Travel" ───────────────────
    let resp = app
        .client
        .put(app.url(&format!("/categories/{cat_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "name": "Travel" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(updated["name"], "Travel");

    // ── Step 5: Delete "Travel" ───────────────────────────────
    let resp = app
        .client
        .delete(app.url(&format!("/categories/{cat_id}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // ── Step 6: List → back to 6 ─────────────────────────────
    let resp = app
        .client
        .get(app.url("/categories"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list.as_array().unwrap().len(), 6);

    // ── Step 7: Delete default → 400 ─────────────────────────
    let default_id = list.as_array().unwrap()[0]["id"].as_str().unwrap();
    let resp = app
        .client
        .delete(app.url(&format!("/categories/{default_id}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // ── Step 8: Update default name → 200 ────────────────────
    let resp = app
        .client
        .put(app.url(&format!("/categories/{default_id}")))
        .bearer_auth(token)
        .json(&serde_json::json!({ "name": "Renamed Default" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let renamed: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(renamed["name"], "Renamed Default");
    assert_eq!(renamed["is_default"], true);
}

#[tokio::test]
async fn smoke_full_push_token_flow() {
    let app = SmokeTestApp::new().await;
    let password = common::TEST_PASSWORD;
    let device_token = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd";

    // ── Step 0: Register and get access token ───────────────────
    let resp = app
        .client
        .post(app.url("/auth/register"))
        .json(&serde_json::json!({
            "email": "push-smoke@example.com",
            "password": password,
        }))
        .send()
        .await
        .expect("register failed");
    assert_eq!(resp.status(), 201);
    let reg: serde_json::Value = resp.json().await.unwrap();
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // ── Step 1: Register push token ─────────────────────────────
    let resp = app
        .client
        .post(app.url("/push-tokens"))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "token": device_token,
            "platform": "ios"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let pt: serde_json::Value = resp.json().await.unwrap();
    let pt_id = pt["id"].as_str().expect("id should be a string");
    assert_eq!(pt["token"], device_token);
    assert_eq!(pt["platform"], "ios");

    // ── Step 2: Idempotent re-register ──────────────────────────
    let resp = app
        .client
        .post(app.url("/push-tokens"))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "token": device_token,
            "platform": "ios"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let pt2: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(pt2["id"], pt_id, "upsert should return same ID");

    // ── Step 3: Unregister push token ───────────────────────────
    let resp = app
        .client
        .delete(app.url(&format!("/push-tokens/{device_token}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // ── Step 4: Unregister again → 404 ──────────────────────────
    let resp = app
        .client
        .delete(app.url(&format!("/push-tokens/{device_token}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
