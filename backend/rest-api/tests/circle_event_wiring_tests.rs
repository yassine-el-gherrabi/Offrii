mod common;

use axum::http::StatusCode;
use common::{TEST_PASSWORD, TestApp};

// ── Helpers ─────────────────────────────────────────────────────────

const ALICE_EMAIL: &str = "alice-cew@example.com";
const BOB_EMAIL: &str = "bob-cew@example.com";
const CHARLIE_EMAIL: &str = "charlie-cew@example.com";

/// Register a user and return (access_token, user_id).
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

/// Create an invite for a circle and return the invite token.
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

/// Invite a user to a circle and have them join.
async fn invite_and_join(app: &TestApp, circle_id: &str, owner_token: &str, joiner_token: &str) {
    let invite_tok = create_invite_token(app, circle_id, owner_token).await;
    let (status, _) = app
        .post_with_auth(&format!("/circles/join/{invite_tok}"), joiner_token)
        .await;
    assert_eq!(status, StatusCode::OK, "join precondition failed");
}

/// Share an item into a circle.
async fn share_item(app: &TestApp, circle_id: &str, item_id: &str, token: &str) {
    let body = serde_json::json!({ "item_id": item_id });
    let (status, resp) = app
        .post_json_with_auth(&format!("/circles/{circle_id}/items"), &body, token)
        .await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "share_item precondition: {resp}"
    );
}

/// Get feed events for a circle.
async fn get_feed(app: &TestApp, circle_id: &str, token: &str) -> serde_json::Value {
    let (status, feed) = app
        .get_with_auth(&format!("/circles/{circle_id}/feed"), token)
        .await;
    assert_eq!(status, StatusCode::OK, "get_feed failed");
    feed
}

/// Collect event types from a feed response.
fn event_types(feed: &serde_json::Value) -> Vec<String> {
    feed["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["event_type"].as_str().unwrap().to_string())
        .collect()
}

// ═══════════════════════════════════════════════════════════════════
// Circle activity feed event wiring tests
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn claim_item_creates_circle_event() {
    let app = TestApp::new().await;
    let (alice, _alice_id) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _bob_id) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "ClaimCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice creates and shares an item
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "ClaimMe" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Bob claims the item
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob should see the item_claimed event in the feed
    let feed = get_feed(&app, &circle_id, &bob).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_claimed".to_string()),
        "feed should contain item_claimed event, got: {types:?}"
    );
}

#[tokio::test]
async fn unclaim_item_creates_circle_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "UnclaimCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "UnclaimMe" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Bob claims then unclaims
    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    let (status, _) = app
        .delete_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let feed = get_feed(&app, &circle_id, &bob).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_unclaimed".to_string()),
        "feed should contain item_unclaimed event, got: {types:?}"
    );
}

#[tokio::test]
async fn mark_received_creates_circle_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "ReceivedCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "ReceiveMe" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Bob claims, then Alice marks as received
    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    let (status, _) = app
        .patch_json_with_auth(
            &format!("/items/{item_id}"),
            &serde_json::json!({ "status": "purchased" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Bob should see the item_received event
    let feed = get_feed(&app, &circle_id, &bob).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_received".to_string()),
        "feed should contain item_received event, got: {types:?}"
    );
}

#[tokio::test]
async fn circle_feed_hides_claims_from_item_owner() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "AntiSpoiler").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Surprise" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Bob claims the item
    app.post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;

    // Alice (item owner) should NOT see item_claimed in her feed
    let feed = get_feed(&app, &circle_id, &alice).await;
    let types = event_types(&feed);
    assert!(
        !types.contains(&"item_claimed".to_string()),
        "item owner should NOT see item_claimed events (anti-spoiler), got: {types:?}"
    );

    // Bob (claimer / third party) SHOULD see it
    let feed = get_feed(&app, &circle_id, &bob).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_claimed".to_string()),
        "claimer should see item_claimed event, got: {types:?}"
    );
}

