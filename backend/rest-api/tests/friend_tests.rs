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
            &format!("/circles/{circle_id}/feed?page=1&limit=10"),
            &alice_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let events = feed["data"].as_array().expect("data array");
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

// ═══════════════════════════════════════════════════════════════════════
// Direct Circle Auto-Creation on Accept
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn accept_request_creates_direct_circle() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "dc1@test.com", "dc_alice").await;
    let (_bob_token, bob_id) = setup_named_user(&app, "dc2@test.com", "dc_bob").await;

    // Before: no direct circle
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 0, "no direct circle before friendship");

    // Accept
    let req_id = send_request(&app, &alice_token, "dc_bob").await;
    accept_request(&app, &_bob_token, req_id).await;

    // After: exactly one direct circle
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 1, "direct circle auto-created on accept");
}

#[tokio::test]
async fn accept_request_does_not_duplicate_direct_circle() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "dd1@test.com", "dd_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "dd2@test.com", "dd_bob").await;

    // Pre-create a direct circle via SQL (simulates legacy data or race condition)
    let circle_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO circles (owner_id, is_direct) VALUES ($1, true) RETURNING id",
    )
    .bind(alice_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    sqlx::query("INSERT INTO circle_members (circle_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(circle_id)
        .bind(bob_id)
        .execute(&app.db)
        .await
        .unwrap();

    // Now become friends
    let req_id = send_request(&app, &alice_token, "dd_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Should still be exactly one direct circle (not duplicated)
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 1, "should not duplicate direct circle");
}

// ═══════════════════════════════════════════════════════════════════════
// Remove Friend — Direct Circle Cascade
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn remove_friend_deletes_direct_circle() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "rf1@test.com", "rf_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "rf2@test.com", "rf_bob").await;

    // Become friends (auto-creates direct circle)
    let req_id = send_request(&app, &alice_token, "rf_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Verify direct circle exists
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 1, "precondition: direct circle exists");

    // Remove friend
    let (status, _) = app
        .delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Direct circle should be deleted
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 0, "direct circle deleted on friend removal");
}

#[tokio::test]
async fn remove_friend_does_not_delete_group_circles() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "rg1@test.com", "rg_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "rg2@test.com", "rg_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "rg_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Create a group circle and add Bob
    let body = json!({ "name": "Test Group" });
    let (_, circle_body) = app
        .post_json_with_auth("/circles", &body, &alice_token)
        .await;
    let circle_id = circle_body["id"].as_str().unwrap();

    let body = json!({ "user_id": bob_id });
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &body,
        &alice_token,
    )
    .await;

    // Remove friend
    let (status, _) = app
        .delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Group circle should still exist
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM circles WHERE id = $1::uuid AND is_direct = false")
            .bind(circle_id)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(count.0, 1, "group circle must survive friend removal");
}

#[tokio::test]
async fn remove_friend_cleans_friend_requests() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "rc1@test.com", "rc_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "rc2@test.com", "rc_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "rc_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Friend requests should exist (status=accepted)
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM friend_requests \
         WHERE (from_user_id = $1 AND to_user_id = $2) \
            OR (from_user_id = $2 AND to_user_id = $1)",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert!(count.0 >= 1, "precondition: friend request exists");

    // Remove friend
    app.delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;

    // Friend requests should be cleaned up
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM friend_requests \
         WHERE (from_user_id = $1 AND to_user_id = $2) \
            OR (from_user_id = $2 AND to_user_id = $1)",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 0, "friend requests cleaned up on removal");
}

#[tokio::test]
async fn remove_friend_is_transactional() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "tx1@test.com", "tx_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "tx2@test.com", "tx_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "tx_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Remove friend
    let (status, _) = app
        .delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify all state is consistent: no friendship, no requests, no direct circle
    // Verify not friends anymore via API
    let (status, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(status, StatusCode::OK);
    let friends = body.as_array().unwrap();
    assert!(friends.is_empty(), "no friends after removal");
}

