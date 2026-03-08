mod common;

use axum::http::StatusCode;
use common::{NEW_PASSWORD, TEST_PASSWORD, TestApp, assert_error};

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
    let (status, body) = app
        .patch_json_with_auth("/users/me", &patch_body, "invalid-token")
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
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

// ── Export data tests ─────────────────────────────────────────────────

#[tokio::test]
async fn export_data_empty_user_200() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let (status, body) = app.get_with_auth("/users/me/export", &token).await;

    assert_eq!(status, StatusCode::OK);
    // Profile present with correct email
    assert_eq!(body["profile"]["email"], "user@example.com");
    assert!(body["profile"]["id"].is_string());
    // No password_hash in export
    assert!(body["profile"].get("password_hash").is_none());
    // Empty arrays for new user
    assert!(body["items"].is_array());
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert!(body["categories"].is_array());
    // Default category "Autre" may exist
    assert!(body["exported_at"].is_string());
}

#[tokio::test]
async fn export_data_includes_items_and_categories() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // Create a category
    let cat = app
        .create_category(
            &token,
            &serde_json::json!({ "name": "Électronique", "icon": "laptop" }),
        )
        .await;
    let cat_id = cat["id"].as_str().unwrap();

    // Create two items (one with category, one without)
    app.create_item(
        &token,
        &serde_json::json!({
            "name": "iPhone",
            "estimated_price": 999.0,
            "priority": 3,
            "category_id": cat_id,
        }),
    )
    .await;
    app.create_item(&token, &serde_json::json!({ "name": "Livre" }))
        .await;

    let (status, body) = app.get_with_auth("/users/me/export", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["profile"]["email"], "user@example.com");

    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2, "should export both items");

    let item_names: Vec<&str> = items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert!(item_names.contains(&"iPhone"));
    assert!(item_names.contains(&"Livre"));

    let categories = body["categories"].as_array().unwrap();
    // At least the custom one + default "Autre"
    let cat_names: Vec<&str> = categories
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(cat_names.contains(&"Électronique"));

    assert!(body["exported_at"].is_string());
}

#[tokio::test]
async fn export_data_without_auth_401() {
    let app = TestApp::new().await;

    let (status, body) = app.get_no_auth("/users/me/export").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn export_data_does_not_leak_other_users() {
    let app = TestApp::new().await;
    let token_a = app
        .setup_user_token("alice@example.com", TEST_PASSWORD)
        .await;
    let token_b = app.setup_user_token("bob@example.com", TEST_PASSWORD).await;

    // Alice creates an item
    app.create_item(
        &token_a,
        &serde_json::json!({ "name": "Alice's secret item" }),
    )
    .await;

    // Bob exports — should NOT see Alice's item
    let (status, body) = app.get_with_auth("/users/me/export", &token_b).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["profile"]["email"], "bob@example.com");

    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 0, "Bob should not see Alice's items");
}

#[tokio::test]
async fn export_after_delete_item_reflects_current_state() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // Create and then delete an item
    let item = app
        .create_item(&token, &serde_json::json!({ "name": "Ephemeral" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/items/{item_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Export should not include the deleted item
    let (status, body) = app.get_with_auth("/users/me/export", &token).await;
    assert_eq!(status, StatusCode::OK);

    let items = body["items"].as_array().unwrap();
    let ids: Vec<&str> = items.iter().map(|i| i["id"].as_str().unwrap()).collect();
    assert!(
        !ids.contains(&item_id),
        "deleted item should not appear in export"
    );
}

// ── Delete account edge cases ─────────────────────────────────────────

#[tokio::test]
async fn delete_account_cascades_items_and_categories() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // Create data
    app.create_category(&token, &serde_json::json!({ "name": "Cat1" }))
        .await;
    app.create_item(
        &token,
        &serde_json::json!({ "name": "Item1", "priority": 2 }),
    )
    .await;

    // Delete account
    let (status, _) = app.delete_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Re-register same email — should work (user is gone)
    let new_token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // New user's export should have no items from previous account
    let (status, body) = app.get_with_auth("/users/me/export", &new_token).await;
    assert_eq!(status, StatusCode::OK);

    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 0, "new account should have no items");
}

#[tokio::test]
async fn delete_account_invalidates_token() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let (status, _) = app.delete_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Token still has valid JWT signature but user is gone → 404
    let (status, body) = app.get_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&body, "NOT_FOUND");
}

// ── Profile edge cases ────────────────────────────────────────────────

#[tokio::test]
async fn get_profile_excludes_password_hash() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let (status, body) = app.get_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::OK);

    // Ensure password_hash is never in the response
    assert!(body.get("password_hash").is_none());
    assert!(body.get("password").is_none());
}

#[tokio::test]
async fn update_display_name_null_is_ignored() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // Set a display name
    let patch = serde_json::json!({ "display_name": "Alice" });
    let (status, body) = app.patch_json_with_auth("/users/me", &patch, &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Alice");

    // Sending null = "don't update" (Option<String> semantics)
    let patch = serde_json::json!({ "display_name": null });
    let (status, body) = app.patch_json_with_auth("/users/me", &patch, &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["display_name"], "Alice",
        "null should not clear the field"
    );
}

#[tokio::test]
async fn update_profile_persists_across_requests() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    let patch = serde_json::json!({
        "display_name": "Persisted",
        "locale": "en",
        "reminder_freq": "daily",
    });
    let (status, _) = app.patch_json_with_auth("/users/me", &patch, &token).await;
    assert_eq!(status, StatusCode::OK);

    // Verify via GET
    let (status, body) = app.get_with_auth("/users/me", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Persisted");
    assert_eq!(body["locale"], "en");
    assert_eq!(body["reminder_freq"], "daily");
}

#[tokio::test]
async fn profile_survives_password_change() {
    let app = TestApp::new().await;
    let token = app
        .setup_user_token("user@example.com", TEST_PASSWORD)
        .await;

    // Set profile data
    let patch = serde_json::json!({ "display_name": "Survivor" });
    let (status, _) = app.patch_json_with_auth("/users/me", &patch, &token).await;
    assert_eq!(status, StatusCode::OK);

    // Change password
    let change = serde_json::json!({
        "current_password": TEST_PASSWORD,
        "new_password": NEW_PASSWORD,
    });
    let (status, _) = app
        .post_json_with_auth("/auth/change-password", &change, &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Re-login
    let login = serde_json::json!({
        "email": "user@example.com",
        "password": NEW_PASSWORD,
    });
    let (status, login_body) = app.post_json("/auth/login", &login).await;
    assert_eq!(status, StatusCode::OK);
    let new_token = login_body["tokens"]["access_token"].as_str().unwrap();

    // Profile data should still be there
    let (status, body) = app.get_with_auth("/users/me", new_token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Survivor");
}
