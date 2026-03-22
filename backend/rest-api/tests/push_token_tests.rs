mod common;

use axum::body::Body;
use axum::http::{StatusCode, header};
use common::{TEST_PASSWORD, TestApp};
use tower::ServiceExt;

#[tokio::test]
async fn register_token_201() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
        "platform": "ios"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        resp["token"],
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd"
    );
    assert_eq!(resp["platform"], "ios");
    assert!(resp["id"].is_string());
    assert!(resp["created_at"].is_string());
}

#[tokio::test]
async fn register_same_token_idempotent() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
        "platform": "ios"
    });

    let (status1, resp1) = app.post_json_with_auth("/push-tokens", &body, &token).await;
    let (status2, resp2) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status1, StatusCode::CREATED);
    assert_eq!(status2, StatusCode::CREATED);
    // Same token ID returned (upsert)
    assert_eq!(resp1["id"], resp2["id"]);
}

#[tokio::test]
async fn unregister_token_204() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
        "platform": "ios"
    });
    app.post_json_with_auth("/push-tokens", &body, &token).await;

    let (status, _) = app
        .delete_with_auth(
            "/push-tokens/a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn unregister_nonexistent_404() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let (status, body) = app
        .delete_with_auth("/push-tokens/nonexistent-token", &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn invalid_platform_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
        "platform": "windows"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

// ── Auth guard tests (401) ──────────────────────────────────────────

#[tokio::test]
async fn register_token_without_auth_401() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
        "platform": "ios"
    });
    let (status, _) = app.post_json("/push-tokens", &body).await;

    assert_ne!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn unregister_token_without_auth_401() {
    let app = TestApp::new().await;

    let (status, body) = app
        .delete_with_auth("/push-tokens/some-token", "invalid-token")
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    common::assert_error(&body, "UNAUTHORIZED");
}

// ── Validation edge cases (400) ─────────────────────────────────────

#[tokio::test]
async fn register_empty_token_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "",
        "platform": "ios"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn register_android_platform_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd",
        "platform": "android"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn register_token_wrong_length_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "abc123",
        "platform": "ios"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn register_token_non_hex_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        "platform": "ios"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn register_token_63_chars_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a".repeat(63),
        "platform": "ios"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn register_token_65_chars_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let body = serde_json::json!({
        "token": "a".repeat(65),
        "platform": "ios"
    });
    let (status, resp) = app.post_json_with_auth("/push-tokens", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn register_missing_token_field_422() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/push-tokens")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({"platform": "ios"})).unwrap(),
        ))
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn register_missing_platform_field_422() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/push-tokens")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({"token": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abcd"})).unwrap(),
        ))
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn register_empty_body_422() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/push-tokens")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({})).unwrap(),
        ))
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
