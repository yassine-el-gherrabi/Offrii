mod common;

use axum::http::StatusCode;

use common::{NEW_PASSWORD, TEST_PASSWORD, TestApp, assert_error};

/// Default test email. Each test gets its own container so no collision risk.
const TEST_EMAIL: &str = "test@example.com";

// ---------------------------------------------------------------------------
// Register
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_success_201() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user(TEST_EMAIL, TEST_PASSWORD).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["tokens"]["access_token"].is_string());
    assert!(body["tokens"]["refresh_token"].is_string());
    assert_eq!(body["tokens"]["token_type"], "Bearer");
    assert!(body["tokens"]["expires_in"].is_u64());
    assert_eq!(body["user"]["email"], TEST_EMAIL);
    assert!(body["user"]["id"].is_string());
    assert!(body["user"]["username"].is_string());
    assert!(body["user"]["created_at"].is_string());

    // Verify auto-generated username format
    let username = body["user"]["username"].as_str().unwrap();
    assert!(username.len() >= 3, "username too short: {username}");
    assert!(
        username.chars().next().unwrap().is_ascii_lowercase(),
        "username should start with a letter: {username}"
    );

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

    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let (status, body) = app.register_user(TEST_EMAIL, TEST_PASSWORD).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn register_bad_email_400() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user("not-an-email", TEST_PASSWORD).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_common_password_400() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user(TEST_EMAIL, "password").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
    assert_eq!(body["error"]["message"], "password_common");
}

#[tokio::test]
async fn register_short_password_400() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user(TEST_EMAIL, "short").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_with_display_name_201() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user_with_name(TEST_EMAIL, TEST_PASSWORD, "Alice")
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["user"]["display_name"], "Alice");
    assert!(body["tokens"]["access_token"].is_string());

    // Username should be derived from display_name
    let username = body["user"]["username"].as_str().unwrap();
    assert!(
        username.starts_with("alice"),
        "username should start with slugified display_name, got: {username}"
    );
}

#[tokio::test]
async fn register_without_display_name_generates_username_from_email() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user("bob@example.com", TEST_PASSWORD).await;

    assert_eq!(status, StatusCode::CREATED);
    let username = body["user"]["username"].as_str().unwrap();
    assert!(
        username.starts_with("bob"),
        "username should be derived from email prefix, got: {username}"
    );
}

// ---------------------------------------------------------------------------
// Register with explicit username
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_with_username_preserves_it() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user_with_username(TEST_EMAIL, TEST_PASSWORD, "alice_e2e")
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        body["user"]["username"], "alice_e2e",
        "username should be preserved exactly as provided"
    );
}

#[tokio::test]
async fn register_with_username_and_display_name() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "display_name": "Alice",
        "username": "custom_user"
    });
    let (status, resp) = app.post_json("/auth/register", &body).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["user"]["username"], "custom_user");
    assert_eq!(resp["user"]["display_name"], "Alice");
}

