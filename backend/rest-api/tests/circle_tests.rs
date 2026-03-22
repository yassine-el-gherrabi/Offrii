mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};

// ── Helpers ─────────────────────────────────────────────────────────

const ALICE_EMAIL: &str = "alice-circle@example.com";
const BOB_EMAIL: &str = "bob-circle@example.com";
const CHARLIE_EMAIL: &str = "charlie-circle@example.com";

/// Register a user, return (access_token, user_id, username).
async fn setup_user_with_id(app: &TestApp, email: &str) -> (String, String) {
    let body = app.setup_user(email, TEST_PASSWORD).await;
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Register a user with explicit username, return (access_token, user_id, username).
async fn setup_user_named(app: &TestApp, email: &str, username: &str) -> (String, String) {
    let (status, body) = app
        .register_user_with_username(email, TEST_PASSWORD, username)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Make two users friends. Requires their tokens and one username.
async fn make_friends(app: &TestApp, a_token: &str, b_token: &str, b_username: &str) {
    let body = serde_json::json!({ "username": b_username });
    let (status, resp) = app
        .post_json_with_auth("/me/friend-requests", &body, a_token)
        .await;
    assert_eq!(status, StatusCode::CREATED, "send friend request: {resp}");
    let req_id = resp["id"].as_str().unwrap();
    let (status, _) = app
        .post_with_auth(&format!("/me/friend-requests/{req_id}/accept"), b_token)
        .await;
    assert_eq!(status, StatusCode::OK, "accept friend request");
}

/// Create a circle and return its id.
async fn create_circle(app: &TestApp, token: &str, name: &str) -> String {
    let body = serde_json::json!({ "name": name });
    let (status, resp) = app.post_json_with_auth("/circles", &body, token).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create_circle precondition: {resp}"
    );
    resp["id"].as_str().unwrap().to_string()
}

/// Create an invite for a circle (default params) and return the invite token.
async fn create_invite_token(app: &TestApp, circle_id: &str, token: &str) -> String {
    let (status, invite) = app
        .post_with_auth(&format!("/circles/{circle_id}/invite"), token)
        .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "create_invite precondition: {invite}"
    );
    invite["token"].as_str().unwrap().to_string()
}

/// Invite a user to a circle and have them join. Returns the invite token.
async fn invite_and_join(
    app: &TestApp,
    circle_id: &str,
    owner_token: &str,
    joiner_token: &str,
) -> String {
    let invite_tok = create_invite_token(app, circle_id, owner_token).await;
    let (status, _) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), joiner_token)
        .await;
    assert_eq!(status, StatusCode::OK, "join precondition failed");
    invite_tok
}

// ═══════════════════════════════════════════════════════════════════
// DEV-45: CRUD & Membership
// ═══════════════════════════════════════════════════════════════════

// ── Auth guards ────────────────────────────────────────────────────

#[tokio::test]
async fn create_circle_without_auth_401() {
    let app = TestApp::new().await;
    let body = serde_json::json!({ "name": "NoAuth" });
    let (status, _) = app.post_json("/circles", &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_circles_without_auth_401() {
    let app = TestApp::new().await;
    let (status, _) = app.get_no_auth("/circles").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_circle_without_auth_401() {
    let app = TestApp::new().await;
    let (status, _) = app
        .get_no_auth("/circles/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn share_item_without_auth_401() {
    let app = TestApp::new().await;
    let body = serde_json::json!({ "item_id": "00000000-0000-0000-0000-000000000000" });
    let (status, _) = app
        .post_json("/circles/00000000-0000-0000-0000-000000000000/items", &body)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn feed_without_auth_401() {
    let app = TestApp::new().await;
    let (status, _) = app
        .get_no_auth("/circles/00000000-0000-0000-0000-000000000000/feed")
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Validation ─────────────────────────────────────────────────────

#[tokio::test]
async fn create_circle_empty_name_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let body = serde_json::json!({ "name": "" });
    let (status, resp) = app.post_json_with_auth("/circles", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_circle_name_too_long_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let long_name = "x".repeat(101);
    let body = serde_json::json!({ "name": long_name });
    let (status, resp) = app.post_json_with_auth("/circles", &body, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_circle_empty_name_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &token, "Original").await;

    let update = serde_json::json!({ "name": "" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/circles/{circle_id}"), &update, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn update_circle_name_too_long_400() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &token, "Original").await;

    let long_name = "y".repeat(101);
    let update = serde_json::json!({ "name": long_name });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/circles/{circle_id}"), &update, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

// ── Create ─────────────────────────────────────────────────────────

#[tokio::test]
async fn create_circle_returns_201() {
    let app = TestApp::new().await;
    let (token, user_id) = setup_user_with_id(&app, ALICE_EMAIL).await;

    let body = serde_json::json!({ "name": "Famille" });
    let (status, resp) = app.post_json_with_auth("/circles", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["name"], "Famille");
    assert_eq!(resp["is_direct"], false);
    assert_eq!(resp["member_count"], 1); // owner auto-added
    assert_eq!(resp["owner_id"], user_id);
    assert!(resp["id"].is_string());
    assert!(resp["created_at"].is_string());
}

#[tokio::test]
async fn create_circle_100_char_name_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let name = "a".repeat(100);
    let body = serde_json::json!({ "name": name });
    let (status, resp) = app.post_json_with_auth("/circles", &body, &token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["name"].as_str().unwrap().len(), 100);
}

// ── List ───────────────────────────────────────────────────────────

#[tokio::test]
async fn list_circles_returns_user_circles() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    create_circle(&app, &token, "Circle A").await;
    create_circle(&app, &token, "Circle B").await;

    let (status, resp) = app.get_with_auth("/circles", &token).await;
    assert_eq!(status, StatusCode::OK);

    let arr = resp["data"].as_array().unwrap();
    assert_eq!(arr.len(), 2);

    let names: Vec<&str> = arr.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Circle A"));
    assert!(names.contains(&"Circle B"));

    // Each entry has required fields
    for c in arr {
        assert!(c["id"].is_string());
        assert!(c["owner_id"].is_string());
        assert!(c["member_count"].is_i64());
        assert!(c["created_at"].is_string());
    }
}

#[tokio::test]
async fn list_circles_isolation_between_users() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    create_circle(&app, &alice, "Alice Circle").await;

    let (_, resp) = app.get_with_auth("/circles", &bob).await;
    assert_eq!(resp["data"].as_array().unwrap().len(), 0);
}

// ── Get detail ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_circle_detail_shows_members() {
    let app = TestApp::new().await;
    let (token, user_id) = setup_user_with_id(&app, ALICE_EMAIL).await;

    let body = serde_json::json!({ "name": "Team" });
    let (_, circle) = app.post_json_with_auth("/circles", &body, &token).await;
    let circle_id = circle["id"].as_str().unwrap();

    let (status, detail) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["name"], "Team");
    assert_eq!(detail["is_direct"], false);
    assert_eq!(detail["owner_id"], user_id);

    let members = detail["members"].as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["role"], "owner");
    assert_eq!(members[0]["user_id"], user_id);
    assert!(members[0]["username"].is_string());
    assert!(!members[0]["username"].as_str().unwrap().is_empty());
    assert!(members[0]["joined_at"].is_string());
}

#[tokio::test]
async fn get_circle_requires_membership() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "Private").await;

    let (status, resp) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn get_nonexistent_circle_returns_403() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let fake_id = "00000000-0000-0000-0000-000000000000";
    let (status, resp) = app
        .get_with_auth(&format!("/circles/{fake_id}"), &token)
        .await;
    // Not a member → FORBIDDEN (circle doesn't exist = no membership row)
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

// ── Update ─────────────────────────────────────────────────────────

#[tokio::test]
async fn update_circle_name_by_owner() {
    let app = TestApp::new().await;
    let (token, user_id) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let circle_id = create_circle(&app, &token, "OldName").await;

    let update = serde_json::json!({ "name": "NewName" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/circles/{circle_id}"), &update, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["name"], "NewName");
    assert_eq!(resp["id"], circle_id);
    assert_eq!(resp["owner_id"], user_id);
}

#[tokio::test]
async fn update_circle_by_non_owner_returns_403() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "AliceCircle").await;

    let update = serde_json::json!({ "name": "BobName" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/circles/{circle_id}"), &update, &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

// ── Delete ─────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_circle_by_owner_returns_204() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &token, "ToDelete").await;

    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone from the list
    let (_, circles) = app.get_with_auth("/circles", &token).await;
    let arr = circles["data"].as_array().unwrap();
    assert!(
        !arr.iter().any(|c| c["id"].as_str() == Some(&circle_id)),
        "deleted circle should not appear in list"
    );

    // Direct access returns FORBIDDEN (membership cascade-deleted)
    let (status, _) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_circle_by_non_owner_returns_403() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "NoDelete").await;

    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

// ═══════════════════════════════════════════════════════════════════
// Direct Circles
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_direct_circle_returns_201() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user_named(&app, "dc-a@test.com", "dc_alice").await;
    let (bob_token, bob_id) = setup_user_named(&app, "dc-b@test.com", "dc_bob").await;

    // Must be friends first
    make_friends(&app, &alice_token, &bob_token, "dc_bob").await;

    // Accept already creates a direct circle, so creating another returns 409
    let (status, _) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "direct circle auto-created on accept"
    );
}

#[tokio::test]
async fn create_direct_circle_requires_friendship() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (_, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    // Not friends — should be forbidden
    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn create_direct_circle_prevents_duplicate() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user_named(&app, "dcd-a@test.com", "dcd_alice").await;
    let (bob_token, bob_id) = setup_user_named(&app, "dcd-b@test.com", "dcd_bob").await;

    // Become friends (auto-creates direct circle)
    make_friends(&app, &alice_token, &bob_token, "dcd_bob").await;

    // Explicit creation attempt fails (already exists)
    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn create_direct_circle_reverse_also_duplicate() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_user_named(&app, "dcr-a@test.com", "dcr_alice").await;
    let (bob_token, bob_id) = setup_user_named(&app, "dcr-b@test.com", "dcr_bob").await;

    // Become friends (auto-creates circle)
    make_friends(&app, &alice_token, &bob_token, "dcr_bob").await;

    // Both directions should fail (circle already exists from accept)
    let (status, _) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{alice_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn create_direct_circle_with_self_returns_400() {
    let app = TestApp::new().await;
    let (token, user_id) = setup_user_with_id(&app, ALICE_EMAIL).await;

    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{user_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn create_direct_circle_nonexistent_user_returns_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{fake_id}"), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn direct_circle_cannot_be_renamed() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_user_named(&app, "dcn-a@test.com", "dcn_alice").await;
    let (bob_token, bob_id) = setup_user_named(&app, "dcn-b@test.com", "dcn_bob").await;

    make_friends(&app, &alice_token, &bob_token, "dcn_bob").await;

    // Find the auto-created direct circle
    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id AND cm1.user_id = $1::uuid \
         JOIN circle_members cm2 ON cm2.circle_id = c.id AND cm2.user_id = $2::uuid \
         WHERE c.is_direct = true",
    )
    .bind(&alice_id)
    .bind(&bob_id)
    .fetch_one(&app.db)
    .await
    .unwrap();

    let update = serde_json::json!({ "name": "Custom" });
    let (status, resp) = app
        .patch_json_with_auth(&format!("/circles/{circle_id}"), &update, &alice_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_error(&resp, "BAD_REQUEST");
}

#[tokio::test]
async fn direct_circle_shows_other_username_as_name() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user_named(&app, "dcs-a@test.com", "dcs_alice").await;
    let (bob_token, _) = setup_user_named(&app, "dcs-b@test.com", "dcs_bob").await;

    make_friends(&app, &alice_token, &bob_token, "dcs_bob").await;

    // Alice's list: direct circle shows Bob's username as name
    let (status, circles) = app.get_with_auth("/circles", &alice_token).await;
    assert_eq!(status, StatusCode::OK);

    let arr = circles["data"].as_array().unwrap();
    let direct = arr.iter().find(|c| c["is_direct"] == true).unwrap();
    assert_eq!(direct["name"].as_str().unwrap(), "dcs_bob");

    // Bob's list: direct circle shows Alice's username as name
    let (_, bob_circles) = app.get_with_auth("/circles", &bob_token).await;
    let bob_arr = bob_circles["data"].as_array().unwrap();
    let bob_direct = bob_arr.iter().find(|c| c["is_direct"] == true).unwrap();
    assert_eq!(bob_direct["name"].as_str().unwrap(), "dcs_alice");
}

// ═══════════════════════════════════════════════════════════════════
// Invites
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn create_invite_returns_201_with_token() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &token, "InviteCircle").await;

    let invite_body = serde_json::json!({ "max_uses": 5, "expires_in_hours": 48 });
    let (status, resp) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/invite"),
            &invite_body,
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    let inv_token = resp["token"].as_str().unwrap();
    assert_eq!(inv_token.len(), 32, "invite token should be 32 chars");
    assert!(inv_token.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(resp["max_uses"], 5);
    assert_eq!(resp["use_count"], 0);
    assert_eq!(resp["circle_id"], circle_id);
    assert!(resp["id"].is_string());
    assert!(resp["expires_at"].is_string());
    assert!(resp["created_at"].is_string());
}

#[tokio::test]
async fn create_invite_default_params() {
    let app = TestApp::new().await;
    let token = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &token, "DefaultInvite").await;

    // POST with empty body
    let (status, resp) = app
        .post_with_auth(&format!("/circles/{circle_id}/invite"), &token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["max_uses"], 1); // default
    assert_eq!(resp["use_count"], 0);
}

#[tokio::test]
async fn create_invite_non_member_returns_403() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "NoInvite").await;

    let (status, resp) = app
        .post_with_auth(&format!("/circles/{circle_id}/invite"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn join_via_invite_adds_member() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "JoinTest").await;
    let invite_tok = create_invite_token(&app, &circle_id, &alice).await;

    // Bob joins
    let (status, resp) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["circle_id"], circle_id);
    assert_eq!(resp["circle_name"], "JoinTest");

    // Verify Bob is now a member with correct role
    let (status, detail) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);

    let members = detail["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);

    let bob_member = members.iter().find(|m| m["role"] == "member").unwrap();
    assert_eq!(bob_member["role"], "member");
    assert!(bob_member["username"].is_string());
}

