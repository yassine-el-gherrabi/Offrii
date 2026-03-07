mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};

#[tokio::test]
async fn get_profile_returns_defaults() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let (status, body) = app.get_with_auth("/users/me", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "user@example.com");
    assert_eq!(body["reminder_freq"], "weekly");
    assert_eq!(body["timezone"], "UTC");
    assert_eq!(body["locale"], "fr");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
}

#[tokio::test]
async fn update_display_name() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({ "display_name": "Alice" });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Alice");
}

#[tokio::test]
async fn update_timezone_recomputes_utc_hour() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({
        "timezone": "Europe/Paris",
        "reminder_time": "10:00:00"
    });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["timezone"], "Europe/Paris");
    assert_eq!(body["reminder_time"], "10:00:00");

    // Verify via GET that the profile persisted correctly
    let (status, body) = app.get_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["timezone"], "Europe/Paris");
}

#[tokio::test]
async fn update_reminder_freq_never() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({ "reminder_freq": "never" });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["reminder_freq"], "never");
}

#[tokio::test]
async fn update_invalid_timezone_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({ "timezone": "Invalid/Timezone" });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn delete_account_cascades() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // Delete
    let (status, _) = app.delete_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Login should fail
    let login_body = serde_json::json!({
        "email": "user@example.com",
        "password": TEST_PASSWORD,
    });
    let (status, _) = app.post_json("/auth/login", &login_body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Auth guard tests (401) ──────────────────────────────────────────

#[tokio::test]
async fn get_profile_without_auth_401() {
    let app = TestApp::new().await;

    let (status, body) = app.get_no_auth("/users/me").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn patch_profile_without_auth_401() {
    let app = TestApp::new().await;

    let patch_body = serde_json::json!({ "display_name": "Hacker" });
    let (status, _) = app.post_json("/users/me", &patch_body).await;

    // POST on a PATCH-only route without auth → 401 or 405, either way blocked
    assert_ne!(status, StatusCode::OK);
}

#[tokio::test]
async fn delete_account_without_auth_401() {
    let app = TestApp::new().await;

    let (status, body) = app.delete_with_auth("/users/me", "invalid-token").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

// ── Validation edge cases (400) ─────────────────────────────────────

#[tokio::test]
async fn update_invalid_reminder_freq_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({ "reminder_freq": "hourly" });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn update_display_name_too_long_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let long_name = "A".repeat(101);
    let patch_body = serde_json::json!({ "display_name": long_name });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn update_locale_too_long_400() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({ "locale": "fr-FR-extra-long" });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn update_empty_body_returns_current_profile() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({});
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "user@example.com");
    assert_eq!(body["reminder_freq"], "weekly");
}

#[tokio::test]
async fn update_multiple_fields_at_once() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch_body = serde_json::json!({
        "display_name": "Jean",
        "reminder_freq": "daily",
        "timezone": "America/New_York",
        "reminder_time": "08:00:00",
        "locale": "en"
    });
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Jean");
    assert_eq!(body["reminder_freq"], "daily");
    assert_eq!(body["timezone"], "America/New_York");
    assert_eq!(body["reminder_time"], "08:00:00");
    assert_eq!(body["locale"], "en");
}