#[tokio::test]
async fn register_with_invalid_username_format_400() {
    let app = TestApp::new().await;

    // Starts with digit
    let (status, body) = app
        .register_user_with_username(TEST_EMAIL, TEST_PASSWORD, "1invalid")
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_with_uppercase_username_400() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user_with_username(TEST_EMAIL, TEST_PASSWORD, "Alice")
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_with_too_short_username_400() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user_with_username(TEST_EMAIL, TEST_PASSWORD, "ab")
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_with_special_chars_username_400() {
    let app = TestApp::new().await;

    let (status, body) = app
        .register_user_with_username(TEST_EMAIL, TEST_PASSWORD, "alice@home")
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_with_taken_username_409() {
    let app = TestApp::new().await;

    // First user takes the username
    let (status, _) = app
        .register_user_with_username("first@example.com", TEST_PASSWORD, "taken_name")
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Second user tries the same username
    let (status, body) = app
        .register_user_with_username("second@example.com", TEST_PASSWORD, "taken_name")
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

#[tokio::test]
async fn register_without_username_auto_generates() {
    let app = TestApp::new().await;

    // No username field → auto-generate (backward compat)
    let (status, body) = app.register_user(TEST_EMAIL, TEST_PASSWORD).await;

    assert_eq!(status, StatusCode::CREATED);
    let username = body["user"]["username"].as_str().unwrap();
    assert!(username.len() >= 3);
    assert!(
        username.chars().next().unwrap().is_ascii_lowercase(),
        "auto-generated username should start with a letter: {username}"
    );
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
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD
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
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/login", &body).await;

    assert_eq!(status, StatusCode::OK);
    assert!(resp["tokens"]["access_token"].is_string());
    assert!(resp["tokens"]["refresh_token"].is_string());
    assert_eq!(resp["tokens"]["token_type"], "Bearer");
    assert!(resp["tokens"]["expires_in"].is_u64());
    assert_eq!(resp["user"]["email"], TEST_EMAIL);
    assert!(resp["user"]["id"].is_string());
    assert!(resp["user"]["username"].is_string());
    assert!(resp["user"]["created_at"].is_string());
}

#[tokio::test]
async fn login_wrong_password_401() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({
        "email": TEST_EMAIL,
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
        "password": TEST_PASSWORD,
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
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
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
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
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
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
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
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
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
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
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
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
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
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Login creates session 2 (also get refresh_1 from a fresh login)
    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });

    let (status, s1) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::OK);
    let refresh_1 = s1["tokens"]["refresh_token"].as_str().unwrap().to_string();

    let (status, s2) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::OK);
    let access_2 = s2["tokens"]["access_token"].as_str().unwrap();
    let refresh_2 = s2["tokens"]["refresh_token"].as_str().unwrap().to_string();

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

#[tokio::test]
async fn refresh_double_concurrent_use_401() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // First refresh succeeds
    let body = serde_json::json!({ "refresh_token": &refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::OK);

    // Second refresh with same token fails (already revoked by first)
    let (status, resp) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&resp, "UNAUTHORIZED");
}

#[tokio::test]
async fn logout_then_refresh_each_session_401() {
    let app = TestApp::new().await;

    // Create 3 sessions: register + 2 logins
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });

    let (_, s1) = app.post_json("/auth/login", &login_body).await;
    let (_, s2) = app.post_json("/auth/login", &login_body).await;

    let access = s2["tokens"]["access_token"].as_str().unwrap();
    let refresh_1 = s1["tokens"]["refresh_token"].as_str().unwrap().to_string();
    let refresh_2 = s2["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Logout (revokes all sessions)
    let (status, _) = app.post_with_auth("/auth/logout", access).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Each session's refresh token should fail
    let body1 = serde_json::json!({ "refresh_token": refresh_1 });
    let (status, _) = app.post_json("/auth/refresh", &body1).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let body2 = serde_json::json!({ "refresh_token": refresh_2 });
    let (status, _) = app.post_json("/auth/refresh", &body2).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Expired token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn refresh_expired_token_401() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Manually expire the token in DB
    sqlx::query("UPDATE refresh_tokens SET expires_at = NOW() - INTERVAL '1 hour'")
        .execute(&app.db)
        .await
        .unwrap();

    let body = serde_json::json!({ "refresh_token": refresh });
    let (status, body) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

// ---------------------------------------------------------------------------
// Concurrent refresh
// ---------------------------------------------------------------------------

#[tokio::test]
async fn refresh_truly_concurrent_one_wins_one_loses() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    let body = serde_json::json!({ "refresh_token": &refresh });

    // Fire two concurrent refreshes with the same token
    let (resp1, resp2) = tokio::join!(
        app.post_json("/auth/refresh", &body),
        app.post_json("/auth/refresh", &body),
    );

    let statuses = [resp1.0, resp2.0];
    // Exactly one should succeed, the other should fail
    assert!(
        statuses.contains(&StatusCode::OK),
        "expected one 200, got {statuses:?}"
    );
    assert!(
        statuses.contains(&StatusCode::UNAUTHORIZED),
        "expected one 401, got {statuses:?}"
    );
}

// ---------------------------------------------------------------------------
// Max refresh tokens
// ---------------------------------------------------------------------------

#[tokio::test]
async fn login_enforces_max_refresh_tokens() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });

    // Login 7 more times (register created 1, so 8 total)
    for _ in 0..7 {
        let (status, _) = app.post_json("/auth/login", &login_body).await;
        assert_eq!(status, StatusCode::OK);
    }

    // Should only have at most 5 active refresh tokens
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM refresh_tokens \
         WHERE user_id = (SELECT id FROM users WHERE email = $1) \
         AND revoked_at IS NULL",
    )
    .bind(TEST_EMAIL)
    .fetch_one(&app.db)
    .await
    .unwrap();

    assert!(
        count.0 <= 5,
        "expected at most 5 active refresh tokens, got {}",
        count.0
    );
}