#[tokio::test]
async fn join_expired_invite_returns_410() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "ExpireTest").await;
    let invite_tok = create_invite_token(&app, &circle_id, &alice).await;

    // Expire the invite in DB
    sqlx::query(
        "UPDATE circle_invites SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1",
    )
    .bind(&invite_tok)
    .execute(&app.db)
    .await
    .unwrap();

    let (status, resp) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), &bob)
        .await;
    assert_eq!(status, StatusCode::GONE);
    assert_error(&resp, "GONE");
}

#[tokio::test]
async fn join_maxed_invite_returns_410() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;
    let charlie = app.setup_user_token(CHARLIE_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "MaxTest").await;

    // Create invite with max_uses=1
    let invite_body = serde_json::json!({ "max_uses": 1 });
    let (_, invite) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/invite"),
            &invite_body,
            &alice,
        )
        .await;
    let invite_tok = invite["token"].as_str().unwrap();

    // Bob joins (uses the only slot)
    let (status, _) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Charlie tries — should fail
    let (status, resp) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), &charlie)
        .await;
    assert_eq!(status, StatusCode::GONE);
    assert_error(&resp, "GONE");
}

#[tokio::test]
async fn join_invite_already_member_returns_409() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "AlreadyIn").await;

    let invite_body = serde_json::json!({ "max_uses": 5 });
    let (_, invite) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/invite"),
            &invite_body,
            &alice,
        )
        .await;
    let invite_tok = invite["token"].as_str().unwrap();

    // Alice tries to join her own circle via invite
    let (status, resp) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), &alice)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn join_nonexistent_invite_returns_404() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let (status, resp) = app
        .post_with_auth("/circles/join/nonexistent_token_12345678", &alice)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════
// Member management
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn remove_member_by_owner() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "RemoveTest").await;
    invite_and_join(&app, &circle_id, &alice, &bob_token).await;

    // Alice removes Bob
    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}/members/{bob_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob can no longer access
    let (status, resp) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");

    // Circle still has 1 member (owner)
    let (_, detail) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &alice)
        .await;
    assert_eq!(detail["members"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn remove_self_from_circle() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "SelfRemove").await;
    invite_and_join(&app, &circle_id, &alice, &bob_token).await;

    // Bob removes himself
    let (status, _) = app
        .delete_with_auth(
            &format!("/circles/{circle_id}/members/{bob_id}"),
            &bob_token,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob can no longer access
    let (status, _) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn remove_member_by_non_owner_returns_403() {
    let app = TestApp::new().await;
    let (alice_token, alice_id) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;
    let charlie = app.setup_user_token(CHARLIE_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice_token, "NoRemove").await;

    let invite_body = serde_json::json!({ "max_uses": 10 });
    let (_, invite) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/invite"),
            &invite_body,
            &alice_token,
        )
        .await;
    let invite_tok = invite["token"].as_str().unwrap();
    app.post_with_auth(&format!("/circles/join/{invite_tok}"), &bob)
        .await;
    app.post_with_auth(&format!("/circles/join/{invite_tok}"), &charlie)
        .await;

    // Bob tries to remove Alice — should fail (not owner)
    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}/members/{alice_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn remove_nonexistent_member_returns_404() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "NoMember").await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}/members/{fake_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════
// Invite listing & revocation
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_invites_returns_active_only() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "ListInvites").await;

    // Create two invites
    create_invite_token(&app, &circle_id, &alice).await;
    let inv2_tok = create_invite_token(&app, &circle_id, &alice).await;

    // Expire the second one
    sqlx::query(
        "UPDATE circle_invites SET expires_at = NOW() - INTERVAL '1 hour' WHERE token = $1",
    )
    .bind(&inv2_tok)
    .execute(&app.db)
    .await
    .unwrap();

    let (status, resp) = app
        .get_with_auth(&format!("/circles/{circle_id}/invites"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    let arr = resp["data"].as_array().unwrap();
    assert_eq!(arr.len(), 1); // only the non-expired one
    // Verify it's not the expired one
    assert_ne!(arr[0]["token"].as_str().unwrap(), inv2_tok);
}

#[tokio::test]
async fn list_invites_non_member_returns_403() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "NoListInvites").await;

    let (status, resp) = app
        .get_with_auth(&format!("/circles/{circle_id}/invites"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn revoke_invite_by_owner_returns_204() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "RevokeTest").await;

    let (_, invite) = app
        .post_with_auth(&format!("/circles/{circle_id}/invite"), &alice)
        .await;
    let invite_id = invite["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}/invites/{invite_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify invite is gone from listing
    let (_, listing) = app
        .get_with_auth(&format!("/circles/{circle_id}/invites"), &alice)
        .await;
    let arr = listing["data"].as_array().unwrap();
    assert!(
        !arr.iter().any(|inv| inv["id"].as_str() == Some(invite_id)),
        "revoked invite should not appear in listing"
    );
}

#[tokio::test]
async fn revoke_invite_by_non_owner_returns_403() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "NoRevoke").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Create another invite
    let (_, invite) = app
        .post_with_auth(&format!("/circles/{circle_id}/invite"), &alice)
        .await;
    let invite_id = invite["id"].as_str().unwrap();

    // Bob (member, not owner) tries to revoke
    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}/invites/{invite_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn revoke_nonexistent_invite_returns_404() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "NoInvite").await;

    let fake_id = "00000000-0000-0000-0000-000000000099";
    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}/invites/{fake_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

// ═══════════════════════════════════════════════════════════════════
// DEV-46: Items & Feed
// ═══════════════════════════════════════════════════════════════════

// ── Share / Unshare ────────────────────────────────────────────────

#[tokio::test]
async fn share_item_to_circle_returns_204() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "ShareCircle").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Cadeau" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify item appears in circle items list
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    let arr = items["data"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], item_id);
    assert_eq!(arr[0]["name"], "Cadeau");
    assert!(arr[0]["shared_at"].is_string());
    assert!(arr[0]["shared_by"].is_string());
}

