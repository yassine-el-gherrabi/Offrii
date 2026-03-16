mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp, assert_error};

// ── Helpers ─────────────────────────────────────────────────────────

const ALICE_EMAIL: &str = "alice-circle@example.com";
const BOB_EMAIL: &str = "bob-circle@example.com";
const CHARLIE_EMAIL: &str = "charlie-circle@example.com";

/// Register a user, return (access_token, user_id).
async fn setup_user_with_id(app: &TestApp, email: &str) -> (String, String) {
    let body = app.setup_user(email, TEST_PASSWORD).await;
    let token = body["tokens"]["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
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

    let arr = resp.as_array().unwrap();
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
    assert_eq!(resp.as_array().unwrap().len(), 0);
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
    let arr = circles.as_array().unwrap();
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
    let (alice_token, _alice_id) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (_bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["is_direct"], true);
    assert_eq!(resp["member_count"], 2);
    // Direct circles have null name in response (computed per-viewer in list)
    assert!(resp["name"].is_null());
}

#[tokio::test]
async fn create_direct_circle_prevents_duplicate() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (_, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    // First time succeeds
    let (status, _) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Second attempt fails
    let (status, resp) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_error(&resp, "CONFLICT");
}

#[tokio::test]
async fn create_direct_circle_reverse_also_duplicate() {
    let app = TestApp::new().await;
    let (alice_token, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;
    let alice_id: String = {
        let id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(ALICE_EMAIL)
            .fetch_one(&app.db)
            .await
            .unwrap();
        id.to_string()
    };

    // Alice → Bob
    app.post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;

    // Bob → Alice should also fail (reverse direction)
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
    let (alice_token, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (_, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let (_, circle) = app
        .post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;
    let circle_id = circle["id"].as_str().unwrap();

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
    let (alice_token, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob_token, bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    // Get Bob's actual username from DB
    let bob_username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1::uuid")
        .bind(&bob_id)
        .fetch_one(&app.db)
        .await
        .unwrap();

    app.post_with_auth(&format!("/circles/direct/{bob_id}"), &alice_token)
        .await;

    // Alice's list: direct circle shows Bob's username as name
    let (status, circles) = app.get_with_auth("/circles", &alice_token).await;
    assert_eq!(status, StatusCode::OK);

    let arr = circles.as_array().unwrap();
    let direct = arr.iter().find(|c| c["is_direct"] == true).unwrap();
    assert_eq!(direct["name"].as_str().unwrap(), bob_username);

    // Bob's list: direct circle shows Alice's username as name
    let alice_username: String = sqlx::query_scalar("SELECT username FROM users WHERE email = $1")
        .bind(ALICE_EMAIL)
        .fetch_one(&app.db)
        .await
        .unwrap();

    let (_, bob_circles) = app.get_with_auth("/circles", &bob_token).await;
    let bob_arr = bob_circles.as_array().unwrap();
    let bob_direct = bob_arr.iter().find(|c| c["is_direct"] == true).unwrap();
    assert_eq!(bob_direct["name"].as_str().unwrap(), alice_username);
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
    let arr = resp.as_array().unwrap();
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
    let arr = listing.as_array().unwrap();
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
    let arr = items.as_array().unwrap();
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
    assert_eq!(items.as_array().unwrap().len(), 1);
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
    assert_eq!(items.as_array().unwrap().len(), 0);
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
    assert_eq!(items.as_array().unwrap().len(), 0);
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
    let arr = items.as_array().unwrap();
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
    let arr = items.as_array().unwrap();
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
    let arr = items.as_array().unwrap();
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
    let arr1 = items1.as_array().unwrap();
    let arr2 = items2.as_array().unwrap();
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
        items1_after.as_array().unwrap().len(),
        0,
        "unshared from circle1"
    );
    assert_eq!(
        items2_after.as_array().unwrap().len(),
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
        .put_json_with_auth(
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
    app.put_json_with_auth(
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
    let circle_items = items.as_array().unwrap();
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
        .put_json_with_auth(
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
        items_before
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
        !items_after
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
        .put_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Second mark — should 409
    let (status, resp) = app
        .put_json_with_auth(
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
        .put_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Unarchive back to active
    let (status, resp) = app
        .put_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "active" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["status"], "active");
}
