mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp};
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────

async fn setup_user(app: &TestApp, email: &str, username: &str) -> (String, Uuid) {
    let (status, body) = app
        .register_user_with_username(email, TEST_PASSWORD, username)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    let user_id = Uuid::parse_str(body["user"]["id"].as_str().unwrap()).unwrap();
    (token, user_id)
}

/// Create a circle and return its ID.
async fn create_circle(app: &TestApp, token: &str, name: &str) -> Uuid {
    let body = json!({ "name": name });
    let (status, resp) = app.post_json_with_auth("/circles", &body, token).await;
    assert_eq!(status, StatusCode::CREATED);
    Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap()
}

/// Add a member to a circle (requires friendship first).
async fn add_member(app: &TestApp, token: &str, circle_id: Uuid, user_id: Uuid) {
    let body = json!({ "user_id": user_id });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/members"), &body, token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

/// Make two users friends.
async fn make_friends(app: &TestApp, token_a: &str, token_b: &str, username_b: &str) {
    let body = json!({ "username": username_b });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, token_a)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let req_id = resp["id"].as_str().unwrap();
    let (status, _) = app
        .post_with_auth(&format!("/me/friend-requests/{req_id}/accept"), token_b)
        .await;
    assert_eq!(status, StatusCode::OK);
}

// ── List notifications ────────────────────────────────────────────────