#[tokio::test]
async fn share_same_item_twice_is_idempotent() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "IdempotentShare").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Twice" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });

    // First share
    let (s1, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;
    assert_eq!(s1, StatusCode::NO_CONTENT);

    // Second share — idempotent, no duplicate events (ON CONFLICT DO NOTHING)
    let (s2, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;
    assert_eq!(s2, StatusCode::NO_CONTENT);

    // Only 1 item in the list (no duplicates)
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn share_item_requires_membership() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "PrivateCircle").await;

    let item = app
        .create_item(&bob, &serde_json::json!({ "name": "BobItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    let (status, resp) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn share_item_requires_item_ownership() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "OwnerCheck").await;

    // Bob's item — Alice can't share it
    let item = app
        .create_item(&bob, &serde_json::json!({ "name": "BobOnly" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    let (status, resp) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn unshare_item_by_sharer_returns_204() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "UnshareCircle").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "RemoveMe" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Share
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Unshare
    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}/items/{item_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify item is gone from the list
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn unshare_item_not_shared_returns_404() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "NotShared").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "NeverShared" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}/items/{item_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn unshare_by_non_sharer_non_owner_returns_403() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "UnsharePerms").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice shares her item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "AliceItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Bob (member, not owner, not sharer) tries to unshare
    let (status, resp) = app
        .delete_with_auth(&format!("/circles/{circle_id}/items/{item_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

#[tokio::test]
async fn list_empty_circle_items_returns_200() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "EmptyItems").await;

    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items["data"].as_array().unwrap().len(), 0);
}

// ── Anti-spoiler (items) ───────────────────────────────────────────

#[tokio::test]
async fn list_circle_items_with_anti_spoiler() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "AntiSpoiler").await;
    invite_and_join(&app, &circle_id, &alice, &bob_token).await;

    // Alice shares an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Surprise" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Bob claims Alice's item directly in DB
    sqlx::query("UPDATE items SET claimed_by = $1, claimed_at = NOW() WHERE id = $2")
        .bind(uuid::Uuid::parse_str(&bob_id).unwrap())
        .bind(uuid::Uuid::parse_str(item_id).unwrap())
        .execute(&app.db)
        .await
        .unwrap();

    // Alice sees is_claimed=true but claimed_by=null (anti-spoiler)
    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    let arr = items["data"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], item_id);
    assert_eq!(arr[0]["name"], "Surprise");
    assert_eq!(arr[0]["is_claimed"], true);
    assert!(
        arr[0]["claimed_by"].is_null(),
        "item owner must NOT see who claimed"
    );
}

#[tokio::test]
async fn list_circle_items_non_owner_sees_claimed_by() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "SeeClaimCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob_token).await;

    // Alice shares item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Visible" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Bob claims
    sqlx::query("UPDATE items SET claimed_by = $1, claimed_at = NOW() WHERE id = $2")
        .bind(uuid::Uuid::parse_str(&bob_id).unwrap())
        .bind(uuid::Uuid::parse_str(item_id).unwrap())
        .execute(&app.db)
        .await
        .unwrap();

    // Bob sees claimed_by (he's NOT the item owner, so no anti-spoiler)
    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let arr = items["data"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["is_claimed"], true);
    assert!(
        arr[0]["claimed_by"].is_object(),
        "non-owner must see claimed_by"
    );
    assert_eq!(arr[0]["claimed_by"]["user_id"], bob_id);
    assert!(arr[0]["claimed_by"]["username"].is_string());
}

#[tokio::test]
async fn list_circle_items_third_party_sees_claimed_by() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;
    let charlie = app.setup_user_token(CHARLIE_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "ThirdParty").await;

    // Invite both Bob and Charlie
    let invite_body = serde_json::json!({ "max_uses": 10 });
    let (_, invite) = app
        .post_json_with_auth(
            &format!("/circles/{circle_id}/invite"),
            &invite_body,
            &alice,
        )
        .await;
    let invite_tok = invite["token"].as_str().unwrap();
    app.post_with_auth(&format!("/circles/join/{invite_tok}"), &bob_token)
        .await;
    app.post_with_auth(&format!("/circles/join/{invite_tok}"), &charlie)
        .await;

    // Alice shares item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Gift" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Bob claims
    sqlx::query("UPDATE items SET claimed_by = $1, claimed_at = NOW() WHERE id = $2")
        .bind(uuid::Uuid::parse_str(&bob_id).unwrap())
        .bind(uuid::Uuid::parse_str(item_id).unwrap())
        .execute(&app.db)
        .await
        .unwrap();

    // Charlie (third-party, not item owner, not claimer) also sees claimed_by
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &charlie)
        .await;
    let arr = items["data"].as_array().unwrap();
    assert_eq!(arr[0]["is_claimed"], true);
    assert!(
        arr[0]["claimed_by"].is_object(),
        "third-party should see claimed_by"
    );
    assert_eq!(arr[0]["claimed_by"]["user_id"], bob_id);
}

// ── Feed ───────────────────────────────────────────────────────────

#[tokio::test]
async fn get_feed_returns_events() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "FeedCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Share an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "FeedItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    let (status, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["pagination"]["total"].as_i64().unwrap(), 2); // member_joined + item_shared
    assert_eq!(feed["pagination"]["page"], 1);
    assert_eq!(feed["pagination"]["limit"], 20);

    let events = feed["data"].as_array().unwrap();
    assert_eq!(events.len(), 2);

    let event_types: Vec<&str> = events
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();
    assert!(event_types.contains(&"member_joined"));
    assert!(event_types.contains(&"item_shared"));

    // Each event has required fields
    for event in events {
        assert!(event["id"].is_string());
        assert!(event["created_at"].is_string());
    }

    // The item_shared event references the correct item
    let shared_event = events
        .iter()
        .find(|e| e["event_type"] == "item_shared")
        .unwrap();
    assert_eq!(shared_event["target_item_id"], item_id);
    assert_eq!(shared_event["target_item_name"], "FeedItem");
}

#[tokio::test]
async fn feed_empty_circle_returns_empty() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "EmptyFeed").await;

    let (status, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["pagination"]["total"], 0);
    assert_eq!(feed["data"].as_array().unwrap().len(), 0);
    assert_eq!(feed["pagination"]["page"], 1);
    assert_eq!(feed["pagination"]["limit"], 20);
}

#[tokio::test]
async fn feed_pagination() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let circle_id = create_circle(&app, &alice, "PaginatedFeed").await;

    // Create 3 items and share them to generate 3 item_shared events
    for i in 1..=3 {
        let item = app
            .create_item(&alice, &serde_json::json!({ "name": format!("Item{i}") }))
            .await;
        let item_id = item["id"].as_str().unwrap();
        let share_body = serde_json::json!({ "item_id": item_id });
        app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
            .await;
    }

    // Page 1 with limit=2
    let (status, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed?page=1&limit=2"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["pagination"]["total"], 3);
    assert_eq!(feed["pagination"]["page"], 1);
    assert_eq!(feed["pagination"]["limit"], 2);
    assert_eq!(feed["data"].as_array().unwrap().len(), 2);

    // Page 2
    let (_, feed2) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed?page=2&limit=2"), &alice)
        .await;
    assert_eq!(feed2["pagination"]["total"], 3);
    assert_eq!(feed2["pagination"]["page"], 2);
    assert_eq!(feed2["data"].as_array().unwrap().len(), 1);

    // Page beyond total
    let (_, feed3) = app
        .get_with_auth(
            &format!("/circles/{circle_id}/feed?page=10&limit=2"),
            &alice,
        )
        .await;
    assert_eq!(feed3["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn feed_requires_membership() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "PrivateFeed").await;

    let (status, resp) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&resp, "FORBIDDEN");
}

// ── Anti-spoiler (feed) ────────────────────────────────────────────

#[tokio::test]
async fn feed_anti_spoiler_hides_claimer_from_item_owner() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "FeedSpoiler").await;
    invite_and_join(&app, &circle_id, &alice, &bob_token).await;

    // Alice shares item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Secret" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Bob claims Alice's item via the real claim endpoint
    let (claim_status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob_token)
        .await;
    assert_eq!(claim_status, StatusCode::NO_CONTENT);

    // Alice views feed — item_claimed event should be completely hidden from the owner
    let (status, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);

    let events = feed["data"].as_array().unwrap();
    let claim_events: Vec<_> = events
        .iter()
        .filter(|e| e["event_type"] == "item_claimed")
        .collect();
    assert!(
        claim_events.is_empty(),
        "item_claimed events must be fully hidden from item owner"
    );

    // Bob views the same feed — he SHOULD see the actor (he is the claimer, not the owner)
    let (_, bob_feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &bob_token)
        .await;
    let bob_events = bob_feed["data"].as_array().unwrap();
    let bob_claim_event = bob_events
        .iter()
        .find(|e| e["event_type"] == "item_claimed")
        .unwrap();
    assert_eq!(
        bob_claim_event["actor_id"], bob_id,
        "non-owner should see actor_id"
    );
    assert!(bob_claim_event["actor_username"].is_string());
}

// ── Multi-circle item ──────────────────────────────────────────────