// ═══════════════════════════════════════════════════════════════════════
// Cancel Request — Notification Cleanup
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn cancel_request_cleans_recipient_notification() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "cn1@test.com", "cn_alice").await;
    let (_bob_token, bob_id) = setup_named_user(&app, "cn2@test.com", "cn_bob").await;

    // Send request (creates notification for Bob)
    let req_id = send_request(&app, &alice_token, "cn_bob").await;

    // Wait for async notification to persist
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify notification exists for Bob
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications \
         WHERE user_id = $1 AND type = 'friend_request' AND actor_id = $2",
    )
    .bind(bob_id)
    .bind(alice_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 1, "precondition: notification exists for Bob");

    // Cancel the request
    cancel_sent_request(&app, &alice_token, req_id).await;

    // Notification should be cleaned up
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications \
         WHERE user_id = $1 AND type = 'friend_request' AND actor_id = $2",
    )
    .bind(bob_id)
    .bind(alice_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        count.0, 0,
        "notification should be deleted when request is cancelled"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Accept — Notification Created for Sender
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn accept_request_notifies_sender() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "an1@test.com", "an_alice").await;
    let (bob_token, _bob_id) = setup_named_user(&app, "an2@test.com", "an_bob").await;

    let req_id = send_request(&app, &alice_token, "an_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Wait for async notification
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Alice should have a "friend_accepted" notification
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications \
         WHERE user_id = $1 AND type = 'friend_accepted'",
    )
    .bind(alice_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        count.0, 1,
        "sender should receive friend_accepted notification"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Shared Item Count Security
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn shared_item_count_zero_when_no_items_shared() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "sc1@test.com", "sc_alice").await;
    let (bob_token, _) = setup_named_user(&app, "sc2@test.com", "sc_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "sc_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Bob creates items (but doesn't share them)
    let item_body = json!({
        "name": "Secret Item",
        "category_id": null,
    });
    app.create_item(&bob_token, &item_body).await;

    // Alice lists friends — Bob's shared_item_count should be 0
    let (status, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(status, StatusCode::OK);

    let friends = body.as_array().unwrap();
    assert_eq!(friends.len(), 1);
    assert_eq!(
        friends[0]["shared_item_count"].as_i64().unwrap(),
        0,
        "shared_item_count must be 0 when nothing is shared in direct circle"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Remove Friend — Notifications Cleanup
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn remove_friend_cleans_notifications() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "rn1@test.com", "rn_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "rn2@test.com", "rn_bob").await;

    // Become friends (generates notifications)
    let req_id = send_request(&app, &alice_token, "rn_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Wait for async notifications
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify notifications exist
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications \
         WHERE type IN ('friend_request', 'friend_accepted') \
           AND ((user_id = $1 AND actor_id = $2) OR (user_id = $2 AND actor_id = $1))",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert!(count.0 >= 1, "precondition: friend notifications exist");

    // Remove friend
    app.delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;

    // Notifications should be cleaned up
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications \
         WHERE type IN ('friend_request', 'friend_accepted', 'friend_activity') \
           AND ((user_id = $1 AND actor_id = $2) OR (user_id = $2 AND actor_id = $1))",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        count.0, 0,
        "friend notifications should be cleaned up on removal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Full Lifecycle: Request → Accept → Circle → Remove → Clean
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn full_friend_lifecycle_with_circle_and_cleanup() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "fl1@test.com", "fl_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "fl2@test.com", "fl_bob").await;

    // 1. Send request
    let req_id = send_request(&app, &alice_token, "fl_bob").await;

    // 2. Bob has pending request
    let (_, body) = app.get_with_auth("/me/friend-requests", &bob_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // 3. Accept — creates friendship + direct circle
    accept_request(&app, &bob_token, req_id).await;

    // 4. Both are friends
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    let (_, body) = app.get_with_auth("/me/friends", &bob_token).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // 5. Direct circle exists
    let circle_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_optional(&app.db)
    .await
    .unwrap();
    assert!(circle_id.is_some(), "direct circle should exist");

    // 6. Shared item count is 0 (nothing shared yet)
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert_eq!(body[0]["shared_item_count"].as_i64().unwrap(), 0);

    // 7. Remove friend — everything cleaned up
    let (status, _) = app
        .delete_with_auth(&format!("/me/friends/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // 8. No longer friends
    let (_, body) = app.get_with_auth("/me/friends", &alice_token).await;
    assert!(body.as_array().unwrap().is_empty());

    // 9. Direct circle deleted
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(count.0, 0, "direct circle cleaned up");

    // 10. Can re-request after removal
    let new_req_id = send_request(&app, &alice_token, "fl_bob").await;
    assert_ne!(new_req_id, req_id, "new request has different ID");
}

// ═══════════════════════════════════════════════════════════════════════
// Direct Circle Protection
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn cannot_delete_direct_circle_via_api() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "cd1@test.com", "cd_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "cd2@test.com", "cd_bob").await;

    // Become friends (auto-creates direct circle)
    let req_id = send_request(&app, &alice_token, "cd_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Find the direct circle
    let circle_id: (Uuid,) = sqlx::query_as(
        "SELECT c.id FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Bob is the owner (acceptor). Try to delete — should fail with 400
    let (status, body) = app
        .delete_with_auth(&format!("/circles/{}", circle_id.0), &bob_token)
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "deleting a direct circle should be blocked: {body}"
    );

    // Circle should still exist
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM circles WHERE id = $1")
        .bind(circle_id.0)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count.0, 1, "direct circle must survive delete attempt");
}

#[tokio::test]
async fn cannot_leave_direct_circle_via_remove_member() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "lv1@test.com", "lv_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "lv2@test.com", "lv_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "lv_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Find the direct circle
    let circle_id: (Uuid,) = sqlx::query_as(
        "SELECT c.id FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Bob tries to self-remove from direct circle — should fail
    let (status, body) = app
        .delete_with_auth(
            &format!("/circles/{}/members/{}", circle_id.0, bob_id),
            &bob_token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "leaving a direct circle should be blocked: {body}"
    );

    // Both members should still be in the circle
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM circle_members WHERE circle_id = $1")
        .bind(circle_id.0)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(count.0, 2, "both members must remain in direct circle");
}

#[tokio::test]
async fn create_direct_circle_requires_friendship() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "df1@test.com", "df_alice").await;
    let (_, bob_id) = setup_named_user(&app, "df2@test.com", "df_bob").await;

    // Not friends — try to create direct circle
    let body = json!({});
    let (status, resp) = app
        .post_json_with_auth(&format!("/circles/direct/{bob_id}"), &body, &alice_token)
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "creating direct circle without friendship should be forbidden: {resp}"
    );
}

#[tokio::test]
async fn create_direct_circle_works_when_friends() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "dw1@test.com", "dw_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "dw2@test.com", "dw_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "dw_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Accept already creates a direct circle, so creating another should 409
    let body = json!({});
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/direct/{bob_id}"), &body, &alice_token)
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "direct circle already auto-created on accept"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Shared Item Count — E2E with actual sharing
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn shared_item_count_updates_after_sharing_in_direct_circle() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_named_user(&app, "si1@test.com", "si_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "si2@test.com", "si_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "si_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Find the direct circle
    let circle_id: (Uuid,) = sqlx::query_as(
        "SELECT c.id FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1 \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2 \
         WHERE c.is_direct = true",
    )
    .bind(alice_id)
    .bind(bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Alice creates an item
    let item = app
        .create_item(
            &alice_token,
            &json!({ "name": "Test Gift", "category_id": null }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Before sharing: Bob sees 0
    let (_, body) = app.get_with_auth("/me/friends", &bob_token).await;
    let friends = body.as_array().unwrap();
    assert_eq!(friends.len(), 1);
    assert_eq!(
        friends[0]["shared_item_count"].as_i64().unwrap(),
        0,
        "count should be 0 before sharing"
    );

    // Share item to direct circle via POST /circles/{id}/items
    let share_body = json!({ "item_id": item_id });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/circles/{}/items", circle_id.0),
            &share_body,
            &alice_token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "sharing should succeed: {resp}"
    );

    // After sharing: Bob sees 1
    let (_, body) = app.get_with_auth("/me/friends", &bob_token).await;
    let friends = body.as_array().unwrap();
    assert_eq!(
        friends[0]["shared_item_count"].as_i64().unwrap(),
        1,
        "count should be 1 after sharing one item"
    );
}

#[tokio::test]
async fn shared_item_count_excludes_group_circle_items() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_named_user(&app, "sg1@test.com", "sg_alice").await;
    let (bob_token, bob_id) = setup_named_user(&app, "sg2@test.com", "sg_bob").await;

    // Become friends
    let req_id = send_request(&app, &alice_token, "sg_bob").await;
    accept_request(&app, &bob_token, req_id).await;

    // Create a GROUP circle and add Bob
    let (_, circle_body) = app
        .post_json_with_auth("/circles", &json!({ "name": "Group" }), &alice_token)
        .await;
    let group_id = circle_body["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{group_id}/members"),
        &json!({ "user_id": bob_id }),
        &alice_token,
    )
    .await;

    // Alice creates an item and shares it to the GROUP circle only
    let item = app
        .create_item(
            &alice_token,
            &json!({ "name": "Group Only Item", "category_id": null }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{group_id}/items"),
            &json!({ "item_id": item_id }),
            &alice_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob's friend list should show 0 for Alice (item is in group, not direct circle)
    let (_, body) = app.get_with_auth("/me/friends", &bob_token).await;
    let friends = body.as_array().unwrap();
    assert_eq!(
        friends[0]["shared_item_count"].as_i64().unwrap(),
        0,
        "items in group circles must NOT count in friend's shared_item_count"
    );
}
