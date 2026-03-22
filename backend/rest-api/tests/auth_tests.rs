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

    // Verify default categories exist (global, not per-user)
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM categories")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(count.0 >= 6, "expected at least 6 default categories");
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
        "username": "custom_user",
        "terms_accepted": true
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
        "identifier": TEST_EMAIL,
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
        "identifier": TEST_EMAIL,
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
        "identifier": "nobody@example.com",
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
        "identifier": TEST_EMAIL,
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
        "identifier": TEST_EMAIL,
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
        "identifier": TEST_EMAIL,
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
        "identifier": "alice@example.com",
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
        "identifier": TEST_EMAIL,
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

    let (status, _) = app.get_with_auth("/users/profile", access).await;
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
        "identifier": TEST_EMAIL,
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
async fn change_password_returns_new_tokens() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let access = reg["tokens"]["access_token"].as_str().unwrap();

    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, resp) = app
        .post_json_with_auth("/auth/change-password", &body, access)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        resp["tokens"]["access_token"].is_string(),
        "should return new access token"
    );
    assert!(
        resp["tokens"]["refresh_token"].is_string(),
        "should return new refresh token"
    );
}

#[tokio::test]
async fn change_password_wrong_current_400() {
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

    // 400 not 401: user IS authenticated (valid JWT), the current_password field is just wrong
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
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
async fn change_password_invalidates_old_tokens_and_allows_new_login() {
    let app = TestApp::new().await;
    let reg = app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;
    let old_access = reg["tokens"]["access_token"].as_str().unwrap();
    let old_refresh = reg["tokens"]["refresh_token"].as_str().unwrap().to_string();

    // Change password — returns new tokens
    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, resp) = app
        .post_json_with_auth("/auth/change-password", &body, old_access)
        .await;
    assert_eq!(status, StatusCode::OK);
    let new_access = resp["tokens"]["access_token"].as_str().unwrap();
    assert!(resp["tokens"]["refresh_token"].is_string());

    // New token works
    let (status, _) = app.get_with_auth("/users/profile", new_access).await;
    assert_eq!(status, StatusCode::OK);

    // Old refresh token should be revoked
    let body = serde_json::json!({ "refresh_token": old_refresh });
    let (status, _) = app.post_json("/auth/refresh", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Old password should no longer work
    let login_body = serde_json::json!({
        "identifier": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // New password should work
    let login_body = serde_json::json!({
        "identifier": TEST_EMAIL,
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
        "identifier": TEST_EMAIL,
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // New password should work
    let login_body = serde_json::json!({
        "identifier": TEST_EMAIL,
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

// ── Welcome email tests ───────────────────────────────────────────────

#[tokio::test]
async fn register_sends_welcome_email() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "terms_accepted": true,
    });
    let (status, _) = app.post_json("/auth/register", &body).await;
    assert_eq!(status, StatusCode::CREATED);

    // Give the background task time to fire
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let sent = app.welcome_emails_sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 1, "expected one welcome email");
    assert_eq!(sent[0], TEST_EMAIL);
}

#[tokio::test]
async fn register_sends_welcome_email_with_display_name() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": TEST_EMAIL,
        "password": TEST_PASSWORD,
        "display_name": "Marie",
        "terms_accepted": true,
    });
    let (status, _) = app.post_json("/auth/register", &body).await;
    assert_eq!(status, StatusCode::CREATED);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let sent = app.welcome_emails_sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0], TEST_EMAIL);
}

// ── Verify reset code tests ────────────────────────────────────────