#[tokio::test]
async fn item_in_multiple_circles() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;

    let c1_id = create_circle(&app, &alice, "Circle1").await;
    let c2_id = create_circle(&app, &alice, "Circle2").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "SharedItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Share to both circles
    let share_body = serde_json::json!({ "item_id": item_id });
    let (s1, _) = app
        .post_json_with_auth(&format!("/circles/{c1_id}/items"), &share_body, &alice)
        .await;
    let (s2, _) = app
        .post_json_with_auth(&format!("/circles/{c2_id}/items"), &share_body, &alice)
        .await;
    assert_eq!(s1, StatusCode::NO_CONTENT);
    assert_eq!(s2, StatusCode::NO_CONTENT);

    // Both circles list the same item
    let (_, items1) = app
        .get_with_auth(&format!("/circles/{c1_id}/items"), &alice)
        .await;
    let (_, items2) = app
        .get_with_auth(&format!("/circles/{c2_id}/items"), &alice)
        .await;
    let arr1 = items1["data"].as_array().unwrap();
    let arr2 = items2["data"].as_array().unwrap();
    assert_eq!(arr1.len(), 1);
    assert_eq!(arr2.len(), 1);
    assert_eq!(
        arr1[0]["id"], item_id,
        "circle1 should contain the same item"
    );
    assert_eq!(
        arr2[0]["id"], item_id,
        "circle2 should contain the same item"
    );
    assert_eq!(arr1[0]["name"], "SharedItem");
    assert_eq!(arr2[0]["name"], "SharedItem");

    // Unshare from one circle doesn't affect the other
    app.delete_with_auth(&format!("/circles/{c1_id}/items/{item_id}"), &alice)
        .await;
    let (_, items1_after) = app
        .get_with_auth(&format!("/circles/{c1_id}/items"), &alice)
        .await;
    let (_, items2_after) = app
        .get_with_auth(&format!("/circles/{c2_id}/items"), &alice)
        .await;
    assert_eq!(
        items1_after["data"].as_array().unwrap().len(),
        0,
        "unshared from circle1"
    );
    assert_eq!(
        items2_after["data"].as_array().unwrap().len(),
        1,
        "still shared in circle2"
    );
}

// ═══════════════════════════════════════════════════════════════════
// DEV-46: Claim/Unclaim Circle Events
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn claim_creates_item_claimed_event_in_feed() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "ClaimEvents").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice shares an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Wishlist item" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    // Bob claims the item
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Feed should contain item_claimed event
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &bob)
        .await;
    let events = feed["data"].as_array().unwrap();
    let event_types: Vec<&str> = events
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();

    assert!(
        event_types.contains(&"item_claimed"),
        "feed should contain item_claimed event, got: {event_types:?}"
    );

    let claim_event = events
        .iter()
        .find(|e| e["event_type"] == "item_claimed")
        .unwrap();
    assert_eq!(claim_event["target_item_id"], item_id);
    assert_eq!(claim_event["target_item_name"], "Wishlist item");
}

#[tokio::test]
async fn unclaim_creates_item_unclaimed_event_in_feed() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "UnclaimEvents").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice shares an item, Bob claims then unclaims
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Gift idea" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    let (status, _) = app
        .delete_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Feed should contain both item_claimed and item_unclaimed
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &bob)
        .await;
    let events = feed["data"].as_array().unwrap();
    let event_types: Vec<&str> = events
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();

    assert!(
        event_types.contains(&"item_claimed"),
        "feed should contain item_claimed, got: {event_types:?}"
    );
    assert!(
        event_types.contains(&"item_unclaimed"),
        "feed should contain item_unclaimed, got: {event_types:?}"
    );

    let unclaim_event = events
        .iter()
        .find(|e| e["event_type"] == "item_unclaimed")
        .unwrap();
    assert_eq!(unclaim_event["target_item_id"], item_id);
    assert_eq!(unclaim_event["target_item_name"], "Gift idea");
}

#[tokio::test]
async fn claim_item_in_multiple_circles_creates_events_in_all() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    // Create two circles with Bob as member
    let c1_id = create_circle(&app, &alice, "Circle A").await;
    let c2_id = create_circle(&app, &alice, "Circle B").await;
    invite_and_join(&app, &c1_id, &alice, &bob).await;
    invite_and_join(&app, &c2_id, &alice, &bob).await;

    // Alice shares the same item to both circles
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "MultiCircle" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{c1_id}/items"), &share_body, &alice)
        .await;
    app.post_json_with_auth(&format!("/circles/{c2_id}/items"), &share_body, &alice)
        .await;

    // Bob claims the item once
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Both circles should have item_claimed event
    let (_, feed1) = app
        .get_with_auth(&format!("/circles/{c1_id}/feed"), &bob)
        .await;
    let (_, feed2) = app
        .get_with_auth(&format!("/circles/{c2_id}/feed"), &bob)
        .await;

    let has_claim_event = |feed: &serde_json::Value| -> bool {
        feed["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_type"] == "item_claimed")
    };

    assert!(
        has_claim_event(&feed1),
        "circle A feed should have item_claimed event"
    );
    assert!(
        has_claim_event(&feed2),
        "circle B feed should have item_claimed event"
    );
}

#[tokio::test]
async fn claim_item_not_in_any_circle_no_error() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    // Alice creates an item but does NOT share it to any circle
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Private item" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Bob claims the item — should still succeed (no circle events needed)
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "claim should succeed even when item is not in any circle"
    );
}

#[tokio::test]
async fn claim_event_has_correct_target_item_name() {
    let app = TestApp::new().await;
    let alice = app.setup_user_token(ALICE_EMAIL, TEST_PASSWORD).await;
    let bob = app.setup_user_token(BOB_EMAIL, TEST_PASSWORD).await;

    let circle_id = create_circle(&app, &alice, "NameCheck").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    let item = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Casque Sony WH-1000XM5" }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;

    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;

    let (_, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &bob)
        .await;
    let claim_event = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == "item_claimed")
        .expect("should have item_claimed event");

    assert_eq!(claim_event["target_item_name"], "Casque Sony WH-1000XM5");
}