// ---------------------------------------------------------------------------
// Email normalization
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_normalizes_email() {
    let app = TestApp::new().await;

    let (status, _) = app.register_user("Alice@Example.COM", TEST_PASSWORD).await;
    assert_eq!(status, StatusCode::CREATED);

    // Login with normalized email should work
    let body = serde_json::json!({
        "email": "alice@example.com",
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &body).await;
    assert_eq!(status, StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Input validation edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_password_too_long_400() {
    let app = TestApp::new().await;
    let long_password = "a".repeat(129);

    let (status, body) = app.register_user(TEST_EMAIL, &long_password).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn register_display_name_too_long_400() {
    let app = TestApp::new().await;
    let long_name = "a".repeat(101);

    let (status, body) = app
        .register_user_with_name(TEST_EMAIL, TEST_PASSWORD, &long_name)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

// ---------------------------------------------------------------------------
// Bearer case-insensitive
// ---------------------------------------------------------------------------

#[tokio::test]
async fn logout_with_lowercase_bearer_204() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    // Send with lowercase "bearer"
    let (status, _) = app
        .post_with_raw_auth("/auth/logout", &format!("bearer {access}"))
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// JTI Blacklist (access token revocation on logout)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn logout_blacklists_access_token_401() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    // Logout — should blacklist the JTI
    let (status, _) = app.post_with_auth("/auth/logout", access).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Using the same access token should now fail with "token has been revoked"
    let (status, body) = app.get_with_auth("/categories", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
    assert_eq!(
        body["error"]["message"].as_str().unwrap(),
        "token has been revoked",
    );
}

#[tokio::test]
async fn logout_only_blacklists_own_token() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });

    // Create two sessions
    let (_, s1) = app.post_json("/auth/login", &login_body).await;
    let (_, s2) = app.post_json("/auth/login", &login_body).await;

    let access_a = s1["tokens"]["access_token"].as_str().unwrap();
    let access_b = s2["tokens"]["access_token"].as_str().unwrap().to_string();

    // Logout session A
    let (status, _) = app.post_with_auth("/auth/logout", access_a).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Session A access token should be rejected
    let (status, _) = app.get_with_auth("/categories", access_a).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Session B access token should still work
    let (status, _) = app.get_with_auth("/categories", &access_b).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn double_logout_second_attempt_401() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    // First logout succeeds
    let (status, _) = app.post_with_auth("/auth/logout", access).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Second logout with same token should fail — JTI is blacklisted
    let (status, body) = app.post_with_auth("/auth/logout", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
    assert_eq!(
        body["error"]["message"].as_str().unwrap(),
        "token has been revoked",
    );
}

#[tokio::test]
async fn logout_blacklists_token_across_endpoints() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let (status, _) = app.post_with_auth("/auth/logout", access).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Blacklisted token should be rejected on every protected endpoint
    let (status, _) = app.get_with_auth("/categories", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = app.get_with_auth("/items", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = app.get_with_auth("/users/me", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// Token versioning (mass revocation)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn token_version_bump_rejects_refresh_with_stale_version() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Bump token_version in DB (simulates invalidate_all_tokens)
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE email = $1")
        .bind(TEST_EMAIL)
        .execute(&app.db)
        .await
        .unwrap();

    // Refresh with old token_version (1) should fail — DB is now at 2
    let body = serde_json::json!({ "refresh_token": refresh });
    let (status, resp) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&resp, "UNAUTHORIZED");
    assert_eq!(
        resp["error"]["message"].as_str().unwrap(),
        "token version revoked",
    );
}

#[tokio::test]
async fn token_version_bump_rejects_access_via_redis() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();
    let user_id = reg["user"]["id"].as_str().unwrap();

    // Simulate invalidate_all_tokens: bump DB + set Redis tkver key
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE email = $1")
        .bind(TEST_EMAIL)
        .execute(&app.db)
        .await
        .unwrap();

    let mut conn = app.redis.get_multiplexed_async_connection().await.unwrap();
    let key = format!("tkver:{user_id}");
    redis::cmd("SET")
        .arg(&key)
        .arg(2)
        .arg("EX")
        .arg(900)
        .query_async::<()>(&mut conn)
        .await
        .unwrap();

    // Access token was issued with version=1, Redis says version=2 → rejected
    let (status, body) = app.get_with_auth("/categories", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
    assert_eq!(
        body["error"]["message"].as_str().unwrap(),
        "token version revoked",
    );
}

#[tokio::test]
async fn new_login_after_version_bump_succeeds() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Bump token_version
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE email = $1")
        .bind(TEST_EMAIL)
        .execute(&app.db)
        .await
        .unwrap();

    // New login should succeed and issue tokens with new version
    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::OK);

    // New access token should work
    let new_access = resp["tokens"]["access_token"].as_str().unwrap();
    let (status, _) = app.get_with_auth("/categories", new_access).await;
    assert_eq!(status, StatusCode::OK);

    // New refresh token should work
    let new_refresh = resp["tokens"]["refresh_token"].as_str().unwrap();
    let body = serde_json::json!({ "refresh_token": new_refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn version_bump_does_not_affect_other_users() {
    let app = TestApp::new().await;

    // User A
    let reg_a = app.setup_user("alice@example.com", TEST_PASSWORD).await;

    // User B
    let reg_b = app.setup_user("bob@example.com", TEST_PASSWORD).await;
    let access_b = reg_b["tokens"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Bump only user A's token_version
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE email = $1")
        .bind("alice@example.com")
        .execute(&app.db)
        .await
        .unwrap();

    // User B should be unaffected
    let (status, _) = app.get_with_auth("/categories", &access_b).await;
    assert_eq!(status, StatusCode::OK);

    // User A's refresh should fail (stale version)
    let refresh_a = reg_a["tokens"]["refresh_token"].as_str().unwrap();
    let body = serde_json::json!({ "refresh_token": refresh_a });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // User B's refresh should still work
    let refresh_b = reg_b["tokens"]["refresh_token"].as_str().unwrap();
    let body = serde_json::json!({ "refresh_token": refresh_b });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Change password
// ---------------------------------------------------------------------------

#[tokio::test]
async fn change_password_success_204() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app
        .post_json_with_auth("/auth/change-password", &body, access)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn change_password_wrong_current_401() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let body = serde_json::json!({
        "current_password": "wrongpassword123",
        "new_password": NEW_PASSWORD,
    });
    let (status, resp) = app
        .post_json_with_auth("/auth/change-password", &body, access)
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&resp, "UNAUTHORIZED");
}

#[tokio::test]
async fn change_password_without_auth_401() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/change-password", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn change_password_short_new_password_400() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": "short",
    });
    let (status, resp) = app
        .post_json_with_auth("/auth/change-password", &body, access)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn change_password_too_long_new_password_400() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": "a".repeat(129),
    });
    let (status, resp) = app
        .post_json_with_auth("/auth/change-password", &body, access)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn change_password_invalidates_tokens_and_allows_new_login() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Change password
    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app
        .post_json_with_auth("/auth/change-password", &body, access)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Old refresh token should be revoked
    let body = serde_json::json!({ "refresh_token": refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Old password should no longer work
    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // New password should work
    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": NEW_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::OK);
    assert!(resp["tokens"]["access_token"].is_string());
}

// ---------------------------------------------------------------------------
// Forgot password
// ---------------------------------------------------------------------------

#[tokio::test]
async fn forgot_password_existing_email_200() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "email": TEST_EMAIL });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;

    assert_eq!(status, StatusCode::OK);

    // Verify a 6-digit code was generated and "emailed"
    let code = app.get_last_reset_code().await;
    assert!(code.is_some(), "expected a reset code to be sent");
    assert_eq!(code.unwrap().len(), 6);
}

