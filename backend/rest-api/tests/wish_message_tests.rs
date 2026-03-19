mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────

async fn age_account(app: &TestApp, email: &str) {
    sqlx::query("UPDATE users SET created_at = NOW() - INTERVAL '48 hours' WHERE email = $1")
        .bind(email)
        .execute(&app.db)
        .await
        .unwrap();
}

async fn setup_aged_user_with_name(app: &TestApp, email: &str, name: &str) -> String {
    let (status, body) = app
        .register_user_with_name(email, TEST_PASSWORD, name)
        .await;
    assert_eq!(status, StatusCode::CREATED, "register {email}: {body}");
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    age_account(app, email).await;
    token
}

async fn setup_aged_user(app: &TestApp, email: &str) -> String {
    let token = app.setup_user_token(email, TEST_PASSWORD).await;
    age_account(app, email).await;
    token
}

async fn get_user_id(app: &TestApp, email: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db)
        .await
        .unwrap();
    row.0
}

async fn wait_for_wish_status(app: &TestApp, wish_id: Uuid, expected: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let row: (String,) = sqlx::query_as("SELECT status FROM community_wishes WHERE id = $1")
            .bind(wish_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
        if row.0 == expected {
            return;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "wish {wish_id} did not reach status '{expected}' within 5s (current: '{}')",
                row.0
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

async fn create_open_wish(app: &TestApp, token: &str) -> Uuid {
    let body = json!({
        "title": "Need winter coat",
        "category": "clothing",
        "is_anonymous": true,
    });
    let (status, resp) = app
        .post_json_with_auth("/community/wishes", &body, token)
        .await;
    assert_eq!(status, StatusCode::CREATED, "create wish: {resp}");
    let wish_id = Uuid::parse_str(resp["id"].as_str().unwrap()).unwrap();
    wait_for_wish_status(app, wish_id, "open").await;
    wish_id
}

async fn force_match(app: &TestApp, wish_id: Uuid, donor_id: Uuid) {
    sqlx::query(
        "UPDATE community_wishes SET status = 'matched', matched_with = $1, matched_at = NOW() WHERE id = $2",
    )
    .bind(donor_id)
    .bind(wish_id)
    .execute(&app.db)
    .await
    .unwrap();
}

/// Set up a matched wish with owner and donor, returns (wish_id, owner_token, donor_token).
async fn setup_matched_wish(app: &TestApp) -> (Uuid, String, String) {
    let owner_token = setup_aged_user_with_name(app, "owner@test.com", "Alice").await;
    let donor_token = setup_aged_user_with_name(app, "donor@test.com", "Bob").await;
    let donor_id = get_user_id(app, "donor@test.com").await;
    let wish_id = create_open_wish(app, &owner_token).await;
    force_match(app, wish_id, donor_id).await;
    (wish_id, owner_token, donor_token)
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 1: Auth Guards
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn send_message_without_auth_401() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4();
    let body = json!({ "body": "hello" });
    let (status, _) = app
        .post_json(&format!("/community/wishes/{id}/messages"), &body)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_messages_without_auth_401() {
    let app = TestApp::new().await;
    let id = Uuid::new_v4();
    let (status, _) = app
        .get_no_auth(&format!("/community/wishes/{id}/messages"))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 2: Send Message
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn send_message_owner_201() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, _donor_token) = setup_matched_wish(&app).await;

    let body = json!({ "body": "Thanks for offering!" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(resp["id"].as_str().is_some());
    assert_eq!(resp["body"].as_str(), Some("Thanks for offering!"));
    assert_eq!(resp["sender_display_name"].as_str(), Some("Alice"));
    assert_eq!(resp["is_mine"].as_bool(), Some(true));
    assert!(resp["created_at"].as_str().is_some());
}

#[tokio::test]
async fn send_message_donor_201() {
    let app = TestApp::new().await;
    let (wish_id, _owner_token, donor_token) = setup_matched_wish(&app).await;

    let body = json!({ "body": "Happy to help!" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["sender_display_name"].as_str(), Some("Bob"));
    assert_eq!(resp["is_mine"].as_bool(), Some(true));
}

#[tokio::test]
async fn send_message_not_matched_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let body = json!({ "body": "Hello" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_message_fulfilled_400() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, _donor_token) = setup_matched_wish(&app).await;

    sqlx::query(
        "UPDATE community_wishes SET status = 'fulfilled', fulfilled_at = NOW() WHERE id = $1",
    )
    .bind(wish_id)
    .execute(&app.db)
    .await
    .unwrap();

    let body = json!({ "body": "Hello" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_message_closed_400() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, _donor_token) = setup_matched_wish(&app).await;

    sqlx::query("UPDATE community_wishes SET status = 'closed', closed_at = NOW() WHERE id = $1")
        .bind(wish_id)
        .execute(&app.db)
        .await
        .unwrap();

    let body = json!({ "body": "Hello" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_message_not_participant_403() {
    let app = TestApp::new().await;
    let (wish_id, _owner_token, _donor_token) = setup_matched_wish(&app).await;

    let random_token = setup_aged_user(&app, "random@test.com").await;
    let body = json!({ "body": "Can I join?" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &random_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn send_message_wish_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "user@test.com").await;
    let fake_id = Uuid::new_v4();

    let body = json!({ "body": "Hello" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{fake_id}/messages"),
            &body,
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn send_message_empty_body_400() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, _donor_token) = setup_matched_wish(&app).await;

    let body = json!({ "body": "" });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_message_body_too_long_400() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, _donor_token) = setup_matched_wish(&app).await;

    let long_body = "x".repeat(2001);
    let body = json!({ "body": long_body });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_message_multiple_messages_201() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    for i in 0..3 {
        let body = json!({ "body": format!("Message {i}") });
        let token = if i % 2 == 0 {
            &owner_token
        } else {
            &donor_token
        };
        let (status, _) = app
            .post_json_with_auth(
                &format!("/community/wishes/{wish_id}/messages"),
                &body,
                token,
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "message {i} should succeed");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 3: List Messages
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_messages_matched_owner_200() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Send a message
    let body = json!({ "body": "Hello from owner" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &owner_token,
    )
    .await;

    let body = json!({ "body": "Reply from donor" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &donor_token,
    )
    .await;

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["pagination"]["total"].as_i64(), Some(2));
}

#[tokio::test]
async fn list_messages_matched_donor_200() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    let body = json!({ "body": "Hello" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &owner_token,
    )
    .await;

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_messages_fulfilled_200() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Send a message while matched
    let body = json!({ "body": "Hello" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &owner_token,
    )
    .await;

    // Fulfill
    sqlx::query(
        "UPDATE community_wishes SET status = 'fulfilled', fulfilled_at = NOW() WHERE id = $1",
    )
    .bind(wish_id)
    .execute(&app.db)
    .await
    .unwrap();

    // Should still be able to read
    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_messages_closed_200() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Send a message while matched
    let body = json!({ "body": "Hello" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &owner_token,
    )
    .await;

    // Close
    sqlx::query("UPDATE community_wishes SET status = 'closed', closed_at = NOW() WHERE id = $1")
        .bind(wish_id)
        .execute(&app.db)
        .await
        .unwrap();

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_messages_open_400() {
    let app = TestApp::new().await;
    let owner_token = setup_aged_user(&app, "owner@test.com").await;
    let wish_id = create_open_wish(&app, &owner_token).await;

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn list_messages_not_participant_403() {
    let app = TestApp::new().await;
    let (wish_id, _owner_token, _donor_token) = setup_matched_wish(&app).await;

    let random_token = setup_aged_user(&app, "random@test.com").await;
    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &random_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn list_messages_empty_200() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, _donor_token) = setup_matched_wish(&app).await;

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 0);
    assert_eq!(resp["pagination"]["total"].as_i64(), Some(0));
}

#[tokio::test]
async fn list_messages_pagination_200() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Send 5 messages
    for i in 0..5 {
        let body = json!({ "body": format!("Message {i}") });
        let token = if i % 2 == 0 {
            &owner_token
        } else {
            &donor_token
        };
        app.post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            token,
        )
        .await;
    }

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages?limit=2&page=1"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["pagination"]["total"].as_i64(), Some(5));
    assert_eq!(resp["pagination"]["page"].as_i64(), Some(1));
    assert_eq!(resp["pagination"]["limit"].as_i64(), Some(2));
    assert_eq!(resp["pagination"]["total_pages"].as_i64(), Some(3));
    assert_eq!(resp["pagination"]["has_more"].as_bool(), Some(true));

    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages?limit=2&page=3"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
    assert_eq!(resp["pagination"]["total"].as_i64(), Some(5));
    assert_eq!(resp["pagination"]["page"].as_i64(), Some(3));
    assert_eq!(resp["pagination"]["has_more"].as_bool(), Some(false));
}

#[tokio::test]
async fn list_messages_is_mine_flag_correct() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Owner sends
    let body = json!({ "body": "From owner" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &owner_token,
    )
    .await;

    // Donor sends
    let body = json!({ "body": "From donor" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &donor_token,
    )
    .await;

    // Owner's perspective
    let (_, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &owner_token,
        )
        .await;
    let msgs = resp["data"].as_array().unwrap();
    // Find the message from owner and donor
    let owner_msg = msgs
        .iter()
        .find(|m| m["body"].as_str() == Some("From owner"))
        .unwrap();
    let donor_msg = msgs
        .iter()
        .find(|m| m["body"].as_str() == Some("From donor"))
        .unwrap();
    assert_eq!(owner_msg["is_mine"].as_bool(), Some(true));
    assert_eq!(donor_msg["is_mine"].as_bool(), Some(false));

    // Donor's perspective
    let (_, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor_token,
        )
        .await;
    let msgs = resp["data"].as_array().unwrap();
    let owner_msg = msgs
        .iter()
        .find(|m| m["body"].as_str() == Some("From owner"))
        .unwrap();
    let donor_msg = msgs
        .iter()
        .find(|m| m["body"].as_str() == Some("From donor"))
        .unwrap();
    assert_eq!(owner_msg["is_mine"].as_bool(), Some(false));
    assert_eq!(donor_msg["is_mine"].as_bool(), Some(true));
}

#[tokio::test]
async fn list_messages_wish_not_found_404() {
    let app = TestApp::new().await;
    let token = setup_aged_user(&app, "user@test.com").await;
    let fake_id = Uuid::new_v4();

    let (status, resp) = app
        .get_with_auth(&format!("/community/wishes/{fake_id}/messages"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 4: E2E
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn e2e_message_exchange_then_confirm() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Exchange messages
    let body = json!({ "body": "Thank you for helping!" });
    let (s, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(s, StatusCode::CREATED);

    let body = json!({ "body": "You're welcome!" });
    let (s, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &donor_token,
        )
        .await;
    assert_eq!(s, StatusCode::CREATED);

    // Confirm the wish (fulfill)
    let (s, _) = app
        .post_with_auth(
            &format!("/community/wishes/{wish_id}/confirm"),
            &owner_token,
        )
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Messages should still be readable after fulfilled
    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn e2e_message_exchange_then_close() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Exchange a message
    let body = json!({ "body": "Actually I found one, thanks anyway" });
    let (s, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &owner_token,
        )
        .await;
    assert_eq!(s, StatusCode::CREATED);

    // Close the wish
    let (s, _) = app
        .post_with_auth(&format!("/community/wishes/{wish_id}/close"), &owner_token)
        .await;
    assert_eq!(s, StatusCode::NO_CONTENT);

    // Messages should still be readable after close
    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["data"].as_array().unwrap().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 6: Edge cases after withdrawal
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn send_message_donor_after_withdraw_400() {
    let app = TestApp::new().await;
    let (wish_id, _owner_token, donor_token) = setup_matched_wish(&app).await;

    // Donor withdraws offer → wish goes back to open
    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Donor tries to send message → wish is open, not matched
    let body = json!({ "body": "hello" });
    let (status, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &body,
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_messages_donor_after_withdraw_400() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Send a message while matched
    let body = json!({ "body": "Hello from owner" });
    app.post_json_with_auth(
        &format!("/community/wishes/{wish_id}/messages"),
        &body,
        &owner_token,
    )
    .await;

    // Donor withdraws offer → wish goes back to open
    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Donor tries to list messages → wish is open, not readable
    let (status, _) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ═══════════════════════════════════════════════════════════════════════
// Cat. 7: Message cleanup on withdraw / reject
// ═══════════════════════════════════════════════════════════════════════

/// Helper: count messages in DB for a given wish (bypasses auth).
async fn count_messages_in_db(app: &TestApp, wish_id: Uuid) -> i64 {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wish_messages WHERE wish_id = $1")
        .bind(wish_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    row.0
}

/// Helper: send N messages alternating between owner and donor.
async fn send_messages(
    app: &TestApp,
    wish_id: Uuid,
    owner_token: &str,
    donor_token: &str,
    count: usize,
) {
    for i in 0..count {
        let token = if i % 2 == 0 { owner_token } else { donor_token };
        let body = json!({ "body": format!("Message #{}", i + 1) });
        let (status, _) = app
            .post_json_with_auth(
                &format!("/community/wishes/{wish_id}/messages"),
                &body,
                token,
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "send message #{}", i + 1);
    }
}

#[tokio::test]
async fn withdraw_deletes_all_messages() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Both sides exchange messages
    send_messages(&app, wish_id, &owner_token, &donor_token, 4).await;
    assert_eq!(count_messages_in_db(&app, wish_id).await, 4);

    // Donor withdraws
    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // All messages must be gone
    assert_eq!(
        count_messages_in_db(&app, wish_id).await,
        0,
        "messages must be deleted after donor withdrawal"
    );
}

#[tokio::test]
async fn reject_deletes_all_messages() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Exchange messages
    send_messages(&app, wish_id, &owner_token, &donor_token, 3).await;
    assert_eq!(count_messages_in_db(&app, wish_id).await, 3);

    // Owner rejects the offer
    let (status, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/reject"),
            &json!({}),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // All messages must be gone
    assert_eq!(
        count_messages_in_db(&app, wish_id).await,
        0,
        "messages must be deleted after owner rejects offer"
    );
}

#[tokio::test]
async fn new_donor_sees_empty_conversation_after_withdraw() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // First donor sends messages
    send_messages(&app, wish_id, &owner_token, &donor_token, 2).await;

    // First donor withdraws → messages deleted
    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // New donor enters the picture
    let donor2_token = setup_aged_user_with_name(&app, "donor2@test.com", "Charlie").await;
    let donor2_id = get_user_id(&app, "donor2@test.com").await;
    force_match(&app, wish_id, donor2_id).await;

    // New donor lists messages → must be empty (no leakage from previous donor)
    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        resp["data"].as_array().unwrap().len(),
        0,
        "new donor must not see messages from previous donor"
    );
}

#[tokio::test]
async fn new_donor_sees_empty_conversation_after_reject() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Exchange messages
    send_messages(&app, wish_id, &owner_token, &donor_token, 3).await;

    // Owner rejects → messages deleted
    let (status, _) = app
        .post_json_with_auth(
            &format!("/community/wishes/{wish_id}/reject"),
            &json!({}),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // New donor
    let donor2_token = setup_aged_user_with_name(&app, "donor2@test.com", "Charlie").await;
    let donor2_id = get_user_id(&app, "donor2@test.com").await;
    force_match(&app, wish_id, donor2_id).await;

    // New donor must see zero messages
    let (status, resp) = app
        .get_with_auth(
            &format!("/community/wishes/{wish_id}/messages"),
            &donor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        resp["data"].as_array().unwrap().len(),
        0,
        "new donor must not see messages from rejected donor"
    );
}

#[tokio::test]
async fn withdraw_with_no_messages_succeeds() {
    let app = TestApp::new().await;
    let (wish_id, _owner_token, donor_token) = setup_matched_wish(&app).await;

    // No messages sent — withdraw should still succeed gracefully
    assert_eq!(count_messages_in_db(&app, wish_id).await, 0);

    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(count_messages_in_db(&app, wish_id).await, 0);
}

#[tokio::test]
async fn owner_sees_empty_conversation_after_withdraw() {
    let app = TestApp::new().await;
    let (wish_id, owner_token, donor_token) = setup_matched_wish(&app).await;

    // Exchange messages
    send_messages(&app, wish_id, &owner_token, &donor_token, 4).await;

    // Donor withdraws
    let (status, _) = app
        .delete_with_auth(&format!("/community/wishes/{wish_id}/offer"), &donor_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Owner tries to list messages → wish is now open with no match,
    // but owner should also not see old messages even via DB
    assert_eq!(
        count_messages_in_db(&app, wish_id).await,
        0,
        "messages must be purged so owner cannot see stale conversation"
    );
}