// ═══════════════════════════════════════════════════════════════════
// GET /circles/{id}/items/{iid} — single circle item
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_circle_item_returns_item_for_member() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-gci@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-gci@test.com").await;

    let cid = create_circle(&app, &alice, "Gift Circle").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    // Alice creates and shares an item
    let item = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Headphones", "priority": 3 }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{cid}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob can fetch the single item
    let (status, resp) = app
        .get_with_auth(&format!("/circles/{cid}/items/{item_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["name"], "Headphones");
    assert_eq!(resp["priority"], 3);
}

#[tokio::test]
async fn get_circle_item_404_when_not_shared() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-gci404@test.com").await;

    let cid = create_circle(&app, &alice, "My Circle").await;

    // Alice creates item but does NOT share it
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Secret" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, resp) = app
        .get_with_auth(&format!("/circles/{cid}/items/{item_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_error(&resp, "NOT_FOUND");
}

#[tokio::test]
async fn get_circle_item_403_for_non_member() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-gci403@test.com").await;
    let (charlie, _) = setup_user_with_id(&app, "charlie-gci403@test.com").await;

    let cid = create_circle(&app, &alice, "Private Circle").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Gift" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{cid}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Charlie is NOT a member — should get 403
    let (status, _) = app
        .get_with_auth(&format!("/circles/{cid}/items/{item_id}"), &charlie)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_circle_item_anti_spoiler_hides_claimer_from_owner() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-gcispoiler@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-gcispoiler@test.com").await;

    let cid = create_circle(&app, &alice, "Surprise Circle").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    // Alice creates and shares an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Surprise Gift" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{cid}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob claims the item
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Alice (owner) should NOT see who claimed — claimed_by is null
    let (status, resp) = app
        .get_with_auth(&format!("/circles/{cid}/items/{item_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(resp["is_claimed"].as_bool().unwrap());
    assert!(resp["claimed_by"].is_null(), "owner should not see claimer");

    // Bob (claimer) SHOULD see who claimed
    let (status, resp) = app
        .get_with_auth(&format!("/circles/{cid}/items/{item_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(resp["is_claimed"].as_bool().unwrap());
    assert!(
        !resp["claimed_by"].is_null(),
        "non-owner should see claimer"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Feed anti-spoiler: claim events hidden from item owner
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn feed_hides_claim_events_from_item_owner() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-feedspoil@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-feedspoil@test.com").await;

    let cid = create_circle(&app, &alice, "Feed Spoiler Test").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    // Alice creates and shares an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Secret Gift" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{cid}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob claims Alice's item
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Alice (owner) should NOT see the claim event in the feed
    let (status, feed) = app
        .get_with_auth(&format!("/circles/{cid}/feed"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    let alice_events: Vec<&serde_json::Value> = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "item_claimed")
        .collect();
    assert!(
        alice_events.is_empty(),
        "item owner should NOT see claim events in feed"
    );

    // Bob (claimer, non-owner) SHOULD see the claim event
    let (status, feed) = app
        .get_with_auth(&format!("/circles/{cid}/feed"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    let bob_events: Vec<&serde_json::Value> = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "item_claimed")
        .collect();
    assert!(
        !bob_events.is_empty(),
        "non-owner should see claim events in feed"
    );
}

#[tokio::test]
async fn feed_hides_unclaim_events_from_item_owner() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-feedunclaim@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-feedunclaim@test.com").await;

    let cid = create_circle(&app, &alice, "Unclaim Feed Test").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Unclaim Test" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let (status, _) = app
        .post_json_with_auth(
            &format!("/circles/{cid}/items"),
            &serde_json::json!({ "item_id": item_id }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob claims then unclaims
    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    app.delete_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;

    // Alice should NOT see unclaim events either
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{cid}/feed"), &alice)
        .await;
    let claim_events: Vec<&serde_json::Value> = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "item_claimed" || e["event_type"] == "item_unclaimed")
        .collect();
    assert!(
        claim_events.is_empty(),
        "owner should not see any claim/unclaim events"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Item received lifecycle
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn item_received_creates_feed_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-received@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-received@test.com").await;

    let cid = create_circle(&app, &alice, "Gift Lifecycle").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    // Alice creates, shares, Bob claims
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Birthday Gift" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{cid}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &alice,
    )
    .await;

    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;

    // Alice marks as received
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Feed should have an item_received event visible to Bob
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{cid}/feed"), &bob)
        .await;
    let received_events: Vec<&serde_json::Value> = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "item_received")
        .collect();
    assert!(
        !received_events.is_empty(),
        "feed should contain item_received event"
    );
    assert_eq!(received_events[0]["target_item_name"], "Birthday Gift");
}

#[tokio::test]
async fn purchased_item_still_visible_in_circle_with_status() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-purch@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-purch@test.com").await;

    let cid = create_circle(&app, &alice, "Status Check").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Watch" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{cid}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &alice,
    )
    .await;

    // Mark as purchased
    app.patch_json_with_auth(
        &format!("/items/{item_id}"),
        &serde_json::json!({ "status": "purchased" }),
        &alice,
    )
    .await;

    // Item should still be in circle items with status "purchased"
    let (status, items) = app
        .get_with_auth(&format!("/circles/{cid}/items"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    let circle_items = items["data"].as_array().unwrap();
    let watch = circle_items.iter().find(|i| i["name"] == "Watch");
    assert!(
        watch.is_some(),
        "purchased item should still appear in circle"
    );
    assert_eq!(watch.unwrap()["status"], "purchased");
}

// ═══════════════════════════════════════════════════════════════════
// Additional edge case tests
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn feed_shows_unclaim_event_to_non_owner() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-uncfeed@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-uncfeed@test.com").await;
    let (charlie, _) = setup_user_with_id(&app, "charlie-uncfeed@test.com").await;

    let cid = create_circle(&app, &alice, "Unclaim Feed Visible").await;
    invite_and_join(&app, &cid, &alice, &bob).await;
    invite_and_join(&app, &cid, &alice, &charlie).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Headset" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{cid}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &alice,
    )
    .await;

    // Bob claims then unclaims
    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    app.delete_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;

    // Charlie (non-owner, non-claimer) should see the unclaim event
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{cid}/feed"), &charlie)
        .await;
    let unclaim_events: Vec<&serde_json::Value> = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "item_unclaimed")
        .collect();
    assert!(
        !unclaim_events.is_empty(),
        "non-owner should see unclaim events"
    );
}

#[tokio::test]
async fn mark_received_unclaimed_item_no_received_event_for_claimer() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-selfbuy@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-selfbuy@test.com").await;

    let cid = create_circle(&app, &alice, "Self Buy").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    // Alice creates and shares but nobody claims
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Self Purchase" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{cid}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &alice,
    )
    .await;

    // Alice marks as received (self-purchase, no claimer)
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Feed should still have item_received event (visible to Bob)
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{cid}/feed"), &bob)
        .await;
    let received = feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == "item_received")
        .collect::<Vec<_>>();
    assert!(
        !received.is_empty(),
        "item_received event should exist even for self-purchase"
    );
}

#[tokio::test]
async fn deleted_claimed_item_disappears_from_circle() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-delclaim@test.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-delclaim@test.com").await;

    let cid = create_circle(&app, &alice, "Delete Claimed").await;
    invite_and_join(&app, &cid, &alice, &bob).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "DeleteMe" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{cid}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &alice,
    )
    .await;

    // Bob claims
    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;

    // Verify item is in circle
    let (_, items_before) = app
        .get_with_auth(&format!("/circles/{cid}/items"), &bob)
        .await;
    assert!(
        items_before["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "DeleteMe"),
        "item should be in circle before delete"
    );

    // Alice deletes the item
    let (status, _) = app
        .delete_with_auth(&format!("/items/{item_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Item should disappear from circle
    let (_, items_after) = app
        .get_with_auth(&format!("/circles/{cid}/items"), &bob)
        .await;
    assert!(
        !items_after["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "DeleteMe"),
        "deleted item should disappear from circle"
    );
}

#[tokio::test]
async fn double_mark_received_returns_409() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-double@test.com").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "DoubleRcv" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // First mark
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Second mark — should 409
    let (status, resp) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn unarchive_item_returns_to_active() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-unarch@test.com").await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Unarchive Test" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Mark purchased
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Unarchive back to active
    let (status, resp) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "active" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "active");
}

// ═══════════════════════════════════════════════════════════════════
// Batch Share Items
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn batch_share_returns_204() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "bs1@test.com", "bs_alice").await;
    let circle_id = create_circle(&app, &alice, "BatchCircle").await;

    let item1 = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Item A", "category_id": null }),
        )
        .await;
    let item2 = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Item B", "category_id": null }),
        )
        .await;

    let body = serde_json::json!({
        "item_ids": [item1["id"].as_str().unwrap(), item2["id"].as_str().unwrap()]
    });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn batch_share_adds_all_items_to_circle() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "ba1@test.com", "ba_alice").await;
    let circle_id = create_circle(&app, &alice, "BatchAddCircle").await;

    let item1 = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Gift 1", "category_id": null }),
        )
        .await;
    let item2 = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Gift 2", "category_id": null }),
        )
        .await;
    let item3 = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Gift 3", "category_id": null }),
        )
        .await;

    let body = serde_json::json!({
        "item_ids": [
            item1["id"].as_str().unwrap(),
            item2["id"].as_str().unwrap(),
            item3["id"].as_str().unwrap()
        ]
    });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &alice)
        .await;

    // Verify all 3 items are in the circle
    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        3,
        "all 3 items should be in circle"
    );
}

#[tokio::test]
async fn batch_share_sends_single_notification() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "bn1@test.com", "bn_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "bn2@test.com", "bn_bob").await;

    // Make friends and get direct circle
    make_friends(&app, &alice, &bob, "bn_bob").await;
    let circle_id = create_circle(&app, &alice, "NotifCircle").await;
    let body = serde_json::json!({ "user_id": bob_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/members"), &body, &alice)
        .await;

    // Share 5 items in batch
    let mut item_ids = Vec::new();
    for i in 0..5 {
        let item = app
            .create_item(
                &alice,
                &serde_json::json!({ "name": format!("Item {i}"), "category_id": null }),
            )
            .await;
        item_ids.push(item["id"].as_str().unwrap().to_string());
    }

    let body = serde_json::json!({ "item_ids": item_ids });
    app.post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &alice)
        .await;

    // Wait for async notification
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Bob should have exactly ONE notification of type item_shared (not 5)
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM notifications \
         WHERE user_id = $1 AND type = 'item_shared'",
    )
    .bind(bob_id.parse::<uuid::Uuid>().unwrap())
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(
        count.0, 1,
        "batch share should produce exactly 1 notification, not 1 per item"
    );
}

#[tokio::test]
async fn batch_share_empty_list_returns_204() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "be1@test.com", "be_alice").await;
    let circle_id = create_circle(&app, &alice, "EmptyBatch").await;

    let body = serde_json::json!({ "item_ids": [] });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &alice)
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "empty batch should succeed silently"
    );
}

#[tokio::test]
async fn batch_share_skips_items_not_owned() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "bo1@test.com", "bo_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "bo2@test.com", "bo_bob").await;

    make_friends(&app, &alice, &bob, "bo_bob").await;
    let circle_id = create_circle(&app, &alice, "OwnerCheck").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Alice creates 1 item, Bob creates 1 item
    let alice_item = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Alice Gift", "category_id": null }),
        )
        .await;
    let bob_item = app
        .create_item(
            &bob,
            &serde_json::json!({ "name": "Bob Gift", "category_id": null }),
        )
        .await;

    // Alice tries to batch-share both (but she doesn't own Bob's item)
    let body = serde_json::json!({
        "item_ids": [alice_item["id"].as_str().unwrap(), bob_item["id"].as_str().unwrap()]
    });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &alice)
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "should succeed silently, skipping non-owned"
    );

    // Only Alice's item should be in the circle
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "only owned items should be shared"
    );
}

#[tokio::test]
async fn batch_share_idempotent() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "bi1@test.com", "bi_alice").await;
    let circle_id = create_circle(&app, &alice, "IdempotentBatch").await;

    let item = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Repeat Item", "category_id": null }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Share same item twice in batch
    let body = serde_json::json!({ "item_ids": [item_id, item_id] });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Item should appear only once
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1, "no duplicates");
}

#[tokio::test]
async fn batch_share_requires_membership() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "bm1@test.com", "bm_alice").await;
    let (bob, _) = setup_user_named(&app, "bm2@test.com", "bm_bob").await;

    let circle_id = create_circle(&app, &alice, "PrivateCircle").await;
    let item = app
        .create_item(
            &bob,
            &serde_json::json!({ "name": "Bob Item", "category_id": null }),
        )
        .await;

    // Bob is NOT a member — should fail
    let body = serde_json::json!({ "item_ids": [item["id"].as_str().unwrap()] });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items/batch"), &body, &bob)
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "non-member cannot batch share"
    );
}

#[tokio::test]
async fn batch_share_no_auth_returns_401() {
    let app = TestApp::new().await;
    let body = serde_json::json!({ "item_ids": [] });
    let (status, _) = app
        .post_json(
            &format!("/circles/{}/items/batch", uuid::Uuid::new_v4()),
            &body,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ═══════════════════════════════════════════════════════════════════
// Leave Circle — Items Cleanup
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn leaving_circle_removes_members_shared_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "lci1@test.com", "lci_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "lci2@test.com", "lci_bob").await;
    make_friends(&app, &alice, &bob, "lci_bob").await;
    let circle_id = create_circle(&app, &alice, "LeaveItemsCircle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;
    let item = app
        .create_item(
            &bob,
            &serde_json::json!({ "name": "Bob Gift", "category_id": null }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &bob,
    )
    .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "precondition: item in circle"
    );
    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}/members/{bob_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "departing member's items removed"
    );
}

#[tokio::test]
async fn leaving_circle_keeps_other_members_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "lco1@test.com", "lco_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "lco2@test.com", "lco_bob").await;
    make_friends(&app, &alice, &bob, "lco_bob").await;
    let circle_id = create_circle(&app, &alice, "KeepOthersCircle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;
    let a_item = app
        .create_item(
            &alice,
            &serde_json::json!({ "name": "Alice Gift", "category_id": null }),
        )
        .await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": a_item["id"].as_str().unwrap() }),
        &alice,
    )
    .await;
    let b_item = app
        .create_item(
            &bob,
            &serde_json::json!({ "name": "Bob Gift", "category_id": null }),
        )
        .await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": b_item["id"].as_str().unwrap() }),
        &bob,
    )
    .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 2);
    app.delete_with_auth(&format!("/circles/{circle_id}/members/{bob_id}"), &bob)
        .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "only remaining member's items stay"
    );
}

