mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp};
use serde_json::json;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────

/// Register a user with a specific username and return (token, user_id).
async fn setup_named_user(app: &TestApp, email: &str, username: &str) -> (String, Uuid) {
    let (status, body) = app
        .register_user_with_username(email, TEST_PASSWORD, username)
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "precondition: register {username} should return 201, got {status}: {body}"
    );
    let token = body["tokens"]["access_token"]
        .as_str()
        .expect("access_token")
        .to_string();
    let user_id =
        Uuid::parse_str(body["user"]["id"].as_str().expect("user.id")).expect("valid uuid");
    (token, user_id)
}

/// Alice sends a friend request to Bob (by Bob's username). Returns request id.
async fn send_request(app: &TestApp, from_token: &str, to_username: &str) -> Uuid {
    let body = json!({ "username": to_username });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, from_token)
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "precondition: send request should return 201, got {status}: {resp}"
    );
    Uuid::parse_str(resp["id"].as_str().expect("id")).expect("valid uuid")
}

/// Accept a friend request. Returns the FriendResponse.
async fn accept_request(app: &TestApp, token: &str, request_id: Uuid) -> serde_json::Value {
    let (status, body) = app
        .post_with_auth(&format!("/me/friend-requests/{request_id}/accept"), token)
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "precondition: accept should return 200, got {status}: {body}"
    );
    body
}

/// Cancel a sent friend request (sender side). Asserts 204.
async fn cancel_sent_request(app: &TestApp, token: &str, request_id: Uuid) {
    let (status, body) = app
        .delete_with_auth(&format!("/me/friend-requests/{request_id}/cancel"), token)
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "precondition: cancel should return 204, got {status}: {body}"
    );
}

/// Set up two users and make them friends. Returns (alice_token, alice_id, bob_token, bob_id).
async fn setup_friends(app: &TestApp) -> (String, Uuid, String, Uuid) {
    let (alice_token, alice_id) = setup_named_user(app, "alice@test.com", "alice_test").await;
    let (bob_token, bob_id) = setup_named_user(app, "bob@test.com", "bob_test").await;

    let req_id = send_request(app, &alice_token, "bob_test").await;
    accept_request(app, &bob_token, req_id).await;

    (alice_token, alice_id, bob_token, bob_id)
}

// ═══════════════════════════════════════════════════════════════════════
// User Search
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn search_users_returns_matching_users() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;
    setup_named_user(&app, "bob2@test.com", "bobby_test").await;

    let (status, body) = app.get_with_auth("/users/search?q=bob", &alice_token).await;

    assert_eq!(status, StatusCode::OK);
    let results = body.as_array().expect("array");
    assert_eq!(
        results.len(),
        2,
        "both bob and bobby should match prefix 'bob'"
    );
    let usernames: Vec<&str> = results
        .iter()
        .map(|r| r["username"].as_str().unwrap())
        .collect();
    assert!(usernames.contains(&"bob_test"));
    assert!(usernames.contains(&"bobby_test"));
}

#[tokio::test]
async fn search_users_excludes_self() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let (status, body) = app
        .get_with_auth("/users/search?q=alice", &alice_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    let results = body.as_array().expect("array");
    assert!(
        results.is_empty(),
        "self should be excluded from search results"
    );
}