#[tokio::test]
async fn list_notifications_empty_returns_200() {
    let app = TestApp::new().await;
    let (token, _) = setup_user(&app, "notif1@example.com", "notifuser1").await;

    let (status, body) = app.get_with_auth("/me/notifications", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["pagination"]["total"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn list_notifications_no_auth_returns_401() {
    let app = TestApp::new().await;
    let (status, _) = app.get_no_auth("/me/notifications").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Unread count ──────────────────────────────────────────────────────

#[tokio::test]
async fn unread_count_empty_returns_zero() {
    let app = TestApp::new().await;
    let (token, _) = setup_user(&app, "notif2@example.com", "notifuser2").await;

    let (status, body) = app
        .get_with_auth("/me/notifications/unread-count", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["count"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn unread_count_no_auth_returns_401() {
    let app = TestApp::new().await;
    let (status, _) = app.get_no_auth("/me/notifications/unread-count").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Mark all read ─────────────────────────────────────────────────────

#[tokio::test]
async fn mark_all_read_empty_returns_204() {
    let app = TestApp::new().await;
    let (token, _) = setup_user(&app, "notif3@example.com", "notifuser3").await;

    let (status, _) = app.post_with_auth("/me/notifications/read", &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn mark_all_read_no_auth_returns_401() {
    let app = TestApp::new().await;
    let (status, _) = app.post_empty("/me/notifications/read").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Mark single read ──────────────────────────────────────────────────

#[tokio::test]
async fn mark_read_nonexistent_returns_204() {
    // mark_read is fire-and-forget — even non-existent IDs return 204
    let app = TestApp::new().await;
    let (token, _) = setup_user(&app, "notif4@example.com", "notifuser4").await;
    let fake_id = Uuid::new_v4();

    let (status, _) = app
        .post_with_auth(&format!("/me/notifications/{fake_id}/read"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn mark_read_no_auth_returns_401() {
    let app = TestApp::new().await;
    let fake_id = Uuid::new_v4();
    let (status, _) = app
        .post_empty(&format!("/me/notifications/{fake_id}/read"))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── E2E: circle action creates notification ───────────────────────────

#[tokio::test]
async fn adding_member_to_circle_creates_notification() {
    let app = TestApp::new().await;
    let (alice_token, _alice_id) = setup_user(&app, "alice_n@test.com", "alice_notif").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_n@test.com", "bob_notif").await;

    // Make friends
    make_friends(&app, &alice_token, &bob_token, "bob_notif").await;

    // Alice creates a circle and adds Bob
    let circle_id = create_circle(&app, &alice_token, "Test Notif Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    // Notifications are sent asynchronously — give the spawned task a moment
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Bob should have at least 1 notification
    let (status, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    assert_eq!(status, StatusCode::OK);
    let notifs = body["data"].as_array().unwrap();
    assert!(
        !notifs.is_empty(),
        "Bob should have at least one notification after being added to circle"
    );

    // Verify unread count
    let (status, body) = app
        .get_with_auth("/me/notifications/unread-count", &bob_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["count"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn mark_all_read_clears_unread_count() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_m@test.com", "alice_mark").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_m@test.com", "bob_mark").await;

    make_friends(&app, &alice_token, &bob_token, "bob_mark").await;
    let circle_id = create_circle(&app, &alice_token, "Mark Read Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify unread > 0
    let (_, body) = app
        .get_with_auth("/me/notifications/unread-count", &bob_token)
        .await;
    assert!(
        body["count"].as_i64().unwrap() >= 1,
        "precondition: unread > 0"
    );

    // Mark all read
    let (status, _) = app
        .post_with_auth("/me/notifications/read", &bob_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Unread count should be 0
    let (_, body) = app
        .get_with_auth("/me/notifications/unread-count", &bob_token)
        .await;
    assert_eq!(body["count"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn notifications_are_user_scoped() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_s@test.com", "alice_scope").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_s@test.com", "bob_scope").await;
    let (carol_token, _) = setup_user(&app, "carol_s@test.com", "carol_scope").await;

    make_friends(&app, &alice_token, &bob_token, "bob_scope").await;
    let circle_id = create_circle(&app, &alice_token, "Scoped Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Bob has notifications
    let (_, body) = app
        .get_with_auth("/me/notifications/unread-count", &bob_token)
        .await;
    assert!(body["count"].as_i64().unwrap() >= 1);

    // Carol has zero (not in the circle)
    let (_, body) = app
        .get_with_auth("/me/notifications/unread-count", &carol_token)
        .await;
    assert_eq!(body["count"].as_i64().unwrap(), 0);
}

// ── Pagination ────────────────────────────────────────────────────────

#[tokio::test]
async fn list_notifications_respects_pagination() {
    let app = TestApp::new().await;
    let (token, _) = setup_user(&app, "pag@test.com", "paguser").await;

    // With no notifications, page=1 limit=5 should return empty
    let (status, body) = app
        .get_with_auth("/me/notifications?page=1&limit=5", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["pagination"]["page"].as_i64().unwrap(), 1);
    assert_eq!(body["pagination"]["limit"].as_i64().unwrap(), 5);
}

// ── Delete notification ───────────────────────────────────────────────

#[tokio::test]
async fn delete_notification_returns_204() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_del@test.com", "alice_del").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_del@test.com", "bob_del").await;

    make_friends(&app, &alice_token, &bob_token, "bob_del").await;
    let circle_id = create_circle(&app, &alice_token, "Delete Notif Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Bob has notifications
    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    let notifs = body["data"].as_array().unwrap();
    assert!(!notifs.is_empty());

    let notif_id = notifs[0]["id"].as_str().unwrap();

    // Delete it
    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{notif_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone from the list
    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    let remaining = body["data"].as_array().unwrap();
    assert!(
        !remaining
            .iter()
            .any(|n| n["id"].as_str().unwrap() == notif_id),
        "deleted notification should not appear in list"
    );
}

#[tokio::test]
async fn delete_notification_nonexistent_returns_404() {
    let app = TestApp::new().await;
    let (token, _) = setup_user(&app, "del404@test.com", "del404user").await;
    let fake_id = Uuid::new_v4();

    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{fake_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_notification_no_auth_returns_401() {
    let app = TestApp::new().await;
    let fake_id = Uuid::new_v4();

    // Use an invalid token to simulate unauthenticated DELETE
    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{fake_id}"), "invalid_token")
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_notification_cannot_delete_other_users_notification() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_iso@test.com", "alice_iso").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_iso@test.com", "bob_iso").await;
    let (carol_token, _) = setup_user(&app, "carol_iso@test.com", "carol_iso").await;

    make_friends(&app, &alice_token, &bob_token, "bob_iso").await;
    let circle_id = create_circle(&app, &alice_token, "Iso Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Bob has a notification
    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    let notif_id = body["data"][0]["id"].as_str().unwrap();

    // Carol tries to delete Bob's notification — should 404 (not found for her)
    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{notif_id}"), &carol_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Bob's notification still exists
    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    assert!(
        body["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["id"].as_str().unwrap() == notif_id),
        "notification should still exist for Bob"
    );
}

#[tokio::test]
async fn delete_notification_decreases_unread_count() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_cnt@test.com", "alice_cnt").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_cnt@test.com", "bob_cnt").await;

    make_friends(&app, &alice_token, &bob_token, "bob_cnt").await;
    let circle_id = create_circle(&app, &alice_token, "Count Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Get initial unread count
    let (_, body) = app
        .get_with_auth("/me/notifications/unread-count", &bob_token)
        .await;
    let initial_count = body["count"].as_i64().unwrap();
    assert!(initial_count >= 1, "precondition: unread > 0");

    // Get and delete one notification
    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    let notif_id = body["data"][0]["id"].as_str().unwrap();
    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{notif_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Unread count should decrease
    let (_, body) = app
        .get_with_auth("/me/notifications/unread-count", &bob_token)
        .await;
    assert_eq!(body["count"].as_i64().unwrap(), initial_count - 1);
}

#[tokio::test]
async fn delete_already_deleted_notification_returns_404() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_dd@test.com", "alice_dd").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_dd@test.com", "bob_dd").await;

    make_friends(&app, &alice_token, &bob_token, "bob_dd").await;
    let circle_id = create_circle(&app, &alice_token, "Double Del Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    let notif_id = body["data"][0]["id"].as_str().unwrap();

    // Delete once — 204
    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{notif_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Delete again — 404
    let (status, _) = app
        .delete_with_auth(&format!("/me/notifications/{notif_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Notification fields ───────────────────────────────────────────────

#[tokio::test]
async fn notification_has_expected_fields() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user(&app, "alice_f@test.com", "alice_fields").await;
    let (bob_token, bob_id) = setup_user(&app, "bob_f@test.com", "bob_fields").await;

    make_friends(&app, &alice_token, &bob_token, "bob_fields").await;
    let circle_id = create_circle(&app, &alice_token, "Fields Circle").await;
    add_member(&app, &alice_token, circle_id, bob_id).await;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let (_, body) = app.get_with_auth("/me/notifications", &bob_token).await;
    let notifs = body["data"].as_array().unwrap();
    assert!(!notifs.is_empty());

    let notif = &notifs[0];
    assert!(notif["id"].is_string(), "notification should have id");
    assert!(notif["type"].is_string(), "notification should have type");
    assert!(notif["title"].is_string(), "notification should have title");
    assert!(notif["body"].is_string(), "notification should have body");
    assert!(notif["read"].is_boolean(), "notification should have read");
    assert!(
        notif["created_at"].is_string(),
        "notification should have created_at"
    );
    assert_eq!(
        notif["read"].as_bool().unwrap(),
        false,
        "new notification should be unread"
    );
    // circle_id should be present for the "added to circle" notification
    assert!(
        notif["circle_id"].is_string(),
        "circle_added notification should have circle_id, got: {}",
        notif
    );
}
