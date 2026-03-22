mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────────────────

async fn request_email_change(
    app: &common::TestApp,
    token: &str,
    new_email: &str,
) -> (StatusCode, serde_json::Value) {
    let body = serde_json::json!({ "new_email": new_email });
    app.post_json_with_auth("/users/email", &body, token).await
}

/// Extract the email change token from the database for a given user.
async fn get_email_change_token(app: &common::TestApp, user_email: &str) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT ect.token FROM email_change_tokens ect \
         JOIN users u ON u.id = ect.user_id \
         WHERE u.email = $1 \
         ORDER BY ect.created_at DESC LIMIT 1",
    )
    .bind(user_email)
    .fetch_optional(&app.db)
    .await
    .unwrap()
}

/// GET the verify-email-change page and return (status, html_body).
async fn get_verify_email_change(app: &common::TestApp, token: &str) -> (StatusCode, String) {
    let uri = format!("/auth/verify-email-change?token={token}");
    let req = Request::builder()
        .method("GET")
        .uri(&uri)
        .body(Body::empty())
        .unwrap();

    let resp = app.router.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

// ── Tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn request_email_change_success_204() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    let (status, _) = request_email_change(&app, token, "new@test.com").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Token should exist in DB
    let db_token = get_email_change_token(&app, "user@test.com").await;
    assert!(db_token.is_some(), "email change token should be stored");
}

#[tokio::test]
async fn request_email_change_invalid_email_400() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    let (status, _) = request_email_change(&app, token, "not-an-email").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn request_email_change_same_email_400() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    let (status, _) = request_email_change(&app, token, "user@test.com").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn request_email_change_duplicate_email_409() {
    let app = common::TestApp::new().await;
    let reg = app
        .setup_user("user1@test.com", common::TEST_PASSWORD)
        .await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // Register second user with the target email
    app.setup_user("taken@test.com", common::TEST_PASSWORD)
        .await;

    let (status, _) = request_email_change(&app, token, "taken@test.com").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn request_email_change_unauthenticated_401() {
    let app = common::TestApp::new().await;

    let (status, _) = request_email_change(&app, "invalid-token", "new@test.com").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn request_email_change_rate_limited_429() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // First request should succeed
    let (status1, _) = request_email_change(&app, token, "new1@test.com").await;
    assert_eq!(status1, StatusCode::NO_CONTENT);

    // Second request within 5 minutes should be rate limited
    let (status2, _) = request_email_change(&app, token, "new2@test.com").await;
    assert_eq!(status2, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn confirm_email_change_success() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("old@test.com", common::TEST_PASSWORD).await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // Request email change
    let (status, _) = request_email_change(&app, token, "new@test.com").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Extract token from DB
    let change_token = get_email_change_token(&app, "old@test.com")
        .await
        .expect("token should exist");

    // Verify via GET
    let (verify_status, html) = get_verify_email_change(&app, &change_token).await;
    assert_eq!(verify_status, StatusCode::OK);
    assert!(html.contains("Email modifié"), "should show success page");

    // Verify email was actually updated in DB
    let row = sqlx::query_as::<_, (String, bool)>(
        "SELECT email, email_verified FROM users WHERE email = $1",
    )
    .bind("new@test.com")
    .fetch_optional(&app.db)
    .await
    .unwrap();

    assert!(row.is_some(), "user should have new email");
    let (email, verified) = row.unwrap();
    assert_eq!(email, "new@test.com");
    assert!(verified, "new email should be marked verified");

    // Old email should no longer exist
    let old = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("old@test.com")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(old, 0, "old email should no longer exist");
}

#[tokio::test]
async fn confirm_email_change_invalid_token() {
    let app = common::TestApp::new().await;

    let (status, html) = get_verify_email_change(&app, "totally-invalid-token").await;
    assert_eq!(status, StatusCode::OK); // HTML page, not API error
    assert!(
        html.contains("invalide ou expiré"),
        "should show error page"
    );
}

#[tokio::test]
async fn confirm_email_change_expired_token() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // Request email change
    let (status, _) = request_email_change(&app, token, "new@test.com").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let change_token = get_email_change_token(&app, "user@test.com")
        .await
        .expect("token should exist");

    // Manually expire the token
    sqlx::query(
        "UPDATE email_change_tokens SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1",
    )
    .bind(&change_token)
    .execute(&app.db)
    .await
    .unwrap();

    // Try to verify — should fail
    let (verify_status, html) = get_verify_email_change(&app, &change_token).await;
    assert_eq!(verify_status, StatusCode::OK);
    assert!(
        html.contains("invalide ou expiré"),
        "should show error for expired token"
    );

    // Email should NOT have changed
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("user@test.com")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count, 1, "original email should still exist");
}

#[tokio::test]
async fn confirm_email_change_invalidates_sessions() {
    let app = common::TestApp::new().await;
    let reg = app.setup_user("user@test.com", common::TEST_PASSWORD).await;
    let old_token = reg["tokens"]["access_token"].as_str().unwrap().to_string();

    // Request email change
    let (status, _) = request_email_change(&app, &old_token, "new@test.com").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let change_token = get_email_change_token(&app, "user@test.com")
        .await
        .expect("token should exist");

    // Confirm the change
    let (verify_status, _) = get_verify_email_change(&app, &change_token).await;
    assert_eq!(verify_status, StatusCode::OK);

    // Old access token should now be rejected (token_version bumped)
    let (profile_status, _) = app.get_with_auth("/users/profile", &old_token).await;
    assert_eq!(
        profile_status,
        StatusCode::UNAUTHORIZED,
        "old token should be invalidated after email change"
    );
}

#[tokio::test]
async fn confirm_email_change_race_condition_duplicate() {
    let app = common::TestApp::new().await;
    let reg = app
        .setup_user("user1@test.com", common::TEST_PASSWORD)
        .await;
    let token = reg["tokens"]["access_token"].as_str().unwrap();

    // User1 requests change to target@test.com
    let (status, _) = request_email_change(&app, token, "target@test.com").await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let change_token = get_email_change_token(&app, "user1@test.com")
        .await
        .expect("token should exist");

    // Meanwhile, user2 registers with target@test.com
    app.setup_user("target@test.com", common::TEST_PASSWORD)
        .await;

    // User1 tries to confirm — should fail due to unique constraint
    let (verify_status, html) = get_verify_email_change(&app, &change_token).await;
    assert_eq!(verify_status, StatusCode::OK);
    assert!(
        html.contains("invalide ou expiré"),
        "should show error when email was taken in the meantime"
    );

    // User1's email should be unchanged
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind("user1@test.com")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count, 1, "user1 email should remain unchanged");
}