#[tokio::test]
async fn search_users_empty_query_returns_400() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let (status, _body) = app.get_with_auth("/users/search?q=", &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_users_no_auth_returns_401() {
    let app = TestApp::new().await;

    let (status, _) = app.get_no_auth("/users/search?q=bob").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn search_users_returns_display_name() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    // Register bob with explicit username AND display name
    let body = serde_json::json!({
        "email": "bob@test.com",
        "password": TEST_PASSWORD,
        "display_name": "Bob Le Grand",
        "username": "bob_display"
    });
    let (status, _) = app.post_json("/auth/register", &body).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = app
        .get_with_auth("/users/search?q=bob_display", &alice_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    let results = body.as_array().expect("array");
    assert_eq!(results.len(), 1, "should find bob_display");
    assert_eq!(results[0]["username"].as_str().unwrap(), "bob_display");
    assert_eq!(
        results[0]["display_name"].as_str().unwrap(),
        "Bob Le Grand",
        "search should return display_name when present"
    );
}

#[tokio::test]
async fn search_users_limits_results_to_10() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    // Create 12 users with same prefix
    for i in 0..12 {
        setup_named_user(&app, &format!("user{i}@test.com"), &format!("ztest_{i:02}")).await;
    }

    let (status, body) = app
        .get_with_auth("/users/search?q=ztest_", &alice_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    let results = body.as_array().expect("array");
    assert!(
        results.len() <= 10,
        "search should be limited to 10 results"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Send Friend Request
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn send_friend_request_happy_path() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let body = json!({ "username": "bob_test" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(resp["id"].as_str().is_some(), "response should have id");
    assert_eq!(resp["from_user_id"].as_str().unwrap(), alice_id.to_string());
    assert_eq!(resp["from_username"].as_str().unwrap(), "alice_test");
    assert_eq!(resp["status"].as_str().unwrap(), "pending");
    assert!(resp["created_at"].as_str().is_some());
}

#[tokio::test]
async fn send_friend_request_to_self_returns_400() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let body = json!({ "username": "alice_test" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_friend_request_unknown_user_returns_404() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let body = json!({ "username": "nonexistent_user" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn send_friend_request_already_friends_returns_409() {
    let app = TestApp::new().await;
    let (alice_token, _, _, _) = setup_friends(&app).await;

    // Alice tries to send another request to Bob (who is already a friend)
    let body = json!({ "username": "bob_test" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn send_friend_request_duplicate_pending_returns_409() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    // First request
    send_request(&app, &alice_token, "bob_test").await;

    // Duplicate request
    let body = json!({ "username": "bob_test" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn send_friend_request_reverse_pending_returns_409() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    // Bob sends request to Alice first
    send_request(&app, &bob_token, "alice_test").await;

    // Alice tries to send request to Bob (reverse direction already pending)
    let body = json!({ "username": "bob_test" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn send_friend_request_empty_username_returns_400() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let body = json!({ "username": "" });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn send_friend_request_no_auth_returns_401() {
    let app = TestApp::new().await;

    let body = json!({ "username": "bob_test" });
    let (status, _) = app.post_json("/me/friend-requests", &body).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════════
// List Pending Requests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_pending_returns_received_requests() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    send_request(&app, &alice_token, "bob_test").await;

    // Bob should see the pending request
    let (status, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["from_username"].as_str().unwrap(), "alice_test");
    assert_eq!(
        requests[0]["from_user_id"].as_str().unwrap(),
        alice_id.to_string()
    );
    assert_eq!(requests[0]["status"].as_str().unwrap(), "pending");
}

#[tokio::test]
async fn list_pending_does_not_show_sent_requests() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    send_request(&app, &alice_token, "bob_test").await;

    // Alice should NOT see her own sent requests in the pending list
    let (status, body) = app.get_with_auth("/me/friend-requests", &alice_token).await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert!(
        requests.is_empty(),
        "sender should not see sent requests in pending list"
    );
}

#[tokio::test]
async fn list_pending_empty_when_none() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let (status, body) = app.get_with_auth("/me/friend-requests", &token).await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn list_pending_multiple_requests() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;
    let (carol_token, _) = setup_named_user(&app, "carol@test.com", "carol_test").await;

    send_request(&app, &alice_token, "bob_test").await;
    send_request(&app, &carol_token, "bob_test").await;

    let (status, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert_eq!(requests.len(), 2, "Bob should have 2 pending requests");

    let senders: Vec<&str> = requests
        .iter()
        .map(|r| r["from_username"].as_str().unwrap())
        .collect();
    assert!(senders.contains(&"alice_test"));
    assert!(senders.contains(&"carol_test"));
}

// ═══════════════════════════════════════════════════════════════════════
// Accept Friend Request
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn accept_request_creates_friendship() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    let resp = accept_request(&app, &bob_token, req_id).await;

    // Response should be a FriendResponse with the sender's info
    assert_eq!(resp["user_id"].as_str().unwrap(), alice_id.to_string());
    assert_eq!(resp["username"].as_str().unwrap(), "alice_test");
    assert!(resp["since"].as_str().is_some(), "should have since field");

    // Verify both users see each other in friends list
    let (status, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().expect("array");
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0]["username"].as_str().unwrap(), "bob_test");

    let (status, body) = app.get_with_auth("/me/friends", &bob_token).await;
    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().expect("array");
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0]["username"].as_str().unwrap(), "alice_test");
}

#[tokio::test]
async fn accept_request_not_recipient_returns_403() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;
    let (carol_token, _) = setup_named_user(&app, "carol@test.com", "carol_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Carol tries to accept Bob's request → 403
    let (status, resp) = app
        .post_with_auth(
            &format!("/me/friend-requests/{req_id}/accept"),
            &carol_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn accept_request_already_accepted_returns_400() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    accept_request(&app, &bob_token, req_id).await;

    // Try to accept again
    let (status, resp) = app
        .post_with_auth(&format!("/me/friend-requests/{req_id}/accept"), &bob_token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn accept_request_not_found_returns_404() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .post_with_auth(&format!("/me/friend-requests/{fake_id}/accept"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn accept_request_clears_from_pending_list() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Verify pending shows 1
    let (_, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    accept_request(&app, &bob_token, req_id).await;

    // Pending should now be empty (accepted requests are not "pending" anymore)
    let (_, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "accepted request should disappear from pending list"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Decline Friend Request
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn decline_request_returns_204() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    let (status, _) = app
        .delete_with_auth(&format!("/me/friend-requests/{req_id}"), &bob_token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Should not appear in pending anymore
    let (_, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;
    assert!(body.as_array().unwrap().is_empty());

    // Should NOT create a friendship
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn decline_request_not_recipient_returns_403() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;
    let (carol_token, _) = setup_named_user(&app, "carol@test.com", "carol_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    let (status, resp) = app
        .delete_with_auth(&format!("/me/friend-requests/{req_id}"), &carol_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn decline_request_already_declined_returns_400() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Decline once
    let (status, _) = app
        .delete_with_auth(&format!("/me/friend-requests/{req_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Decline again
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friend-requests/{req_id}"), &bob_token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn decline_request_not_found_returns_404() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friend-requests/{fake_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════════
// List Friends
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_friends_returns_accepted_friends() {
    let app = TestApp::new().await;
    let (alice_token, _, bob_token, bob_id) = setup_friends(&app).await;

    let (status, body) = app.get_with_auth("/me/friends", &alice_token).await;

    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().expect("array");
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0]["user_id"].as_str().unwrap(), bob_id.to_string());
    assert_eq!(friends[0]["username"].as_str().unwrap(), "bob_test");
    assert!(
        friends[0]["since"].as_str().is_some(),
        "should include friendship date"
    );

    // Bob should also see Alice
    let (status, body) = app.get_with_auth("/me/friends", &bob_token).await;
    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().expect("array");
    assert_eq!(friends.len(), 1);
    assert_eq!(friends[0]["username"].as_str().unwrap(), "alice_test");
}

#[tokio::test]
async fn list_friends_empty_when_none() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let (status, body) = app.get_with_auth("/me/friends", &token).await;

    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().expect("array");
    assert!(friends.is_empty());
}

#[tokio::test]
async fn list_friends_does_not_include_pending_requests() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    // Send request but don't accept
    send_request(&app, &alice_token, "bob_test").await;

    // Alice's friends list should be empty
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "pending requests should not appear in friends list"
    );
}

#[tokio::test]
async fn list_friends_no_auth_returns_401() {
    let app = TestApp::new().await;

    let (status, _) = app.get_no_auth("/me/friends").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_friends_user_isolation() {
    let app = TestApp::new().await;
    let (alice_token, _, _, _) = setup_friends(&app).await;

    // Carol has no friends
    let (carol_token, _) = setup_named_user(&app, "carol@test.com", "carol_test").await;

    let (_, body) = app.get_with_auth("/me/friends", &carol_token).await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "Carol should not see Alice/Bob's friendship"
    );

    // But Alice still has Bob
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Remove Friend
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn remove_friend_deletes_for_both_users() {
    let app = TestApp::new().await;
    let (alice_token, _, bob_token, bob_id) = setup_friends(&app).await;

    // Alice removes Bob
    let (status, _) = app
        .delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Neither should see the other
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(body.as_array().unwrap().is_empty());

    let (_, body) = app.get_with_auth("/me/friends", &bob_token).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn remove_friend_bob_can_remove_alice() {
    let app = TestApp::new().await;
    let (alice_token, alice_id, bob_token, _) = setup_friends(&app).await;

    // Bob removes Alice (reverse direction should also work)
    let (status, _) = app
        .delete_with_auth(&format!("/me/friends/{alice_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn remove_friend_nonexistent_returns_404() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friends/{fake_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn remove_friend_not_friends_returns_404() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (_, bob_id) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    // Try to remove a user who is not a friend
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn remove_friend_allows_re_requesting() {
    let app = TestApp::new().await;
    let (alice_token, _, _, bob_id) = setup_friends(&app).await;

    // Remove friend
    app.delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;

    // Should be able to send a new friend request
    let body = json!({ "username": "bob_test" });
    let (status, _) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "should be able to re-request friendship after removal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Add Member to Circle (via friendship)
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn add_member_requires_friendship() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (_, bob_id) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    // Alice creates a circle
    let circle_body = json!({ "name": "Test Circle" });
    let (status, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let circle_id = circle["id"].as_str().unwrap();

    // Alice tries to add Bob without being friends → 403
    let member_body = json!({ "user_id": bob_id.to_string() });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn add_member_friend_succeeds() {
    let app = TestApp::new().await;
    let (alice_token, _, bob_token, bob_id) = setup_friends(&app).await;

    // Alice creates a circle
    let circle_body = json!({ "name": "Friend Circle" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    // Alice adds Bob (who is a friend) → 201
    let member_body = json!({ "user_id": bob_id.to_string() });
    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);

    // Verify Bob can see the circle
    let (status, body) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let members = body["members"].as_array().expect("members array");
    assert_eq!(members.len(), 2, "circle should have Alice + Bob");
}

#[tokio::test]
async fn add_member_already_member_returns_409() {
    let app = TestApp::new().await;
    let (alice_token, _, _, bob_id) = setup_friends(&app).await;

    let circle_body = json!({ "name": "Test Circle" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let member_body = json!({ "user_id": bob_id.to_string() });

    // Add once
    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &alice_token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Add again → 409
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    common::assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn add_member_not_circle_member_returns_403() {
    let app = TestApp::new().await;
    let (alice_token, alice_id, bob_token, _) = setup_friends(&app).await;
    let (carol_token, _) = setup_named_user(&app, "carol@test.com", "carol_test").await;

    // Make Bob and Carol friends too
    let req_id = send_request(&app, &bob_token, "carol_test").await;
    accept_request(&app, &carol_token, req_id).await;

    // Alice creates a circle (only Alice is member)
    let circle_body = json!({ "name": "Alice Only" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    // Bob tries to add Alice to a circle Bob is NOT a member of → 403
    let member_body = json!({ "user_id": alice_id.to_string() });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &bob_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn add_member_nonexistent_user_returns_404() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let circle_body = json!({ "name": "Test Circle" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let fake_user_id = Uuid::new_v4();
    let member_body = json!({ "user_id": fake_user_id.to_string() });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn add_member_creates_member_joined_event() {
    let app = TestApp::new().await;
    let (alice_token, _, _, bob_id) = setup_friends(&app).await;

    let circle_body = json!({ "name": "Event Circle" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    // Add Bob
    let member_body = json!({ "user_id": bob_id.to_string() });
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &member_body,
        &alice_token,
    )
    .await;

    // Check feed for member_joined event
    let (status, feed) = app
        .get_with_auth(
            &format!("/circles/{circle_id}/feed?page=1&per_page=10"),
            &alice_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let events = feed["events"].as_array().expect("events array");
    let joined_events: Vec<_> = events
        .iter()
        .filter(|e| e["event_type"].as_str() == Some("member_joined"))
        .collect();

    // At least 1 member_joined event (there might also be the owner's join)
    assert!(
        !joined_events.is_empty(),
        "should have member_joined event in feed"
    );

    // Bob's ID is the actor_id in the member_joined event
    let bob_joined = joined_events
        .iter()
        .any(|e| e["actor_id"].as_str() == Some(&bob_id.to_string()));
    assert!(bob_joined, "should have Bob's member_joined event");
}

#[tokio::test]
async fn add_member_no_auth_returns_401() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let circle_body = json!({ "name": "Test Circle" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let member_body = json!({ "user_id": Uuid::new_v4().to_string() });
    let (status, _) = app
        .post_json(&format!("/circles/{circle_id}/members"), &member_body)
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════════
// List Sent Requests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_sent_returns_sent_requests() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (_, bob_id) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    send_request(&app, &alice_token, "bob_test").await;

    let (status, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["to_username"].as_str().unwrap(), "bob_test");
    assert_eq!(
        requests[0]["to_user_id"].as_str().unwrap(),
        bob_id.to_string()
    );
    assert_eq!(requests[0]["status"].as_str().unwrap(), "pending");
    assert!(requests[0]["id"].as_str().is_some());
    assert!(requests[0]["created_at"].as_str().is_some());
}

#[tokio::test]
async fn list_sent_does_not_show_received_requests() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    // Bob sends to Alice
    send_request(&app, &bob_token, "alice_test").await;

    // Alice's sent list should be empty
    let (status, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert!(
        requests.is_empty(),
        "received requests should not appear in sent list"
    );
}

#[tokio::test]
async fn list_sent_empty_when_none() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let (status, body) = app.get_with_auth("/me/friend-requests/sent", &token).await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn list_sent_no_auth_returns_401() {
    let app = TestApp::new().await;

    let (status, _) = app.get_no_auth("/me/friend-requests/sent").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_sent_clears_after_cancel() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Verify it appears
    let (_, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // Cancel
    cancel_sent_request(&app, &alice_token, req_id).await;

    // Should be gone
    let (_, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "cancelled request should disappear from sent list"
    );
}

#[tokio::test]
async fn list_sent_clears_after_recipient_accepts() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    accept_request(&app, &bob_token, req_id).await;

    // Alice's sent list should be empty (request is no longer pending)
    let (_, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "accepted request should disappear from sent list"
    );
}

#[tokio::test]
async fn list_sent_clears_after_recipient_declines() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    app.delete_with_auth(&format!("/me/friend-requests/{req_id}"), &bob_token)
        .await;

    let (_, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "declined request should disappear from sent list"
    );
}

#[tokio::test]
async fn list_sent_multiple_requests() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;
    setup_named_user(&app, "carol@test.com", "carol_test").await;

    send_request(&app, &alice_token, "bob_test").await;
    send_request(&app, &alice_token, "carol_test").await;

    let (status, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    let requests = body.as_array().expect("array");
    assert_eq!(requests.len(), 2);

    let targets: Vec<&str> = requests
        .iter()
        .map(|r| r["to_username"].as_str().unwrap())
        .collect();
    assert!(targets.contains(&"bob_test"));
    assert!(targets.contains(&"carol_test"));
}

#[tokio::test]
async fn list_sent_includes_display_name() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    // Register bob with display name
    let body = json!({
        "email": "bob@test.com",
        "password": TEST_PASSWORD,
        "display_name": "Robert",
        "username": "bob_test"
    });
    let (status, _) = app.post_json("/auth/register", &body).await;
    assert_eq!(status, StatusCode::CREATED);

    send_request(&app, &alice_token, "bob_test").await;

    let (_, body) = app
        .get_with_auth("/me/friend-requests/sent", &alice_token)
        .await;
    let requests = body.as_array().expect("array");
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0]["to_display_name"].as_str().unwrap(),
        "Robert",
        "sent request should include target display_name"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Cancel Friend Request
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn cancel_request_happy_path() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    let (status, _) = app
        .delete_with_auth(
            &format!("/me/friend-requests/{req_id}/cancel"),
            &alice_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob should NOT see it in pending anymore
    let (_, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;
    assert!(
        body.as_array().unwrap().is_empty(),
        "cancelled request should not appear in recipient's pending list"
    );

    // Should NOT create a friendship
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn cancel_request_not_sender_returns_403() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Bob (recipient) tries to cancel → 403
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friend-requests/{req_id}/cancel"), &bob_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn cancel_request_third_party_returns_403() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;
    let (carol_token, _) = setup_named_user(&app, "carol@test.com", "carol_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Carol (unrelated) tries to cancel → 403
    let (status, resp) = app
        .delete_with_auth(
            &format!("/me/friend-requests/{req_id}/cancel"),
            &carol_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn cancel_request_not_found_returns_404() {
    let app = TestApp::new().await;
    let (token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;

    let fake_id = Uuid::new_v4();
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friend-requests/{fake_id}/cancel"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn cancel_request_already_cancelled_returns_400() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    cancel_sent_request(&app, &alice_token, req_id).await;

    // Try again
    let (status, resp) = app
        .delete_with_auth(
            &format!("/me/friend-requests/{req_id}/cancel"),
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn cancel_request_already_accepted_returns_400() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    accept_request(&app, &bob_token, req_id).await;

    // Sender tries to cancel an accepted request → 400
    let (status, resp) = app
        .delete_with_auth(
            &format!("/me/friend-requests/{req_id}/cancel"),
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn cancel_request_no_auth_returns_401() {
    let app = TestApp::new().await;

    let fake_id = Uuid::new_v4();
    let (status, _) = app
        .delete_with_auth(
            &format!("/me/friend-requests/{fake_id}/cancel"),
            "invalid-token",
        )
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn cancelled_request_allows_re_requesting() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    cancel_sent_request(&app, &alice_token, req_id).await;

    // Should be able to send a new request
    let body = json!({ "username": "bob_test" });
    let (status, _) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "should allow new request after cancel"
    );
}

#[tokio::test]
async fn cancel_request_status_persisted_in_db() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    cancel_sent_request(&app, &alice_token, req_id).await;

    let status: (String,) = sqlx::query_as("SELECT status FROM friend_requests WHERE id = $1")
        .bind(req_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(status.0, "cancelled");
}

// ═══════════════════════════════════════════════════════════════════════
// Edge Cases & Integration
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn full_friendship_lifecycle() {
    let app = TestApp::new().await;
    let (alice_token, _alice_id) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, bob_id) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    // 1. Search finds Bob
    let (_, body) = app.get_with_auth("/users/search?q=bob", &alice_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // 2. Send request
    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // 3. Bob sees pending request
    let (_, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // 4. Bob accepts
    accept_request(&app, &bob_token, req_id).await;

    // 5. Both see each other as friends
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    let (_, body) = app.get_with_auth("/me/friends", &bob_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // 6. Create circle and add friend
    let circle_body = json!({ "name": "Lifecycle Circle" });
    let (_, circle) = app
        .post_json_with_auth("/circles", &circle_body, &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

    let member_body = json!({ "user_id": bob_id.to_string() });
    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/members"),
            &member_body,
            &alice_token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // 7. Remove friend
    app.delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;

    // 8. Friends list empty
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(body.as_array().unwrap().is_empty());

    // 9. Can re-request
    let body = json!({ "username": "bob_test" });
    let (status, _) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn declined_request_allows_new_request() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    // Alice sends, Bob declines
    let req_id = send_request(&app, &alice_token, "bob_test").await;
    app.delete_with_auth(&format!("/me/friend-requests/{req_id}"), &bob_token)
        .await;

    // Alice can send a new request
    let body = json!({ "username": "bob_test" });
    let (status, _) = app
        .post_json_with_auth("/me/friend-requests", &body, &alice_token)
        .await;

    assert_eq!(
        status,
        StatusCode::CREATED,
        "should allow new request after decline"
    );
}

#[tokio::test]
async fn sender_cannot_accept_own_request() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Alice tries to accept her own request → 403
    let (status, resp) = app
        .post_with_auth(
            &format!("/me/friend-requests/{req_id}/accept"),
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn sender_cannot_decline_own_request() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Alice tries to decline her own request → 403
    let (status, resp) = app
        .delete_with_auth(&format!("/me/friend-requests/{req_id}"), &alice_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    common::assert_error(&resp, "FORBIDDEN");
}

// ═══════════════════════════════════════════════════════════════════════
// Database integrity
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn friendship_stored_in_canonical_order() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, bob_id) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;
    accept_request(&app, &bob_token, req_id).await;

    // Verify canonical order in DB: user_a_id < user_b_id
    let (a, b) = if alice_id < bob_id {
        (alice_id, bob_id)
    } else {
        (bob_id, alice_id)
    };

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM friendships WHERE user_a_id = $1 AND user_b_id = $2")
            .bind(a)
            .bind(b)
            .fetch_one(&app.db)
            .await
            .unwrap();

    assert_eq!(count.0, 1, "friendship should exist in canonical order");
}

#[tokio::test]
async fn friend_request_status_transitions_persisted() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "alice@test.com", "alice_test").await;
    let (bob_token, _) = setup_named_user(&app, "bob@test.com", "bob_test").await;

    let req_id = send_request(&app, &alice_token, "bob_test").await;

    // Check pending in DB
    let status: (String,) = sqlx::query_as("SELECT status FROM friend_requests WHERE id = $1")
        .bind(req_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(status.0, "pending");

    // Accept
    accept_request(&app, &bob_token, req_id).await;

    // Check accepted in DB
    let status: (String,) = sqlx::query_as("SELECT status FROM friend_requests WHERE id = $1")
        .bind(req_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(status.0, "accepted");
}