#[tokio::test]
async fn owner_removing_member_cleans_their_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "orm1@test.com", "orm_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "orm2@test.com", "orm_bob").await;
    make_friends(&app, &alice, &bob, "orm_bob").await;
    let circle_id = create_circle(&app, &alice, "OwnerRemoveCircle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;
    let item = app
        .create_item(
            &bob,
            &serde_json::json!({ "name": "Bob Item", "category_id": null }),
        )
        .await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item["id"].as_str().unwrap() }),
        &bob,
    )
    .await;
    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}/members/{bob_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "removed member's items cleaned"
    );
}

#[tokio::test]
async fn leaving_circle_item_still_in_wishlist() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "lie1@test.com", "lie_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "lie2@test.com", "lie_bob").await;
    make_friends(&app, &alice, &bob, "lie_bob").await;
    let circle_id = create_circle(&app, &alice, "SurvivesCircle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;
    let item = app
        .create_item(
            &bob,
            &serde_json::json!({ "name": "Survives", "category_id": null }),
        )
        .await;
    let item_id = item["id"].as_str().unwrap();
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &bob,
    )
    .await;
    app.delete_with_auth(&format!("/circles/{circle_id}/members/{bob_id}"), &bob)
        .await;
    let (status, body) = app.get_with_auth(&format!("/items/{item_id}"), &bob).await;
    assert_eq!(status, StatusCode::OK, "item still exists for owner");
    assert_eq!(body["name"], "Survives");
}

// ═══════════════════════════════════════════════════════════════════
// Share Rules — Dynamic Sharing for Direct Circles
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn share_rule_all_makes_items_visible() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "sra1@test.com", "sra_alice").await;
    let (bob, _bob_id) = setup_user_named(&app, "sra2@test.com", "sra_bob").await;
    make_friends(&app, &alice, &bob, "sra_bob").await;

    // Find direct circle
    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'sra_alice') \
         LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Alice creates items
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Gift A", "category_id": null }),
    )
    .await;
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Gift B", "category_id": null }),
    )
    .await;

    // No share rule → Bob sees nothing
    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "no items before rule"
    );

    // Alice sets share rule to "all"
    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob now sees both items
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        2,
        "all items visible after rule"
    );
}

#[tokio::test]
async fn share_rule_all_excludes_private_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "srp1@test.com", "srp_alice").await;
    let (bob, _bob_id) = setup_user_named(&app, "srp2@test.com", "srp_bob").await;
    make_friends(&app, &alice, &bob, "srp_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'srp_alice') \
         LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Alice creates a public and a private item
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Public", "category_id": null }),
    )
    .await;
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Private", "category_id": null, "is_private": true }),
    )
    .await;

    // Share all
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Bob sees only public item
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "private item excluded"
    );
    assert_eq!(items["data"][0]["name"], "Public");
}

#[tokio::test]
async fn share_rule_all_dynamic_new_items_appear() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "srd1@test.com", "srd_alice").await;
    let (bob, _bob_id) = setup_user_named(&app, "srd2@test.com", "srd_bob").await;
    make_friends(&app, &alice, &bob, "srd_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'srd_alice') \
         LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Share all first
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Create item AFTER setting rule
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "New Gift", "category_id": null }),
    )
    .await;

    // Bob sees it immediately (dynamic!)
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "new item appears dynamically"
    );
    assert_eq!(items["data"][0]["name"], "New Gift");
}

#[tokio::test]
async fn share_rule_none_hides_everything() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "srn1@test.com", "srn_alice").await;
    let (bob, _bob_id) = setup_user_named(&app, "srn2@test.com", "srn_bob").await;
    make_friends(&app, &alice, &bob, "srn_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'srn_alice') \
         LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Gift", "category_id": null }),
    )
    .await;

    // Set to all then back to none
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);

    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "none" }),
        &alice,
    )
    .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "none mode hides everything"
    );
}

#[tokio::test]
async fn get_share_rule_default_none() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gsr1@test.com", "gsr_alice").await;
    let (bob, _) = setup_user_named(&app, "gsr2@test.com", "gsr_bob").await;
    make_friends(&app, &alice, &bob, "gsr_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'gsr_alice') \
         LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    let (status, body) = app
        .get_with_auth(&format!("/circles/{circle_id}/share-rule"), &alice)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["share_mode"], "none");
}

#[tokio::test]
async fn share_rule_categories_filters_correctly() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "src1@test.com", "src_alice").await;
    let (bob, _) = setup_user_named(&app, "src2@test.com", "src_bob").await;
    make_friends(&app, &alice, &bob, "src_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'src_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Get a global category (Tech)
    let (_, cats) = app.get_with_auth("/categories", &alice).await;
    let cat_id = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"].as_str().unwrap() == "Tech")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create items: one in Tech, one without category
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Laptop", "category_id": cat_id }),
    )
    .await;
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Random", "category_id": null }),
    )
    .await;

    // Share only Tech category
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "categories", "category_ids": [cat_id] }),
        &alice,
    )
    .await;

    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "only Tech item visible"
    );
    assert_eq!(items["data"][0]["name"], "Laptop");
}

#[tokio::test]
async fn share_rule_categories_empty_ids_returns_nothing() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "sce1@test.com", "sce_alice").await;
    let (bob, _) = setup_user_named(&app, "sce2@test.com", "sce_bob").await;
    make_friends(&app, &alice, &bob, "sce_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'sce_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Gift", "category_id": null }),
    )
    .await;

    // Categories mode with empty array
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "categories", "category_ids": [] }),
        &alice,
    )
    .await;

    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "empty categories = no items"
    );
}

#[tokio::test]
async fn share_rule_invalid_mode_rejected() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "sri1@test.com", "sri_alice").await;
    let (bob, _) = setup_user_named(&app, "sri2@test.com", "sri_bob").await;
    make_friends(&app, &alice, &bob, "sri_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'sri_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "invalid_mode" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "invalid mode rejected");
}

#[tokio::test]
async fn share_rule_non_member_forbidden() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "srf1@test.com", "srf_alice").await;
    let (bob, _) = setup_user_named(&app, "srf2@test.com", "srf_bob").await;
    let (charlie, _) = setup_user_named(&app, "srf3@test.com", "srf_charlie").await;
    make_friends(&app, &alice, &bob, "srf_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'srf_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Charlie is not a member
    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
            &charlie,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "non-member cannot set rule");

    let (status, _) = app
        .get_with_auth(&format!("/circles/{circle_id}/share-rule"), &charlie)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "non-member cannot get rule");
}

#[tokio::test]
async fn share_rule_bidirectional_independent() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "srb1@test.com", "srb_alice").await;
    let (bob, _) = setup_user_named(&app, "srb2@test.com", "srb_bob").await;
    make_friends(&app, &alice, &bob, "srb_bob").await;

    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'srb_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Alice shares all, Bob shares nothing
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Alice Gift", "category_id": null }),
    )
    .await;
    app.create_item(
        &bob,
        &serde_json::json!({ "name": "Bob Gift", "category_id": null }),
    )
    .await;

    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;
    // Bob doesn't set a rule (defaults to none)

    // Bob sees Alice's items
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);
    assert_eq!(items["data"][0]["name"], "Alice Gift");

    // Alice sees nothing from Bob (Bob has no rule)
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &alice)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "Alice sees her own shared items + nothing from Bob"
    );
}

