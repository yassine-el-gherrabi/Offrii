mod common;

use axum::http::StatusCode;

use common::{TEST_PASSWORD, TestApp};

#[tokio::test]
async fn create_share_link_returns_201() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (status, body) = app.post_with_auth("/share-links", &token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert!(body["url"].is_string());
    assert!(body["id"].is_string());
    let url = body["url"].as_str().unwrap();
    assert!(url.contains("/shared/"));
}

#[tokio::test]
async fn list_share_links_returns_created_links() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create two links
    app.post_with_auth("/share-links", &token).await;
    app.post_with_auth("/share-links", &token).await;

    let (status, body) = app.get_with_auth("/share-links", &token).await;

    assert_eq!(status, StatusCode::OK);
    let links = body.as_array().unwrap();
    assert_eq!(links.len(), 2);
    assert!(links[0]["token"].is_string());
    assert!(links[0]["id"].is_string());
}

#[tokio::test]
async fn delete_share_link_returns_204() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let (_, body) = app.post_with_auth("/share-links", &token).await;
    let link_id = body["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/share-links/{link_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify it's gone
    let (_, body) = app.get_with_auth("/share-links", &token).await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn delete_nonexistent_share_link_returns_404() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, body) = app
        .delete_with_auth(&format!("/share-links/{fake_id}"), &token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn cannot_delete_another_users_share_link() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    let (_, body) = app.post_with_auth("/share-links", &alice_token).await;
    let link_id = body["id"].as_str().unwrap();

    let (status, _) = app
        .delete_with_auth(&format!("/share-links/{link_id}"), &bob_token)
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_shared_view_returns_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create an item
    let item_body = serde_json::json!({ "name": "Nintendo Switch" });
    app.create_item(&token, &item_body).await;

    // Create share link
    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Access shared view (public endpoint, no auth needed)
    let (status, body) = app.get_no_auth(&format!("/shared/{share_token}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["items"].is_array());
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Nintendo Switch");
}

#[tokio::test]
async fn get_shared_view_invalid_token_returns_404() {
    let app = TestApp::new().await;

    let (status, body) = app.get_no_auth("/shared/nonexistent_token").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    common::assert_error(&body, "NOT_FOUND");
}

#[tokio::test]
async fn claim_via_share_link() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    // Alice creates an item
    let item_body = serde_json::json!({ "name": "PS5" });
    let item = app.create_item(&alice_token, &item_body).await;
    let item_id = item["id"].as_str().unwrap();

    // Alice creates a share link
    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Bob claims the item via share link
    let (status, _) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify item is claimed in the shared view
    let (_, body) = app.get_no_auth(&format!("/shared/{share_token}")).await;
    let items = body["items"].as_array().unwrap();
    assert!(items[0]["is_claimed"].as_bool().unwrap());
}

#[tokio::test]
async fn unclaim_via_share_link() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;
    let bob_token = app.setup_user_token("bob@test.com", TEST_PASSWORD).await;

    // Alice creates an item
    let item_body = serde_json::json!({ "name": "PS5" });
    let item = app.create_item(&alice_token, &item_body).await;
    let item_id = item["id"].as_str().unwrap();

    // Alice creates a share link
    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    // Bob claims then unclaims
    app.post_with_auth(
        &format!("/shared/{share_token}/items/{item_id}/claim"),
        &bob_token,
    )
    .await;

    let (status, _) = app
        .delete_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &bob_token,
        )
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn cannot_claim_own_item_via_share() {
    let app = TestApp::new().await;
    let alice_token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    let item_body = serde_json::json!({ "name": "PS5" });
    let item = app.create_item(&alice_token, &item_body).await;
    let item_id = item["id"].as_str().unwrap();

    let (_, link_body) = app.post_with_auth("/share-links", &alice_token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, body) = app
        .post_with_auth(
            &format!("/shared/{share_token}/items/{item_id}/claim"),
            &alice_token,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    common::assert_error(&body, "BAD_REQUEST");
}

#[tokio::test]
async fn share_link_requires_auth_for_protected_endpoints() {
    let app = TestApp::new().await;

    // POST /share-links without auth
    let (status, _) = app.post_empty("/share-links").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // GET /share-links without auth
    let (status, _) = app.get_no_auth("/share-links").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn shared_view_does_not_show_deleted_items() {
    let app = TestApp::new().await;
    let token = app.setup_user_token("alice@test.com", TEST_PASSWORD).await;

    // Create two items
    let item1 = app
        .create_item(&token, &serde_json::json!({ "name": "Item 1" }))
        .await;
    app.create_item(&token, &serde_json::json!({ "name": "Item 2" }))
        .await;

    // Delete first item
    let item1_id = item1["id"].as_str().unwrap();
    app.delete_with_auth(&format!("/items/{item1_id}"), &token)
        .await;

    // Create share link and view
    let (_, link_body) = app.post_with_auth("/share-links", &token).await;
    let share_token = link_body["token"].as_str().unwrap();

    let (status, body) = app.get_no_auth(&format!("/shared/{share_token}")).await;

    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Item 2");
}