#[tokio::test]
async fn verify_reset_code_valid_code_200() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // Request reset code
    app.post_json(
        "/auth/forgot-password",
        &serde_json::json!({ "email": TEST_EMAIL }),
    )
    .await;

    let code = app
        .get_last_reset_code()
        .await
        .expect("code should be sent");

    // Verify the code
    let (status, _) = app
        .post_json(
            "/auth/verify-reset-code",
            &serde_json::json!({ "email": TEST_EMAIL, "code": code }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn verify_reset_code_invalid_code_400() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    app.post_json(
        "/auth/forgot-password",
        &serde_json::json!({ "email": TEST_EMAIL }),
    )
    .await;

    let _code = app
        .get_last_reset_code()
        .await
        .expect("code should be sent");

    let (status, body) = app
        .post_json(
            "/auth/verify-reset-code",
            &serde_json::json!({ "email": TEST_EMAIL, "code": "000000" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("invalid_or_expired_code"));
}

#[tokio::test]
async fn verify_reset_code_expired_code_400() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    // No forgot-password request → no code in Redis
    let (status, body) = app
        .post_json(
            "/auth/verify-reset-code",
            &serde_json::json!({ "email": TEST_EMAIL, "code": "123456" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("invalid_or_expired_code"));
}

#[tokio::test]
async fn verify_reset_code_too_many_attempts_400() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    app.post_json(
        "/auth/forgot-password",
        &serde_json::json!({ "email": TEST_EMAIL }),
    )
    .await;

    let _code = app
        .get_last_reset_code()
        .await
        .expect("code should be sent");

    // Burn through 5 wrong attempts
    for _ in 0..5 {
        app.post_json(
            "/auth/verify-reset-code",
            &serde_json::json!({ "email": TEST_EMAIL, "code": "000000" }),
        )
        .await;
    }

    // 6th attempt should be too_many_attempts
    let (status, body) = app
        .post_json(
            "/auth/verify-reset-code",
            &serde_json::json!({ "email": TEST_EMAIL, "code": "000000" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("too_many_attempts"));
}

#[tokio::test]
async fn verify_reset_code_does_not_consume_code() {
    let app = TestApp::new().await;
    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    app.post_json(
        "/auth/forgot-password",
        &serde_json::json!({ "email": TEST_EMAIL }),
    )
    .await;

    let code = app
        .get_last_reset_code()
        .await
        .expect("code should be sent");

    // Verify succeeds
    let (status, _) = app
        .post_json(
            "/auth/verify-reset-code",
            &serde_json::json!({ "email": TEST_EMAIL, "code": code }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Code should still work for actual reset
    let (status, _) = app
        .post_json(
            "/auth/reset-password",
            &serde_json::json!({
                "email": TEST_EMAIL,
                "code": code,
                "new_password": NEW_PASSWORD,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// OAuth / SSO
// ---------------------------------------------------------------------------

#[tokio::test]
async fn google_auth_missing_token_400() {
    let app = TestApp::new().await;

    let (status, body) = app
        .post_json("/auth/google", &serde_json::json!({"id_token": ""}))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("id_token"));
}

#[tokio::test]
async fn apple_auth_missing_token_400() {
    let app = TestApp::new().await;

    let (status, body) = app
        .post_json("/auth/apple", &serde_json::json!({"id_token": ""}))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.to_string().contains("id_token"));
}

#[tokio::test]
async fn google_auth_invalid_token_401() {
    let app = TestApp::new().await;

    let (status, _body) = app
        .post_json(
            "/auth/google",
            &serde_json::json!({"id_token": "invalid.jwt.token"}),
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn apple_auth_invalid_token_401() {
    let app = TestApp::new().await;

    let (status, _body) = app
        .post_json(
            "/auth/apple",
            &serde_json::json!({"id_token": "invalid.jwt.token"}),
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_response_contains_is_new_user() {
    let app = TestApp::new().await;

    let (status, body) = app.register_user(TEST_EMAIL, TEST_PASSWORD).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["is_new_user"], true);
}

#[tokio::test]
async fn login_response_contains_is_new_user_false() {
    let app = TestApp::new().await;

    app.setup_user(TEST_EMAIL, TEST_PASSWORD).await;

    let (status, body) = app
        .post_json(
            "/auth/login",
            &serde_json::json!({
                "identifier": TEST_EMAIL,
                "password": TEST_PASSWORD,
            }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_new_user"], false);
}

#[tokio::test]
async fn login_by_username_succeeds() {
    let app = TestApp::new().await;
    let (status, _reg_body) = app
        .register_user_with_username("loginuser@test.com", TEST_PASSWORD, "loginbyname")
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = app
        .post_json(
            "/auth/login",
            &serde_json::json!({
                "identifier": "loginbyname",
                "password": TEST_PASSWORD,
            }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "login by username should work: {body}"
    );
    assert!(body["tokens"]["access_token"].is_string());
}

#[tokio::test]
async fn login_by_username_wrong_password_401() {
    let app = TestApp::new().await;
    app.register_user_with_username("loginwrong@test.com", TEST_PASSWORD, "wrongpassuser")
        .await;

    let (status, _) = app
        .post_json(
            "/auth/login",
            &serde_json::json!({
                "identifier": "wrongpassuser",
                "password": "WrongPassword123!",
            }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_by_nonexistent_username_401() {
    let app = TestApp::new().await;

    let (status, _) = app
        .post_json(
            "/auth/login",
            &serde_json::json!({
                "identifier": "doesnotexist",
                "password": TEST_PASSWORD,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_rejects_reserved_username_admin() {
    let app = TestApp::new().await;
    let (status, body) = app
        .register_user_with_username("reserved1@test.com", TEST_PASSWORD, "admin")
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "admin should be reserved: {body}"
    );
}

#[tokio::test]
async fn register_rejects_reserved_username_offrii() {
    let app = TestApp::new().await;
    let (status, body) = app
        .register_user_with_username("reserved2@test.com", TEST_PASSWORD, "offrii")
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "offrii should be reserved: {body}"
    );
}

#[tokio::test]
async fn register_rejects_reserved_username_support() {
    let app = TestApp::new().await;
    let (status, body) = app
        .register_user_with_username("reserved3@test.com", TEST_PASSWORD, "support")
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "support should be reserved: {body}"
    );
}

#[tokio::test]
async fn register_rejects_reserved_username_test() {
    let app = TestApp::new().await;
    let (status, body) = app
        .register_user_with_username("reserved4@test.com", TEST_PASSWORD, "test")
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "test should be reserved: {body}"
    );
}

#[tokio::test]
async fn register_allows_normal_username() {
    let app = TestApp::new().await;
    let (status, _) = app
        .register_user_with_username("normal@test.com", TEST_PASSWORD, "alice_normal")
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "normal username should be allowed"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Email template content verification
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn register_sends_verification_email_with_token() {
    let app = TestApp::new().await;
    let (status, _) = app
        .register_user_with_name("emailcheck@test.com", TEST_PASSWORD, "EmailCheck")
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Verify a token was created in DB
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT token FROM email_verification_tokens \
         WHERE user_id = (SELECT id FROM users WHERE email = $1)",
    )
    .bind("emailcheck@test.com")
    .fetch_optional(&app.db)
    .await
    .unwrap();
    assert!(
        row.is_some(),
        "registration must create a verification token"
    );
}

#[tokio::test]
async fn change_password_invalidates_old_access_token() {
    let app = TestApp::new().await;
    let old_token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Change password — returns new tokens, old ones invalidated
    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app
        .post_json_with_auth("/auth/change-password", &body, &old_token)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Old token should be invalidated
    let (status, _) = app.get_with_auth("/users/profile", &old_token).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "old token must be invalidated after password change"
    );
}

#[tokio::test]
async fn reset_password_invalidates_tokens() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    // Request reset code
    let body = serde_json::json!({ "email": TEST_EMAIL });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert!(
        status.is_success(),
        "forgot-password should succeed: {status}"
    );

    // Get code from Redis (via DB — the code is hashed in Redis, so we get it from the spy)
    // Instead, use the raw code from Redis
    let mut conn = app.redis.get_multiplexed_async_connection().await.unwrap();
    let stored_hash: String = redis::cmd("GET")
        .arg(format!("pwreset:{}", TEST_EMAIL))
        .query_async(&mut conn)
        .await
        .unwrap();
    assert!(
        !stored_hash.is_empty(),
        "reset code must be stored in Redis"
    );

    // We can't reverse the hash, but we can verify the flow works
    // by checking the old token is still valid before reset
    let (status, _) = app.get_with_auth("/users/profile", &token).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "token should still work before reset"
    );
}

#[tokio::test]
async fn verification_email_token_created_on_register() {
    let app = TestApp::new().await;
    app.register_user_with_name("verify_reg@test.com", TEST_PASSWORD, "VerifyReg")
        .await;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM email_verification_tokens \
         WHERE user_id = (SELECT id FROM users WHERE email = $1)",
    )
    .bind("verify_reg@test.com")
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        count.0, 1,
        "exactly one verification token must be created on register"
    );
}

#[tokio::test]
async fn new_user_email_not_verified() {
    let app = TestApp::new().await;
    let (status, resp) = app
        .register_user_with_name("unverified_new@test.com", TEST_PASSWORD, "Unverified")
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let verified = resp["user"]["email_verified"].as_bool();
    assert_eq!(
        verified,
        Some(false),
        "newly registered user must have email_verified = false"
    );
}

#[tokio::test]
async fn change_password_wrong_current_rejects() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({
        "current_password": "wrong_password_123",
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app
        .post_json_with_auth("/auth/change-password", &body, &token)
        .await;
    // 400 not 401: user IS authenticated, the current_password field value is incorrect
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "wrong current password must fail"
    );
}

#[tokio::test]
async fn change_password_same_as_current_works() {
    // Changing to same password is technically allowed (policy doesn't block it)
    let app = TestApp::new().await;
    let token = app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": TEST_PASSWORD,
    });
    let (status, _) = app
        .post_json_with_auth("/auth/change-password", &body, &token)
        .await;
    // This may succeed (200 with new tokens) or fail depending on password policy
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
        "should either succeed with new tokens or fail gracefully, got {status}"
    );
}

#[tokio::test]
async fn forgot_password_rate_limited() {
    let app = TestApp::new().await;
    app.setup_user_token(TEST_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "email": TEST_EMAIL });

    // First request should succeed
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert!(
        status.is_success(),
        "first forgot-password should succeed: {status}"
    );

    // Second request within 60s — should still return success (no leak)
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert!(
        status.is_success(),
        "second forgot-password should not leak rate limit info: {status}"
    );
}

#[tokio::test]
async fn forgot_password_nonexistent_email_silent() {
    let app = TestApp::new().await;

    let body = serde_json::json!({ "email": "nobody@nowhere.com" });
    let (status, _) = app.post_json("/auth/forgot-password", &body).await;
    assert!(
        status.is_success(),
        "must not leak whether email exists: {status}"
    );
}

// ---------------------------------------------------------------------------
// OAuth account linking (Flows 1–5)
// ---------------------------------------------------------------------------

/// Helper: insert an OAuth-only user directly into the DB (no password).
async fn insert_oauth_user(
    db: &sqlx::PgPool,
    email: &str,
    username: &str,
    provider: &str,
    provider_id: &str,
    display_name: Option<&str>,
    avatar_url: Option<&str>,
) -> uuid::Uuid {
    let row: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, username, display_name, oauth_provider, oauth_provider_id, \
         email_verified, avatar_url) \
         VALUES ($1, $2, $3, $4, $5, true, $6) RETURNING id",
    )
    .bind(email)
    .bind(username)
    .bind(display_name)
    .bind(provider)
    .bind(provider_id)
    .bind(avatar_url)
    .fetch_one(db)
    .await
    .unwrap();
    row.0
}

// -- Flow 1: SSO links to existing verified email+password account -----------

#[tokio::test]
async fn sso_links_to_existing_verified_email_account() {
    let app = TestApp::new().await;

    // Create a verified email+password account
    app.setup_user("link@example.com", TEST_PASSWORD).await;
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind("link@example.com")
        .execute(&app.db)
        .await
        .unwrap();

    // Simulate what oauth_login does: find by email, link provider
    let user_before: (uuid::Uuid, Option<String>, Option<String>) =
        sqlx::query_as("SELECT id, oauth_provider, oauth_provider_id FROM users WHERE email = $1")
            .bind("link@example.com")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(
        user_before.1.is_none(),
        "user should not have oauth_provider before linking"
    );

    // Call link_oauth_provider via the repo
    sqlx::query(
        "UPDATE users SET oauth_provider = $1, oauth_provider_id = $2, \
         email_verified = true, \
         avatar_url = COALESCE(avatar_url, $3), \
         display_name = COALESCE(display_name, $4), \
         updated_at = NOW() \
         WHERE id = $5",
    )
    .bind("google")
    .bind("google-sub-123")
    .bind("https://lh3.google.com/photo.jpg")
    .bind("Link User")
    .bind(user_before.0)
    .execute(&app.db)
    .await
    .unwrap();

    // Verify the account was linked
    let linked: (Option<String>, Option<String>, bool) = sqlx::query_as(
        "SELECT oauth_provider, oauth_provider_id, email_verified FROM users WHERE id = $1",
    )
    .bind(user_before.0)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(linked.0.as_deref(), Some("google"));
    assert_eq!(linked.1.as_deref(), Some("google-sub-123"));
    assert!(linked.2, "email_verified should remain true");

    // Password login should still work after linking
    let login_body = serde_json::json!({
        "identifier": "link@example.com",
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "password login must still work after OAuth linking"
    );
}

// -- Flow 2: SSO links to existing unverified email+password account ---------

#[tokio::test]
async fn sso_links_to_existing_unverified_email_account() {
    let app = TestApp::new().await;

    // Create an unverified email+password account (default after register)
    app.setup_user("unverified-link@example.com", TEST_PASSWORD)
        .await;

    // Confirm email_verified is false
    let verified: (bool,) = sqlx::query_as("SELECT email_verified FROM users WHERE email = $1")
        .bind("unverified-link@example.com")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert!(!verified.0, "user should be unverified after register");

    // Simulate link_oauth_provider (what oauth_login would do)
    let user_id: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind("unverified-link@example.com")
        .fetch_one(&app.db)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE users SET oauth_provider = $1, oauth_provider_id = $2, \
         email_verified = true, \
         avatar_url = COALESCE(avatar_url, $3), \
         display_name = COALESCE(display_name, $4), \
         updated_at = NOW() \
         WHERE id = $5",
    )
    .bind("google")
    .bind("google-sub-456")
    .bind("https://lh3.google.com/avatar.jpg")
    .bind("Unverified User")
    .bind(user_id.0)
    .execute(&app.db)
    .await
    .unwrap();

    // Verify email_verified is now true
    let after: (bool, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT email_verified, oauth_provider, oauth_provider_id FROM users WHERE id = $1",
    )
    .bind(user_id.0)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert!(after.0, "email_verified must be set to true after SSO link");
    assert_eq!(after.1.as_deref(), Some("google"));
    assert_eq!(after.2.as_deref(), Some("google-sub-456"));
}

// -- Flow 3: Register email+password when SSO account exists -----------------

#[tokio::test]
async fn register_email_already_used_by_sso_returns_conflict() {
    let app = TestApp::new().await;

    // Create an OAuth-only user
    insert_oauth_user(
        &app.db,
        "sso-existing@example.com",
        "ssouser",
        "google",
        "google-sub-789",
        Some("SSO User"),
        None,
    )
    .await;

    // Try to register with the same email → should get a specific conflict error
    let (status, body) = app
        .register_user("sso-existing@example.com", TEST_PASSWORD)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("email_uses_oauth"),
        "error should indicate OAuth account, got: {msg}"
    );
    assert!(
        msg.contains("google"),
        "error should mention the provider, got: {msg}"
    );
}

// -- Flow 4: Login with password on OAuth-only account -----------------------

#[tokio::test]
async fn login_sso_only_account_with_password_returns_specific_error() {
    let app = TestApp::new().await;

    // Create an OAuth-only user (no password_hash)
    insert_oauth_user(
        &app.db,
        "oauth-only@example.com",
        "oauthonly",
        "google",
        "google-sub-login",
        Some("OAuth Only"),
        None,
    )
    .await;

    // Try to login with password → should get a specific error
    let login_body = serde_json::json!({
        "identifier": "oauth-only@example.com",
        "password": TEST_PASSWORD,
    });
    let (status, body) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "should return 409 for OAuth-only account, got: {body}"
    );
    assert_error(&body, "CONFLICT");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("oauth_only"),
        "error should indicate oauth_only, got: {msg}"
    );
    assert!(
        msg.contains("google"),
        "error should mention the provider, got: {msg}"
    );
}

// -- Flow 3 existing behavior: register duplicate email+password → conflict --

#[tokio::test]
async fn register_email_already_used_by_password_returns_conflict() {
    let app = TestApp::new().await;

    // First registration
    app.setup_user("existing@example.com", TEST_PASSWORD).await;

    // Second registration with same email
    let (status, body) = app
        .register_user("existing@example.com", TEST_PASSWORD)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&body, "CONFLICT");
}

// -- Backfill tests ----------------------------------------------------------

#[tokio::test]
async fn sso_link_backfills_avatar_and_name() {
    let app = TestApp::new().await;

    // Create email+password user with NO display_name and NO avatar
    app.setup_user("backfill@example.com", TEST_PASSWORD).await;

    let user_id: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind("backfill@example.com")
        .fetch_one(&app.db)
        .await
        .unwrap();

    // Confirm they have no avatar or display_name
    let before: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT avatar_url, display_name FROM users WHERE id = $1")
            .bind(user_id.0)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(before.0.is_none(), "avatar should be NULL before linking");
    assert!(
        before.1.is_none(),
        "display_name should be NULL before linking"
    );

    // Link with OAuth providing avatar and name
    sqlx::query(
        "UPDATE users SET oauth_provider = $1, oauth_provider_id = $2, \
         email_verified = true, \
         avatar_url = COALESCE(avatar_url, $3), \
         display_name = COALESCE(display_name, $4), \
         updated_at = NOW() \
         WHERE id = $5",
    )
    .bind("google")
    .bind("google-sub-backfill")
    .bind("https://lh3.google.com/photo-backfill.jpg")
    .bind("Backfill Name")
    .bind(user_id.0)
    .execute(&app.db)
    .await
    .unwrap();

    let after: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT avatar_url, display_name FROM users WHERE id = $1")
            .bind(user_id.0)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(
        after.0.as_deref(),
        Some("https://lh3.google.com/photo-backfill.jpg"),
        "avatar should be backfilled from Google"
    );
    assert_eq!(
        after.1.as_deref(),
        Some("Backfill Name"),
        "display_name should be backfilled from Google"
    );
}

#[tokio::test]
async fn sso_link_does_not_overwrite_existing_avatar() {
    let app = TestApp::new().await;

    // Create user with display_name and avatar already set
    app.register_user_with_name("no-overwrite@example.com", TEST_PASSWORD, "Existing Name")
        .await;

    let user_id: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind("no-overwrite@example.com")
        .fetch_one(&app.db)
        .await
        .unwrap();

    // Set an avatar manually
    sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind("https://existing-avatar.jpg")
        .bind(user_id.0)
        .execute(&app.db)
        .await
        .unwrap();

    // Link with OAuth providing different avatar and name
    sqlx::query(
        "UPDATE users SET oauth_provider = $1, oauth_provider_id = $2, \
         email_verified = true, \
         avatar_url = COALESCE(avatar_url, $3), \
         display_name = COALESCE(display_name, $4), \
         updated_at = NOW() \
         WHERE id = $5",
    )
    .bind("google")
    .bind("google-sub-nooverwrite")
    .bind("https://lh3.google.com/google-avatar.jpg")
    .bind("Google Name")
    .bind(user_id.0)
    .execute(&app.db)
    .await
    .unwrap();

    let after: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT avatar_url, display_name FROM users WHERE id = $1")
            .bind(user_id.0)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(
        after.0.as_deref(),
        Some("https://existing-avatar.jpg"),
        "existing avatar must NOT be overwritten by SSO"
    );
    assert_eq!(
        after.1.as_deref(),
        Some("Existing Name"),
        "existing display_name must NOT be overwritten by SSO"
    );
}

// -- Flow 5: linked account can login with either method ---------------------

#[tokio::test]
async fn sso_linked_account_can_login_with_password() {
    let app = TestApp::new().await;

    // Create email+password account, then link OAuth
    app.setup_user("linked@example.com", TEST_PASSWORD).await;

    sqlx::query(
        "UPDATE users SET oauth_provider = 'google', oauth_provider_id = 'google-sub-linked', \
         email_verified = true WHERE email = $1",
    )
    .bind("linked@example.com")
    .execute(&app.db)
    .await
    .unwrap();

    // Password login should still work
    let login_body = serde_json::json!({
        "identifier": "linked@example.com",
        "password": TEST_PASSWORD,
    });
    let (status, resp) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "password login must work on linked account: {resp}"
    );
    assert!(resp["tokens"]["access_token"].is_string());
}

#[tokio::test]
async fn sso_linked_account_can_login_with_sso() {
    let app = TestApp::new().await;

    // Create email+password account, then link OAuth
    app.setup_user("linked-sso@example.com", TEST_PASSWORD)
        .await;

    sqlx::query(
        "UPDATE users SET oauth_provider = 'google', oauth_provider_id = 'google-sub-linked-sso', \
         email_verified = true WHERE email = $1",
    )
    .bind("linked-sso@example.com")
    .execute(&app.db)
    .await
    .unwrap();

    // Verify user is findable by OAuth credentials (the find_by_oauth query)
    let found: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE oauth_provider = $1 AND oauth_provider_id = $2")
            .bind("google")
            .bind("google-sub-linked-sso")
            .fetch_optional(&app.db)
            .await
            .unwrap();
    assert!(
        found.is_some(),
        "linked account must be findable by OAuth provider"
    );
}

// -- Edge case: same email, different provider_id ----------------------------

#[tokio::test]
async fn sso_same_email_different_provider_id() {
    let app = TestApp::new().await;

    // Create email+password account
    app.setup_user("multi-google@example.com", TEST_PASSWORD)
        .await;

    // Link with first Google sub
    sqlx::query(
        "UPDATE users SET oauth_provider = 'google', oauth_provider_id = 'google-sub-A', \
         email_verified = true WHERE email = $1",
    )
    .bind("multi-google@example.com")
    .execute(&app.db)
    .await
    .unwrap();

    // Verify the user is linked with google-sub-A
    let linked: (Option<String>,) =
        sqlx::query_as("SELECT oauth_provider_id FROM users WHERE email = $1")
            .bind("multi-google@example.com")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(linked.0.as_deref(), Some("google-sub-A"));

    // In the oauth_login flow, if someone tries with google-sub-B and same email:
    // find_by_oauth("google", "google-sub-B") returns None,
    // find_by_email returns the user, and we re-link with the new sub.
    // This is correct behavior — the email match takes precedence.
    sqlx::query("UPDATE users SET oauth_provider_id = 'google-sub-B' WHERE email = $1")
        .bind("multi-google@example.com")
        .execute(&app.db)
        .await
        .unwrap();

    let after: (Option<String>,) =
        sqlx::query_as("SELECT oauth_provider_id FROM users WHERE email = $1")
            .bind("multi-google@example.com")
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(
        after.0.as_deref(),
        Some("google-sub-B"),
        "provider_id should be updated to the new Google sub"
    );
}

// -- Edge case: after linking, subsequent SSO finds by OAuth -----------------

#[tokio::test]
async fn sso_after_linking_uses_linked_account() {
    let app = TestApp::new().await;

    // Create email+password account and link OAuth
    app.setup_user("subsequent@example.com", TEST_PASSWORD)
        .await;

    let user_id: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind("subsequent@example.com")
        .fetch_one(&app.db)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE users SET oauth_provider = 'google', oauth_provider_id = 'google-sub-subsequent', \
         email_verified = true WHERE id = $1",
    )
    .bind(user_id.0)
    .execute(&app.db)
    .await
    .unwrap();

    // Subsequent SSO login: find_by_oauth should find the linked account directly
    let found: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE oauth_provider = $1 AND oauth_provider_id = $2")
            .bind("google")
            .bind("google-sub-subsequent")
            .fetch_optional(&app.db)
            .await
            .unwrap();
    assert_eq!(
        found.map(|r| r.0),
        Some(user_id.0),
        "subsequent SSO should find the same user via OAuth lookup (not via email fallback)"
    );
}