#[tokio::test]
async fn unshared_item_claim_no_circle_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;
    let (charlie, _) = setup_user_with_id(&app, CHARLIE_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "NoEventCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;
    invite_and_join(&app, &circle_id, &alice, &charlie).await;

    // Alice creates an item but does NOT share it to the circle
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "Private" }))
        .await;
    let item_id = item["id"].as_str().unwrap();

    // Bob claims the unshared item (via direct link / share link - we use app claim endpoint)
    let (status, _) = app
        .post_with_auth(&format!("/items/{item_id}/claim"), &bob)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Feed should only have member_joined events, no item_claimed
    let feed = get_feed(&app, &circle_id, &charlie).await;
    let types = event_types(&feed);
    assert!(
        !types.contains(&"item_claimed".to_string()),
        "claiming an unshared item should not produce circle events, got: {types:?}"
    );
}

#[tokio::test]
async fn owner_unclaim_web_creates_circle_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "WebUnclaimCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;

    // Alice creates an item and shares it
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "WebItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Create a share link for Alice's list
    let (status, link_resp) = app
        .post_json_with_auth(
            "/share-links",
            &serde_json::json!({ "permissions": "view_and_claim" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "share link: {link_resp}");
    let share_token = link_resp["token"].as_str().unwrap();

    // Anonymous web claim
    let (status, claim_resp) = app
        .post_json(
            &format!("/shared/{share_token}/items/{item_id}/web-claim"),
            &serde_json::json!({ "name": "Grandma" }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "web claim: {claim_resp}");

    // Alice removes the web claim
    let (status, _) = app
        .delete_with_auth(&format!("/items/{item_id}/web-claim"), &alice)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Bob should see item_unclaimed in the feed
    let feed = get_feed(&app, &circle_id, &bob).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_unclaimed".to_string()),
        "owner unclaiming a web claim should produce item_unclaimed event, got: {types:?}"
    );
}

#[tokio::test]
async fn claim_via_share_link_creates_circle_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;
    let (charlie, _) = setup_user_with_id(&app, CHARLIE_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "ShareClaimCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;
    invite_and_join(&app, &circle_id, &alice, &charlie).await;

    // Alice creates an item and shares it to the circle
    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "ShareClaimItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Create a share link
    let (status, link_resp) = app
        .post_json_with_auth(
            "/share-links",
            &serde_json::json!({ "permissions": "view_and_claim" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let share_token = link_resp["token"].as_str().unwrap();

    // Bob claims via the share link (authenticated claim)
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Charlie should see item_claimed in the feed
    let feed = get_feed(&app, &circle_id, &charlie).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_claimed".to_string()),
        "claim via share link should produce item_claimed event, got: {types:?}"
    );
}

#[tokio::test]
async fn unclaim_via_share_link_creates_circle_event() {
    let app = TestApp::new().await;
    let (alice, _) = setup_user_with_id(&app, ALICE_EMAIL).await;
    let (bob, _) = setup_user_with_id(&app, BOB_EMAIL).await;
    let (charlie, _) = setup_user_with_id(&app, CHARLIE_EMAIL).await;

    let circle_id = create_circle(&app, &alice, "ShareUnclaimCircle").await;
    invite_and_join(&app, &circle_id, &alice, &bob).await;
    invite_and_join(&app, &circle_id, &alice, &charlie).await;

    let item = app
        .create_item(&alice, &serde_json::json!({ "name": "ShareUnclaimItem" }))
        .await;
    let item_id = item["id"].as_str().unwrap();
    share_item(&app, &circle_id, item_id, &alice).await;

    // Create a share link
    let (status, link_resp) = app
        .post_json_with_auth(
            "/share-links",
            &serde_json::json!({ "permissions": "view_and_claim" }),
            &alice,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let share_token = link_resp["token"].as_str().unwrap();

    // Bob claims via share link
    app.post_with_auth(
        &format!("/shared/{share_token}/items/{item_id}/claim"),
        &bob,
    )
    .await;

    // Bob unclaims via share link
    let (status, _) = app
        .delete_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Charlie should see item_unclaimed in the feed
    let feed = get_feed(&app, &circle_id, &charlie).await;
    let types = event_types(&feed);
    assert!(
        types.contains(&"item_unclaimed".to_string()),
        "unclaim via share link should produce item_unclaimed event, got: {types:?}"
    );
}