#[tokio::test]
async fn set_share_rule_no_auth_401() {
    let app = TestApp::new().await;
    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = app
        .post_json(
            &format!("/circles/{fake_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
        )
        .await;
    // PUT is not POST, but test unauthorized access
    assert!(status == StatusCode::UNAUTHORIZED || status == StatusCode::METHOD_NOT_ALLOWED);
}

// ── Share rules on GROUP circles ─────────────────────────────────────

#[tokio::test]
async fn share_rule_group_circle_all_works() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "grp_a@test.com", "grp_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "grp_b@test.com", "grp_bob").await;
    let (carol, carol_id) = setup_user_named(&app, "grp_c@test.com", "grp_carol").await;

    // Alice creates a group circle
    make_friends(&app, &alice, &bob, "grp_bob").await;
    make_friends(&app, &alice, &carol, "grp_carol").await;
    let circle_id = create_circle(&app, &alice, "Group Share Test").await;
    let body = serde_json::json!({ "user_id": bob_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/members"), &body, &alice)
        .await;
    let body = serde_json::json!({ "user_id": carol_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/members"), &body, &alice)
        .await;

    // Alice creates items
    app.create_item(&alice, &serde_json::json!({ "name": "Alice Item 1" }))
        .await;
    app.create_item(&alice, &serde_json::json!({ "name": "Alice Item 2" }))
        .await;

    // Alice sets share rule "all" on the group
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Bob should see Alice's items
    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(items["data"].as_array().unwrap().len(), 2);

    // Carol should also see them
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &carol)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn share_rule_group_backward_compat() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "bkc_a@test.com", "bkc_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "bkc_b@test.com", "bkc_bob").await;

    make_friends(&app, &alice, &bob, "bkc_bob").await;
    let circle_id = create_circle(&app, &alice, "Backward Compat Circle").await;
    let body = serde_json::json!({ "user_id": bob_id });
    app.post_json_with_auth(&format!("/circles/{circle_id}/members"), &body, &alice)
        .await;

    // Alice shares items via batch (no share rule set — old way)
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Legacy Item" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items/batch"),
        &serde_json::json!({ "item_ids": [item_id] }),
        &alice,
    )
    .await;

    // Bob should still see the item (backward compat fallback)
    let (status, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "legacy circle_items should still be visible without share rule"
    );
    assert_eq!(items["data"][0]["name"], "Legacy Item");
}

#[tokio::test]
async fn share_rule_group_mixed_rules() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "mix_a@test.com", "mix_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "mix_b@test.com", "mix_bob").await;
    let (carol, carol_id) = setup_user_named(&app, "mix_c@test.com", "mix_carol").await;

    make_friends(&app, &alice, &bob, "mix_bob").await;
    make_friends(&app, &alice, &carol, "mix_carol").await;
    make_friends(&app, &bob, &carol, "mix_carol").await;
    let circle_id = create_circle(&app, &alice, "Mixed Rules Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": carol_id }),
        &alice,
    )
    .await;

    // Alice: share rule "all" (2 items)
    app.create_item(&alice, &serde_json::json!({ "name": "Alice All 1" }))
        .await;
    app.create_item(&alice, &serde_json::json!({ "name": "Alice All 2" }))
        .await;
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Bob: legacy circle_items (1 item, no rule)
    let bob_item = app
        .create_item(&bob, &serde_json::json!({ "name": "Bob Legacy" }))
        .await;
    let bob_item_id = bob_item["id"].as_str().unwrap();
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items/batch"),
        &serde_json::json!({ "item_ids": [bob_item_id] }),
        &bob,
    )
    .await;

    // Carol: no rule, no items → contributes nothing

    // Alice sees Bob's legacy item + Carol's nothing = 1
    // (Alice doesn't see her own items)
    // Actually Alice sees all members' items including her own
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &carol)
        .await;
    let names: Vec<&str> = items["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"Alice All 1"),
        "Alice's items via rule 'all'"
    );
    assert!(
        names.contains(&"Alice All 2"),
        "Alice's items via rule 'all'"
    );
    assert!(
        names.contains(&"Bob Legacy"),
        "Bob's items via legacy circle_items"
    );
    assert_eq!(names.len(), 3, "total: 2 from Alice + 1 from Bob");
}

#[tokio::test]
async fn share_rule_group_set_rule_clears_circle_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "clr_a@test.com", "clr_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "clr_b@test.com", "clr_bob").await;

    make_friends(&app, &alice, &bob, "clr_bob").await;
    let circle_id = create_circle(&app, &alice, "Clear Items Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Alice shares an item via batch (legacy)
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "To Be Cleared" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items/batch"),
        &serde_json::json!({ "item_ids": [item_id] }),
        &alice,
    )
    .await;

    // Now Alice switches to "all" mode — circle_items should be cleaned up
    let (status, _) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob still sees Alice's items (now via dynamic "all" rule, not circle_items)
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert!(
        !items["data"].as_array().unwrap().is_empty(),
        "items should be visible via 'all' rule"
    );
}

// ── My share rules endpoint ──────────────────────────────────────────

#[tokio::test]
async fn list_my_share_rules_returns_active_rules() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "msr_a@test.com", "msr_alice").await;
    let (bob, _) = setup_user_named(&app, "msr_b@test.com", "msr_bob").await;

    make_friends(&app, &alice, &bob, "msr_bob").await;

    // Get Alice's direct circle with Bob
    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'msr_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Set a share rule
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Fetch my-share-rules
    let (status, body) = app.get_with_auth("/circles/my-share-rules", &alice).await;
    assert_eq!(status, StatusCode::OK);
    let rules = body["data"].as_array().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["share_mode"], "all");
    assert_eq!(rules[0]["circle_id"], circle_id);
}

#[tokio::test]
async fn list_my_share_rules_empty_when_no_rules() {
    let app = TestApp::new().await;
    let (token, _) = setup_user_named(&app, "msr_empty@test.com", "msr_empty").await;

    let (status, body) = app.get_with_auth("/circles/my-share-rules", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn list_my_share_rules_no_auth_401() {
    let app = TestApp::new().await;
    let (status, _) = app.get_no_auth("/circles/my-share-rules").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_my_share_rules_does_not_leak_other_users() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "leak_a@test.com", "leak_alice").await;
    let (bob, _) = setup_user_named(&app, "leak_b@test.com", "leak_bob").await;

    make_friends(&app, &alice, &bob, "leak_bob").await;

    // Get Alice's direct circle
    let circle_id: String = sqlx::query_scalar(
        "SELECT c.id::text FROM circles c \
         JOIN circle_members cm1 ON cm1.circle_id = c.id \
         JOIN circle_members cm2 ON cm2.circle_id = c.id \
         WHERE c.is_direct = true AND cm1.user_id != cm2.user_id \
         AND cm1.user_id = (SELECT id FROM users WHERE username = 'leak_alice') LIMIT 1",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();

    // Alice sets a rule
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Bob should NOT see Alice's rule in his my-share-rules
    let (_, body) = app.get_with_auth("/circles/my-share-rules", &bob).await;
    assert_eq!(
        body["data"].as_array().unwrap().len(),
        0,
        "Bob should not see Alice's share rules"
    );
}

// ── Group circle: categories mode ────────────────────────────────────

#[tokio::test]
async fn share_rule_group_categories_works() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gcat_a@test.com", "gcat_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "gcat_b@test.com", "gcat_bob").await;

    make_friends(&app, &alice, &bob, "gcat_bob").await;
    let circle_id = create_circle(&app, &alice, "Group Cat Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Get a category
    let (_, cats) = app.get_with_auth("/categories", &alice).await;
    let tech_id = cats
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"].as_str().unwrap() == "Tech")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Alice creates items: one Tech, one without category
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "MacBook", "category_id": tech_id }),
    )
    .await;
    app.create_item(&alice, &serde_json::json!({ "name": "Random Thing" }))
        .await;

    // Set categories mode with Tech only
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "categories", "category_ids": [tech_id] }),
        &alice,
    )
    .await;

    // Bob should see only the Tech item
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);
    assert_eq!(items["data"][0]["name"], "MacBook");
}

// ── Group circle: selection mode ─────────────────────────────────────

#[tokio::test]
async fn share_rule_group_selection_works() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gsel_a@test.com", "gsel_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "gsel_b@test.com", "gsel_bob").await;

    make_friends(&app, &alice, &bob, "gsel_bob").await;
    let circle_id = create_circle(&app, &alice, "Group Sel Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Alice creates 2 items
    let item1 = app
        .create_item(&alice, &serde_json::json!({ "name": "Sel Item 1" }))
        .await;
    let item1_id = item1["id"].as_str().unwrap();
    app.create_item(&alice, &serde_json::json!({ "name": "Sel Item 2" }))
        .await;

    // Set selection mode and share only item1
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "selection" }),
        &alice,
    )
    .await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items/batch"),
        &serde_json::json!({ "item_ids": [item1_id] }),
        &alice,
    )
    .await;

    // Bob sees only item1
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);
    assert_eq!(items["data"][0]["name"], "Sel Item 1");
}

// ── Group circle: none mode hides everything ─────────────────────────

#[tokio::test]
async fn share_rule_group_none_hides_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gnon_a@test.com", "gnon_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "gnon_b@test.com", "gnon_bob").await;

    make_friends(&app, &alice, &bob, "gnon_bob").await;
    let circle_id = create_circle(&app, &alice, "Group None Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    app.create_item(&alice, &serde_json::json!({ "name": "Hidden Item" }))
        .await;

    // Set "all" first so items are visible
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert!(
        !items["data"].as_array().unwrap().is_empty(),
        "precondition: visible"
    );

    // Switch to "none"
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "none" }),
        &alice,
    )
    .await;
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "none mode should hide all items"
    );
}

// ── Group circle: private items excluded in all mode ─────────────────

#[tokio::test]
async fn share_rule_group_all_excludes_private() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gpriv_a@test.com", "gpriv_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "gpriv_b@test.com", "gpriv_bob").await;

    make_friends(&app, &alice, &bob, "gpriv_bob").await;
    let circle_id = create_circle(&app, &alice, "Group Private Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Alice creates a public and a private item
    app.create_item(&alice, &serde_json::json!({ "name": "Public Item" }))
        .await;
    app.create_item(
        &alice,
        &serde_json::json!({ "name": "Secret Item", "is_private": true }),
    )
    .await;

    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    let names: Vec<&str> = items["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Public Item"));
    assert!(
        !names.contains(&"Secret Item"),
        "private items must be excluded"
    );
}

// ── Group circle: non-member cannot set rule ─────────────────────────

#[tokio::test]
async fn share_rule_group_non_member_forbidden() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gnm_a@test.com", "gnm_alice").await;
    let (carol, _) = setup_user_named(&app, "gnm_c@test.com", "gnm_carol").await;

    let circle_id = create_circle(&app, &alice, "Group NM Circle").await;

    // Carol (not a member) tries to set a rule
    let (status, body) = app
        .put_json_with_auth(
            &format!("/circles/{circle_id}/share-rule"),
            &serde_json::json!({ "share_mode": "all" }),
            &carol,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_error(&body, "FORBIDDEN");
}

// ── Group circle: dynamic — new item appears automatically ───────────

#[tokio::test]
async fn share_rule_group_all_dynamic() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gdyn_a@test.com", "gdyn_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "gdyn_b@test.com", "gdyn_bob").await;

    make_friends(&app, &alice, &bob, "gdyn_bob").await;
    let circle_id = create_circle(&app, &alice, "Group Dynamic Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Set rule before creating items
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    // Create item AFTER setting rule
    app.create_item(&alice, &serde_json::json!({ "name": "Late Addition" }))
        .await;

    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);
    assert_eq!(items["data"][0]["name"], "Late Addition");
}

// ── Backward compat: batch share then set rule transitions cleanly ───