#[tokio::test]
async fn forgot_password_nonexistent_email_200() {
    let app = TestApp::new().await;

    // No user registered — should still return 200 (no email enumeration)
    let body = serde_json::json!({ "email": "nobody@example.com" });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn forgot_password_invalid_email_400() {
    let app = TestApp::new().await;

    let body = serde_json::json!({ "email": "not-an-email" });
    let (status, resp) = app.post_json("/auth/forgot-password", &body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn forgot_password_rate_limit_no_new_code() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // First call — should succeed and send a code
    let body = serde_json::json!({ "email": TEST_EMAIL });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert_eq!(status, StatusCode::OK);

    let code1 = app.get_last_reset_code().await;
    assert!(code1.is_some());

    // Clear the spy
    *app.last_reset_code.lock().unwrap() = None;

    // Second call within 60s — should return 200 but NOT send a new code
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert_eq!(status, StatusCode::OK);

    // Wait briefly in case a background task fires
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let code2 = app.last_reset_code.lock().unwrap().clone();
    assert!(code2.is_none(), "expected no new code due to rate limiting");
}

#[tokio::test]
async fn forgot_password_normalizes_email() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Send with uppercase email — should still match the registered user
    let body = serde_json::json!({ "email": "TEST@EXAMPLE.COM" });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert_eq!(status, StatusCode::OK);

    let code = app.get_last_reset_code().await;
    assert!(
        code.is_some(),
        "should find user even with different case email"
    );
}

// ---------------------------------------------------------------------------
// Reset password
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reset_password_valid_code_204() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Trigger forgot password to get a code
    let body = serde_json::json!({ "email": TEST_EMAIL });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert_eq!(status, StatusCode::OK);

    let code = app.get_last_reset_code().await.unwrap();

    // Reset with the valid code
    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": code,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/reset-password", &body).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn reset_password_invalid_code_400() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Trigger forgot password
    let body = serde_json::json!({ "email": TEST_EMAIL });
    app.post_json("/auth/forgot-password", &body).await;

    let real_code = app.get_last_reset_code().await.unwrap();
    let wrong_code = if real_code == "000000" {
        "111111"
    } else {
        "000000"
    };

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": wrong_code,
        "new_password": NEW_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/reset-password", &body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reset_password_no_code_exists_400() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // No forgot-password called — no code in Redis
    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": "123456",
        "new_password": NEW_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/reset-password", &body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reset_password_short_password_400() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": "123456",
        "new_password": "short",
    });
    let (status, resp) = app.post_json("/auth/reset-password", &body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reset_password_allows_login_with_new_password() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Trigger forgot password + get code
    let body = serde_json::json!({ "email": TEST_EMAIL });
    app.post_json("/auth/forgot-password", &body).await;
    let code = app.get_last_reset_code().await.unwrap();

    // Reset password
    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": code,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/reset-password", &body).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Old password should fail
    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // New password should work
    let login_body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": NEW_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::OK);
    assert!(resp["tokens"]["access_token"].is_string());
}

