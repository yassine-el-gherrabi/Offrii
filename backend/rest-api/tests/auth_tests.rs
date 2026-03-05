mod common;

use axum::http::StatusCode;

use common::{TestApp, assert_error};

// ---------------------------------------------------------------------------
// Register
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_success_201() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user("alice@example.com", "strongpass123")
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["tokens"]["access_token"].is_string());
    assert!(body["tokens"]["refresh_token"].is_string());
    assert_eq!(body["tokens"]["token_type"], "Bearer");
    assert!(body["tokens"]["expires_in"].is_u64());
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert!(body["user"]["id"].is_string());
    assert!(body["user"]["created_at"].is_string());

    // Verify 6 default categories were copied
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories WHERE user_id IS NOT NULL")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count.0, 6);
}

#[tokio::test]
async fn register_duplicate_email_409() {
    let app = TestApp::new().await;

    app.register_user("dup@example.com", "strongpass123").await;
    let (status, body) = app.register_user("dup@example.com", "strongpass123").await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn register_bad_email_400() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user("not-an-email", "strongpass123").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_short_password_400() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user("user@example.com", "short").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_with_display_name_201() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user_with_name("named@example.com", "strongpass123", "Alice")
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["user"]["display_name"], "Alice");
    assert!(body["tokens"]["access_token"].is_string());
}

#[tokio::test]
async fn register_malformed_json_400() {
    let app = TestApp::new().await;

    let (status, _) = app
        .post_raw("/auth/register", b"not json", Some("application/json"))
        .await;

    // Axum's Json extractor returns 400 for syntactically invalid JSON
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn register_missing_content_type_415() {
    let app = TestApp::new().await;

    let valid_json = serde_json::json!({
        "email": "test@example.com",
        "password": "strongpass123"
    });
    let (status, _) = app
        .post_raw(
            "/auth/register",
            &serde_json::to_vec(&valid_json).unwrap(),
            None,
        )
        .await;

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

// ---------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------

#[tokio::test]
async fn login_success_200() {
    let app = TestApp::new().await;
    app.register_user("bob@example.com", "strongpass123").await;

    let body = serde_json::json!({
        "email": "bob@example.com",
        "password": "strongpass123",
    });
    let (status, resp) = app.post_json("/auth/login", &body).await;

    assert_eq!(status, StatusCode::OK);
    assert!(resp["tokens"]["access_token"].is_string());
    assert!(resp["tokens"]["refresh_token"].is_string());
    assert_eq!(resp["tokens"]["token_type"], "Bearer");
    assert!(resp["tokens"]["expires_in"].is_u64());
    assert_eq!(resp["user"]["email"], "bob@example.com");
    assert!(resp["user"]["id"].is_string());
    assert!(resp["user"]["created_at"].is_string());
}

#[tokio::test]
async fn login_wrong_password_401() {
    let app = TestApp::new().await;
    app.register_user("carol@example.com", "strongpass123")
        .await;

    let body = serde_json::json!({
        "email": "carol@example.com",
        "password": "wrongpassword",
    });
    let (status, body) = app.post_json("/auth/login", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn login_nonexistent_email_401() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "nobody@example.com",
        "password": "strongpass123",
    });
    let (status, body) = app.post_json("/auth/login", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

// ---------------------------------------------------------------------------
// Refresh
// ---------------------------------------------------------------------------

#[tokio::test]
async fn refresh_success_200() {
    let app = TestApp::new().await;
    let (_, reg) = app.register_user("dan@example.com", "strongpass123").await;
    let old_access = reg["tokens"]["access_token"].as_str().unwrap().to_string();
    let old_refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    let body = serde_json::json!({ "refresh_token": old_refresh });
    let (status, resp) = app.post_json("/auth/refresh", &body).await;

    assert_eq!(status, StatusCode::OK);

    let new_access = resp["tokens"]["access_token"].as_str().unwrap();
    let new_refresh = resp["tokens"]["refresh_token"].as_str().unwrap();

    assert_ne!(new_access, old_access);
    assert_ne!(new_refresh, old_refresh);
    assert_eq!(resp["tokens"]["token_type"], "Bearer");
    assert!(resp["tokens"]["expires_in"].is_u64());
    // RefreshResponse has no `user` field
    assert!(resp["user"].is_null());
}

#[tokio::test]
async fn refresh_old_token_revoked_401() {
    let app = TestApp::new().await;
    let (_, reg) = app.register_user("eve@example.com", "strongpass123").await;
    let old_refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // First refresh succeeds
    let body = serde_json::json!({ "refresh_token": &old_refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::OK);

    // Second refresh with old token fails (revoked)
    let (status, body) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn refresh_invalid_token_401() {
    let app = TestApp::new().await;

    let body = serde_json::json!({ "refresh_token": "garbage.token.here" });
    let (status, body) = app.post_json("/auth/refresh", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn refresh_with_access_token_type_401() {
    let app = TestApp::new().await;
    let (_, reg) = app
        .register_user("typemix@example.com", "strongpass123")
        .await;
    let access_token = reg["tokens"]["access_token"].as_str().unwrap();

    // Send an access token where a refresh token is expected
    let body = serde_json::json!({ "refresh_token": access_token });
    let (status, body) = app.post_json("/auth/refresh", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

// ---------------------------------------------------------------------------
// Logout
// ---------------------------------------------------------------------------

#[tokio::test]
async fn logout_success_204() {
    let app = TestApp::new().await;
    let (_, reg) = app
        .register_user("frank@example.com", "strongpass123")
        .await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let (status, body) = app.post_with_auth("/auth/logout", access).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, serde_json::Value::Null);
}

#[tokio::test]
async fn logout_without_auth_401() {
    let app = TestApp::new().await;

    let (status, _) = app.post_empty("/auth/logout").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_revokes_all_refresh_tokens() {
    let app = TestApp::new().await;
    let (_, reg) = app
        .register_user("grace@example.com", "strongpass123")
        .await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Logout
    let (status, _) = app.post_with_auth("/auth/logout", access).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Refresh with the old token should fail
    let body = serde_json::json!({ "refresh_token": refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_without_bearer_prefix_401() {
    let app = TestApp::new().await;
    let (_, reg) = app
        .register_user("prefix@example.com", "strongpass123")
        .await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    // Send token WITHOUT "Bearer " prefix
    let (status, body) = app.post_with_raw_auth("/auth/logout", access).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("invalid authorization header format"),
        "expected message about invalid format, got: {}",
        body["error"]["message"]
    );
}

#[tokio::test]
async fn multiple_sessions_logout_revokes_all() {
    let app = TestApp::new().await;

    // Register creates session 1
    let (_, reg) = app
        .register_user("multi@example.com", "strongpass123")
        .await;
    let refresh_1 = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Login creates session 2
    let login_body = serde_json::json!({
        "email": "multi@example.com",
        "password": "strongpass123",
    });
    let (status, login_resp) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::OK);
    let access_2 = login_resp["tokens"]["access_token"].as_str().unwrap();
    let refresh_2 = login_resp["tokens"]["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Logout using session 2 access token (revokes ALL refresh tokens for user)
    let (status, _) = app.post_with_auth("/auth/logout", access_2).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Both refresh tokens should now fail
    let body1 = serde_json::json!({ "refresh_token": refresh_1 });
    let (status, _) = app.post_json("/auth/refresh", &body1).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let body2 = serde_json::json!({ "refresh_token": refresh_2 });
    let (status, _) = app.post_json("/auth/refresh", &body2).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