#[tokio::test]
async fn share_rule_group_transition_from_legacy_to_all() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "gtrans_a@test.com", "gtrans_alice").await;
    let (bob, bob_id) = setup_user_named(&app, "gtrans_b@test.com", "gtrans_bob").await;

    make_friends(&app, &alice, &bob, "gtrans_bob").await;
    let circle_id = create_circle(&app, &alice, "Group Transition Circle").await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/members"),
        &serde_json::json!({ "user_id": bob_id }),
        &alice,
    )
    .await;

    // Alice creates 2 items, shares only 1 via legacy batch
    let item1 = app
        .create_item(&alice, &serde_json::json!({ "name": "Legacy Shared" }))
        .await;
    let item1_id = item1["id"].as_str().unwrap();
    app.create_item(&alice, &serde_json::json!({ "name": "Not Yet Shared" }))
        .await;

    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items/batch"),
        &serde_json::json!({ "item_ids": [item1_id] }),
        &alice,
    )
    .await;

    // Bob sees 1 item (legacy)
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);

    // Alice switches to "all" mode — should now see both
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &serde_json::json!({ "share_mode": "all" }),
        &alice,
    )
    .await;

    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        2,
        "switching to 'all' should reveal all items"
    );
}

// ═══════════════════════════════════════════════════════════════════
// Private Items — circle_items cleanup and filtering
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn make_private_removes_circle_items() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "priv-rm1@test.com", "priv_rm_alice").await;
    let (bob, _) = setup_user_named(&app, "priv-rm2@test.com", "priv_rm_bob").await;
    make_friends(&app, &alice, &bob, "priv_rm_bob").await;

    let circle_id = create_circle(&app, &alice, "PrivRmCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice creates an item and shares it with the circle
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "SharedItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    let share_body = serde_json::json!({ "item_id": item_id });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &share_body, &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob can see the item
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        1,
        "item visible before private"
    );

    // Alice sets the item to private
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "is_private": true }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Bob should no longer see the item
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "circle_items row should be deleted when item is set to private"
    );
}

#[tokio::test]
async fn make_private_selection_mode_excludes() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "priv-sel1@test.com", "priv_sel_alice").await;
    let (bob, _) = setup_user_named(&app, "priv-sel2@test.com", "priv_sel_bob").await;
    make_friends(&app, &alice, &bob, "priv_sel_bob").await;

    let circle_id = create_circle(&app, &alice, "PrivSelCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice creates two items
    let item1 = app
        .create_item(&alice, &serde_json::json!({ "name": "KeepThis" }))
        .await;
    let item1_id = item1["id"].as_str().unwrap();
    let item2 = app
        .create_item(&alice, &serde_json::json!({ "name": "HideThis" }))
        .await;
    let item2_id = item2["id"].as_str().unwrap();

    // Share both items manually (selection mode via circle_items)
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item1_id }),
        &alice,
    )
    .await;
    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item2_id }),
        &alice,
    )
    .await;

    // Bob sees 2 items
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        2,
        "both items visible"
    );

    // Alice makes item2 private
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item2_id}"),
            &serde_json::json!({ "is_private": true }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Bob sees only 1 item — the private one is excluded
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    let items_arr = items["data"].as_array().unwrap();
    assert_eq!(
        items_arr.len(),
        1,
        "private item excluded from selection mode"
    );
    assert_eq!(items_arr[0]["name"], "KeepThis");
}

#[tokio::test]
async fn make_public_again_does_not_restore_shares() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "priv-pub1@test.com", "priv_pub_alice").await;
    let (bob, _) = setup_user_named(&app, "priv-pub2@test.com", "priv_pub_bob").await;
    make_friends(&app, &alice, &bob, "priv_pub_bob").await;

    let circle_id = create_circle(&app, &alice, "PrivPubCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice creates and shares an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Vanishes" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    app.post_json_with_auth(
        &format!("/circles/{circle_id}/items"),
        &serde_json::json!({ "item_id": item_id }),
        &alice,
    )
    .await;

    // Confirm Bob sees the item
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(items["data"].as_array().unwrap().len(), 1);

    // Alice makes the item private (deletes circle_items)
    app.patch_json_with_auth(
        &format!("/items/{item_id}"),
        &serde_json::json!({ "is_private": true }),
        &alice,
    )
    .await;

    // Alice makes the item public again
    app.patch_json_with_auth(
        &format!("/items/{item_id}"),
        &serde_json::json!({ "is_private": false }),
        &alice,
    )
    .await;

    // Bob should still NOT see the item — shares were deleted, not restored
    let (_, items) = app
        .get_with_auth(&format!("/circles/{circle_id}/items"), &bob)
        .await;
    assert_eq!(
        items["data"].as_array().unwrap().len(),
        0,
        "making public again should NOT auto-restore circle_items shares"
    );
}

#[tokio::test]
async fn make_private_item_not_in_shared_view() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_named(&app, "priv-sv1@test.com", "priv_sv_alice").await;

    // Alice creates a public item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Visible" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Create a share link (scope=all)
    let (status, link_body) = app.post_with_auth("/share-links", &alice).await;
    assert_eq!(status, StatusCode::CREATED);
    let share_token = link_body["token"].as_str().unwrap();

    // Shared view should show the item
    let (status, view) = app.get_no_auth(&format!("/shared/{share_token}")).await;
    assert_eq!(status, StatusCode::OK);
    let items = &view["items"];
    assert!(
        items
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "Visible"),
        "public item should appear in shared view"
    );

    // Alice makes the item private
    app.patch_json_with_auth(
        &format!("/items/{item_id}"),
        &serde_json::json!({ "is_private": true }),
        &alice,
    )
    .await;

    // Shared view should no longer show the item
    let (_, view) = app.get_no_auth(&format!("/shared/{share_token}")).await;
    let items = &view["items"];
    assert!(
        !items
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["name"] == "Visible"),
        "private item should NOT appear in shared view"
    );
}

// ── Transfer Ownership ──────────────────────────────────────────────

#[tokio::test]
async fn transfer_ownership_success() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-xfer@example.com").await;
    let (bob, bob_id) = setup_user_with_id(&app, "bob-xfer@example.com").await;

    let circle_id = create_circle(&app, &alice, "Team").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Transfer ownership to Bob
    let body = serde_json::json!({ "user_id": bob_id });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/transfer"), &body, &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify Bob is now the owner
    let (_, detail) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &bob)
        .await;
    assert_eq!(detail["owner_id"].as_str().unwrap(), bob_id);
}

#[tokio::test]
async fn transfer_ownership_non_owner_forbidden() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-xfer2@example.com").await;
    let (bob, bob_id) = setup_user_with_id(&app, "bob-xfer2@example.com").await;

    let circle_id = create_circle(&app, &alice, "Team2").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Bob (non-owner) tries to transfer
    let body = serde_json::json!({ "user_id": bob_id });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/transfer"), &body, &bob)
        .await;
    assert!(
        status == StatusCode::FORBIDDEN || status == StatusCode::BAD_REQUEST,
        "non-owner transfer should be rejected, got {status}"
    );
}

#[tokio::test]
async fn transfer_ownership_to_non_member_rejected() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-xfer3@example.com").await;
    let (_, charlie_id) = setup_user_with_id(&app, "charlie-xfer3@example.com").await;

    let circle_id = create_circle(&app, &alice, "Solo").await;

    let body = serde_json::json!({ "user_id": charlie_id });
    let (status, _) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/transfer"), &body, &alice)
        .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND,
        "transfer to non-member should fail, got {status}"
    );
}

// ── Share Rule Events in Feed ───────────────────────────────────────

#[tokio::test]
async fn set_share_rule_creates_feed_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-srfeed@example.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-srfeed@example.com").await;

    let circle_id = create_circle(&app, &alice, "FeedTest").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Set share rule
    let rule = serde_json::json!({ "share_mode": "all", "category_ids": [] });
    app.put_json_with_auth(&format!("/circles/{circle_id}/share-rule"), &rule, &alice)
        .await;

    // Check feed for the event
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &alice)
        .await;

    let events = feed["data"].as_array().unwrap();
    let has_rule_event = events
        .iter()
        .any(|e| e["event_type"].as_str() == Some("share_rule_set"));
    assert!(
        has_rule_event,
        "feed should contain a share_rule_set event after setting a share rule"
    );
}

#[tokio::test]
async fn remove_share_rule_creates_feed_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-srrm@example.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-srrm@example.com").await;

    let circle_id = create_circle(&app, &alice, "RuleRm").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Set then remove share rule
    let set_rule = serde_json::json!({ "share_mode": "all", "category_ids": [] });
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &set_rule,
        &alice,
    )
    .await;
    let rm_rule = serde_json::json!({ "share_mode": "selection", "category_ids": [] });
    app.put_json_with_auth(
        &format!("/circles/{circle_id}/share-rule"),
        &rm_rule,
        &alice,
    )
    .await;

    // Check feed
    let (_, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), &alice)
        .await;

    let events = feed["data"].as_array().unwrap();
    let has_removed_event = events
        .iter()
        .any(|e| e["event_type"].as_str() == Some("share_rule_removed"));
    assert!(
        has_removed_event,
        "feed should contain a share_rule_removed event"
    );
}

// ── Delete Circle ───────────────────────────────────────────────────

#[tokio::test]
async fn delete_circle_as_owner() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-delc@example.com").await;

    let circle_id = create_circle(&app, &alice, "ToDelete").await;

    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone (backend returns 403 or 404 depending on implementation)
    let (status, _) = app
        .get_with_auth(&format!("/circles/{circle_id}"), &alice)
        .await;
    assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::FORBIDDEN,
        "deleted circle should not be accessible, got {status}"
    );
}

#[tokio::test]
async fn delete_circle_non_owner_forbidden() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, "alice-delc2@example.com").await;
    let (bob, _) = setup_user_with_id(&app, "bob-delc2@example.com").await;

    let circle_id = create_circle(&app, &alice, "NoDelete").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Bob (non-owner) tries to delete
    let (status, _) = app
        .delete_with_auth(&format!("/circles/{circle_id}"), &bob)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