#[tokio::test]
async fn reset_password_invalidates_old_tokens() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();
    let refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Trigger forgot password + reset
    let body = serde_json::json!({ "email": TEST_EMAIL });
    app.post_json("/auth/forgot-password", &body).await;
    let code = app.get_last_reset_code().await.unwrap();

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": code,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/reset-password", &body).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Old refresh token should be revoked
    let body = serde_json::json!({ "refresh_token": refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Old access token should be rejected (token version bumped)
    let (status, _) = app.get_with_auth("/categories", access).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn reset_password_code_consumed_once() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Trigger forgot password
    let body = serde_json::json!({ "email": TEST_EMAIL });
    app.post_json("/auth/forgot-password", &body).await;
    let code = app.get_last_reset_code().await.unwrap();

    // First reset succeeds
    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": &code,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/reset-password", &body).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Second reset with same code fails (code was consumed / Redis key deleted)
    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "code": &code,
        "new_password": "An0therStr0ng!Pass#2026",
    });
    let (status, resp) = app.post_json("/auth/reset-password", &body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn reset_password_wrong_email_400() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    app.setup_user("other@example.com", TEST_PASSWORD).await;

    // Trigger forgot password for TEST_EMAIL
    let body = serde_json::json!({ "email": TEST_EMAIL });
    app.post_json("/auth/forgot-password", &body).await;
    let code = app.get_last_reset_code().await.unwrap();

    // Try resetting with the code but a different email — Redis key mismatch
    let body = serde_json::json!({
        "email": "other@example.com",
        "code": code,
        "new_password": NEW_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/reset-password", &body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}
